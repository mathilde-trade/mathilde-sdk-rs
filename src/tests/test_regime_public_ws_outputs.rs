use crate::core::auth::BearerToken;
use crate::core::config::{HttpTransportConfig, RegimeConfig, WsTransportConfig};
use crate::systems::regime::{
    OutputsWsFormat, OutputsWsInboundFrame, OutputsWsPhase, OutputsWsSubscribeRequest, Regime,
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
struct CapturedOutputsConnect {
    path: String,
    authorization: Option<String>,
    subscribe_text: String,
}

fn config_for_ws(base_url: &str, bearer_token: Option<BearerToken>) -> RegimeConfig {
    RegimeConfig {
        http: HttpTransportConfig::new("http://127.0.0.1:1").expect("valid dummy http url"),
        grpc: None,
        ws: Some(WsTransportConfig::new(base_url).expect("valid ws url")),
        bearer_token,
    }
}

fn meta_frame(close_ms: i64, phase: OutputsWsPhase) -> String {
    serde_json::json!({
        "tf": "1h",
        "close_ms": close_ms,
        "watermark_end_ms": close_ms,
        "phase": phase,
        "missing_pairs": []
    })
    .to_string()
}

async fn spawn_outputs_ws_server() -> (String, oneshot::Receiver<CapturedOutputsConnect>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind outputs ws test server");
    let addr = listener.local_addr().expect("outputs ws test addr");
    let (captured_tx, captured_rx) = oneshot::channel();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept outputs ws conn");
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
        .expect("accept outputs ws handshake");

        let subscribe_text = match ws.next().await {
            Some(Ok(Message::Text(text))) => text.to_string(),
            other => panic!("expected subscribe text, got {other:?}"),
        };
        let (path, authorization) = capture
            .lock()
            .expect("capture mutex")
            .take()
            .expect("captured request");
        let _ = captured_tx.send(CapturedOutputsConnect {
            path,
            authorization,
            subscribe_text,
        });

        ws.send(Message::Text(
            meta_frame(1770003600000, OutputsWsPhase::Live).into(),
        ))
        .await
        .expect("send meta");
        ws.send(Message::Text(
            serde_json::json!([{
                "pair": "BTCUSDT",
                "tf": "1h",
                "open_ms": 1770000000000_i64,
                "close_ms": 1770003600000_i64,
                "open_utc": "2026-02-02T00:00:00Z",
                "close_utc": "2026-02-02T01:00:00Z",
                "o": 100.0,
                "h": 101.0,
                "l": 99.5,
                "c": 100.5,
                "v": 12.34,
                "tr_klts_score": 0.75,
                "age_ms": 10
            }])
            .to_string()
            .into(),
        ))
        .await
        .expect("send rows");
    });

    (format!("http://{addr}"), captured_rx)
}

#[tokio::test]
async fn test_regime_connect_outputs_ws_sends_auth_secondary_and_decodes_json_frames() {
    let (base_url, captured_rx) = spawn_outputs_ws_server().await;
    let client = Regime::new(config_for_ws(
        &base_url,
        Some(BearerToken::new("feed_public_token").expect("valid token")),
    ))
    .expect("regime client");

    let request = OutputsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::H1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        secondary: Some(false),
        from_close: None,
        last_n_bars: Some(1),
        format: Some(OutputsWsFormat::Json),
    };

    let mut connection = client
        .connect_outputs_ws(&request)
        .await
        .expect("connect outputs ws");

    let meta = connection
        .next_frame(&request)
        .await
        .expect("next frame")
        .expect("meta frame");
    let rows = connection
        .next_frame(&request)
        .await
        .expect("next frame")
        .expect("rows frame");
    let captured = captured_rx.await.expect("captured outputs ws request");
    let subscribe_json: serde_json::Value =
        serde_json::from_str(&captured.subscribe_text).expect("subscribe json");

    assert_eq!(captured.path, "/v1/ws/outputs");
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer feed_public_token")
    );
    assert_eq!(subscribe_json["pairs"], serde_json::json!(["BTCUSDT"]));
    assert_eq!(subscribe_json["tfs"], serde_json::json!(["1h"]));
    assert_eq!(subscribe_json["secondary"], false);
    assert_eq!(subscribe_json["format"], "json");

    match meta {
        OutputsWsInboundFrame::Meta(frame) => {
            assert_eq!(frame.phase, OutputsWsPhase::Live);
            assert_eq!(frame.close_ms, Some(1770003600000));
        }
        other => panic!("expected meta frame, got {other:?}"),
    }

    match rows {
        OutputsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows[0].row.pair, "BTCUSDT");
            assert_eq!(rows[0].row.computed.f64("tr_klts_score"), Some(0.75));
            assert_eq!(rows[0].age_ms, 10);
        }
        other => panic!("expected json rows, got {other:?}"),
    }
}
