use crate::core::auth::BearerToken;
use crate::core::config::{AggregatorConfig, HttpTransportConfig, WsTransportConfig};
use crate::core::error::SdkError;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1 as proto;
use crate::streaming::make_before_break::MakeBeforeBreakConfig;
use crate::streaming::subscription::ExponentialBackoffConfig;
use crate::systems::aggregator::BarsWsMakeBeforeBreak;
use crate::systems::aggregator::{
    Aggregator, BarsWsInboundFrame, BarsWsMetaFrame, BarsWsPhase, BarsWsSubscribeRequest,
};
use crate::systems::types::Timeframe;
use futures_util::{SinkExt, StreamExt};
use prost::Message as ProstMessage;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::time::sleep;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

#[derive(Debug)]
struct CapturedWsConnect {
    path: String,
    authorization: Option<String>,
    subscribe_text: String,
}

fn config_for_ws(base_url: &str, bearer_token: Option<BearerToken>) -> AggregatorConfig {
    AggregatorConfig {
        http: HttpTransportConfig::new("http://127.0.0.1:1").expect("valid dummy http url"),
        grpc: None,
        ws: Some(WsTransportConfig::new(base_url).expect("valid ws url")),
        bearer_token,
    }
}

fn meta_frame(close_ms: i64, phase: BarsWsPhase) -> String {
    serde_json::to_string(&BarsWsMetaFrame {
        tf: Some("1m".to_string()),
        close_ms: Some(close_ms),
        watermark_end_ms: close_ms,
        phase,
        missing_pairs: Vec::new(),
        event: None,
    })
    .expect("meta frame json")
}

fn replay_done_frame(close_ms: i64) -> String {
    serde_json::to_string(&BarsWsMetaFrame {
        tf: Some("1m".to_string()),
        close_ms: Some(close_ms),
        watermark_end_ms: close_ms,
        phase: BarsWsPhase::Replay,
        missing_pairs: Vec::new(),
        event: Some("replay_done".to_string()),
    })
    .expect("replay_done frame json")
}

fn json_min_rows_frame(pair: &str, close_ms: i64) -> String {
    serde_json::json!([{
        "pair": pair,
        "tf": "1m",
        "open_ms": close_ms - 60_000,
        "close_ms": close_ms,
        "open_utc": "2026-02-02T00:00:00Z",
        "close_utc": "2026-02-02T00:01:00Z",
        "o": 100.0,
        "h": 101.0,
        "l": 99.0,
        "c": 100.5,
        "v": 12.0,
        "quote_v": null,
        "taker_known_v": null,
        "taker_signed_v": null,
        "taker_known_quote_v": null,
        "taker_signed_quote_v": null,
        "taker_known_n": null,
        "taker_signed_n": null,
        "vw": null,
        "n": 1
    }])
    .to_string()
}

fn proto_full_payload(pair: &str) -> Vec<u8> {
    proto::BarsRowsPayloadV1 {
        view: proto::BarsViewV1::Full as i32,
        rows: vec![proto::BarsPresentRowV1 {
            bar: Some(proto::BarRowV1 {
                pair: pair.to_string(),
                tf: "1m".to_string(),
                s_ms: 1770000000000,
                e_ms: 1770000060000,
                s_utc: Some("2026-02-02T00:00:00Z".to_string()),
                e_utc: Some("2026-02-02T00:01:00Z".to_string()),
                o: 100.0,
                h: 101.0,
                l: 99.0,
                c: 100.5,
                v: 12.0,
                quote_v: None,
                taker_known_v: None,
                taker_signed_v: None,
                taker_known_quote_v: None,
                taker_signed_quote_v: None,
                taker_known_n: None,
                taker_signed_n: None,
                vw: None,
                n: Some(1),
                coverage_ratio: None,
                at_ms: None,
                metadata: Some(proto::BarMetadataV1 {
                    source: "frontier".to_string(),
                    process: None,
                    venues_expected: vec!["binance".to_string()],
                    venues_with_trades: vec!["binance".to_string()],
                    ingested_at_ms: None,
                    ingested_at_utc: None,
                    target_ingested_at_ms: None,
                    target_ingested_at_utc: None,
                    built_at_ms: None,
                    built_at_utc: None,
                    committed_at_ms: None,
                    committed_at_utc: None,
                    harmonized_at_ms: None,
                    harmonized_at_utc: None,
                    recomputed_at_ms: None,
                    recomputed_at_utc: None,
                    recomputed_reason: None,
                    covered_1m_count: None,
                    expected_1m_count: None,
                    coverage_ratio: Some(0.95),
                    inputs_source_counts_frontier: None,
                    inputs_source_counts_api: None,
                    inputs_source_counts_synthetic: None,
                    inputs_source_counts_fix_data: None,
                    frontier_5s_inputs_coverage_ratio: None,
                    frontier_5s_expected: None,
                    frontier_5s_synth_n: None,
                    frontier_5s_synth_ratio: None,
                    frontier_5s_trade_n: None,
                    frontier_5s_trade_ratio: None,
                }),
            }),
            age_ms: Some(10),
        }],
    }
    .encode_to_vec()
}

async fn spawn_capture_ws_server() -> (String, oneshot::Receiver<CapturedWsConnect>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ws test server");
    let addr = listener.local_addr().expect("ws test addr");
    let (captured_tx, captured_rx) = oneshot::channel();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept ws test conn");
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
        .expect("accept ws handshake");

        let subscribe_text = match ws.next().await {
            Some(Ok(Message::Text(text))) => text.to_string(),
            other => panic!("expected subscribe text, got {other:?}"),
        };
        let (path, authorization) = capture
            .lock()
            .expect("capture mutex")
            .take()
            .expect("captured request");
        let _ = captured_tx.send(CapturedWsConnect {
            path,
            authorization,
            subscribe_text,
        });

        ws.send(Message::Text(
            meta_frame(1770000060000, BarsWsPhase::Live).into(),
        ))
        .await
        .expect("send meta");
        ws.send(Message::Text(
            serde_json::json!([{
                "pair": "BTCUSDT",
                "tf": "1m",
                "open_ms": 1770000000000_i64,
                "close_ms": 1770000060000_i64,
                "open_utc": "2026-02-02T00:00:00Z",
                "close_utc": "2026-02-02T00:01:00Z",
                "o": 100.0,
                "h": 101.0,
                "l": 99.0,
                "c": 100.5,
                "v": 12.0,
                "quote_v": null,
                "taker_known_v": null,
                "taker_signed_v": null,
                "taker_known_quote_v": null,
                "taker_signed_quote_v": null,
                "taker_known_n": null,
                "taker_signed_n": null,
                "vw": null,
                "n": 1
            }])
            .to_string()
            .into(),
        ))
        .await
        .expect("send rows");
    });

    (format!("http://{addr}"), captured_rx)
}

async fn spawn_protobuf_ws_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind protobuf ws test server");
    let addr = listener.local_addr().expect("protobuf ws test addr");

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept protobuf ws conn");
        let mut ws = accept_hdr_async(stream, |_request: &Request, response: Response| {
            Ok(response)
        })
        .await
        .expect("accept ws handshake");

        let _ = ws.next().await;
        ws.send(Message::Binary(proto_full_payload("BTCUSDT").into()))
            .await
            .expect("send protobuf payload");
    });

    format!("http://{addr}")
}

async fn spawn_replay_bars_ws_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind replay bars ws test server");
    let addr = listener.local_addr().expect("replay bars ws test addr");

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept replay bars ws conn");
        let mut ws = accept_hdr_async(stream, |_request: &Request, response: Response| {
            Ok(response)
        })
        .await
        .expect("accept replay bars handshake");

        let _ = ws.next().await;
        ws.send(Message::Text(
            meta_frame(1770000060000, BarsWsPhase::Replay).into(),
        ))
        .await
        .expect("send replay meta");
        ws.send(Message::Text(
            json_min_rows_frame("BTCUSDT", 1770000060000).into(),
        ))
        .await
        .expect("send replay rows");
        ws.send(Message::Text(replay_done_frame(1770000060000).into()))
            .await
            .expect("send replay_done");
    });

    format!("http://{addr}")
}

async fn spawn_make_before_break_ws_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mbb ws test server");
    let addr = listener.local_addr().expect("mbb ws test addr");

    tokio::spawn(async move {
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.expect("accept mbb ws conn");
            tokio::spawn(async move {
                let mut ws = accept_hdr_async(stream, |_request: &Request, response: Response| {
                    Ok(response)
                })
                .await
                .expect("accept mbb handshake");

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
                    ws.send(Message::Text(json_min_rows_frame("BTCUSDT", 1).into()))
                        .await
                        .expect("send old rows");
                } else {
                    sleep(Duration::from_millis(35)).await;
                    ws.send(Message::Text(json_min_rows_frame("ETHUSDT", 2).into()))
                        .await
                        .expect("send new rows");
                }
            });
        }
    });

    format!("http://{addr}")
}

async fn spawn_recovering_bars_ws_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind recovering bars ws test server");
    let addr = listener.local_addr().expect("recovering bars ws test addr");

    tokio::spawn(async move {
        for close_ms in [10_i64, 20_i64] {
            let (stream, _) = listener
                .accept()
                .await
                .expect("accept recovering bars ws conn");
            tokio::spawn(async move {
                let mut ws = accept_hdr_async(stream, |_request: &Request, response: Response| {
                    Ok(response)
                })
                .await
                .expect("accept recovering bars handshake");

                let _ = ws.next().await;
                ws.send(Message::Text(
                    json_min_rows_frame("BTCUSDT", close_ms).into(),
                ))
                .await
                .expect("send recovering bars rows");
                let _ = ws.close(None).await;
            });
        }
    });

    format!("http://{addr}")
}

#[tokio::test]
async fn test_connect_bars_ws_sends_auth_and_subscribe_and_decodes_json_min() {
    let (base_url, captured_rx) = spawn_capture_ws_server().await;
    let client = Aggregator::new(config_for_ws(
        &base_url,
        Some(BearerToken::new("feed_public_token").expect("valid token")),
    ))
    .expect("aggregator client");

    let request = BarsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        from_close: None,
        last_n_bars: Some(10),
        format: None,
    };

    let mut connection = client
        .connect_bars_ws(&request)
        .await
        .expect("connect bars ws");

    let captured = captured_rx.await.expect("captured ws connect");
    assert_eq!(captured.path, "/v1/ws/bars");
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer feed_public_token")
    );

    let subscribe_json: serde_json::Value =
        serde_json::from_str(&captured.subscribe_text).expect("subscribe json");
    assert_eq!(
        subscribe_json["pairs"],
        serde_json::json!(["BTCUSDT", "ETHUSDT"])
    );
    assert_eq!(subscribe_json["tfs"], serde_json::json!(["1m"]));
    assert_eq!(subscribe_json["metadata"], serde_json::json!(false));
    assert_eq!(subscribe_json["last_n_bars"], serde_json::json!(10));
    assert_eq!(subscribe_json["format"], serde_json::json!("json"));

    let meta = connection
        .next_frame(&request)
        .await
        .expect("meta frame")
        .expect("some meta frame");
    match meta {
        BarsWsInboundFrame::Meta(frame) => {
            assert_eq!(frame.phase, BarsWsPhase::Live);
            assert_eq!(frame.close_ms, Some(1770000060000));
        }
        other => panic!("expected meta frame, got {other:?}"),
    }

    let rows = connection
        .next_frame(&request)
        .await
        .expect("rows frame")
        .expect("some rows frame");
    match rows {
        BarsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].pair, "BTCUSDT");
            assert_eq!(rows[0].close_ms, 1770000060000);
            assert!(rows[0].metadata.is_none());
        }
        other => panic!("expected json min rows, got {other:?}"),
    }
}

#[tokio::test]
async fn test_connect_bars_ws_decodes_protobuf_full_rows() {
    let base_url = spawn_protobuf_ws_server().await;
    let client = Aggregator::new(config_for_ws(&base_url, None)).expect("aggregator client");

    let request = BarsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(true),
        from_close: None,
        last_n_bars: None,
        format: Some(crate::systems::aggregator::BarsWsFormat::Protobuf),
    };

    let mut connection = client
        .connect_bars_ws(&request)
        .await
        .expect("connect protobuf bars ws");

    let frame = connection
        .next_frame(&request)
        .await
        .expect("protobuf frame")
        .expect("some protobuf frame");

    match frame {
        BarsWsInboundFrame::ProtobufRows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].pair, "BTCUSDT");
            assert_eq!(rows[0].age_ms, Some(10));
            let metadata = rows[0].metadata.as_ref().expect("metadata");
            assert_eq!(metadata.source, "frontier");
            assert_eq!(metadata.coverage_ratio, Some(0.95));
        }
        other => panic!("expected protobuf full rows, got {other:?}"),
    }
}

#[tokio::test]
async fn test_connect_bars_ws_replay_last_n_bars_yields_rows_then_replay_done() {
    let base_url = spawn_replay_bars_ws_server().await;
    let client = Aggregator::new(config_for_ws(&base_url, None)).expect("aggregator client");

    let request = BarsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        from_close: None,
        last_n_bars: Some(1),
        format: None,
    };

    let mut connection = client
        .connect_bars_ws(&request)
        .await
        .expect("connect replay bars ws");

    match connection
        .next_frame(&request)
        .await
        .expect("replay meta")
        .expect("some frame")
    {
        BarsWsInboundFrame::Meta(frame) => {
            assert_eq!(frame.phase, BarsWsPhase::Replay);
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
        BarsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].pair, "BTCUSDT");
            assert_eq!(rows[0].close_ms, 1770000060000);
        }
        other => panic!("expected replay rows frame, got {other:?}"),
    }

    match connection
        .next_frame(&request)
        .await
        .expect("replay_done")
        .expect("some frame")
    {
        BarsWsInboundFrame::Meta(frame) => assert!(frame.is_replay_done()),
        other => panic!("expected replay_done frame, got {other:?}"),
    }
}

#[tokio::test]
async fn test_bars_ws_make_before_break_keeps_old_until_new_is_stable_then_swaps() {
    let base_url = spawn_make_before_break_ws_server().await;
    let config = config_for_ws(&base_url, None);
    let ws_config = config.ws.as_ref().expect("ws config");
    let transport = crate::transport::ws::WsTransport::new(ws_config, None);

    let old_request = BarsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        from_close: None,
        last_n_bars: None,
        format: None,
    };
    let new_request = BarsWsSubscribeRequest {
        pairs: vec!["ETHUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        from_close: None,
        last_n_bars: None,
        format: None,
    };

    let mut mbb = BarsWsMakeBeforeBreak::connect(
        &transport,
        &old_request,
        MakeBeforeBreakConfig {
            validation_window: Duration::from_millis(20),
        },
    )
    .await
    .expect("connect mbb");

    mbb.begin_swap(&new_request).expect("begin swap");
    assert!(mbb.swap_in_progress());

    let first = mbb
        .next_frame()
        .await
        .expect("old frame during validation")
        .expect("some frame");
    match first {
        BarsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].pair, "BTCUSDT");
            assert_eq!(rows[0].close_ms, 1);
        }
        other => panic!("expected old rows frame, got {other:?}"),
    }
    assert_eq!(mbb.active_request().pairs, vec!["BTCUSDT".to_string()]);

    sleep(Duration::from_millis(35)).await;

    let second = mbb
        .next_frame()
        .await
        .expect("new frame after promotion")
        .expect("some frame");
    match second {
        BarsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].pair, "ETHUSDT");
            assert_eq!(rows[0].close_ms, 2);
        }
        other => panic!("expected new rows frame, got {other:?}"),
    }
    assert_eq!(mbb.active_request().pairs, vec!["ETHUSDT".to_string()]);
}

#[tokio::test]
async fn test_connect_bars_ws_recovering_reconnects_with_same_request_after_close() {
    let base_url = spawn_recovering_bars_ws_server().await;
    let client = Aggregator::new(config_for_ws(&base_url, None)).expect("aggregator client");

    let request = BarsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        from_close: None,
        last_n_bars: Some(1),
        format: None,
    };

    let mut connection = client
        .connect_bars_ws_recovering(
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
        .expect("connect recovering bars ws");

    match connection
        .next_frame()
        .await
        .expect("first recovering bars frame")
        .expect("some frame")
    {
        BarsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].pair, "BTCUSDT");
            assert_eq!(rows[0].close_ms, 10);
        }
        other => panic!("expected first rows frame, got {other:?}"),
    }

    match connection
        .next_frame()
        .await
        .expect("second recovering bars frame")
        .expect("some frame")
    {
        BarsWsInboundFrame::JsonRows(rows) => {
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].pair, "BTCUSDT");
            assert_eq!(rows[0].close_ms, 20);
        }
        other => panic!("expected second rows frame, got {other:?}"),
    }

    assert_eq!(connection.active_request(), &request);
    assert_eq!(connection.next_attempt(), 1);
}

#[test]
fn test_bars_ws_subscribe_rejects_non_positive_last_n_bars() {
    let request = BarsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        from_close: None,
        last_n_bars: Some(0),
        format: None,
    };

    let err = request
        .to_subscribe_text()
        .expect_err("non-positive last_n_bars should fail");

    match err {
        SdkError::RequestBuild { message } => {
            assert!(message.contains("last_n_bars"));
            assert!(message.contains("> 0"));
        }
        other => panic!("expected RequestBuild error, got {other:?}"),
    }
}
