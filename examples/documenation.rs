// Example: public documentation surfaces
//
// What this example does:
// 1. Reads the public intro root to confirm the available workflow sections.
// 2. Fetches the public documentation surfaces for Aggregator.
// 3. Fetches the public documentation surfaces for Primitives, including a
//    filtered registry request by family and group.
// 4. Fetches the public documentation surfaces for Regime, including a
//    filtered registry request by family and group.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one small print summary per returned document

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::Aggregator;
use mathilde_sdk_rs::systems::intro::Intro;
use mathilde_sdk_rs::systems::primitives::{
    DocsRegistryRequest as PrimitivesDocsRegistryRequest, Primitives,
    ProcessorFamily as PrimitivesProcessorFamily, ProcessorGroup as PrimitivesProcessorGroup,
};
use mathilde_sdk_rs::systems::regime::{
    DocsRegistryRequest as RegimeDocsRegistryRequest, ProcessorFamily as RegimeProcessorFamily,
    ProcessorGroup as RegimeProcessorGroup, Regime,
};
use serde_json::Value;

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    // Read the public bearer once and reuse it across all clients.
    let bearer = read_public_bearer()?;

    // Build one client per public system surface.
    let intro = Intro::client(Some(bearer.clone()))?;
    let aggregator = Aggregator::client(Some(bearer.clone()))?;
    let primitives = Primitives::client(Some(bearer.clone()))?;
    let regime = Regime::client(Some(bearer))?;

    // Confirm the live intro root still exposes the workflow sections.
    let intro_doc = intro.intro().await?;
    println!(
        "intro understanding_workflow steps={}",
        array_len(&intro_doc, "understanding_workflow")
    );
    println!(
        "intro example_workflows count={}",
        array_len(&intro_doc, "example_workflows")
    );

    // Read the public Aggregator documentation surfaces.
    let aggregator_summary = aggregator.docs_summary().await?;
    let aggregator_system = aggregator.docs_system().await?;
    let aggregator_themes = aggregator.docs_themes().await?;
    let aggregator_endpoints = aggregator.docs_endpoints().await?;
    print_doc_shape("aggregator summary", &aggregator_summary);
    print_doc_shape("aggregator system", &aggregator_system);
    print_doc_shape("aggregator themes", &aggregator_themes);
    print_doc_shape("aggregator endpoints", &aggregator_endpoints);

    // Read the public Primitives documentation surfaces and a filtered registry slice.
    let primitives_summary = primitives.docs_summary().await?;
    let primitives_system = primitives.docs_system().await?;
    let primitives_taxonomy = primitives.docs_taxonomy().await?;

    let primitives_registry = primitives
        .docs_registry(&PrimitivesDocsRegistryRequest {
            family: Some(vec![
                PrimitivesProcessorFamily::Drawdown,
                PrimitivesProcessorFamily::Oscillators,
            ]),
            group: Some(vec![
                PrimitivesProcessorGroup::DrawdownSeries,
                PrimitivesProcessorGroup::Rsi,
            ]),
        })
        .await?;

    let primitives_endpoints = primitives.docs_endpoints().await?;
    print_doc_shape("primitives summary", &primitives_summary);
    print_doc_shape("primitives system", &primitives_system);
    print_doc_shape("primitives taxonomy", &primitives_taxonomy);
    print_doc_shape("primitives filtered registry", &primitives_registry);
    print_doc_shape("primitives endpoints", &primitives_endpoints);

    // Read the public Regime documentation surfaces and a filtered registry slice.
    let regime_summary = regime.docs_summary().await?;
    let regime_system = regime.docs_system().await?;
    let regime_taxonomy = regime.docs_taxonomy().await?;

    let regime_registry = regime
        .docs_registry(&RegimeDocsRegistryRequest {
            family: Some(vec![
                RegimeProcessorFamily::Trend,
                RegimeProcessorFamily::Dependency,
            ]),
            group: Some(vec![
                RegimeProcessorGroup::TrendQ1,
                RegimeProcessorGroup::DependencyBtcQ1,
            ]),
        })
        .await?;

    let regime_endpoints = regime.docs_endpoints().await?;
    print_doc_shape("regime summary", &regime_summary);
    print_doc_shape("regime system", &regime_system);
    print_doc_shape("regime taxonomy", &regime_taxonomy);
    print_doc_shape("regime filtered registry", &regime_registry);
    print_doc_shape("regime endpoints", &regime_endpoints);

    Ok(())
}

fn read_public_bearer() -> Result<BearerToken, Box<dyn Error>> {
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

fn array_len(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}
