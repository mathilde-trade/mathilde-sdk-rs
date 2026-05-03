use crate::core::auth::BearerToken;
use crate::core::config::{AggregatorConfig, HttpTransportConfig};
use crate::core::error::SdkError;
use crate::systems::aggregator::{AggregatorClient, FilesDownloadsRequest, FilesDownloadsRow};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config_for_http(base_url: &str) -> AggregatorConfig {
    AggregatorConfig {
        http: HttpTransportConfig::new(base_url).expect("valid test url"),
        grpc: None,
        ws: None,
        bearer_token: None,
    }
}

fn config_for_http_with_bearer(base_url: &str, bearer: &str) -> AggregatorConfig {
    AggregatorConfig {
        http: HttpTransportConfig::new(base_url).expect("valid test url"),
        grpc: None,
        ws: None,
        bearer_token: Some(BearerToken::new(bearer).expect("valid token")),
    }
}

fn sample_download_row(url: String, pair: &str, tf: &str, label_utc: &str) -> FilesDownloadsRow {
    FilesDownloadsRow {
        period: "day".to_string(),
        pair: pair.to_string(),
        tf: tf.to_string(),
        label_utc: label_utc.to_string(),
        url,
        expires_at_utc: "2026-02-21T00:05:00Z".to_string(),
    }
}

fn unique_temp_root(test_name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("mathilde-sdk-rs-{test_name}-{nonce}"))
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
            ResponseTemplate::new(503).set_body_string(
                r#"{"kind":"service_unavailable","error":"files_s3_not_configured"}"#,
            ),
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

#[tokio::test]
async fn test_files_download_items_passes_bearer_to_absolute_public_download_url() {
    let server = MockServer::start().await;
    let row = sample_download_row(
        format!("{}/v1/files/download?token=abc123", server.uri()),
        "BTCUSDT",
        "1m",
        "2026-02-21",
    );
    let destination_root = unique_temp_root("download-auth");

    Mock::given(method("GET"))
        .and(path("/v1/files/download"))
        .and(query_param("token", "abc123"))
        .and(header("authorization", "Bearer public-token"))
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("location", format!("{}/blob/btc.parquet", server.uri())),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/blob/btc.parquet"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"btc-file".to_vec()))
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http_with_bearer(&server.uri(), "public-token"))
        .expect("client");
    let out = client
        .files_download_items(&[row], Some(destination_root.as_path()))
        .await
        .expect("download should succeed");

    assert_eq!(out.len(), 1);
    assert_eq!(out[0].row.pair, "BTCUSDT");

    let written = fs::read(&out[0].destination_path).expect("file should be written");
    assert_eq!(written, b"btc-file");

    let _ = fs::remove_dir_all(&destination_root);
}

#[tokio::test]
async fn test_files_download_items_follows_redirect_and_writes_bytes_to_disk() {
    let server = MockServer::start().await;
    let row = sample_download_row(
        format!("{}/v1/files/download?token=eth123", server.uri()),
        "ETHUSDT",
        "5m",
        "2026-02-20",
    );
    let destination_root = unique_temp_root("download-redirect");

    Mock::given(method("GET"))
        .and(path("/v1/files/download"))
        .and(query_param("token", "eth123"))
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("location", format!("{}/blob/eth.parquet", server.uri())),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/blob/eth.parquet"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"eth-file".to_vec()))
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http_with_bearer(&server.uri(), "public-token"))
        .expect("client");
    let out = client
        .files_download_items(&[row], Some(destination_root.as_path()))
        .await
        .expect("download should succeed");

    assert_eq!(out[0].bytes_written, 8);
    assert!(Path::new(&out[0].destination_path).ends_with("day/ETHUSDT/5m/2026-02-20.parquet"));

    let written = fs::read(&out[0].destination_path).expect("file should be written");
    assert_eq!(written, b"eth-file");

    let _ = fs::remove_dir_all(&destination_root);
}

#[tokio::test]
async fn test_files_download_items_uses_default_tmp_mathilde_destination_when_omitted() {
    let server = MockServer::start().await;
    let row = sample_download_row(
        format!("{}/v1/files/download?token=sol123", server.uri()),
        "SDKTMPUSDT",
        "1m",
        "2099-01-01",
    );

    Mock::given(method("GET"))
        .and(path("/v1/files/download"))
        .and(query_param("token", "sol123"))
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("location", format!("{}/blob/sol.parquet", server.uri())),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/blob/sol.parquet"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"sol-file".to_vec()))
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http_with_bearer(&server.uri(), "public-token"))
        .expect("client");
    let out = client
        .files_download_items(&[row], None)
        .await
        .expect("download should succeed");

    assert_eq!(out.len(), 1);
    assert!(out[0].destination_path.starts_with("/tmp/mathilde/"));
    assert!(Path::new(&out[0].destination_path).ends_with("day/SDKTMPUSDT/1m/2099-01-01.parquet"));

    let _ = fs::remove_file(&out[0].destination_path);
    if let Some(parent) = Path::new(&out[0].destination_path).parent() {
        let _ = fs::remove_dir_all(parent.ancestors().nth(2).unwrap_or(parent));
    }
}

#[tokio::test]
async fn test_files_download_items_preserves_row_order_in_return_value() {
    let server = MockServer::start().await;
    let destination_root = unique_temp_root("download-order");
    let rows = vec![
        sample_download_row(
            format!("{}/v1/files/download?token=one", server.uri()),
            "BTCUSDT",
            "1m",
            "2026-02-21",
        ),
        sample_download_row(
            format!("{}/v1/files/download?token=two", server.uri()),
            "ETHUSDT",
            "5m",
            "2026-02-20",
        ),
    ];

    Mock::given(method("GET"))
        .and(path("/v1/files/download"))
        .and(query_param("token", "one"))
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("location", format!("{}/blob/one.parquet", server.uri())),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/blob/one.parquet"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"one".to_vec()))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/v1/files/download"))
        .and(query_param("token", "two"))
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("location", format!("{}/blob/two.parquet", server.uri())),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/blob/two.parquet"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"two".to_vec()))
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http_with_bearer(&server.uri(), "public-token"))
        .expect("client");
    let out = client
        .files_download_items(&rows, Some(destination_root.as_path()))
        .await
        .expect("download should succeed");

    assert_eq!(out.len(), 2);
    assert_eq!(out[0].row.pair, "BTCUSDT");
    assert_eq!(out[1].row.pair, "ETHUSDT");

    let _ = fs::remove_dir_all(&destination_root);
}

#[tokio::test]
async fn test_files_download_items_without_bearer_surfaces_typed_usage_error() {
    let server = MockServer::start().await;
    let row = sample_download_row(
        format!("{}/v1/files/download?token=abc123", server.uri()),
        "BTCUSDT",
        "1m",
        "2026-02-21",
    );

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .files_download_items(&[row], None)
        .await
        .expect_err("expected usage error");

    match err {
        SdkError::UnsupportedOrUnprovedUsage { message } => {
            assert!(message.contains("requires bearer auth"));
        }
        other => panic!("expected UnsupportedOrUnprovedUsage error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_files_download_items_rejects_foreign_origin_absolute_url() {
    let server = MockServer::start().await;
    let row = sample_download_row(
        "https://evil.example.com/v1/files/download?token=abc123".to_string(),
        "BTCUSDT",
        "1m",
        "2026-02-21",
    );

    let client = AggregatorClient::new(config_for_http_with_bearer(&server.uri(), "public-token"))
        .expect("client");
    let err = client
        .files_download_items(&[row], None)
        .await
        .expect_err("foreign-origin download URL should fail");

    match err {
        SdkError::RequestBuild { message } => {
            assert!(message.contains("does not match configured http origin"));
        }
        other => panic!("expected RequestBuild error, got {other:?}"),
    }
}
