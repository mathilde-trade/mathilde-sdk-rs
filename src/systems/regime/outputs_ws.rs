use crate::core::error::SdkError;
use crate::core::time::TimeInput;
use crate::generated::regime::{ProcessorFamily, ProcessorGroup};
use crate::streaming::make_before_break::MakeBeforeBreakConfig;
use crate::streaming::subscription::{ExponentialBackoffConfig, ReconnectBackoff};
use crate::systems::regime::types::{
    LatestOutputsPresentRow, decode_latest_outputs_ws_json, decode_latest_outputs_ws_proto,
    diagnostics_enabled, ensure_supported_regime_tf, infer_output_mode, normalize_family_selectors,
    normalize_group_selectors, normalize_pair_values, selector_family_names, selector_group_names,
};
use crate::systems::types::Timeframe;
use crate::transport::ws::WsTransport;
use futures_util::{SinkExt, StreamExt};
use std::collections::VecDeque;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep, timeout};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

const OUTPUTS_WS_PATH: &str = "/v1/ws/outputs";

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputsWsFormat {
    #[default]
    Json,
    Protobuf,
}

impl OutputsWsFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Protobuf => "protobuf",
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputsWsPhase {
    Replay,
    Live,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct OutputsWsSubscribeRequest {
    pub pairs: Vec<String>,
    pub tfs: Vec<Timeframe>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub secondary: Option<bool>,
    pub from_close: Option<TimeInput>,
    pub last_n_bars: Option<i64>,
    pub format: Option<OutputsWsFormat>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct NormalizedOutputsWsSubscribeRequest {
    pub pairs: Vec<String>,
    pub tfs: Vec<String>,
    pub metadata: bool,
    pub diagnostics: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<bool>,
    #[serde(rename = "from_close_ms", skip_serializing_if = "Option::is_none")]
    pub from_close_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_n_bars: Option<i64>,
    pub format: OutputsWsFormat,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct OutputsWsMetaFrame {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_ms: Option<i64>,
    pub watermark_end_ms: i64,
    pub phase: OutputsWsPhase,
    #[serde(default)]
    pub missing_pairs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct OutputsWsErrorFrame {
    pub kind: String,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputsWsInboundFrame {
    Meta(OutputsWsMetaFrame),
    Error(OutputsWsErrorFrame),
    JsonRows(Vec<LatestOutputsPresentRow>),
    ProtobufRows(Vec<LatestOutputsPresentRow>),
}

#[derive(Debug)]
pub struct OutputsWsConnection {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

#[derive(Debug)]
pub struct OutputsWsMakeBeforeBreak {
    transport: WsTransport,
    config: MakeBeforeBreakConfig,
    active_request: OutputsWsSubscribeRequest,
    active: OutputsWsConnection,
    active_buffer: VecDeque<OutputsWsInboundFrame>,
    pending_swap: Option<PendingOutputsWsSwap>,
}

#[derive(Debug)]
pub struct RecoveringOutputsWsConnection {
    transport: WsTransport,
    request: OutputsWsSubscribeRequest,
    backoff: ReconnectBackoff,
    active: OutputsWsConnection,
}

#[derive(Debug)]
struct PendingOutputsWsSwap {
    task: JoinHandle<Result<ValidatedOutputsWsCandidate, SdkError>>,
}

#[derive(Debug)]
struct ValidatedOutputsWsCandidate {
    request: OutputsWsSubscribeRequest,
    connection: OutputsWsConnection,
    buffered_frames: VecDeque<OutputsWsInboundFrame>,
}

impl OutputsWsMetaFrame {
    pub fn is_replay_done(&self) -> bool {
        self.phase == OutputsWsPhase::Replay && self.event.as_deref() == Some("replay_done")
    }
}

impl OutputsWsSubscribeRequest {
    pub(crate) fn output_mode(
        &self,
    ) -> Result<crate::systems::regime::types::RegimeOutputMode, SdkError> {
        infer_output_mode(
            self.family.as_deref(),
            self.group.as_deref(),
            self.secondary,
            self.metadata,
        )
    }

    pub fn normalize(&self) -> Result<NormalizedOutputsWsSubscribeRequest, SdkError> {
        let pairs = normalize_pair_values(&self.pairs);
        if pairs.is_empty() {
            return Err(SdkError::request_build(
                "outputs ws subscribe requires at least one pair",
            ));
        }

        if self.tfs.is_empty() {
            return Err(SdkError::request_build(
                "outputs ws subscribe requires at least one timeframe",
            ));
        }
        for tf in &self.tfs {
            ensure_supported_regime_tf(*tf, "outputs ws subscribe")?;
        }

        if self.from_close.is_some() && self.last_n_bars.is_some() {
            return Err(SdkError::request_build(
                "outputs ws subscribe accepts `from_close` or `last_n_bars`, not both",
            ));
        }

        if let Some(last_n_bars) = self.last_n_bars {
            if last_n_bars <= 0 {
                return Err(SdkError::request_build(
                    "outputs ws subscribe requires `last_n_bars` > 0 when provided",
                ));
            }
        }

        let output_mode = self.output_mode()?;
        let format = self.format.unwrap_or_default();
        if output_mode.is_projected() && matches!(format, OutputsWsFormat::Protobuf) {
            return Err(SdkError::unsupported_or_unproved_usage(
                "outputs ws projected protobuf decoding is not proved because unselected computed fields collapse to unset optional fields",
            ));
        }

        let family = normalize_family_selectors(self.family.as_deref());
        let group = normalize_group_selectors(self.group.as_deref());

        Ok(NormalizedOutputsWsSubscribeRequest {
            pairs,
            tfs: self
                .tfs
                .iter()
                .map(|timeframe| timeframe.as_str().to_string())
                .collect(),
            metadata: output_mode.has_metadata(),
            diagnostics: diagnostics_enabled(self.diagnostics),
            family: {
                let names = selector_family_names(family.as_deref());
                if names.is_empty() { None } else { Some(names) }
            },
            group: {
                let names = selector_group_names(group.as_deref());
                if names.is_empty() { None } else { Some(names) }
            },
            secondary: self.secondary,
            from_close_ms: self
                .from_close
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?,
            last_n_bars: self.last_n_bars,
            format,
        })
    }

    pub fn to_subscribe_text(&self) -> Result<String, SdkError> {
        serde_json::to_string(&self.normalize()?).map_err(|source| {
            SdkError::request_build(format!("outputs ws subscribe JSON failed: {source}"))
        })
    }
}

impl OutputsWsConnection {
    pub async fn connect(
        transport: &WsTransport,
        request: &OutputsWsSubscribeRequest,
    ) -> Result<Self, SdkError> {
        let url = transport.endpoint_url(OUTPUTS_WS_PATH)?;
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
        request: &OutputsWsSubscribeRequest,
    ) -> Result<Option<OutputsWsInboundFrame>, SdkError> {
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

impl OutputsWsMakeBeforeBreak {
    pub async fn connect(
        transport: &WsTransport,
        request: &OutputsWsSubscribeRequest,
        config: MakeBeforeBreakConfig,
    ) -> Result<Self, SdkError> {
        let active = OutputsWsConnection::connect(transport, request).await?;
        Ok(Self {
            transport: transport.clone(),
            config,
            active_request: request.clone(),
            active,
            active_buffer: VecDeque::new(),
            pending_swap: None,
        })
    }

    pub fn active_request(&self) -> &OutputsWsSubscribeRequest {
        &self.active_request
    }

    pub fn swap_in_progress(&self) -> bool {
        self.pending_swap.is_some()
    }

    pub fn begin_swap(&mut self, request: &OutputsWsSubscribeRequest) -> Result<(), SdkError> {
        if self.pending_swap.is_some() {
            return Err(SdkError::request_build(
                "outputs ws make-before-break swap already in progress",
            ));
        }

        let transport = self.transport.clone();
        let config = self.config;
        let request = request.clone();
        let task = tokio::spawn(async move {
            validate_candidate_stream(transport, request, config.validation_window).await
        });
        self.pending_swap = Some(PendingOutputsWsSwap { task });
        Ok(())
    }

    pub async fn next_frame(&mut self) -> Result<Option<OutputsWsInboundFrame>, SdkError> {
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
            .ok_or_else(|| SdkError::contract_drift("outputs ws pending swap disappeared"))?;

        match pending.task.await {
            Ok(Ok(candidate)) => {
                self.active_request = candidate.request;
                self.active = candidate.connection;
                self.active_buffer = candidate.buffered_frames;
                Ok(())
            }
            Ok(Err(error)) => Err(error),
            Err(join_error) => Err(SdkError::ws_transport(format!(
                "outputs ws make-before-break task failed: {join_error}"
            ))),
        }
    }
}

impl RecoveringOutputsWsConnection {
    pub async fn connect(
        transport: &WsTransport,
        request: &OutputsWsSubscribeRequest,
        config: ExponentialBackoffConfig,
    ) -> Result<Self, SdkError> {
        let active = OutputsWsConnection::connect(transport, request).await?;
        Ok(Self {
            transport: transport.clone(),
            request: request.clone(),
            backoff: ReconnectBackoff::new(config)?,
            active,
        })
    }

    pub fn active_request(&self) -> &OutputsWsSubscribeRequest {
        &self.request
    }

    pub fn backoff_config(&self) -> ExponentialBackoffConfig {
        self.backoff.config()
    }

    pub fn next_attempt(&self) -> u32 {
        self.backoff.next_attempt()
    }

    pub async fn next_frame(&mut self) -> Result<Option<OutputsWsInboundFrame>, SdkError> {
        loop {
            match self.active.next_frame(&self.request).await {
                Ok(Some(frame)) => return Ok(Some(frame)),
                Ok(None) => self.reconnect("outputs ws connection closed").await?,
                Err(error) => {
                    self.reconnect(&format!("outputs ws receive failed: {error}"))
                        .await?
                }
            }
        }
    }

    async fn reconnect(&mut self, reason: &str) -> Result<(), SdkError> {
        loop {
            let delay = self.backoff.next_sleep_duration().ok_or_else(|| {
                SdkError::ws_transport(format!(
                    "{reason}; ws recovery attempts exhausted for outputs"
                ))
            })?;
            sleep(delay).await;

            match OutputsWsConnection::connect(&self.transport, &self.request).await {
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
    request: &OutputsWsSubscribeRequest,
) -> Result<OutputsWsInboundFrame, SdkError> {
    if let Ok(error) = serde_json::from_str::<OutputsWsErrorFrame>(text) {
        if !error.kind.is_empty() && !error.error.is_empty() {
            return Ok(OutputsWsInboundFrame::Error(error));
        }
    }

    if let Ok(meta) = serde_json::from_str::<OutputsWsMetaFrame>(text) {
        return Ok(OutputsWsInboundFrame::Meta(meta));
    }

    let output_mode = request.output_mode()?;
    let rows =
        decode_latest_outputs_ws_json(text, output_mode, diagnostics_enabled(request.diagnostics))?;
    Ok(OutputsWsInboundFrame::JsonRows(rows))
}

fn decode_binary_frame(
    bytes: &[u8],
    request: &OutputsWsSubscribeRequest,
) -> Result<OutputsWsInboundFrame, SdkError> {
    let rows = decode_latest_outputs_ws_proto(
        bytes,
        request.output_mode()?,
        diagnostics_enabled(request.diagnostics),
    )?;
    Ok(OutputsWsInboundFrame::ProtobufRows(rows))
}

async fn validate_candidate_stream(
    transport: WsTransport,
    request: OutputsWsSubscribeRequest,
    validation_window: std::time::Duration,
) -> Result<ValidatedOutputsWsCandidate, SdkError> {
    let mut connection = OutputsWsConnection::connect(&transport, &request).await?;
    let started = Instant::now();
    let mut buffered_frames = VecDeque::new();

    loop {
        let elapsed = started.elapsed();
        if elapsed >= validation_window {
            return Ok(ValidatedOutputsWsCandidate {
                request,
                connection,
                buffered_frames,
            });
        }

        let remaining = validation_window - elapsed;
        match timeout(remaining, connection.next_frame(&request)).await {
            Ok(Ok(Some(OutputsWsInboundFrame::Error(error)))) => {
                return Err(SdkError::ws_transport(format!(
                    "outputs ws candidate validation failed: {}: {}",
                    error.kind, error.error
                )));
            }
            Ok(Ok(Some(frame))) => buffered_frames.push_back(frame),
            Ok(Ok(None)) => {
                return Err(SdkError::ws_transport(
                    "outputs ws candidate closed during validation window",
                ));
            }
            Ok(Err(error)) => return Err(error),
            Err(_) => {
                return Ok(ValidatedOutputsWsCandidate {
                    request,
                    connection,
                    buffered_frames,
                });
            }
        }
    }
}
