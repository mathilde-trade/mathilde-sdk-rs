use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use chrono::{SecondsFormat, Utc};
use futures_util::{SinkExt, StreamExt};
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::core::config::{
    AggregatorConfig, GrpcTransportConfig, HttpTransportConfig, WsTransportConfig,
};
use mathilde_sdk_rs::core::error::SdkError;
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::streaming::subscription::ExponentialBackoffConfig;
use mathilde_sdk_rs::systems::aggregator::{
    Aggregator, BarsWsFormat, BarsWsInboundFrame, BarsWsSubscribeRequest, FilesDownloadsRequest,
    LatestBarsGrpcRequest, LatestBarsRequest, LatestBarsResponse, MessagesWsServerFrame,
    MessagesWsSubscribeFrame, PairsListRequest, PairsStatusRequest, RangeBarsGrpcRequest,
    RangeBarsRequest, RangeBarsResponse, SearchBarsGrpcRequest, SearchBarsRequest,
    SearchBarsResponse, TimeMachineBarsGrpcRequest, TimeMachineBarsRequest,
    TimeMachineBarsResponse,
};
use mathilde_sdk_rs::systems::primitives::{
    DocsRegistryRequest as PrimitivesDocsRegistryRequest, Primitives,
};
use mathilde_sdk_rs::systems::types::{AlignMode, HttpFormat, LatestMode, Timeframe};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, timeout};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, accept_async, connect_async};

const BIN_NAME: &str = "sdk_live_public_surface_check";
const HTTP_ENV: &str = "AGGREGATOR_FEED_HTTP_BASE_URL";
const GRPC_ENV: &str = "AGGREGATOR_FEED_GRPC_BASE_URL";
const WS_ENV: &str = "AGGREGATOR_FEED_WS_BASE_URL";
const BEARER_ENV: &str = "AGGREGATOR_FEED_BEARER_TOKEN";
const RAW_BARS_WS_WINDOW_SECS: u64 = 120;
const RAW_MESSAGES_WS_WINDOW_SECS: u64 = 120;
const RECOVERING_BARS_WS_WINDOW_SECS: u64 = 30;
const RECOVERING_MESSAGES_WS_WINDOW_SECS: u64 = 120;
const WS_PHASE_MINIMUM_SECS: u64 = 300;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum CheckStatus {
    NotRun,
    Passed,
    Failed,
    Skipped,
}

impl CheckStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::NotRun => "not_run",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone)]
struct SurfaceResult {
    family: &'static str,
    surface: &'static str,
    status: CheckStatus,
    note: String,
}

#[derive(Debug, Clone)]
struct RuntimeConfigSummary {
    http_base_url: String,
    grpc_base_url: Option<String>,
    ws_base_url: Option<String>,
    bearer_token_present: bool,
}

#[derive(Debug)]
struct Report {
    title: String,
    execution_timestamp_utc: String,
    config: Option<RuntimeConfigSummary>,
    results: Vec<SurfaceResult>,
    proved_observations: Vec<String>,
    failures: Vec<String>,
    skipped: Vec<String>,
    final_status: String,
}

#[derive(Debug)]
struct RuntimeConfig {
    summary: RuntimeConfigSummary,
    client: Aggregator,
    primitives_client: Primitives,
}

#[derive(Debug, Clone)]
struct WsRecoveryProxy {
    local_base_url: String,
    connection_count: Arc<AtomicUsize>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn output_dir() -> PathBuf {
    repo_root().join("bin").join(BIN_NAME)
}

fn report_path(timestamp: &str) -> PathBuf {
    output_dir().join(format!("{BIN_NAME}_{timestamp}.md"))
}

fn timestamp_for_filename() -> String {
    Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

fn timestamp_for_report() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn parse_dotenv_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let (key, value) = trimmed.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }

    let value = value.trim().trim_matches('"').trim_matches('\'');
    Some((key.to_string(), value.to_string()))
}

fn load_dotenv_if_present(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Ok(());
    }

    let content = fs::read_to_string(path)
        .map_err(|source| format!("failed to read dotenv file {}: {source}", path.display()))?;

    for line in content.lines() {
        if let Some((key, value)) = parse_dotenv_line(line) {
            if env::var_os(&key).is_none() {
                // SAFETY: this binary is single-process and env mutation is limited to startup.
                unsafe { env::set_var(key, value) };
            }
        }
    }

    Ok(())
}

fn required_env(name: &'static str) -> Result<String, SdkError> {
    env::var(name)
        .map(|value| value.trim().to_string())
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            SdkError::request_build(format!("missing required environment variable `{name}`"))
        })
}

fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_ws_base_url(base_url: &str) -> Result<String, SdkError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.starts_with("ws://") || trimmed.starts_with("wss://") {
        return Ok(trimmed.to_string());
    }
    if let Some(rest) = trimmed.strip_prefix("http://") {
        return Ok(format!("ws://{rest}"));
    }
    if let Some(rest) = trimmed.strip_prefix("https://") {
        return Ok(format!("wss://{rest}"));
    }
    Err(SdkError::request_build(format!(
        "unsupported ws base url scheme `{base_url}`"
    )))
}

fn join_ws_url(base_url: &str, path: &str) -> Result<String, SdkError> {
    Ok(format!("{}{}", normalize_ws_base_url(base_url)?, path))
}

fn build_client_with_ws_override(
    runtime: &RuntimeConfig,
    ws_base_url: &str,
) -> Result<Aggregator, SdkError> {
    let bearer_token = optional_env(BEARER_ENV).map(BearerToken::new).transpose()?;

    let config = AggregatorConfig {
        http: HttpTransportConfig::new(&runtime.summary.http_base_url)?,
        grpc: runtime
            .summary
            .grpc_base_url
            .as_deref()
            .map(GrpcTransportConfig::new)
            .transpose()?,
        ws: Some(WsTransportConfig::new(ws_base_url)?),
        bearer_token,
    };

    Aggregator::new(config)
}

fn is_bars_payload_message(message: &Message) -> bool {
    match message {
        Message::Binary(bytes) => !bytes.is_empty(),
        Message::Text(text) => {
            let trimmed = text.trim_start();
            trimmed.starts_with('[')
        }
        _ => false,
    }
}

fn is_messages_message_frame(message: &Message) -> bool {
    match message {
        Message::Text(text) => {
            serde_json::from_str::<serde_json::Value>(text)
                .ok()
                .and_then(|value| {
                    value
                        .get("type")
                        .and_then(|kind| kind.as_str().map(str::to_string))
                })
                .as_deref()
                == Some("message")
        }
        _ => false,
    }
}

async fn connect_upstream_ws(
    base_url: &str,
    path: &str,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, SdkError> {
    let mut request = join_ws_url(base_url, path)?
        .into_client_request()
        .map_err(|source| {
            SdkError::ws_transport(format!("proxy upstream request build failed: {source}"))
        })?;

    if let Some(token) = optional_env(BEARER_ENV) {
        let header = format!("Bearer {token}");
        request.headers_mut().insert(
            "authorization",
            header.parse().map_err(|source| {
                SdkError::ws_transport(format!("proxy auth header build failed: {source}"))
            })?,
        );
    }

    let (stream, _) = connect_async(request).await.map_err(|source| {
        SdkError::ws_transport(format!("proxy upstream connect failed: {source}"))
    })?;
    Ok(stream)
}

async fn proxy_ws_connection(
    client_stream: TcpStream,
    upstream_base_url: String,
    path: &'static str,
    force_disconnect_on_first_match: bool,
    matcher: fn(&Message) -> bool,
) {
    let mut client_ws = match accept_async(client_stream).await {
        Ok(stream) => stream,
        Err(_) => return,
    };

    let mut upstream_ws = match connect_upstream_ws(&upstream_base_url, path).await {
        Ok(stream) => stream,
        Err(_) => {
            let _ = client_ws.close(None).await;
            return;
        }
    };

    loop {
        tokio::select! {
            upstream_message = upstream_ws.next() => {
                let Some(upstream_message) = upstream_message else {
                    let _ = client_ws.close(None).await;
                    break;
                };

                match upstream_message {
                    Ok(message) => {
                        if client_ws.send(message.clone()).await.is_err() {
                            break;
                        }
                        if force_disconnect_on_first_match && matcher(&message) {
                            let _ = client_ws.close(None).await;
                            let _ = upstream_ws.close(None).await;
                            break;
                        }
                    }
                    Err(_) => {
                        let _ = client_ws.close(None).await;
                        break;
                    }
                }
            }
            client_message = client_ws.next() => {
                let Some(client_message) = client_message else {
                    let _ = upstream_ws.close(None).await;
                    break;
                };

                match client_message {
                    Ok(message) => {
                        if upstream_ws.send(message).await.is_err() {
                            let _ = client_ws.close(None).await;
                            break;
                        }
                    }
                    Err(_) => {
                        let _ = upstream_ws.close(None).await;
                        break;
                    }
                }
            }
        }
    }
}

async fn spawn_recovery_proxy(
    upstream_base_url: &str,
    path: &'static str,
    matcher: fn(&Message) -> bool,
) -> Result<WsRecoveryProxy, SdkError> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|source| SdkError::ws_transport(format!("proxy bind failed: {source}")))?;
    let addr = listener
        .local_addr()
        .map_err(|source| SdkError::ws_transport(format!("proxy local_addr failed: {source}")))?;
    let connection_count = Arc::new(AtomicUsize::new(0));
    let count_for_task = connection_count.clone();
    let upstream_base_url = upstream_base_url.to_string();

    tokio::spawn(async move {
        let mut forced_disconnect_done = false;
        loop {
            let accepted = listener.accept().await;
            let Ok((client_stream, _)) = accepted else {
                break;
            };

            count_for_task.fetch_add(1, Ordering::SeqCst);
            proxy_ws_connection(
                client_stream,
                upstream_base_url.clone(),
                path,
                !forced_disconnect_done,
                matcher,
            )
            .await;
            forced_disconnect_done = true;
        }
    });

    Ok(WsRecoveryProxy {
        local_base_url: format!("http://{addr}"),
        connection_count,
    })
}

async fn observe_raw_bars_ws(client: &Aggregator, pair: &str) -> Result<String, SdkError> {
    let request = BarsWsSubscribeRequest {
        pairs: vec![pair.to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        from_close: None,
        last_n_bars: Some(2),
        format: Some(BarsWsFormat::Json),
    };

    let deadline = Instant::now() + Duration::from_secs(RAW_BARS_WS_WINDOW_SECS);
    let mut stream = client.connect_bars_ws(&request).await?;
    let mut meta_count = 0usize;
    let mut payload_frames = 0usize;
    let mut payload_rows = 0usize;
    let mut last_close_ms = 0i64;

    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let frame = match timeout(remaining, stream.next_frame(&request)).await {
            Ok(frame) => frame?,
            Err(_) => break,
        };

        match frame {
            Some(BarsWsInboundFrame::Meta(meta)) => {
                meta_count += 1;
                if let Some(close_ms) = meta.close_ms {
                    if close_ms > 0 {
                        last_close_ms = close_ms;
                    }
                }
            }
            Some(BarsWsInboundFrame::JsonRowsMin(rows)) => {
                payload_frames += 1;
                payload_rows += rows.len();
                if let Some(first) = rows.first() {
                    last_close_ms = first.close_ms;
                }
            }
            Some(BarsWsInboundFrame::JsonRowsFull(rows)) => {
                payload_frames += 1;
                payload_rows += rows.len();
                if let Some(first) = rows.first() {
                    last_close_ms = first.close_ms;
                }
            }
            Some(BarsWsInboundFrame::ProtobufRowsMin(rows)) => {
                payload_frames += 1;
                payload_rows += rows.len();
                if let Some(first) = rows.first() {
                    last_close_ms = first.close_ms;
                }
            }
            Some(BarsWsInboundFrame::ProtobufRowsFull(rows)) => {
                payload_frames += 1;
                payload_rows += rows.len();
                if let Some(first) = rows.first() {
                    last_close_ms = first.close_ms;
                }
            }
            Some(BarsWsInboundFrame::Error(err)) => {
                return Err(SdkError::contract_drift(format!(
                    "bars ws error frame: {} {}",
                    err.kind, err.error
                )));
            }
            None => {
                return Err(SdkError::ws_transport(
                    "bars ws closed during sustained observation window",
                ));
            }
        }
    }

    if meta_count == 0 || payload_frames == 0 || payload_rows == 0 || last_close_ms <= 0 {
        return Err(SdkError::ws_transport(format!(
            "raw bars ws observation was not meaningful: meta_count={meta_count} payload_frames={payload_frames} payload_rows={payload_rows} last_close_ms={last_close_ms}"
        )));
    }

    Ok(format!(
        "window_s={} meta_count={} payload_frames={} payload_rows={} last_close_ms={}",
        RAW_BARS_WS_WINDOW_SECS, meta_count, payload_frames, payload_rows, last_close_ms
    ))
}

async fn observe_raw_messages_ws(client: &Aggregator, pair: &str) -> Result<String, SdkError> {
    let deadline = Instant::now() + Duration::from_secs(RAW_MESSAGES_WS_WINDOW_SECS);
    let mut stream = client.connect_messages_ws().await?;
    let subscribe = MessagesWsSubscribeFrame {
        id: "sdk_live_rule_1".to_string(),
        tfs: Some(vec![Timeframe::M1]),
        predicate: format!("{pair}.close > 0"),
        message: "sdk live sustained rule".to_string(),
        payload: Some(serde_json::json!({"probe":"sdk_live_public_surface_check"})),
    };

    stream.send_subscribe(&subscribe).await?;

    let mut subscribed = false;
    let mut heartbeat_count = 0usize;
    let mut message_count = 0usize;
    let mut last_close_ms = 0i64;

    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let frame = match timeout(remaining, stream.next_frame()).await {
            Ok(frame) => frame?,
            Err(_) => break,
        };

        match frame {
            Some(MessagesWsServerFrame::Subscribed(frame)) => {
                if frame.id == subscribe.id {
                    subscribed = true;
                }
            }
            Some(MessagesWsServerFrame::Message(frame)) => {
                message_count += 1;
                last_close_ms = frame.close_ms;
            }
            Some(MessagesWsServerFrame::Heartbeat(_)) => heartbeat_count += 1,
            Some(MessagesWsServerFrame::Error(frame)) => {
                return Err(SdkError::contract_drift(format!(
                    "messages ws error frame: {} {}",
                    frame.kind, frame.error
                )));
            }
            None => {
                return Err(SdkError::ws_transport(
                    "messages ws closed during sustained observation window",
                ));
            }
        }
    }

    if !subscribed || message_count == 0 || last_close_ms <= 0 {
        return Err(SdkError::ws_transport(format!(
            "raw messages ws observation was not meaningful: subscribed={subscribed} message_count={message_count} last_close_ms={last_close_ms}"
        )));
    }

    Ok(format!(
        "window_s={} subscribed={} message_count={} heartbeat_count={} last_close_ms={}",
        RAW_MESSAGES_WS_WINDOW_SECS, subscribed, message_count, heartbeat_count, last_close_ms
    ))
}

async fn observe_recovering_bars_ws(
    runtime: &RuntimeConfig,
    upstream_ws_base_url: &str,
    pair: &str,
) -> Result<String, SdkError> {
    let proxy =
        spawn_recovery_proxy(upstream_ws_base_url, "/v1/ws/bars", is_bars_payload_message).await?;
    let client = build_client_with_ws_override(runtime, &proxy.local_base_url)?;
    let request = BarsWsSubscribeRequest {
        pairs: vec![pair.to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        from_close: None,
        last_n_bars: Some(1),
        format: Some(BarsWsFormat::Json),
    };
    let mut stream = client
        .connect_bars_ws_recovering(
            &request,
            ExponentialBackoffConfig {
                initial_delay: Duration::from_millis(250),
                multiplier: 2,
                max_delay: Duration::from_secs(5),
                max_attempts: None,
                jitter_ratio: 0.0,
            },
        )
        .await?;
    let deadline = Instant::now() + Duration::from_secs(RECOVERING_BARS_WS_WINDOW_SECS);
    let mut pre_reconnect_frames = 0usize;
    let mut post_reconnect_frames = 0usize;
    let mut last_close_ms = 0i64;

    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let frame = match timeout(remaining, stream.next_frame()).await {
            Ok(frame) => frame?,
            Err(_) => break,
        };

        let reconnected = proxy.connection_count.load(Ordering::SeqCst) >= 2;
        match frame {
            Some(BarsWsInboundFrame::Meta(meta)) => {
                if let Some(close_ms) = meta.close_ms {
                    if close_ms > 0 {
                        last_close_ms = close_ms;
                        if reconnected {
                            post_reconnect_frames += 1;
                        } else {
                            pre_reconnect_frames += 1;
                        }
                    }
                }
            }
            Some(BarsWsInboundFrame::JsonRowsMin(rows)) => {
                if let Some(first) = rows.first() {
                    last_close_ms = first.close_ms;
                }
                if reconnected {
                    post_reconnect_frames += rows.len().max(1);
                } else {
                    pre_reconnect_frames += rows.len().max(1);
                }
            }
            Some(BarsWsInboundFrame::JsonRowsFull(rows)) => {
                if let Some(first) = rows.first() {
                    last_close_ms = first.close_ms;
                }
                if reconnected {
                    post_reconnect_frames += rows.len().max(1);
                } else {
                    pre_reconnect_frames += rows.len().max(1);
                }
            }
            Some(BarsWsInboundFrame::ProtobufRowsMin(rows)) => {
                if let Some(first) = rows.first() {
                    last_close_ms = first.close_ms;
                }
                if reconnected {
                    post_reconnect_frames += rows.len().max(1);
                } else {
                    pre_reconnect_frames += rows.len().max(1);
                }
            }
            Some(BarsWsInboundFrame::ProtobufRowsFull(rows)) => {
                if let Some(first) = rows.first() {
                    last_close_ms = first.close_ms;
                }
                if reconnected {
                    post_reconnect_frames += rows.len().max(1);
                } else {
                    pre_reconnect_frames += rows.len().max(1);
                }
            }
            Some(BarsWsInboundFrame::Error(err)) => {
                return Err(SdkError::contract_drift(format!(
                    "recovering bars ws error frame: {} {}",
                    err.kind, err.error
                )));
            }
            None => {
                return Err(SdkError::ws_transport(
                    "recovering bars ws returned None during sustained observation window",
                ));
            }
        }
    }

    let reconnect_count = proxy.connection_count.load(Ordering::SeqCst);
    if reconnect_count < 2
        || pre_reconnect_frames == 0
        || post_reconnect_frames == 0
        || last_close_ms <= 0
    {
        return Err(SdkError::ws_transport(format!(
            "recovering bars ws observation was not meaningful: reconnect_count={reconnect_count} pre_reconnect_frames={pre_reconnect_frames} post_reconnect_frames={post_reconnect_frames} last_close_ms={last_close_ms}"
        )));
    }

    Ok(format!(
        "window_s={} reconnect_count={} pre_reconnect_frames={} post_reconnect_frames={} last_close_ms={}",
        RECOVERING_BARS_WS_WINDOW_SECS,
        reconnect_count,
        pre_reconnect_frames,
        post_reconnect_frames,
        last_close_ms
    ))
}

async fn observe_recovering_messages_ws(
    runtime: &RuntimeConfig,
    upstream_ws_base_url: &str,
    pair: &str,
) -> Result<String, SdkError> {
    let proxy = spawn_recovery_proxy(
        upstream_ws_base_url,
        "/v1/ws/messages",
        is_messages_message_frame,
    )
    .await?;
    let client = build_client_with_ws_override(runtime, &proxy.local_base_url)?;
    let mut stream = client
        .connect_messages_ws_recovering(ExponentialBackoffConfig {
            initial_delay: Duration::from_millis(250),
            multiplier: 2,
            max_delay: Duration::from_secs(5),
            max_attempts: None,
            jitter_ratio: 0.0,
        })
        .await?;
    let subscribe = MessagesWsSubscribeFrame {
        id: "sdk_live_recovering_rule_1".to_string(),
        tfs: Some(vec![Timeframe::M1]),
        predicate: format!("{pair}.close > 0"),
        message: "sdk recovering rule".to_string(),
        payload: Some(serde_json::json!({"probe":"sdk_live_public_surface_check_recovering"})),
    };
    stream.send_subscribe(&subscribe).await?;

    let deadline = Instant::now() + Duration::from_secs(RECOVERING_MESSAGES_WS_WINDOW_SECS);
    let mut subscribed_before_reconnect = false;
    let mut subscribed_after_reconnect = false;
    let mut pre_reconnect_messages = 0usize;
    let mut post_reconnect_messages = 0usize;
    let mut last_close_ms = 0i64;

    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let frame = match timeout(remaining, stream.next_frame()).await {
            Ok(frame) => frame?,
            Err(_) => break,
        };

        let reconnected = proxy.connection_count.load(Ordering::SeqCst) >= 2;
        match frame {
            Some(MessagesWsServerFrame::Subscribed(frame)) => {
                if frame.id == subscribe.id {
                    if reconnected {
                        subscribed_after_reconnect = true;
                    } else {
                        subscribed_before_reconnect = true;
                    }
                }
            }
            Some(MessagesWsServerFrame::Message(frame)) => {
                last_close_ms = frame.close_ms;
                if reconnected {
                    post_reconnect_messages += 1;
                } else {
                    pre_reconnect_messages += 1;
                }
            }
            Some(MessagesWsServerFrame::Heartbeat(_)) => {}
            Some(MessagesWsServerFrame::Error(frame)) => {
                return Err(SdkError::contract_drift(format!(
                    "recovering messages ws error frame: {} {}",
                    frame.kind, frame.error
                )));
            }
            None => {
                return Err(SdkError::ws_transport(
                    "recovering messages ws returned None during sustained observation window",
                ));
            }
        }
    }

    let reconnect_count = proxy.connection_count.load(Ordering::SeqCst);
    if reconnect_count < 2
        || !subscribed_before_reconnect
        || !subscribed_after_reconnect
        || pre_reconnect_messages == 0
        || post_reconnect_messages == 0
        || last_close_ms <= 0
    {
        return Err(SdkError::ws_transport(format!(
            "recovering messages ws observation was not meaningful: reconnect_count={reconnect_count} subscribed_before_reconnect={subscribed_before_reconnect} subscribed_after_reconnect={subscribed_after_reconnect} pre_reconnect_messages={pre_reconnect_messages} post_reconnect_messages={post_reconnect_messages} last_close_ms={last_close_ms}"
        )));
    }

    Ok(format!(
        "window_s={} reconnect_count={} subscribed_before_reconnect={} subscribed_after_reconnect={} pre_reconnect_messages={} post_reconnect_messages={} last_close_ms={}",
        RECOVERING_MESSAGES_WS_WINDOW_SECS,
        reconnect_count,
        subscribed_before_reconnect,
        subscribed_after_reconnect,
        pre_reconnect_messages,
        post_reconnect_messages,
        last_close_ms
    ))
}

fn initial_surface_results() -> Vec<SurfaceResult> {
    [
        ("docs", "docs_system"),
        ("docs", "docs_themes"),
        ("docs", "docs_endpoints"),
        ("docs", "openapi"),
        ("primitives_docs", "primitives.docs_system"),
        ("primitives_docs", "primitives.docs_summary"),
        ("primitives_docs", "primitives.docs_taxonomy"),
        ("primitives_docs", "primitives.docs_registry"),
        ("primitives_docs", "primitives.docs_endpoints"),
        ("primitives_docs", "primitives.openapi"),
        ("discovery", "pairs_status"),
        ("discovery", "pairs_list"),
        ("discovery", "files_downloads"),
        ("bars_http", "latest"),
        ("bars_http", "range"),
        ("bars_http", "search"),
        ("bars_http", "time_machine"),
        ("bars_grpc", "latest_grpc"),
        ("bars_grpc", "range_grpc"),
        ("bars_grpc", "search_grpc"),
        ("bars_grpc", "time_machine_grpc"),
        ("ws", "connect_bars_ws"),
        ("ws", "connect_messages_ws"),
        ("ws_optional", "connect_bars_ws_recovering"),
        ("ws_optional", "connect_messages_ws_recovering"),
    ]
    .into_iter()
    .map(|(family, surface)| SurfaceResult {
        family,
        surface,
        status: CheckStatus::NotRun,
        note: "not executed in Phase L-A foundation pass".to_string(),
    })
    .collect()
}

fn markdown_report(report: &Report) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", report.title));
    out.push_str(&format!(
        "- execution_timestamp_utc: `{}`\n",
        report.execution_timestamp_utc
    ));
    out.push_str(&format!("- final_status: `{}`\n\n", report.final_status));

    out.push_str("## Configuration Summary\n\n");
    match &report.config {
        Some(config) => {
            out.push_str(&format!("- http_base_url: `{}`\n", config.http_base_url));
            out.push_str(&format!(
                "- grpc_base_url: `{}`\n",
                config.grpc_base_url.as_deref().unwrap_or("not_configured")
            ));
            out.push_str(&format!(
                "- ws_base_url: `{}`\n",
                config.ws_base_url.as_deref().unwrap_or("not_configured")
            ));
            out.push_str(&format!(
                "- bearer_token_present: `{}`\n\n",
                config.bearer_token_present
            ));
        }
        None => out.push_str("- configuration was not established\n\n"),
    }

    out.push_str("## Surface Results\n\n");
    out.push_str("| Family | Surface | Status | Note |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    for result in &report.results {
        let note = result.note.replace('\n', " ");
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} |\n",
            result.family,
            result.surface,
            result.status.as_str(),
            note
        ));
    }
    out.push('\n');

    out.push_str("## Proved Observations\n\n");
    if report.proved_observations.is_empty() {
        out.push_str("- none yet\n");
    } else {
        for item in &report.proved_observations {
            out.push_str(&format!("- {item}\n"));
        }
    }
    out.push('\n');

    out.push_str("## Failures\n\n");
    if report.failures.is_empty() {
        out.push_str("- none\n");
    } else {
        for item in &report.failures {
            out.push_str(&format!("- {item}\n"));
        }
    }
    out.push('\n');

    out.push_str("## Skipped Checks\n\n");
    if report.skipped.is_empty() {
        out.push_str("- none\n");
    } else {
        for item in &report.skipped {
            out.push_str(&format!("- {item}\n"));
        }
    }
    out.push('\n');

    out.push_str("## Final Status\n\n");
    out.push_str(&format!("`{}`\n", report.final_status));
    out
}

fn write_report(report: &Report, timestamp: &str) -> Result<PathBuf, String> {
    let dir = output_dir();
    fs::create_dir_all(&dir).map_err(|source| {
        format!(
            "failed to create report directory {}: {source}",
            dir.display()
        )
    })?;

    let path = report_path(timestamp);
    fs::write(&path, markdown_report(report))
        .map_err(|source| format!("failed to write report {}: {source}", path.display()))?;
    Ok(path)
}

fn set_surface_status(
    report: &mut Report,
    surface: &'static str,
    status: CheckStatus,
    note: impl Into<String>,
) {
    let note = note.into();
    if let Some(row) = report.results.iter_mut().find(|row| row.surface == surface) {
        row.status = status;
        row.note = note;
    } else {
        report.results.push(SurfaceResult {
            family: "unknown",
            surface,
            status,
            note,
        });
    }
}

fn record_pass(report: &mut Report, surface: &'static str, note: impl Into<String>) {
    let note = note.into();
    set_surface_status(report, surface, CheckStatus::Passed, note.clone());
    report
        .proved_observations
        .push(format!("`{surface}` passed: {note}"));
}

fn record_fail(report: &mut Report, surface: &'static str, error: impl Into<String>) {
    let error = error.into();
    set_surface_status(report, surface, CheckStatus::Failed, error.clone());
    report.failures.push(format!("`{surface}` failed: {error}"));
}

fn build_runtime_config() -> Result<RuntimeConfig, SdkError> {
    let http_base_url = required_env(HTTP_ENV)?;
    let grpc_base_url = optional_env(GRPC_ENV);
    let ws_base_url = optional_env(WS_ENV);
    let bearer_token = optional_env(BEARER_ENV).map(BearerToken::new).transpose()?;
    let bearer_token_present = bearer_token.is_some();

    let config = AggregatorConfig {
        http: HttpTransportConfig::new(&http_base_url)?,
        grpc: grpc_base_url
            .as_ref()
            .map(GrpcTransportConfig::new)
            .transpose()?,
        ws: ws_base_url
            .as_ref()
            .map(WsTransportConfig::new)
            .transpose()?,
        bearer_token: bearer_token.clone(),
    };

    let client = Aggregator::new(config)?;
    let primitives_client = Primitives::client(bearer_token.clone())?;
    Ok(RuntimeConfig {
        summary: RuntimeConfigSummary {
            http_base_url,
            grpc_base_url,
            ws_base_url,
            bearer_token_present,
        },
        client,
        primitives_client,
    })
}

fn pair_from_latest_response(out: &LatestBarsResponse) -> Option<&str> {
    match out {
        LatestBarsResponse::Min(out) => out.rows.first().map(|row| row.bar.pair.as_str()),
        LatestBarsResponse::Full(out) => out.rows.first().map(|row| row.bar.pair.as_str()),
    }
}

fn close_end_from_latest_response(out: &LatestBarsResponse) -> i64 {
    match out {
        LatestBarsResponse::Min(out) => out.close_end_ms,
        LatestBarsResponse::Full(out) => out.close_end_ms,
    }
}

fn range_rows_len(out: &RangeBarsResponse) -> usize {
    match out {
        RangeBarsResponse::Min(out) => out.rows.len(),
        RangeBarsResponse::Full(out) => out.rows.len(),
    }
}

fn search_hits_len(out: &SearchBarsResponse) -> usize {
    match out {
        SearchBarsResponse::Min(out) => out.hits.len(),
        SearchBarsResponse::Full(out) => out.hits.len(),
    }
}

fn time_machine_rows_len(out: &TimeMachineBarsResponse) -> usize {
    match out {
        TimeMachineBarsResponse::Min(out) => out.rows.len(),
        TimeMachineBarsResponse::Full(out) => out.rows.len(),
    }
}

fn json_str<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    value.get(key)?.as_str()
}

fn json_array_len(value: &serde_json::Value, key: &str) -> Option<usize> {
    value.get(key)?.as_array().map(|rows| rows.len())
}

async fn run_live_checks(runtime: &RuntimeConfig, report: &mut Report) {
    let client = &runtime.client;
    let primitives = &runtime.primitives_client;

    match client.docs_system().await {
        Ok(out) if json_str(&out, "intro").is_some_and(|intro| !intro.trim().is_empty()) => {
            record_pass(
                report,
                "docs_system",
                format!(
                    "subsystem={} sections={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    json_array_len(&out, "sections").unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(report, "docs_system", "empty docs content"),
        Err(error) => record_fail(report, "docs_system", error.to_string()),
    }

    match client.docs_summary().await {
        Ok(out) if json_str(&out, "intro").is_some_and(|intro| !intro.trim().is_empty()) => {
            record_pass(
                report,
                "docs_summary",
                format!(
                    "subsystem={} sections={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    json_array_len(&out, "sections").unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(report, "docs_summary", "empty summary content"),
        Err(error) => record_fail(report, "docs_summary", error.to_string()),
    }

    match client.docs_themes().await {
        Ok(out) if json_array_len(&out, "themes").is_some_and(|count| count > 0) => {
            record_pass(
                report,
                "docs_themes",
                format!(
                    "subsystem={} themes={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    json_array_len(&out, "themes").unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(report, "docs_themes", "empty themes content"),
        Err(error) => record_fail(report, "docs_themes", error.to_string()),
    }

    match primitives.docs_system().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "primitives.docs_system",
                format!(
                    "subsystem={} keys={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.docs_system",
            "docs_system was not a JSON object",
        ),
        Err(error) => record_fail(report, "primitives.docs_system", error.to_string()),
    }

    match primitives.docs_summary().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "primitives.docs_summary",
                format!(
                    "subsystem={} keys={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.docs_summary",
            "docs_summary was not a JSON object",
        ),
        Err(error) => record_fail(report, "primitives.docs_summary", error.to_string()),
    }

    match primitives.docs_taxonomy().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "primitives.docs_taxonomy",
                format!(
                    "keys={}",
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.docs_taxonomy",
            "docs_taxonomy was not a JSON object",
        ),
        Err(error) => record_fail(report, "primitives.docs_taxonomy", error.to_string()),
    }

    match primitives
        .docs_registry(&PrimitivesDocsRegistryRequest::default())
        .await
    {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "primitives.docs_registry",
                format!(
                    "keys={}",
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.docs_registry",
            "docs_registry was not a JSON object",
        ),
        Err(error) => record_fail(report, "primitives.docs_registry", error.to_string()),
    }

    match primitives.docs_endpoints().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "primitives.docs_endpoints",
                format!(
                    "keys={}",
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.docs_endpoints",
            "docs_endpoints was not a JSON object",
        ),
        Err(error) => record_fail(report, "primitives.docs_endpoints", error.to_string()),
    }

    match primitives.openapi().await {
        Ok(out) if out.get("openapi").is_some() => {
            record_pass(
                report,
                "primitives.openapi",
                format!(
                    "openapi={} paths={}",
                    json_str(&out, "openapi").unwrap_or(""),
                    out.get("paths")
                        .and_then(|value| value.as_object())
                        .map(|value| value.len())
                        .unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.openapi",
            "openapi document missing `openapi` key",
        ),
        Err(error) => record_fail(report, "primitives.openapi", error.to_string()),
    }

    match client.docs_endpoints().await {
        Ok(out) if json_str(&out, "intro").is_some_and(|intro| !intro.trim().is_empty()) => {
            record_pass(
                report,
                "docs_endpoints",
                format!(
                    "subsystem={} sections={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    json_array_len(&out, "sections").unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(report, "docs_endpoints", "empty endpoints content"),
        Err(error) => record_fail(report, "docs_endpoints", error.to_string()),
    }

    match client.openapi().await {
        Ok(out) if out.get("openapi").is_some() && out.get("paths").is_some() => {
            let path_count = out["paths"]
                .as_object()
                .map(|paths| paths.len())
                .unwrap_or(0);
            record_pass(report, "openapi", format!("paths={path_count}"));
        }
        Ok(_) => record_fail(report, "openapi", "missing `openapi` or `paths` keys"),
        Err(error) => record_fail(report, "openapi", error.to_string()),
    }

    let pairs_list_out = match client
        .pairs_list(&PairsListRequest {
            after_pair: None,
            limit: Some(5),
            enabled_only: Some(true),
        })
        .await
    {
        Ok(out) => {
            let len = out.pairs.len();
            record_pass(report, "pairs_list", format!("pairs={len}"));
            Some(out)
        }
        Err(error) => {
            record_fail(report, "pairs_list", error.to_string());
            None
        }
    };

    let target_pair = pairs_list_out
        .as_ref()
        .and_then(|out| out.pairs.first())
        .cloned()
        .unwrap_or_else(|| "BTCUSDT".to_string());

    match client
        .pairs_status(&PairsStatusRequest {
            after_pair: None,
            limit: Some(1),
            pairs: Some(vec![target_pair.clone()]),
            filters: Some(vec!["status".to_string(), "frontier".to_string()]),
        })
        .await
    {
        Ok(out) if !out.pairs.is_empty() => {
            record_pass(
                report,
                "pairs_status",
                format!("pair={} rows={}", out.pairs[0].pair, out.pairs.len()),
            );
        }
        Ok(_) => record_fail(report, "pairs_status", "empty pairs status response"),
        Err(error) => record_fail(report, "pairs_status", error.to_string()),
    }

    match client
        .files_downloads(&FilesDownloadsRequest {
            period: Some("day".to_string()),
            pairs: vec![target_pair.clone()],
            tfs: vec!["1m".to_string()],
            start_label_utc: None,
            end_label_utc: None,
            order: Some("desc".to_string()),
        })
        .await
    {
        Ok(out)
            if !out.rows.is_empty()
                && out.rows[0].url.starts_with("http")
                && !out.rows[0].expires_at_utc.trim().is_empty() =>
        {
            record_pass(
                report,
                "files_downloads",
                format!(
                    "rows={} first_pair={} expires_at_utc={}",
                    out.rows.len(),
                    out.rows[0].pair,
                    out.rows[0].expires_at_utc
                ),
            );
        }
        Ok(out) => record_fail(
            report,
            "files_downloads",
            format!("unexpected rows shape; rows={}", out.rows.len()),
        ),
        Err(error) => record_fail(report, "files_downloads", error.to_string()),
    }

    let latest_http_min_request = LatestBarsRequest {
        pairs: vec![target_pair.clone()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    };

    let latest_http_full_request = LatestBarsRequest {
        metadata: Some(true),
        ..latest_http_min_request.clone()
    };

    let latest_http_min = client.latest(&latest_http_min_request).await;
    let latest_http_full = client.latest(&latest_http_full_request).await;

    let anchor_close_ms = match (&latest_http_min, &latest_http_full) {
        (Ok(min), Ok(full))
            if pair_from_latest_response(min).is_some()
                && pair_from_latest_response(min) == pair_from_latest_response(full) =>
        {
            let close_end_ms = close_end_from_latest_response(min);
            record_pass(
                report,
                "latest",
                format!(
                    "pair={} close_end_ms={}",
                    pair_from_latest_response(min).unwrap_or("unknown"),
                    close_end_ms
                ),
            );
            close_end_ms
        }
        (Ok(_), Ok(_)) => {
            record_fail(report, "latest", "min/full latest parity mismatch");
            0
        }
        (Err(error), _) => {
            record_fail(report, "latest", error.to_string());
            0
        }
        (_, Err(error)) => {
            record_fail(report, "latest", error.to_string());
            0
        }
    };

    if anchor_close_ms <= 0 {
        record_fail(report, "range", "latest anchor was not established");
        record_fail(report, "search", "latest anchor was not established");
        record_fail(report, "time_machine", "latest anchor was not established");
    } else {
        let range_min_request = RangeBarsRequest {
            pairs: vec![target_pair.clone()],
            tf: Timeframe::M1,
            align_mode: Some(AlignMode::Exact),
            close_start: Some(TimeInput::Ms(anchor_close_ms - 10 * 60_000)),
            cursor: None,
            close_end: Some(TimeInput::Ms(anchor_close_ms)),
            limit: Some(5),
            metadata: Some(false),
            format: Some(HttpFormat::Json),
        };
        let range_full_request = RangeBarsRequest {
            metadata: Some(true),
            ..range_min_request.clone()
        };

        match (
            client.range(&range_min_request).await,
            client.range(&range_full_request).await,
        ) {
            (Ok(min), Ok(full))
                if range_rows_len(&min) > 0 && range_rows_len(&min) == range_rows_len(&full) =>
            {
                record_pass(
                    report,
                    "range",
                    format!(
                        "rows={} close_end_ms={anchor_close_ms}",
                        range_rows_len(&min)
                    ),
                );
            }
            (Ok(min), Ok(full)) => record_fail(
                report,
                "range",
                format!(
                    "unexpected min/full range rows: min={} full={}",
                    range_rows_len(&min),
                    range_rows_len(&full)
                ),
            ),
            (Err(error), _) => record_fail(report, "range", error.to_string()),
            (_, Err(error)) => record_fail(report, "range", error.to_string()),
        }

        let predicate = format!("{target_pair}.close > 0");
        let search_min_request = SearchBarsRequest {
            tf: Timeframe::M1,
            close_start: TimeInput::Ms(anchor_close_ms - 60 * 60_000),
            close_end: Some(TimeInput::Ms(anchor_close_ms)),
            cursor: None,
            predicate: predicate.clone(),
            evaluate_pair: Some(target_pair.clone()),
            metadata: Some(false),
            max_hits: Some(20),
            format: Some(HttpFormat::Json),
        };
        let search_full_request = SearchBarsRequest {
            metadata: Some(true),
            ..search_min_request.clone()
        };

        match (
            client.search(&search_min_request).await,
            client.search(&search_full_request).await,
        ) {
            (Ok(min), Ok(full))
                if search_hits_len(&min) > 0 && search_hits_len(&min) == search_hits_len(&full) =>
            {
                record_pass(
                    report,
                    "search",
                    format!("hits={} predicate={predicate}", search_hits_len(&min)),
                );
            }
            (Ok(min), Ok(full)) => record_fail(
                report,
                "search",
                format!(
                    "unexpected min/full search hits: min={} full={}",
                    search_hits_len(&min),
                    search_hits_len(&full)
                ),
            ),
            (Err(error), _) => record_fail(report, "search", error.to_string()),
            (_, Err(error)) => record_fail(report, "search", error.to_string()),
        }

        let time_machine_min_request = TimeMachineBarsRequest {
            tf: Timeframe::M1,
            close_start: TimeInput::Ms(anchor_close_ms - 20 * 60_000),
            close_end: Some(TimeInput::Ms(anchor_close_ms)),
            cursor: None,
            predicate: None,
            hits: Some(vec![anchor_close_ms - 120_000, anchor_close_ms - 60_000]),
            output_pairs: Some(vec![target_pair.clone()]),
            metadata: Some(false),
            before_bars: Some(2),
            after_bars: Some(2),
            max_hits: Some(10),
            overlap_mode: Some("clip".to_string()),
            format: Some(HttpFormat::Json),
        };
        let time_machine_full_request = TimeMachineBarsRequest {
            metadata: Some(true),
            ..time_machine_min_request.clone()
        };

        match (
            client.time_machine(&time_machine_min_request).await,
            client.time_machine(&time_machine_full_request).await,
        ) {
            (Ok(min), Ok(full))
                if time_machine_rows_len(&min) > 0
                    && time_machine_rows_len(&min) == time_machine_rows_len(&full) =>
            {
                record_pass(
                    report,
                    "time_machine",
                    format!("rows={}", time_machine_rows_len(&min)),
                );
            }
            (Ok(min), Ok(full)) => record_fail(
                report,
                "time_machine",
                format!(
                    "unexpected min/full time-machine rows: min={} full={}",
                    time_machine_rows_len(&min),
                    time_machine_rows_len(&full)
                ),
            ),
            (Err(error), _) => record_fail(report, "time_machine", error.to_string()),
            (_, Err(error)) => record_fail(report, "time_machine", error.to_string()),
        }

        if runtime.summary.grpc_base_url.is_none() {
            record_fail(
                report,
                "latest_grpc",
                format!("missing required environment variable `{GRPC_ENV}`"),
            );
            record_fail(
                report,
                "range_grpc",
                format!("missing required environment variable `{GRPC_ENV}`"),
            );
            record_fail(
                report,
                "search_grpc",
                format!("missing required environment variable `{GRPC_ENV}`"),
            );
            record_fail(
                report,
                "time_machine_grpc",
                format!("missing required environment variable `{GRPC_ENV}`"),
            );
        } else {
            let latest_grpc_min_request = LatestBarsGrpcRequest::from(&latest_http_min_request);
            let latest_grpc_full_request = LatestBarsGrpcRequest::from(&latest_http_full_request);

            match (
                client.latest_grpc(&latest_grpc_min_request).await,
                client.latest_grpc(&latest_grpc_full_request).await,
            ) {
                (Ok(min), Ok(full))
                    if pair_from_latest_response(&min).is_some()
                        && pair_from_latest_response(&min) == pair_from_latest_response(&full) =>
                {
                    record_pass(
                        report,
                        "latest_grpc",
                        format!(
                            "pair={} close_end_ms={}",
                            pair_from_latest_response(&min).unwrap_or("unknown"),
                            close_end_from_latest_response(&min)
                        ),
                    );
                }
                (Ok(_), Ok(_)) => {
                    record_fail(report, "latest_grpc", "min/full latest parity mismatch")
                }
                (Err(error), _) => record_fail(report, "latest_grpc", error.to_string()),
                (_, Err(error)) => record_fail(report, "latest_grpc", error.to_string()),
            }

            let range_grpc_min_request = RangeBarsGrpcRequest::from(&range_min_request);
            let range_grpc_full_request = RangeBarsGrpcRequest::from(&range_full_request);
            match (
                client.range_grpc(&range_grpc_min_request).await,
                client.range_grpc(&range_grpc_full_request).await,
            ) {
                (Ok(min), Ok(full))
                    if range_rows_len(&min) > 0
                        && range_rows_len(&min) == range_rows_len(&full) =>
                {
                    record_pass(
                        report,
                        "range_grpc",
                        format!("rows={}", range_rows_len(&min)),
                    );
                }
                (Ok(min), Ok(full)) => record_fail(
                    report,
                    "range_grpc",
                    format!(
                        "unexpected min/full range rows: min={} full={}",
                        range_rows_len(&min),
                        range_rows_len(&full)
                    ),
                ),
                (Err(error), _) => record_fail(report, "range_grpc", error.to_string()),
                (_, Err(error)) => record_fail(report, "range_grpc", error.to_string()),
            }

            let search_grpc_min_request = SearchBarsGrpcRequest::from(&search_min_request);
            let search_grpc_full_request = SearchBarsGrpcRequest::from(&search_full_request);
            match (
                client.search_grpc(&search_grpc_min_request).await,
                client.search_grpc(&search_grpc_full_request).await,
            ) {
                (Ok(min), Ok(full))
                    if search_hits_len(&min) > 0
                        && search_hits_len(&min) == search_hits_len(&full) =>
                {
                    record_pass(
                        report,
                        "search_grpc",
                        format!("hits={}", search_hits_len(&min)),
                    );
                }
                (Ok(min), Ok(full)) => record_fail(
                    report,
                    "search_grpc",
                    format!(
                        "unexpected min/full search hits: min={} full={}",
                        search_hits_len(&min),
                        search_hits_len(&full)
                    ),
                ),
                (Err(error), _) => record_fail(report, "search_grpc", error.to_string()),
                (_, Err(error)) => record_fail(report, "search_grpc", error.to_string()),
            }

            let time_machine_grpc_min_request =
                TimeMachineBarsGrpcRequest::from(&time_machine_min_request);
            let time_machine_grpc_full_request =
                TimeMachineBarsGrpcRequest::from(&time_machine_full_request);
            match (
                client
                    .time_machine_grpc(&time_machine_grpc_min_request)
                    .await,
                client
                    .time_machine_grpc(&time_machine_grpc_full_request)
                    .await,
            ) {
                (Ok(min), Ok(full))
                    if time_machine_rows_len(&min) > 0
                        && time_machine_rows_len(&min) == time_machine_rows_len(&full) =>
                {
                    record_pass(
                        report,
                        "time_machine_grpc",
                        format!("rows={}", time_machine_rows_len(&min)),
                    );
                }
                (Ok(min), Ok(full)) => record_fail(
                    report,
                    "time_machine_grpc",
                    format!(
                        "unexpected min/full time-machine rows: min={} full={}",
                        time_machine_rows_len(&min),
                        time_machine_rows_len(&full)
                    ),
                ),
                (Err(error), _) => record_fail(report, "time_machine_grpc", error.to_string()),
                (_, Err(error)) => record_fail(report, "time_machine_grpc", error.to_string()),
            }
        }
    }

    if runtime.summary.ws_base_url.is_none() {
        record_fail(
            report,
            "connect_bars_ws",
            format!("missing required environment variable `{WS_ENV}`"),
        );
        record_fail(
            report,
            "connect_messages_ws",
            format!("missing required environment variable `{WS_ENV}`"),
        );
        record_fail(
            report,
            "connect_bars_ws_recovering",
            format!("missing required environment variable `{WS_ENV}`"),
        );
        record_fail(
            report,
            "connect_messages_ws_recovering",
            format!("missing required environment variable `{WS_ENV}`"),
        );
    } else {
        let ws_phase_started = Instant::now();
        let upstream_ws_base_url = runtime.summary.ws_base_url.as_deref().unwrap_or_default();

        match observe_raw_bars_ws(client, &target_pair).await {
            Ok(note) => record_pass(report, "connect_bars_ws", note),
            Err(error) => record_fail(report, "connect_bars_ws", error.to_string()),
        }

        match observe_raw_messages_ws(client, &target_pair).await {
            Ok(note) => record_pass(report, "connect_messages_ws", note),
            Err(error) => record_fail(report, "connect_messages_ws", error.to_string()),
        }

        match observe_recovering_bars_ws(runtime, upstream_ws_base_url, &target_pair).await {
            Ok(note) => record_pass(report, "connect_bars_ws_recovering", note),
            Err(error) => record_fail(report, "connect_bars_ws_recovering", error.to_string()),
        }

        match observe_recovering_messages_ws(runtime, upstream_ws_base_url, &target_pair).await {
            Ok(note) => record_pass(report, "connect_messages_ws_recovering", note),
            Err(error) => record_fail(report, "connect_messages_ws_recovering", error.to_string()),
        }

        let ws_phase_elapsed = ws_phase_started.elapsed();
        if ws_phase_elapsed < Duration::from_secs(WS_PHASE_MINIMUM_SECS) {
            record_fail(
                report,
                "connect_bars_ws",
                format!(
                    "ws validation floor not met; observed only {}s, required at least {}s",
                    ws_phase_elapsed.as_secs(),
                    WS_PHASE_MINIMUM_SECS
                ),
            );
        } else {
            report.proved_observations.push(format!(
                "WS validation phase lasted {} seconds across raw and recovering checks",
                ws_phase_elapsed.as_secs()
            ));
        }
    }
}

async fn run() -> Result<Report, String> {
    let repo_dotenv = repo_root().join(".env");
    load_dotenv_if_present(&repo_dotenv)?;

    let mut report = Report {
        title: "SDK Live Public Surface Verification Report".to_string(),
        execution_timestamp_utc: timestamp_for_report(),
        config: None,
        results: initial_surface_results(),
        proved_observations: Vec::new(),
        failures: Vec::new(),
        skipped: Vec::new(),
        final_status: "foundation_failed".to_string(),
    };

    println!("[{BIN_NAME}] starting live public surface checks");
    println!(
        "[{BIN_NAME}] loading environment from {}",
        repo_dotenv.display()
    );

    let runtime = match build_runtime_config() {
        Ok(runtime) => runtime,
        Err(error) => {
            report
                .failures
                .push(format!("client foundation setup failed: {error}"));
            return Ok(report);
        }
    };

    report.config = Some(runtime.summary.clone());
    report.proved_observations.push(format!(
        "loaded `{}` and constructed `Aggregator` successfully",
        HTTP_ENV
    ));
    report.proved_observations.push(
        "constructed `Primitives` from checked-in public defaults without introducing new environment variables".to_string(),
    );
    report
        .proved_observations
        .push("Markdown report directory and per-surface scaffold were initialized".to_string());

    println!("[{BIN_NAME}] client construction ok");
    run_live_checks(&runtime, &mut report).await;
    report.final_status = if report.failures.is_empty() {
        "live_public_surface_checks_passed".to_string()
    } else {
        "live_public_surface_checks_failed".to_string()
    };

    Ok(report)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let timestamp = timestamp_for_filename();
    let report = match run().await {
        Ok(report) => report,
        Err(error) => Report {
            title: "SDK Live Public Surface Verification Report".to_string(),
            execution_timestamp_utc: timestamp_for_report(),
            config: None,
            results: initial_surface_results(),
            proved_observations: Vec::new(),
            failures: vec![error],
            skipped: Vec::new(),
            final_status: "foundation_failed".to_string(),
        },
    };

    match write_report(&report, &timestamp) {
        Ok(path) => {
            println!("[{BIN_NAME}] report_written={}", path.display());
            if report.failures.is_empty() {
                std::process::exit(0);
            }
            eprintln!("[{BIN_NAME}] failures_present");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("[{BIN_NAME}] report_write_failed: {error}");
            std::process::exit(1);
        }
    }
}
