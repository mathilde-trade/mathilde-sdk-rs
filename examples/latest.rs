// Example: public latest requests across bars and computed outputs
//
// What this example does:
// 1. Builds one authenticated client per public system surface.
// 2. Reads one latest Aggregator bar for BTCUSDT on the 1m grid.
// 3. Reads one latest Primitives row and accesses processor fields through the
//    computed getter helpers.
// 4. Reads one latest Regime row and accesses processor fields through the
//    computed getter helpers.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one latest request per system
// - one compact summary per returned row

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{Aggregator, LatestRequest as AggregatorLatestRequest};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::primitives::{
    LatestRequest as PrimitivesLatestRequest, Primitives,
    ProcessorFamily as PrimitivesProcessorFamily, ProcessorGroup as PrimitivesProcessorGroup,
};
use mathilde_sdk_rs::systems::regime::{
    LatestRequest as RegimeLatestRequest, ProcessorFamily as RegimeProcessorFamily,
    ProcessorGroup as RegimeProcessorGroup, Regime,
};
use mathilde_sdk_rs::systems::types::{LatestMode, Timeframe};

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

    // Read one latest Aggregator bar with the fixed public bar shape.
    let aggregator_latest = aggregator
        .latest(&AggregatorLatestRequest {
            pairs: pairs(["BTCUSDT"]),
            tf: Timeframe::M1,
            latest_mode: LatestMode::ExactWatermark,
            metadata: Some(true),
            format: None,
        })
        .await?;

    println!(
        "aggregator latest rows={} latest_mode={:?} missing_pairs={}",
        aggregator_latest.rows.len(),
        aggregator_latest.latest_mode,
        aggregator_latest.missing_pairs.len()
    );

    if let Some(bar) = aggregator_latest.rows.first() {
        println!(
            "aggregator pair={} close_utc={} close={} age_ms={:?} metadata_present={}",
            bar.pair,
            bar.close_utc,
            bar.c,
            bar.age_ms,
            bar.metadata.is_some()
        );
    }

    // Read one latest Primitives row and access processor outputs through computed.f64(...).
    let primitives_latest = primitives
        .latest(&PrimitivesLatestRequest {
            pairs: pairs(["BTCUSDT"]),
            tf: Timeframe::M1,
            latest_mode: None,
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
        "primitives latest rows={} view={:?} missing_pairs={}",
        primitives_latest.rows.len(),
        primitives_latest.view,
        primitives_latest.missing_pairs.len()
    );

    if let Some(present) = primitives_latest.rows.first() {
        let row = &present.row;

        // Processor-computed fields are exposed through the computed getter helpers.
        let rsi = row.computed.f64("osc_rsi_p14");
        let drawdown = row.computed.f64("dd_drawdown");

        println!(
            "primitives pair={} close_utc={} close={} age_ms={} osc_rsi_p14={rsi:?} dd_drawdown={drawdown:?}",
            row.pair, row.close_utc, row.c, present.age_ms
        );
    }

    // Read one latest Regime row and access processor outputs through computed.f64(...).
    let regime_latest = regime
        .latest(&RegimeLatestRequest {
            pairs: pairs(["ETHUSDT"]),
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

    println!(
        "regime latest rows={} view={:?} missing_pairs={}",
        regime_latest.rows.len(),
        regime_latest.view,
        regime_latest.missing_pairs.len()
    );

    if let Some(present) = regime_latest.rows.first() {
        let row = &present.row;
        let trend_score = row.computed.f64("tr_klts_score");
        let transition_pressure = row.computed.f64("in_itps_transition_pressure");

        println!(
            "regime pair={} close_utc={} close={} age_ms={} tr_klts_score={trend_score:?} in_itps_transition_pressure={transition_pressure:?}",
            row.pair, row.close_utc, row.c, present.age_ms
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
