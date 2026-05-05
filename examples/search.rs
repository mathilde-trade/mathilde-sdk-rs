// Example: public search requests with evaluated rows
//
// What this example does:
// 1. Builds one authenticated client per public system surface.
// 2. Runs one bounded Aggregator search and prints the matching evaluated bar.
// 3. Runs one bounded Primitives search and reads projected processor fields
//    from the evaluated row.
// 4. Runs one bounded Regime search and reads projected processor fields from
//    the evaluated row.
// 5. Shows where Search is the better first step than TimeMachine: reusable
//    hit discovery, especially when evaluate_pair is omitted.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one explicit close_start/close_end window per request
// - one compact summary per search response

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{Aggregator, SearchRequest as AggregatorSearchRequest};
use mathilde_sdk_rs::systems::primitives::{
    Primitives, ProcessorFamily as PrimitivesProcessorFamily,
    ProcessorGroup as PrimitivesProcessorGroup, SearchRequest as PrimitivesSearchRequest,
};
use mathilde_sdk_rs::systems::regime::{
    ProcessorFamily as RegimeProcessorFamily, ProcessorGroup as RegimeProcessorGroup, Regime,
    SearchRequest as RegimeSearchRequest,
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

    // evaluate_pair enables evaluated_rows in the public search response.
    // Leave it unset when Search is only being used as the fast reusable
    // hit-discovery step before one or more TimeMachine calls.
    let aggregator_search = aggregator
        .search(&AggregatorSearchRequest {
            tf: Timeframe::M1,
            close_start: "2026-05-05T11:00:00Z".into(),
            close_end: Some("2026-05-05T11:10:00Z".into()),
            cursor: None,
            predicate: "BTCUSDT.c > BTCUSDT.o".to_string(),
            evaluate_pair: Some("BTCUSDT".to_string()),
            metadata: Some(false),
            max_hits: Some(5),
            format: None,
        })
        .await?;

    println!(
        "aggregator search hits={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
        aggregator_search.hits.len(),
        aggregator_search.returned_hits,
        aggregator_search.effective_hits_limit,
        aggregator_search.done,
        aggregator_search.next_cursor()
    );
    println!(
        "aggregator predicate_normalized={}",
        aggregator_search.predicate_normalized
    );

    if let Some(rows) = &aggregator_search.evaluated_rows
        && let Some(bar) = rows.first()
    {
        println!(
            "aggregator evaluated pair={} close_utc={} close={} metadata_present={}",
            bar.pair,
            bar.close_utc,
            bar.c,
            bar.metadata.is_some()
        );
    }

    let primitives_search = primitives
        .search(&PrimitivesSearchRequest {
            tf: Timeframe::M1,
            close_start: "2026-05-05T11:00:00Z".into(),
            close_end: Some("2026-05-05T11:10:00Z".into()),
            cursor: None,
            predicate: "BTCUSDT.c > BTCUSDT.o".to_string(),
            evaluate_pair: Some("BTCUSDT".to_string()),
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields in the projected evaluated row.
            family: Some(vec![PrimitivesProcessorFamily::Drawdown]),
            group: Some(vec![PrimitivesProcessorGroup::Rsi]),
            metadata: Some(false),
            diagnostics: Some(false),
            max_hits: Some(5),
            format: None,
        })
        .await?;

    println!(
        "primitives search hits={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
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

    if let Some(rows) = &primitives_search.evaluated_rows
        && let Some(row) = rows.first()
    {
        // Processor-computed fields are exposed through the computed getter helpers.
        let rsi = row.computed.f64("osc_rsi_p14");
        let drawdown = row.computed.f64("dd_drawdown");

        println!(
            "primitives evaluated pair={} close_utc={} close={} osc_rsi_p14={rsi:?} dd_drawdown={drawdown:?}",
            row.pair, row.close_utc, row.c
        );
    }

    let regime_search = regime
        .search(&RegimeSearchRequest {
            tf: Timeframe::H1,
            close_start: "2026-05-05T08:00:00Z".into(),
            close_end: Some("2026-05-05T11:00:00Z".into()),
            cursor: None,
            predicate: "ETHUSDT.tr_klts_score > 0.0".to_string(),
            evaluate_pair: Some("ETHUSDT".to_string()),
            // Family and group selectors are a union: matching either side keeps the
            // corresponding computed fields in the projected evaluated row.
            family: Some(vec![RegimeProcessorFamily::Trend]),
            group: Some(vec![RegimeProcessorGroup::InflectionQ1]),
            secondary: Some(false),
            metadata: Some(false),
            diagnostics: Some(false),
            max_hits: Some(5),
            format: None,
        })
        .await?;

    println!(
        "regime search hits={} returned_hits={} effective_hits_limit={} done={} next_cursor={:?}",
        regime_search.hits.len(),
        regime_search.returned_hits,
        regime_search.effective_hits_limit,
        regime_search.done,
        regime_search.next_cursor()
    );
    println!(
        "regime predicate_normalized={}",
        regime_search.predicate_normalized
    );

    if let Some(rows) = &regime_search.evaluated_rows
        && let Some(row) = rows.first()
    {
        let trend_score = row.computed.f64("tr_klts_score");
        let transition_pressure = row.computed.f64("in_itps_transition_pressure");

        println!(
            "regime evaluated pair={} close_utc={} close={} tr_klts_score={trend_score:?} in_itps_transition_pressure={transition_pressure:?}",
            row.pair, row.close_utc, row.c
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
