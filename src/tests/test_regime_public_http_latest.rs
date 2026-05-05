use crate::core::auth::BearerToken;
use crate::core::config::{HttpTransportConfig, MathildePublicHosts, RegimeConfig};
use crate::systems::regime::{LatestRequest, Regime};
use crate::systems::types::{HttpFormat, LatestMode, Timeframe};
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

#[test]
fn test_regime_config_mathilde_public_default_uses_manifest_hosts() {
    let token = BearerToken::new("feed_public_token").expect("valid token");
    let config =
        RegimeConfig::mathilde_public_default(Some(token.clone())).expect("default config");

    assert_eq!(
        config.http.base_url.as_str(),
        "https://regime.api.mathilde.dev/"
    );
    assert_eq!(
        config.grpc.as_ref().expect("grpc").base_url.as_str(),
        "https://regime.grpc.mathilde.dev/"
    );
    assert_eq!(
        config.ws.as_ref().expect("ws").base_url.as_str(),
        "wss://regime.api.mathilde.dev/"
    );
    assert_eq!(
        config.bearer_token.as_ref().map(BearerToken::as_str),
        Some("feed_public_token")
    );

    assert_eq!(
        MathildePublicHosts::REGIME_HTTP,
        "https://regime.api.mathilde.dev"
    );
    assert_eq!(
        MathildePublicHosts::REGIME_GRPC,
        "https://regime.grpc.mathilde.dev"
    );
}

#[tokio::test]
async fn test_regime_client_mathilde_public_default_builds_transports() {
    let token = BearerToken::new("feed_public_token").expect("valid token");
    let _client = Regime::client(Some(token)).expect("default client");
}

#[tokio::test]
async fn test_regime_docs_system_forms_correct_path_and_preserves_json_key_order() {
    let server = MockServer::start().await;
    let body = r#"{
        "subsystem": "regime",
        "kind": "system",
        "title": "Regime",
        "anchor": "regime",
        "intro": "Regime intro.",
        "sections": [{
            "heading": "What It Is",
            "slug": "what-it-is",
            "level": 2,
            "content": "Regime content.",
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

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client.docs_system().await.expect("docs_system success");
    let serialized = serde_json::to_string(&out).expect("serialize docs json");

    assert_eq!(out["subsystem"].as_str(), Some("regime"));
    assert_eq!(out["anchor"].as_str(), Some("regime"));
    assert_eq!(out["sections"][0]["slug"].as_str(), Some("what-it-is"));
    assert!(
        serialized.find("\"subsystem\"") < serialized.find("\"kind\""),
        "preserve_order should keep object key order"
    );
}

#[tokio::test]
async fn test_regime_latest_uses_post_and_decodes_projected_min_response() {
    let server = MockServer::start().await;
    let request = LatestRequest {
        pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        tf: Timeframe::H1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        secondary: Some(false),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Json),
    };

    let expected_body = serde_json::to_value(request.normalize_http().expect("normalize latest"))
        .expect("latest request json");

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "watermark_end_ms": 1770003600000_i64,
        "close_end_ms": 1770003600000_i64,
        "latest_mode": "exact_watermark",
        "view": "min",
        "rows": [{
            "pair": "BTCUSDT",
            "tf": "1h",
            "open_ms": 1770000000000_i64,
            "close_ms": 1770003600000_i64,
            "open_utc": "2026-02-02T00:00:00Z",
            "close_utc": "2026-02-02T01:00:00Z",
            "o": 100.0,
            "h": 101.0,
            "l": 99.5,
            "c": 100.5,
            "v": 12.34,
            "tr_klts_score": 0.75,
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

    let client = Regime::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .latest(&request)
        .await
        .expect("latest outputs success");

    assert_eq!(out.missing_pairs, vec!["ETHUSDT".to_string()]);
    assert_eq!(out.rows[0].age_ms, 101);
    assert_eq!(out.rows[0].row.pair, "BTCUSDT");
    assert_eq!(out.rows[0].row.computed.f64("tr_klts_score"), Some(0.75));
    assert!(out.rows[0].row.diagnostics.is_none());
}
