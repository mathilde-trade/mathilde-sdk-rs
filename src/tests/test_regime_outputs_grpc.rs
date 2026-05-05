use crate::core::config::GrpcTransportConfig;
use crate::core::error::SdkError;
use crate::systems::regime::{LatestGrpcRequest, latest_outputs_grpc};
use crate::systems::types::Timeframe;
use crate::transport::grpc::GrpcTransport;

#[tokio::test]
async fn test_regime_grpc_projected_latest_is_rejected_before_transport() {
    let transport = GrpcTransport::new(
        &GrpcTransportConfig::new("https://regime.grpc.mathilde.dev").expect("config"),
        None,
    )
    .expect("transport");
    let request = LatestGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::H1,
        latest_mode: None,
        family: None,
        group: None,
        secondary: Some(false),
        metadata: Some(false),
        diagnostics: Some(false),
    };

    let error = latest_outputs_grpc(&transport, &request)
        .await
        .expect_err("projected grpc latest must fail closed");

    assert!(matches!(error, SdkError::UnsupportedOrUnprovedUsage { .. }));
}

#[tokio::test]
async fn test_regime_grpc_latest_rejects_non_h1_before_transport() {
    let transport = GrpcTransport::new(
        &GrpcTransportConfig::new("https://regime.grpc.mathilde.dev").expect("config"),
        None,
    )
    .expect("transport");
    let request = LatestGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: None,
        family: None,
        group: None,
        secondary: Some(true),
        metadata: Some(false),
        diagnostics: Some(false),
    };

    let error = latest_outputs_grpc(&transport, &request)
        .await
        .expect_err("non-h1 grpc latest must fail closed");

    match error {
        SdkError::UnsupportedOrUnprovedUsage { message } => {
            assert!(message.contains("tf=1h"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
