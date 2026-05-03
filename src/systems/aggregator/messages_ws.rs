use crate::core::error::SdkError;
use crate::streaming::subscription::{ExponentialBackoffConfig, ReconnectBackoff};
use crate::systems::types::Timeframe;
use crate::transport::ws::WsTransport;
use futures_util::{SinkExt, StreamExt};
use std::collections::BTreeMap;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

const MESSAGES_WS_PATH: &str = "/v1/ws/messages";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessagesWsClientFrame {
    Subscribe(MessagesWsSubscribeFrame),
    Unsubscribe(MessagesWsUnsubscribeFrame),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MessagesWsSubscribeFrame {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tfs: Option<Vec<Timeframe>>,
    pub predicate: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MessagesWsUnsubscribeFrame {
    pub id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessagesWsServerFrame {
    Subscribed(MessagesWsSubscribedFrame),
    Message(MessagesWsMessageFrame),
    #[serde(rename = "ws_aggregator_messages_heartbeat")]
    Heartbeat(MessagesWsHeartbeatFrame),
    Error(MessagesWsErrorFrame),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct MessagesWsSubscribedFrame {
    pub id: String,
    pub tfs: Vec<String>,
    pub predicate_pairs: Vec<String>,
    pub normalized_predicate: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct MessagesWsMessageFrame {
    pub id: String,
    pub tf: String,
    pub close_ms: i64,
    pub close_utc: String,
    pub at_ms: i64,
    pub message: String,
    pub predicate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct MessagesWsHeartbeatFrame {
    pub at_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark_end_ms: Option<i64>,
    pub subscription_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct MessagesWsErrorFrame {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: String,
    pub error: String,
}

#[derive(Debug)]
pub struct MessagesWsConnection {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

#[derive(Debug)]
pub struct RecoveringMessagesWsConnection {
    transport: WsTransport,
    backoff: ReconnectBackoff,
    active_subscriptions: BTreeMap<String, MessagesWsSubscribeFrame>,
    active: MessagesWsConnection,
}

impl MessagesWsSubscribeFrame {
    pub fn to_control_text(&self) -> Result<String, SdkError> {
        serde_json::to_string(&MessagesWsClientFrame::Subscribe(self.clone())).map_err(|source| {
            SdkError::request_build(format!("messages ws subscribe JSON failed: {source}"))
        })
    }
}

impl MessagesWsUnsubscribeFrame {
    pub fn to_control_text(&self) -> Result<String, SdkError> {
        serde_json::to_string(&MessagesWsClientFrame::Unsubscribe(self.clone())).map_err(|source| {
            SdkError::request_build(format!("messages ws unsubscribe JSON failed: {source}"))
        })
    }
}

impl MessagesWsConnection {
    pub async fn connect(transport: &WsTransport) -> Result<Self, SdkError> {
        let url = transport.endpoint_url(MESSAGES_WS_PATH)?;
        let mut upgrade = url.as_str().into_client_request().map_err(|source| {
            SdkError::ws_transport(format!(
                "messages ws upgrade request build failed: {source}"
            ))
        })?;

        let headers = transport.upgrade_headers()?;
        for (name, value) in headers.iter() {
            upgrade.headers_mut().insert(name, value.clone());
        }

        let (stream, _) = connect_async(upgrade).await.map_err(|source| {
            SdkError::ws_transport(format!("messages ws connect failed: {source}"))
        })?;

        Ok(Self { stream })
    }

    pub async fn send_subscribe(
        &mut self,
        frame: &MessagesWsSubscribeFrame,
    ) -> Result<(), SdkError> {
        self.stream
            .send(Message::Text(frame.to_control_text()?.into()))
            .await
            .map_err(|source| {
                SdkError::ws_transport(format!("messages ws subscribe send failed: {source}"))
            })
    }

    pub async fn send_unsubscribe(
        &mut self,
        frame: &MessagesWsUnsubscribeFrame,
    ) -> Result<(), SdkError> {
        self.stream
            .send(Message::Text(frame.to_control_text()?.into()))
            .await
            .map_err(|source| {
                SdkError::ws_transport(format!("messages ws unsubscribe send failed: {source}"))
            })
    }

    pub async fn next_frame(&mut self) -> Result<Option<MessagesWsServerFrame>, SdkError> {
        loop {
            let message = match self.stream.next().await {
                Some(Ok(message)) => message,
                Some(Err(source)) => {
                    return Err(SdkError::ws_transport(format!(
                        "messages ws receive failed: {source}"
                    )));
                }
                None => return Ok(None),
            };

            match message {
                Message::Text(text) => return decode_server_frame(text.as_ref()).map(Some),
                Message::Close(_) => return Ok(None),
                Message::Ping(payload) => {
                    self.stream
                        .send(Message::Pong(payload))
                        .await
                        .map_err(|source| {
                            SdkError::ws_transport(format!(
                                "messages ws pong send failed: {source}"
                            ))
                        })?;
                }
                Message::Pong(_) => {}
                Message::Binary(_) => {
                    return Err(SdkError::contract_drift(
                        "messages ws server sent unexpected binary frame",
                    ));
                }
                Message::Frame(_) => {}
            }
        }
    }
}

impl RecoveringMessagesWsConnection {
    pub async fn connect(
        transport: &WsTransport,
        config: ExponentialBackoffConfig,
    ) -> Result<Self, SdkError> {
        let active = MessagesWsConnection::connect(transport).await?;
        Ok(Self {
            transport: transport.clone(),
            backoff: ReconnectBackoff::new(config)?,
            active_subscriptions: BTreeMap::new(),
            active,
        })
    }

    pub fn backoff_config(&self) -> ExponentialBackoffConfig {
        self.backoff.config()
    }

    pub fn next_attempt(&self) -> u32 {
        self.backoff.next_attempt()
    }

    pub fn active_subscription_ids(&self) -> Vec<&str> {
        self.active_subscriptions
            .keys()
            .map(String::as_str)
            .collect()
    }

    pub async fn send_subscribe(
        &mut self,
        frame: &MessagesWsSubscribeFrame,
    ) -> Result<(), SdkError> {
        self.active.send_subscribe(frame).await?;
        self.active_subscriptions
            .insert(frame.id.clone(), frame.clone());
        Ok(())
    }

    pub async fn send_unsubscribe(
        &mut self,
        frame: &MessagesWsUnsubscribeFrame,
    ) -> Result<(), SdkError> {
        self.active.send_unsubscribe(frame).await?;
        self.active_subscriptions.remove(&frame.id);
        Ok(())
    }

    pub async fn next_frame(&mut self) -> Result<Option<MessagesWsServerFrame>, SdkError> {
        loop {
            match self.active.next_frame().await {
                Ok(Some(frame)) => return Ok(Some(frame)),
                Ok(None) => self.reconnect("messages ws connection closed").await?,
                Err(error) => {
                    self.reconnect(&format!("messages ws receive failed: {error}"))
                        .await?
                }
            }
        }
    }

    async fn reconnect(&mut self, reason: &str) -> Result<(), SdkError> {
        loop {
            let delay = self.backoff.next_sleep_duration().ok_or_else(|| {
                SdkError::ws_transport(format!(
                    "{reason}; ws recovery attempts exhausted for messages"
                ))
            })?;
            sleep(delay).await;

            match MessagesWsConnection::connect(&self.transport).await {
                Ok(mut connection) => {
                    for frame in self.active_subscriptions.values() {
                        connection.send_subscribe(frame).await?;
                    }
                    self.active = connection;
                    self.backoff.reset();
                    return Ok(());
                }
                Err(_) => continue,
            }
        }
    }
}

fn decode_server_frame(text: &str) -> Result<MessagesWsServerFrame, SdkError> {
    serde_json::from_str::<MessagesWsServerFrame>(text).map_err(|source| {
        SdkError::contract_drift(format!("messages ws frame decode failed: {source}"))
    })
}
