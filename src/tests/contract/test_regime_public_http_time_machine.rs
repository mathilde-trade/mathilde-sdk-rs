use crate::core::config::{HttpTransportConfig, RegimeConfig};
use crate::core::time::TimeInput;
use crate::generated::regime::{ProcessorFamily, ProcessorGroup};
use crate::systems::regime::{Regime, TimeMachineRequest};
use crate::systems::types::{HttpFormat, Timeframe};
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
async fn test_regime_time_machine_outputs_uses_post_and_decodes_projected_with_meta_response() {
    let server = MockServer::start().await;
    let request = TimeMachineRequest {
        tf: Timeframe::H1,
        close_start: TimeInput::from(1_770_000_000_000_i64),
        close_end: Some(TimeInput::from(1_770_021_600_000_i64)),
        cursor: None,
        predicate: Some("BTCUSDT.c > 100".to_string()),
        hits: None,
        output_pairs: Some(vec!["BTCUSDT".to_string()]),
        family: Some(vec![ProcessorFamily::Trend]),
        group: Some(vec![ProcessorGroup::TrendQ1]),
        secondary: Some(false),
        metadata: Some(true),
        diagnostics: Some(false),
        before_bars: Some(2),
        after_bars: Some(2),
        max_hits: Some(10),
        overlap_mode: Some("merge".to_string()),
        format: Some(HttpFormat::Json),
    };

    let expected_body =
        serde_json::to_value(request.normalize_http().expect("normalize time machine"))
            .expect("time-machine request json");

    Mock::given(method("POST"))
        .and(path("/v1/outputs/time-machine"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "rows": [{
                "hit_close_ms": 1_770_003_600_000_i64,
                "offset": 0,
                "output": {
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
                    "tr_klts_score": 1.5,
                    "metadata": {
                        "source": "feed",
                        "process": "batch",
                        "computed_at_ms": 1_770_003_600_100_i64,
                        "computed_at_utc": "2026-02-02T01:00:00.100Z",
                        "tail_bar_provenance": {}
                    }
                }
            }],
            "next_cursor": null,
            "done": true,
            "returned_hits": 1,
            "effective_hits_limit": 10,
            "truncated": false,
            "predicate_pairs": ["BTCUSDT"],
            "predicate_normalized": "BTCUSDT.c > 100"
        })))
        .mount(&server)
        .await;

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .time_machine(&request)
        .await
        .expect("time-machine outputs success");

    assert_eq!(out.rows.len(), 1);
    assert_eq!(
        out.rows[0].row.metadata.as_ref().expect("metadata").source,
        "feed"
    );
    assert_eq!(out.rows[0].row.computed.f64("tr_klts_score"), Some(1.5));
    assert!(out.rows[0].row.diagnostics.is_none());
}
