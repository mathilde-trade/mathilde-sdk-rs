// Workflow: due_diligence_review_packs
//
// This workflow is separate from understanding_workflow.
//
// It covers the intro-host due-diligence surface for approved review packs on
// https://api.mathilde.dev/v1/due-diligence.
//
// What this example does:
// 1. Reads the intro root and confirms the due-diligence discovery pointer is present.
// 2. Reads the due-diligence index on the intro host.
// 3. Reads the two approved regime review packs.
// 4. Reads the two approved primitives family review packs.
// 5. Prints one compact summary per returned document.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one Intro client for the single host used
// - one small print summary per returned document

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::intro::Intro;
use serde_json::Value;

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    // Read the example bearer once and reuse it across the one intro-host client.
    let bearer = read_example_bearer()?;

    // Build one client for the deploy-owned intro and due-diligence surface.
    let intro = Intro::client(Some(bearer))?;

    let intro_doc = intro.intro().await?;
    println!(
        "intro due_diligence_pointer_present={}",
        intro_doc
            .to_string()
            .contains("https://api.mathilde.dev/v1/due-diligence")
    );

    let due_diligence_index = intro.due_diligence().await?;
    println!(
        "due_diligence available_packs={}",
        array_len(&due_diligence_index, "available_packs")
    );

    let regime_kalman = intro
        .due_diligence_regime_kalman_local_trend_state()
        .await?;
    let regime_flow = intro
        .due_diligence_regime_flow_absorption_elasticity_state()
        .await?;
    let primitives_correlation = intro.due_diligence_primitives_correlation().await?;
    let primitives_drawdown = intro.due_diligence_primitives_drawdown().await?;

    print_pack_shape("regime kalman_local_trend_state", &regime_kalman);
    print_pack_shape("regime flow_absorption_elasticity_state", &regime_flow);
    print_pack_shape("primitives correlation", &primitives_correlation);
    print_pack_shape("primitives drawdown", &primitives_drawdown);

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

fn print_pack_shape(label: &str, value: &Value) {
    let review_count = value
        .get("review_display_order")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    println!(
        "{label} system={} subject_id={} review_count={review_count}",
        value
            .get("system")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        value
            .get("subject_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
}

fn array_len(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}
