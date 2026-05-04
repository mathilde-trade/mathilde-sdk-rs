use crate::core::auth::BearerToken;
use crate::core::config::{HttpTransportConfig, PrimitivesConfig, WsTransportConfig};
use crate::streaming::subscription::ExponentialBackoffConfig;
use crate::systems::primitives::{
    MessagesWsServerFrame, MessagesWsSubscribeFrame, MessagesWsUnsubscribeFrame, Primitives,
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

fn config_for_ws(base_url: &str, bearer_token: Option<BearerToken>) -> PrimitivesConfig {
    PrimitivesConfig {
        http: HttpTransportConfig::new("http://127.0.0.1:1").expect("valid dummy http url"),
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
    let client = Primitives::new(config_for_ws(
        &base_url,
        Some(BearerToken::new("feed_public_token").expect("valid token")),
    ))
    .expect("primitives client");

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

    let subscribed = connection
        .next_frame()
        .await
        .expect("subscribed frame")
        .expect("subscribed message");
    let message = connection
        .next_frame()
        .await
        .expect("message frame")
        .expect("message");
    let captured = captured_rx.await.expect("captured ws request");
    let subscribe_json: serde_json::Value =
        serde_json::from_str(&captured.subscribe_text).expect("subscribe json");
    let unsubscribe_json: serde_json::Value =
        serde_json::from_str(&captured.unsubscribe_text).expect("unsubscribe json");

    assert_eq!(captured.path, "/v1/ws/messages");
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer feed_public_token")
    );
    assert_eq!(subscribe_json["type"], "subscribe");
    assert_eq!(subscribe_json["id"], "rule_1");
    assert_eq!(
        unsubscribe_json,
        serde_json::json!({"type":"unsubscribe","id":"rule_1"})
    );

    match subscribed {
        MessagesWsServerFrame::Subscribed(frame) => {
            assert_eq!(frame.id, "rule_1");
            assert_eq!(frame.tfs, vec!["1m".to_string(), "5m".to_string()]);
        }
        other => panic!("expected subscribed frame, got {other:?}"),
    }

    match message {
        MessagesWsServerFrame::Message(frame) => {
            assert_eq!(frame.id, "rule_1");
            assert_eq!(frame.message, "rule triggered");
            assert_eq!(frame.payload, Some(serde_json::json!({"strategy":"alpha"})));
        }
        other => panic!("expected message frame, got {other:?}"),
    }
}

#[tokio::test]
async fn test_recovering_messages_ws_replays_active_subscriptions_after_disconnect() {
    let (base_url, captured_rx) = spawn_recovering_messages_ws_server().await;
    let client = Primitives::new(config_for_ws(&base_url, None)).expect("primitives client");

    let mut connection = client
        .connect_messages_ws_recovering(ExponentialBackoffConfig::default())
        .await
        .expect("connect recovering messages ws");

    connection
        .send_subscribe(&MessagesWsSubscribeFrame {
            id: "rule_1".to_string(),
            tfs: Some(vec![Timeframe::M1]),
            predicate: "BTCUSDT.c > 0".to_string(),
            message: "rule triggered".to_string(),
            payload: None,
        })
        .await
        .expect("send subscribe");

    let frame = connection
        .next_frame()
        .await
        .expect("next recovering frame")
        .expect("subscribed frame");
    let captured = captured_rx.await.expect("captured replayed subscriptions");

    assert_eq!(captured.len(), 2);
    assert_eq!(captured[0], captured[1]);

    match frame {
        MessagesWsServerFrame::Subscribed(frame) => {
            assert_eq!(frame.id, "rule_1");
            assert_eq!(frame.tfs, vec!["1m".to_string()]);
        }
        other => panic!("expected subscribed frame, got {other:?}"),
    }
}
