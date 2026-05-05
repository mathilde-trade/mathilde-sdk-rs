use crate::core::auth::BearerToken;
use crate::core::config::{HttpTransportConfig, PrimitivesConfig};
use crate::core::error::SdkError;
use crate::systems::primitives::{FilesDownloadsRequest, FilesDownloadsRow, Primitives};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config_for_http(base_url: &str) -> PrimitivesConfig {
    PrimitivesConfig {
        http: HttpTransportConfig::new(base_url).expect("valid test url"),
        grpc: None,
        ws: None,
        bearer_token: None,
    }
}

fn config_for_http_with_bearer(base_url: &str, bearer: &str) -> PrimitivesConfig {
    PrimitivesConfig {
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

    Mock::given(method("POST"))
        .and(path("/v1/files/downloads"))
        .and(body_json(serde_json::json!({
            "period": "day",
            "pairs": ["BTCUSDT", "ETHUSDT"],
            "tfs": ["1m", "5m"],
            "start_label_utc": "2026-02-20",
            "end_label_utc": "2026-02-21",
            "order": "desc"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "rows": [{
                "period": "day",
                "pair": "BTCUSDT",
                "tf": "1m",
                "label_utc": "2026-02-21",
                "url": "https://example.invalid/presigned-1",
                "expires_at_utc": "2026-02-21T00:05:00Z"
            }]
        })))
        .mount(&server)
        .await;

    let client = Primitives::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .files_downloads(&request)
        .await
        .expect("files_downloads success");

    assert_eq!(out.rows.len(), 1);
    assert_eq!(out.rows[0].pair, "BTCUSDT");
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
    let destination_root = unique_temp_root("primitives-download");

    Mock::given(method("GET"))
        .and(path("/v1/files/download"))
        .and(query_param("token", "eth123"))
        .and(header("authorization", "Bearer public-token"))
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

    let client = Primitives::new(config_for_http_with_bearer(&server.uri(), "public-token"))
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
async fn test_files_download_items_rejects_foreign_origin_absolute_url() {
    let client = Primitives::new(config_for_http_with_bearer(
        "https://primitives.api.mathilde.dev",
        "public-token",
    ))
    .expect("client");
    let row = sample_download_row(
        "https://evil.example.com/v1/files/download?token=x".to_string(),
        "BTCUSDT",
        "1m",
        "2026-02-21",
    );

    let error = client
        .files_download_items(&[row], None)
        .await
        .expect_err("foreign origin must fail");

    match error {
        SdkError::RequestBuild { message } => {
            assert!(message.contains("does not match configured http origin"));
        }
        other => panic!("expected RequestBuild error, got {other:?}"),
    }
}
