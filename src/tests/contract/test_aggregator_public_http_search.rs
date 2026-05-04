use crate::core::config::{AggregatorConfig, HttpTransportConfig};
use crate::core::error::SdkError;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1 as proto;
use crate::systems::aggregator::{Aggregator, SearchBarsRequest, SearchBarsResponse};
use crate::systems::types::{HttpFormat, Timeframe};
use prost::Message;
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn config_for_http(base_url: &str) -> AggregatorConfig {
    AggregatorConfig {
        http: HttpTransportConfig::new(base_url).expect("valid test url"),
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
        coverage_ratio: Some(0.95),
        at_ms: Some(1770000060005),
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
        age_ms: Some(202),
    }
}

fn proto_search_response_min() -> proto::BarsSearchResponseV1 {
    proto::BarsSearchResponseV1 {
        hits: vec![1770000060000, 1770000120000],
        evaluated_rows: vec![proto_bar_min("BTCUSDT")],
        returned_hits: 2,
        effective_hits_limit: 500,
        truncated: false,
        predicate_pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        predicate_normalized: "BTCUSDT.c > ETHUSDT.c * 1.5".to_string(),
        next_cursor: Some("cursor-1".to_string()),
        done: false,
    }
}

fn proto_search_response_full() -> proto::BarsSearchResponseV1 {
    let mut bar = proto_bar_min("BTCUSDT");
    bar.metadata = Some(proto_metadata());

    proto::BarsSearchResponseV1 {
        hits: vec![1770000060000],
        evaluated_rows: vec![bar],
        returned_hits: 1,
        effective_hits_limit: 500,
        truncated: false,
        predicate_pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        predicate_normalized: "BTCUSDT.c > ETHUSDT.c * 1.5".to_string(),
        next_cursor: None,
        done: true,
    }
}

#[tokio::test]
async fn test_search_bars_uses_post_and_normalizes_time_inputs_and_decodes_min_json() {
    let server = MockServer::start().await;
    let request = SearchBarsRequest {
        tf: Timeframe::M1,
        close_start: "2026-02-02T00:00:00Z".into(),
        close_end: Some(1770003600000_i64.into()),
        cursor: None,
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5 && ETHUSDT.v > 10".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        metadata: Some(false),
        max_hits: Some(500),
        format: Some(HttpFormat::Json),
    };

    let expected_body = serde_json::json!({
        "tf": "1m",
        "close_start_ms": 1769990400000i64,
        "close_end_ms": 1770003600000i64,
        "cursor": null,
        "predicate": "BTCUSDT.c > ETHUSDT.c * 1.5 && ETHUSDT.v > 10",
        "evaluate_pair": "BTCUSDT",
        "metadata": false,
        "max_hits": 500,
        "format": "json"
    });

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "hits": [1770000060000i64, 1770000120000i64],
        "evaluated_rows": [{
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
        "next_cursor": "cursor-1",
        "done": false,
        "returned_hits": 2,
        "effective_hits_limit": 500,
        "truncated": false,
        "predicate_pairs": ["BTCUSDT", "ETHUSDT"],
        "predicate_normalized": "BTCUSDT.c > ETHUSDT.c * 1.5"
    }));

    Mock::given(method("POST"))
        .and(path("/v1/bars/search"))
        .and(body_json(expected_body))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let out = client.search(&request).await.expect("search success");

    match out {
        SearchBarsResponse::Min(out) => {
            assert_eq!(out.hits.len(), 2);
            assert_eq!(out.evaluated_rows.as_ref().expect("rows").len(), 1);
            assert_eq!(out.next_cursor.as_deref(), Some("cursor-1"));
            assert!(!out.done);
        }
        SearchBarsResponse::Full(other) => {
            panic!("expected min search response, got full: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_search_bars_omitted_close_end_serializes_as_absent_and_decodes_full_json() {
    let server = MockServer::start().await;
    let request = SearchBarsRequest {
        tf: Timeframe::M1,
        close_start: "2026-02-02:00:00".into(),
        close_end: None,
        cursor: None,
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        metadata: Some(true),
        max_hits: Some(100),
        format: Some(HttpFormat::Json),
    };

    let expected_body = serde_json::json!({
        "tf": "1m",
        "close_start_ms": 1769990400000i64,
        "close_end_ms": null,
        "cursor": null,
        "predicate": "BTCUSDT.c > ETHUSDT.c * 1.5",
        "evaluate_pair": "BTCUSDT",
        "metadata": true,
        "max_hits": 100,
        "format": "json"
    });

    let response = ResponseTemplate::new(200)
        .insert_header("content-type", "application/json")
        .set_body_string(
            r#"{
                "hits": [1770000060000],
                "evaluated_rows": [{
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
                "next_cursor": null,
                "done": true,
                "returned_hits": 1,
                "effective_hits_limit": 100,
                "truncated": false,
                "predicate_pairs": ["BTCUSDT", "ETHUSDT"],
                "predicate_normalized": "BTCUSDT.c > ETHUSDT.c * 1.5"
            }"#,
        );

    Mock::given(method("POST"))
        .and(path("/v1/bars/search"))
        .and(body_json(expected_body))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let out = client.search(&request).await.expect("search success");

    match out {
        SearchBarsResponse::Full(out) => {
            assert_eq!(out.hits.len(), 1);
            assert_eq!(
                out.evaluated_rows.as_ref().expect("rows")[0]
                    .metadata
                    .source,
                "frontier"
            );
            assert!(out.done);
        }
        SearchBarsResponse::Min(other) => {
            panic!("expected full search response, got min: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_search_bars_protobuf_decodes_min_response() {
    let server = MockServer::start().await;
    let request = SearchBarsRequest {
        tf: Timeframe::M1,
        close_start: 1770000000000_i64.into(),
        close_end: Some(1770003600000_i64.into()),
        cursor: Some("cursor-1".to_string()),
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        metadata: Some(false),
        max_hits: Some(100),
        format: Some(HttpFormat::Protobuf),
    };

    let body = proto_search_response_min().encode_to_vec();
    let response = ResponseTemplate::new(200)
        .insert_header("content-type", "application/x-protobuf")
        .set_body_bytes(body);

    Mock::given(method("POST"))
        .and(path("/v1/bars/search"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .search(&request)
        .await
        .expect("protobuf search success");

    match out {
        SearchBarsResponse::Min(out) => {
            assert_eq!(out.hits.len(), 2);
            assert_eq!(
                out.evaluated_rows.as_ref().expect("rows")[0].pair,
                "BTCUSDT"
            );
            assert_eq!(
                out.evaluated_rows.as_ref().expect("rows")[0].coverage_ratio,
                Some(0.95)
            );
            assert_eq!(
                out.evaluated_rows.as_ref().expect("rows")[0].at_ms,
                Some(1770000060005)
            );
            assert_eq!(out.next_cursor.as_deref(), Some("cursor-1"));
        }
        SearchBarsResponse::Full(other) => {
            panic!("expected min protobuf search response, got full: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_search_bars_protobuf_decodes_full_response() {
    let server = MockServer::start().await;
    let request = SearchBarsRequest {
        tf: Timeframe::M1,
        close_start: 1770000000000_i64.into(),
        close_end: Some(1770003600000_i64.into()),
        cursor: None,
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        metadata: Some(true),
        max_hits: Some(100),
        format: Some(HttpFormat::Protobuf),
    };

    let body = proto_search_response_full().encode_to_vec();
    let response = ResponseTemplate::new(200)
        .insert_header("content-type", "application/x-protobuf")
        .set_body_bytes(body);

    Mock::given(method("POST"))
        .and(path("/v1/bars/search"))
        .respond_with(response)
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let out = client
        .search(&request)
        .await
        .expect("protobuf full search success");

    match out {
        SearchBarsResponse::Full(out) => {
            assert_eq!(out.hits.len(), 1);
            assert_eq!(
                out.evaluated_rows.as_ref().expect("rows")[0]
                    .metadata
                    .source,
                "frontier"
            );
            assert_eq!(
                out.evaluated_rows.as_ref().expect("rows")[0]
                    .metadata
                    .age_ms,
                Some(202)
            );
        }
        SearchBarsResponse::Min(other) => {
            panic!("expected full protobuf search response, got min: {other:?}")
        }
    }
}

#[tokio::test]
async fn test_search_bars_non_success_http_status_returns_typed_error() {
    let server = MockServer::start().await;
    let request = SearchBarsRequest {
        tf: Timeframe::M1,
        close_start: 1770000000000_i64.into(),
        close_end: Some(1770003600000_i64.into()),
        cursor: None,
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5".to_string(),
        evaluate_pair: None,
        metadata: Some(false),
        max_hits: Some(10),
        format: Some(HttpFormat::Json),
    };

    Mock::given(method("POST"))
        .and(path("/v1/bars/search"))
        .respond_with(ResponseTemplate::new(400).set_body_string("predicate must be non-empty"))
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .search(&request)
        .await
        .expect_err("expected http status error");

    match err {
        SdkError::HttpStatus { status, body } => {
            assert_eq!(status, 400);
            assert!(body.contains("predicate"));
        }
        other => panic!("expected http status error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_search_bars_invalid_json_returns_decode_error() {
    let server = MockServer::start().await;
    let request = SearchBarsRequest {
        tf: Timeframe::M1,
        close_start: 1770000000000_i64.into(),
        close_end: Some(1770003600000_i64.into()),
        cursor: None,
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5".to_string(),
        evaluate_pair: None,
        metadata: Some(false),
        max_hits: Some(10),
        format: Some(HttpFormat::Json),
    };

    Mock::given(method("POST"))
        .and(path("/v1/bars/search"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_string("{not-json"),
        )
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .search(&request)
        .await
        .expect_err("expected decode error");

    match err {
        SdkError::Decode { .. } => {}
        other => panic!("expected decode error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_search_bars_invalid_protobuf_returns_contract_drift() {
    let server = MockServer::start().await;
    let request = SearchBarsRequest {
        tf: Timeframe::M1,
        close_start: 1770000000000_i64.into(),
        close_end: Some(1770003600000_i64.into()),
        cursor: None,
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5".to_string(),
        evaluate_pair: None,
        metadata: Some(false),
        max_hits: Some(10),
        format: Some(HttpFormat::Protobuf),
    };

    Mock::given(method("POST"))
        .and(path("/v1/bars/search"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-protobuf")
                .set_body_bytes(b"not-protobuf".to_vec()),
        )
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let err = client
        .search(&request)
        .await
        .expect_err("expected contract drift error");

    match err {
        SdkError::ContractDrift { message } => {
            assert!(message.contains("protobuf decode failed"));
        }
        other => panic!("expected contract drift error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_search_bars_call_send_matches_one_page_method() {
    let server = MockServer::start().await;
    let request = SearchBarsRequest {
        tf: Timeframe::M1,
        close_start: "2026-02-02T00:00:00Z".into(),
        close_end: Some(1770003600000_i64.into()),
        cursor: None,
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5 && ETHUSDT.v > 10".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        metadata: Some(false),
        max_hits: Some(500),
        format: Some(HttpFormat::Json),
    };

    let expected_body = serde_json::json!({
        "tf": "1m",
        "close_start_ms": 1769990400000i64,
        "close_end_ms": 1770003600000i64,
        "cursor": null,
        "predicate": "BTCUSDT.c > ETHUSDT.c * 1.5 && ETHUSDT.v > 10",
        "evaluate_pair": "BTCUSDT",
        "metadata": false,
        "max_hits": 500,
        "format": "json"
    });

    let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "hits": [1770000060000i64, 1770000120000i64],
        "evaluated_rows": [{
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
        "next_cursor": "cursor-1",
        "done": false,
        "returned_hits": 2,
        "effective_hits_limit": 500,
        "truncated": false,
        "predicate_pairs": ["BTCUSDT", "ETHUSDT"],
        "predicate_normalized": "BTCUSDT.c > ETHUSDT.c * 1.5"
    }));

    Mock::given(method("POST"))
        .and(path("/v1/bars/search"))
        .and(body_json(expected_body))
        .respond_with(response)
        .expect(2)
        .mount(&server)
        .await;

    let client = Aggregator::new(config_for_http(&server.uri())).expect("client");
    let one_page = client
        .search(&request)
        .await
        .expect("one-page search success");
    let via_call = client
        .search_call(request.clone())
        .send()
        .await
        .expect("wrapper search send success");

    assert_eq!(via_call, one_page);
}

#[tokio::test]
async fn test_search_bars_call_traverse_requires_explicit_close_end() {
    let client = Aggregator::new(config_for_http("http://127.0.0.1:1")).expect("dummy client");
    let request = SearchBarsRequest {
        tf: Timeframe::M1,
        close_start: "2026-02-02T00:00:00Z".into(),
        close_end: None,
        cursor: None,
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        metadata: Some(false),
        max_hits: Some(100),
        format: Some(HttpFormat::Json),
    };

    let err = client
        .search_call(request)
        .traverse()
        .await
        .expect_err("open-ended search traverse must fail closed");

    match err {
        SdkError::UnsupportedOrUnprovedUsage { message } => {
            assert_eq!(message, "search traversal requires explicit close_end");
        }
        other => panic!("expected UnsupportedOrUnprovedUsage, got {other:?}"),
    }
}

#[tokio::test]
async fn test_search_bars_pager_requires_explicit_close_end() {
    let client = Aggregator::new(config_for_http("http://127.0.0.1:1")).expect("dummy client");
    let request = SearchBarsRequest {
        tf: Timeframe::M1,
        close_start: "2026-02-02T00:00:00Z".into(),
        close_end: None,
        cursor: None,
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        metadata: Some(false),
        max_hits: Some(100),
        format: Some(HttpFormat::Json),
    };

    let err = client
        .search_call(request)
        .pager()
        .expect_err("open-ended search pager must fail closed");

    match err {
        SdkError::UnsupportedOrUnprovedUsage { message } => {
            assert_eq!(message, "search traversal requires explicit close_end");
        }
        other => panic!("expected UnsupportedOrUnprovedUsage, got {other:?}"),
    }
}
