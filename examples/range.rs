// Example: public range requests and bounded traversal
//
// What this example does:
// 1. Builds one authenticated client per public system surface.
// 2. Reads one bounded Aggregator range page for BTCUSDT on the 1m grid.
// 3. Reads one bounded Primitives range page and accesses projected processor
//    fields through the computed getter helpers.
// 4. Reads one bounded Regime range page and accesses projected processor
//    fields through the computed getter helpers.
// 5. Runs one bounded Aggregator traverse() call to show the multi-page path.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one explicit close_start/close_end window per request
// - one compact summary per returned page

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{Aggregator, RangeRequest as AggregatorRangeRequest};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::primitives::{
    Primitives, ProcessorFamily as PrimitivesProcessorFamily,
    ProcessorGroup as PrimitivesProcessorGroup, RangeRequest as PrimitivesRangeRequest,
};
use mathilde_sdk_rs::systems::regime::{
    ProcessorFamily as RegimeProcessorFamily, ProcessorGroup as RegimeProcessorGroup,
    RangeRequest as RegimeRangeRequest, Regime,
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

    let range_start_m1 = Some("2026-05-05T11:00:00Z".into());
    let range_end_m1 = Some("2026-05-05T11:05:00Z".into());
    let range_start_h1 = Some("2026-05-05T08:00:00Z".into());
    let range_end_h1 = Some("2026-05-05T11:00:00Z".into());

    // Read one bounded Aggregator page with the fixed public bar shape.
    let aggregator_range = aggregator
        .range(&AggregatorRangeRequest {
            pairs: pairs(["BTCUSDT"]),
            tf: Timeframe::M1,
            align_mode: None,
            close_start: range_start_m1.clone(),
            cursor: None,
            close_end: range_end_m1.clone(),
            limit: Some(3),
            metadata: Some(true),
            format: None,
        })
        .await?;

    println!(
        "aggregator range rows={} close_end_ms={} next_cursor={:?}",
        aggregator_range.rows.len(),
        aggregator_range.close_end_ms,
        aggregator_range.next_cursor()
    );

    if let Some(bar) = aggregator_range.rows.first() {
        println!(
            "aggregator pair={} close_utc={} close={} metadata_present={}",
            bar.pair,
            bar.close_utc,
            bar.c,
            bar.metadata.is_some()
        );
    }

    // Read one bounded Primitives page and project computed fields through family/group selectors.
    let primitives_range = primitives
        .range(&PrimitivesRangeRequest {
            pairs: pairs(["BTCUSDT"]),
            tf: Timeframe::M1,
            align_mode: None,
            close_start: range_start_m1.clone(),
            cursor: None,
            close_end: range_end_m1.clone(),
            limit: Some(3),
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
        "primitives range rows={} close_end_ms={} next_cursor={:?}",
        primitives_range.rows.len(),
        primitives_range.close_end_ms,
        primitives_range.next_cursor()
    );

    if let Some(row) = primitives_range.rows.first() {
        // Processor-computed fields are exposed through the computed getter helpers.
        let rsi = row.computed.f64("osc_rsi_p14");
        let drawdown = row.computed.f64("dd_drawdown");

        println!(
            "primitives pair={} close_utc={} close={} osc_rsi_p14={rsi:?} dd_drawdown={drawdown:?}",
            row.pair, row.close_utc, row.c
        );
    }

    // Read one bounded Regime page and project computed fields through family/group selectors.
    let regime_range = regime
        .range(&RegimeRangeRequest {
            pairs: pairs(["ETHUSDT"]),
            tf: Timeframe::H1,
            align_mode: None,
            close_start: range_start_h1,
            cursor: None,
            close_end: range_end_h1,
            limit: Some(3),
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields in the projected row.
            family: Some(vec![RegimeProcessorFamily::Trend]),
            group: Some(vec![RegimeProcessorGroup::DependencyBtcQ1]),
            secondary: Some(false),
            metadata: Some(false),
            diagnostics: Some(false),
            format: None,
        })
        .await?;

    println!(
        "regime range rows={} close_end_ms={} next_cursor={:?}",
        regime_range.rows.len(),
        regime_range.close_end_ms,
        regime_range.next_cursor()
    );

    if let Some(row) = regime_range.rows.first() {
        let trend_score = row.computed.f64("tr_klts_score");
        let dependency_strength = row.computed.f64("dep_btc_dcds_dependence_strength");

        println!(
            "regime pair={} close_utc={} close={} tr_klts_score={trend_score:?} dep_btc_dcds_dependence_strength={dependency_strength:?}",
            row.pair, row.close_utc, row.c
        );
    }

    // traverse() materializes all fetched pages, so keep the window explicit and bounded.
    let aggregator_traverse = aggregator
        .range_call(AggregatorRangeRequest {
            pairs: pairs(["BTCUSDT"]),
            tf: Timeframe::M1,
            align_mode: None,
            close_start: range_start_m1,
            cursor: None,
            close_end: range_end_m1,
            limit: Some(2),
            metadata: Some(false),
            format: None,
        })
        .traverse()
        .await?;

    let traversed_rows: usize = aggregator_traverse
        .pages
        .iter()
        .map(|page| page.rows.len())
        .sum();

    println!(
        "aggregator traverse pages_fetched={} total_rows={traversed_rows}",
        aggregator_traverse.pages_fetched
    );

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
