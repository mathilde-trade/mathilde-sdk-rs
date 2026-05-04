use crate::core::config::{AggregatorConfig, HttpTransportConfig};
use crate::core::error::SdkError;
use crate::systems::aggregator::{Aggregator, PairsListRequest, PairsStatusRequest};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config_for_http(base_url: &str) -> AggregatorConfig {
    AggregatorConfig {
        http: HttpTransportConfig::new(base_url).expect("valid test url"),
        grpc: None,
        ws: None,
        bearer_token: None,
    }
}

#[tokio::test]
async fn test_docs_summary_forms_correct_path_and_decodes_payload() {
    let server = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "subsystem": "aggregator",
        "title": "Aggregator Summary",
        "anchor": "aggregator-summary",
        "source_path": "docs/public/systems/aggregator/public/summary.md",
        "generated_by": "export_public_page_json.sh",
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

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let out = client.docs_summary().await.expect("docs_summary success");

    assert_eq!(out["subsystem"].as_str(), Some("aggregator"));
    assert_eq!(out["anchor"].as_str(), Some("aggregator-summary"));
    assert_eq!(out["sections"].as_array().map(|rows| rows.len()), Some(1));
    assert_eq!(out["sections"][0]["slug"].as_str(), Some("start-here"));
}

#[tokio::test]
async fn test_docs_themes_forms_correct_path_and_decodes_payload() {
    let server = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "subsystem": "aggregator",
        "source_manifest": "docs/public/themes/manifest.json",
        "generated_by": "compile_public_themes.py",
        "themes": [{
            "theme_key": "why-time-bars-are-a-modeling-choice",
            "title": "Why Time Bars Are A Modeling Choice",
            "anchor": "why-time-bars-are-a-modeling-choice",
            "source_path": "docs/public/themes/why_time_bars.md",
            "intro": "Theme intro.",
            "sections": [{
                "heading": "Core Idea",
                "slug": "core-idea",
                "content": "Theme content."
            }]
        }]
    }));

    Mock::given(method("GET"))
        .and(path("/v1/docs/themes"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let out = client.docs_themes().await.expect("docs_themes success");

    assert_eq!(out["subsystem"].as_str(), Some("aggregator"));
    assert_eq!(out["themes"].as_array().map(|rows| rows.len()), Some(1));
    assert_eq!(
        out["themes"][0]["anchor"].as_str(),
        Some("why-time-bars-are-a-modeling-choice")
    );
    assert_eq!(
        out["themes"][0]["sections"][0]["slug"].as_str(),
        Some("core-idea")
    );
}

#[tokio::test]
async fn test_docs_endpoints_forms_correct_path_and_decodes_payload() {
    let server = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "subsystem": "aggregator_feed_public_endpoint_usage",
        "title": "Feed Public Endpoint Usage",
        "anchor": "feed-public-endpoint-usage",
        "source_path": "docs/public/endpoints/feed_public_endpoint_usage.md",
        "generated_by": "export_public_page_json.sh",
        "intro": "Endpoints intro.",
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

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .docs_endpoints()
        .await
        .expect("docs_endpoints success");

    assert_eq!(
        out["subsystem"].as_str(),
        Some("aggregator_feed_public_endpoint_usage")
    );
    assert_eq!(out["anchor"].as_str(), Some("feed-public-endpoint-usage"));
    assert_eq!(out["sections"].as_array().map(|rows| rows.len()), Some(1));
}

#[tokio::test]
async fn test_pairs_status_serializes_csv_query_and_decodes_nested_blocks() {
    let server = MockServer::start().await;
    let request = PairsStatusRequest {
        after_pair: Some("BTCUSDT".to_string()),
        limit: Some(10),
        pairs: Some(vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]),
        filters: Some(vec!["status".to_string(), "frontier".to_string()]),
    };

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "pairs": [{
            "pair": "BTCUSDT",
            "status": {
                "enabled": true,
                "run_state": "active",
                "last_error": null,
                "initial_date_utc": "2022-01-01T00:00:00Z",
                "bootstrap": {
                    "done": true,
                    "harmonized": true
                }
            },
            "frontier": {
                "frontier_subscribed": true,
                "frontier_subscribed_at_utc": "2026-03-26T16:40:53Z",
                "frontier_t0_pair_utc": "2026-03-26T16:41:00Z",
                "frontier_last_status_update_utc": "2026-03-26T16:53:55Z",
                "frontier_last_finalized_e_utc": "2026-03-26T16:53:55Z",
                "frontier_enabled_venues_n": 3,
                "frontier_connected_venues_n": 3,
                "frontier_last_error": null
            }
        }]
    }));

    Mock::given(method("GET"))
        .and(path("/v1/pairs/status"))
        .and(query_param("after_pair", "BTCUSDT"))
        .and(query_param("limit", "10"))
        .and(query_param("pairs", "BTCUSDT,ETHUSDT"))
        .and(query_param("filters", "status,frontier"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .pairs_status(&request)
        .await
        .expect("pairs_status success");

    assert_eq!(out.pairs.len(), 1);
    assert_eq!(out.pairs[0].pair, "BTCUSDT");
    assert!(out.pairs[0].status.as_ref().expect("status block").enabled);
    assert_eq!(
        out.pairs[0]
            .frontier
            .as_ref()
            .expect("frontier block")
            .frontier_enabled_venues_n,
        3
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

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "pairs": ["ETHUSDT", "SOLUSDT"],
        "next_after_pair": "SOLUSDT"
    }));

    Mock::given(method("GET"))
        .and(path("/v1/pairs/list"))
        .and(query_param("after_pair", "BTCUSDT"))
        .and(query_param("limit", "2"))
        .and(query_param("enabled_only", "true"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
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
        "info": {
            "title": "MATHILDE Feed Public API"
        },
        "paths": {
            "/v1/docs/themes": {}
        }
    }));

    Mock::given(method("GET"))
        .and(path("/openapi.json"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let out = client.openapi().await.expect("openapi success");

    assert_eq!(out["openapi"], "3.1.0");
    assert_eq!(out["info"]["title"], "MATHILDE Feed Public API");
    assert!(out["paths"]["/v1/docs/themes"].is_object());
}

#[tokio::test]
async fn test_openapi_non_success_http_status_is_typed_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/openapi.json"))
        .respond_with(
            ResponseTemplate::new(503)
                .set_body_string(r#"{"kind":"service_unavailable","error":"service_unavailable"}"#),
        )
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .openapi()
        .await
        .expect_err("expected http status error");

    match err {
        SdkError::HttpStatus { status, body } => {
            assert_eq!(status, 503);
            assert!(body.contains("service_unavailable"));
        }
        other => panic!("expected HttpStatus error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_pairs_list_invalid_json_is_decode_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/pairs/list"))
        .and(query_param("enabled_only", "true"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_string(r#"{"pairs":"not-an-array","next_after_pair":null}"#),
        )
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .pairs_list(&PairsListRequest {
            enabled_only: Some(true),
            ..PairsListRequest::default()
        })
        .await
        .expect_err("expected decode error");

    match err {
        SdkError::Decode { .. } => {}
        other => panic!("expected Decode error, got {other:?}"),
    }
}
