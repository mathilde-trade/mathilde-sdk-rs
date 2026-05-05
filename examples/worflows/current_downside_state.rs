// Workflow: current_downside_state
//
// This file preserves the authoritative example_workflows entry from https://api.mathilde.dev
//
// Question: What is the current measured BTC downside state, and when did a
// materially similar measured downside state occur before?
// Why: This workflow stays on the measurement side. It retrieves the current
// outputs row, finds historical outputs rows with similar downside structure,
// and replays local context around those matched moments.
//
// Steps:
// 1. Use: Call Primitives latest.
//    Route: POST https://primitives.api.mathilde.dev/v1/outputs/latest
//    Retrieve: The newest stable computed outputs row for BTC on the target
//    timeframe.
// 2. Use: Inspect the current downside-related outputs fields.
//    Route: same response
//    Retrieve: The current measured downside structure that will define the
//    historical predicate.
// 3. Use: Call Primitives search with that predicate.
//    Route: POST https://primitives.api.mathilde.dev/v1/outputs/search
//    Retrieve: Historical timestamps where a materially similar measured
//    downside state was true.
// 4. Use: Call Primitives time-machine on the matched timestamps.
//    Route: POST https://primitives.api.mathilde.dev/v1/outputs/time-machine
//    Retrieve: Local computed-output context before and after each matched
//    historical moment.
// 5. Use: If bar-truth context is also needed, call Aggregator time-machine on
//    the same matched windows.
//    Route: POST https://aggregator.api.mathilde.dev/v1/bars/time-machine
//    Retrieve: The bounded bar context around the same matched moments.
//
// Stop when: The current measured downside state, the matched historical
// moments, and the replay context around those moments are all retrieved.
// Non-goal: Do not turn historical similarity into a prediction claim.
//
// Example note:
// - This runnable example keeps the authoritative two-step structure.
// - Search is the reusable hit-discovery step.
// - TimeMachine is the replay step fed by those explicit hits.
// - This is the cleaner default when the same matched timestamps will be reused
//   across Primitives and Aggregator.
//
// Example: current downside state plus matched historical context
//
// What this example does:
// 1. Builds one authenticated Primitives client and one authenticated
//    Aggregator client.
// 2. Reads the current BTCUSDT downside-related fields from Primitives latest.
// 3. Builds a broad “materially similar downside state” predicate from those
//    same measured fields.
// 4. Calls Primitives search without evaluated_rows to retrieve reusable hit
//    timestamps first.
// 5. Reuses those matched hits in Primitives time-machine and optional
//    Aggregator time-machine.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one current row inspection step
// - one hit-discovery step
// - one replay step reused across both systems

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{
    Aggregator, TimeMachineRequest as AggregatorTimeMachineRequest,
};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::primitives::{
    LatestRequest as PrimitivesLatestRequest, Primitives,
    ProcessorFamily as PrimitivesProcessorFamily, ProcessorGroup as PrimitivesProcessorGroup,
    SearchRequest as PrimitivesSearchRequest, TimeMachineRequest as PrimitivesTimeMachineRequest,
};
use mathilde_sdk_rs::systems::types::Timeframe;

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    // Read the example bearer once and reuse it across both clients.
    let bearer = read_example_bearer()?;

    // Build one client per public system surface used by this workflow.
    let primitives = Primitives::client(Some(bearer.clone()))?;
    let aggregator = Aggregator::client(Some(bearer))?;

    let primitives_latest = primitives
        .latest(&PrimitivesLatestRequest {
            pairs: pairs(["BTCUSDT"]),
            tf: Timeframe::H1,
            latest_mode: None,
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields in the projected row.
            family: Some(vec![PrimitivesProcessorFamily::Drawdown]),
            group: Some(vec![
                PrimitivesProcessorGroup::RollingDownsideDeviationBelow,
                PrimitivesProcessorGroup::EwmaDownsideDeviationBelow,
            ]),
            metadata: Some(false),
            diagnostics: Some(false),
            format: None,
        })
        .await?;

    let current = primitives_latest
        .rows
        .first()
        .ok_or("primitives latest returned no rows for BTCUSDT")?;
    let current_row = &current.row;

    // Processor-computed fields are exposed through the computed getter helpers.
    let current_drawdown =
        require_computed_f64("dd_drawdown", current_row.computed.f64("dd_drawdown"))?;
    let current_rolling_downside = require_computed_f64(
        "disp_rolling_downside_deviation_below_lr_p20",
        current_row
            .computed
            .f64("disp_rolling_downside_deviation_below_lr_p20"),
    )?;
    let current_ewma_downside = require_computed_f64(
        "disp_ewma_downside_deviation_below_lr_p20",
        current_row
            .computed
            .f64("disp_ewma_downside_deviation_below_lr_p20"),
    )?;

    println!(
        "current downside state close_utc={} age_ms={} dd_drawdown={} disp_rolling_downside_deviation_below_lr_p20={} disp_ewma_downside_deviation_below_lr_p20={}",
        current_row.close_utc,
        current.age_ms,
        current_drawdown,
        current_rolling_downside,
        current_ewma_downside
    );

    let predicate = build_downside_predicate(
        current_drawdown,
        current_rolling_downside,
        current_ewma_downside,
    );
    println!("downside predicate={predicate}");

    let before_bars = Some(2);
    let after_bars = Some(2);

    let primitives_search = primitives
        .search(&PrimitivesSearchRequest {
            tf: Timeframe::H1,
            close_start: "2026-04-28T00:00:00Z".into(),
            close_end: Some("2026-05-05T12:00:00Z".into()),
            cursor: None,
            predicate,
            // Leave evaluate_pair unset when search is only used to collect
            // fast reusable hit timestamps for later replay calls.
            evaluate_pair: None,
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields eligible for the predicate.
            family: Some(vec![PrimitivesProcessorFamily::Drawdown]),
            group: Some(vec![
                PrimitivesProcessorGroup::RollingDownsideDeviationBelow,
                PrimitivesProcessorGroup::EwmaDownsideDeviationBelow,
            ]),
            metadata: Some(false),
            diagnostics: Some(false),
            max_hits: Some(2),
            format: None,
        })
        .await?;

    println!(
        "primitives downside search hits={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
        primitives_search.hits.len(),
        primitives_search.returned_hits,
        primitives_search.effective_hits_limit,
        primitives_search.done,
        primitives_search.next_cursor()
    );

    let matched_hits = primitives_search.hits.clone();

    if matched_hits.is_empty() {
        println!("no downside hits found in the bounded window; skip replay steps");
        return Ok(());
    }

    let primitives_time_machine = primitives
        .time_machine(&PrimitivesTimeMachineRequest {
            tf: Timeframe::H1,
            close_start: "2026-04-28T00:00:00Z".into(),
            close_end: Some("2026-05-05T12:00:00Z".into()),
            cursor: None,
            predicate: None,
            hits: Some(matched_hits.clone()),
            output_pairs: Some(pairs(["BTCUSDT"])),
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields in the projected row.
            family: Some(vec![PrimitivesProcessorFamily::Drawdown]),
            group: Some(vec![
                PrimitivesProcessorGroup::RollingDownsideDeviationBelow,
                PrimitivesProcessorGroup::EwmaDownsideDeviationBelow,
            ]),
            metadata: Some(false),
            diagnostics: Some(false),
            before_bars,
            after_bars,
            max_hits: Some(2),
            overlap_mode: None,
            format: None,
        })
        .await?;

    println!(
        "primitives downside context rows={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
        primitives_time_machine.rows.len(),
        primitives_time_machine.returned_hits,
        primitives_time_machine.effective_hits_limit,
        primitives_time_machine.done,
        primitives_time_machine.next_cursor()
    );

    for row in primitives_time_machine.rows.iter().take(8) {
        let drawdown = row.row.computed.f64("dd_drawdown");
        let rolling = row
            .row
            .computed
            .f64("disp_rolling_downside_deviation_below_lr_p20");
        let ewma = row
            .row
            .computed
            .f64("disp_ewma_downside_deviation_below_lr_p20");

        println!(
            "primitives hit_close_ms={} offset={} close_utc={} close={} dd_drawdown={drawdown:?} rolling_downside={rolling:?} ewma_downside={ewma:?}",
            row.hit_close_ms, row.offset, row.row.close_utc, row.row.c
        );
    }

    let aggregator_time_machine = aggregator
        .time_machine(&AggregatorTimeMachineRequest {
            tf: Timeframe::H1,
            close_start: "2026-04-28T00:00:00Z".into(),
            close_end: Some("2026-05-05T12:00:00Z".into()),
            cursor: None,
            predicate: None,
            hits: Some(matched_hits),
            output_pairs: Some(pairs(["BTCUSDT"])),
            metadata: Some(false),
            before_bars,
            after_bars,
            max_hits: Some(2),
            overlap_mode: None,
            format: None,
        })
        .await?;

    println!(
        "aggregator matched bars rows={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
        aggregator_time_machine.rows.len(),
        aggregator_time_machine.returned_hits,
        aggregator_time_machine.effective_hits_limit,
        aggregator_time_machine.done,
        aggregator_time_machine.next_cursor()
    );

    for row in aggregator_time_machine.rows.iter().take(8) {
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

fn require_computed_f64(name: &str, value: Option<f64>) -> Result<f64, Box<dyn Error>> {
    value.ok_or_else(|| format!("required computed field missing: {name}").into())
}

fn build_downside_predicate(drawdown: f64, rolling_downside: f64, ewma_downside: f64) -> String {
    let drawdown_ceiling = if drawdown < 0.0 {
        drawdown * 0.5
    } else {
        -0.01
    };
    let rolling_floor = (rolling_downside * 0.5).max(0.0);
    let ewma_floor = (ewma_downside * 0.5).max(0.0);

    format!(
        "BTCUSDT.dd_drawdown <= {drawdown_ceiling:.6} && BTCUSDT.disp_rolling_downside_deviation_below_lr_p20 >= {rolling_floor:.6} && BTCUSDT.disp_ewma_downside_deviation_below_lr_p20 >= {ewma_floor:.6}"
    )
}
