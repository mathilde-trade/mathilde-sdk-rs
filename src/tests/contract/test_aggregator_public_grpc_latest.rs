use crate::core::auth::BearerToken;
use crate::core::config::{AggregatorConfig, GrpcTransportConfig, HttpTransportConfig};
use crate::core::error::SdkError;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1 as proto;
use crate::systems::aggregator::{Aggregator, LatestGrpcRequest};
use crate::systems::types::{BarsView, LatestMode, Timeframe};
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
struct CapturedLatestRequest {
    path: String,
    authorization: Option<String>,
    body: proto::LatestBarsRequestV1,
}

#[derive(Debug, Clone)]
enum GrpcUnaryReply {
    Success(proto::BarsLatestResponseV1),
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
        coverage_ratio: None,
        at_ms: None,
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
        coverage_ratio: Some(0.95),
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
    }
}

fn proto_latest_response_min() -> proto::BarsLatestResponseV1 {
    proto::BarsLatestResponseV1 {
        watermark_end_ms: 1770000060000,
        close_end_ms: 1770000060000,
        latest_mode: "exact_watermark".to_string(),
        view: proto::BarsViewV1::Min as i32,
        rows: vec![proto::BarsPresentRowV1 {
            bar: Some(proto_bar_min("BTCUSDT")),
            age_ms: Some(101),
        }],
        missing_pairs: vec!["ETHUSDT".to_string()],
    }
}

fn proto_latest_response_full() -> proto::BarsLatestResponseV1 {
    let mut bar = proto_bar_min("BTCUSDT");
    bar.metadata = Some(proto_metadata());

    proto::BarsLatestResponseV1 {
        watermark_end_ms: 1770000060000,
        close_end_ms: 1770000060000,
        latest_mode: "exact_watermark".to_string(),
        view: proto::BarsViewV1::Full as i32,
        rows: vec![proto::BarsPresentRowV1 {
            bar: Some(bar),
            age_ms: Some(101),
        }],
        missing_pairs: Vec::new(),
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
    crate::tests::contract::grpc_test_support::decode_test_grpc_message(body)
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

async fn spawn_latest_grpc_server(
    reply: GrpcUnaryReply,
) -> (String, oneshot::Receiver<CapturedLatestRequest>) {
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
                let decoded = decode_grpc_message::<proto::LatestBarsRequestV1>(&body_bytes);

                if let Some(sender) = captured_tx.lock().expect("capture mutex").take() {
                    let _ = sender.send(CapturedLatestRequest {
                        path,
                        authorization,
                        body: decoded,
                    });
                }

                let response = match reply {
                    GrpcUnaryReply::Success(message) => Response::builder()
                        .status(200)
                        .header("content-type", "application/grpc")
                        .header("grpc-status", "0")
                        .body(Full::new(Bytes::from(encode_grpc_message(message))))
                        .expect("grpc success response"),
                    GrpcUnaryReply::Status { code, message } => Response::builder()
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
async fn test_latest_bars_grpc_uses_unary_path_and_decodes_min_response() {
    let (base_url, captured_rx) =
        spawn_latest_grpc_server(GrpcUnaryReply::Success(proto_latest_response_min())).await;

    let token = BearerToken::new("feed_public_token").expect("valid token");
    let client = Aggregator::new(config_for_grpc(&base_url, Some(token))).expect("client");
    let request = LatestGrpcRequest {
        pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(false),
    };

    let out = client
        .latest_grpc(&request)
        .await
        .expect("latest bars grpc success");

    let captured = captured_rx.await.expect("captured grpc request");
    assert_eq!(
        captured.path,
        "/mathilde.feed.bars.v1.BarsServiceV1/LatestBars"
    );
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer feed_public_token")
    );
    assert_eq!(captured.body.pairs, vec!["BTCUSDT", "ETHUSDT"]);
    assert_eq!(captured.body.tf, "1m");
    assert_eq!(captured.body.latest_mode, "exact_watermark");
    assert!(!captured.body.metadata);

    assert_eq!(out.latest_mode, LatestMode::ExactWatermark);
    assert_eq!(out.view, BarsView::Min);
    assert_eq!(out.rows.len(), 1);
    assert_eq!(out.rows[0].pair, "BTCUSDT");
    assert_eq!(out.rows[0].age_ms, Some(101));
    assert!(out.rows[0].metadata.is_none());
    assert_eq!(out.missing_pairs, vec!["ETHUSDT".to_string()]);
}

#[tokio::test]
async fn test_latest_bars_grpc_decodes_full_response() {
    let (base_url, _) =
        spawn_latest_grpc_server(GrpcUnaryReply::Success(proto_latest_response_full())).await;

    let client = Aggregator::new(config_for_grpc(&base_url, None)).expect("client");
    let request = LatestGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(true),
    };

    let out = client
        .latest_grpc(&request)
        .await
        .expect("latest bars grpc full success");

    assert_eq!(out.view, BarsView::Full);
    assert_eq!(out.rows.len(), 1);
    assert_eq!(out.rows[0].pair, "BTCUSDT");
    assert_eq!(out.rows[0].age_ms, Some(101));
    let metadata = out.rows[0].metadata.as_ref().expect("metadata");
    assert_eq!(metadata.source, "frontier");
    assert_eq!(metadata.coverage_ratio, Some(0.95));
}

#[tokio::test]
async fn test_latest_bars_grpc_missing_grpc_config_is_typed_error() {
    let client = Aggregator::new(AggregatorConfig {
        http: HttpTransportConfig::new("http://127.0.0.1:1").expect("valid http url"),
        grpc: None,
        ws: None,
        bearer_token: None,
    })
    .expect("client");

    let request = LatestGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(false),
    };

    let error = client
        .latest_grpc(&request)
        .await
        .expect_err("expected missing grpc config error");

    match error {
        SdkError::MissingTransportConfig { transport } => assert_eq!(transport, "grpc"),
        other => panic!("expected missing grpc transport config, got {other:?}"),
    }
}

#[tokio::test]
async fn test_latest_bars_grpc_missing_present_row_age_ms_is_contract_drift() {
    let mut reply = proto_latest_response_min();
    reply.rows[0].age_ms = None;
    let (base_url, _) = spawn_latest_grpc_server(GrpcUnaryReply::Success(reply)).await;

    let client = Aggregator::new(config_for_grpc(&base_url, None)).expect("client");
    let request = LatestGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(false),
    };

    let err = client
        .latest_grpc(&request)
        .await
        .expect_err("missing present-row age_ms should fail");

    match err {
        SdkError::ContractDrift { message } => {
            assert!(message.contains("missing `age_ms`"));
        }
        other => panic!("expected contract drift error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_latest_bars_grpc_missing_bar_s_utc_is_contract_drift() {
    let mut reply = proto_latest_response_min();
    reply.rows[0].bar.as_mut().expect("bar").s_utc = None;
    let (base_url, _) = spawn_latest_grpc_server(GrpcUnaryReply::Success(reply)).await;

    let client = Aggregator::new(config_for_grpc(&base_url, None)).expect("client");
    let request = LatestGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(false),
    };

    let err = client
        .latest_grpc(&request)
        .await
        .expect_err("missing s_utc should fail");

    match err {
        SdkError::ContractDrift { message } => {
            assert!(message.contains("missing `s_utc`"));
        }
        other => panic!("expected contract drift error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_latest_bars_grpc_missing_bar_e_utc_is_contract_drift() {
    let mut reply = proto_latest_response_min();
    reply.rows[0].bar.as_mut().expect("bar").e_utc = None;
    let (base_url, _) = spawn_latest_grpc_server(GrpcUnaryReply::Success(reply)).await;

    let client = Aggregator::new(config_for_grpc(&base_url, None)).expect("client");
    let request = LatestGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(false),
    };

    let err = client
        .latest_grpc(&request)
        .await
        .expect_err("missing e_utc should fail");

    match err {
        SdkError::ContractDrift { message } => {
            assert!(message.contains("missing `e_utc`"));
        }
        other => panic!("expected contract drift error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_latest_bars_grpc_non_ok_status_is_typed_error() {
    let (base_url, _) = spawn_latest_grpc_server(GrpcUnaryReply::Status {
        code: tonic::Code::PermissionDenied,
        message: "forbidden",
    })
    .await;

    let client = Aggregator::new(config_for_grpc(&base_url, None)).expect("client");
    let request = LatestGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(false),
    };

    let error = client
        .latest_grpc(&request)
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
