use crate::core::auth::BearerToken;
use crate::core::config::{GrpcTransportConfig, HttpTransportConfig, PrimitivesConfig};
use crate::generated::primitives::outputs_proto::mathilde::feed::outputs::v1 as proto;
use crate::systems::primitives::{LatestGrpcRequest, Primitives};
use crate::systems::types::{LatestMode, Timeframe};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use prost::Message;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Debug)]
struct CapturedLatestRequest {
    path: String,
    authorization: Option<String>,
    body: proto::LatestOutputsRequestV1,
}

fn config_for_grpc(base_url: &str, bearer_token: Option<BearerToken>) -> PrimitivesConfig {
    PrimitivesConfig {
        http: HttpTransportConfig::new("http://127.0.0.1:1").expect("valid dummy http url"),
        grpc: Some(GrpcTransportConfig::new(base_url).expect("valid grpc url")),
        ws: None,
        bearer_token,
    }
}

fn proto_output_row_min(pair: &str) -> proto::OutputRowV1 {
    proto::OutputRowV1 {
        pair: pair.to_string(),
        tf: "1m".to_string(),
        open_ms: 1770000000000,
        close_ms: 1770000060000,
        open_utc: Some("2026-02-02T00:00:00Z".to_string()),
        close_utc: Some("2026-02-02T00:01:00Z".to_string()),
        o: 100.0,
        h: 101.0,
        l: 99.5,
        c: 100.5,
        v: 12.34,
        bs_close_window_min: Some(0.75),
        ..Default::default()
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

async fn spawn_latest_grpc_server() -> (String, oneshot::Receiver<CapturedLatestRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind grpc test server");
    let addr = listener.local_addr().expect("grpc test addr");
    let (captured_tx, captured_rx) = oneshot::channel();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept grpc test conn");
        let io = TokioIo::new(stream);
        let captured_tx = Arc::new(Mutex::new(Some(captured_tx)));
        let service = service_fn(move |request: Request<Incoming>| {
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
                let decoded = decode_grpc_message::<proto::LatestOutputsRequestV1>(&body_bytes);
                if let Some(sender) = captured_tx.lock().expect("capture mutex").take() {
                    let _ = sender.send(CapturedLatestRequest {
                        path,
                        authorization,
                        body: decoded,
                    });
                }

                let message = proto::OutputsLatestResponseV1 {
                    watermark_end_ms: 1770000060000,
                    close_end_ms: 1770000060000,
                    latest_mode: "exact_watermark".to_string(),
                    view: proto::OutputsViewV1::Min as i32,
                    rows: vec![proto::OutputsPresentRowV1 {
                        output: Some(proto_output_row_min("BTCUSDT")),
                        age_ms: Some(101),
                    }],
                    missing_pairs: vec!["ETHUSDT".to_string()],
                };

                let response = Response::builder()
                    .status(200)
                    .header("content-type", "application/grpc")
                    .header("grpc-status", "0")
                    .body(Full::new(Bytes::from(encode_grpc_message(message))))
                    .expect("grpc success response");

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
async fn test_latest_outputs_grpc_uses_unary_path_and_decodes_min_response() {
    let (base_url, captured_rx) = spawn_latest_grpc_server().await;
    let token = BearerToken::new("feed_public_token").expect("valid token");
    let client = Primitives::new(config_for_grpc(&base_url, Some(token))).expect("client");
    let request = LatestGrpcRequest {
        pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        metadata: Some(false),
        diagnostics: Some(false),
    };

    let out = client
        .latest_grpc(&request)
        .await
        .expect("latest grpc success");
    let captured = captured_rx.await.expect("captured grpc request");

    assert_eq!(
        captured.path,
        "/mathilde.feed.outputs.v1.OutputsServiceV1/LatestOutputs"
    );
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer feed_public_token")
    );
    assert_eq!(captured.body.pairs, vec!["BTCUSDT", "ETHUSDT"]);
    assert_eq!(captured.body.tf, "1m");
    assert_eq!(captured.body.latest_mode, "exact_watermark");
    assert!(captured.body.exclude_sources.is_empty());

    assert_eq!(out.missing_pairs, vec!["ETHUSDT".to_string()]);
    assert_eq!(out.rows[0].row.pair, "BTCUSDT");
    assert_eq!(
        out.rows[0].row.computed.f64("bs_close_window_min"),
        Some(0.75)
    );
}
