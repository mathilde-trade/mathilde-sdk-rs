// Workflow: current_grouped_regime_state
//
// This file preserves the authoritative example_workflows entry from https://api.mathilde.dev
//
// Question: What is the current grouped BTC regime state, and where did a
// materially similar grouped regime state occur before?
// Why: This workflow stays on decomposed market-state measurement rather than
// on raw bar truth or generic primitive outputs.
//
// Steps:
// 1. Use: If the relevant regime families or groups are not known yet, open
//    Regime taxonomy first.
//    Route: GET https://regime.api.mathilde.dev/v1/docs/taxonomy
//    Retrieve: The family-and-group space needed before building the grouped
//    regime predicate.
// 2. Use: If deeper algorithm meaning is still needed, open Regime registry.
//    Route: GET https://regime.api.mathilde.dev/v1/docs/registry
//    Retrieve: The deeper regime algorithm and shipped-output discovery
//    surface.
// 3. Use: Call Regime latest for BTC on the defended 1h lane.
//    Route: POST https://regime.api.mathilde.dev/v1/outputs/latest
//    Retrieve: The newest stable grouped regime outputs row for BTC on the
//    supported timeframe.
// 4. Use: Call Regime time-machine with a coarse grouped-state predicate built
//    from that current row.
//    Route: POST https://regime.api.mathilde.dev/v1/outputs/time-machine
//    Retrieve: The matched historical moments and their bounded local grouped
//    regime context in one pass.
//
// Stop when: The current grouped regime state, the matched historical moments,
// and the replay context around those moments are all retrieved.
// Non-goal: Do not turn grouped historical similarity into a prediction claim.
//
// Example note:
// - This runnable example intentionally skips Search.
// - TimeMachine is enough here because the matched timestamps are not reused
//   across another replay surface.
// - The grouped-state predicate uses coarse measurement bands rather than exact
//   equality so it can retrieve materially similar historical states.
//
// Example: current grouped BTC regime state plus matched historical context
//
// What this example does:
// 1. Builds one authenticated Regime client.
// 2. Reads Regime taxonomy and one filtered registry slice so the grouped
//    fields are grounded before retrieval.
// 3. Reads the current BTC grouped regime row from Regime latest.
// 4. Builds one coarse grouped-state predicate from those same measured fields.
// 5. Replays the matched historical grouped context in one direct Regime
//    time-machine call.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client for the one system used
// - one current row inspection step
// - one one-pass replay step

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::regime::{
    DocsRegistryRequest as RegimeDocsRegistryRequest, LatestRequest as RegimeLatestRequest,
    ProcessorFamily as RegimeProcessorFamily, ProcessorGroup as RegimeProcessorGroup, Regime,
    TimeMachineRequest as RegimeTimeMachineRequest,
};
use mathilde_sdk_rs::systems::types::Timeframe;
use serde_json::Value;

const HISTORY_WINDOW_START: &str = "2026-04-01T00:00:00Z";

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    // Read the example bearer once and reuse it across the workflow.
    let bearer = read_example_bearer()?;

    // Build one client for the single public system surface used here.
    let regime = Regime::client(Some(bearer))?;

    let taxonomy = regime.docs_taxonomy().await?;

    let registry = regime
        .docs_registry(&RegimeDocsRegistryRequest {
            family: Some(vec![RegimeProcessorFamily::Trend]),
            group: Some(vec![RegimeProcessorGroup::InflectionQ1]),
        })
        .await?;

    print_doc_shape("regime taxonomy", &taxonomy);
    print_doc_shape("regime filtered registry", &registry);

    let regime_latest = regime
        .latest(&RegimeLatestRequest {
            pairs: pairs(["BTCUSDT"]),
            tf: Timeframe::H1,
            latest_mode: None,
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields in the projected row.
            family: Some(vec![RegimeProcessorFamily::Trend]),
            group: Some(vec![RegimeProcessorGroup::InflectionQ1]),
            secondary: Some(false),
            metadata: Some(false),
            diagnostics: Some(false),
            format: None,
        })
        .await?;

    let current = regime_latest
        .rows
        .first()
        .ok_or("regime latest returned no rows for BTCUSDT")?;
    let current_row = &current.row;

    // Processor-computed fields are exposed through the computed getter helpers.
    let current_trend_score =
        require_computed_f64("tr_klts_score", current_row.computed.f64("tr_klts_score"))?;
    let current_trend_stress =
        require_computed_f64("tr_trss_stress", current_row.computed.f64("tr_trss_stress"))?;
    let current_transition_pressure = require_computed_f64(
        "in_itps_transition_pressure",
        current_row.computed.f64("in_itps_transition_pressure"),
    )?;

    println!(
        "current grouped regime state close_utc={} age_ms={} tr_klts_score={} tr_trss_stress={} in_itps_transition_pressure={}",
        current_row.close_utc,
        current.age_ms,
        current_trend_score,
        current_trend_stress,
        current_transition_pressure
    );

    let predicate = build_grouped_regime_predicate(
        current_trend_score,
        current_trend_stress,
        current_transition_pressure,
    );
    println!("grouped regime predicate={predicate}");

    // before_bars/after_bars define the bounded context around each grouped-state hit.
    let before_bars = Some(2);
    let after_bars = Some(2);

    let regime_time_machine = regime
        .time_machine(&RegimeTimeMachineRequest {
            tf: Timeframe::H1,
            close_start: HISTORY_WINDOW_START.into(),
            close_end: Some(current_row.close_utc.clone().into()),
            cursor: None,
            predicate: Some(predicate),
            hits: None,
            output_pairs: Some(pairs(["BTCUSDT"])),
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields in the projected row.
            family: Some(vec![RegimeProcessorFamily::Trend]),
            group: Some(vec![RegimeProcessorGroup::InflectionQ1]),
            secondary: Some(false),
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
        "regime grouped context rows={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
        regime_time_machine.rows.len(),
        regime_time_machine.returned_hits,
        regime_time_machine.effective_hits_limit,
        regime_time_machine.done,
        regime_time_machine.next_cursor()
    );
    println!(
        "regime predicate_normalized={:?}",
        regime_time_machine.predicate_normalized
    );

    if regime_time_machine.rows.is_empty() {
        println!("no grouped regime matches found in the bounded window");
        return Ok(());
    }

    for row in regime_time_machine.rows.iter().take(8) {
        let trend_score = row.row.computed.f64("tr_klts_score");
        let trend_stress = row.row.computed.f64("tr_trss_stress");
        let transition_pressure = row.row.computed.f64("in_itps_transition_pressure");

        println!(
            "regime hit_close_ms={} offset={} close_utc={} close={} tr_klts_score={trend_score:?} tr_trss_stress={trend_stress:?} in_itps_transition_pressure={transition_pressure:?}",
            row.hit_close_ms, row.offset, row.row.close_utc, row.row.c
        );
    }

    Ok(())
}

fn build_grouped_regime_predicate(
    trend_score: f64,
    trend_stress: f64,
    transition_pressure: f64,
) -> String {
    let trend_clause = if trend_score >= 0.25 {
        "BTCUSDT.tr_klts_score >= 0.25 && BTCUSDT.tr_klts_score <= 0.75"
    } else if trend_score <= -0.25 {
        "BTCUSDT.tr_klts_score >= -0.75 && BTCUSDT.tr_klts_score <= -0.25"
    } else {
        "BTCUSDT.tr_klts_score >= -0.25 && BTCUSDT.tr_klts_score <= 0.25"
    };

    let stress_clause = if trend_stress <= 0.05 {
        "BTCUSDT.tr_trss_stress <= 0.05"
    } else if trend_stress <= 0.10 {
        "BTCUSDT.tr_trss_stress >= 0.05 && BTCUSDT.tr_trss_stress <= 0.10"
    } else {
        "BTCUSDT.tr_trss_stress >= 0.10"
    };

    let transition_clause = if transition_pressure.abs() <= 0.001 {
        "BTCUSDT.in_itps_transition_pressure >= -0.001 && BTCUSDT.in_itps_transition_pressure <= 0.001"
    } else if transition_pressure > 0.0 {
        "BTCUSDT.in_itps_transition_pressure > 0.0"
    } else {
        "BTCUSDT.in_itps_transition_pressure < 0.0"
    };

    format!("{trend_clause} && {stress_clause} && {transition_clause}")
}

fn print_doc_shape(label: &str, value: &Value) {
    let top_level_keys = value.as_object().map_or(0, |object| object.len());
    println!("{label} top_level_keys={top_level_keys}");
}

fn require_computed_f64(field: &str, value: Option<f64>) -> Result<f64, Box<dyn Error>> {
    value.ok_or_else(|| format!("missing computed field `{field}` in regime latest row").into())
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
