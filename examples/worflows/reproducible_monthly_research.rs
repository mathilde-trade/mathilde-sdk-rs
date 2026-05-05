// Workflow: reproducible_monthly_research
//
// This file preserves the authoritative example_workflows entry from https://api.mathilde.dev
//
// Question: How do I build a reproducible monthly BTC 1h research slice that
// joins stable bars and computed outputs locally?
// Why: This workflow is for offline reproducible analysis rather than
// interactive live retrieval.
//
// Steps:
// 1. Use: Call Aggregator file-download discovery for the required monthly BTC
//    1h bars objects.
//    Route: POST https://aggregator.api.mathilde.dev/v1/files/downloads
//    Retrieve: Signed parquet download URLs for the required bars slice.
// 2. Use: Call Primitives file-download discovery for the matching monthly BTC
//    1h outputs objects.
//    Route: POST https://primitives.api.mathilde.dev/v1/files/downloads
//    Retrieve: Signed parquet download URLs for the required outputs slice.
// 3. Use: Download the parquet files.
//    Route: same signed URLs
//    Retrieve: The exact monthly files requested from both surfaces.
// 4. Use: Join the files locally on canonical row identity.
//    Route: local alignment step
//    Retrieve: A reproducible offline research table with aligned bars and
//    outputs.
// 5. Use: Preserve the pair, timeframe, month labels, and file identities.
//    Route: local bookkeeping step
//    Retrieve: An auditable research slice that can be rebuilt later.
//
// Stop when: The monthly slice is downloaded, joined, and locally reproducible.
// Non-goal: Do not use this workflow when the real question is the newest
// stable live snapshot.
//
// Example: reproducible monthly BTC 1h research slice
//
// What this example does:
// 1. Builds one authenticated Aggregator client and one authenticated
//    Primitives client.
// 2. Discovers one monthly BTCUSDT 1h file from each surface for the same
//    month label.
// 3. Downloads both parquet files into an example-local timestamped run
//    directory.
// 4. Writes a manifest and the exact DuckDB join SQL used for local
//    reproducibility.
// 5. Runs the local DuckDB join and materializes one joined monthly parquet
//    slice.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one month label reused across both file-discovery requests
// - one explicit timestamped run directory under examples/downloads
// - one local join contract based on canonical row identity

use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{
    Aggregator, FilesDownloadsRequest as AggregatorFilesDownloadsRequest,
};
use mathilde_sdk_rs::systems::primitives::{
    FilesDownloadsRequest as PrimitivesFilesDownloadsRequest, Primitives,
};
use serde_json::json;

const PERIOD: &str = "month";
const PAIR: &str = "BTCUSDT";
const TF: &str = "1h";
const MONTH_LABEL_UTC: &str = "2026-04";

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    // Read the example bearer once and reuse it across both clients.
    let bearer = read_example_bearer()?;

    // Build one client per public system surface used by this workflow.
    let aggregator = Aggregator::client(Some(bearer.clone()))?;
    let primitives = Primitives::client(Some(bearer))?;

    let period = Some(PERIOD.to_string());
    let pairs = vec![PAIR.to_string()];
    let tfs = vec![TF.to_string()];
    let start_label_utc = Some(MONTH_LABEL_UTC.to_string());
    let end_label_utc = Some(MONTH_LABEL_UTC.to_string());
    let order = Some("desc".to_string());

    // Discover the exact monthly bars and outputs objects for the same month label.
    let aggregator_files = aggregator
        .files_downloads(&AggregatorFilesDownloadsRequest {
            period: period.clone(),
            pairs: pairs.clone(),
            tfs: tfs.clone(),
            start_label_utc: start_label_utc.clone(),
            end_label_utc: end_label_utc.clone(),
            order: order.clone(),
        })
        .await?;

    let primitives_files = primitives
        .files_downloads(&PrimitivesFilesDownloadsRequest {
            period,
            pairs,
            tfs,
            start_label_utc,
            end_label_utc,
            order,
        })
        .await?;

    println!(
        "aggregator monthly files_rows={}",
        aggregator_files.rows.len()
    );
    println!(
        "primitives monthly files_rows={}",
        primitives_files.rows.len()
    );

    let aggregator_row = aggregator_files
        .rows
        .first()
        .ok_or("aggregator monthly files discovery returned no rows")?;
    let primitives_row = primitives_files
        .rows
        .first()
        .ok_or("primitives monthly files discovery returned no rows")?;

    print_first_row(
        "aggregator",
        &aggregator_row.period,
        &aggregator_row.pair,
        &aggregator_row.tf,
        &aggregator_row.label_utc,
        &aggregator_row.expires_at_utc,
    );
    print_first_row(
        "primitives",
        &primitives_row.period,
        &primitives_row.pair,
        &primitives_row.tf,
        &primitives_row.label_utc,
        &primitives_row.expires_at_utc,
    );

    let run_root = build_run_root();
    let aggregator_root = run_root.join("aggregator");
    let primitives_root = run_root.join("primitives");

    // Download the exact monthly files into this run directory so the slice can
    // be inspected or removed easily after testing.
    let aggregator_download = aggregator
        .files_download_items(
            std::slice::from_ref(aggregator_row),
            Some(aggregator_root.as_path()),
        )
        .await?;
    let primitives_download = primitives
        .files_download_items(
            std::slice::from_ref(primitives_row),
            Some(primitives_root.as_path()),
        )
        .await?;

    let aggregator_file = aggregator_download
        .first()
        .ok_or("aggregator files_download_items returned no downloaded files")?;
    let primitives_file = primitives_download
        .first()
        .ok_or("primitives files_download_items returned no downloaded files")?;

    print_download_summary(
        "aggregator",
        &aggregator_file.destination_path,
        aggregator_file.bytes_written,
    );
    print_download_summary(
        "primitives",
        &primitives_file.destination_path,
        primitives_file.bytes_written,
    );

    let join_sql_path = run_root.join("join.sql");
    let duckdb_path = run_root.join("research_slice.duckdb");
    let joined_parquet_path = run_root.join("joined_monthly_slice.parquet");
    let manifest_path = run_root.join("slice_manifest.json");

    let join_sql = build_join_sql(
        &aggregator_file.destination_path,
        &primitives_file.destination_path,
        &joined_parquet_path,
    );
    std::fs::write(&join_sql_path, &join_sql)?;

    let joined_rows = run_duckdb_join(&duckdb_path, &join_sql)?;

    let manifest = json!({
        "created_at_utc": Utc::now().to_rfc3339(),
        "pair": PAIR,
        "tf": TF,
        "period": PERIOD,
        "month_label_utc": MONTH_LABEL_UTC,
        "canonical_row_identity": {
            "public_identity": ["pair", "tf", "close_ms"],
            "local_join_keys_used": ["pair", "close_ms"],
            "why": "tf is fixed to 1h for this monthly slice, so pair+close_ms is sufficient for the local file join"
        },
        "files": {
            "aggregator": {
                "period": aggregator_row.period,
                "pair": aggregator_row.pair,
                "tf": aggregator_row.tf,
                "label_utc": aggregator_row.label_utc,
                "expires_at_utc": aggregator_row.expires_at_utc,
                "destination_path": aggregator_file.destination_path,
                "bytes_written": aggregator_file.bytes_written
            },
            "primitives": {
                "period": primitives_row.period,
                "pair": primitives_row.pair,
                "tf": primitives_row.tf,
                "label_utc": primitives_row.label_utc,
                "expires_at_utc": primitives_row.expires_at_utc,
                "destination_path": primitives_file.destination_path,
                "bytes_written": primitives_file.bytes_written
            }
        },
        "local_outputs": {
            "run_root": run_root.display().to_string(),
            "duckdb_path": duckdb_path.display().to_string(),
            "join_sql_path": join_sql_path.display().to_string(),
            "joined_parquet_path": joined_parquet_path.display().to_string(),
            "joined_rows": joined_rows
        }
    });
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    println!("research slice run_root={}", run_root.display());
    println!("research slice duckdb_path={}", duckdb_path.display());
    println!("research slice join_sql_path={}", join_sql_path.display());
    println!(
        "research slice joined_parquet_path={} joined_rows={}",
        joined_parquet_path.display(),
        joined_rows
    );
    println!("research slice manifest_path={}", manifest_path.display());

    Ok(())
}

fn build_run_root() -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%dT%H%M%SZ");
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("downloads")
        .join(format!("reproducible_monthly_research_{timestamp}"))
}

fn build_join_sql(
    aggregator_path: &str,
    primitives_path: &str,
    joined_parquet_path: &Path,
) -> String {
    let aggregator_path = escape_sql_path(aggregator_path);
    let primitives_path = escape_sql_path(primitives_path);
    let joined_parquet_path = escape_sql_path(&joined_parquet_path.display().to_string());

    format!(
        "-- Canonical public row identity for this slice is (pair, tf, close_ms).\n\
         -- The local file join uses (pair, close_ms) because tf is fixed to {TF}.\n\
         CREATE OR REPLACE TABLE monthly_btc_1h_research_slice AS\n\
         SELECT\n\
             a.pair,\n\
             a.tf,\n\
             a.s_ms AS open_ms,\n\
             a.e_ms AS close_ms,\n\
             a.s_utc AS open_utc,\n\
             a.e_utc AS close_utc,\n\
             a.o,\n\
             a.h,\n\
             a.l,\n\
             a.c,\n\
             a.v,\n\
             a.n,\n\
             a.quote_v,\n\
             a.taker_known_v,\n\
             a.taker_signed_v,\n\
             a.taker_known_quote_v,\n\
             a.taker_signed_quote_v,\n\
             a.taker_known_n,\n\
             a.taker_signed_n,\n\
             a.vw,\n\
             a.* EXCLUDE (\n\
                 pair,\n\
                 tf,\n\
                 s_ms,\n\
                 e_ms,\n\
                 s_utc,\n\
                 e_utc,\n\
                 o,\n\
                 h,\n\
                 l,\n\
                 c,\n\
                 v,\n\
                 n,\n\
                 quote_v,\n\
                 taker_known_v,\n\
                 taker_signed_v,\n\
                 taker_known_quote_v,\n\
                 taker_signed_quote_v,\n\
                 taker_known_n,\n\
                 taker_signed_n,\n\
                 vw\n\
             ),\n\
             p.* EXCLUDE (\n\
                 pair,\n\
                 open_time_ms,\n\
                 close_time_ms,\n\
                 open_time_utc,\n\
                 close_time_utc,\n\
                 o,\n\
                 h,\n\
                 l,\n\
                 c,\n\
                 v,\n\
                 n,\n\
                 quote_v,\n\
                 taker_known_v,\n\
                 taker_signed_v,\n\
                 taker_known_quote_v,\n\
                 taker_signed_quote_v,\n\
                 taker_known_n,\n\
                 taker_signed_n,\n\
                 vw\n\
             )\n\
         FROM read_parquet('{aggregator_path}') AS a\n\
         INNER JOIN read_parquet('{primitives_path}') AS p\n\
             ON a.pair = p.pair AND a.e_ms = p.close_time_ms\n\
         ORDER BY close_ms;\n\
         COPY (\n\
             SELECT *\n\
             FROM monthly_btc_1h_research_slice\n\
             ORDER BY close_ms\n\
         ) TO '{joined_parquet_path}' (FORMAT parquet);\n\
         SELECT count(*) AS joined_rows FROM monthly_btc_1h_research_slice;\n"
    )
}

fn run_duckdb_join(duckdb_path: &Path, join_sql: &str) -> Result<u64, Box<dyn Error>> {
    let duckdb = find_duckdb_binary()?;
    let output = Command::new(duckdb)
        .arg(duckdb_path)
        .arg("-batch")
        .arg("-csv")
        .arg("-noheader")
        .arg("-c")
        .arg(join_sql)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("duckdb join failed stdout={stdout} stderr={stderr}").into());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let joined_rows = stdout
        .lines()
        .rev()
        .find_map(|line| line.trim().parse::<u64>().ok())
        .ok_or("duckdb join did not print a final joined_rows count")?;

    Ok(joined_rows)
}

fn find_duckdb_binary() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(path) = env::var_os("DUCKDB_BIN") {
        return Ok(PathBuf::from(path));
    }

    let path_var = env::var_os("PATH").ok_or("PATH is not available for duckdb lookup")?;
    for candidate_dir in env::split_paths(&path_var) {
        let candidate = candidate_dir.join("duckdb");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(
        "duckdb was not found in PATH; install duckdb or export DUCKDB_BIN before running this example"
            .into(),
    )
}

fn escape_sql_path(path: &str) -> String {
    path.replace('\'', "''")
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
