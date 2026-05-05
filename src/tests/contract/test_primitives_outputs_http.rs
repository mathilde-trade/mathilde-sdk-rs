use crate::core::config::HttpTransportConfig;
use crate::core::error::SdkError;
use crate::systems::primitives::{LatestRequest, ProcessorGroup, latest_outputs};
use crate::systems::types::{HttpFormat, Timeframe};
use crate::transport::http::HttpTransport;

#[tokio::test]
async fn test_http_projected_protobuf_latest_is_rejected_before_transport() {
    let transport = HttpTransport::new(
        &HttpTransportConfig::new("https://primitives.api.mathilde.dev").expect("config"),
        None,
    );
    let request = LatestRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: None,
        family: None,
        group: Some(vec![ProcessorGroup::Ema]),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Protobuf),
    };

    let error = latest_outputs(&transport, &request)
        .await
        .expect_err("projected protobuf latest must fail closed");

    match error {
        SdkError::UnsupportedOrUnprovedUsage { .. } => {}
        other => panic!("unexpected error: {other:?}"),
    }
}
