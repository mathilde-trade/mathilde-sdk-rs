use crate::core::auth::BearerToken;
use crate::core::config::{AggregatorConfig, GrpcTransportConfig, HttpTransportConfig};
use crate::core::error::SdkError;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1 as proto;
use crate::systems::aggregator::{Aggregator, RangeBarsGrpcRequest, RangeBarsResponse};
use crate::systems::types::{AlignMode, Timeframe};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use prost::Message;
use std::collections::VecDeque;
use std::convert::Infallible;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
struct CapturedRangeRequest {
    path: String,
    authorization: Option<String>,
    body: proto::RangeBarsRequestV1,
}

#[derive(Debug, Clone)]
enum RangeGrpcUnaryReply {
    Success(proto::BarsRangeResponseV1),
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
        venues_expected: vec!["binance".to_string(), "bybit".to_string()],
        venues_with_trades: vec!["binance".to_string()],
        ingested_at_ms: Some(1770000060101),
        ingested_at_utc: Some("2026-02-02T00:01:00Z".to_string()),
        target_ingested_at_ms: None,
        target_ingested_at_utc: None,
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

fn proto_range_response_min() -> proto::BarsRangeResponseV1 {
    proto::BarsRangeResponseV1 {
        rows: vec![proto_bar_min("BTCUSDT")],
        next_cursor: Some("cursor-1".to_string()),
        close_end_ms: 1770003600000,
    }
}

fn proto_range_response_full() -> proto::BarsRangeResponseV1 {
    let mut bar = proto_bar_min("BTCUSDT");
    bar.metadata = Some(proto_metadata());

    proto::BarsRangeResponseV1 {
        rows: vec![bar],
        next_cursor: None,
        close_end_ms: 1770003600000,
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

async fn spawn_range_grpc_server(
    reply: RangeGrpcUnaryReply,
) -> (String, oneshot::Receiver<CapturedRangeRequest>) {
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
                let decoded = decode_grpc_message::<proto::RangeBarsRequestV1>(&body_bytes);

                if let Some(sender) = captured_tx.lock().expect("capture mutex").take() {
                    let _ = sender.send(CapturedRangeRequest {
                        path,
                        authorization,
                        body: decoded,
                    });
                }

                let response = match reply {
                    RangeGrpcUnaryReply::Success(message) => Response::builder()
                        .status(200)
                        .header("content-type", "application/grpc")
                        .header("grpc-status", "0")
                        .body(Full::new(Bytes::from(encode_grpc_message(message))))
                        .expect("grpc success response"),
                    RangeGrpcUnaryReply::Status { code, message } => Response::builder()
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

async fn spawn_range_grpc_server_sequence(
    replies: Vec<RangeGrpcUnaryReply>,
) -> (String, mpsc::UnboundedReceiver<CapturedRangeRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind grpc test server");
    let addr = listener.local_addr().expect("grpc test addr");
    let (captured_tx, captured_rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept grpc test conn");
        let io = TokioIo::new(stream);
        let captured_tx = captured_tx.clone();
        let replies = std::sync::Arc::new(std::sync::Mutex::new(VecDeque::from(replies)));

        let service = service_fn(move |request: Request<Incoming>| {
            let captured_tx = captured_tx.clone();
            let replies = replies.clone();

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
                let decoded = decode_grpc_message::<proto::RangeBarsRequestV1>(&body_bytes);

                let _ = captured_tx.send(CapturedRangeRequest {
                    path,
                    authorization,
                    body: decoded,
                });

                let reply = replies
                    .lock()
                    .expect("reply mutex")
                    .pop_front()
                    .expect("grpc reply available");

                let response = match reply {
                    RangeGrpcUnaryReply::Success(message) => Response::builder()
                        .status(200)
                        .header("content-type", "application/grpc")
                        .header("grpc-status", "0")
                        .body(Full::new(Bytes::from(encode_grpc_message(message))))
                        .expect("grpc success response"),
                    RangeGrpcUnaryReply::Status { code, message } => Response::builder()
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
async fn test_range_bars_grpc_tail_mode_uses_unary_path_and_decodes_min_response() {
    let (base_url, captured_rx) =
        spawn_range_grpc_server(RangeGrpcUnaryReply::Success(proto_range_response_min())).await;

    let token = BearerToken::new("feed_public_token").expect("valid token");
    let client = Aggregator::new(config_for_grpc(&base_url, Some(token))).expect("client");
    let request = RangeBarsGrpcRequest {
        pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        tf: Timeframe::M1,
        align_mode: None,
        close_start: None,
        cursor: None,
        close_end: None,
        limit: Some(100),
        metadata: Some(false),
    };

    let out = client
        .range_grpc(&request)
        .await
        .expect("range bars grpc success");

    let captured = captured_rx.await.expect("captured grpc request");
    assert_eq!(
        captured.path,
        "/mathilde.feed.bars.v1.BarsServiceV1/RangeBars"
    );
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer feed_public_token")
    );
    assert_eq!(captured.body.pairs, vec!["BTCUSDT", "ETHUSDT"]);
    assert_eq!(captured.body.tf, "1m");
    assert_eq!(captured.body.close_start_ms, 0);
    assert_eq!(captured.body.close_end_ms, 0);
    assert!(captured.body.cursor.is_none());
    assert_eq!(captured.body.limit, Some(100));
    assert!(!captured.body.metadata);
    assert!(captured.body.align_mode.is_none());

    match out {
        RangeBarsResponse::Min(out) => {
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].pair, "BTCUSDT");
            assert_eq!(out.rows[0].coverage_ratio, Some(0.95));
            assert_eq!(out.rows[0].at_ms, Some(1770000060005));
            assert_eq!(out.next_cursor.as_deref(), Some("cursor-1"));
            assert_eq!(out.close_end_ms, 1770003600000);
        }
        RangeBarsResponse::Full(other) => {
            panic!("expected min range grpc response, got full: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_range_bars_grpc_explicit_window_decodes_full_response() {
    let (base_url, captured_rx) =
        spawn_range_grpc_server(RangeGrpcUnaryReply::Success(proto_range_response_full())).await;

    let client = Aggregator::new(config_for_grpc(&base_url, None)).expect("client");
    let request = RangeBarsGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        align_mode: Some(AlignMode::Exact),
        close_start: Some(1770000000000_i64.into()),
        cursor: Some("cursor-1".to_string()),
        close_end: Some(1770003600000_i64.into()),
        limit: Some(100),
        metadata: Some(true),
    };

    let out = client
        .range_grpc(&request)
        .await
        .expect("range bars grpc full success");

    let captured = captured_rx.await.expect("captured grpc request");
    assert_eq!(captured.body.close_start_ms, 1770000000000);
    assert_eq!(captured.body.close_end_ms, 1770003600000);
    assert_eq!(captured.body.cursor.as_deref(), Some("cursor-1"));
    assert_eq!(captured.body.align_mode.as_deref(), Some("exact"));
    assert!(captured.body.metadata);

    match out {
        RangeBarsResponse::Full(out) => {
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].pair, "BTCUSDT");
            assert_eq!(out.rows[0].metadata.source, "frontier");
            assert_eq!(out.rows[0].metadata.age_ms, Some(202));
            assert!(out.next_cursor.is_none());
        }
        RangeBarsResponse::Min(other) => {
            panic!("expected full range grpc response, got min: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_range_bars_grpc_missing_grpc_config_is_typed_error() {
    let client = Aggregator::new(AggregatorConfig {
        http: HttpTransportConfig::new("http://127.0.0.1:1").expect("valid http url"),
        grpc: None,
        ws: None,
        bearer_token: None,
    })
    .expect("client");

    let request = RangeBarsGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        align_mode: None,
        close_start: None,
        cursor: None,
        close_end: None,
        limit: Some(100),
        metadata: Some(false),
    };

    let error = client
        .range_grpc(&request)
        .await
        .expect_err("expected missing grpc config error");

    match error {
        SdkError::MissingTransportConfig { transport } => assert_eq!(transport, "grpc"),
        other => panic!("expected missing grpc transport config, got {other:?}"),
    }
}

#[tokio::test]
async fn test_range_bars_grpc_non_ok_status_is_typed_error() {
    let (base_url, _) = spawn_range_grpc_server(RangeGrpcUnaryReply::Status {
        code: tonic::Code::PermissionDenied,
        message: "forbidden",
    })
    .await;

    let client = Aggregator::new(config_for_grpc(&base_url, None)).expect("client");
    let request = RangeBarsGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        align_mode: None,
        close_start: None,
        cursor: None,
        close_end: None,
        limit: Some(100),
        metadata: Some(false),
    };

    let error = client
        .range_grpc(&request)
        .await
        .expect_err("expected grpc status failure");

    match error {
        SdkError::GrpcStatus { code, message } => {
            assert_eq!(code, tonic::Code::PermissionDenied);
            assert_eq!(message, "forbidden");
        }
        other => panic!("expected grpc status error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_range_bars_grpc_call_send_matches_one_page_method() {
    let (base_url, _) =
        spawn_range_grpc_server(RangeGrpcUnaryReply::Success(proto_range_response_min())).await;

    let client = Aggregator::new(config_for_grpc(&base_url, None)).expect("client");
    let request = RangeBarsGrpcRequest {
        pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        tf: Timeframe::M1,
        align_mode: None,
        close_start: None,
        cursor: None,
        close_end: None,
        limit: Some(100),
        metadata: Some(false),
    };

    let one_page = client
        .range_grpc(&request)
        .await
        .expect("one-page grpc range success");
    let via_call = client
        .range_grpc_call(request.clone())
        .send()
        .await
        .expect("wrapper grpc range send success");

    assert_eq!(via_call, one_page);
}

#[tokio::test]
async fn test_range_bars_grpc_call_traverse_freezes_omitted_close_end_from_first_page() {
    let mut second_reply = proto_range_response_min();
    second_reply.rows = vec![proto_bar_min("ETHUSDT")];
    second_reply.next_cursor = None;

    let (base_url, mut captured_rx) = spawn_range_grpc_server_sequence(vec![
        RangeGrpcUnaryReply::Success(proto_range_response_min()),
        RangeGrpcUnaryReply::Success(second_reply),
    ])
    .await;

    let client = Aggregator::new(config_for_grpc(&base_url, None)).expect("client");
    let request = RangeBarsGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        align_mode: None,
        close_start: Some("2026-02-02T00:00:00Z".into()),
        cursor: None,
        close_end: None,
        limit: Some(2),
        metadata: Some(false),
    };

    let out = client
        .range_grpc_call(request)
        .traverse()
        .await
        .expect("grpc range traverse success");

    let first = captured_rx
        .recv()
        .await
        .expect("first captured grpc request");
    let second = captured_rx
        .recv()
        .await
        .expect("second captured grpc request");

    assert_eq!(first.body.close_end_ms, 0);
    assert!(first.body.cursor.is_none());
    assert_eq!(second.body.close_end_ms, 1770003600000);
    assert_eq!(second.body.cursor.as_deref(), Some("cursor-1"));
    assert_eq!(out.pages_fetched, 2);
    assert_eq!(out.pages.len(), 2);

    match &out.pages[0] {
        RangeBarsResponse::Min(response) => {
            assert_eq!(response.rows[0].pair, "BTCUSDT");
            assert_eq!(response.next_cursor.as_deref(), Some("cursor-1"));
        }
        other => panic!("expected min first grpc page, got {other:?}"),
    }

    match &out.pages[1] {
        RangeBarsResponse::Min(response) => {
            assert_eq!(response.rows[0].pair, "ETHUSDT");
            assert!(response.next_cursor.is_none());
        }
        other => panic!("expected min second grpc page, got {other:?}"),
    }
}
