use crate::core::auth::BearerToken;
use crate::core::config::{HttpTransportConfig, MathildePublicHosts, PrimitivesConfig};
use crate::systems::primitives::{LatestOutputsRequest, PrimitiveOutput, Primitives};
use crate::systems::types::{HttpFormat, LatestMode, Timeframe};
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

#[test]
fn test_primitives_config_mathilde_public_default_uses_manifest_hosts() {
    let token = BearerToken::new("feed_public_token").expect("valid token");
    let config =
        PrimitivesConfig::mathilde_public_default(Some(token.clone())).expect("default config");

    assert_eq!(
        config.http.base_url.as_str(),
        "https://primitives.api.mathilde.dev/"
    );
    assert_eq!(
        config.grpc.as_ref().expect("grpc").base_url.as_str(),
        "https://primitives.grpc.mathilde.dev/"
    );
    assert_eq!(
        config.ws.as_ref().expect("ws").base_url.as_str(),
        "wss://primitives.api.mathilde.dev/"
    );
    assert_eq!(
        config.bearer_token.as_ref().map(BearerToken::as_str),
        Some("feed_public_token")
    );

    assert_eq!(
        MathildePublicHosts::PRIMITIVES_HTTP,
        "https://primitives.api.mathilde.dev"
    );
    assert_eq!(
        MathildePublicHosts::PRIMITIVES_GRPC,
        "https://primitives.grpc.mathilde.dev"
    );
}

#[tokio::test]
async fn test_primitives_client_mathilde_public_default_builds_transports() {
    let token = BearerToken::new("feed_public_token").expect("valid token");
    let _client = Primitives::client(Some(token)).expect("default client");
}

#[tokio::test]
async fn test_primitives_docs_system_forms_correct_path_and_preserves_json_key_order() {
    let server = MockServer::start().await;
    let body = r#"{
        "subsystem": "primitives",
        "kind": "system",
        "title": "Primitives",
        "anchor": "primitives",
        "intro": "Primitives intro.",
        "sections": [{
            "heading": "What It Is",
            "slug": "what-it-is",
            "level": 2,
            "content": "Primitives content.",
            "children": []
        }]
    }"#;

    Mock::given(method("GET"))
        .and(path("/v1/docs/system"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let client = Primitives::new(config_for_http(&server.uri())).expect("client");
    let out = client.docs_system().await.expect("docs_system success");
    let serialized = serde_json::to_string(&out).expect("serialize docs json");

    assert_eq!(out["subsystem"].as_str(), Some("primitives"));
    assert_eq!(out["anchor"].as_str(), Some("primitives"));
    assert_eq!(out["sections"][0]["slug"].as_str(), Some("what-it-is"));
    assert!(serialized.find("\"subsystem\"").is_some());
    assert!(serialized.find("\"kind\"").is_some());
    assert!(
        serialized.find("\"subsystem\"") < serialized.find("\"kind\""),
        "preserve_order should keep object key order"
    );
}

#[tokio::test]
async fn test_latest_outputs_uses_post_and_decodes_min_response() {
    let server = MockServer::start().await;
    let request = LatestOutputsRequest {
        pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        metadata: Some(false),
        diagnostics: Some(true),
        format: Some(HttpFormat::Json),
    };

    let expected_body = serde_json::to_value(request.normalize_http().expect("normalize latest"))
        .expect("latest request json");

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "watermark_end_ms": 1770000060000_i64,
        "close_end_ms": 1770000060000_i64,
        "latest_mode": "exact_watermark",
        "view": "min",
        "rows": [{
            "pair": "BTCUSDT",
            "tf": "1m",
            "open_ms": 1770000000000_i64,
            "close_ms": 1770000060000_i64,
            "open_utc": "2026-02-02T00:00:00Z",
            "close_utc": "2026-02-02T00:01:00Z",
            "o": 100.0,
            "h": 101.0,
            "l": 99.5,
            "c": 100.5,
            "v": 12.34,
            "quote_v": 1234.56,
            "taker_known_v": 6.17,
            "taker_signed_v": 1.23,
            "taker_known_quote_v": 617.28,
            "taker_signed_quote_v": 123.45,
            "taker_known_n": 18,
            "taker_signed_n": 3,
            "vw": 100.21,
            "n": null,
            "bs_close_window_min": 0.75,
            "age_ms": 101
        }],
        "missing_pairs": ["ETHUSDT"]
    }));

    Mock::given(method("POST"))
        .and(path("/v1/outputs/latest"))
        .and(body_json(expected_body))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Primitives::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .latest(&request)
        .await
        .expect("latest outputs success");

    assert_eq!(out.missing_pairs, vec!["ETHUSDT".to_string()]);
    assert_eq!(out.rows.len(), 1);
    match &out.rows[0].output {
        PrimitiveOutput::Min(output) => {
            assert_eq!(output.pair, "BTCUSDT");
            assert_eq!(output.bs_close_window_min, Some(0.75));
            assert!(output.diagnostics.is_none());
        }
        other => panic!("expected min output, got {other:?}"),
    }
}
