use crate::core::config::HttpTransportConfig;
use crate::core::error::SdkError;
use crate::systems::regime::{LatestRequest, latest_outputs};
use crate::systems::types::{HttpFormat, Timeframe};
use crate::transport::http::HttpTransport;

#[tokio::test]
async fn test_regime_http_projected_protobuf_latest_is_rejected_before_transport() {
    let transport = HttpTransport::new(
        &HttpTransportConfig::new("https://regime.api.mathilde.dev").expect("config"),
        None,
    );
    let request = LatestRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::H1,
        latest_mode: None,
        family: None,
        group: None,
        secondary: Some(false),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Protobuf),
    };

    let error = latest_outputs(&transport, &request)
        .await
        .expect_err("projected protobuf latest must fail closed");

    assert!(matches!(error, SdkError::UnsupportedOrUnprovedUsage { .. }));
}

#[tokio::test]
async fn test_regime_http_latest_rejects_non_h1_before_transport() {
    let transport = HttpTransport::new(
        &HttpTransportConfig::new("https://regime.api.mathilde.dev").expect("config"),
        None,
    );
    let request = LatestRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: None,
        family: None,
        group: None,
        secondary: Some(true),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Json),
    };

    let error = latest_outputs(&transport, &request)
        .await
        .expect_err("non-h1 latest must fail closed");

    match error {
        SdkError::UnsupportedOrUnprovedUsage { message } => {
            assert!(message.contains("tf=1h"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
