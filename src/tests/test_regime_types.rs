use crate::core::time::TimeInput;
use crate::generated::regime::outputs_proto::mathilde::feed::outputs::v1 as proto;
use crate::systems::regime::{
    LatestGrpcRequest, LatestRequest, LatestResponse, OutputView, ProcessorFamily, ProcessorGroup,
    RangeRequest, RegimeOutputMode, SearchRequest, TimeMachineRequest, diagnostics_enabled,
};
use crate::systems::types::{HttpFormat, LatestMode, Timeframe};

#[test]
fn test_regime_processor_family_surface_excludes_metadata() {
    assert!(ProcessorFamily::parse("metadata").is_none());
    assert!(
        ProcessorFamily::ALL
            .iter()
            .all(|family| family.canonical_name() != "metadata")
    );
}

#[test]
fn test_regime_range_outputs_request_infers_projected_min_mode_when_secondary_is_false() {
    let request = RangeRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::H1,
        align_mode: None,
        close_start: None,
        cursor: None,
        close_end: None,
        limit: Some(10),
        family: None,
        group: None,
        secondary: Some(false),
        metadata: Some(false),
        diagnostics: None,
        format: Some(HttpFormat::Json),
    };

    assert_eq!(
        request.output_mode().expect("range mode"),
        RegimeOutputMode::ProjectedMin
    );
}

#[test]
fn test_regime_search_outputs_request_infers_with_meta_mode_when_secondary_is_true() {
    let request = SearchRequest {
        tf: Timeframe::H1,
        close_start: TimeInput::from(1_700_000_000_000_i64),
        close_end: None,
        cursor: None,
        predicate: "c > o".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        family: None,
        group: None,
        secondary: Some(true),
        metadata: Some(true),
        diagnostics: Some(true),
        max_hits: Some(5),
        format: Some(HttpFormat::Json),
    };

    assert_eq!(
        request.output_mode().expect("search mode"),
        RegimeOutputMode::WithMeta
    );
}

#[test]
fn test_regime_time_machine_outputs_request_rejects_non_h1() {
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
        secondary: Some(true),
        metadata: Some(false),
        diagnostics: None,
        before_bars: Some(2),
        after_bars: Some(2),
        max_hits: Some(10),
        overlap_mode: None,
        format: Some(HttpFormat::Json),
    };

    let error = request
        .validate()
        .expect_err("non-h1 time machine must fail");
    assert!(error.to_string().contains("tf=1h"));
}

#[test]
fn test_regime_latest_outputs_grpc_request_from_http_request_preserves_secondary_and_selectors() {
    let request = LatestRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::H1,
        latest_mode: Some(LatestMode::LatestAvailableLeWatermark),
        family: Some(vec![ProcessorFamily::Trend]),
        group: Some(vec![ProcessorGroup::TrendQ1]),
        secondary: Some(false),
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
    assert_eq!(grpc.secondary, request.secondary);
}

#[test]
fn test_regime_latest_outputs_grpc_to_proto_uses_canonical_selectors_secondary_and_empty_excludes()
{
    let request = LatestGrpcRequest {
        pairs: vec![" BTCUSDT ".to_string(), "".to_string()],
        tf: Timeframe::H1,
        latest_mode: None,
        family: Some(vec![ProcessorFamily::Trend]),
        group: Some(vec![ProcessorGroup::TrendQ1]),
        secondary: Some(true),
        metadata: Some(true),
        diagnostics: Some(true),
    };

    let proto = request.to_proto().expect("grpc latest proto");

    assert_eq!(proto.pairs, vec!["BTCUSDT".to_string()]);
    assert_eq!(proto.tf, "1h");
    assert_eq!(proto.latest_mode, "exact_watermark");
    assert_eq!(proto.family, vec!["trend".to_string()]);
    assert_eq!(proto.group, vec!["trend.q1".to_string()]);
    assert!(proto.secondary);
    assert!(proto.exclude_sources.is_empty());
}

#[test]
fn test_regime_latest_http_normalize_uses_canonical_group_selector_names() {
    let request = LatestRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::H1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: Some(vec![ProcessorFamily::Trend]),
        group: Some(vec![ProcessorGroup::TrendQ1]),
        secondary: Some(false),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Json),
    };

    let normalized = request.normalize_http().expect("normalize latest http");
    let body = serde_json::to_value(normalized).expect("serialize normalized latest");

    assert_eq!(body["family"], serde_json::json!(["trend"]));
    assert_eq!(body["group"], serde_json::json!(["trend.q1"]));
}

#[test]
fn test_regime_latest_outputs_proto_min_decode_preserves_required_fields() {
    let response = proto::OutputsLatestResponseV1 {
        watermark_end_ms: 1_700_000_000_000,
        close_end_ms: 1_700_000_000_000,
        latest_mode: "exact_watermark".to_string(),
        view: proto::OutputsViewV1::Min as i32,
        rows: vec![proto::OutputsPresentRowV1 {
            output: Some(proto::OutputRowV1 {
                pair: "BTCUSDT".to_string(),
                tf: "1h".to_string(),
                open_ms: 1_699_999_940_000,
                close_ms: 1_700_000_000_000,
                open_utc: Some("2023-11-14T22:12:20Z".to_string()),
                close_utc: Some("2023-11-14T22:13:20Z".to_string()),
                o: 1.0,
                h: 2.0,
                l: 0.5,
                c: 1.5,
                v: 3.0,
                tr_klts_score: Some(0.75),
                diagnostics: Vec::new(),
                ..Default::default()
            }),
            age_ms: Some(123),
        }],
        missing_pairs: vec!["ETHUSDT".to_string()],
    };

    let decoded = LatestResponse::from_proto(
        response,
        RegimeOutputMode::Min,
        diagnostics_enabled(Some(false)),
    )
    .expect("latest proto decode");

    assert_eq!(decoded.view, OutputView::Min);
    assert_eq!(decoded.rows.len(), 1);
    assert_eq!(decoded.missing_pairs, vec!["ETHUSDT".to_string()]);
    assert_eq!(decoded.rows[0].age_ms, 123);
    assert_eq!(decoded.rows[0].row.pair, "BTCUSDT");
    assert_eq!(decoded.rows[0].row.tf, "1h");
    assert_eq!(decoded.rows[0].row.open_utc, "2023-11-14T22:12:20Z");
    assert_eq!(decoded.rows[0].row.close_utc, "2023-11-14T22:13:20Z");
    assert_eq!(decoded.rows[0].row.diagnostics, None);
    assert_eq!(
        decoded.rows[0].row.computed.f64("tr_klts_score"),
        Some(0.75)
    );
}
