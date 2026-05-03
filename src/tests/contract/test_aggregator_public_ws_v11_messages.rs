use crate::core::auth::BearerToken;
use crate::core::config::{AggregatorConfig, HttpTransportConfig, WsTransportConfig};
use crate::streaming::subscription::ExponentialBackoffConfig;
use crate::systems::aggregator::{
    AggregatorClient, MessagesWsServerFrame, MessagesWsSubscribeFrame, MessagesWsUnsubscribeFrame,
};
use crate::systems::types::Timeframe;
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

#[derive(Debug)]
struct CapturedMessagesConnect {
    path: String,
    authorization: Option<String>,
    subscribe_text: String,
    unsubscribe_text: String,
}

fn config_for_ws(base_url: &str, bearer_token: Option<BearerToken>) -> AggregatorConfig {
    AggregatorConfig {
        http: Some(HttpTransportConfig::new("http://127.0.0.1:1").expect("valid dummy http url")),
        grpc: None,
        ws: Some(WsTransportConfig::new(base_url).expect("valid ws url")),
        bearer_token,
    }
}

async fn spawn_messages_ws_server() -> (String, oneshot::Receiver<CapturedMessagesConnect>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind messages ws test server");
    let addr = listener.local_addr().expect("messages ws test addr");
    let (captured_tx, captured_rx) = oneshot::channel();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept messages ws conn");
        let capture = Arc::new(Mutex::new(None::<(String, Option<String>)>));
        let capture_for_cb = capture.clone();
        let mut ws = accept_hdr_async(stream, move |request: &Request, response: Response| {
            let authorization = request
                .headers()
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(ToOwned::to_owned);
            *capture_for_cb.lock().expect("capture mutex") =
                Some((request.uri().path().to_string(), authorization));
            Ok(response)
        })
        .await
        .expect("accept messages ws handshake");

        let subscribe_text = match ws.next().await {
            Some(Ok(Message::Text(text))) => text.to_string(),
            other => panic!("expected subscribe text, got {other:?}"),
        };
        let unsubscribe_text = match ws.next().await {
            Some(Ok(Message::Text(text))) => text.to_string(),
            other => panic!("expected unsubscribe text, got {other:?}"),
        };

        let (path, authorization) = capture
            .lock()
            .expect("capture mutex")
            .take()
            .expect("captured request");
        let _ = captured_tx.send(CapturedMessagesConnect {
            path,
            authorization,
            subscribe_text,
            unsubscribe_text,
        });

        ws.send(Message::Text(
            serde_json::json!({
                "type": "subscribed",
                "id": "rule_1",
                "tfs": ["1m", "5m"],
                "predicate_pairs": ["BTCUSDT", "ETHUSDT"],
                "normalized_predicate": "BTCUSDT.c > ETHUSDT.c"
            })
            .to_string()
            .into(),
        ))
        .await
        .expect("send subscribed");

        ws.send(Message::Text(
            serde_json::json!({
                "type": "message",
                "id": "rule_1",
                "tf": "1m",
                "close_ms": 1774693020000_i64,
                "close_utc": "2026-03-28T10:17:00Z",
                "at_ms": 1774693020118_i64,
                "message": "rule triggered",
                "predicate": "BTCUSDT.c > ETHUSDT.c",
                "payload": {"strategy":"alpha"}
            })
            .to_string()
            .into(),
        ))
        .await
        .expect("send message");

        ws.send(Message::Text(
            serde_json::json!({
                "type": "ws_aggregator_messages_heartbeat",
                "at_ms": 1774693080000_i64,
                "watermark_end_ms": null,
                "subscription_ids": ["rule_1"]
            })
            .to_string()
            .into(),
        ))
        .await
        .expect("send heartbeat");

        ws.send(Message::Text(
            serde_json::json!({
                "type": "error",
                "id": "rule_2",
                "kind": "invalid_request",
                "error": "unknown subscription id"
            })
            .to_string()
            .into(),
        ))
        .await
        .expect("send error");
    });

    (format!("http://{addr}"), captured_rx)
}

async fn spawn_recovering_messages_ws_server() -> (String, oneshot::Receiver<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind recovering messages ws test server");
    let addr = listener
        .local_addr()
        .expect("recovering messages ws test addr");
    let (captured_tx, captured_rx) = oneshot::channel();

    tokio::spawn(async move {
        let mut captured = Vec::new();

        let (stream_1, _) = listener
            .accept()
            .await
            .expect("accept first recovering messages conn");
        let mut ws_1 = accept_hdr_async(stream_1, |_request: &Request, response: Response| {
            Ok(response)
        })
        .await
        .expect("accept first recovering messages handshake");
        let subscribe_text = match ws_1.next().await {
            Some(Ok(Message::Text(text))) => text.to_string(),
            other => panic!("expected first subscribe text, got {other:?}"),
        };
        captured.push(subscribe_text);
        let _ = ws_1.close(None).await;

        let (stream_2, _) = listener
            .accept()
            .await
            .expect("accept second recovering messages conn");
        let mut ws_2 = accept_hdr_async(stream_2, |_request: &Request, response: Response| {
            Ok(response)
        })
        .await
        .expect("accept second recovering messages handshake");
        let replayed_subscribe_text = match ws_2.next().await {
            Some(Ok(Message::Text(text))) => text.to_string(),
            other => panic!("expected replayed subscribe text, got {other:?}"),
        };
        captured.push(replayed_subscribe_text);
        let _ = captured_tx.send(captured);

        ws_2.send(Message::Text(
            serde_json::json!({
                "type": "subscribed",
                "id": "rule_1",
                "tfs": ["1m"],
                "predicate_pairs": ["BTCUSDT"],
                "normalized_predicate": "BTCUSDT.c > 0"
            })
            .to_string()
            .into(),
        ))
        .await
        .expect("send replayed subscribed frame");
    });

    (format!("http://{addr}"), captured_rx)
}

#[tokio::test]
async fn test_connect_messages_ws_sends_auth_and_control_frames_and_decodes_server_frames() {
    let (base_url, captured_rx) = spawn_messages_ws_server().await;
    let client = AggregatorClient::new(config_for_ws(
        &base_url,
        Some(BearerToken::new("feed_public_token").expect("valid token")),
    ))
    .expect("aggregator client");

    let mut connection = client
        .connect_messages_ws()
        .await
        .expect("connect messages ws");

    connection
        .send_subscribe(&MessagesWsSubscribeFrame {
            id: "rule_1".to_string(),
            tfs: Some(vec![Timeframe::M1, Timeframe::M5]),
            predicate: "BTCUSDT.c > ETHUSDT.c".to_string(),
            message: "rule triggered".to_string(),
            payload: Some(serde_json::json!({"strategy":"alpha"})),
        })
        .await
        .expect("send subscribe");

    connection
        .send_unsubscribe(&MessagesWsUnsubscribeFrame {
            id: "rule_1".to_string(),
        })
        .await
        .expect("send unsubscribe");

    let captured = captured_rx.await.expect("captured messages ws connect");
    assert_eq!(captured.path, "/v1/ws/messages");
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer feed_public_token")
    );

    let subscribe_json: serde_json::Value =
        serde_json::from_str(&captured.subscribe_text).expect("subscribe json");
    assert_eq!(subscribe_json["type"], serde_json::json!("subscribe"));
    assert_eq!(subscribe_json["id"], serde_json::json!("rule_1"));
    assert_eq!(subscribe_json["tfs"], serde_json::json!(["1m", "5m"]));
    assert_eq!(
        subscribe_json["predicate"],
        serde_json::json!("BTCUSDT.c > ETHUSDT.c")
    );
    assert_eq!(
        subscribe_json["message"],
        serde_json::json!("rule triggered")
    );
    assert_eq!(
        subscribe_json["payload"],
        serde_json::json!({"strategy":"alpha"})
    );

    let unsubscribe_json: serde_json::Value =
        serde_json::from_str(&captured.unsubscribe_text).expect("unsubscribe json");
    assert_eq!(unsubscribe_json["type"], serde_json::json!("unsubscribe"));
    assert_eq!(unsubscribe_json["id"], serde_json::json!("rule_1"));

    match connection
        .next_frame()
        .await
        .expect("subscribed frame")
        .expect("some frame")
    {
        MessagesWsServerFrame::Subscribed(frame) => {
            assert_eq!(frame.id, "rule_1");
            assert_eq!(frame.tfs, vec!["1m".to_string(), "5m".to_string()]);
            assert_eq!(
                frame.predicate_pairs,
                vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]
            );
        }
        other => panic!("expected subscribed frame, got {other:?}"),
    }

    match connection
        .next_frame()
        .await
        .expect("message frame")
        .expect("some frame")
    {
        MessagesWsServerFrame::Message(frame) => {
            assert_eq!(frame.id, "rule_1");
            assert_eq!(frame.tf, "1m");
            assert_eq!(frame.close_ms, 1774693020000);
            assert_eq!(frame.message, "rule triggered");
        }
        other => panic!("expected message frame, got {other:?}"),
    }

    match connection
        .next_frame()
        .await
        .expect("heartbeat frame")
        .expect("some frame")
    {
        MessagesWsServerFrame::Heartbeat(frame) => {
            assert_eq!(frame.at_ms, 1774693080000);
            assert_eq!(frame.watermark_end_ms, None);
            assert_eq!(frame.subscription_ids, vec!["rule_1".to_string()]);
        }
        other => panic!("expected heartbeat frame, got {other:?}"),
    }

    match connection
        .next_frame()
        .await
        .expect("error frame")
        .expect("some frame")
    {
        MessagesWsServerFrame::Error(frame) => {
            assert_eq!(frame.id.as_deref(), Some("rule_2"));
            assert_eq!(frame.kind, "invalid_request");
            assert_eq!(frame.error, "unknown subscription id");
        }
        other => panic!("expected error frame, got {other:?}"),
    }
}

#[tokio::test]
async fn test_connect_messages_ws_missing_config_is_typed_error() {
    let client = AggregatorClient::new(AggregatorConfig {
        http: Some(HttpTransportConfig::new("http://127.0.0.1:1").expect("valid dummy http url")),
        grpc: None,
        ws: None,
        bearer_token: None,
    })
    .expect("aggregator client");

    let error = client
        .connect_messages_ws()
        .await
        .expect_err("missing ws config must fail");

    match error {
        crate::core::error::SdkError::MissingTransportConfig { transport } => {
            assert_eq!(transport, "ws");
        }
        other => panic!("expected missing transport config, got {other:?}"),
    }
}

#[tokio::test]
async fn test_connect_messages_ws_recovering_replays_active_subscriptions_after_close() {
    let (base_url, captured_rx) = spawn_recovering_messages_ws_server().await;
    let client = AggregatorClient::new(config_for_ws(&base_url, None)).expect("aggregator client");

    let mut connection = client
        .connect_messages_ws_recovering(ExponentialBackoffConfig {
            initial_delay: std::time::Duration::from_millis(1),
            multiplier: 2,
            max_delay: std::time::Duration::from_millis(5),
            max_attempts: Some(3),
            jitter_ratio: 0.0,
        })
        .await
        .expect("connect recovering messages ws");

    connection
        .send_subscribe(&MessagesWsSubscribeFrame {
            id: "rule_1".to_string(),
            tfs: Some(vec![Timeframe::M1]),
            predicate: "BTCUSDT.c > 0".to_string(),
            message: "recover me".to_string(),
            payload: None,
        })
        .await
        .expect("send subscribe");

    match connection
        .next_frame()
        .await
        .expect("replayed subscribed frame")
        .expect("some frame")
    {
        MessagesWsServerFrame::Subscribed(frame) => {
            assert_eq!(frame.id, "rule_1");
            assert_eq!(frame.tfs, vec!["1m".to_string()]);
        }
        other => panic!("expected subscribed frame after reconnect, got {other:?}"),
    }

    let captured = captured_rx
        .await
        .expect("captured recovering messages frames");
    assert_eq!(captured.len(), 2);

    let first: serde_json::Value =
        serde_json::from_str(&captured[0]).expect("first subscribe json");
    let second: serde_json::Value =
        serde_json::from_str(&captured[1]).expect("second subscribe json");
    assert_eq!(first, second);
    assert_eq!(connection.active_subscription_ids(), vec!["rule_1"]);
    assert_eq!(connection.next_attempt(), 1);
}
