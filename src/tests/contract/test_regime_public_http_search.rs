use crate::core::config::{HttpTransportConfig, RegimeConfig};
use crate::core::time::TimeInput;
use crate::systems::regime::{Regime, RegimeOutput, SearchOutputsRequest};
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
async fn test_regime_search_outputs_uses_post_and_decodes_with_meta_response() {
    let server = MockServer::start().await;
    let request = SearchOutputsRequest {
        tf: Timeframe::H1,
        close_start: TimeInput::from(1_770_000_000_000_i64),
        close_end: Some(TimeInput::from(1_770_021_600_000_i64)),
        cursor: None,
        predicate: "BTCUSDT.c > 100".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        family: None,
        group: None,
        secondary: Some(true),
        metadata: Some(true),
        diagnostics: Some(true),
        max_hits: Some(5),
        format: Some(HttpFormat::Json),
    };

    let expected_body = serde_json::to_value(request.normalize_http().expect("normalize search"))
        .expect("search request json");

    Mock::given(method("POST"))
        .and(path("/v1/outputs/search"))
        .and(body_json(expected_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "hits": [1_770_003_600_000_i64],
            "evaluated_rows": [{
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
                "diagnostics": [{
                    "indicator": "trend",
                    "message": "ok"
                }],
                "metadata": {
                    "source": "feed",
                    "process": "batch",
                    "computed_at_ms": 1_770_003_600_100_i64,
                    "computed_at_utc": "2026-02-02T01:00:00.100Z",
                    "tail_bar_provenance": {}
                }
            }],
            "next_cursor": null,
            "done": true,
            "returned_hits": 1,
            "effective_hits_limit": 5,
            "truncated": false,
            "predicate_pairs": ["BTCUSDT"],
            "predicate_normalized": "BTCUSDT.c > 100"
        })))
        .mount(&server)
        .await;

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .search(&request)
        .await
        .expect("search outputs success");

    assert_eq!(out.hits, vec![1_770_003_600_000_i64]);
    assert_eq!(out.evaluated_rows.as_ref().map(Vec::len), Some(1));
    match &out.evaluated_rows.as_ref().expect("evaluated rows")[0] {
        RegimeOutput::WithMeta(output) => {
            assert_eq!(output.metadata.source, "feed");
            assert_eq!(output.diagnostics.as_ref().map(Vec::len), Some(1));
        }
        other => panic!("expected with-meta output, got {other:?}"),
    }
}
