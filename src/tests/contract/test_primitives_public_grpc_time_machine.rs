use crate::core::auth::BearerToken;
use crate::core::config::{GrpcTransportConfig, HttpTransportConfig, PrimitivesConfig};
use crate::core::time::TimeInput;
use crate::generated::primitives::outputs_proto::mathilde::feed::outputs::v1 as proto;
use crate::systems::primitives::{PrimitiveOutput, Primitives, TimeMachineOutputsGrpcRequest};
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
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Debug)]
struct CapturedTimeMachineRequest {
    path: String,
    authorization: Option<String>,
    body: proto::TimeMachineOutputsRequestV1,
}

fn config_for_grpc(base_url: &str, bearer_token: Option<BearerToken>) -> PrimitivesConfig {
    PrimitivesConfig {
        http: HttpTransportConfig::new("http://127.0.0.1:1").expect("valid dummy http url"),
        grpc: Some(GrpcTransportConfig::new(base_url).expect("valid grpc url")),
        ws: None,
        bearer_token,
    }
}

fn proto_output_row_with_meta(pair: &str) -> proto::OutputRowV1 {
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
        metadata: Some(proto::OutputMetadataV1 {
            source: "feed".to_string(),
            process: "batch".to_string(),
            computed_at_ms: 1770000060100,
            computed_at_utc: "2026-02-02T00:01:00.100Z".to_string(),
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
    assert!(body.len() >= 5, "grpc frame too short");
    assert_eq!(body[0], 0, "compressed grpc frame unsupported in test");
    let len = u32::from_be_bytes([body[1], body[2], body[3], body[4]]) as usize;
    assert_eq!(body.len(), 5 + len, "grpc frame length mismatch");
    M::decode(&body[5..]).expect("decode grpc message")
}

async fn spawn_time_machine_grpc_server() -> (String, oneshot::Receiver<CapturedTimeMachineRequest>)
{
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
                let decoded =
                    decode_grpc_message::<proto::TimeMachineOutputsRequestV1>(&body_bytes);
                if let Some(sender) = captured_tx.lock().expect("capture mutex").take() {
                    let _ = sender.send(CapturedTimeMachineRequest {
                        path,
                        authorization,
                        body: decoded,
                    });
                }

                let message = proto::OutputsTimeMachineResponseV1 {
                    rows: vec![proto::OutputsTimeMachineRowV1 {
                        hit_close_ms: 1770000060000,
                        offset: 0,
                        output: Some(proto_output_row_with_meta("BTCUSDT")),
                    }],
                    returned_hits: 1,
                    effective_hits_limit: 10,
                    truncated: false,
                    predicate_pairs: vec!["BTCUSDT".to_string()],
                    predicate_normalized: Some("BTCUSDT.c > 100".to_string()),
                    next_cursor: None,
                    done: true,
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
async fn test_time_machine_outputs_grpc_uses_unary_path_and_decodes_rows() {
    let (base_url, captured_rx) = spawn_time_machine_grpc_server().await;
    let token = BearerToken::new("feed_public_token").expect("valid token");
    let client = Primitives::new(config_for_grpc(&base_url, Some(token))).expect("client");
    let request = TimeMachineOutputsGrpcRequest {
        tf: Timeframe::M1,
        close_start: TimeInput::from(1770000000000_i64),
        close_end: Some(TimeInput::from(1770003600000_i64)),
        cursor: None,
        predicate: Some("BTCUSDT.c > 100".to_string()),
        hits: None,
        output_pairs: Some(vec!["BTCUSDT".to_string()]),
        family: None,
        group: None,
        metadata: Some(true),
        diagnostics: Some(false),
        before_bars: Some(2),
        after_bars: Some(2),
        max_hits: Some(10),
        overlap_mode: Some("merge".to_string()),
    };

    let out = client
        .time_machine_grpc(&request)
        .await
        .expect("time-machine grpc success");
    let captured = captured_rx.await.expect("captured grpc request");

    assert_eq!(
        captured.path,
        "/mathilde.feed.outputs.v1.OutputsServiceV1/TimeMachineOutputs"
    );
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer feed_public_token")
    );
    assert_eq!(captured.body.predicate.as_deref(), Some("BTCUSDT.c > 100"));
    assert_eq!(captured.body.output_pairs, vec!["BTCUSDT"]);
    assert!(captured.body.exclude_sources.is_empty());

    assert_eq!(out.rows.len(), 1);
    match &out.rows[0].output {
        PrimitiveOutput::WithMeta(output) => assert_eq!(output.metadata.source, "feed"),
        other => panic!("expected with-meta output, got {other:?}"),
    }
}
