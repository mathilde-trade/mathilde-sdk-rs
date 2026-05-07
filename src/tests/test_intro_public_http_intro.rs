use crate::core::auth::BearerToken;
use crate::core::config::{HttpTransportConfig, IntroConfig, MathildePublicHosts};
use crate::systems::intro::Intro;
use reqwest::header::{AUTHORIZATION, LOCATION};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config_for_http(base_url: &str) -> IntroConfig {
    IntroConfig {
        http: HttpTransportConfig::new(base_url).expect("valid test url"),
        bearer_token: None,
    }
}

#[test]
fn test_intro_config_mathilde_public_default_uses_manifest_host() {
    let token = BearerToken::new("intro_public_token").expect("valid token");
    let config = IntroConfig::mathilde_public_default(Some(token.clone())).expect("default config");

    assert_eq!(config.http.base_url.as_str(), "https://api.mathilde.dev/");
    assert_eq!(
        config.bearer_token.as_ref().map(BearerToken::as_str),
        Some("intro_public_token")
    );
    assert_eq!(MathildePublicHosts::INTRO, "https://api.mathilde.dev");
}

#[tokio::test]
async fn test_intro_client_mathilde_public_default_builds_transport() {
    let token = BearerToken::new("intro_public_token").expect("valid token");
    let _client = Intro::client(Some(token)).expect("default client");
}

#[tokio::test]
async fn test_intro_forms_root_path_and_preserves_json_key_order() {
    let server = MockServer::start().await;
    let body = r#"{
        "subsystem": "intro",
        "title": "MATHILDE Public Intro Root",
        "scope": "system overview",
        "intro": "Public intro root.",
        "sections": [{
            "heading": "Start Here",
            "slug": "start-here",
            "level": 2,
            "content": "Intro content.",
            "children": []
        }]
    }"#;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let client = Intro::new(config_for_http(&server.uri())).expect("client");
    let out = client.intro().await.expect("intro success");
    let serialized = serde_json::to_string(&out).expect("serialize intro json");

    assert_eq!(out["subsystem"].as_str(), Some("intro"));
    assert_eq!(out["title"].as_str(), Some("MATHILDE Public Intro Root"));
    assert_eq!(out["sections"][0]["slug"].as_str(), Some("start-here"));
    assert!(
        serialized.find("\"subsystem\"") < serialized.find("\"title\""),
        "preserve_order should keep object key order"
    );
}

#[tokio::test]
async fn test_intro_propagates_bearer_auth_to_root_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/"))
        .and(header(AUTHORIZATION.as_str(), "Bearer intro_public_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "subsystem": "intro",
            "intro": "ok"
        })))
        .mount(&server)
        .await;

    let client = Intro::new(IntroConfig {
        http: HttpTransportConfig::new(server.uri()).expect("valid test url"),
        bearer_token: Some(BearerToken::new("intro_public_token").expect("valid token")),
    })
    .expect("client");

    let out = client.intro().await.expect("intro success");
    assert_eq!(out["subsystem"].as_str(), Some("intro"));
}

#[tokio::test]
async fn test_intro_root_redirect_response_is_accepted_by_http_client() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(307).insert_header(LOCATION.as_str(), "/v1/intro"))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1/intro"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "subsystem": "intro",
            "intro": "redirected"
        })))
        .mount(&server)
        .await;

    let client = Intro::new(config_for_http(&server.uri())).expect("client");
    let out = client.intro().await.expect("redirected intro success");
    assert_eq!(out["intro"].as_str(), Some("redirected"));
}

#[tokio::test]
async fn test_due_diligence_methods_form_exact_paths_and_preserve_json_key_order() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/due-diligence"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{
                "surface": "due_diligence_index",
                "title": "MATHILDE Due Diligence",
                "endpoint": "/v1/due-diligence",
                "available_packs": []
            }"#,
        ))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1/due-diligence/regime/kalman_local_trend_state"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "surface": "due_diligence_pack",
            "system": "regime",
            "subject_id": "kalman_local_trend_state"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path(
            "/v1/due-diligence/regime/flow_absorption_elasticity_state",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "surface": "due_diligence_pack",
            "system": "regime",
            "subject_id": "flow_absorption_elasticity_state"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1/due-diligence/primitives/correlation"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "surface": "due_diligence_pack",
            "system": "primitives",
            "subject_id": "correlation"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1/due-diligence/primitives/drawdown"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "surface": "due_diligence_pack",
            "system": "primitives",
            "subject_id": "drawdown"
        })))
        .mount(&server)
        .await;

    let client = Intro::new(config_for_http(&server.uri())).expect("client");

    let index = client
        .due_diligence()
        .await
        .expect("due_diligence index success");
    let serialized = serde_json::to_string(&index).expect("serialize due_diligence index");
    assert_eq!(index["endpoint"].as_str(), Some("/v1/due-diligence"));
    assert!(
        serialized.find("\"surface\"") < serialized.find("\"title\""),
        "preserve_order should keep object key order"
    );

    let kalman = client
        .due_diligence_regime_kalman_local_trend_state()
        .await
        .expect("kalman due_diligence success");
    assert_eq!(kalman["system"].as_str(), Some("regime"));
    assert_eq!(
        kalman["subject_id"].as_str(),
        Some("kalman_local_trend_state")
    );

    let flow = client
        .due_diligence_regime_flow_absorption_elasticity_state()
        .await
        .expect("flow due_diligence success");
    assert_eq!(flow["system"].as_str(), Some("regime"));
    assert_eq!(
        flow["subject_id"].as_str(),
        Some("flow_absorption_elasticity_state")
    );

    let correlation = client
        .due_diligence_primitives_correlation()
        .await
        .expect("correlation due_diligence success");
    assert_eq!(correlation["system"].as_str(), Some("primitives"));
    assert_eq!(correlation["subject_id"].as_str(), Some("correlation"));

    let drawdown = client
        .due_diligence_primitives_drawdown()
        .await
        .expect("drawdown due_diligence success");
    assert_eq!(drawdown["system"].as_str(), Some("primitives"));
    assert_eq!(drawdown["subject_id"].as_str(), Some("drawdown"));
}

#[tokio::test]
async fn test_due_diligence_propagates_bearer_auth_to_index_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/due-diligence"))
        .and(header(AUTHORIZATION.as_str(), "Bearer intro_public_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "surface": "due_diligence_index",
            "endpoint": "/v1/due-diligence"
        })))
        .mount(&server)
        .await;

    let client = Intro::new(IntroConfig {
        http: HttpTransportConfig::new(server.uri()).expect("valid test url"),
        bearer_token: Some(BearerToken::new("intro_public_token").expect("valid token")),
    })
    .expect("client");

    let out = client
        .due_diligence()
        .await
        .expect("due_diligence index success");
    assert_eq!(out["surface"].as_str(), Some("due_diligence_index"));
}
