use crate::core::config::{AggregatorConfig, HttpTransportConfig};
use crate::core::error::SdkError;
use crate::systems::aggregator::{AggregatorClient, FilesDownloadsRequest};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config_for_http(base_url: &str) -> AggregatorConfig {
    AggregatorConfig {
        http: Some(HttpTransportConfig::new(base_url).expect("valid test url")),
        grpc: None,
        ws: None,
        bearer_token: None,
    }
}

#[tokio::test]
async fn test_files_downloads_uses_post_and_serializes_body_and_decodes_response() {
    let server = MockServer::start().await;
    let request = FilesDownloadsRequest {
        period: Some("day".to_string()),
        pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        tfs: vec!["1m".to_string(), "5m".to_string()],
        start_label_utc: Some("2026-02-20".to_string()),
        end_label_utc: Some("2026-02-21".to_string()),
        order: Some("desc".to_string()),
    };

    let expected_body = serde_json::json!({
        "period": "day",
        "pairs": ["BTCUSDT", "ETHUSDT"],
        "tfs": ["1m", "5m"],
        "start_label_utc": "2026-02-20",
        "end_label_utc": "2026-02-21",
        "order": "desc"
    });

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "rows": [
            {
                "period": "day",
                "pair": "BTCUSDT",
                "tf": "1m",
                "label_utc": "2026-02-21",
                "url": "https://example.invalid/presigned-1",
                "expires_at_utc": "2026-02-21T00:05:00Z"
            },
            {
                "period": "day",
                "pair": "ETHUSDT",
                "tf": "5m",
                "label_utc": "2026-02-20",
                "url": "https://example.invalid/presigned-2",
                "expires_at_utc": "2026-02-21T00:05:00Z"
            }
        ]
    }));

    Mock::given(method("POST"))
        .and(path("/v1/files/downloads"))
        .and(body_json(expected_body))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .files_downloads(&request)
        .await
        .expect("files_downloads success");

    assert_eq!(out.rows.len(), 2);
    assert_eq!(out.rows[0].pair, "BTCUSDT");
    assert_eq!(out.rows[0].tf, "1m");
    assert_eq!(out.rows[1].pair, "ETHUSDT");
    assert_eq!(out.rows[1].url, "https://example.invalid/presigned-2");
}

#[tokio::test]
async fn test_files_downloads_non_success_http_status_is_typed_error() {
    let server = MockServer::start().await;
    let request = FilesDownloadsRequest {
        period: Some("day".to_string()),
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec!["1m".to_string()],
        start_label_utc: None,
        end_label_utc: None,
        order: None,
    };

    Mock::given(method("POST"))
        .and(path("/v1/files/downloads"))
        .respond_with(
            ResponseTemplate::new(503)
                .set_body_string(r#"{"kind":"service_unavailable","error":"files_s3_not_configured"}"#),
        )
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .files_downloads(&request)
        .await
        .expect_err("expected http status error");

    match err {
        SdkError::HttpStatus { status, body } => {
            assert_eq!(status, 503);
            assert!(body.contains("files_s3_not_configured"));
        }
        other => panic!("expected HttpStatus error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_files_downloads_invalid_json_is_decode_error() {
    let server = MockServer::start().await;
    let request = FilesDownloadsRequest {
        period: Some("day".to_string()),
        pairs: vec!["BTCUSDT".to_string()],
        tfs: vec!["1m".to_string()],
        start_label_utc: None,
        end_label_utc: None,
        order: None,
    };

    Mock::given(method("POST"))
        .and(path("/v1/files/downloads"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_string(r#"{"rows":"not-an-array"}"#),
        )
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .files_downloads(&request)
        .await
        .expect_err("expected decode error");

    match err {
        SdkError::Decode { .. } => {}
        other => panic!("expected Decode error, got {other:?}"),
    }
}
