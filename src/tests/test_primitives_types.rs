use crate::core::auth::BearerToken;
use crate::core::time::TimeInput;
use crate::generated::primitives::outputs_proto::mathilde::feed::outputs::v1 as proto;
use crate::systems::primitives::{
    LatestGrpcRequest, LatestRequest, LatestResponse, OutputView, PrimitiveOutputMode, Primitives,
    ProcessorFamily, ProcessorGroup, RangeRequest, SearchRequest, SearchResponse,
    TimeMachineRequest, diagnostics_enabled,
};
use crate::systems::types::{HttpFormat, LatestMode, Timeframe};

#[test]
fn test_primitives_range_request_infers_projected_min_mode() {
    let request = RangeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        align_mode: None,
        close_start: None,
        cursor: None,
        close_end: None,
        limit: Some(10),
        family: None,
        group: Some(vec![ProcessorGroup::Min]),
        metadata: Some(false),
        diagnostics: None,
        format: Some(HttpFormat::Json),
    };

    assert_eq!(
        request.output_mode().expect("range mode"),
        PrimitiveOutputMode::ProjectedMin
    );
}

#[test]
fn test_primitives_search_request_infers_with_meta_mode() {
    let request = SearchRequest {
        tf: Timeframe::M1,
        close_start: TimeInput::from(1_700_000_000_000_i64),
        close_end: None,
        cursor: None,
        predicate: "c > o".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        family: None,
        group: None,
        metadata: Some(true),
        diagnostics: Some(true),
        max_hits: Some(5),
        format: Some(HttpFormat::Json),
    };

    assert_eq!(
        request.output_mode().expect("search mode"),
        PrimitiveOutputMode::WithMeta
    );
}

#[test]
fn test_primitives_time_machine_request_infers_min_mode() {
    let request = TimeMachineRequest {
        tf: Timeframe::M1,
        close_start: TimeInput::from(1_700_000_000_000_i64),
        close_end: None,
        cursor: None,
        predicate: Some("c > o".to_string()),
        hits: None,
        output_pairs: None,
        family: None,
        group: None,
        metadata: Some(false),
        diagnostics: None,
        before_bars: Some(2),
        after_bars: Some(2),
        max_hits: Some(10),
        overlap_mode: None,
        format: Some(HttpFormat::Json),
    };

    assert_eq!(
        request.output_mode().expect("time machine mode"),
        PrimitiveOutputMode::Min
    );
}

#[test]
fn test_primitives_latest_grpc_request_from_http_request_preserves_typed_selectors() {
    let request = LatestRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: Some(LatestMode::LatestAvailableLeWatermark),
        family: Some(vec![ProcessorFamily::MovingAverages]),
        group: Some(vec![ProcessorGroup::Ema]),
        metadata: Some(true),
        diagnostics: Some(true),
        format: Some(HttpFormat::Protobuf),
    };

    let grpc = LatestGrpcRequest::from(&request);

    assert_eq!(grpc.pairs, request.pairs);
    assert_eq!(grpc.tf, request.tf);
    assert_eq!(grpc.latest_mode, request.latest_mode);
    assert_eq!(grpc.family, request.family);
    assert_eq!(grpc.group, request.group);
    assert_eq!(grpc.metadata, request.metadata);
    assert_eq!(grpc.diagnostics, request.diagnostics);
}

#[test]
fn test_primitives_latest_grpc_to_proto_uses_canonical_selectors_and_empty_excludes() {
    let request = LatestGrpcRequest {
        pairs: vec![" BTCUSDT ".to_string(), "".to_string()],
        tf: Timeframe::M1,
        latest_mode: None,
        family: Some(vec![ProcessorFamily::MovingAverages]),
        group: Some(vec![ProcessorGroup::Ema]),
        metadata: Some(true),
        diagnostics: Some(true),
    };

    let proto = request.to_proto().expect("grpc latest proto");

    assert_eq!(proto.pairs, vec!["BTCUSDT".to_string()]);
    assert_eq!(proto.latest_mode, "exact_watermark");
    assert_eq!(proto.family, vec!["moving_averages".to_string()]);
    assert_eq!(proto.group, vec!["ema".to_string()]);
    assert!(proto.exclude_sources.is_empty());
}

#[test]
fn test_primitives_latest_proto_min_decode_preserves_required_fields() {
    let response = proto::OutputsLatestResponseV1 {
        watermark_end_ms: 1_700_000_000_000,
        close_end_ms: 1_700_000_000_000,
        latest_mode: "exact_watermark".to_string(),
        view: proto::OutputsViewV1::Min as i32,
        rows: vec![proto::OutputsPresentRowV1 {
            output: Some(proto::OutputRowV1 {
                pair: "BTCUSDT".to_string(),
                tf: "1m".to_string(),
                open_ms: 1_699_999_940_000,
                close_ms: 1_700_000_000_000,
                open_utc: Some("2023-11-14T22:12:20Z".to_string()),
                close_utc: Some("2023-11-14T22:13:20Z".to_string()),
                o: 1.0,
                h: 2.0,
                l: 0.5,
                c: 1.5,
                v: 3.0,
                bs_close_window_min: Some(0.75),
                diagnostics: Vec::new(),
                ..Default::default()
            }),
            age_ms: Some(123),
        }],
        missing_pairs: vec!["ETHUSDT".to_string()],
    };

    let decoded = LatestResponse::from_proto(
        response,
        PrimitiveOutputMode::Min,
        diagnostics_enabled(Some(false)),
    )
    .expect("latest proto decode");

    assert_eq!(decoded.view, OutputView::Min);
    assert_eq!(decoded.rows.len(), 1);
    assert_eq!(decoded.missing_pairs, vec!["ETHUSDT".to_string()]);
    assert_eq!(decoded.rows[0].age_ms, 123);
    assert_eq!(decoded.rows[0].row.pair, "BTCUSDT");
    assert_eq!(decoded.rows[0].row.open_utc, "2023-11-14T22:12:20Z");
    assert_eq!(decoded.rows[0].row.close_utc, "2023-11-14T22:13:20Z");
    assert!(decoded.rows[0].row.diagnostics.is_none());
    assert_eq!(
        decoded.rows[0].row.computed.f64("bs_close_window_min"),
        Some(0.75)
    );
}

#[test]
fn test_primitives_latest_proto_full_decode_defaults_tail_bar_provenance() {
    let response = proto::OutputsLatestResponseV1 {
        watermark_end_ms: 1_700_000_000_000,
        close_end_ms: 1_700_000_000_000,
        latest_mode: "exact_watermark".to_string(),
        view: proto::OutputsViewV1::Full as i32,
        rows: vec![proto::OutputsPresentRowV1 {
            output: Some(proto::OutputRowV1 {
                pair: "BTCUSDT".to_string(),
                tf: "1m".to_string(),
                open_ms: 1_699_999_940_000,
                close_ms: 1_700_000_000_000,
                open_utc: Some("2023-11-14T22:12:20Z".to_string()),
                close_utc: Some("2023-11-14T22:13:20Z".to_string()),
                o: 1.0,
                h: 2.0,
                l: 0.5,
                c: 1.5,
                v: 3.0,
                diagnostics: vec![proto::OutputProcessDiagnosticV1 {
                    indicator: "test".to_string(),
                    message: "ok".to_string(),
                }],
                metadata: Some(proto::OutputMetadataV1 {
                    source: "feed".to_string(),
                    process: "batch".to_string(),
                    computed_at_ms: 1_700_000_000_100,
                    computed_at_utc: "2023-11-14T22:13:20.100Z".to_string(),
                    tail_bar_provenance: None,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            age_ms: Some(123),
        }],
        missing_pairs: Vec::new(),
    };

    let decoded = LatestResponse::from_proto(
        response,
        PrimitiveOutputMode::WithMeta,
        diagnostics_enabled(Some(true)),
    )
    .expect("latest full proto decode");

    let row = &decoded.rows[0].row;
    assert_eq!(row.metadata.as_ref().expect("metadata").source, "feed");
    assert_eq!(
        row.metadata
            .as_ref()
            .expect("metadata")
            .tail_bar_provenance
            .source,
        None
    );
    assert_eq!(row.diagnostics.as_ref().map(Vec::len), Some(1));
}

#[test]
fn test_primitives_search_proto_rejects_evaluated_rows_without_evaluate_pair() {
    let response = proto::OutputsSearchResponseV1 {
        hits: vec![1],
        evaluated_rows: vec![proto::OutputRowV1 {
            pair: "BTCUSDT".to_string(),
            tf: "1m".to_string(),
            open_ms: 1,
            close_ms: 2,
            open_utc: Some("2023-11-14T22:12:20Z".to_string()),
            close_utc: Some("2023-11-14T22:13:20Z".to_string()),
            o: 1.0,
            h: 1.0,
            l: 1.0,
            c: 1.0,
            v: 1.0,
            ..Default::default()
        }],
        returned_hits: 1,
        effective_hits_limit: 1,
        truncated: false,
        predicate_pairs: vec!["BTCUSDT".to_string()],
        predicate_normalized: "BTCUSDT.c > 1".to_string(),
        next_cursor: None,
        done: true,
    };

    let error = SearchResponse::from_proto(
        response,
        PrimitiveOutputMode::Min,
        diagnostics_enabled(Some(false)),
        false,
    )
    .expect_err("unexpected evaluated rows must fail");

    assert!(error.to_string().contains("evaluate_pair"));
}

#[tokio::test]
async fn test_primitives_client_mathilde_public_default_builds_transports() {
    let token = BearerToken::new("feed_public_token").expect("valid token");
    let client = Primitives::client(Some(token)).expect("client builds");

    assert_eq!(
        client
            .http
            .endpoint_url("/v1/outputs/latest")
            .expect("http endpoint")
            .as_str(),
        "https://primitives.api.mathilde.dev/v1/outputs/latest"
    );
    assert_eq!(
        client.grpc.as_ref().expect("grpc").endpoint(),
        "https://primitives.grpc.mathilde.dev"
    );
    assert_eq!(
        client
            .ws
            .as_ref()
            .expect("ws")
            .endpoint_url("/v1/ws/outputs")
            .expect("ws endpoint")
            .as_str(),
        "wss://primitives.api.mathilde.dev/v1/ws/outputs"
    );
}
