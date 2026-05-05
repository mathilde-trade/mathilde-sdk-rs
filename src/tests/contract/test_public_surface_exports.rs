use crate::streaming::subscription::ExponentialBackoffConfig;
use crate::systems::aggregator::{
    Aggregator, Bar, BarMetadata, LatestRequest as AggregatorLatestRequest,
    LatestResponse as AggregatorLatestResponse, RangeRequest as AggregatorRangeRequest,
    RangeResponse as AggregatorRangeResponse, SearchRequest as AggregatorSearchRequest,
    SearchResponse as AggregatorSearchResponse, TimeMachineRequest as AggregatorTimeMachineRequest,
    TimeMachineResponse as AggregatorTimeMachineResponse,
};
use crate::systems::primitives::{
    LatestRequest as PrimitivesLatestRequest, LatestResponse as PrimitivesLatestResponse,
    OutputMetadata as PrimitivesOutputMetadata, OutputProcessDiagnostic as PrimitivesDiagnostic,
    OutputRow as PrimitivesOutputRow, PrimitiveOutputMode, Primitives,
    ProcessorFamily as PrimitiveProcessorFamily, ProcessorGroup as PrimitiveProcessorGroup,
    RangeRequest as PrimitivesRangeRequest, RangeResponse as PrimitivesRangeResponse,
    SearchRequest as PrimitivesSearchRequest, SearchResponse as PrimitivesSearchResponse,
    TimeMachineRequest as PrimitivesTimeMachineRequest,
    TimeMachineResponse as PrimitivesTimeMachineResponse,
};
use crate::systems::regime::{
    LatestRequest as RegimeLatestRequest, LatestResponse as RegimeLatestResponse,
    OutputMetadata as RegimeOutputMetadata, OutputProcessDiagnostic as RegimeDiagnostic,
    OutputRow as RegimeOutputRow, ProcessorFamily as RegimeProcessorFamily,
    ProcessorGroup as RegimeProcessorGroup, RangeRequest as RegimeRangeRequest,
    RangeResponse as RegimeRangeResponse, Regime, RegimeOutputMode,
    SearchRequest as RegimeSearchRequest, SearchResponse as RegimeSearchResponse,
    TimeMachineRequest as RegimeTimeMachineRequest,
    TimeMachineResponse as RegimeTimeMachineResponse,
};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_curated_public_surface_types_compile() {
    let _: Option<Aggregator> = None;
    let _: Option<AggregatorLatestRequest> = None;
    let _: Option<AggregatorLatestResponse> = None;
    let _: Option<AggregatorRangeRequest> = None;
    let _: Option<AggregatorRangeResponse> = None;
    let _: Option<AggregatorSearchRequest> = None;
    let _: Option<AggregatorSearchResponse> = None;
    let _: Option<AggregatorTimeMachineRequest> = None;
    let _: Option<AggregatorTimeMachineResponse> = None;
    let _: Option<Bar> = None;
    let _: Option<BarMetadata> = None;

    let _: Option<Primitives> = None;
    let _: Option<PrimitivesLatestRequest> = None;
    let _: Option<PrimitivesLatestResponse> = None;
    let _: Option<PrimitivesRangeRequest> = None;
    let _: Option<PrimitivesRangeResponse> = None;
    let _: Option<PrimitivesSearchRequest> = None;
    let _: Option<PrimitivesSearchResponse> = None;
    let _: Option<PrimitivesTimeMachineRequest> = None;
    let _: Option<PrimitivesTimeMachineResponse> = None;
    let _: Option<PrimitivesOutputRow> = None;
    let _: Option<PrimitivesOutputMetadata> = None;
    let _: Option<PrimitivesDiagnostic> = None;
    let _: PrimitiveOutputMode = PrimitiveOutputMode::Min;
    let _: PrimitiveProcessorFamily = PrimitiveProcessorFamily::MovingAverages;
    let _: PrimitiveProcessorGroup = PrimitiveProcessorGroup::Ema;

    let _: Option<Regime> = None;
    let _: Option<RegimeLatestRequest> = None;
    let _: Option<RegimeLatestResponse> = None;
    let _: Option<RegimeRangeRequest> = None;
    let _: Option<RegimeRangeResponse> = None;
    let _: Option<RegimeSearchRequest> = None;
    let _: Option<RegimeSearchResponse> = None;
    let _: Option<RegimeTimeMachineRequest> = None;
    let _: Option<RegimeTimeMachineResponse> = None;
    let _: Option<RegimeOutputRow> = None;
    let _: Option<RegimeOutputMetadata> = None;
    let _: Option<RegimeDiagnostic> = None;
    let _: RegimeOutputMode = RegimeOutputMode::Min;
    let _: RegimeProcessorFamily = RegimeProcessorFamily::Trend;
    let _: RegimeProcessorGroup = RegimeProcessorGroup::TrendQ1;

    let _ = ExponentialBackoffConfig::default();
}

#[test]
fn test_internal_modules_are_not_publicly_declared() {
    let lib_rs = fs::read_to_string(repo_root().join("src/lib.rs")).expect("read lib.rs");
    let aggregator_mod = fs::read_to_string(repo_root().join("src/systems/aggregator/mod.rs"))
        .expect("read aggregator mod");
    let primitives_mod = fs::read_to_string(repo_root().join("src/systems/primitives/mod.rs"))
        .expect("read primitives mod");
    let regime_mod =
        fs::read_to_string(repo_root().join("src/systems/regime/mod.rs")).expect("read regime mod");

    assert!(lib_rs.contains("mod transport;"));
    assert!(!lib_rs.contains("pub mod transport;"));

    for content in [&aggregator_mod, &primitives_mod, &regime_mod] {
        assert!(!content.contains("pub mod client;"));
        assert!(!content.contains("pub mod types;"));
    }

    assert!(!aggregator_mod.contains("pub mod bars_http;"));
    assert!(!aggregator_mod.contains("pub mod bars_grpc;"));
    assert!(!aggregator_mod.contains("pub mod bars_pagination;"));
    assert!(!aggregator_mod.contains("pub mod bars_ws;"));

    assert!(!primitives_mod.contains("pub mod outputs_http;"));
    assert!(!primitives_mod.contains("pub mod outputs_grpc;"));
    assert!(!primitives_mod.contains("pub mod outputs_pagination;"));
    assert!(!primitives_mod.contains("pub mod outputs_ws;"));

    assert!(!regime_mod.contains("pub mod outputs_http;"));
    assert!(!regime_mod.contains("pub mod outputs_grpc;"));
    assert!(!regime_mod.contains("pub mod outputs_pagination;"));
    assert!(!regime_mod.contains("pub mod outputs_ws;"));
}

#[test]
fn test_short_public_names_are_reexported_and_verbose_names_are_absent() {
    let aggregator_mod = fs::read_to_string(repo_root().join("src/systems/aggregator/mod.rs"))
        .expect("read aggregator mod");
    let primitives_mod = fs::read_to_string(repo_root().join("src/systems/primitives/mod.rs"))
        .expect("read primitives mod");
    let regime_mod =
        fs::read_to_string(repo_root().join("src/systems/regime/mod.rs")).expect("read regime mod");

    for short_name in [
        "LatestRequest",
        "RangeRequest",
        "SearchRequest",
        "TimeMachineRequest",
    ] {
        assert!(aggregator_mod.contains(short_name));
        assert!(primitives_mod.contains(short_name));
        assert!(regime_mod.contains(short_name));
    }

    for forbidden in [
        "LatestBarsRequest",
        "RangeBarsRequest",
        "SearchBarsRequest",
        "TimeMachineBarsRequest",
    ] {
        assert!(!aggregator_mod.contains(forbidden));
    }

    for forbidden in [
        "LatestOutputsRequest",
        "RangeOutputsRequest",
        "SearchOutputsRequest",
        "TimeMachineOutputsRequest",
    ] {
        assert!(!primitives_mod.contains(forbidden));
    }

    for forbidden in [
        "LatestOutputsRequest",
        "RangeOutputsRequest",
        "SearchOutputsRequest",
        "TimeMachineOutputsRequest",
    ] {
        assert!(!regime_mod.contains(forbidden));
    }
}
