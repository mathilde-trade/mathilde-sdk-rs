use crate::core::config::{HttpTransportConfig, PrimitivesConfig};
use crate::generated::primitives::{ProcessorFamily, ProcessorGroup};
use crate::systems::primitives::{
    DocsRegistryRequest, PairsListRequest, PairsStatusRequest, Primitives,
};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config_for_http(base_url: &str) -> PrimitivesConfig {
    PrimitivesConfig {
        http: HttpTransportConfig::new(base_url).expect("valid test url"),
        grpc: None,
        ws: None,
        bearer_token: None,
    }
}

fn readiness_cell(ready: bool, at: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "ready": ready,
        "ready_at_utc": at
    })
}

#[tokio::test]
async fn test_docs_summary_forms_correct_path_and_decodes_payload() {
    let server = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "subsystem": "primitives",
        "title": "Primitives Summary",
        "anchor": "primitives-summary",
        "intro": "Summary intro.",
        "sections": [{
            "heading": "Start Here",
            "slug": "start-here",
            "level": 2,
            "content": "Summary content.",
            "children": []
        }]
    }));

    Mock::given(method("GET"))
        .and(path("/v1/docs/summary"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Primitives::new(config_for_http(&server.uri())).expect("client");
    let out = client.docs_summary().await.expect("docs_summary success");

    assert_eq!(out["subsystem"].as_str(), Some("primitives"));
    assert_eq!(out["anchor"].as_str(), Some("primitives-summary"));
}

#[tokio::test]
async fn test_docs_taxonomy_forms_correct_path_and_decodes_payload() {
    let server = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "subsystem": "primitives",
        "families": [{
            "family": "moving_averages",
            "title": "Moving averages"
        }]
    }));

    Mock::given(method("GET"))
        .and(path("/v1/docs/taxonomy"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Primitives::new(config_for_http(&server.uri())).expect("client");
    let out = client.docs_taxonomy().await.expect("docs_taxonomy success");

    assert_eq!(out["subsystem"].as_str(), Some("primitives"));
    assert_eq!(
        out["families"][0]["family"].as_str(),
        Some("moving_averages")
    );
}

#[tokio::test]
async fn test_docs_registry_serializes_typed_selectors_and_decodes_payload() {
    let server = MockServer::start().await;
    let request = DocsRegistryRequest {
        family: Some(vec![
            ProcessorFamily::MovingAverages,
            ProcessorFamily::Metadata,
        ]),
        group: Some(vec![ProcessorGroup::Ema]),
    };

    Mock::given(method("GET"))
        .and(path("/v1/docs/registry"))
        .and(query_param("family", "moving_averages,metadata"))
        .and(query_param("group", "ema"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "rows": [{
                "family": "moving_averages",
                "group": "ema",
                "indicator": "ma_ema_p20"
            }]
        })))
        .mount(&server)
        .await;

    let client = Primitives::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .docs_registry(&request)
        .await
        .expect("docs_registry success");

    assert_eq!(out["rows"][0]["indicator"].as_str(), Some("ma_ema_p20"));
}

#[tokio::test]
async fn test_docs_endpoints_forms_correct_path_and_decodes_payload() {
    let server = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "subsystem": "primitives_feed_public_endpoint_usage",
        "anchor": "primitives-feed-public-endpoint-usage",
        "sections": [{
            "heading": "Families",
            "slug": "families",
            "level": 2,
            "content": "Endpoint content.",
            "children": []
        }]
    }));

    Mock::given(method("GET"))
        .and(path("/v1/docs/endpoints"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Primitives::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .docs_endpoints()
        .await
        .expect("docs_endpoints success");

    assert_eq!(
        out["subsystem"].as_str(),
        Some("primitives_feed_public_endpoint_usage")
    );
    assert_eq!(out["sections"][0]["slug"].as_str(), Some("families"));
}

#[tokio::test]
async fn test_pairs_status_serializes_csv_query_and_decodes_nested_blocks() {
    let server = MockServer::start().await;
    let request = PairsStatusRequest {
        after_pair: Some("BTCUSDT".to_string()),
        limit: Some(10),
        pairs: Some(vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]),
        filters: Some(vec!["status".to_string(), "readiness".to_string()]),
    };

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
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
                "m1":   { "ready": true,  "ready_at_utc": "2026-03-26T16:40:53Z" },
                "m5":   { "ready": true,  "ready_at_utc": "2026-03-26T16:40:53Z" },
                "m15":  { "ready": true,  "ready_at_utc": "2026-03-26T16:40:53Z" },
                "m30":  { "ready": true,  "ready_at_utc": "2026-03-26T16:40:53Z" },
                "h1":   { "ready": false, "ready_at_utc": null },
                "h4":   { "ready": false, "ready_at_utc": null },
                "h6":   { "ready": false, "ready_at_utc": null },
                "h12":  { "ready": false, "ready_at_utc": null },
                "d1":   { "ready": false, "ready_at_utc": null }
            },
            "coverage": {
                "m1": { "ready": true, "ready_at_utc": "2026-03-26T16:40:53Z" }
            }
        }]
    }));

    Mock::given(method("GET"))
        .and(path("/v1/pairs/status"))
        .and(query_param("after_pair", "BTCUSDT"))
        .and(query_param("limit", "10"))
        .and(query_param("pairs", "BTCUSDT,ETHUSDT"))
        .and(query_param("filters", "status,readiness"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Primitives::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .pairs_status(&request)
        .await
        .expect("pairs_status success");

    assert_eq!(out.pairs.len(), 1);
    assert_eq!(out.pairs[0].pair, "BTCUSDT");
    assert!(out.pairs[0].status.as_ref().expect("status block").enabled);
    assert_eq!(
        out.pairs[0].readiness.as_ref().expect("readiness").m1,
        serde_json::from_value(readiness_cell(true, Some("2026-03-26T16:40:53Z")))
            .expect("readiness cell")
    );
    assert!(
        out.pairs[0]
            .coverage
            .as_ref()
            .expect("coverage")
            .is_object()
    );
}

#[tokio::test]
async fn test_pairs_list_serializes_query_and_decodes_response() {
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

    let client = Primitives::new(config_for_http(&server.uri())).expect("client");
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
async fn test_openapi_forms_correct_path_and_decodes_raw_json() {
    let server = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "openapi": "3.1.0",
        "info": { "title": "MATHILDE Primitives Public API" },
        "paths": { "/v1/docs/taxonomy": {} }
    }));

    Mock::given(method("GET"))
        .and(path("/openapi.json"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Primitives::new(config_for_http(&server.uri())).expect("client");
    let out = client.openapi().await.expect("openapi success");

    assert_eq!(out["openapi"], "3.1.0");
    assert_eq!(out["info"]["title"], "MATHILDE Primitives Public API");
    assert!(out["paths"]["/v1/docs/taxonomy"].is_object());
}
