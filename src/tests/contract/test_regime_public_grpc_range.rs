use crate::core::auth::BearerToken;
use crate::core::config::{GrpcTransportConfig, HttpTransportConfig, RegimeConfig};
use crate::core::time::TimeInput;
use crate::generated::regime::outputs_proto::mathilde::feed::outputs::v1 as proto;
use crate::systems::regime::{RangeGrpcRequest, Regime};
use crate::systems::types::{AlignMode, Timeframe};
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
struct CapturedRangeRequest {
    path: String,
    authorization: Option<String>,
    body: proto::RangeOutputsRequestV1,
}

fn config_for_grpc(base_url: &str, bearer_token: Option<BearerToken>) -> RegimeConfig {
    RegimeConfig {
        http: HttpTransportConfig::new("http://127.0.0.1:1").expect("valid dummy http url"),
        grpc: Some(GrpcTransportConfig::new(base_url).expect("valid grpc url")),
        ws: None,
        bearer_token,
    }
}

fn proto_output_row_with_meta(pair: &str) -> proto::OutputRowV1 {
    proto::OutputRowV1 {
        pair: pair.to_string(),
        tf: "1h".to_string(),
        open_ms: 1770000000000,
        close_ms: 1770003600000,
        open_utc: Some("2026-02-02T00:00:00Z".to_string()),
        close_utc: Some("2026-02-02T01:00:00Z".to_string()),
        o: 100.0,
        h: 101.0,
        l: 99.5,
        c: 100.5,
        v: 12.34,
        metadata: Some(proto::OutputMetadataV1 {
            source: "feed".to_string(),
            process: "batch".to_string(),
            computed_at_ms: 1770003600100,
            computed_at_utc: "2026-02-02T01:00:00.100Z".to_string(),
            tail_bar_provenance: Some(proto::OutputBarsMetadataV1::default()),
            ..Default::default()
        }),
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

async fn spawn_range_grpc_server() -> (String, oneshot::Receiver<CapturedRangeRequest>) {
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
                let decoded = decode_grpc_message::<proto::RangeOutputsRequestV1>(&body_bytes);
                if let Some(sender) = captured_tx.lock().expect("capture mutex").take() {
                    let _ = sender.send(CapturedRangeRequest {
                        path,
                        authorization,
                        body: decoded,
                    });
                }

                let message = proto::OutputsRangeResponseV1 {
                    rows: vec![proto_output_row_with_meta("BTCUSDT")],
                    next_cursor: Some("next-range".to_string()),
                    close_end_ms: 1770003600000,
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
async fn test_regime_range_outputs_grpc_uses_unary_path_and_decodes_with_meta_response() {
    let (base_url, captured_rx) = spawn_range_grpc_server().await;
    let token = BearerToken::new("feed_public_token").expect("valid token");
    let client = Regime::new(config_for_grpc(&base_url, Some(token))).expect("client");
    let request = RangeGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::H1,
        align_mode: Some(AlignMode::Exact),
        close_start: Some(TimeInput::from(1770000000000_i64)),
        cursor: None,
        close_end: Some(TimeInput::from(1770003600000_i64)),
        limit: Some(100),
        family: None,
        group: None,
        secondary: Some(true),
        metadata: Some(true),
        diagnostics: Some(true),
    };

    let out = client
        .range_grpc(&request)
        .await
        .expect("range grpc success");
    let captured = captured_rx.await.expect("captured grpc request");

    assert_eq!(
        captured.path,
        "/mathilde.feed.outputs.v1.OutputsServiceV1/RangeOutputs"
    );
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer feed_public_token")
    );
    assert_eq!(captured.body.tf, "1h");
    assert!(captured.body.secondary);
    assert!(captured.body.exclude_sources.is_empty());
    assert_eq!(out.next_cursor.as_deref(), Some("next-range"));
    assert_eq!(
        out.rows[0].metadata.as_ref().expect("metadata").source,
        "feed"
    );
}
