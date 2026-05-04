use crate::core::error::SdkError;
use crate::systems::regime::{OutputsWsFormat, OutputsWsSubscribeRequest};
use crate::systems::types::Timeframe;

#[test]
fn test_regime_outputs_ws_projected_protobuf_is_rejected_before_transport() {
    let request = OutputsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::H1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        secondary: Some(false),
        from_close: None,
        last_n_bars: Some(1),
        format: Some(OutputsWsFormat::Protobuf),
    };

    let error = request
        .normalize()
        .expect_err("projected protobuf ws subscribe must fail closed");

    assert!(matches!(error, SdkError::UnsupportedOrUnprovedUsage { .. }));
}

#[test]
fn test_regime_outputs_ws_rejects_non_h1_before_transport() {
    let request = OutputsWsSubscribeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        secondary: Some(true),
        from_close: None,
        last_n_bars: Some(1),
        format: Some(OutputsWsFormat::Json),
    };

    let error = request
        .normalize()
        .expect_err("non-h1 outputs ws subscribe must fail closed");

    match error {
        SdkError::UnsupportedOrUnprovedUsage { message } => {
            assert!(message.contains("tf=1h"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
