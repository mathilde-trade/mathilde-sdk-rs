// Example: public files listing and controlled download
//
// What this example does:
// 1. Builds one authenticated client per public system surface.
// 2. Requests one small files window from Aggregator, Primitives, and Regime.
// 3. Prints one compact listing summary per system.
// 4. Downloads at most one selected row per system into explicit example-local folders.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one bounded listing request per system
// - one explicit download selection per system

use std::env;
use std::error::Error;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{
    Aggregator, FilesDownloadsRequest as AggregatorFilesDownloadsRequest,
};
use mathilde_sdk_rs::systems::primitives::{
    FilesDownloadsRequest as PrimitivesFilesDownloadsRequest, Primitives,
};
use mathilde_sdk_rs::systems::regime::{
    FilesDownloadsRequest as RegimeFilesDownloadsRequest, Regime,
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

    let period = Some("day".to_string());
    let target_pairs = vec!["BTCUSDT".to_string()];
    let target_tfs = vec!["1h".to_string()];
    let start_label_utc = Some("2026-05-01".to_string());
    let end_label_utc = Some("2026-05-01".to_string());
    let order = Some("desc".to_string());

    // Request one small files window per system.
    let aggregator_files = aggregator
        .files_downloads(&AggregatorFilesDownloadsRequest {
            period: period.clone(),
            pairs: target_pairs.clone(),
            tfs: target_tfs.clone(),
            start_label_utc: start_label_utc.clone(),
            end_label_utc: end_label_utc.clone(),
            order: order.clone(),
        })
        .await?;

    let primitives_files = primitives
        .files_downloads(&PrimitivesFilesDownloadsRequest {
            period: period.clone(),
            pairs: target_pairs.clone(),
            tfs: target_tfs.clone(),
            start_label_utc: start_label_utc.clone(),
            end_label_utc: end_label_utc.clone(),
            order: order.clone(),
        })
        .await?;

    let regime_files = regime
        .files_downloads(&RegimeFilesDownloadsRequest {
            period,
            pairs: target_pairs,
            tfs: target_tfs,
            start_label_utc,
            end_label_utc,
            order,
        })
        .await?;

    print_listing_summary("aggregator", aggregator_files.rows.len());
    if let Some(row) = aggregator_files.rows.first() {
        print_first_row(
            "aggregator",
            &row.period,
            &row.pair,
            &row.tf,
            &row.label_utc,
            &row.expires_at_utc,
        );
    }

    print_listing_summary("primitives", primitives_files.rows.len());
    if let Some(row) = primitives_files.rows.first() {
        print_first_row(
            "primitives",
            &row.period,
            &row.pair,
            &row.tf,
            &row.label_utc,
            &row.expires_at_utc,
        );
    }

    print_listing_summary("regime", regime_files.rows.len());
    if let Some(row) = regime_files.rows.first() {
        print_first_row(
            "regime",
            &row.period,
            &row.pair,
            &row.tf,
            &row.label_utc,
            &row.expires_at_utc,
        );
    }

    // Download at most one explicit row per system into separate example-local roots.
    let example_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("downloads")
        .join("files");
    let aggregator_root = example_root.join("aggregator");
    let primitives_root = example_root.join("primitives");
    let regime_root = example_root.join("regime");

    if let Some(row) = aggregator_files.rows.first() {
        let downloaded = aggregator
            .files_download_items(std::slice::from_ref(row), Some(aggregator_root.as_path()))
            .await?;

        if let Some(file) = downloaded.first() {
            print_download_summary("aggregator", &file.destination_path, file.bytes_written);
        }
    } else {
        println!("aggregator no download rows matched this files window");
    }

    if let Some(row) = primitives_files.rows.first() {
        let downloaded = primitives
            .files_download_items(std::slice::from_ref(row), Some(primitives_root.as_path()))
            .await?;

        if let Some(file) = downloaded.first() {
            print_download_summary("primitives", &file.destination_path, file.bytes_written);
        }
    } else {
        println!("primitives no download rows matched this files window");
    }

    if let Some(row) = regime_files.rows.first() {
        let downloaded = regime
            .files_download_items(std::slice::from_ref(row), Some(regime_root.as_path()))
            .await?;

        if let Some(file) = downloaded.first() {
            print_download_summary("regime", &file.destination_path, file.bytes_written);
        }
    } else {
        println!("regime no download rows matched this files window");
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

fn print_listing_summary(label: &str, row_count: usize) {
    println!("{label} files_rows={row_count}");
}

fn print_first_row(
    label: &str,
    period: &str,
    pair: &str,
    tf: &str,
    label_utc: &str,
    expires_at_utc: &str,
) {
    println!(
        "{label} first_row period={period} pair={pair} tf={tf} label_utc={label_utc} expires_at_utc={expires_at_utc}"
    );
}

fn print_download_summary(label: &str, destination_path: &str, bytes_written: u64) {
    println!(
        "{label} downloaded destination_path={destination_path} bytes_written={bytes_written}"
    );
}
