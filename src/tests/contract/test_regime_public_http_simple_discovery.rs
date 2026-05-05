use crate::core::config::{HttpTransportConfig, RegimeConfig};
use crate::generated::regime::{ProcessorFamily, ProcessorGroup};
use crate::systems::regime::{DocsRegistryRequest, PairsListRequest, PairsStatusRequest, Regime};
use wiremock::matchers::{method, path, query_param};
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
async fn test_regime_docs_summary_forms_correct_path_and_decodes_payload() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/docs/summary"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "subsystem": "regime",
            "title": "Regime Summary",
            "anchor": "regime-summary",
            "intro": "Summary intro.",
            "sections": []
        })))
        .mount(&server)
        .await;

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client.docs_summary().await.expect("docs_summary success");
    assert_eq!(out["subsystem"].as_str(), Some("regime"));
    assert_eq!(out["anchor"].as_str(), Some("regime-summary"));
}

#[tokio::test]
async fn test_regime_docs_taxonomy_forms_correct_path_and_decodes_payload() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/docs/taxonomy"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "subsystem": "regime",
            "families": [{
                "family": "trend",
                "title": "Trend"
            }]
        })))
        .mount(&server)
        .await;

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client.docs_taxonomy().await.expect("docs_taxonomy success");
    assert_eq!(out["subsystem"].as_str(), Some("regime"));
    assert_eq!(out["families"][0]["family"].as_str(), Some("trend"));
}

#[tokio::test]
async fn test_regime_docs_registry_serializes_typed_selectors_and_decodes_payload() {
    let server = MockServer::start().await;
    let request = DocsRegistryRequest {
        family: Some(vec![ProcessorFamily::Trend]),
        group: Some(vec![ProcessorGroup::TrendQ1]),
    };

    Mock::given(method("GET"))
        .and(path("/v1/docs/registry"))
        .and(query_param("family", "trend"))
        .and(query_param("group", "trend.q1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "rows": [{
                "family": "trend",
                "group": "trend.q1",
                "indicator": "tr_klts_score"
            }]
        })))
        .mount(&server)
        .await;

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .docs_registry(&request)
        .await
        .expect("docs_registry success");
    assert_eq!(out["rows"][0]["indicator"].as_str(), Some("tr_klts_score"));
}

#[tokio::test]
async fn test_regime_docs_endpoints_forms_correct_path_and_decodes_payload() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/docs/endpoints"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "subsystem": "regime_feed_public_endpoint_usage",
            "sections": [{ "slug": "families" }]
        })))
        .mount(&server)
        .await;

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .docs_endpoints()
        .await
        .expect("docs_endpoints success");
    assert_eq!(
        out["subsystem"].as_str(),
        Some("regime_feed_public_endpoint_usage")
    );
    assert_eq!(out["sections"][0]["slug"].as_str(), Some("families"));
}

#[tokio::test]
async fn test_regime_pairs_status_serializes_csv_query_and_decodes_h1_readiness_block() {
    let server = MockServer::start().await;
    let request = PairsStatusRequest {
        after_pair: Some("BTCUSDT".to_string()),
        limit: Some(10),
        pairs: Some(vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]),
        filters: Some(vec!["status".to_string(), "readiness".to_string()]),
    };

    Mock::given(method("GET"))
        .and(path("/v1/pairs/status"))
        .and(query_param("after_pair", "BTCUSDT"))
        .and(query_param("limit", "10"))
        .and(query_param("pairs", "BTCUSDT,ETHUSDT"))
        .and(query_param("filters", "status,readiness"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "pairs": [{
                "pair": "BTCUSDT",
                "status": {
                    "enabled": true,
                    "run_state": "active",
                    "last_error": null,
                    "initial_date_utc": "2022-01-01T00:00:00Z",
                    "bootstrap": { "done": true }
                },
                "history": {
                    "seed_enabled": true,
                    "seed_done": true,
                    "seed_state": "done",
                    "seed_last_error": null
                },
                "readiness": {
                    "h1": { "ready": true, "ready_at_utc": "2026-03-26T16:40:53Z" }
                },
                "coverage": {
                    "h1": { "ready": true, "ready_at_utc": "2026-03-26T16:40:53Z" }
                }
            }]
        })))
        .mount(&server)
        .await;

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .pairs_status(&request)
        .await
        .expect("pairs_status success");

    assert_eq!(out.pairs.len(), 1);
    assert_eq!(out.pairs[0].pair, "BTCUSDT");
    assert!(out.pairs[0].readiness.as_ref().expect("readiness").h1.ready);
}

#[tokio::test]
async fn test_regime_pairs_list_serializes_query_and_decodes_response() {
    let server = MockServer::start().await;
    let request = PairsListRequest {
        after_pair: Some("BTCUSDT".to_string()),
        limit: Some(2),
        enabled_only: Some(true),
    };

    Mock::given(method("GET"))
        .and(path("/v1/pairs/list"))
        .and(query_param("after_pair", "BTCUSDT"))
        .and(query_param("limit", "2"))
        .and(query_param("enabled_only", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "pairs": ["ETHUSDT", "SOLUSDT"],
            "next_after_pair": "SOLUSDT"
        })))
        .mount(&server)
        .await;

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .pairs_list(&request)
        .await
        .expect("pairs_list success");
    assert_eq!(
        out.pairs,
        vec!["ETHUSDT".to_string(), "SOLUSDT".to_string()]
    );
    assert_eq!(out.next_after_pair.as_deref(), Some("SOLUSDT"));
}

#[tokio::test]
async fn test_regime_openapi_forms_correct_path_and_decodes_raw_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/openapi.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "openapi": "3.1.0",
            "info": { "title": "MATHILDE Regime Public API" },
            "paths": { "/v1/docs/taxonomy": {} }
        })))
        .mount(&server)
        .await;

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client.openapi().await.expect("openapi success");
    assert_eq!(out["openapi"], "3.1.0");
    assert_eq!(out["info"]["title"], "MATHILDE Regime Public API");
    assert!(out["paths"]["/v1/docs/taxonomy"].is_object());
}
