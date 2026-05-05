// Example: one latest bars request across HTTP JSON, HTTP protobuf, and gRPC
//
// What this example does:
// 1. Builds one authenticated Aggregator client.
// 2. Sends one latest request over HTTP JSON.
// 3. Sends the same latest request over HTTP protobuf.
// 4. Adapts the same latest request into a gRPC request with From<&LatestRequest>
//    and sends it over gRPC.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client
// - one shared base request
// - one compact summary per transport path

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{
    Aggregator, LatestGrpcRequest as AggregatorLatestGrpcRequest,
    LatestRequest as AggregatorLatestRequest, LatestResponse,
};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::types::{HttpFormat, LatestMode, Timeframe};

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    // Read the example bearer once and reuse it across all transport calls.
    let bearer = read_example_bearer()?;

    // Build one Aggregator client with HTTP and gRPC transports enabled.
    let aggregator = Aggregator::client(Some(bearer))?;

    // Define the request once, then reuse it across the three transport paths.
    let latest_request = AggregatorLatestRequest {
        pairs: pairs(["BTCUSDT"]),
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(true),
        format: None,
    };

    let http_json = aggregator
        .latest(&AggregatorLatestRequest {
            format: Some(HttpFormat::Json),
            ..latest_request.clone()
        })
        .await?;

    let http_protobuf = aggregator
        .latest(&AggregatorLatestRequest {
            format: Some(HttpFormat::Protobuf),
            ..latest_request.clone()
        })
        .await?;

    // The gRPC request reuses the same semantic fields; the adapter removes the
    // HTTP-only format field and keeps the rest unchanged.
    let grpc_request = AggregatorLatestGrpcRequest::from(&latest_request);
    let grpc = aggregator.latest_grpc(&grpc_request).await?;

    print_latest_summary("http json", &http_json);
    print_latest_summary("http protobuf", &http_protobuf);
    print_latest_summary("grpc", &grpc);

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

fn print_latest_summary(label: &str, response: &LatestResponse) {
    println!(
        "{label} rows={} latest_mode={:?} missing_pairs={}",
        response.rows.len(),
        response.latest_mode,
        response.missing_pairs.len()
    );

    if let Some(bar) = response.rows.first() {
        println!(
            "{label} pair={} close_utc={} close={} age_ms={:?} metadata_present={}",
            bar.pair,
            bar.close_utc,
            bar.c,
            bar.age_ms,
            bar.metadata.is_some()
        );
    }
}
