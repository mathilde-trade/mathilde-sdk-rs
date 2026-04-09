use crate::core::auth::{BearerToken, apply_bearer_auth};
use crate::core::config::{AggregatorConfig, HttpTransportConfig};
use crate::core::error::SdkError;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1 as proto;
use crate::systems::aggregator::{AggregatorClient, LatestBarsRequest, LatestBarsResponse};
use crate::systems::types::{BarsView, ExcludeSource, HttpFormat, LatestMode, Timeframe};
use prost::Message;
use reqwest::header::{AUTHORIZATION, HeaderMap};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config_for_http(base_url: &str) -> AggregatorConfig {
    AggregatorConfig {
        http: Some(HttpTransportConfig::new(base_url).expect("valid test url")),
        grpc: None,
        ws: None,
        bearer_token: None,
    }
}

fn proto_bar_min(pair: &str) -> proto::BarRowV1 {
    proto::BarRowV1 {
        pair: pair.to_string(),
        tf: "1m".to_string(),
        s_ms: 1770000000000,
        e_ms: 1770000060000,
        s_utc: Some("2026-02-02T00:00:00Z".to_string()),
        e_utc: Some("2026-02-02T00:01:00Z".to_string()),
        o: 100.0,
        h: 101.0,
        l: 99.5,
        c: 100.5,
        v: 12.34,
        quote_v: Some(1234.56),
        taker_known_v: Some(6.17),
        taker_signed_v: Some(1.23),
        taker_known_quote_v: Some(617.28),
        taker_signed_quote_v: Some(123.45),
        taker_known_n: Some(18),
        taker_signed_n: Some(3),
        vw: Some(100.21),
        n: None,
        coverage_ratio: None,
        at_ms: None,
        metadata: None,
    }
}

fn proto_metadata() -> proto::BarMetadataV1 {
    proto::BarMetadataV1 {
        source: "frontier".to_string(),
        process: None,
        venues_expected: vec![
            "binance".to_string(),
            "bybit".to_string(),
            "okx".to_string(),
        ],
        venues_with_trades: vec!["binance".to_string()],
        ingested_at_ms: Some(1770000060101),
        ingested_at_utc: Some("2026-02-02T00:01:00Z".to_string()),
        target_ingested_at_ms: Some(1770000060150),
        target_ingested_at_utc: Some("2026-02-02T00:01:00Z".to_string()),
        built_at_ms: None,
        built_at_utc: None,
        committed_at_ms: Some(1770000060102),
        committed_at_utc: Some("2026-02-02T00:01:00Z".to_string()),
        harmonized_at_ms: None,
        harmonized_at_utc: None,
        recomputed_at_ms: None,
        recomputed_at_utc: None,
        recomputed_reason: None,
        covered_1m_count: None,
        expected_1m_count: None,
        coverage_ratio: None,
        inputs_source_counts_frontier: None,
        inputs_source_counts_api: None,
        inputs_source_counts_synthetic: None,
        inputs_source_counts_fix_data: None,
        frontier_5s_inputs_coverage_ratio: None,
        frontier_5s_expected: Some(12),
        frontier_5s_synth_n: Some(0),
        frontier_5s_synth_ratio: Some(0.0),
        frontier_5s_trade_n: Some(12),
        frontier_5s_trade_ratio: Some(1.0),
        age_ms: None,
    }
}

fn proto_latest_response_min() -> proto::BarsLatestResponseV1 {
    proto::BarsLatestResponseV1 {
        watermark_end_ms: 1770000060000,
        close_end_ms: 1770000060000,
        latest_mode: "exact_watermark".to_string(),
        view: proto::BarsViewV1::Min as i32,
        rows: vec![proto::BarsPresentRowV1 {
            bar: Some(proto_bar_min("BTCUSDT")),
            age_ms: Some(101),
        }],
        missing_pairs: vec![],
        excluded_sources: vec!["no_trade_fill".to_string()],
        excluded_rows_total: Some(1),
        excluded_rows_by_source: vec![proto::ExcludedSourceCountV1 {
            source: "no_trade_fill".to_string(),
            count: 1,
        }],
    }
}

fn proto_latest_response_full() -> proto::BarsLatestResponseV1 {
    let mut bar = proto_bar_min("BTCUSDT");
    bar.metadata = Some(proto_metadata());

    proto::BarsLatestResponseV1 {
        watermark_end_ms: 1770000060000,
        close_end_ms: 1770000060000,
        latest_mode: "exact_watermark".to_string(),
        view: proto::BarsViewV1::Full as i32,
        rows: vec![proto::BarsPresentRowV1 {
            bar: Some(bar),
            age_ms: Some(101),
        }],
        missing_pairs: vec![],
        excluded_sources: vec!["no_trade_fill".to_string()],
        excluded_rows_total: Some(0),
        excluded_rows_by_source: vec![],
    }
}

#[test]
fn test_config_rejects_malformed_http_base_url() {
    let err = HttpTransportConfig::new("not a url").expect_err("expected invalid url");
    match err {
        SdkError::InvalidUrl { .. } => {}
        other => panic!("expected invalid url error, got {other:?}"),
    }
}

#[test]
fn test_auth_helper_injects_bearer_token_when_present() {
    let token = BearerToken::new("abc123").expect("valid token");
    let headers = apply_bearer_auth(HeaderMap::new(), Some(&token)).expect("header injection");
    let value = headers
        .get(AUTHORIZATION)
        .expect("authorization header must exist");
    assert_eq!(value.to_str().expect("ascii header"), "Bearer abc123");
}

#[test]
fn test_auth_helper_omits_bearer_token_when_absent() {
    let headers = apply_bearer_auth(HeaderMap::new(), None).expect("header passthrough");
    assert!(headers.get(AUTHORIZATION).is_none());
}

#[tokio::test]
async fn test_docs_system_forms_correct_path_and_decodes_payload() {
    let server = MockServer::start().await;
    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "slug": "aggregator",
        "kind": "system",
        "title": "Aggregator",
        "format": "markdown",
        "content": "# Aggregator"
    }));

    Mock::given(method("GET"))
        .and(path("/v1/docs/system"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let doc = client.docs_system().await.expect("docs_system success");

    assert_eq!(doc.slug, "aggregator");
    assert_eq!(doc.kind, "system");
    assert_eq!(doc.format, "markdown");
}

#[tokio::test]
async fn test_latest_bars_uses_post_and_serializes_body_and_decodes_response() {
    let server = MockServer::start().await;
    let request = LatestBarsRequest {
        pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        exclude_sources: Some(vec![ExcludeSource::NoTradeFill, ExcludeSource::FixData]),
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    };

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "watermark_end_ms": 1770000060000i64,
        "close_end_ms": 1770000060000i64,
        "latest_mode": "exact_watermark",
        "view": "min",
        "rows": [{
            "pair": "BTCUSDT",
            "tf": "1m",
            "open_ms": 1770000000000i64,
            "close_ms": 1770000060000i64,
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
            "age_ms": 101
        }],
        "missing_pairs": [],
        "excluded_sources": ["no_trade_fill", "fix-data"],
        "excluded_rows_total": 1,
        "excluded_rows_by_source": [{"source": "no_trade_fill", "count": 1}]
    }));

    let expected_body = serde_json::json!({
        "pairs": "BTCUSDT,ETHUSDT",
        "tf": "1m",
        "latest_mode": "exact_watermark",
        "exclude_sources": ["no_trade_fill", "fix-data"],
        "metadata": false,
        "format": "json"
    });

    Mock::given(method("POST"))
        .and(path("/v1/bars/latest"))
        .and(body_json(expected_body))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .latest_bars(&request)
        .await
        .expect("latest bars success");

    match out {
        LatestBarsResponse::Min(out) => {
            assert_eq!(out.latest_mode, LatestMode::ExactWatermark);
            assert_eq!(out.view, BarsView::Min);
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].bar.pair, "BTCUSDT");
        }
        LatestBarsResponse::Full(other) => {
            panic!("expected min latest bars response, got full: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_latest_bars_metadata_true_decodes_full_response() {
    let server = MockServer::start().await;
    let request = LatestBarsRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        exclude_sources: Some(vec![ExcludeSource::NoTradeFill]),
        metadata: Some(true),
        format: Some(HttpFormat::Json),
    };

    let response = ResponseTemplate::new(200)
        .insert_header("content-type", "application/json")
        .set_body_string(
            r#"{
                "watermark_end_ms": 1770000060000,
                "close_end_ms": 1770000060000,
                "latest_mode": "exact_watermark",
                "view": "full",
                "rows": [{
                    "pair": "BTCUSDT",
                    "tf": "1m",
                    "open_ms": 1770000000000,
                    "close_ms": 1770000060000,
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
                    "metadata": {
                        "source": "frontier",
                        "process": null,
                        "venues_expected": ["binance", "bybit", "okx"],
                        "venues_with_trades": ["binance"],
                        "ingested_at_ms": 1770000060101,
                        "ingested_at_utc": "2026-02-02T00:01:00Z",
                        "target_ingested_at_ms": 1770000060150,
                        "target_ingested_at_utc": "2026-02-02T00:01:00Z",
                        "built_at_ms": null,
                        "built_at_utc": null,
                        "committed_at_ms": 1770000060102,
                        "committed_at_utc": "2026-02-02T00:01:00Z",
                        "harmonized_at_ms": null,
                        "harmonized_at_utc": null,
                        "recomputed_at_ms": null,
                        "recomputed_at_utc": null,
                        "recomputed_reason": null,
                        "covered_1m_count": null,
                        "expected_1m_count": null,
                        "coverage_ratio": null,
                        "inputs_source_counts_frontier": null,
                        "inputs_source_counts_api": null,
                        "inputs_source_counts_synthetic": null,
                        "inputs_source_counts_fix_data": null,
                        "frontier_5s_inputs_coverage_ratio": null,
                        "frontier_5s_expected": 12,
                        "frontier_5s_synth_n": 0,
                        "frontier_5s_synth_ratio": 0.0,
                        "frontier_5s_trade_n": 12,
                        "frontier_5s_trade_ratio": 1.0
                    },
                    "age_ms": 101
                }],
                "missing_pairs": [],
                "excluded_sources": ["no_trade_fill"],
                "excluded_rows_total": 0,
                "excluded_rows_by_source": []
            }"#,
        );

    let expected_body = serde_json::json!({
        "pairs": "BTCUSDT",
        "tf": "1m",
        "latest_mode": "exact_watermark",
        "exclude_sources": ["no_trade_fill"],
        "metadata": true,
        "format": "json"
    });

    Mock::given(method("POST"))
        .and(path("/v1/bars/latest"))
        .and(body_json(expected_body))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .latest_bars(&request)
        .await
        .expect("latest bars full success");

    match out {
        LatestBarsResponse::Full(out) => {
            assert_eq!(out.latest_mode, LatestMode::ExactWatermark);
            assert_eq!(out.view, BarsView::Full);
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].bar.pair, "BTCUSDT");
            assert_eq!(out.rows[0].bar.metadata.source, "frontier");
            assert_eq!(
                out.rows[0].bar.metadata.venues_expected,
                Some(vec![
                    "binance".to_string(),
                    "bybit".to_string(),
                    "okx".to_string()
                ])
            );
            assert_eq!(out.rows[0].bar.metadata.frontier_5s_expected, Some(12));
        }
        LatestBarsResponse::Min(other) => {
            panic!("expected full latest bars response, got min: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_latest_bars_omitted_format_still_uses_json_branch() {
    let server = MockServer::start().await;
    let request = LatestBarsRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        exclude_sources: None,
        metadata: Some(false),
        format: None,
    };

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "watermark_end_ms": 1770000060000i64,
        "close_end_ms": 1770000060000i64,
        "latest_mode": "exact_watermark",
        "view": "min",
        "rows": [],
        "missing_pairs": ["BTCUSDT"],
        "excluded_sources": [],
        "excluded_rows_total": 0,
        "excluded_rows_by_source": []
    }));

    let expected_body = serde_json::json!({
        "pairs": "BTCUSDT",
        "tf": "1m",
        "latest_mode": "exact_watermark",
        "exclude_sources": null,
        "metadata": false,
        "format": null
    });

    Mock::given(method("POST"))
        .and(path("/v1/bars/latest"))
        .and(body_json(expected_body))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .latest_bars(&request)
        .await
        .expect("latest bars success");

    match out {
        LatestBarsResponse::Min(out) => {
            assert_eq!(out.view, BarsView::Min);
            assert_eq!(out.missing_pairs, vec!["BTCUSDT".to_string()]);
        }
        LatestBarsResponse::Full(other) => {
            panic!("expected min latest bars response, got full: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_latest_bars_format_protobuf_decodes_min_response() {
    let server = MockServer::start().await;
    let request = LatestBarsRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        exclude_sources: Some(vec![ExcludeSource::NoTradeFill]),
        metadata: Some(false),
        format: Some(HttpFormat::Protobuf),
    };

    let body = proto_latest_response_min().encode_to_vec();

    let expected_body = serde_json::json!({
        "pairs": "BTCUSDT",
        "tf": "1m",
        "latest_mode": "exact_watermark",
        "exclude_sources": ["no_trade_fill"],
        "metadata": false,
        "format": "protobuf"
    });

    Mock::given(method("POST"))
        .and(path("/v1/bars/latest"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(body),
        )
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .latest_bars(&request)
        .await
        .expect("protobuf latest bars min success");

    match out {
        LatestBarsResponse::Min(out) => {
            assert_eq!(out.view, BarsView::Min);
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].bar.pair, "BTCUSDT");
            assert_eq!(out.rows[0].age_ms, 101);
        }
        LatestBarsResponse::Full(other) => {
            panic!("expected min latest bars protobuf response, got full: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_latest_bars_format_protobuf_decodes_full_response() {
    let server = MockServer::start().await;
    let request = LatestBarsRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        exclude_sources: Some(vec![ExcludeSource::NoTradeFill]),
        metadata: Some(true),
        format: Some(HttpFormat::Protobuf),
    };

    let body = proto_latest_response_full().encode_to_vec();

    let expected_body = serde_json::json!({
        "pairs": "BTCUSDT",
        "tf": "1m",
        "latest_mode": "exact_watermark",
        "exclude_sources": ["no_trade_fill"],
        "metadata": true,
        "format": "protobuf"
    });

    Mock::given(method("POST"))
        .and(path("/v1/bars/latest"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(body),
        )
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .latest_bars(&request)
        .await
        .expect("protobuf latest bars full success");

    match out {
        LatestBarsResponse::Full(out) => {
            assert_eq!(out.view, BarsView::Full);
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].bar.pair, "BTCUSDT");
            assert_eq!(out.rows[0].bar.metadata.source, "frontier");
            assert_eq!(out.rows[0].bar.metadata.frontier_5s_expected, Some(12));
        }
        LatestBarsResponse::Min(other) => {
            panic!("expected full latest bars protobuf response, got min: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_latest_bars_invalid_protobuf_is_contract_drift() {
    let server = MockServer::start().await;
    let request = LatestBarsRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        exclude_sources: None,
        metadata: Some(false),
        format: Some(HttpFormat::Protobuf),
    };

    let expected_body = serde_json::json!({
        "pairs": "BTCUSDT",
        "tf": "1m",
        "latest_mode": "exact_watermark",
        "exclude_sources": null,
        "metadata": false,
        "format": "protobuf"
    });

    Mock::given(method("POST"))
        .and(path("/v1/bars/latest"))
        .and(body_json(expected_body))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(vec![0xff, 0x00, 0x7f]),
        )
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .latest_bars(&request)
        .await
        .expect_err("expected protobuf decode failure");

    match err {
        SdkError::ContractDrift { .. } => {}
        other => panic!("expected contract drift error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_non_success_http_status_is_typed_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/docs/system"))
        .respond_with(ResponseTemplate::new(403).set_body_string("{\"kind\":\"forbidden\"}"))
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let err = client.docs_system().await.expect_err("expected forbidden");

    match err {
        SdkError::HttpStatus { status, .. } => assert_eq!(status, 403),
        other => panic!("expected http status error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_invalid_json_is_decode_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/docs/system"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_string("{not-json"),
        )
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .docs_system()
        .await
        .expect_err("expected decode failure");

    match err {
        SdkError::Decode { .. } => {}
        other => panic!("expected decode error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_docs_system_sends_bearer_auth_when_present() {
    let server = MockServer::start().await;
    let token = BearerToken::new("public-token").expect("valid token");
    let config = AggregatorConfig {
        http: Some(HttpTransportConfig::new(server.uri()).expect("valid test url")),
        grpc: None,
        ws: None,
        bearer_token: Some(token),
    };

    Mock::given(method("GET"))
        .and(path("/v1/docs/system"))
        .and(header("authorization", "Bearer public-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "slug": "aggregator",
            "kind": "system",
            "title": "Aggregator",
            "format": "markdown",
            "content": "# Aggregator"
        })))
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config).expect("client");
    let doc = client.docs_system().await.expect("docs_system success");
    assert_eq!(doc.slug, "aggregator");
}
