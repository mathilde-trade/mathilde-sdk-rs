use crate::core::config::{AggregatorConfig, HttpTransportConfig};
use crate::core::error::SdkError;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1 as proto;
use crate::systems::aggregator::{AggregatorClient, RangeBarsRequest, RangeBarsResponse};
use crate::systems::types::{AlignMode, ExcludeSource, HttpFormat, Timeframe};
use prost::Message;
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

fn proto_range_response_min() -> proto::BarsRangeResponseV1 {
    proto::BarsRangeResponseV1 {
        rows: vec![proto_bar_min("BTCUSDT")],
        next_cursor: Some("cursor-1".to_string()),
        excluded_sources: vec!["no_trade_fill".to_string()],
        excluded_rows_total: Some(1),
        excluded_rows_by_source: vec![proto::ExcludedSourceCountV1 {
            source: "no_trade_fill".to_string(),
            count: 1,
        }],
        close_end_ms: 1770003600000,
    }
}

fn proto_range_response_full() -> proto::BarsRangeResponseV1 {
    let mut bar = proto_bar_min("BTCUSDT");
    bar.metadata = Some(proto_metadata());

    proto::BarsRangeResponseV1 {
        rows: vec![bar],
        next_cursor: None,
        excluded_sources: vec!["no_trade_fill".to_string()],
        excluded_rows_total: Some(0),
        excluded_rows_by_source: vec![],
        close_end_ms: 1770003600000,
    }
}

#[tokio::test]
async fn test_range_bars_uses_post_and_normalizes_time_inputs_and_decodes_min_json() {
    let server = MockServer::start().await;
    let request = RangeBarsRequest {
        pairs: "BTCUSDT,ETHUSDT".to_string(),
        tf: Timeframe::M1,
        align_mode: Some(AlignMode::Floor),
        close_start: Some("2026-02-02T00:00:00Z".into()),
        cursor: None,
        close_end: Some(1770003600000_i64.into()),
        limit: Some(1000),
        exclude_sources: Some(vec![ExcludeSource::NoTradeFill, ExcludeSource::FixData]),
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    };

    let expected_body = serde_json::json!({
        "pairs": "BTCUSDT,ETHUSDT",
        "tf": "1m",
        "align_mode": "floor",
        "close_start_ms": 1769990400000i64,
        "cursor": null,
        "close_end_ms": 1770003600000i64,
        "limit": 1000,
        "exclude_sources": ["no_trade_fill", "fix-data"],
        "metadata": false,
        "format": "json"
    });

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
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
            "n": null
        }],
        "close_end_ms": 1770003600000i64,
        "next_cursor": "cursor-1",
        "excluded_sources": ["no_trade_fill", "fix-data"],
        "excluded_rows_total": 1,
        "excluded_rows_by_source": [{"source": "no_trade_fill", "count": 1}]
    }));

    Mock::given(method("POST"))
        .and(path("/v1/bars/range"))
        .and(body_json(expected_body))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let out = client.range_bars(&request).await.expect("range success");

    match out {
        RangeBarsResponse::Min(out) => {
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].pair, "BTCUSDT");
            assert_eq!(out.next_cursor.as_deref(), Some("cursor-1"));
        }
        RangeBarsResponse::Full(other) => {
            panic!("expected min range response, got full: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_range_bars_omitted_close_end_serializes_as_absent_and_decodes_full_json() {
    let server = MockServer::start().await;
    let request = RangeBarsRequest {
        pairs: "BTCUSDT".to_string(),
        tf: Timeframe::M1,
        align_mode: None,
        close_start: Some("2026-02-02:00:00".into()),
        cursor: None,
        close_end: None,
        limit: Some(10),
        exclude_sources: Some(vec![ExcludeSource::NoTradeFill]),
        metadata: Some(true),
        format: Some(HttpFormat::Json),
    };

    let expected_body = serde_json::json!({
        "pairs": "BTCUSDT",
        "tf": "1m",
        "align_mode": null,
        "close_start_ms": 1769990400000i64,
        "cursor": null,
        "close_end_ms": null,
        "limit": 10,
        "exclude_sources": ["no_trade_fill"],
        "metadata": true,
        "format": "json"
    });

    let response = ResponseTemplate::new(200)
        .insert_header("content-type", "application/json")
        .set_body_string(
            r#"{
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
                    }
                }],
                "close_end_ms": 1770003600000,
                "next_cursor": null,
                "excluded_sources": ["no_trade_fill"],
                "excluded_rows_total": 0,
                "excluded_rows_by_source": []
            }"#,
        );

    Mock::given(method("POST"))
        .and(path("/v1/bars/range"))
        .and(body_json(expected_body))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let out = client.range_bars(&request).await.expect("range success");

    match out {
        RangeBarsResponse::Full(out) => {
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].pair, "BTCUSDT");
            assert_eq!(out.rows[0].metadata.source, "frontier");
        }
        RangeBarsResponse::Min(other) => {
            panic!("expected full range response, got min: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_range_bars_protobuf_decodes_min_response() {
    let server = MockServer::start().await;
    let request = RangeBarsRequest {
        pairs: "BTCUSDT".to_string(),
        tf: Timeframe::M1,
        align_mode: Some(AlignMode::Exact),
        close_start: Some(1770000000000_i64.into()),
        cursor: Some("cursor-1".to_string()),
        close_end: Some(1770003600000_i64.into()),
        limit: Some(100),
        exclude_sources: Some(vec![ExcludeSource::NoTradeFill]),
        metadata: Some(false),
        format: Some(HttpFormat::Protobuf),
    };

    let body = proto_range_response_min().encode_to_vec();
    let response = ResponseTemplate::new(200)
        .insert_header("content-type", "application/x-protobuf")
        .set_body_bytes(body);

    Mock::given(method("POST"))
        .and(path("/v1/bars/range"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .range_bars(&request)
        .await
        .expect("protobuf range success");

    match out {
        RangeBarsResponse::Min(out) => {
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].pair, "BTCUSDT");
            assert_eq!(out.next_cursor.as_deref(), Some("cursor-1"));
        }
        RangeBarsResponse::Full(other) => {
            panic!("expected min protobuf range response, got full: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_range_bars_protobuf_decodes_full_response() {
    let server = MockServer::start().await;
    let request = RangeBarsRequest {
        pairs: "BTCUSDT".to_string(),
        tf: Timeframe::M1,
        align_mode: Some(AlignMode::Exact),
        close_start: Some(1770000000000_i64.into()),
        cursor: None,
        close_end: Some(1770003600000_i64.into()),
        limit: Some(100),
        exclude_sources: Some(vec![ExcludeSource::NoTradeFill]),
        metadata: Some(true),
        format: Some(HttpFormat::Protobuf),
    };

    let body = proto_range_response_full().encode_to_vec();
    let response = ResponseTemplate::new(200)
        .insert_header("content-type", "application/x-protobuf")
        .set_body_bytes(body);

    Mock::given(method("POST"))
        .and(path("/v1/bars/range"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .range_bars(&request)
        .await
        .expect("protobuf full range success");

    match out {
        RangeBarsResponse::Full(out) => {
            assert_eq!(out.rows.len(), 1);
            assert_eq!(out.rows[0].metadata.source, "frontier");
        }
        RangeBarsResponse::Min(other) => {
            panic!("expected full protobuf range response, got min: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_range_bars_non_success_http_status_returns_typed_error() {
    let server = MockServer::start().await;
    let request = RangeBarsRequest {
        pairs: "BTCUSDT".to_string(),
        tf: Timeframe::M1,
        align_mode: None,
        close_start: Some(1770000000000_i64.into()),
        cursor: None,
        close_end: Some(1770003600000_i64.into()),
        limit: Some(10),
        exclude_sources: None,
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    };

    Mock::given(method("POST"))
        .and(path("/v1/bars/range"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_string("cursor is invalid when close_start_ms is absent"),
        )
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .range_bars(&request)
        .await
        .expect_err("expected http status error");

    match err {
        SdkError::HttpStatus { status, body } => {
            assert_eq!(status, 400);
            assert!(body.contains("cursor"));
        }
        other => panic!("expected http status error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_range_bars_invalid_json_returns_decode_error() {
    let server = MockServer::start().await;
    let request = RangeBarsRequest {
        pairs: "BTCUSDT".to_string(),
        tf: Timeframe::M1,
        align_mode: None,
        close_start: Some(1770000000000_i64.into()),
        cursor: None,
        close_end: Some(1770003600000_i64.into()),
        limit: Some(10),
        exclude_sources: None,
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    };

    Mock::given(method("POST"))
        .and(path("/v1/bars/range"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_string("{not-json"),
        )
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .range_bars(&request)
        .await
        .expect_err("expected decode error");

    match err {
        SdkError::Decode { .. } => {}
        other => panic!("expected decode error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_range_bars_invalid_protobuf_returns_contract_drift() {
    let server = MockServer::start().await;
    let request = RangeBarsRequest {
        pairs: "BTCUSDT".to_string(),
        tf: Timeframe::M1,
        align_mode: None,
        close_start: Some(1770000000000_i64.into()),
        cursor: None,
        close_end: Some(1770003600000_i64.into()),
        limit: Some(10),
        exclude_sources: None,
        metadata: Some(false),
        format: Some(HttpFormat::Protobuf),
    };

    Mock::given(method("POST"))
        .and(path("/v1/bars/range"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(b"not-protobuf".to_vec()),
        )
        .mount(&server)
        .await;

    let client = AggregatorClient::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .range_bars(&request)
        .await
        .expect_err("expected contract drift error");

    match err {
        SdkError::ContractDrift { message } => {
            assert!(message.contains("protobuf decode failed"));
        }
        other => panic!("expected contract drift error, got {other:?}"),
    }
}
