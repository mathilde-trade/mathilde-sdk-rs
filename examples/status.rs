// Example: public pair status surfaces
//
// What this example does:
// 1. Reads a small enabled pair list from Aggregator, Primitives, and Regime.
// 2. Reads a small status slice for BTCUSDT and ETHUSDT from each system.
// 3. Prints one compact summary per system so the caller can compare the public
//    pair-state surfaces.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one small summary per list call and status call

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{
    Aggregator, PairsListRequest as AggregatorPairsListRequest,
    PairsStatusRequest as AggregatorPairsStatusRequest,
};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::primitives::{
    PairsListRequest as PrimitivesPairsListRequest,
    PairsStatusRequest as PrimitivesPairsStatusRequest, Primitives,
};
use mathilde_sdk_rs::systems::regime::{
    PairsListRequest as RegimePairsListRequest, PairsStatusRequest as RegimePairsStatusRequest,
    Regime,
};

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

    // Read one compact pair catalogue per system.
    let aggregator_list = aggregator
        .pairs_list(&AggregatorPairsListRequest {
            after_pair: None,
            limit: Some(5),
            enabled_only: Some(true),
        })
        .await?;

    let primitives_list = primitives
        .pairs_list(&PrimitivesPairsListRequest {
            after_pair: None,
            limit: Some(5),
            enabled_only: Some(true),
        })
        .await?;

    let regime_list = regime
        .pairs_list(&RegimePairsListRequest {
            after_pair: None,
            limit: Some(5),
            enabled_only: Some(true),
        })
        .await?;

    println!("aggregator listed_pairs={}", aggregator_list.pairs.len());
    println!("primitives listed_pairs={}", primitives_list.pairs.len());
    println!("regime listed_pairs={}", regime_list.pairs.len());

    // Read one small status slice for the same public pairs across systems.
    let target_pairs = Some(pairs(["BTCUSDT", "ETHUSDT"]));
    let target_filters = Some(vec![
        "status".to_string(),
        "history".to_string(),
        "readiness".to_string(),
    ]);

    let aggregator_status = aggregator
        .pairs_status(&AggregatorPairsStatusRequest {
            after_pair: None,
            limit: Some(2),
            pairs: target_pairs.clone(),
            filters: target_filters.clone(),
        })
        .await?;

    let primitives_status = primitives
        .pairs_status(&PrimitivesPairsStatusRequest {
            after_pair: None,
            limit: Some(2),
            pairs: target_pairs.clone(),
            filters: target_filters.clone(),
        })
        .await?;

    let regime_status = regime
        .pairs_status(&RegimePairsStatusRequest {
            after_pair: None,
            limit: Some(2),
            pairs: target_pairs,
            filters: target_filters,
        })
        .await?;

    print_aggregator_status(&aggregator_status);
    print_primitives_status(&primitives_status);
    print_regime_status(&regime_status);

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

fn print_aggregator_status(response: &mathilde_sdk_rs::systems::aggregator::PairsStatusResponse) {
    println!("aggregator status_rows={}", response.pairs.len());
    for row in &response.pairs {
        let enabled = row.status.as_ref().map(|block| block.enabled);
        let run_state = row.status.as_ref().map(|block| block.run_state.as_str());
        let seed_done = row.history.as_ref().and_then(|block| block.seed_done);
        let ready_1m = row.readiness.as_ref().map(|block| block.m1.ready);
        println!(
            "aggregator pair={} enabled={enabled:?} run_state={run_state:?} seed_done={seed_done:?} ready_1m={ready_1m:?}",
            row.pair
        );
    }
}

fn print_primitives_status(response: &mathilde_sdk_rs::systems::primitives::PairsStatusResponse) {
    println!("primitives status_rows={}", response.pairs.len());
    for row in &response.pairs {
        let enabled = row.status.as_ref().map(|block| block.enabled);
        let run_state = row.status.as_ref().map(|block| block.run_state.as_str());
        let seed_done = row.history.as_ref().and_then(|block| block.seed_done);
        let ready_1m = row.readiness.as_ref().map(|block| block.m1.ready);
        println!(
            "primitives pair={} enabled={enabled:?} run_state={run_state:?} seed_done={seed_done:?} ready_1m={ready_1m:?}",
            row.pair
        );
    }
}

fn print_regime_status(response: &mathilde_sdk_rs::systems::regime::PairsStatusResponse) {
    println!("regime status_rows={}", response.pairs.len());
    for row in &response.pairs {
        let enabled = row.status.as_ref().map(|block| block.enabled);
        let run_state = row.status.as_ref().map(|block| block.run_state.as_str());
        let seed_done = row.history.as_ref().and_then(|block| block.seed_done);
        let ready_1h = row.readiness.as_ref().map(|block| block.h1.ready);
        println!(
            "regime pair={} enabled={enabled:?} run_state={run_state:?} seed_done={seed_done:?} ready_1h={ready_1h:?}",
            row.pair
        );
    }
}
