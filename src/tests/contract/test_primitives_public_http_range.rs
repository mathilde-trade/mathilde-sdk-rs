use crate::core::config::{HttpTransportConfig, PrimitivesConfig};
use crate::core::error::SdkError;
use crate::core::time::TimeInput;
use crate::generated::primitives::{ProcessorFamily, ProcessorGroup};
use crate::systems::primitives::{Primitives, RangeRequest};
use crate::systems::types::{AlignMode, HttpFormat, Timeframe};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config_for_http(base_url: &str) -> PrimitivesConfig {
    PrimitivesConfig {
        http: HttpTransportConfig::new(base_url).expect("valid test url"),
        grpc: None,
        ws: None,
        bearer_token: None,
    }
}

#[tokio::test]
async fn test_range_outputs_uses_post_and_decodes_projected_min_response() {
    let server = MockServer::start().await;
    let request = RangeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        align_mode: Some(AlignMode::Exact),
        close_start: Some(TimeInput::from(1_770_000_000_000_i64)),
        cursor: None,
        close_end: Some(TimeInput::from(1_770_000_060_000_i64)),
        limit: Some(100),
        family: Some(vec![ProcessorFamily::MovingAverages]),
        group: Some(vec![ProcessorGroup::Ema]),
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
                "tf": "1m",
                "open_ms": 1_770_000_000_000_i64,
                "close_ms": 1_770_000_060_000_i64,
                "open_utc": "2026-02-02T00:00:00Z",
                "close_utc": "2026-02-02T00:01:00Z",
                "o": 100.0,
                "h": 101.0,
                "l": 99.5,
                "c": 100.5,
                "v": 12.34,
                "bs_close_window_min": 1.25
            }],
            "close_end_ms": 1_770_000_060_000_i64,
            "next_cursor": "next-range"
        })))
        .mount(&server)
        .await;

    let client = Primitives::new(config_for_http(&server.uri())).expect("client");
    let out = client.range(&request).await.expect("range outputs success");

    assert_eq!(out.next_cursor.as_deref(), Some("next-range"));
    assert_eq!(out.rows.len(), 1);
    assert_eq!(out.rows[0].pair, "BTCUSDT");
    assert_eq!(out.rows[0].computed.f64("bs_close_window_min"), Some(1.25));
    assert_eq!(out.rows[0].computed.len(), 1);
    assert!(out.rows[0].diagnostics.is_none());
}

#[tokio::test]
async fn test_range_outputs_projected_protobuf_is_rejected_before_transport() {
    let client =
        Primitives::new(config_for_http("https://primitives.api.mathilde.dev")).expect("client");
    let request = RangeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        align_mode: None,
        close_start: None,
        cursor: None,
        close_end: None,
        limit: Some(10),
        family: Some(vec![ProcessorFamily::MovingAverages]),
        group: None,
        metadata: Some(false),
        diagnostics: None,
        format: Some(HttpFormat::Protobuf),
    };

    let error = client
        .range(&request)
        .await
        .expect_err("projected protobuf must fail before transport");

    match error {
        SdkError::UnsupportedOrUnprovedUsage { .. } => {}
        other => panic!("expected UnsupportedOrUnprovedUsage, got {other:?}"),
    }
}
