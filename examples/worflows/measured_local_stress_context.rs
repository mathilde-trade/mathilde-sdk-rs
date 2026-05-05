// Workflow: measured_local_stress_context
//
// This file preserves the authoritative example_workflows entry from https://api.mathilde.dev
//
// Question: When did BTC show the same measured local-stress condition before,
// and what was the immediate context around those hits?
// Why: This workflow begins from a concrete measured condition rather than from
// a fixed historical window.
//
// Steps:
// 1. Use: If the relevant outputs families are not known yet, open Primitives
//    taxonomy first.
//    Route: GET https://primitives.api.mathilde.dev/v1/docs/taxonomy
//    Retrieve: The family and group space needed before building the predicate.
// 2. Use: If deeper algorithm meaning is still needed, open Primitives
//    registry.
//    Route: GET https://primitives.api.mathilde.dev/v1/docs/registry
//    Retrieve: The deeper algorithm and shipped-output discovery surface.
// 3. Use: Call Primitives search with the measured local-stress predicate.
//    Route: POST https://primitives.api.mathilde.dev/v1/outputs/search
//    Retrieve: The hit timestamps where that measured condition became true.
//    Note: Keep search as the primary reusable hit-discovery step before
//    replaying context in one or more time-machine calls.
// 4. Use: Call Primitives time-machine on those hits.
//    Route: POST https://primitives.api.mathilde.dev/v1/outputs/time-machine
//    Retrieve: The bounded computed-output context around each hit rather than
//    hit timestamps alone.
// 5. Use: If the bar path itself must also be inspected, call Aggregator
//    time-machine on the same windows.
//    Route: POST https://aggregator.api.mathilde.dev/v1/bars/time-machine
//    Retrieve: The matching bar-truth context around the same local-stress
//    hits.
//
// Stop when: The hit timestamps and their bounded context windows are both
// retrieved.
// Non-goal: Do not stop at search alone when local context is still required.
//
// Example: measured local stress hit discovery plus replay context
//
// What this example does:
// 1. Builds one authenticated Primitives client and one authenticated
//    Aggregator client.
// 2. Reads Primitives taxonomy and one filtered registry slice so the predicate
//    fields are grounded before search.
// 3. Runs Primitives search as the reusable hit-discovery step for a concrete
//    local-stress condition.
// 4. Replays the matched output context with Primitives time-machine.
// 5. Replays the same matched bar context with Aggregator time-machine.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one predicate reused across search and replay
// - one shared hits list reused across both replay surfaces

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{
    Aggregator, TimeMachineRequest as AggregatorTimeMachineRequest,
};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::primitives::{
    DocsRegistryRequest as PrimitivesDocsRegistryRequest, Primitives,
    ProcessorFamily as PrimitivesProcessorFamily, ProcessorGroup as PrimitivesProcessorGroup,
    SearchRequest as PrimitivesSearchRequest, TimeMachineRequest as PrimitivesTimeMachineRequest,
};
use mathilde_sdk_rs::systems::types::Timeframe;
use serde_json::Value;

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    const WINDOW_START: &str = "2026-02-06T09:00:00Z";
    const WINDOW_END: &str = "2026-02-06T13:00:00Z";

    // Read the example bearer once and reuse it across both clients.
    let bearer = read_example_bearer()?;

    // Build one client per public system surface used by this workflow.
    let primitives = Primitives::client(Some(bearer.clone()))?;
    let aggregator = Aggregator::client(Some(bearer))?;

    let taxonomy = primitives.docs_taxonomy().await?;

    let registry = primitives
        .docs_registry(&PrimitivesDocsRegistryRequest {
            family: Some(vec![
                PrimitivesProcessorFamily::Drawdown,
                PrimitivesProcessorFamily::Dispersion,
            ]),
            group: Some(vec![
                PrimitivesProcessorGroup::DrawdownDepthSeries,
                PrimitivesProcessorGroup::RollingDownsideDeviationBelow,
                PrimitivesProcessorGroup::EwmaDownsideDeviationBelow,
            ]),
        })
        .await?;

    print_doc_shape("primitives taxonomy", &taxonomy);
    print_doc_shape("primitives filtered registry", &registry);

    let predicate = concat!(
        "BTCUSDT.dd_drawdown_depth >= 0.030000",
        " && BTCUSDT.disp_rolling_downside_deviation_below_lr_p20 >= 0.010000",
        " && BTCUSDT.disp_ewma_downside_deviation_below_lr_p20 >= 0.010000"
    )
    .to_string();

    println!("local stress predicate={predicate}");

    let primitives_search = primitives
        .search(&PrimitivesSearchRequest {
            tf: Timeframe::H1,
            close_start: WINDOW_START.into(),
            close_end: Some(WINDOW_END.into()),
            cursor: None,
            predicate: predicate.clone(),
            // Leave evaluate_pair unset when search is only used to collect
            // fast reusable hit timestamps for later replay calls.
            evaluate_pair: None,
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields eligible for the predicate.
            family: Some(vec![
                PrimitivesProcessorFamily::Drawdown,
                PrimitivesProcessorFamily::Dispersion,
            ]),
            group: Some(vec![
                PrimitivesProcessorGroup::DrawdownDepthSeries,
                PrimitivesProcessorGroup::RollingDownsideDeviationBelow,
                PrimitivesProcessorGroup::EwmaDownsideDeviationBelow,
            ]),
            metadata: Some(false),
            diagnostics: Some(false),
            max_hits: Some(3),
            format: None,
        })
        .await?;

    println!(
        "primitives local_stress search hits={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
        primitives_search.hits.len(),
        primitives_search.returned_hits,
        primitives_search.effective_hits_limit,
        primitives_search.done,
        primitives_search.next_cursor()
    );
    println!(
        "primitives predicate_normalized={}",
        primitives_search.predicate_normalized
    );

    let before_bars = Some(2);
    let after_bars = Some(2);
    let matched_hits = primitives_search.hits.clone();

    if matched_hits.is_empty() {
        println!("no local-stress hits found in the bounded window; skip replay steps");
        return Ok(());
    }

    let primitives_time_machine = primitives
        .time_machine(&PrimitivesTimeMachineRequest {
            tf: Timeframe::H1,
            close_start: WINDOW_START.into(),
            close_end: Some(WINDOW_END.into()),
            cursor: None,
            predicate: None,
            hits: Some(matched_hits.clone()),
            output_pairs: Some(pairs(["BTCUSDT"])),
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields in the projected row.
            family: Some(vec![
                PrimitivesProcessorFamily::Drawdown,
                PrimitivesProcessorFamily::Dispersion,
            ]),
            group: Some(vec![
                PrimitivesProcessorGroup::DrawdownDepthSeries,
                PrimitivesProcessorGroup::RollingDownsideDeviationBelow,
                PrimitivesProcessorGroup::EwmaDownsideDeviationBelow,
            ]),
            metadata: Some(false),
            diagnostics: Some(false),
            before_bars,
            after_bars,
            max_hits: Some(3),
            overlap_mode: None,
            format: None,
        })
        .await?;

    println!(
        "primitives local_stress context rows={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
        primitives_time_machine.rows.len(),
        primitives_time_machine.returned_hits,
        primitives_time_machine.effective_hits_limit,
        primitives_time_machine.done,
        primitives_time_machine.next_cursor()
    );

    for row in primitives_time_machine.rows.iter().take(9) {
        let drawdown_depth = row.row.computed.f64("dd_drawdown_depth");
        let rolling = row
            .row
            .computed
            .f64("disp_rolling_downside_deviation_below_lr_p20");
        let ewma = row
            .row
            .computed
            .f64("disp_ewma_downside_deviation_below_lr_p20");

        println!(
            "primitives hit_close_ms={} offset={} close_utc={} close={} dd_drawdown_depth={drawdown_depth:?} rolling_downside={rolling:?} ewma_downside={ewma:?}",
            row.hit_close_ms, row.offset, row.row.close_utc, row.row.c
        );
    }

    let aggregator_time_machine = aggregator
        .time_machine(&AggregatorTimeMachineRequest {
            tf: Timeframe::H1,
            close_start: WINDOW_START.into(),
            close_end: Some(WINDOW_END.into()),
            cursor: None,
            predicate: None,
            hits: Some(matched_hits),
            output_pairs: Some(pairs(["BTCUSDT"])),
            metadata: Some(false),
            before_bars,
            after_bars,
            max_hits: Some(3),
            overlap_mode: None,
            format: None,
        })
        .await?;

    println!(
        "aggregator local_stress bars rows={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
        aggregator_time_machine.rows.len(),
        aggregator_time_machine.returned_hits,
        aggregator_time_machine.effective_hits_limit,
        aggregator_time_machine.done,
        aggregator_time_machine.next_cursor()
    );

    for row in aggregator_time_machine.rows.iter().take(9) {
        println!(
            "aggregator hit_close_ms={} offset={} close_utc={} close={}",
            row.hit_close_ms, row.offset, row.bar.close_utc, row.bar.c
        );
    }

    Ok(())
}

fn read_example_bearer() -> Result<BearerToken, Box<dyn Error>> {
    match env::var("EXAMPLE_BEARER_TOKEN") {
        Ok(raw) => Ok(BearerToken::new(raw)?),
        Err(env::VarError::NotPresent) => Err(
            "EXAMPLE_BEARER_TOKEN is not exported for this process; cargo run does not load .env automatically"
                .into(),
        ),
        Err(error) => Err(Box::new(error)),
    }
}

fn print_doc_shape(label: &str, value: &Value) {
    let top_level_keys = value.as_object().map_or(0, |object| object.len());
    println!("{label} top_level_keys={top_level_keys}");
}
