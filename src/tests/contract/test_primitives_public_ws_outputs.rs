use crate::core::auth::BearerToken;
use crate::core::config::{HttpTransportConfig, PrimitivesConfig, WsTransportConfig};
use crate::streaming::make_before_break::MakeBeforeBreakConfig;
use crate::streaming::subscription::ExponentialBackoffConfig;
use crate::systems::primitives::{
    OutputsWsFormat, OutputsWsInboundFrame, OutputsWsMakeBeforeBreak, OutputsWsPhase,
    OutputsWsSubscribeRequest, Primitives,
};
use crate::systems::types::Timeframe;
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::time::sleep;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

#[derive(Debug)]
struct CapturedOutputsConnect {
    path: String,
    authorization: Option<String>,
    subscribe_text: String,
}

fn config_for_ws(base_url: &str, bearer_token: Option<BearerToken>) -> PrimitivesConfig {
    PrimitivesConfig {
        http: HttpTransportConfig::new("http://127.0.0.1:1").expect("valid dummy http url"),
        grpc: None,
        ws: Some(WsTransportConfig::new(base_url).expect("valid ws url")),
        bearer_token,
    }
}

fn meta_frame(close_ms: i64, phase: OutputsWsPhase) -> String {
    serde_json::json!({
        "tf": "1m",
        "close_ms": close_ms,
        "watermark_end_ms": close_ms,
        "phase": phase,
        "missing_pairs": []
    })
    .to_string()
}

fn replay_done_frame(close_ms: i64) -> String {
    serde_json::json!({
        "tf": "1m",
        "close_ms": close_ms,
        "watermark_end_ms": close_ms,
        "phase": "replay",
        "missing_pairs": [],
        "event": "replay_done"
    })
    .to_string()
}

fn json_rows_frame(pair: &str, close_ms: i64) -> String {
    serde_json::json!([{
        "pair": pair,
        "tf": "1m",
        "open_ms": close_ms - 60_000,
        "close_ms": close_ms,
        "open_utc": "2026-02-02T00:00:00Z",
        "close_utc": "2026-02-02T00:01:00Z",
        "o": 100.0,
        "h": 101.0,
        "l": 99.5,
        "c": 100.5,
        "v": 12.34,
        "bs_close_window_min": 0.75,
        "age_ms": 10
    }])
    .to_string()
}

fn json_replay_rows_frame(pair: &str, close_ms: i64) -> String {
    serde_json::json!([{
        "pair": pair,
        "tf": "1m",
        "open_ms": close_ms - 60_000,
        "close_ms": close_ms,
        "open_utc": "2026-02-02T00:00:00Z",
        "close_utc": "2026-02-02T00:01:00Z",
        "o": 100.0,
        "h": 101.0,
        "l": 99.5,
        "c": 100.5,
        "v": 12.34,
        "bs_close_window_min": 0.75
    }])
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
            meta_frame(1770000060000, OutputsWsPhase::Live).into(),
        ))
        .await
        .expect("send meta");
        ws.send(Message::Text(
            json_rows_frame("BTCUSDT", 1770000060000).into(),
        ))
        .await
        .expect("send rows");
    });

    (format!("http://{addr}"), captured_rx)
}

async fn spawn_replay_outputs_ws_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind replay outputs ws test server");
    let addr = listener.local_addr().expect("replay outputs ws test addr");

    tokio::spawn(async move {
        let (stream, _) = listener
            .accept()
            .await
            .expect("accept replay outputs ws conn");
        let mut ws = accept_hdr_async(stream, |_request: &Request, response: Response| {
            Ok(response)
        })
        .await
        .expect("accept replay outputs ws handshake");

        let _ = ws.next().await;
        ws.send(Message::Text(
            meta_frame(1770000060000, OutputsWsPhase::Replay).into(),
        ))
        .await
        .expect("send replay meta");
        ws.send(Message::Text(
            json_replay_rows_frame("BTCUSDT", 1770000060000).into(),
        ))
        .await
        .expect("send replay rows");
        ws.send(Message::Text(replay_done_frame(1770000060000).into()))
            .await
            .expect("send replay_done");
    });

    format!("http://{addr}")
}

async fn spawn_make_before_break_outputs_ws_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind outputs mbb ws test server");
    let addr = listener.local_addr().expect("outputs mbb ws test addr");

    tokio::spawn(async move {
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.expect("accept outputs mbb ws conn");
            tokio::spawn(async move {
                let mut ws = accept_hdr_async(stream, |_request: &Request, response: Response| {
                    Ok(response)
                })
                .await
                .expect("accept outputs mbb handshake");

                let subscribe_text = match ws.next().await {
                    Some(Ok(Message::Text(text))) => text.to_string(),
                    other => panic!("expected subscribe text, got {other:?}"),
                };
                let subscribe_json: serde_json::Value =
                    serde_json::from_str(&subscribe_text).expect("subscribe json");
                let pair = subscribe_json["pairs"][0]
                    .as_str()
                    .expect("pair in subscribe")
                    .to_string();

                if pair == "BTCUSDT" {
                    sleep(Duration::from_millis(5)).await;
                    ws.send(Message::Text(json_rows_frame("BTCUSDT", 1).into()))
                        .await
                        .expect("send old rows");
                    sleep(Duration::from_millis(100)).await;
                } else {
                    sleep(Duration::from_millis(35)).await;
                    ws.send(Message::Text(json_rows_frame("ETHUSDT", 2).into()))
                        .await
                        .expect("send new rows");
                    sleep(Duration::from_millis(100)).await;
                }
            });
        }
    });

    format!("http://{addr}")
}

async fn spawn_recovering_outputs_ws_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind recovering outputs ws test server");
    let addr = listener
        .local_addr()
        .expect("recovering outputs ws test addr");

    tokio::spawn(async move {
        for close_ms in [10_i64, 20_i64] {
            let (stream, _) = listener
                .accept()
                .await
                .expect("accept recovering outputs ws conn");
            tokio::spawn(async move {
                let mut ws = accept_hdr_async(stream, |_request: &Request, response: Response| {
                    Ok(response)
                })
                .await
                .expect("accept recovering outputs ws handshake");

                let _ = ws.next().await;
                ws.send(Message::Text(json_rows_frame("BTCUSDT", close_ms).into()))
                    .await
                    .expect("send recovering rows");
                let _ = ws.close(None).await;
            });
        }
    });

    format!("http://{addr}")
}

#[tokio::test]
async fn test_connect_outputs_ws_sends_auth_and_decodes_json_frames() {
    let (base_url, captured_rx) = spawn_outputs_ws_server().await;
    let client = Primitives::new(config_for_ws(
        &base_url,
        Some(BearerToken::new("feed_public_token").expect("valid token")),
    ))
    .expect("primitives client");

    let request = OutputsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
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
    assert_eq!(subscribe_json["tfs"], serde_json::json!(["1m"]));
    assert_eq!(subscribe_json["metadata"], false);
    assert_eq!(subscribe_json["diagnostics"], false);
    assert_eq!(subscribe_json["last_n_bars"], 1);
    assert_eq!(subscribe_json["format"], "json");

    match meta {
        OutputsWsInboundFrame::Meta(frame) => {
            assert_eq!(frame.phase, OutputsWsPhase::Live);
            assert_eq!(frame.close_ms, Some(1770000060000));
        }
        other => panic!("expected meta frame, got {other:?}"),
    }

    match rows {
        OutputsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows[0].row.pair, "BTCUSDT");
            assert_eq!(rows[0].age_ms, 10);
        }
        other => panic!("expected json rows, got {other:?}"),
    }
}

#[tokio::test]
async fn test_connect_outputs_ws_replay_last_n_bars_yields_rows_then_replay_done() {
    let base_url = spawn_replay_outputs_ws_server().await;
    let client = Primitives::new(config_for_ws(&base_url, None)).expect("primitives client");

    let request = OutputsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        from_close: None,
        last_n_bars: Some(1),
        format: Some(OutputsWsFormat::Json),
    };

    let mut connection = client
        .connect_outputs_ws(&request)
        .await
        .expect("connect replay outputs ws");

    match connection
        .next_frame(&request)
        .await
        .expect("replay meta")
        .expect("some frame")
    {
        OutputsWsInboundFrame::Meta(frame) => {
            assert_eq!(frame.phase, OutputsWsPhase::Replay);
            assert_eq!(frame.close_ms, Some(1770000060000));
        }
        other => panic!("expected replay meta frame, got {other:?}"),
    }

    match connection
        .next_frame(&request)
        .await
        .expect("replay rows")
        .expect("some frame")
    {
        OutputsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows[0].row.pair, "BTCUSDT");
            assert_eq!(rows[0].age_ms, 0);
        }
        other => panic!("expected replay rows frame, got {other:?}"),
    }

    match connection
        .next_frame(&request)
        .await
        .expect("replay_done")
        .expect("some frame")
    {
        OutputsWsInboundFrame::Meta(frame) => assert!(frame.is_replay_done()),
        other => panic!("expected replay_done frame, got {other:?}"),
    }
}

#[tokio::test]
async fn test_outputs_ws_make_before_break_keeps_old_rows_until_new_rows_arrive() {
    let base_url = spawn_make_before_break_outputs_ws_server().await;
    let config = config_for_ws(&base_url, None);
    let ws_config = config.ws.as_ref().expect("ws config");
    let transport = crate::transport::ws::WsTransport::new(ws_config, None);

    let old_request = OutputsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        from_close: None,
        last_n_bars: Some(1),
        format: Some(OutputsWsFormat::Json),
    };
    let new_request = OutputsWsSubscribeRequest {
        pairs: vec!["ETHUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        from_close: None,
        last_n_bars: Some(1),
        format: Some(OutputsWsFormat::Json),
    };

    let mut connection = OutputsWsMakeBeforeBreak::connect(
        &transport,
        &old_request,
        MakeBeforeBreakConfig {
            validation_window: Duration::from_millis(20),
        },
    )
    .await
    .expect("connect outputs mbb");

    connection.begin_swap(&new_request).expect("begin swap");
    assert!(connection.swap_in_progress());

    match connection
        .next_frame()
        .await
        .expect("old rows during validation")
        .expect("some frame")
    {
        OutputsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows[0].row.pair, "BTCUSDT");
            assert_eq!(rows[0].row.close_ms, 1);
        }
        other => panic!("expected old rows frame, got {other:?}"),
    }
    assert_eq!(
        connection.active_request().pairs,
        vec!["BTCUSDT".to_string()]
    );

    sleep(Duration::from_millis(35)).await;

    match connection
        .next_frame()
        .await
        .expect("new rows after promotion")
        .expect("some frame")
    {
        OutputsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows[0].row.pair, "ETHUSDT");
            assert_eq!(rows[0].row.close_ms, 2);
        }
        other => panic!("expected new rows frame, got {other:?}"),
    }
    assert_eq!(
        connection.active_request().pairs,
        vec!["ETHUSDT".to_string()]
    );
}

#[tokio::test]
async fn test_connect_outputs_ws_recovering_reconnects_with_same_request_after_close_and_rows() {
    let base_url = spawn_recovering_outputs_ws_server().await;
    let client = Primitives::new(config_for_ws(&base_url, None)).expect("primitives client");

    let request = OutputsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        from_close: None,
        last_n_bars: Some(1),
        format: Some(OutputsWsFormat::Json),
    };

    let mut connection = client
        .connect_outputs_ws_recovering(
            &request,
            ExponentialBackoffConfig {
                initial_delay: Duration::from_millis(1),
                multiplier: 2,
                max_delay: Duration::from_millis(5),
                max_attempts: Some(3),
                jitter_ratio: 0.0,
            },
        )
        .await
        .expect("connect recovering outputs ws");

    match connection
        .next_frame()
        .await
        .expect("first recovering rows")
        .expect("some frame")
    {
        OutputsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows[0].row.pair, "BTCUSDT");
            assert_eq!(rows[0].row.close_ms, 10);
        }
        other => panic!("expected first rows frame, got {other:?}"),
    }

    match connection
        .next_frame()
        .await
        .expect("second recovering rows")
        .expect("some frame")
    {
        OutputsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows[0].row.pair, "BTCUSDT");
            assert_eq!(rows[0].row.close_ms, 20);
        }
        other => panic!("expected second rows frame, got {other:?}"),
    }

    assert_eq!(connection.active_request(), &request);
    assert_eq!(connection.next_attempt(), 1);
}
