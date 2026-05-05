use crate::core::auth::BearerToken;
use crate::core::config::{HttpTransportConfig, RegimeConfig};
use crate::core::time::TimeInput;
use crate::systems::regime::{
    RangeCall, RangeRequest, Regime, SearchCall, SearchRequest, TimeMachineCall, TimeMachineRequest,
};
use crate::systems::types::Timeframe;

fn test_client() -> Regime {
    let token = BearerToken::new("feed_public_token").expect("token");
    Regime::new(RegimeConfig {
        http: HttpTransportConfig::new("https://regime.api.mathilde.dev").expect("http"),
        grpc: None,
        ws: None,
        bearer_token: Some(token),
    })
    .expect("client")
}

#[test]
fn test_regime_range_outputs_pager_allows_implicit_close_end() {
    let client = test_client();
    let call = RangeCall::new(
        &client,
        RangeRequest {
            pairs: vec!["BTCUSDT".to_string()],
            tf: Timeframe::H1,
            align_mode: None,
            close_start: None,
            cursor: None,
            close_end: None,
            limit: Some(10),
            family: None,
            group: None,
            secondary: Some(false),
            metadata: Some(false),
            diagnostics: Some(false),
            format: None,
        },
    );

    call.pager().expect("range pager should build");
}

#[test]
fn test_regime_search_outputs_pager_requires_explicit_close_end() {
    let client = test_client();
    let call = SearchCall::new(
        &client,
        SearchRequest {
            tf: Timeframe::H1,
            close_start: TimeInput::from(1_700_000_000_000_i64),
            close_end: None,
            cursor: None,
            predicate: "BTCUSDT.c > 1".to_string(),
            evaluate_pair: None,
            family: None,
            group: None,
            secondary: Some(true),
            metadata: Some(false),
            diagnostics: Some(false),
            max_hits: Some(10),
            format: None,
        },
    );

    let error = call
        .pager()
        .expect_err("search pager must require explicit close_end");
    assert!(matches!(
        error,
        crate::core::error::SdkError::UnsupportedOrUnprovedUsage { .. }
    ));
}

#[test]
fn test_regime_time_machine_outputs_pager_requires_explicit_close_end() {
    let client = test_client();
    let call = TimeMachineCall::new(
        &client,
        TimeMachineRequest {
            tf: Timeframe::H1,
            close_start: TimeInput::from(1_700_000_000_000_i64),
            close_end: None,
            cursor: None,
            predicate: Some("BTCUSDT.c > 1".to_string()),
            hits: None,
            output_pairs: Some(vec!["BTCUSDT".to_string()]),
            family: None,
            group: None,
            secondary: Some(true),
            metadata: Some(false),
            diagnostics: Some(false),
            before_bars: Some(1),
            after_bars: Some(1),
            max_hits: Some(10),
            overlap_mode: None,
            format: None,
        },
    );

    let error = call
        .pager()
        .expect_err("time-machine pager must require explicit close_end");
    assert!(matches!(
        error,
        crate::core::error::SdkError::UnsupportedOrUnprovedUsage { .. }
    ));
}
