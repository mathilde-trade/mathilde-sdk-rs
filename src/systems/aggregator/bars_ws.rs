use crate::core::error::SdkError;
use crate::core::time::TimeInput;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1 as proto;
use crate::streaming::make_before_break::MakeBeforeBreakConfig;
use crate::streaming::subscription::{ExponentialBackoffConfig, ReconnectBackoff};
use crate::systems::aggregator::types::Bar;
use crate::systems::aggregator::types::normalize_pair_values;
use crate::systems::types::Timeframe;
use crate::transport::ws::WsTransport;
use futures_util::{SinkExt, StreamExt};
use prost::Message as ProstMessage;
use std::collections::VecDeque;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep, timeout};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

const BARS_WS_PATH: &str = "/v1/ws/bars";

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BarsWsFormat {
    #[default]
    Json,
    Protobuf,
}

impl BarsWsFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Protobuf => "protobuf",
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BarsWsPhase {
    Replay,
    Live,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BarsWsSubscribeRequest {
    pub pairs: Vec<String>,
    pub tfs: Vec<Timeframe>,
    pub metadata: Option<bool>,
    pub from_close: Option<TimeInput>,
    pub last_n_bars: Option<i64>,
    pub format: Option<BarsWsFormat>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct NormalizedBarsWsSubscribeRequest {
    pub pairs: Vec<String>,
    pub tfs: Vec<String>,
    pub metadata: bool,
    #[serde(rename = "from_close_ms", skip_serializing_if = "Option::is_none")]
    pub from_close_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_n_bars: Option<i64>,
    pub format: BarsWsFormat,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BarsWsMetaFrame {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_ms: Option<i64>,
    pub watermark_end_ms: i64,
    pub phase: BarsWsPhase,
    #[serde(default)]
    pub missing_pairs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BarsWsErrorFrame {
    pub kind: String,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BarsWsInboundFrame {
    Meta(BarsWsMetaFrame),
    Error(BarsWsErrorFrame),
    JsonRows(Vec<Bar>),
    ProtobufRows(Vec<Bar>),
}

#[derive(Debug)]
pub struct BarsWsConnection {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

#[derive(Debug)]
pub struct BarsWsMakeBeforeBreak {
    transport: WsTransport,
    config: MakeBeforeBreakConfig,
    active_request: BarsWsSubscribeRequest,
    active: BarsWsConnection,
    active_buffer: VecDeque<BarsWsInboundFrame>,
    pending_swap: Option<PendingBarsWsSwap>,
}

#[derive(Debug)]
pub struct RecoveringBarsWsConnection {
    transport: WsTransport,
    request: BarsWsSubscribeRequest,
    backoff: ReconnectBackoff,
    active: BarsWsConnection,
}

#[derive(Debug)]
struct PendingBarsWsSwap {
    task: JoinHandle<Result<ValidatedBarsWsCandidate, SdkError>>,
}

#[derive(Debug)]
struct ValidatedBarsWsCandidate {
    request: BarsWsSubscribeRequest,
    connection: BarsWsConnection,
    buffered_frames: VecDeque<BarsWsInboundFrame>,
}

impl BarsWsMetaFrame {
    pub fn is_replay_done(&self) -> bool {
        self.phase == BarsWsPhase::Replay && self.event.as_deref() == Some("replay_done")
    }
}

impl BarsWsSubscribeRequest {
    pub fn normalize(&self) -> Result<NormalizedBarsWsSubscribeRequest, SdkError> {
        let pairs = normalize_pair_values(&self.pairs);
        if pairs.is_empty() {
            return Err(SdkError::request_build(
                "bars ws subscribe requires at least one pair",
            ));
        }

        if self.tfs.is_empty() {
            return Err(SdkError::request_build(
                "bars ws subscribe requires at least one timeframe",
            ));
        }

        if self.from_close.is_some() && self.last_n_bars.is_some() {
            return Err(SdkError::request_build(
                "bars ws subscribe accepts `from_close` or `last_n_bars`, not both",
            ));
        }

        if let Some(last_n_bars) = self.last_n_bars {
            if last_n_bars <= 0 {
                return Err(SdkError::request_build(
                    "bars ws subscribe requires `last_n_bars` > 0 when provided",
                ));
            }
        }

        Ok(NormalizedBarsWsSubscribeRequest {
            pairs,
            tfs: self
                .tfs
                .iter()
                .map(|timeframe| timeframe.as_str().to_string())
                .collect(),
            metadata: self.metadata.unwrap_or(false),
            from_close_ms: self
                .from_close
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?,
            last_n_bars: self.last_n_bars,
            format: self.format.unwrap_or_default(),
        })
    }

    pub fn to_subscribe_text(&self) -> Result<String, SdkError> {
        serde_json::to_string(&self.normalize()?).map_err(|source| {
            SdkError::request_build(format!("bars ws subscribe JSON failed: {source}"))
        })
    }
}

impl BarsWsConnection {
    pub async fn connect(
        transport: &WsTransport,
        request: &BarsWsSubscribeRequest,
    ) -> Result<Self, SdkError> {
        let url = transport.endpoint_url(BARS_WS_PATH)?;
        let mut upgrade = url.as_str().into_client_request().map_err(|source| {
            SdkError::ws_transport(format!("ws upgrade request build failed: {source}"))
        })?;

        let headers = transport.upgrade_headers()?;
        for (name, value) in headers.iter() {
            upgrade.headers_mut().insert(name, value.clone());
        }

        let (mut stream, _) = connect_async(upgrade)
            .await
            .map_err(|source| SdkError::ws_transport(format!("ws connect failed: {source}")))?;

        stream
            .send(Message::Text(request.to_subscribe_text()?.into()))
            .await
            .map_err(|source| {
                SdkError::ws_transport(format!("ws subscribe send failed: {source}"))
            })?;

        Ok(Self { stream })
    }

    pub async fn next_frame(
        &mut self,
        request: &BarsWsSubscribeRequest,
    ) -> Result<Option<BarsWsInboundFrame>, SdkError> {
        loop {
            let message = match self.stream.next().await {
                Some(Ok(message)) => message,
                Some(Err(source)) => {
                    return Err(SdkError::ws_transport(format!(
                        "ws receive failed: {source}"
                    )));
                }
                None => return Ok(None),
            };

            match message {
                Message::Text(text) => return decode_text_frame(text.as_ref(), request).map(Some),
                Message::Binary(bytes) => {
                    return decode_binary_frame(bytes.as_ref(), request).map(Some);
                }
                Message::Close(_) => return Ok(None),
                Message::Ping(payload) => {
                    self.stream
                        .send(Message::Pong(payload))
                        .await
                        .map_err(|source| {
                            SdkError::ws_transport(format!("ws pong send failed: {source}"))
                        })?;
                }
                Message::Pong(_) => {}
                Message::Frame(_) => {}
            }
        }
    }
}

impl BarsWsMakeBeforeBreak {
    pub async fn connect(
        transport: &WsTransport,
        request: &BarsWsSubscribeRequest,
        config: MakeBeforeBreakConfig,
    ) -> Result<Self, SdkError> {
        let active = BarsWsConnection::connect(transport, request).await?;
        Ok(Self {
            transport: transport.clone(),
            config,
            active_request: request.clone(),
            active,
            active_buffer: VecDeque::new(),
            pending_swap: None,
        })
    }

    pub fn active_request(&self) -> &BarsWsSubscribeRequest {
        &self.active_request
    }

    pub fn swap_in_progress(&self) -> bool {
        self.pending_swap.is_some()
    }

    pub fn begin_swap(&mut self, request: &BarsWsSubscribeRequest) -> Result<(), SdkError> {
        if self.pending_swap.is_some() {
            return Err(SdkError::request_build(
                "bars ws make-before-break swap already in progress",
            ));
        }

        let transport = self.transport.clone();
        let config = self.config;
        let request = request.clone();
        let task = tokio::spawn(async move {
            validate_candidate_stream(transport, request, config.validation_window).await
        });
        self.pending_swap = Some(PendingBarsWsSwap { task });
        Ok(())
    }

    pub async fn next_frame(&mut self) -> Result<Option<BarsWsInboundFrame>, SdkError> {
        if let Some(frame) = self.active_buffer.pop_front() {
            return Ok(Some(frame));
        }

        self.promote_swap_if_ready().await?;

        if let Some(frame) = self.active_buffer.pop_front() {
            return Ok(Some(frame));
        }

        self.active.next_frame(&self.active_request).await
    }

    async fn promote_swap_if_ready(&mut self) -> Result<(), SdkError> {
        let finished = self
            .pending_swap
            .as_ref()
            .map(|pending| pending.task.is_finished())
            .unwrap_or(false);
        if !finished {
            return Ok(());
        }

        let pending = self
            .pending_swap
            .take()
            .ok_or_else(|| SdkError::contract_drift("bars ws pending swap disappeared"))?;

        match pending.task.await {
            Ok(Ok(candidate)) => {
                self.active_request = candidate.request;
                self.active = candidate.connection;
                self.active_buffer = candidate.buffered_frames;
                Ok(())
            }
            Ok(Err(error)) => Err(error),
            Err(join_error) => Err(SdkError::ws_transport(format!(
                "bars ws make-before-break task failed: {join_error}"
            ))),
        }
    }
}

impl RecoveringBarsWsConnection {
    pub async fn connect(
        transport: &WsTransport,
        request: &BarsWsSubscribeRequest,
        config: ExponentialBackoffConfig,
    ) -> Result<Self, SdkError> {
        let active = BarsWsConnection::connect(transport, request).await?;
        Ok(Self {
            transport: transport.clone(),
            request: request.clone(),
            backoff: ReconnectBackoff::new(config)?,
            active,
        })
    }

    pub fn active_request(&self) -> &BarsWsSubscribeRequest {
        &self.request
    }

    pub fn backoff_config(&self) -> ExponentialBackoffConfig {
        self.backoff.config()
    }

    pub fn next_attempt(&self) -> u32 {
        self.backoff.next_attempt()
    }

    pub async fn next_frame(&mut self) -> Result<Option<BarsWsInboundFrame>, SdkError> {
        loop {
            match self.active.next_frame(&self.request).await {
                Ok(Some(frame)) => return Ok(Some(frame)),
                Ok(None) => self.reconnect("bars ws connection closed").await?,
                Err(error) => {
                    self.reconnect(&format!("bars ws receive failed: {error}"))
                        .await?
                }
            }
        }
    }

    async fn reconnect(&mut self, reason: &str) -> Result<(), SdkError> {
        loop {
            let delay = self.backoff.next_sleep_duration().ok_or_else(|| {
                SdkError::ws_transport(format!("{reason}; ws recovery attempts exhausted for bars"))
            })?;
            sleep(delay).await;

            match BarsWsConnection::connect(&self.transport, &self.request).await {
                Ok(connection) => {
                    self.active = connection;
                    self.backoff.reset();
                    return Ok(());
                }
                Err(_) => continue,
            }
        }
    }
}

fn decode_text_frame(
    text: &str,
    request: &BarsWsSubscribeRequest,
) -> Result<BarsWsInboundFrame, SdkError> {
    if let Ok(error) = serde_json::from_str::<BarsWsErrorFrame>(text) {
        if !error.kind.is_empty() && !error.error.is_empty() {
            return Ok(BarsWsInboundFrame::Error(error));
        }
    }

    if let Ok(meta) = serde_json::from_str::<BarsWsMetaFrame>(text) {
        return Ok(BarsWsInboundFrame::Meta(meta));
    }

    let metadata_required = request.metadata.unwrap_or(false);
    let rows = serde_json::from_str::<Vec<Bar>>(text).map_err(|source| {
        SdkError::contract_drift(format!("bars ws JSON rows decode failed: {source}"))
    })?;
    for row in &rows {
        row.ensure_metadata_shape(metadata_required, "bars ws JSON row")?;
    }
    Ok(BarsWsInboundFrame::JsonRows(rows))
}

fn decode_binary_frame(
    bytes: &[u8],
    request: &BarsWsSubscribeRequest,
) -> Result<BarsWsInboundFrame, SdkError> {
    let payload = proto::BarsRowsPayloadV1::decode(bytes).map_err(|source| {
        SdkError::contract_drift(format!("bars ws protobuf payload decode failed: {source}"))
    })?;

    let view = proto::BarsViewV1::try_from(payload.view).map_err(|_| {
        SdkError::contract_drift(format!(
            "bars ws protobuf payload has unknown view `{}`",
            payload.view
        ))
    })?;

    let metadata_required = request.metadata.unwrap_or(false);

    match view {
        proto::BarsViewV1::Min | proto::BarsViewV1::Full => {
            if metadata_required != matches!(view, proto::BarsViewV1::Full) {
                return Err(SdkError::contract_drift(
                    "bars ws protobuf payload view does not match metadata request",
                ));
            }

            let rows = payload
                .rows
                .into_iter()
                .map(Bar::from_proto_latest)
                .collect::<Result<Vec<_>, _>>()?;
            for row in &rows {
                row.ensure_metadata_shape(metadata_required, "bars ws protobuf row")?;
            }
            Ok(BarsWsInboundFrame::ProtobufRows(rows))
        }
        proto::BarsViewV1::Unspecified => Err(SdkError::contract_drift(
            "bars ws protobuf payload view is unspecified",
        )),
    }
}

async fn validate_candidate_stream(
    transport: WsTransport,
    request: BarsWsSubscribeRequest,
    validation_window: std::time::Duration,
) -> Result<ValidatedBarsWsCandidate, SdkError> {
    let mut connection = BarsWsConnection::connect(&transport, &request).await?;
    let started = Instant::now();
    let mut buffered_frames = VecDeque::new();

    loop {
        let elapsed = started.elapsed();
        if elapsed >= validation_window {
            return Ok(ValidatedBarsWsCandidate {
                request,
                connection,
                buffered_frames,
            });
        }

        let remaining = validation_window - elapsed;
        match timeout(remaining, connection.next_frame(&request)).await {
            Ok(Ok(Some(BarsWsInboundFrame::Error(error)))) => {
                return Err(SdkError::ws_transport(format!(
                    "bars ws candidate validation failed: {}: {}",
                    error.kind, error.error
                )));
            }
            Ok(Ok(Some(frame))) => buffered_frames.push_back(frame),
            Ok(Ok(None)) => {
                return Err(SdkError::ws_transport(
                    "bars ws candidate closed during validation window",
                ));
            }
            Ok(Err(error)) => return Err(error),
            Err(_) => {
                return Ok(ValidatedBarsWsCandidate {
                    request,
                    connection,
                    buffered_frames,
                });
            }
        }
    }
}
