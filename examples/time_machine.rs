// Example: public time-machine requests around predicate hits
//
// What this example does:
// 1. Builds one authenticated client per public system surface.
// 2. Runs one bounded Aggregator time-machine request around bar-hit matches.
// 3. Runs one bounded Primitives time-machine request and reads projected
//    processor fields from the returned offset rows.
// 4. Runs one bounded Regime time-machine request and reads projected
//    processor fields from the returned offset rows.
// 5. Shows the direct one-pass replay path; when the same matched timestamps
//    must be reused across multiple replay calls, Search is the cleaner first
//    step.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one explicit close_start/close_end window per request
// - one compact summary per response plus a few offset rows

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{
    Aggregator, TimeMachineRequest as AggregatorTimeMachineRequest,
};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::primitives::{
    Primitives, ProcessorFamily as PrimitivesProcessorFamily,
    ProcessorGroup as PrimitivesProcessorGroup, TimeMachineRequest as PrimitivesTimeMachineRequest,
};
use mathilde_sdk_rs::systems::regime::{
    ProcessorFamily as RegimeProcessorFamily, ProcessorGroup as RegimeProcessorGroup, Regime,
    TimeMachineRequest as RegimeTimeMachineRequest,
};
use mathilde_sdk_rs::systems::types::Timeframe;

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    // Read the example bearer once and reuse it across all clients.
    let bearer = read_example_bearer()?;

    // Build one client per public system surface.
    let aggregator = Aggregator::client(Some(bearer.clone()))?;
    let primitives = Primitives::client(Some(bearer.clone()))?;
    let regime = Regime::client(Some(bearer))?;

    // before_bars/after_bars define the context window around each hit:
    // offset -2 means two closed bars before the hit, offset 0 is the hit bar,
    // and offset +2 means two closed bars after the hit.
    let before_bars = Some(2);
    let after_bars = Some(2);

    let aggregator_time_machine = aggregator
        .time_machine(&AggregatorTimeMachineRequest {
            tf: Timeframe::M1,
            close_start: "2026-05-05T11:00:00Z".into(),
            close_end: Some("2026-05-05T11:10:00Z".into()),
            cursor: None,
            predicate: Some("BTCUSDT.c > BTCUSDT.o".to_string()),
            hits: None,
            output_pairs: Some(pairs(["BTCUSDT"])),
            metadata: Some(true),
            before_bars,
            after_bars,
            max_hits: Some(2),
            overlap_mode: None,
            format: None,
        })
        .await?;

    println!(
        "aggregator time_machine rows={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
        aggregator_time_machine.rows.len(),
        aggregator_time_machine.returned_hits,
        aggregator_time_machine.effective_hits_limit,
        aggregator_time_machine.done,
        aggregator_time_machine.next_cursor()
    );
    println!(
        "aggregator predicate_normalized={:?}",
        aggregator_time_machine.predicate_normalized
    );

    for row in aggregator_time_machine.rows.iter().take(5) {
        println!(
            "aggregator hit_close_ms={} offset={} pair={} close_utc={} close={} metadata_present={}",
            row.hit_close_ms,
            row.offset,
            row.bar.pair,
            row.bar.close_utc,
            row.bar.c,
            row.bar.metadata.is_some()
        );
    }

    let primitives_time_machine = primitives
        .time_machine(&PrimitivesTimeMachineRequest {
            tf: Timeframe::M1,
            close_start: "2026-05-05T11:00:00Z".into(),
            close_end: Some("2026-05-05T11:10:00Z".into()),
            cursor: None,
            predicate: Some("BTCUSDT.c > BTCUSDT.o".to_string()),
            hits: None,
            output_pairs: Some(pairs(["BTCUSDT"])),
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields in the projected row.
            family: Some(vec![PrimitivesProcessorFamily::Drawdown]),
            group: Some(vec![PrimitivesProcessorGroup::Rsi]),
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
        "primitives time_machine rows={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
        primitives_time_machine.rows.len(),
        primitives_time_machine.returned_hits,
        primitives_time_machine.effective_hits_limit,
        primitives_time_machine.done,
        primitives_time_machine.next_cursor()
    );
    println!(
        "primitives predicate_normalized={:?}",
        primitives_time_machine.predicate_normalized
    );

    for row in primitives_time_machine.rows.iter().take(5) {
        // Processor-computed fields are exposed through the computed getter helpers.
        let rsi = row.row.computed.f64("osc_rsi_p14");
        let drawdown = row.row.computed.f64("dd_drawdown");

        println!(
            "primitives hit_close_ms={} offset={} pair={} close_utc={} close={} osc_rsi_p14={rsi:?} dd_drawdown={drawdown:?}",
            row.hit_close_ms, row.offset, row.row.pair, row.row.close_utc, row.row.c
        );
    }

    let regime_time_machine = regime
        .time_machine(&RegimeTimeMachineRequest {
            tf: Timeframe::H1,
            close_start: "2026-05-05T08:00:00Z".into(),
            close_end: Some("2026-05-05T11:00:00Z".into()),
            cursor: None,
            predicate: Some("ETHUSDT.tr_klts_score > 0.0".to_string()),
            hits: None,
            output_pairs: Some(pairs(["ETHUSDT"])),
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
        "regime time_machine rows={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
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

    for row in regime_time_machine.rows.iter().take(5) {
        let trend_score = row.row.computed.f64("tr_klts_score");
        let transition_pressure = row.row.computed.f64("in_itps_transition_pressure");

        println!(
            "regime hit_close_ms={} offset={} pair={} close_utc={} close={} tr_klts_score={trend_score:?} in_itps_transition_pressure={transition_pressure:?}",
            row.hit_close_ms, row.offset, row.row.pair, row.row.close_utc, row.row.c
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
