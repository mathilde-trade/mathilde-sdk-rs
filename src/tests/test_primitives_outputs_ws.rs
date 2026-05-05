use crate::core::error::SdkError;
use crate::systems::primitives::{OutputsWsFormat, OutputsWsSubscribeRequest, ProcessorFamily};
use crate::systems::types::Timeframe;

#[test]
fn test_outputs_ws_projected_protobuf_is_rejected_before_transport() {
    let request = OutputsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: Some(vec![ProcessorFamily::MovingAverages]),
        group: None,
        from_close: None,
        last_n_bars: Some(1),
        format: Some(OutputsWsFormat::Protobuf),
    };

    let error = request
        .normalize()
        .expect_err("projected protobuf ws subscribe must fail closed");
    assert!(matches!(error, SdkError::UnsupportedOrUnprovedUsage { .. }));
}
