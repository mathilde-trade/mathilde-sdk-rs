// Workflow: bounded_recent_window
//
// This file preserves the authoritative example_workflows entry from https://api.mathilde.dev
//
// Question: How did BTC bar truth and computed outputs evolve over the last
// 24 hours?
// Why: This workflow reconstructs a bounded recent window instead of only
// reading the newest row.
//
// Steps:
// 1. Use: Call Aggregator range for BTC on the target timeframe over the last
//    24 hours.
//    Route: POST https://aggregator.api.mathilde.dev/v1/bars/range
//    Retrieve: The bounded bar-truth window.
// 2. Use: Call Primitives range over the same pair, timeframe, and time window.
//    Route: POST https://primitives.api.mathilde.dev/v1/outputs/range
//    Retrieve: The bounded computed-outputs window aligned to the same period.
// 3. Use: Align the bars and outputs rows by timestamp.
//    Route: local alignment step
//    Retrieve: One recent sequence where market truth and computed measurement
//    are bound together.
// 4. Use: Read the ordered sequence from oldest to newest.
//    Route: same aligned window
//    Retrieve: How the current measured state formed across the last 24 hours.
//
// Stop when: The last-24h window is fully reconstructed as an aligned
// bars-plus-outputs sequence.
// Non-goal: Do not use this workflow when the real question is
// unknown-timestamp discovery.
//
// Example: reconstruct one recent bounded window from bars plus computed outputs
//
// What this example does:
// 1. Builds one authenticated Aggregator client and one authenticated
//    Primitives client.
// 2. Fetches one 24-hour BTCUSDT bars window on the 1h grid.
// 3. Fetches one aligned 24-hour BTCUSDT Primitives window on the same 1h grid.
// 4. Aligns both windows locally by close timestamp and reads the sequence from
//    oldest to newest.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one explicit 24-hour window reused across both requests
// - one compact aligned summary for the first and last few rows

use std::collections::BTreeMap;
use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{Aggregator, RangeRequest as AggregatorRangeRequest};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::primitives::{
    Primitives, ProcessorFamily as PrimitivesProcessorFamily,
    ProcessorGroup as PrimitivesProcessorGroup, RangeRequest as PrimitivesRangeRequest,
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
    let aggregator = Aggregator::client(Some(bearer.clone()))?;
    let primitives = Primitives::client(Some(bearer))?;

    let close_start = Some("2026-05-04T12:00:00Z".into());
    let close_end = Some("2026-05-05T12:00:00Z".into());

    let aggregator_range = aggregator
        .range(&AggregatorRangeRequest {
            pairs: pairs(["BTCUSDT"]),
            tf: Timeframe::H1,
            align_mode: None,
            close_start: close_start.clone(),
            cursor: None,
            close_end: close_end.clone(),
            limit: Some(48),
            metadata: Some(false),
            format: None,
        })
        .await?;

    let primitives_range = primitives
        .range(&PrimitivesRangeRequest {
            pairs: pairs(["BTCUSDT"]),
            tf: Timeframe::H1,
            align_mode: None,
            close_start,
            cursor: None,
            close_end,
            limit: Some(48),
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields in the projected row.
            family: Some(vec![PrimitivesProcessorFamily::Drawdown]),
            group: Some(vec![PrimitivesProcessorGroup::Rsi]),
            metadata: Some(false),
            diagnostics: Some(false),
            format: None,
        })
        .await?;

    println!(
        "aggregator bounded_window rows={} close_end_ms={} next_cursor={:?}",
        aggregator_range.rows.len(),
        aggregator_range.close_end_ms,
        aggregator_range.next_cursor()
    );
    println!(
        "primitives bounded_window rows={} close_end_ms={} next_cursor={:?}",
        primitives_range.rows.len(),
        primitives_range.close_end_ms,
        primitives_range.next_cursor()
    );

    // Align the two windows locally by close timestamp so one ordered sequence
    // carries both bar truth and computed measurement.
    let primitives_by_close_ms: BTreeMap<i64, _> = primitives_range
        .rows
        .iter()
        .map(|row| (row.close_ms, row))
        .collect();

    let aligned_rows: Vec<_> = aggregator_range
        .rows
        .iter()
        .filter_map(|bar| {
            primitives_by_close_ms
                .get(&bar.close_ms)
                .map(|row| (bar, *row))
        })
        .collect();

    println!("aligned sequence rows={}", aligned_rows.len());

    for (index, (bar, row)) in aligned_rows.iter().enumerate() {
        if index < 3 || index + 3 >= aligned_rows.len() {
            // Processor-computed fields are exposed through the computed getter helpers.
            let rsi = row.computed.f64("osc_rsi_p14");
            let drawdown = row.computed.f64("dd_drawdown");

            println!(
                "aligned close_utc={} bar_close={} osc_rsi_p14={rsi:?} dd_drawdown={drawdown:?}",
                bar.close_utc, bar.c
            );
        }
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
