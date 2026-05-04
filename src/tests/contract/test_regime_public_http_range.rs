use crate::core::config::{HttpTransportConfig, RegimeConfig};
use crate::core::error::SdkError;
use crate::core::time::TimeInput;
use crate::generated::regime::ProjectedValue;
use crate::systems::regime::{RangeOutputsRequest, Regime, RegimeOutput};
use crate::systems::types::{AlignMode, HttpFormat, Timeframe};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config_for_http(base_url: &str) -> RegimeConfig {
    RegimeConfig {
        http: HttpTransportConfig::new(base_url).expect("valid test url"),
        grpc: None,
        ws: None,
        bearer_token: None,
    }
}

#[tokio::test]
async fn test_regime_range_outputs_uses_post_and_decodes_projected_min_response() {
    let server = MockServer::start().await;
    let request = RangeOutputsRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::H1,
        align_mode: Some(AlignMode::Exact),
        close_start: Some(TimeInput::from(1_770_000_000_000_i64)),
        cursor: None,
        close_end: Some(TimeInput::from(1_770_003_600_000_i64)),
        limit: Some(100),
        family: None,
        group: None,
        secondary: Some(false),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Json),
    };

    let expected_body = serde_json::to_value(request.normalize_http().expect("normalize range"))
        .expect("range request json");

    Mock::given(method("POST"))
        .and(path("/v1/outputs/range"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "rows": [{
                "pair": "BTCUSDT",
                "tf": "1h",
                "open_ms": 1_770_000_000_000_i64,
                "close_ms": 1_770_003_600_000_i64,
                "open_utc": "2026-02-02T00:00:00Z",
                "close_utc": "2026-02-02T01:00:00Z",
                "o": 100.0,
                "h": 101.0,
                "l": 99.5,
                "c": 100.5,
                "v": 12.34,
                "tr_klts_score": 1.25
            }],
            "close_end_ms": 1_770_003_600_000_i64,
            "next_cursor": "next-range"
        })))
        .mount(&server)
        .await;

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client.range(&request).await.expect("range outputs success");

    assert_eq!(out.next_cursor.as_deref(), Some("next-range"));
    match &out.rows[0] {
        RegimeOutput::ProjectedMin(output) => {
            assert_eq!(output.pair, "BTCUSDT");
            assert_eq!(output.tr_klts_score, ProjectedValue::Included(Some(1.25)));
            assert!(output.diagnostics.is_none());
        }
        other => panic!("expected projected min output, got {other:?}"),
    }
}

#[tokio::test]
async fn test_regime_range_outputs_projected_protobuf_is_rejected_before_transport() {
    let client = Regime::new(config_for_http("https://regime.api.mathilde.dev")).expect("client");
    let request = RangeOutputsRequest {
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
        diagnostics: None,
        format: Some(HttpFormat::Protobuf),
    };

    let error = client
        .range(&request)
        .await
        .expect_err("projected protobuf must fail before transport");

    assert!(matches!(error, SdkError::UnsupportedOrUnprovedUsage { .. }));
}
