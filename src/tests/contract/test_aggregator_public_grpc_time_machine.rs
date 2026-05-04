use crate::core::auth::BearerToken;
use crate::core::config::{AggregatorConfig, GrpcTransportConfig, HttpTransportConfig};
use crate::core::error::SdkError;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1 as proto;
use crate::systems::aggregator::{Aggregator, TimeMachineBarsGrpcRequest, TimeMachineBarsResponse};
use crate::systems::types::Timeframe;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use prost::Message;
use std::convert::Infallible;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Debug)]
struct CapturedTimeMachineRequest {
    path: String,
    authorization: Option<String>,
    body: proto::TimeMachineBarsRequestV1,
}

#[derive(Debug, Clone)]
enum TimeMachineGrpcUnaryReply {
    Success(proto::BarsTimeMachineResponseV1),
    Status {
        code: tonic::Code,
        message: &'static str,
    },
}

fn config_for_grpc(base_url: &str, bearer_token: Option<BearerToken>) -> AggregatorConfig {
    AggregatorConfig {
        http: HttpTransportConfig::new("http://127.0.0.1:1").expect("valid dummy http url"),
        grpc: Some(GrpcTransportConfig::new(base_url).expect("valid grpc url")),
        ws: None,
        bearer_token,
    }
}

fn proto_bar_min(pair: &str) -> proto::BarRowV1 {
    proto::BarRowV1 {
        pair: pair.to_string(),
        tf: "1m".to_string(),
        s_ms: 1770000000000,
        e_ms: 1770000060000,
        s_utc: Some("2026-02-02T00:00:00Z".to_string()),
        e_utc: Some("2026-02-02T00:01:00Z".to_string()),
        o: 100.0,
        h: 101.0,
        l: 99.5,
        c: 100.5,
        v: 12.34,
        quote_v: Some(1234.56),
        taker_known_v: Some(6.17),
        taker_signed_v: Some(1.23),
        taker_known_quote_v: Some(617.28),
        taker_signed_quote_v: Some(123.45),
        taker_known_n: Some(18),
        taker_signed_n: Some(3),
        vw: Some(100.21),
        n: None,
        coverage_ratio: Some(0.95),
        at_ms: Some(1770000060005),
        metadata: None,
    }
}

fn proto_metadata() -> proto::BarMetadataV1 {
    proto::BarMetadataV1 {
        source: "frontier".to_string(),
        process: None,
        venues_expected: vec![
            "binance".to_string(),
            "bybit".to_string(),
            "okx".to_string(),
        ],
        venues_with_trades: vec!["binance".to_string()],
        ingested_at_ms: Some(1770000060101),
        ingested_at_utc: Some("2026-02-02T00:01:00Z".to_string()),
        target_ingested_at_ms: Some(1770000060150),
        target_ingested_at_utc: Some("2026-02-02T00:01:00Z".to_string()),
        built_at_ms: None,
        built_at_utc: None,
        committed_at_ms: Some(1770000060102),
        committed_at_utc: Some("2026-02-02T00:01:00Z".to_string()),
        harmonized_at_ms: None,
        harmonized_at_utc: None,
        recomputed_at_ms: None,
        recomputed_at_utc: None,
        recomputed_reason: None,
        covered_1m_count: None,
        expected_1m_count: None,
        coverage_ratio: None,
        inputs_source_counts_frontier: None,
        inputs_source_counts_api: None,
        inputs_source_counts_synthetic: None,
        inputs_source_counts_fix_data: None,
        frontier_5s_inputs_coverage_ratio: None,
        frontier_5s_expected: Some(12),
        frontier_5s_synth_n: Some(0),
        frontier_5s_synth_ratio: Some(0.0),
        frontier_5s_trade_n: Some(12),
        frontier_5s_trade_ratio: Some(1.0),
        age_ms: Some(202),
    }
}

fn proto_time_machine_response_min() -> proto::BarsTimeMachineResponseV1 {
    proto::BarsTimeMachineResponseV1 {
        rows: vec![proto::BarsTimeMachineRowV1 {
            hit_close_ms: 1770000060000,
            offset: 0,
            bar: Some(proto_bar_min("BTCUSDT")),
        }],
        returned_hits: 1,
        effective_hits_limit: 500,
        truncated: false,
        predicate_pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        predicate_normalized: Some("BTCUSDT.c > ETHUSDT.c * 1.5".to_string()),
        next_cursor: Some("cursor-1".to_string()),
        done: false,
    }
}

fn proto_time_machine_response_full() -> proto::BarsTimeMachineResponseV1 {
    let mut bar = proto_bar_min("BTCUSDT");
    bar.metadata = Some(proto_metadata());

    proto::BarsTimeMachineResponseV1 {
        rows: vec![proto::BarsTimeMachineRowV1 {
            hit_close_ms: 1770000060000,
            offset: 0,
            bar: Some(bar),
        }],
        returned_hits: 1,
        effective_hits_limit: 100,
        truncated: false,
        predicate_pairs: vec!["BTCUSDT".to_string()],
        predicate_normalized: None,
        next_cursor: None,
        done: true,
    }
}

fn encode_grpc_message<M: Message>(message: M) -> Vec<u8> {
    let body = message.encode_to_vec();
    let mut frame = Vec::with_capacity(5 + body.len());
    frame.push(0);
    frame.extend_from_slice(&(body.len() as u32).to_be_bytes());
    frame.extend_from_slice(&body);
    frame
}

fn decode_grpc_message<M: Message + Default>(body: &[u8]) -> M {
    assert!(body.len() >= 5, "grpc frame too short");
    assert_eq!(body[0], 0, "compressed grpc frame unsupported in test");
    let len = u32::from_be_bytes([body[1], body[2], body[3], body[4]]) as usize;
    assert_eq!(body.len(), 5 + len, "grpc frame length mismatch");
    M::decode(&body[5..]).expect("decode grpc message")
}

fn grpc_status_number(code: tonic::Code) -> &'static str {
    match code {
        tonic::Code::Ok => "0",
        tonic::Code::Cancelled => "1",
        tonic::Code::Unknown => "2",
        tonic::Code::InvalidArgument => "3",
        tonic::Code::DeadlineExceeded => "4",
        tonic::Code::NotFound => "5",
        tonic::Code::AlreadyExists => "6",
        tonic::Code::PermissionDenied => "7",
        tonic::Code::ResourceExhausted => "8",
        tonic::Code::FailedPrecondition => "9",
        tonic::Code::Aborted => "10",
        tonic::Code::OutOfRange => "11",
        tonic::Code::Unimplemented => "12",
        tonic::Code::Internal => "13",
        tonic::Code::Unavailable => "14",
        tonic::Code::DataLoss => "15",
        tonic::Code::Unauthenticated => "16",
    }
}

async fn spawn_time_machine_grpc_server(
    reply: TimeMachineGrpcUnaryReply,
) -> (String, oneshot::Receiver<CapturedTimeMachineRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind grpc test server");
    let addr = listener.local_addr().expect("grpc test addr");
    let (captured_tx, captured_rx) = oneshot::channel();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept grpc test conn");
        let io = TokioIo::new(stream);
        let captured_tx = std::sync::Arc::new(std::sync::Mutex::new(Some(captured_tx)));
        let reply = reply.clone();

        let service = service_fn(move |request: Request<Incoming>| {
            let reply = reply.clone();
            let captured_tx = captured_tx.clone();

            async move {
                let path = request.uri().path().to_string();
                let authorization = request
                    .headers()
                    .get("authorization")
                    .and_then(|value| value.to_str().ok())
                    .map(ToOwned::to_owned);
                let body_bytes = request
                    .into_body()
                    .collect()
                    .await
                    .expect("collect grpc request body")
                    .to_bytes();
                let decoded = decode_grpc_message::<proto::TimeMachineBarsRequestV1>(&body_bytes);

                if let Some(sender) = captured_tx.lock().expect("capture mutex").take() {
                    let _ = sender.send(CapturedTimeMachineRequest {
                        path,
                        authorization,
                        body: decoded,
                    });
                }

                let response = match reply {
                    TimeMachineGrpcUnaryReply::Success(message) => Response::builder()
                        .status(200)
                        .header("content-type", "application/grpc")
                        .header("grpc-status", "0")
                        .body(Full::new(Bytes::from(encode_grpc_message(message))))
                        .expect("grpc success response"),
                    TimeMachineGrpcUnaryReply::Status { code, message } => Response::builder()
                        .status(200)
                        .header("content-type", "application/grpc")
                        .header("grpc-status", grpc_status_number(code))
                        .header("grpc-message", message)
                        .body(Full::new(Bytes::new()))
                        .expect("grpc status response"),
                };

                Ok::<_, Infallible>(response)
            }
        });

        http2::Builder::new(TokioExecutor::new())
            .serve_connection(io, service)
            .await
            .expect("serve grpc test connection");
    });

    (format!("http://{addr}"), captured_rx)
}

#[tokio::test]
async fn test_time_machine_bars_grpc_predicate_mode_uses_unary_path_and_decodes_min_response() {
    let (base_url, captured_rx) = spawn_time_machine_grpc_server(
        TimeMachineGrpcUnaryReply::Success(proto_time_machine_response_min()),
    )
    .await;

    let token = BearerToken::new("feed_public_token").expect("valid token");
    let client = Aggregator::new(config_for_grpc(&base_url, Some(token))).expect("client");
    let request = TimeMachineBarsGrpcRequest {
        tf: Timeframe::M1,
        close_start: "2026-02-02T00:00:00Z".into(),
        close_end: Some(1770007200000_i64.into()),
        cursor: None,
        predicate: Some("BTCUSDT.c > ETHUSDT.c * 1.5".to_string()),
        hits: None,
        output_pairs: Some(vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]),
        metadata: Some(false),
        before_bars: Some(10),
        after_bars: Some(10),
        max_hits: Some(500),
        overlap_mode: Some("merge".to_string()),
    };

    let out = client
        .time_machine_grpc(&request)
        .await
        .expect("time-machine grpc success");

    let captured = captured_rx.await.expect("captured grpc request");
    assert_eq!(
        captured.path,
        "/mathilde.feed.bars.v1.BarsServiceV1/TimeMachineBars"
    );
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer feed_public_token")
    );
    assert_eq!(captured.body.tf, "1m");
    assert_eq!(captured.body.close_start_ms, 1769990400000);
    assert_eq!(captured.body.close_end_ms, 1770007200000);
    assert!(captured.body.cursor.is_none());
    assert_eq!(
        captured.body.predicate.as_deref(),
        Some("BTCUSDT.c > ETHUSDT.c * 1.5")
    );
    assert!(captured.body.hits.is_empty());
    assert_eq!(
        captured.body.output_pairs,
        vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]
    );
    assert!(!captured.body.metadata);
    assert_eq!(captured.body.before_bars, Some(10));
    assert_eq!(captured.body.after_bars, Some(10));
    assert_eq!(captured.body.max_hits, Some(500));
    assert_eq!(captured.body.overlap_mode.as_deref(), Some("merge"));

    match out {
        TimeMachineBarsResponse::Min(out) => {
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].bar.pair, "BTCUSDT");
            assert_eq!(out.rows[0].offset, 0);
            assert_eq!(out.next_cursor.as_deref(), Some("cursor-1"));
            assert!(!out.done);
            assert_eq!(out.returned_hits, 1);
            assert_eq!(out.effective_hits_limit, 500);
            assert!(!out.truncated);
            assert_eq!(out.predicate_pairs, vec!["BTCUSDT", "ETHUSDT"]);
            assert_eq!(
                out.predicate_normalized.as_deref(),
                Some("BTCUSDT.c > ETHUSDT.c * 1.5")
            );
        }
        TimeMachineBarsResponse::Full(other) => {
            panic!("expected min time-machine grpc response, got full: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_time_machine_bars_grpc_hits_mode_omitted_close_end_decodes_full_response() {
    let (base_url, captured_rx) = spawn_time_machine_grpc_server(
        TimeMachineGrpcUnaryReply::Success(proto_time_machine_response_full()),
    )
    .await;

    let client = Aggregator::new(config_for_grpc(&base_url, None)).expect("client");
    let request = TimeMachineBarsGrpcRequest {
        tf: Timeframe::M1,
        close_start: "2026-02-02:00:00".into(),
        close_end: None,
        cursor: Some("cursor-1".to_string()),
        predicate: None,
        hits: Some(vec![1770000060000, 1770000120000]),
        output_pairs: Some(vec!["BTCUSDT".to_string()]),
        metadata: Some(true),
        before_bars: Some(2),
        after_bars: Some(2),
        max_hits: Some(100),
        overlap_mode: Some("clip".to_string()),
    };

    let out = client
        .time_machine_grpc(&request)
        .await
        .expect("time-machine grpc success");

    let captured = captured_rx.await.expect("captured grpc request");
    assert_eq!(captured.body.close_start_ms, 1769990400000);
    assert_eq!(captured.body.close_end_ms, 0);
    assert_eq!(captured.body.cursor.as_deref(), Some("cursor-1"));
    assert!(captured.body.predicate.is_none());
    assert_eq!(captured.body.hits, vec![1770000060000, 1770000120000]);
    assert_eq!(captured.body.output_pairs, vec!["BTCUSDT".to_string()]);
    assert!(captured.body.metadata);
    assert_eq!(captured.body.before_bars, Some(2));
    assert_eq!(captured.body.after_bars, Some(2));
    assert_eq!(captured.body.max_hits, Some(100));
    assert_eq!(captured.body.overlap_mode.as_deref(), Some("clip"));

    match out {
        TimeMachineBarsResponse::Full(out) => {
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].bar.pair, "BTCUSDT");
            assert_eq!(out.rows[0].bar.coverage_ratio, Some(0.95));
            assert_eq!(out.rows[0].bar.at_ms, Some(1770000060005));
            assert_eq!(out.rows[0].bar.metadata.age_ms, Some(202));
            assert_eq!(out.rows[0].offset, 0);
            assert!(out.next_cursor.is_none());
            assert!(out.done);
            assert_eq!(out.returned_hits, 1);
            assert_eq!(out.effective_hits_limit, 100);
            assert!(!out.truncated);
            assert_eq!(out.predicate_pairs, vec!["BTCUSDT"]);
            assert!(out.predicate_normalized.is_none());
        }
        TimeMachineBarsResponse::Min(other) => {
            panic!("expected full time-machine grpc response, got min: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_time_machine_bars_grpc_returns_missing_config_error_without_grpc_transport() {
    let client = Aggregator::new(AggregatorConfig {
        http: HttpTransportConfig::new("http://127.0.0.1:1").expect("valid dummy http url"),
        grpc: None,
        ws: None,
        bearer_token: None,
    })
    .expect("client");

    let err = client
        .time_machine_grpc(&TimeMachineBarsGrpcRequest {
            tf: Timeframe::M1,
            close_start: "2026-02-02T00:00:00Z".into(),
            close_end: None,
            cursor: None,
            predicate: Some("BTCUSDT.c > 0".to_string()),
            hits: None,
            output_pairs: Some(vec!["BTCUSDT".to_string()]),
            metadata: Some(false),
            before_bars: Some(1),
            after_bars: Some(1),
            max_hits: Some(10),
            overlap_mode: Some("merge".to_string()),
        })
        .await
        .expect_err("missing grpc config should fail");

    match err {
        SdkError::MissingTransportConfig { transport } => assert_eq!(transport, "grpc"),
        other => panic!("expected missing grpc config error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_time_machine_bars_grpc_maps_non_ok_grpc_status() {
    let (base_url, _) = spawn_time_machine_grpc_server(TimeMachineGrpcUnaryReply::Status {
        code: tonic::Code::InvalidArgument,
        message: "predicate or hits mode required",
    })
    .await;

    let client = Aggregator::new(config_for_grpc(&base_url, None)).expect("client");
    let err = client
        .time_machine_grpc(&TimeMachineBarsGrpcRequest {
            tf: Timeframe::M1,
            close_start: "2026-02-02T00:00:00Z".into(),
            close_end: None,
            cursor: None,
            predicate: Some("BTCUSDT.c > 0".to_string()),
            hits: None,
            output_pairs: Some(vec!["BTCUSDT".to_string()]),
            metadata: Some(false),
            before_bars: Some(1),
            after_bars: Some(1),
            max_hits: Some(10),
            overlap_mode: Some("merge".to_string()),
        })
        .await
        .expect_err("grpc status should fail");

    match err {
        SdkError::GrpcStatus { code, message } => {
            assert_eq!(code, tonic::Code::InvalidArgument);
            assert!(message.contains("predicate or hits mode required"));
        }
        other => panic!("expected grpc status error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_time_machine_bars_grpc_call_traverse_requires_explicit_close_end() {
    let client = Aggregator::new(config_for_grpc("http://127.0.0.1:1", None)).expect("client");
    let request = TimeMachineBarsGrpcRequest {
        tf: Timeframe::M1,
        close_start: "2026-02-02T00:00:00Z".into(),
        close_end: None,
        cursor: None,
        predicate: Some("BTCUSDT.c > ETHUSDT.c * 1.5".to_string()),
        hits: None,
        output_pairs: Some(vec!["BTCUSDT".to_string()]),
        metadata: Some(false),
        before_bars: Some(5),
        after_bars: Some(5),
        max_hits: Some(100),
        overlap_mode: Some("merge".to_string()),
    };

    let err = client
        .time_machine_grpc_call(request)
        .traverse()
        .await
        .expect_err("open-ended grpc time-machine traverse must fail closed");

    match err {
        SdkError::UnsupportedOrUnprovedUsage { message } => {
            assert_eq!(
                message,
                "time-machine traversal requires explicit close_end"
            );
        }
        other => panic!("expected UnsupportedOrUnprovedUsage, got {other:?}"),
    }
}
