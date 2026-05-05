// Workflow: understanding_system
//
// This file preserves the authoritative understanding_workflow from https://api.mathilde.dev
//
// Step 1
// Surface: Aggregator summary
// Route: GET https://aggregator.api.mathilde.dev/v1/docs/summary
// Question: What problem does MATHILDE solve at the bar-truth layer?
// Why now: Start here because it is the shortest correct orientation to stable
// boundaries, canonical minute truth, and why raw market streams are not yet a
// safe dataset.
// Do not start with: Do not start with OpenAPI, taxonomy, or registry before
// this orientation exists.
//
// Step 2
// Surface: Aggregator system
// Route: GET https://aggregator.api.mathilde.dev/v1/docs/system
// Question: How does Aggregator turn imperfect external streams into bounded,
// safe-to-serve bars?
// Why now: This is the conceptual foundation for every downstream surface.
// Do not start with: Do not move to computed outputs before the upstream
// bar-truth contract is clear.
//
// Step 3
// Surface: Aggregator endpoints
// Route: GET https://aggregator.api.mathilde.dev/v1/docs/endpoints
// Question: Now that bars are understood, which read family retrieves them
// correctly?
// Why now: Endpoint family choice becomes meaningful only after the measured
// object is understood.
// Do not start with: Do not use route names alone to infer family meaning.
//
// Step 4
// Surface: Primitives summary
// Route: GET https://primitives.api.mathilde.dev/v1/docs/summary
// Question: What is a primitives outputs row in MATHILDE terms?
// Why now: Primitives is downstream of Aggregator, so it should be read only
// after bar truth is clear.
// Do not start with: Do not begin selector discovery before the outputs row
// itself is understood.
//
// Step 5
// Surface: Primitives system
// Route: GET https://primitives.api.mathilde.dev/v1/docs/system
// Question: What counts as a primitive measurement, and why are outputs grouped
// this way?
// Why now: This document explains the conceptual model behind the grouped
// outputs surface.
// Do not start with: Do not jump into taxonomy or registry before this model
// is clear.
//
// Step 6
// Surface: Primitives taxonomy
// Route: GET https://primitives.api.mathilde.dev/v1/docs/taxonomy
// Question: Which primitive families and groups exist?
// Why now: Taxonomy is the selector-space map. It should narrow the search
// space before deeper algorithm research.
// Do not start with: Do not treat taxonomy as an onboarding narrative. It is a
// discovery payload.
//
// Step 7
// Surface: Primitives registry
// Route: GET https://primitives.api.mathilde.dev/v1/docs/registry
// Question: Which exact primitive algorithms and shipped outputs exist inside
// the selected families?
// Why now: Registry is large and only becomes legible after taxonomy has
// narrowed the space.
// Do not start with: Do not read the full registry cold if the relevant
// families are still unknown.
//
// Step 8
// Surface: Primitives endpoints
// Route: GET https://primitives.api.mathilde.dev/v1/docs/endpoints
// Question: Given a known outputs object and selector space, which read family
// should be called?
// Why now: This is the correct time to convert conceptual understanding into
// retrieval routing.
// Do not start with: Do not begin with endpoints before the outputs object and
// selector space are understood.
//
// Step 9
// Surface: Regime summary
// Route: GET https://regime.api.mathilde.dev/v1/docs/summary
// Question: What is Regime measuring at a high level?
// Why now: Regime is easier to place after lower-level bars and primitives
// surfaces are already clear.
// Do not start with: Do not start here if bar truth and primitive outputs are
// still conceptually unclear.
//
// Step 10
// Surface: Regime system
// Route: GET https://regime.api.mathilde.dev/v1/docs/system
// Question: How is the fixed family-and-question matrix organized, and why is
// Regime 1h only?
// Why now: This is the conceptual contract for Regime as a measured state
// system.
// Do not start with: Do not infer the matrix from route names alone.
//
// Step 11
// Surface: Regime taxonomy
// Route: GET https://regime.api.mathilde.dev/v1/docs/taxonomy
// Question: Which dimensions and question slots exist inside the Regime matrix?
// Why now: Taxonomy makes the matrix machine-readable after the system-level
// explanation is clear.
// Do not start with: Do not use taxonomy as the first narrative explanation of
// Regime.
//
// Step 12
// Surface: Regime registry
// Route: GET https://regime.api.mathilde.dev/v1/docs/registry
// Question: Which exact Regime kernels, questions, and shipped fields exist?
// Why now: Registry is the deeper kernel-level discovery surface after taxonomy
// has narrowed the search space.
// Do not start with: Do not read the full registry cold before the matrix and
// taxonomy are known.
//
// Step 13
// Surface: Regime endpoints
// Route: GET https://regime.api.mathilde.dev/v1/docs/endpoints
// Question: Now that the Regime matrix is understood, which read family
// retrieves it correctly?
// Why now: This is the final routing step for the Regime surface itself.
// Do not start with: Do not start with endpoints before understanding what
// Regime is measuring.
//
// Step 14
// Surface: OpenAPI
// Route: GET https://aggregator.api.mathilde.dev/openapi.json,
// GET https://primitives.api.mathilde.dev/openapi.json,
// GET https://regime.api.mathilde.dev/openapi.json
// Question: What is the exact transport and schema contract once the object
// and family are already known?
// Why now: OpenAPI is exact and useful, but it is a transport contract, not an
// onboarding explanation.
// Do not start with: Do not start here if the measured object or endpoint
// family is still unclear.
//
// Example: understanding the public system surfaces in the intended order
//
// What this example does:
// 1. Confirms that the live intro root still exposes understanding_workflow.
// 2. Walks the Aggregator, Primitives, and Regime documentation surfaces in the
//    same order as the authoritative workflow above.
// 3. Uses filtered registry requests for the large compute registries so the
//    example stays readable while still demonstrating the intended discovery
//    flow.
// 4. Ends with the three OpenAPI documents as the final transport-contract
//    step.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one compact summary per workflow step

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
    // Read the example bearer once and reuse it across all clients.
    let bearer = read_example_bearer()?;

    // Build one client per public system surface used by the workflow.
    let intro = Intro::client(Some(bearer.clone()))?;
    let aggregator = Aggregator::client(Some(bearer.clone()))?;
    let primitives = Primitives::client(Some(bearer.clone()))?;
    let regime = Regime::client(Some(bearer))?;

    // Confirm that the live intro root still exposes the understanding workflow.
    let intro_doc = intro.intro().await?;
    println!(
        "intro understanding_workflow steps={}",
        array_len(&intro_doc, "understanding_workflow")
    );

    let aggregator_summary = aggregator.docs_summary().await?;
    let aggregator_system = aggregator.docs_system().await?;
    let aggregator_endpoints = aggregator.docs_endpoints().await?;
    print_doc_shape("step 1 aggregator summary", &aggregator_summary);
    print_doc_shape("step 2 aggregator system", &aggregator_system);
    print_doc_shape("step 3 aggregator endpoints", &aggregator_endpoints);

    let primitives_summary = primitives.docs_summary().await?;
    let primitives_system = primitives.docs_system().await?;
    let primitives_taxonomy = primitives.docs_taxonomy().await?;

    // Registry is intentionally filtered so the example models the "narrow first,
    // then inspect exact algorithms" workflow instead of dumping the full surface.
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
    print_doc_shape("step 4 primitives summary", &primitives_summary);
    print_doc_shape("step 5 primitives system", &primitives_system);
    print_doc_shape("step 6 primitives taxonomy", &primitives_taxonomy);
    print_doc_shape("step 7 primitives filtered registry", &primitives_registry);
    print_doc_shape("step 8 primitives endpoints", &primitives_endpoints);

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
    print_doc_shape("step 9 regime summary", &regime_summary);
    print_doc_shape("step 10 regime system", &regime_system);
    print_doc_shape("step 11 regime taxonomy", &regime_taxonomy);
    print_doc_shape("step 12 regime filtered registry", &regime_registry);
    print_doc_shape("step 13 regime endpoints", &regime_endpoints);

    let aggregator_openapi = aggregator.openapi().await?;
    let primitives_openapi = primitives.openapi().await?;
    let regime_openapi = regime.openapi().await?;
    print_openapi_shape("step 14 aggregator openapi", &aggregator_openapi);
    print_openapi_shape("step 14 primitives openapi", &primitives_openapi);
    print_openapi_shape("step 14 regime openapi", &regime_openapi);

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

fn print_openapi_shape(label: &str, value: &Value) {
    let top_level_keys = value.as_object().map_or(0, |object| object.len());
    let paths = value
        .get("paths")
        .and_then(Value::as_object)
        .map_or(0, |object| object.len());
    println!("{label} top_level_keys={top_level_keys} paths={paths}");
}

fn array_len(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}
