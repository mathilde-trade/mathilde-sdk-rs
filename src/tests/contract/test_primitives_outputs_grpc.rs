use crate::core::config::GrpcTransportConfig;
use crate::core::error::SdkError;
use crate::generated::primitives::ProcessorFamily;
use crate::systems::primitives::LatestGrpcRequest;
use crate::systems::primitives::latest_outputs_grpc;
use crate::systems::types::Timeframe;
use crate::transport::grpc::GrpcTransport;

#[tokio::test]
async fn test_grpc_projected_latest_is_rejected_before_transport() {
    let transport = GrpcTransport::new(
        &GrpcTransportConfig::new("https://primitives.grpc.mathilde.dev").expect("config"),
        None,
    )
    .expect("transport");
    let request = LatestGrpcRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: None,
        family: Some(vec![ProcessorFamily::MovingAverages]),
        group: None,
        metadata: Some(false),
        diagnostics: Some(false),
    };

    let error = latest_outputs_grpc(&transport, &request)
        .await
        .expect_err("projected grpc latest must fail closed");

    match error {
        SdkError::UnsupportedOrUnprovedUsage { .. } => {}
        other => panic!("unexpected error: {other:?}"),
    }
}
