use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::core::config::{IntroConfig, MathildePublicHosts};
use mathilde_sdk_rs::core::error::SdkError;
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::generated::aggregator::bars_proto::mathilde::feed::bars::v1 as aggregator_proto;
use mathilde_sdk_rs::generated::primitives::{
    ProcessorFamily as PrimitiveProcessorFamily, ProcessorGroup as PrimitiveProcessorGroup,
    outputs_proto::mathilde::feed::outputs::v1 as primitives_proto,
};
use mathilde_sdk_rs::generated::regime::outputs_proto::mathilde::feed::outputs::v1 as regime_proto;
use mathilde_sdk_rs::streaming::subscription::ExponentialBackoffConfig;
use mathilde_sdk_rs::systems::aggregator::{
    Aggregator, BarsWsInboundFrame, BarsWsSubscribeRequest, FilesDownloadsRequest,
    LatestGrpcRequest, LatestRequest, LatestResponse,
    MessagesWsServerFrame as AggregatorMessagesWsServerFrame,
    MessagesWsSubscribeFrame as AggregatorMessagesWsSubscribeFrame,
    MessagesWsUnsubscribeFrame as AggregatorMessagesWsUnsubscribeFrame, PairsListRequest,
    PairsStatusRequest, RangeGrpcRequest, RangeRequest, RangeResponse, SearchGrpcRequest,
    SearchRequest, SearchResponse, TimeMachineGrpcRequest, TimeMachineRequest, TimeMachineResponse,
};
use mathilde_sdk_rs::systems::intro::Intro;
use mathilde_sdk_rs::systems::primitives::{
    self as primitives_system, MessagesWsServerFrame as PrimitiveMessagesWsServerFrame,
    MessagesWsSubscribeFrame as PrimitiveMessagesWsSubscribeFrame,
    MessagesWsUnsubscribeFrame as PrimitiveMessagesWsUnsubscribeFrame, OutputsWsFormat,
    OutputsWsInboundFrame as PrimitiveOutputsWsInboundFrame, OutputsWsSubscribeRequest, Primitives,
};
use mathilde_sdk_rs::systems::regime::{
    self as regime_system, MessagesWsServerFrame as RegimeMessagesWsServerFrame,
    MessagesWsSubscribeFrame as RegimeMessagesWsSubscribeFrame,
    MessagesWsUnsubscribeFrame as RegimeMessagesWsUnsubscribeFrame,
    OutputsWsFormat as RegimeOutputsWsFormat, OutputsWsInboundFrame as RegimeOutputsWsInboundFrame,
    OutputsWsSubscribeRequest as RegimeOutputsWsSubscribeRequest, Regime,
};
use mathilde_sdk_rs::systems::types::{AlignMode, HttpFormat, LatestMode, Timeframe};
use serde_json::{Value, json};
use tokio::time::{sleep, timeout};
use tonic::client::Grpc;
use tonic::codec::ProstCodec;
use tonic::codegen::http::uri::PathAndQuery;
use tonic::metadata::MetadataValue;
use tonic::transport::Endpoint;

const BIN_NAME: &str = "endpoint_test";
const BEARER_ENV: &str = "AGGREGATOR_FEED_BEARER_TOKEN";
const AGGREGATOR_LATEST_GPRC_PATH: &str = "/mathilde.feed.bars.v1.BarsServiceV1/LatestBars";
const AGGREGATOR_RANGE_GPRC_PATH: &str = "/mathilde.feed.bars.v1.BarsServiceV1/RangeBars";
const AGGREGATOR_SEARCH_GPRC_PATH: &str = "/mathilde.feed.bars.v1.BarsServiceV1/SearchBars";
const AGGREGATOR_TIME_MACHINE_GPRC_PATH: &str =
    "/mathilde.feed.bars.v1.BarsServiceV1/TimeMachineBars";
const OUTPUTS_LATEST_GPRC_PATH: &str = "/mathilde.feed.outputs.v1.OutputsServiceV1/LatestOutputs";
const OUTPUTS_RANGE_GPRC_PATH: &str = "/mathilde.feed.outputs.v1.OutputsServiceV1/RangeOutputs";
const OUTPUTS_SEARCH_GPRC_PATH: &str = "/mathilde.feed.outputs.v1.OutputsServiceV1/SearchOutputs";
const OUTPUTS_TIME_MACHINE_GPRC_PATH: &str =
    "/mathilde.feed.outputs.v1.OutputsServiceV1/TimeMachineOutputs";

#[derive(Debug, Clone)]
struct Settings {
    dotenv_path: PathBuf,
    report_dir: PathBuf,
}

#[derive(Debug, Clone, Copy)]
enum CheckStatus {
    NotRun,
    Passed,
    Failed,
}

impl CheckStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::NotRun => "not_run",
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
struct CaseResult {
    family: &'static str,
    surface: &'static str,
    status: CheckStatus,
    note: String,
}

#[derive(Debug, Clone)]
struct RuntimeConfigSummary {
    aggregator_http_base_url: String,
    aggregator_grpc_base_url: Option<String>,
    aggregator_ws_base_url: Option<String>,
    intro_http_base_url: String,
    primitives_http_base_url: String,
    regime_http_base_url: String,
    bearer_token_present: bool,
}

#[derive(Debug)]
struct Report {
    title: String,
    execution_timestamp_utc: String,
    config: Option<RuntimeConfigSummary>,
    results: Vec<CaseResult>,
    proved_observations: Vec<String>,
    failures: Vec<String>,
    skipped: Vec<String>,
    final_status: String,
}

#[derive(Debug)]
struct RuntimeConfig {
    summary: RuntimeConfigSummary,
    bearer_token: Option<BearerToken>,
    aggregator: Aggregator,
    intro: Intro,
    primitives: Primitives,
    regime: Regime,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn default_dotenv_path() -> PathBuf {
    repo_root().join(".env")
}

fn default_report_dir() -> PathBuf {
    repo_root().join("bin").join(BIN_NAME)
}

fn report_path(report_dir: &Path, timestamp: &str) -> PathBuf {
    report_dir.join(format!("{BIN_NAME}_{timestamp}.md"))
}

fn timestamp_for_filename() -> String {
    Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

fn timestamp_for_report() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn print_help() {
    println!(
        "\
{BIN_NAME}

SDK full public endpoint verification binary.

Usage:
  cargo run --bin {BIN_NAME} -- [--dotenv <PATH>] [--report-dir <PATH>]
  cargo run --bin {BIN_NAME} -- --help

Options:
  --dotenv <PATH>      Optional dotenv file to load before reading env vars.
                       Default: <repo_root>/.env
  --report-dir <PATH>  Directory where the markdown report will be written.
                       Default: <repo_root>/bin/{BIN_NAME}
  --help               Print this help and exit.

Environment:
  {BEARER_ENV}
"
    );
}

fn parse_args() -> Result<Option<Settings>, String> {
    let mut dotenv_path = default_dotenv_path();
    let mut report_dir = default_report_dir();
    let mut it = env::args().skip(1);

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(None);
            }
            "--dotenv" => {
                let value = it
                    .next()
                    .ok_or_else(|| "missing value for --dotenv".to_string())?;
                dotenv_path = PathBuf::from(value);
            }
            "--report-dir" => {
                let value = it
                    .next()
                    .ok_or_else(|| "missing value for --report-dir".to_string())?;
                report_dir = PathBuf::from(value);
            }
            other => {
                return Err(format!("unknown argument `{other}`; use --help for usage"));
            }
        }
    }

    Ok(Some(Settings {
        dotenv_path,
        report_dir,
    }))
}

fn parse_dotenv_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let (key, value) = trimmed.split_once('=')?;
    let key = key.trim();
    if key.is_empty() {
        return None;
    }

    let value = value.trim().trim_matches('"').trim_matches('\'');
    Some((key.to_string(), value.to_string()))
}

fn load_dotenv_if_present(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Ok(());
    }

    let content = fs::read_to_string(path)
        .map_err(|source| format!("failed to read dotenv file {}: {source}", path.display()))?;

    for line in content.lines() {
        if let Some((key, value)) = parse_dotenv_line(line) {
            if env::var_os(&key).is_none() {
                // SAFETY: this binary mutates env only during startup on the main thread.
                unsafe { env::set_var(key, value) };
            }
        }
    }

    Ok(())
}

fn optional_env(name: &'static str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn initial_case_results() -> Vec<CaseResult> {
    [
        ("foundation", "foundation.client_construction"),
        ("foundation", "foundation.report_scaffold"),
        ("intro", "intro.intro"),
        ("aggregator.docs", "aggregator.docs_system"),
        ("aggregator.docs", "aggregator.docs_summary"),
        ("aggregator.docs", "aggregator.docs_themes"),
        ("aggregator.docs", "aggregator.docs_endpoints"),
        ("aggregator.docs", "aggregator.openapi"),
        ("aggregator.discovery", "aggregator.pairs_status"),
        ("aggregator.discovery", "aggregator.pairs_list"),
        ("aggregator.discovery", "aggregator.files_downloads"),
        ("aggregator.discovery", "aggregator.files_download_items"),
        ("aggregator.http", "aggregator.latest"),
        ("aggregator.http", "aggregator.range"),
        ("aggregator.http", "aggregator.search"),
        ("aggregator.http", "aggregator.time_machine"),
        ("aggregator.grpc", "aggregator.latest_grpc"),
        ("aggregator.grpc", "aggregator.range_grpc"),
        ("aggregator.grpc", "aggregator.search_grpc"),
        ("aggregator.grpc", "aggregator.time_machine_grpc"),
        ("aggregator.pagination", "aggregator.range_call"),
        ("aggregator.pagination", "aggregator.range_grpc_call"),
        ("aggregator.pagination", "aggregator.search_call"),
        ("aggregator.pagination", "aggregator.search_grpc_call"),
        ("aggregator.pagination", "aggregator.time_machine_call"),
        ("aggregator.pagination", "aggregator.time_machine_grpc_call"),
        ("aggregator.parity", "aggregator.latest_http_grpc_parity"),
        ("aggregator.ws", "aggregator.connect_bars_ws"),
        (
            "aggregator.ws",
            "aggregator.connect_bars_ws_make_before_break",
        ),
        ("aggregator.ws", "aggregator.connect_bars_ws_recovering"),
        ("aggregator.ws", "aggregator.connect_messages_ws"),
        ("aggregator.ws", "aggregator.connect_messages_ws_recovering"),
        ("primitives.docs", "primitives.docs_system"),
        ("primitives.docs", "primitives.docs_summary"),
        ("primitives.docs", "primitives.docs_taxonomy"),
        ("primitives.docs", "primitives.docs_registry"),
        ("primitives.docs", "primitives.docs_endpoints"),
        ("primitives.docs", "primitives.openapi"),
        ("primitives.discovery", "primitives.pairs_status"),
        ("primitives.discovery", "primitives.pairs_list"),
        ("primitives.discovery", "primitives.files_downloads"),
        ("primitives.discovery", "primitives.files_download_items"),
        ("primitives.http", "primitives.latest"),
        ("primitives.http", "primitives.range"),
        ("primitives.http", "primitives.search"),
        ("primitives.http", "primitives.time_machine"),
        ("primitives.grpc", "primitives.latest_grpc"),
        ("primitives.grpc", "primitives.range_grpc"),
        ("primitives.grpc", "primitives.search_grpc"),
        ("primitives.grpc", "primitives.time_machine_grpc"),
        (
            "primitives.contracts",
            "primitives.projected_http_protobuf_rejection",
        ),
        (
            "primitives.contracts",
            "primitives.projected_grpc_rejection",
        ),
        (
            "primitives.contracts",
            "primitives.projected_outputs_ws_protobuf_rejection",
        ),
        ("primitives.pagination", "primitives.range_call"),
        ("primitives.pagination", "primitives.range_grpc_call"),
        ("primitives.pagination", "primitives.search_call"),
        ("primitives.pagination", "primitives.search_grpc_call"),
        ("primitives.pagination", "primitives.time_machine_call"),
        ("primitives.pagination", "primitives.time_machine_grpc_call"),
        ("primitives.parity", "primitives.latest_http_grpc_parity"),
        ("primitives.ws", "primitives.connect_outputs_ws"),
        (
            "primitives.ws",
            "primitives.connect_outputs_ws_make_before_break",
        ),
        ("primitives.ws", "primitives.connect_outputs_ws_recovering"),
        ("primitives.ws", "primitives.connect_messages_ws"),
        ("primitives.ws", "primitives.connect_messages_ws_recovering"),
        ("regime.docs", "regime.docs_system"),
        ("regime.docs", "regime.docs_summary"),
        ("regime.docs", "regime.docs_taxonomy"),
        ("regime.docs", "regime.docs_registry"),
        ("regime.docs", "regime.docs_endpoints"),
        ("regime.docs", "regime.openapi"),
        ("regime.discovery", "regime.pairs_status"),
        ("regime.discovery", "regime.pairs_list"),
        ("regime.discovery", "regime.files_downloads"),
        ("regime.discovery", "regime.files_download_items"),
        ("regime.http", "regime.latest"),
        ("regime.http", "regime.range"),
        ("regime.http", "regime.search"),
        ("regime.http", "regime.time_machine"),
        ("regime.grpc", "regime.latest_grpc"),
        ("regime.grpc", "regime.range_grpc"),
        ("regime.grpc", "regime.search_grpc"),
        ("regime.grpc", "regime.time_machine_grpc"),
        (
            "regime.contracts",
            "regime.projected_http_protobuf_rejection",
        ),
        ("regime.contracts", "regime.projected_grpc_rejection"),
        ("regime.contracts", "regime.non_h1_http_rejection"),
        ("regime.contracts", "regime.non_h1_grpc_rejection"),
        (
            "regime.contracts",
            "regime.projected_outputs_ws_protobuf_rejection",
        ),
        ("regime.contracts", "regime.non_h1_outputs_ws_rejection"),
        ("regime.pagination", "regime.range_call"),
        ("regime.pagination", "regime.range_grpc_call"),
        ("regime.pagination", "regime.search_call"),
        ("regime.pagination", "regime.search_grpc_call"),
        ("regime.pagination", "regime.time_machine_call"),
        ("regime.pagination", "regime.time_machine_grpc_call"),
        ("regime.parity", "regime.latest_http_grpc_parity"),
        ("regime.ws", "regime.connect_outputs_ws"),
        ("regime.ws", "regime.connect_outputs_ws_make_before_break"),
        ("regime.ws", "regime.connect_outputs_ws_recovering"),
        ("regime.ws", "regime.connect_messages_ws"),
        ("regime.ws", "regime.connect_messages_ws_recovering"),
    ]
    .into_iter()
    .map(|(family, surface)| CaseResult {
        family,
        surface,
        status: CheckStatus::NotRun,
        note: "not executed in the current verification phase".to_string(),
    })
    .collect()
}

fn set_case_status(
    report: &mut Report,
    surface: &'static str,
    status: CheckStatus,
    note: impl Into<String>,
) {
    let note = note.into();
    if let Some(case) = report
        .results
        .iter_mut()
        .find(|case| case.surface == surface)
    {
        case.status = status;
        case.note = note;
    } else {
        report.results.push(CaseResult {
            family: "unplanned",
            surface,
            status,
            note,
        });
    }
}

fn record_pass(report: &mut Report, surface: &'static str, note: impl Into<String>) {
    let note = note.into();
    set_case_status(report, surface, CheckStatus::Passed, note.clone());
    report
        .proved_observations
        .push(format!("`{surface}` passed: {note}"));
}

fn record_fail(report: &mut Report, surface: &'static str, error: impl Into<String>) {
    let error = error.into();
    set_case_status(report, surface, CheckStatus::Failed, error.clone());
    report.failures.push(format!("`{surface}` failed: {error}"));
}

fn markdown_report(report: &Report) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", report.title));
    out.push_str(&format!(
        "- execution_timestamp_utc: `{}`\n",
        report.execution_timestamp_utc
    ));
    out.push_str(&format!("- final_status: `{}`\n\n", report.final_status));

    out.push_str("## Configuration Summary\n\n");
    match &report.config {
        Some(config) => {
            out.push_str(&format!(
                "- aggregator_http_base_url: `{}`\n",
                config.aggregator_http_base_url
            ));
            out.push_str(&format!(
                "- aggregator_grpc_base_url: `{}`\n",
                config
                    .aggregator_grpc_base_url
                    .as_deref()
                    .unwrap_or("not_configured")
            ));
            out.push_str(&format!(
                "- aggregator_ws_base_url: `{}`\n",
                config
                    .aggregator_ws_base_url
                    .as_deref()
                    .unwrap_or("not_configured")
            ));
            out.push_str(&format!(
                "- intro_http_base_url: `{}`\n",
                config.intro_http_base_url
            ));
            out.push_str(&format!(
                "- primitives_http_base_url: `{}`\n",
                config.primitives_http_base_url
            ));
            out.push_str(&format!(
                "- regime_http_base_url: `{}`\n",
                config.regime_http_base_url
            ));
            out.push_str(&format!(
                "- bearer_token_present: `{}`\n\n",
                config.bearer_token_present
            ));
        }
        None => out.push_str("- configuration was not established\n\n"),
    }

    out.push_str("## Surface Results\n\n");
    out.push_str("| Family | Surface | Status | Note |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    for result in &report.results {
        let note = result.note.replace('\n', " ");
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} |\n",
            result.family,
            result.surface,
            result.status.as_str(),
            note
        ));
    }
    out.push('\n');

    out.push_str("## Proved Observations\n\n");
    if report.proved_observations.is_empty() {
        out.push_str("- none recorded\n\n");
    } else {
        for observation in &report.proved_observations {
            out.push_str(&format!("- {}\n", observation));
        }
        out.push('\n');
    }

    out.push_str("## Failures\n\n");
    if report.failures.is_empty() {
        out.push_str("- none\n\n");
    } else {
        for failure in &report.failures {
            out.push_str(&format!("- {}\n", failure));
        }
        out.push('\n');
    }

    out.push_str("## Skipped Checks\n\n");
    if report.skipped.is_empty() {
        out.push_str("- none\n");
    } else {
        for skipped in &report.skipped {
            out.push_str(&format!("- {}\n", skipped));
        }
    }

    out
}

fn write_report(report: &Report, settings: &Settings, timestamp: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(&settings.report_dir).map_err(|source| {
        format!(
            "failed to create report directory {}: {source}",
            settings.report_dir.display()
        )
    })?;
    let path = report_path(&settings.report_dir, timestamp);
    fs::write(&path, markdown_report(report))
        .map_err(|source| format!("failed to write report {}: {source}", path.display()))?;
    Ok(path)
}

fn build_runtime_config() -> Result<RuntimeConfig, SdkError> {
    let bearer_token = optional_env(BEARER_ENV).map(BearerToken::new).transpose()?;
    let bearer_token_present = bearer_token.is_some();

    let aggregator = Aggregator::client(bearer_token.clone())?;
    let intro = Intro::new(IntroConfig::mathilde_public_default(bearer_token.clone())?)?;
    let primitives = Primitives::client(bearer_token.clone())?;
    let regime = Regime::client(bearer_token.clone())?;

    Ok(RuntimeConfig {
        summary: RuntimeConfigSummary {
            aggregator_http_base_url: MathildePublicHosts::AGGREGATOR_HTTP.to_string(),
            aggregator_grpc_base_url: Some(MathildePublicHosts::AGGREGATOR_GRPC.to_string()),
            aggregator_ws_base_url: Some(
                MathildePublicHosts::AGGREGATOR_HTTP.replacen("https://", "wss://", 1),
            ),
            intro_http_base_url: MathildePublicHosts::INTRO.to_string(),
            primitives_http_base_url: MathildePublicHosts::PRIMITIVES_HTTP.to_string(),
            regime_http_base_url: MathildePublicHosts::REGIME_HTTP.to_string(),
            bearer_token_present,
        },
        bearer_token,
        aggregator,
        intro,
        primitives,
        regime,
    })
}

fn pair_from_latest_response(out: &LatestResponse) -> Option<&str> {
    out.rows.first().map(|row| row.pair.as_str())
}

fn close_end_from_latest_response(out: &LatestResponse) -> i64 {
    out.close_end_ms
}

fn range_rows_len(out: &RangeResponse) -> usize {
    out.rows.len()
}

fn search_hits_len(out: &SearchResponse) -> usize {
    out.hits.len()
}

fn time_machine_rows_len(out: &TimeMachineResponse) -> usize {
    out.rows.len()
}

fn regime_output_kind(mode: LocalOutputMode) -> &'static str {
    mode.label()
}

fn json_str<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    value.get(key)?.as_str()
}

fn json_array_len(value: &serde_json::Value, key: &str) -> Option<usize> {
    value.get(key)?.as_array().map(|rows| rows.len())
}

fn ws_timeout() -> Duration {
    Duration::from_secs(15)
}

fn ws_replay_timeout() -> Duration {
    Duration::from_secs(60)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalOutputMode {
    Min,
    WithMeta,
    ProjectedMin,
    ProjectedWithMeta,
}

impl LocalOutputMode {
    fn label(self) -> &'static str {
        match self {
            Self::Min => "min",
            Self::WithMeta => "with_meta",
            Self::ProjectedMin => "projected_min",
            Self::ProjectedWithMeta => "projected_with_meta",
        }
    }

    fn view(self) -> &'static str {
        match self {
            Self::Min | Self::ProjectedMin => "min",
            Self::WithMeta | Self::ProjectedWithMeta => "full",
        }
    }
}

fn json_value<T: serde::Serialize>(value: &T, context: &str) -> Result<Value, String> {
    serde_json::to_value(value).map_err(|error| format!("{context} serialization failed: {error}"))
}

fn json_string<T: serde::Serialize>(value: &T, context: &str) -> Result<String, String> {
    match json_value(value, context)? {
        Value::String(value) => Ok(value),
        other => Err(format!(
            "{context} serialized to non-string JSON value: {}",
            other
        )),
    }
}

fn json_string_list<T: serde::Serialize>(
    values: &[T],
    context: &str,
) -> Result<Vec<String>, String> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| json_string(value, &format!("{context}[{index}]")))
        .collect()
}

fn normalize_required_pairs(values: &[String], context: &str) -> Result<Vec<String>, String> {
    let normalized = values
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        return Err(format!("{context} requires at least one pair"));
    }
    Ok(normalized)
}

fn normalize_optional_pairs(values: Option<&[String]>) -> Option<Vec<String>> {
    let normalized = values
        .unwrap_or(&[])
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn normalize_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn normalize_required_string(value: &str, context: &str) -> Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(format!("{context} must not be empty"));
    }
    Ok(normalized.to_string())
}

fn primitive_mode_from_selectors_and_metadata(
    family: Option<&[PrimitiveProcessorFamily]>,
    group: Option<&[PrimitiveProcessorGroup]>,
    metadata: Option<bool>,
) -> LocalOutputMode {
    let projected = family.is_some_and(|values| !values.is_empty())
        || group.is_some_and(|values| !values.is_empty());
    match (projected, metadata.unwrap_or(false)) {
        (false, false) => LocalOutputMode::Min,
        (false, true) => LocalOutputMode::WithMeta,
        (true, false) => LocalOutputMode::ProjectedMin,
        (true, true) => LocalOutputMode::ProjectedWithMeta,
    }
}

fn regime_mode_from_selectors_secondary_and_metadata(
    family: Option<&[regime_system::ProcessorFamily]>,
    group: Option<&[regime_system::ProcessorGroup]>,
    secondary: Option<bool>,
    metadata: Option<bool>,
) -> LocalOutputMode {
    let projected = family.is_some_and(|values| !values.is_empty())
        || group.is_some_and(|values| !values.is_empty())
        || !secondary.unwrap_or(false);
    match (projected, metadata.unwrap_or(false)) {
        (false, false) => LocalOutputMode::Min,
        (false, true) => LocalOutputMode::WithMeta,
        (true, false) => LocalOutputMode::ProjectedMin,
        (true, true) => LocalOutputMode::ProjectedWithMeta,
    }
}

fn compare_semantic_values(
    surface: &'static str,
    sdk: &Value,
    direct: &Value,
) -> Result<(), String> {
    let mut sdk = sdk.clone();
    let mut direct = direct.clone();
    normalize_semantic_value(&mut sdk);
    normalize_semantic_value(&mut direct);

    if sdk == direct {
        return Ok(());
    }

    let sdk_pretty = serde_json::to_string_pretty(&sdk)
        .unwrap_or_else(|_| sdk.to_string())
        .chars()
        .take(2_000)
        .collect::<String>();
    let direct_pretty = serde_json::to_string_pretty(&direct)
        .unwrap_or_else(|_| direct.to_string())
        .chars()
        .take(2_000)
        .collect::<String>();
    Err(format!(
        "{surface} semantic mismatch\nsdk={sdk_pretty}\ndirect={direct_pretty}"
    ))
}

async fn raw_http_post_json(
    base_url: &str,
    path: &str,
    bearer_token: Option<&BearerToken>,
    body: &Value,
) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let mut request = client.post(format!("{base_url}{path}")).json(body);
    if let Some(token) = bearer_token {
        request = request.bearer_auth(token.as_str());
    }

    let response = request
        .send()
        .await
        .map_err(|error| format!("raw HTTP request failed for {base_url}{path}: {error}"))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|error| format!("raw HTTP response read failed for {base_url}{path}: {error}"))?;
    if !status.is_success() {
        return Err(format!(
            "raw HTTP request failed for {base_url}{path}: status={} body={text}",
            status.as_u16()
        ));
    }

    serde_json::from_str(&text).map_err(|error| {
        format!("raw HTTP JSON decode failed for {base_url}{path}: {error}; body={text}")
    })
}

async fn raw_grpc_unary<Req, Resp>(
    endpoint_uri: &str,
    path: &'static str,
    bearer_token: Option<&BearerToken>,
    request_body: Req,
) -> Result<Resp, String>
where
    Req: prost::Message + Default + 'static,
    Resp: prost::Message + Default + 'static,
{
    let endpoint = Endpoint::new(endpoint_uri.to_string())
        .map_err(|error| format!("raw gRPC endpoint parse failed for {endpoint_uri}: {error}"))?;
    let channel = endpoint
        .connect()
        .await
        .map_err(|error| format!("raw gRPC connect failed for {endpoint_uri}: {error}"))?;
    let mut grpc = Grpc::new(channel);
    grpc.ready()
        .await
        .map_err(|error| format!("raw gRPC readiness failed for {endpoint_uri}: {error}"))?;

    let mut request = tonic::Request::new(request_body);
    if let Some(token) = bearer_token {
        let metadata = MetadataValue::try_from(format!("Bearer {}", token.as_str()))
            .map_err(|error| format!("raw gRPC authorization metadata build failed: {error}"))?;
        request.metadata_mut().insert("authorization", metadata);
    }

    let path_display = path.to_string();
    let path = PathAndQuery::from_static(path);
    grpc.unary(request, path, ProstCodec::default())
        .await
        .map(|response| response.into_inner())
        .map_err(|error| format!("raw gRPC unary failed for {endpoint_uri}{path_display}: {error}"))
}

fn canonical_output_wrapper(payload: Value, mode: LocalOutputMode) -> Value {
    json!({
        "mode": mode.label(),
        "payload": payload,
    })
}

fn canonical_primitive_row_sdk(
    row: &primitives_system::OutputRow,
    mode: LocalOutputMode,
) -> Result<Value, String> {
    let mut payload = json_value(row, "primitive output row")?;
    let object = payload
        .as_object_mut()
        .ok_or_else(|| "primitive output row did not serialize as an object".to_string())?;
    let computed = object
        .remove("computed")
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
    let computed = computed
        .as_object()
        .ok_or_else(|| "primitive output row computed bag was not an object".to_string())?;
    for (key, value) in computed {
        object.insert(key.clone(), value.clone());
    }
    Ok(canonical_output_wrapper(payload, mode))
}

fn canonical_regime_row_sdk(
    row: &regime_system::OutputRow,
    mode: LocalOutputMode,
) -> Result<Value, String> {
    let mut payload = json_value(row, "regime output row")?;
    let object = payload
        .as_object_mut()
        .ok_or_else(|| "regime output row did not serialize as an object".to_string())?;
    let computed = object
        .remove("computed")
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
    let computed = computed
        .as_object()
        .ok_or_else(|| "regime output row computed bag was not an object".to_string())?;
    for (key, value) in computed {
        object.insert(key.clone(), value.clone());
    }
    Ok(canonical_output_wrapper(payload, mode))
}

fn strip_volatile_fields(value: &mut Value) {
    match value {
        Value::Object(object) => {
            object.remove("age_ms");
            for nested in object.values_mut() {
                strip_volatile_fields(nested);
            }
        }
        Value::Array(items) => {
            for item in items {
                strip_volatile_fields(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn strip_null_fields(value: &mut Value) {
    match value {
        Value::Object(object) => {
            object.retain(|_, nested| {
                strip_null_fields(nested);
                !nested.is_null()
            });
        }
        Value::Array(items) => {
            for item in items {
                strip_null_fields(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn strip_empty_diagnostics_arrays(value: &mut Value) {
    match value {
        Value::Object(object) => {
            object.retain(|key, nested| {
                strip_empty_diagnostics_arrays(nested);
                !(key == "diagnostics" && matches!(nested, Value::Array(items) if items.is_empty()))
            });
        }
        Value::Array(items) => {
            for item in items {
                strip_empty_diagnostics_arrays(item);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn normalize_semantic_value(value: &mut Value) {
    strip_volatile_fields(value);
    strip_null_fields(value);
    strip_empty_diagnostics_arrays(value);
}

fn canonical_compute_view(raw_view: &Value, mode: LocalOutputMode) -> Result<Value, String> {
    match raw_view {
        Value::String(value) => Ok(Value::String(value.clone())),
        Value::Number(value) if value.as_i64() == Some(1) => Ok(Value::String("min".to_string())),
        Value::Number(value) if value.as_i64() == Some(2) => Ok(Value::String("full".to_string())),
        Value::Null => Ok(Value::String(mode.view().to_string())),
        other => Err(format!("unsupported compute view value: {other}")),
    }
}

fn canonical_compute_latest_raw(mut value: Value, mode: LocalOutputMode) -> Result<Value, String> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| "compute latest raw payload was not a JSON object".to_string())?;
    let raw_view = object.get("view").cloned().unwrap_or(Value::Null);
    let rows = object
        .remove("rows")
        .ok_or_else(|| "compute latest raw payload missing rows".to_string())?;
    let rows = rows
        .as_array()
        .ok_or_else(|| "compute latest raw rows were not an array".to_string())?;

    let normalized_rows = rows
        .iter()
        .map(|row| {
            let row = row
                .as_object()
                .ok_or_else(|| "compute latest raw row was not an object".to_string())?;
            let payload = row.get("output").cloned().unwrap_or_else(|| {
                let mut payload = row.clone();
                payload.remove("age_ms");
                Value::Object(payload)
            });
            Ok(json!({
                "output": canonical_output_wrapper(payload, mode),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(json!({
        "watermark_end_ms": object.get("watermark_end_ms").cloned().ok_or_else(|| "compute latest raw payload missing watermark_end_ms".to_string())?,
        "close_end_ms": object.get("close_end_ms").cloned().ok_or_else(|| "compute latest raw payload missing close_end_ms".to_string())?,
        "latest_mode": object.get("latest_mode").cloned().ok_or_else(|| "compute latest raw payload missing latest_mode".to_string())?,
        "view": canonical_compute_view(&raw_view, mode)?,
        "rows": normalized_rows,
        "missing_pairs": object.get("missing_pairs").cloned().unwrap_or_else(|| json!([])),
    }))
}

fn canonical_compute_rows_raw(rows: &[Value], mode: LocalOutputMode) -> Value {
    Value::Array(
        rows.iter()
            .cloned()
            .map(|row| canonical_output_wrapper(row, mode))
            .collect(),
    )
}

fn canonical_compute_range_raw(mut value: Value, mode: LocalOutputMode) -> Result<Value, String> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| "compute range raw payload was not a JSON object".to_string())?;
    let rows = object
        .remove("rows")
        .ok_or_else(|| "compute range raw payload missing rows".to_string())?;
    let rows = rows
        .as_array()
        .ok_or_else(|| "compute range raw rows were not an array".to_string())?;
    Ok(json!({
        "rows": canonical_compute_rows_raw(rows, mode),
        "close_end_ms": object.get("close_end_ms").cloned().ok_or_else(|| "compute range raw payload missing close_end_ms".to_string())?,
        "next_cursor": object.get("next_cursor").cloned().unwrap_or(Value::Null),
    }))
}

fn canonical_compute_search_raw(mut value: Value, mode: LocalOutputMode) -> Result<Value, String> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| "compute search raw payload was not a JSON object".to_string())?;
    let evaluated_rows = object.remove("evaluated_rows").unwrap_or(Value::Null);
    let evaluated_rows = match evaluated_rows {
        Value::Null => Value::Null,
        Value::Array(rows) => canonical_compute_rows_raw(&rows, mode),
        other => {
            return Err(format!(
                "compute search raw evaluated_rows had unsupported shape: {other}"
            ));
        }
    };

    Ok(json!({
        "hits": object.get("hits").cloned().unwrap_or_else(|| json!([])),
        "evaluated_rows": evaluated_rows,
        "next_cursor": object.get("next_cursor").cloned().unwrap_or(Value::Null),
        "done": object.get("done").cloned().unwrap_or(Value::Bool(false)),
        "returned_hits": object.get("returned_hits").cloned().unwrap_or(Value::Null),
        "effective_hits_limit": object.get("effective_hits_limit").cloned().unwrap_or(Value::Null),
        "truncated": object.get("truncated").cloned().unwrap_or(Value::Bool(false)),
        "predicate_pairs": object.get("predicate_pairs").cloned().unwrap_or_else(|| json!([])),
        "predicate_normalized": object.get("predicate_normalized").cloned().unwrap_or(Value::Null),
    }))
}

fn canonical_compute_time_machine_raw(
    mut value: Value,
    mode: LocalOutputMode,
) -> Result<Value, String> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| "compute time-machine raw payload was not a JSON object".to_string())?;
    let rows = object
        .remove("rows")
        .ok_or_else(|| "compute time-machine raw payload missing rows".to_string())?;
    let rows = rows
        .as_array()
        .ok_or_else(|| "compute time-machine raw rows were not an array".to_string())?;
    let rows = rows
        .iter()
        .map(|row| {
            let row = row
                .as_object()
                .ok_or_else(|| "compute time-machine raw row was not an object".to_string())?;
            let output = row
                .get("output")
                .cloned()
                .ok_or_else(|| "compute time-machine raw row missing output".to_string())?;
            Ok(json!({
                "hit_close_ms": row.get("hit_close_ms").cloned().ok_or_else(|| "compute time-machine raw row missing hit_close_ms".to_string())?,
                "offset": row.get("offset").cloned().ok_or_else(|| "compute time-machine raw row missing offset".to_string())?,
                "output": canonical_output_wrapper(output, mode),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(json!({
        "rows": rows,
        "next_cursor": object.get("next_cursor").cloned().unwrap_or(Value::Null),
        "done": object.get("done").cloned().unwrap_or(Value::Bool(false)),
        "returned_hits": object.get("returned_hits").cloned().unwrap_or(Value::Null),
        "effective_hits_limit": object.get("effective_hits_limit").cloned().unwrap_or(Value::Null),
        "truncated": object.get("truncated").cloned().unwrap_or(Value::Bool(false)),
        "predicate_pairs": object.get("predicate_pairs").cloned().unwrap_or_else(|| json!([])),
        "predicate_normalized": object.get("predicate_normalized").cloned().unwrap_or(Value::Null),
    }))
}

fn mode_has_metadata(mode: LocalOutputMode) -> bool {
    matches!(
        mode,
        LocalOutputMode::WithMeta | LocalOutputMode::ProjectedWithMeta
    )
}

fn canonical_primitive_latest_grpc_raw(
    value: &primitives_proto::OutputsLatestResponseV1,
    mode: LocalOutputMode,
    diagnostics_enabled: bool,
) -> Result<Value, String> {
    let value = primitives_system::LatestResponse::from_grpc_proto(
        value.clone(),
        mode_has_metadata(mode),
        diagnostics_enabled,
    )
    .map_err(|error| error.to_string())?;
    canonical_primitive_latest_sdk(&value, mode)
}

fn canonical_primitive_range_grpc_raw(
    value: &primitives_proto::OutputsRangeResponseV1,
    mode: LocalOutputMode,
    diagnostics_enabled: bool,
) -> Result<Value, String> {
    let value = primitives_system::RangeResponse::from_grpc_proto(
        value.clone(),
        mode_has_metadata(mode),
        diagnostics_enabled,
    )
    .map_err(|error| error.to_string())?;
    canonical_primitive_range_sdk(&value, mode)
}

fn canonical_primitive_search_grpc_raw(
    value: &primitives_proto::OutputsSearchResponseV1,
    mode: LocalOutputMode,
    diagnostics_enabled: bool,
    evaluated_rows_enabled: bool,
) -> Result<Value, String> {
    let value = primitives_system::SearchResponse::from_grpc_proto(
        value.clone(),
        mode_has_metadata(mode),
        diagnostics_enabled,
        evaluated_rows_enabled,
    )
    .map_err(|error| error.to_string())?;
    canonical_primitive_search_sdk(&value, mode)
}

fn canonical_primitive_time_machine_grpc_raw(
    value: &primitives_proto::OutputsTimeMachineResponseV1,
    mode: LocalOutputMode,
    diagnostics_enabled: bool,
) -> Result<Value, String> {
    let value = primitives_system::TimeMachineResponse::from_grpc_proto(
        value.clone(),
        mode_has_metadata(mode),
        diagnostics_enabled,
    )
    .map_err(|error| error.to_string())?;
    canonical_primitive_time_machine_sdk(&value, mode)
}

fn canonical_regime_latest_grpc_raw(
    value: &regime_proto::OutputsLatestResponseV1,
    mode: LocalOutputMode,
    diagnostics_enabled: bool,
) -> Result<Value, String> {
    let value = regime_system::LatestResponse::from_grpc_proto(
        value.clone(),
        mode_has_metadata(mode),
        diagnostics_enabled,
    )
    .map_err(|error| error.to_string())?;
    canonical_regime_latest_sdk(&value, mode)
}

fn canonical_regime_range_grpc_raw(
    value: &regime_proto::OutputsRangeResponseV1,
    mode: LocalOutputMode,
    diagnostics_enabled: bool,
) -> Result<Value, String> {
    let value = regime_system::RangeResponse::from_grpc_proto(
        value.clone(),
        mode_has_metadata(mode),
        diagnostics_enabled,
    )
    .map_err(|error| error.to_string())?;
    canonical_regime_range_sdk(&value, mode)
}

fn canonical_regime_search_grpc_raw(
    value: &regime_proto::OutputsSearchResponseV1,
    mode: LocalOutputMode,
    diagnostics_enabled: bool,
    evaluated_rows_enabled: bool,
) -> Result<Value, String> {
    let value = regime_system::SearchResponse::from_grpc_proto(
        value.clone(),
        mode_has_metadata(mode),
        diagnostics_enabled,
        evaluated_rows_enabled,
    )
    .map_err(|error| error.to_string())?;
    canonical_regime_search_sdk(&value, mode)
}

fn canonical_regime_time_machine_grpc_raw(
    value: &regime_proto::OutputsTimeMachineResponseV1,
    mode: LocalOutputMode,
    diagnostics_enabled: bool,
) -> Result<Value, String> {
    let value = regime_system::TimeMachineResponse::from_grpc_proto(
        value.clone(),
        mode_has_metadata(mode),
        diagnostics_enabled,
    )
    .map_err(|error| error.to_string())?;
    canonical_regime_time_machine_sdk(&value, mode)
}

fn canonical_primitive_latest_sdk(
    value: &primitives_system::LatestResponse,
    mode: LocalOutputMode,
) -> Result<Value, String> {
    Ok(json!({
        "watermark_end_ms": value.watermark_end_ms,
        "close_end_ms": value.close_end_ms,
        "latest_mode": json_value(&value.latest_mode, "primitive latest_mode")?,
        "view": json_value(&value.view, "primitive output view")?,
        "rows": value.rows.iter().map(|row| {
            Ok(json!({
                "output": canonical_primitive_row_sdk(&row.row, mode)?,
            }))
        }).collect::<Result<Vec<_>, String>>()?,
        "missing_pairs": value.missing_pairs,
    }))
}

fn canonical_regime_latest_sdk(
    value: &regime_system::LatestResponse,
    mode: LocalOutputMode,
) -> Result<Value, String> {
    Ok(json!({
        "watermark_end_ms": value.watermark_end_ms,
        "close_end_ms": value.close_end_ms,
        "latest_mode": json_value(&value.latest_mode, "regime latest_mode")?,
        "view": json_value(&value.view, "regime output view")?,
        "rows": value.rows.iter().map(|row| {
            Ok(json!({
                "output": canonical_regime_row_sdk(&row.row, mode)?,
            }))
        }).collect::<Result<Vec<_>, String>>()?,
        "missing_pairs": value.missing_pairs,
    }))
}

fn canonical_primitive_range_sdk(
    value: &primitives_system::RangeResponse,
    mode: LocalOutputMode,
) -> Result<Value, String> {
    Ok(json!({
        "rows": value.rows.iter().map(|row| canonical_primitive_row_sdk(row, mode)).collect::<Result<Vec<_>, String>>()?,
        "close_end_ms": value.close_end_ms,
        "next_cursor": value.next_cursor,
    }))
}

fn canonical_regime_range_sdk(
    value: &regime_system::RangeResponse,
    mode: LocalOutputMode,
) -> Result<Value, String> {
    Ok(json!({
        "rows": value.rows.iter().map(|row| canonical_regime_row_sdk(row, mode)).collect::<Result<Vec<_>, String>>()?,
        "close_end_ms": value.close_end_ms,
        "next_cursor": value.next_cursor,
    }))
}

fn canonical_primitive_search_sdk(
    value: &primitives_system::SearchResponse,
    mode: LocalOutputMode,
) -> Result<Value, String> {
    Ok(json!({
        "hits": value.hits,
        "evaluated_rows": value.evaluated_rows.as_ref().map(|rows| rows.iter().map(|row| canonical_primitive_row_sdk(row, mode)).collect::<Result<Vec<_>, String>>()).transpose()?,
        "next_cursor": value.next_cursor,
        "done": value.done,
        "returned_hits": value.returned_hits,
        "effective_hits_limit": value.effective_hits_limit,
        "truncated": value.truncated,
        "predicate_pairs": value.predicate_pairs,
        "predicate_normalized": value.predicate_normalized,
    }))
}

fn canonical_regime_search_sdk(
    value: &regime_system::SearchResponse,
    mode: LocalOutputMode,
) -> Result<Value, String> {
    Ok(json!({
        "hits": value.hits,
        "evaluated_rows": value.evaluated_rows.as_ref().map(|rows| rows.iter().map(|row| canonical_regime_row_sdk(row, mode)).collect::<Result<Vec<_>, String>>()).transpose()?,
        "next_cursor": value.next_cursor,
        "done": value.done,
        "returned_hits": value.returned_hits,
        "effective_hits_limit": value.effective_hits_limit,
        "truncated": value.truncated,
        "predicate_pairs": value.predicate_pairs,
        "predicate_normalized": value.predicate_normalized,
    }))
}

fn canonical_primitive_time_machine_sdk(
    value: &primitives_system::TimeMachineResponse,
    mode: LocalOutputMode,
) -> Result<Value, String> {
    Ok(json!({
        "rows": value.rows.iter().map(|row| {
            Ok(json!({
                "hit_close_ms": row.hit_close_ms,
                "offset": row.offset,
                "output": canonical_primitive_row_sdk(&row.row, mode)?,
            }))
        }).collect::<Result<Vec<_>, String>>()?,
        "next_cursor": value.next_cursor,
        "done": value.done,
        "returned_hits": value.returned_hits,
        "effective_hits_limit": value.effective_hits_limit,
        "truncated": value.truncated,
        "predicate_pairs": value.predicate_pairs,
        "predicate_normalized": value.predicate_normalized,
    }))
}

fn canonical_regime_time_machine_sdk(
    value: &regime_system::TimeMachineResponse,
    mode: LocalOutputMode,
) -> Result<Value, String> {
    Ok(json!({
        "rows": value.rows.iter().map(|row| {
            Ok(json!({
                "hit_close_ms": row.hit_close_ms,
                "offset": row.offset,
                "output": canonical_regime_row_sdk(&row.row, mode)?,
            }))
        }).collect::<Result<Vec<_>, String>>()?,
        "next_cursor": value.next_cursor,
        "done": value.done,
        "returned_hits": value.returned_hits,
        "effective_hits_limit": value.effective_hits_limit,
        "truncated": value.truncated,
        "predicate_pairs": value.predicate_pairs,
        "predicate_normalized": value.predicate_normalized,
    }))
}

fn canonical_aggregator_latest_grpc_raw(
    value: &aggregator_proto::BarsLatestResponseV1,
) -> Result<Value, String> {
    let value = LatestResponse::from_proto(value.clone()).map_err(|error| error.to_string())?;
    canonical_aggregator_latest_sdk(&value)
}

fn canonical_aggregator_range_grpc_raw(
    value: &aggregator_proto::BarsRangeResponseV1,
    metadata: bool,
) -> Result<Value, String> {
    let value =
        RangeResponse::from_proto(value.clone(), metadata).map_err(|error| error.to_string())?;
    canonical_aggregator_range_sdk(&value)
}

fn canonical_aggregator_search_grpc_raw(
    value: &aggregator_proto::BarsSearchResponseV1,
    metadata: bool,
) -> Result<Value, String> {
    let value =
        SearchResponse::from_proto(value.clone(), metadata).map_err(|error| error.to_string())?;
    canonical_aggregator_search_sdk(&value)
}

fn canonical_aggregator_time_machine_grpc_raw(
    value: &aggregator_proto::BarsTimeMachineResponseV1,
    metadata: bool,
) -> Result<Value, String> {
    let value = TimeMachineResponse::from_proto(value.clone(), metadata)
        .map_err(|error| error.to_string())?;
    canonical_aggregator_time_machine_sdk(&value)
}

fn canonical_aggregator_latest_sdk(value: &LatestResponse) -> Result<Value, String> {
    let mut value = json_value(value, "aggregator latest sdk")?;
    strip_volatile_fields(&mut value);
    Ok(value)
}

fn canonical_aggregator_latest_http_raw(mut value: Value) -> Value {
    strip_volatile_fields(&mut value);
    value
}

fn canonical_aggregator_range_sdk(value: &RangeResponse) -> Result<Value, String> {
    json_value(value, "aggregator range sdk")
}

fn canonical_aggregator_search_sdk(value: &SearchResponse) -> Result<Value, String> {
    json_value(value, "aggregator search sdk")
}

fn canonical_aggregator_time_machine_sdk(value: &TimeMachineResponse) -> Result<Value, String> {
    json_value(value, "aggregator time-machine sdk")
}

fn aggregator_latest_http_body(request: &LatestRequest) -> Result<Value, String> {
    Ok(json!({
        "pairs": normalize_required_pairs(&request.pairs, "aggregator latest bars")?,
        "tf": request.tf,
        "latest_mode": request.latest_mode,
        "metadata": request.metadata,
        "format": request.format,
    }))
}

fn aggregator_latest_grpc_proto(
    request: &LatestGrpcRequest,
) -> Result<aggregator_proto::LatestBarsRequestV1, String> {
    Ok(aggregator_proto::LatestBarsRequestV1 {
        pairs: normalize_required_pairs(&request.pairs, "aggregator latest bars gRPC")?,
        tf: json_string(&request.tf, "aggregator latest gRPC tf")?,
        latest_mode: json_string(&request.latest_mode, "aggregator latest gRPC latest_mode")?,
        metadata: request.metadata.unwrap_or(false),
    })
}

fn aggregator_range_http_body(request: &RangeRequest) -> Result<Value, String> {
    Ok(json!({
        "pairs": normalize_required_pairs(&request.pairs, "aggregator range bars")?,
        "tf": request.tf,
        "align_mode": request.align_mode,
        "close_start_ms": request.close_start.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "cursor": request.cursor,
        "close_end_ms": request.close_end.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "limit": request.limit,
        "metadata": request.metadata,
        "format": request.format,
    }))
}

fn aggregator_range_grpc_proto(
    request: &RangeGrpcRequest,
) -> Result<aggregator_proto::RangeBarsRequestV1, String> {
    Ok(aggregator_proto::RangeBarsRequestV1 {
        pairs: normalize_required_pairs(&request.pairs, "aggregator range bars gRPC")?,
        tf: json_string(&request.tf, "aggregator range gRPC tf")?,
        close_end_ms: request
            .close_end
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        cursor: request.cursor.clone(),
        limit: request.limit,
        metadata: request.metadata.unwrap_or(false),
        close_start_ms: request
            .close_start
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        align_mode: request
            .align_mode
            .as_ref()
            .map(|value| json_string(value, "aggregator range gRPC align_mode"))
            .transpose()?,
    })
}

fn aggregator_search_http_body(request: &SearchRequest) -> Result<Value, String> {
    Ok(json!({
        "tf": request.tf,
        "close_start_ms": request.close_start.to_utc_ms().map_err(|error| error.to_string())?,
        "close_end_ms": request.close_end.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "cursor": request.cursor,
        "predicate": normalize_required_string(&request.predicate, "aggregator search predicate")?,
        "evaluate_pair": normalize_optional_string(request.evaluate_pair.as_deref()),
        "metadata": request.metadata,
        "max_hits": request.max_hits,
        "format": request.format,
    }))
}

fn aggregator_search_grpc_proto(
    request: &SearchGrpcRequest,
) -> Result<aggregator_proto::SearchBarsRequestV1, String> {
    Ok(aggregator_proto::SearchBarsRequestV1 {
        tf: json_string(&request.tf, "aggregator search gRPC tf")?,
        close_start_ms: request
            .close_start
            .to_utc_ms()
            .map_err(|error| error.to_string())?,
        close_end_ms: request
            .close_end
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        cursor: request.cursor.clone(),
        predicate: normalize_required_string(
            &request.predicate,
            "aggregator search gRPC predicate",
        )?,
        evaluate_pair: normalize_optional_string(request.evaluate_pair.as_deref()),
        metadata: request.metadata.unwrap_or(false),
        max_hits: request.max_hits,
    })
}

fn aggregator_time_machine_http_body(request: &TimeMachineRequest) -> Result<Value, String> {
    Ok(json!({
        "tf": request.tf,
        "close_start_ms": request.close_start.to_utc_ms().map_err(|error| error.to_string())?,
        "close_end_ms": request.close_end.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "cursor": request.cursor,
        "predicate": request.predicate.as_deref().map(|value| normalize_required_string(value, "aggregator time-machine predicate")).transpose()?,
        "hits": request.hits,
        "output_pairs": normalize_optional_pairs(request.output_pairs.as_deref()),
        "metadata": request.metadata,
        "before_bars": request.before_bars,
        "after_bars": request.after_bars,
        "max_hits": request.max_hits,
        "overlap_mode": normalize_optional_string(request.overlap_mode.as_deref()),
        "format": request.format,
    }))
}

fn aggregator_time_machine_grpc_proto(
    request: &TimeMachineGrpcRequest,
) -> Result<aggregator_proto::TimeMachineBarsRequestV1, String> {
    Ok(aggregator_proto::TimeMachineBarsRequestV1 {
        tf: json_string(&request.tf, "aggregator time-machine gRPC tf")?,
        close_start_ms: request
            .close_start
            .to_utc_ms()
            .map_err(|error| error.to_string())?,
        close_end_ms: request
            .close_end
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        cursor: request.cursor.clone(),
        predicate: request
            .predicate
            .as_deref()
            .map(|value| normalize_required_string(value, "aggregator time-machine gRPC predicate"))
            .transpose()?,
        hits: request.hits.clone().unwrap_or_default(),
        output_pairs: normalize_optional_pairs(request.output_pairs.as_deref()).unwrap_or_default(),
        metadata: request.metadata.unwrap_or(false),
        before_bars: request.before_bars,
        after_bars: request.after_bars,
        max_hits: request.max_hits,
        overlap_mode: normalize_optional_string(request.overlap_mode.as_deref()),
    })
}

fn primitives_latest_http_body(
    request: &primitives_system::LatestRequest,
) -> Result<Value, String> {
    Ok(json!({
        "pairs": normalize_required_pairs(&request.pairs, "primitives latest outputs")?,
        "tf": request.tf,
        "latest_mode": request.latest_mode,
        "family": request.family,
        "group": request.group,
        "metadata": request.metadata,
        "diagnostics": request.diagnostics,
        "format": request.format,
    }))
}

fn primitives_latest_grpc_proto(
    request: &primitives_system::LatestGrpcRequest,
) -> Result<primitives_proto::LatestOutputsRequestV1, String> {
    Ok(primitives_proto::LatestOutputsRequestV1 {
        pairs: normalize_required_pairs(&request.pairs, "primitives latest outputs gRPC")?,
        tf: json_string(&request.tf, "primitives latest gRPC tf")?,
        latest_mode: json_string(
            &request.latest_mode.unwrap_or(LatestMode::ExactWatermark),
            "primitives latest gRPC latest_mode",
        )?,
        exclude_sources: Vec::new(),
        metadata: request.metadata.unwrap_or(false),
        family: request
            .family
            .as_deref()
            .map(|values| json_string_list(values, "primitives latest gRPC family"))
            .transpose()?
            .unwrap_or_default(),
        group: request
            .group
            .as_deref()
            .map(|values| json_string_list(values, "primitives latest gRPC group"))
            .transpose()?
            .unwrap_or_default(),
        diagnostics: request.diagnostics,
    })
}

fn primitives_range_http_body(request: &primitives_system::RangeRequest) -> Result<Value, String> {
    Ok(json!({
        "pairs": normalize_required_pairs(&request.pairs, "primitives range outputs")?,
        "tf": request.tf,
        "align_mode": request.align_mode,
        "close_start_ms": request.close_start.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "cursor": normalize_optional_string(request.cursor.as_deref()),
        "close_end_ms": request.close_end.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "limit": request.limit,
        "family": request.family,
        "group": request.group,
        "metadata": request.metadata,
        "diagnostics": request.diagnostics,
        "format": request.format,
    }))
}

fn primitives_range_grpc_proto(
    request: &primitives_system::RangeGrpcRequest,
) -> Result<primitives_proto::RangeOutputsRequestV1, String> {
    Ok(primitives_proto::RangeOutputsRequestV1 {
        pairs: normalize_required_pairs(&request.pairs, "primitives range outputs gRPC")?,
        tf: json_string(&request.tf, "primitives range gRPC tf")?,
        close_end_ms: request
            .close_end
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        cursor: normalize_optional_string(request.cursor.as_deref()),
        limit: request.limit,
        exclude_sources: Vec::new(),
        metadata: request.metadata.unwrap_or(false),
        close_start_ms: request
            .close_start
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        align_mode: request
            .align_mode
            .as_ref()
            .map(|value| json_string(value, "primitives range gRPC align_mode"))
            .transpose()?,
        family: request
            .family
            .as_deref()
            .map(|values| json_string_list(values, "primitives range gRPC family"))
            .transpose()?
            .unwrap_or_default(),
        group: request
            .group
            .as_deref()
            .map(|values| json_string_list(values, "primitives range gRPC group"))
            .transpose()?
            .unwrap_or_default(),
        diagnostics: request.diagnostics,
    })
}

fn primitives_search_http_body(
    request: &primitives_system::SearchRequest,
) -> Result<Value, String> {
    Ok(json!({
        "tf": request.tf,
        "close_start_ms": request.close_start.to_utc_ms().map_err(|error| error.to_string())?,
        "close_end_ms": request.close_end.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "cursor": normalize_optional_string(request.cursor.as_deref()),
        "predicate": normalize_required_string(&request.predicate, "primitives search predicate")?,
        "evaluate_pair": normalize_optional_string(request.evaluate_pair.as_deref()),
        "family": request.family,
        "group": request.group,
        "metadata": request.metadata,
        "diagnostics": request.diagnostics,
        "max_hits": request.max_hits,
        "format": request.format,
    }))
}

fn primitives_search_grpc_proto(
    request: &primitives_system::SearchGrpcRequest,
) -> Result<primitives_proto::SearchOutputsRequestV1, String> {
    Ok(primitives_proto::SearchOutputsRequestV1 {
        tf: json_string(&request.tf, "primitives search gRPC tf")?,
        close_start_ms: request
            .close_start
            .to_utc_ms()
            .map_err(|error| error.to_string())?,
        close_end_ms: request
            .close_end
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        cursor: normalize_optional_string(request.cursor.as_deref()),
        predicate: normalize_required_string(
            &request.predicate,
            "primitives search gRPC predicate",
        )?,
        evaluate_pair: normalize_optional_string(request.evaluate_pair.as_deref()),
        exclude_sources: Vec::new(),
        metadata: request.metadata.unwrap_or(false),
        max_hits: request.max_hits,
        family: request
            .family
            .as_deref()
            .map(|values| json_string_list(values, "primitives search gRPC family"))
            .transpose()?
            .unwrap_or_default(),
        group: request
            .group
            .as_deref()
            .map(|values| json_string_list(values, "primitives search gRPC group"))
            .transpose()?
            .unwrap_or_default(),
        diagnostics: request.diagnostics,
    })
}

fn primitives_time_machine_http_body(
    request: &primitives_system::TimeMachineRequest,
) -> Result<Value, String> {
    Ok(json!({
        "tf": request.tf,
        "close_start_ms": request.close_start.to_utc_ms().map_err(|error| error.to_string())?,
        "close_end_ms": request.close_end.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "cursor": normalize_optional_string(request.cursor.as_deref()),
        "predicate": request.predicate.as_deref().map(|value| normalize_required_string(value, "primitives time-machine predicate")).transpose()?,
        "hits": request.hits,
        "output_pairs": normalize_optional_pairs(request.output_pairs.as_deref()),
        "family": request.family,
        "group": request.group,
        "metadata": request.metadata,
        "diagnostics": request.diagnostics,
        "before_bars": request.before_bars,
        "after_bars": request.after_bars,
        "max_hits": request.max_hits,
        "overlap_mode": normalize_optional_string(request.overlap_mode.as_deref()),
        "format": request.format,
    }))
}

fn primitives_time_machine_grpc_proto(
    request: &primitives_system::TimeMachineGrpcRequest,
) -> Result<primitives_proto::TimeMachineOutputsRequestV1, String> {
    Ok(primitives_proto::TimeMachineOutputsRequestV1 {
        tf: json_string(&request.tf, "primitives time-machine gRPC tf")?,
        close_start_ms: request
            .close_start
            .to_utc_ms()
            .map_err(|error| error.to_string())?,
        close_end_ms: request
            .close_end
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        cursor: normalize_optional_string(request.cursor.as_deref()),
        predicate: request
            .predicate
            .as_deref()
            .map(|value| normalize_required_string(value, "primitives time-machine gRPC predicate"))
            .transpose()?,
        hits: request.hits.clone().unwrap_or_default(),
        output_pairs: normalize_optional_pairs(request.output_pairs.as_deref()).unwrap_or_default(),
        exclude_sources: Vec::new(),
        metadata: request.metadata.unwrap_or(false),
        before_bars: request.before_bars,
        after_bars: request.after_bars,
        max_hits: request.max_hits,
        overlap_mode: normalize_optional_string(request.overlap_mode.as_deref()),
        family: request
            .family
            .as_deref()
            .map(|values| json_string_list(values, "primitives time-machine gRPC family"))
            .transpose()?
            .unwrap_or_default(),
        group: request
            .group
            .as_deref()
            .map(|values| json_string_list(values, "primitives time-machine gRPC group"))
            .transpose()?
            .unwrap_or_default(),
        diagnostics: request.diagnostics,
    })
}

fn regime_latest_http_body(request: &regime_system::LatestRequest) -> Result<Value, String> {
    Ok(json!({
        "pairs": normalize_required_pairs(&request.pairs, "regime latest outputs")?,
        "tf": request.tf,
        "latest_mode": request.latest_mode,
        "family": request.family,
        "group": request.group,
        "secondary": request.secondary,
        "metadata": request.metadata,
        "diagnostics": request.diagnostics,
        "format": request.format,
    }))
}

fn regime_latest_grpc_proto(
    request: &regime_system::LatestGrpcRequest,
) -> Result<regime_proto::LatestOutputsRequestV1, String> {
    Ok(regime_proto::LatestOutputsRequestV1 {
        pairs: normalize_required_pairs(&request.pairs, "regime latest outputs gRPC")?,
        tf: json_string(&request.tf, "regime latest gRPC tf")?,
        latest_mode: json_string(
            &request.latest_mode.unwrap_or(LatestMode::ExactWatermark),
            "regime latest gRPC latest_mode",
        )?,
        exclude_sources: Vec::new(),
        metadata: request.metadata.unwrap_or(false),
        family: request
            .family
            .as_deref()
            .map(|values| json_string_list(values, "regime latest gRPC family"))
            .transpose()?
            .unwrap_or_default(),
        group: request
            .group
            .as_deref()
            .map(|values| json_string_list(values, "regime latest gRPC group"))
            .transpose()?
            .unwrap_or_default(),
        diagnostics: request.diagnostics,
        secondary: request.secondary.unwrap_or(false),
    })
}

fn regime_range_http_body(request: &regime_system::RangeRequest) -> Result<Value, String> {
    Ok(json!({
        "pairs": normalize_required_pairs(&request.pairs, "regime range outputs")?,
        "tf": request.tf,
        "align_mode": request.align_mode,
        "close_start_ms": request.close_start.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "cursor": normalize_optional_string(request.cursor.as_deref()),
        "close_end_ms": request.close_end.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "limit": request.limit,
        "family": request.family,
        "group": request.group,
        "secondary": request.secondary,
        "metadata": request.metadata,
        "diagnostics": request.diagnostics,
        "format": request.format,
    }))
}

fn regime_range_grpc_proto(
    request: &regime_system::RangeGrpcRequest,
) -> Result<regime_proto::RangeOutputsRequestV1, String> {
    Ok(regime_proto::RangeOutputsRequestV1 {
        pairs: normalize_required_pairs(&request.pairs, "regime range outputs gRPC")?,
        tf: json_string(&request.tf, "regime range gRPC tf")?,
        close_end_ms: request
            .close_end
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        cursor: normalize_optional_string(request.cursor.as_deref()),
        limit: request.limit,
        exclude_sources: Vec::new(),
        metadata: request.metadata.unwrap_or(false),
        close_start_ms: request
            .close_start
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        align_mode: request
            .align_mode
            .as_ref()
            .map(|value| json_string(value, "regime range gRPC align_mode"))
            .transpose()?,
        family: request
            .family
            .as_deref()
            .map(|values| json_string_list(values, "regime range gRPC family"))
            .transpose()?
            .unwrap_or_default(),
        group: request
            .group
            .as_deref()
            .map(|values| json_string_list(values, "regime range gRPC group"))
            .transpose()?
            .unwrap_or_default(),
        diagnostics: request.diagnostics,
        secondary: request.secondary.unwrap_or(false),
    })
}

fn regime_search_http_body(request: &regime_system::SearchRequest) -> Result<Value, String> {
    Ok(json!({
        "tf": request.tf,
        "close_start_ms": request.close_start.to_utc_ms().map_err(|error| error.to_string())?,
        "close_end_ms": request.close_end.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "cursor": normalize_optional_string(request.cursor.as_deref()),
        "predicate": normalize_required_string(&request.predicate, "regime search predicate")?,
        "evaluate_pair": normalize_optional_string(request.evaluate_pair.as_deref()),
        "family": request.family,
        "group": request.group,
        "secondary": request.secondary,
        "metadata": request.metadata,
        "diagnostics": request.diagnostics,
        "max_hits": request.max_hits,
        "format": request.format,
    }))
}

fn regime_search_grpc_proto(
    request: &regime_system::SearchGrpcRequest,
) -> Result<regime_proto::SearchOutputsRequestV1, String> {
    Ok(regime_proto::SearchOutputsRequestV1 {
        tf: json_string(&request.tf, "regime search gRPC tf")?,
        close_start_ms: request
            .close_start
            .to_utc_ms()
            .map_err(|error| error.to_string())?,
        close_end_ms: request
            .close_end
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        cursor: normalize_optional_string(request.cursor.as_deref()),
        predicate: normalize_required_string(&request.predicate, "regime search gRPC predicate")?,
        evaluate_pair: normalize_optional_string(request.evaluate_pair.as_deref()),
        exclude_sources: Vec::new(),
        metadata: request.metadata.unwrap_or(false),
        max_hits: request.max_hits,
        family: request
            .family
            .as_deref()
            .map(|values| json_string_list(values, "regime search gRPC family"))
            .transpose()?
            .unwrap_or_default(),
        group: request
            .group
            .as_deref()
            .map(|values| json_string_list(values, "regime search gRPC group"))
            .transpose()?
            .unwrap_or_default(),
        diagnostics: request.diagnostics,
        secondary: request.secondary.unwrap_or(false),
    })
}

fn regime_time_machine_http_body(
    request: &regime_system::TimeMachineRequest,
) -> Result<Value, String> {
    Ok(json!({
        "tf": request.tf,
        "close_start_ms": request.close_start.to_utc_ms().map_err(|error| error.to_string())?,
        "close_end_ms": request.close_end.as_ref().map(TimeInput::to_utc_ms).transpose().map_err(|error| error.to_string())?,
        "cursor": normalize_optional_string(request.cursor.as_deref()),
        "predicate": request.predicate.as_deref().map(|value| normalize_required_string(value, "regime time-machine predicate")).transpose()?,
        "hits": request.hits,
        "output_pairs": normalize_optional_pairs(request.output_pairs.as_deref()),
        "family": request.family,
        "group": request.group,
        "secondary": request.secondary,
        "metadata": request.metadata,
        "diagnostics": request.diagnostics,
        "before_bars": request.before_bars,
        "after_bars": request.after_bars,
        "max_hits": request.max_hits,
        "overlap_mode": normalize_optional_string(request.overlap_mode.as_deref()),
        "format": request.format,
    }))
}

fn regime_time_machine_grpc_proto(
    request: &regime_system::TimeMachineGrpcRequest,
) -> Result<regime_proto::TimeMachineOutputsRequestV1, String> {
    Ok(regime_proto::TimeMachineOutputsRequestV1 {
        tf: json_string(&request.tf, "regime time-machine gRPC tf")?,
        close_start_ms: request
            .close_start
            .to_utc_ms()
            .map_err(|error| error.to_string())?,
        close_end_ms: request
            .close_end
            .as_ref()
            .map(TimeInput::to_utc_ms)
            .transpose()
            .map_err(|error| error.to_string())?
            .unwrap_or(0),
        cursor: normalize_optional_string(request.cursor.as_deref()),
        predicate: request
            .predicate
            .as_deref()
            .map(|value| normalize_required_string(value, "regime time-machine gRPC predicate"))
            .transpose()?,
        hits: request.hits.clone().unwrap_or_default(),
        output_pairs: normalize_optional_pairs(request.output_pairs.as_deref()).unwrap_or_default(),
        exclude_sources: Vec::new(),
        metadata: request.metadata.unwrap_or(false),
        before_bars: request.before_bars,
        after_bars: request.after_bars,
        max_hits: request.max_hits,
        overlap_mode: normalize_optional_string(request.overlap_mode.as_deref()),
        family: request
            .family
            .as_deref()
            .map(|values| json_string_list(values, "regime time-machine gRPC family"))
            .transpose()?
            .unwrap_or_default(),
        group: request
            .group
            .as_deref()
            .map(|values| json_string_list(values, "regime time-machine gRPC group"))
            .transpose()?
            .unwrap_or_default(),
        diagnostics: request.diagnostics,
        secondary: request.secondary.unwrap_or(false),
    })
}

fn download_root_for(system: &str) -> PathBuf {
    repo_root()
        .join("target")
        .join("endpoint_test_downloads")
        .join(system)
}

fn aggregator_bars_ws_frame_kind(frame: &BarsWsInboundFrame) -> &'static str {
    match frame {
        BarsWsInboundFrame::Meta(_) => "meta",
        BarsWsInboundFrame::JsonRows(_) => "json_rows",
        BarsWsInboundFrame::ProtobufRows(_) => "protobuf_rows",
        BarsWsInboundFrame::Error(_) => "error",
    }
}

fn aggregator_bars_ws_is_payload(frame: &BarsWsInboundFrame) -> bool {
    matches!(
        frame,
        BarsWsInboundFrame::JsonRows(_) | BarsWsInboundFrame::ProtobufRows(_)
    )
}

fn aggregator_messages_ws_frame_kind(frame: &AggregatorMessagesWsServerFrame) -> &'static str {
    match frame {
        AggregatorMessagesWsServerFrame::Subscribed(_) => "subscribed",
        AggregatorMessagesWsServerFrame::Message(_) => "message",
        AggregatorMessagesWsServerFrame::Heartbeat(_) => "heartbeat",
        AggregatorMessagesWsServerFrame::Error(_) => "error",
    }
}

fn primitive_outputs_ws_frame_kind(frame: &PrimitiveOutputsWsInboundFrame) -> &'static str {
    match frame {
        PrimitiveOutputsWsInboundFrame::Meta(_) => "meta",
        PrimitiveOutputsWsInboundFrame::JsonRows(_) => "json_rows",
        PrimitiveOutputsWsInboundFrame::ProtobufRows(_) => "protobuf_rows",
        PrimitiveOutputsWsInboundFrame::Error(_) => "error",
    }
}

fn primitive_outputs_ws_is_payload(frame: &PrimitiveOutputsWsInboundFrame) -> bool {
    matches!(
        frame,
        PrimitiveOutputsWsInboundFrame::JsonRows(_)
            | PrimitiveOutputsWsInboundFrame::ProtobufRows(_)
    )
}

fn primitive_messages_ws_frame_kind(frame: &PrimitiveMessagesWsServerFrame) -> &'static str {
    match frame {
        PrimitiveMessagesWsServerFrame::Subscribed(_) => "subscribed",
        PrimitiveMessagesWsServerFrame::Message(_) => "message",
        PrimitiveMessagesWsServerFrame::Heartbeat(_) => "heartbeat",
        PrimitiveMessagesWsServerFrame::Error(_) => "error",
    }
}

fn regime_outputs_ws_frame_kind(frame: &RegimeOutputsWsInboundFrame) -> &'static str {
    match frame {
        RegimeOutputsWsInboundFrame::Meta(_) => "meta",
        RegimeOutputsWsInboundFrame::JsonRows(_) => "json_rows",
        RegimeOutputsWsInboundFrame::ProtobufRows(_) => "protobuf_rows",
        RegimeOutputsWsInboundFrame::Error(_) => "error",
    }
}

fn regime_messages_ws_frame_kind(frame: &RegimeMessagesWsServerFrame) -> &'static str {
    match frame {
        RegimeMessagesWsServerFrame::Subscribed(_) => "subscribed",
        RegimeMessagesWsServerFrame::Message(_) => "message",
        RegimeMessagesWsServerFrame::Heartbeat(_) => "heartbeat",
        RegimeMessagesWsServerFrame::Error(_) => "error",
    }
}

async fn run_phase_3_foundation(runtime: &RuntimeConfig, report: &mut Report) {
    let _ = (
        &runtime.aggregator,
        &runtime.intro,
        &runtime.primitives,
        &runtime.regime,
    );

    record_pass(
        report,
        "foundation.client_construction",
        "constructed Intro, Aggregator, Primitives, and Regime clients",
    );
    record_pass(
        report,
        "foundation.report_scaffold",
        "initialized report scaffold with full planned surface table",
    );
}

async fn run_phase_4_intro_and_aggregator(runtime: &RuntimeConfig, report: &mut Report) {
    let intro = &runtime.intro;
    let client = &runtime.aggregator;

    match intro.intro().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "intro.intro",
                format!(
                    "subsystem={} keys={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(report, "intro.intro", "intro root was not a JSON object"),
        Err(error) => record_fail(report, "intro.intro", error.to_string()),
    }

    match client.docs_system().await {
        Ok(out) if json_str(&out, "intro").is_some_and(|intro| !intro.trim().is_empty()) => {
            record_pass(
                report,
                "aggregator.docs_system",
                format!(
                    "subsystem={} sections={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    json_array_len(&out, "sections").unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(report, "aggregator.docs_system", "empty docs content"),
        Err(error) => record_fail(report, "aggregator.docs_system", error.to_string()),
    }

    match client.docs_summary().await {
        Ok(out) if json_str(&out, "intro").is_some_and(|intro| !intro.trim().is_empty()) => {
            record_pass(
                report,
                "aggregator.docs_summary",
                format!(
                    "subsystem={} sections={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    json_array_len(&out, "sections").unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(report, "aggregator.docs_summary", "empty summary content"),
        Err(error) => record_fail(report, "aggregator.docs_summary", error.to_string()),
    }

    match client.docs_themes().await {
        Ok(out) if json_array_len(&out, "themes").is_some_and(|count| count > 0) => {
            record_pass(
                report,
                "aggregator.docs_themes",
                format!(
                    "subsystem={} themes={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    json_array_len(&out, "themes").unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(report, "aggregator.docs_themes", "empty themes content"),
        Err(error) => record_fail(report, "aggregator.docs_themes", error.to_string()),
    }

    match client.docs_endpoints().await {
        Ok(out) if json_str(&out, "intro").is_some_and(|intro| !intro.trim().is_empty()) => {
            record_pass(
                report,
                "aggregator.docs_endpoints",
                format!(
                    "subsystem={} sections={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    json_array_len(&out, "sections").unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "aggregator.docs_endpoints",
            "empty endpoints content",
        ),
        Err(error) => record_fail(report, "aggregator.docs_endpoints", error.to_string()),
    }

    match client.openapi().await {
        Ok(out) if out.get("openapi").is_some() && out.get("paths").is_some() => {
            let path_count = out
                .get("paths")
                .and_then(|paths| paths.as_object())
                .map(|paths| paths.len())
                .unwrap_or(0);
            record_pass(report, "aggregator.openapi", format!("paths={path_count}"));
        }
        Ok(_) => record_fail(
            report,
            "aggregator.openapi",
            "missing `openapi` or `paths` keys",
        ),
        Err(error) => record_fail(report, "aggregator.openapi", error.to_string()),
    }

    let pairs_list_out = match client
        .pairs_list(&PairsListRequest {
            after_pair: None,
            limit: Some(5),
            enabled_only: Some(true),
        })
        .await
    {
        Ok(out) if !out.pairs.is_empty() => {
            let len = out.pairs.len();
            record_pass(report, "aggregator.pairs_list", format!("pairs={len}"));
            Some(out)
        }
        Ok(out) => {
            record_fail(
                report,
                "aggregator.pairs_list",
                format!("pairs={}", out.pairs.len()),
            );
            None
        }
        Err(error) => {
            record_fail(report, "aggregator.pairs_list", error.to_string());
            None
        }
    };

    let target_pair = pairs_list_out
        .as_ref()
        .and_then(|out| out.pairs.first())
        .cloned()
        .unwrap_or_else(|| "BTCUSDT".to_string());

    match client
        .pairs_status(&PairsStatusRequest {
            after_pair: None,
            limit: Some(1),
            pairs: Some(vec![target_pair.clone()]),
            filters: Some(vec!["status".to_string(), "frontier".to_string()]),
        })
        .await
    {
        Ok(out) if !out.pairs.is_empty() => {
            record_pass(
                report,
                "aggregator.pairs_status",
                format!("pair={} rows={}", out.pairs[0].pair, out.pairs.len()),
            );
        }
        Ok(_) => record_fail(
            report,
            "aggregator.pairs_status",
            "empty pairs status response",
        ),
        Err(error) => record_fail(report, "aggregator.pairs_status", error.to_string()),
    }

    match client
        .files_downloads(&FilesDownloadsRequest {
            period: Some("day".to_string()),
            pairs: vec![target_pair.clone()],
            tfs: vec!["1m".to_string()],
            start_label_utc: None,
            end_label_utc: None,
            order: Some("desc".to_string()),
        })
        .await
    {
        Ok(out)
            if !out.rows.is_empty()
                && out.rows[0].url.starts_with("http")
                && !out.rows[0].expires_at_utc.trim().is_empty() =>
        {
            record_pass(
                report,
                "aggregator.files_downloads",
                format!(
                    "rows={} first_pair={} expires_at_utc={}",
                    out.rows.len(),
                    out.rows[0].pair,
                    out.rows[0].expires_at_utc
                ),
            );
        }
        Ok(out) => record_fail(
            report,
            "aggregator.files_downloads",
            format!("unexpected rows shape; rows={}", out.rows.len()),
        ),
        Err(error) => record_fail(report, "aggregator.files_downloads", error.to_string()),
    }

    let latest_http_min_request = LatestRequest {
        pairs: vec![target_pair.clone()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    };
    let latest_http_full_request = LatestRequest {
        metadata: Some(true),
        ..latest_http_min_request.clone()
    };

    let anchor_close_ms = match async {
        let mut last_error = None;
        for _attempt in 0..4 {
            let min = client
                .latest(&latest_http_min_request)
                .await
                .map_err(|error| error.to_string())?;
            let full = client
                .latest(&latest_http_full_request)
                .await
                .map_err(|error| error.to_string())?;

            if pair_from_latest_response(&min).is_none()
                || pair_from_latest_response(&min) != pair_from_latest_response(&full)
                || close_end_from_latest_response(&min) != close_end_from_latest_response(&full)
            {
                last_error = Some("min/full latest parity mismatch".to_string());
                sleep(Duration::from_millis(250)).await;
                continue;
            }

            let expected_close_end_ms = close_end_from_latest_response(&min);
            let min_direct = canonical_aggregator_latest_http_raw(
                raw_http_post_json(
                    &runtime.summary.aggregator_http_base_url,
                    "/v1/bars/latest",
                    runtime.bearer_token.as_ref(),
                    &aggregator_latest_http_body(&latest_http_min_request)?,
                )
                .await?,
            );
            let full_direct = canonical_aggregator_latest_http_raw(
                raw_http_post_json(
                    &runtime.summary.aggregator_http_base_url,
                    "/v1/bars/latest",
                    runtime.bearer_token.as_ref(),
                    &aggregator_latest_http_body(&latest_http_full_request)?,
                )
                .await?,
            );
            let min_direct_close_end_ms = min_direct
                .get("close_end_ms")
                .and_then(Value::as_i64)
                .ok_or_else(|| "aggregator latest min direct payload missing close_end_ms".to_string())?;
            let full_direct_close_end_ms = full_direct
                .get("close_end_ms")
                .and_then(Value::as_i64)
                .ok_or_else(|| "aggregator latest full direct payload missing close_end_ms".to_string())?;

            if min_direct_close_end_ms != expected_close_end_ms
                || full_direct_close_end_ms != expected_close_end_ms
            {
                last_error = Some(format!(
                    "latest alignment drift sdk_close_end_ms={expected_close_end_ms} direct_min_close_end_ms={min_direct_close_end_ms} direct_full_close_end_ms={full_direct_close_end_ms}"
                ));
                sleep(Duration::from_millis(250)).await;
                continue;
            }

            let min_sdk = canonical_aggregator_latest_sdk(&min)?;
            let full_sdk = canonical_aggregator_latest_sdk(&full)?;
            compare_semantic_values("aggregator.latest[min]", &min_sdk, &min_direct)?;
            compare_semantic_values("aggregator.latest[full]", &full_sdk, &full_direct)?;
            return Ok::<(String, i64), String>((
                pair_from_latest_response(&min).unwrap_or("unknown").to_string(),
                expected_close_end_ms,
            ));
        }

        Err(last_error.unwrap_or_else(|| "aggregator latest alignment retries exhausted".to_string()))
    }
    .await
    {
        Ok((pair, close_end_ms)) => {
            record_pass(
                report,
                "aggregator.latest",
                format!(
                    "pair={} close_end_ms={} direct_http_semantic_match=true",
                    pair, close_end_ms
                ),
            );
            close_end_ms
        }
        Err(error) => {
            record_fail(report, "aggregator.latest", error);
            0
        }
    };

    let grpc_available = runtime.summary.aggregator_grpc_base_url.is_some();
    if grpc_available {
        let latest_grpc_min_request = LatestGrpcRequest::from(&latest_http_min_request);
        let latest_grpc_full_request = LatestGrpcRequest::from(&latest_http_full_request);
        match (
            client.latest_grpc(&latest_grpc_min_request).await,
            client.latest_grpc(&latest_grpc_full_request).await,
        ) {
            (Ok(min), Ok(full))
                if pair_from_latest_response(&min).is_some()
                    && pair_from_latest_response(&min) == pair_from_latest_response(&full) =>
            {
                match async {
                    let direct_min = raw_grpc_unary::<
                        aggregator_proto::LatestBarsRequestV1,
                        aggregator_proto::BarsLatestResponseV1,
                    >(
                        runtime
                            .summary
                            .aggregator_grpc_base_url
                            .as_deref()
                            .unwrap_or(""),
                        AGGREGATOR_LATEST_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        aggregator_latest_grpc_proto(&latest_grpc_min_request)?,
                    )
                    .await?;
                    let direct_full = raw_grpc_unary::<
                        aggregator_proto::LatestBarsRequestV1,
                        aggregator_proto::BarsLatestResponseV1,
                    >(
                        runtime
                            .summary
                            .aggregator_grpc_base_url
                            .as_deref()
                            .unwrap_or(""),
                        AGGREGATOR_LATEST_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        aggregator_latest_grpc_proto(&latest_grpc_full_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "aggregator.latest_grpc[min]",
                        &canonical_aggregator_latest_sdk(&min)?,
                        &canonical_aggregator_latest_grpc_raw(&direct_min)?,
                    )?;
                    compare_semantic_values(
                        "aggregator.latest_grpc[full]",
                        &canonical_aggregator_latest_sdk(&full)?,
                        &canonical_aggregator_latest_grpc_raw(&direct_full)?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "aggregator.latest_grpc",
                        format!(
                            "pair={} close_end_ms={} direct_grpc_semantic_match=true",
                            pair_from_latest_response(&min).unwrap_or("unknown"),
                            close_end_from_latest_response(&min)
                        ),
                    ),
                    Err(error) => record_fail(report, "aggregator.latest_grpc", error),
                }
            }
            (Ok(_), Ok(_)) => record_fail(
                report,
                "aggregator.latest_grpc",
                "min/full latest parity mismatch",
            ),
            (Err(error), _) => record_fail(report, "aggregator.latest_grpc", error.to_string()),
            (_, Err(error)) => record_fail(report, "aggregator.latest_grpc", error.to_string()),
        }
    } else {
        for surface in [
            "aggregator.latest_grpc",
            "aggregator.range_grpc",
            "aggregator.search_grpc",
            "aggregator.time_machine_grpc",
            "aggregator.range_grpc_call",
            "aggregator.search_grpc_call",
            "aggregator.time_machine_grpc_call",
        ] {
            record_fail(
                report,
                surface,
                "public aggregator gRPC base url missing from runtime config",
            );
        }
    }

    if anchor_close_ms <= 0 {
        for surface in [
            "aggregator.range",
            "aggregator.search",
            "aggregator.time_machine",
            "aggregator.range_call",
            "aggregator.search_call",
            "aggregator.time_machine_call",
        ] {
            record_fail(report, surface, "latest anchor was not established");
        }
        if grpc_available {
            for surface in [
                "aggregator.range_grpc",
                "aggregator.search_grpc",
                "aggregator.time_machine_grpc",
                "aggregator.range_grpc_call",
                "aggregator.search_grpc_call",
                "aggregator.time_machine_grpc_call",
            ] {
                record_fail(report, surface, "latest anchor was not established");
            }
        }
        return;
    }

    let range_min_request = RangeRequest {
        pairs: vec![target_pair.clone()],
        tf: Timeframe::M1,
        align_mode: Some(AlignMode::Exact),
        close_start: Some(TimeInput::Ms(anchor_close_ms - 10 * 60_000)),
        cursor: None,
        close_end: Some(TimeInput::Ms(anchor_close_ms)),
        limit: Some(5),
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    };
    let range_full_request = RangeRequest {
        metadata: Some(true),
        ..range_min_request.clone()
    };

    match (
        client.range(&range_min_request).await,
        client.range(&range_full_request).await,
    ) {
        (Ok(min), Ok(full))
            if range_rows_len(&min) > 0 && range_rows_len(&min) == range_rows_len(&full) =>
        {
            match async {
                let min_body = aggregator_range_http_body(&range_min_request)?;
                let full_body = aggregator_range_http_body(&range_full_request)?;
                let min_direct = raw_http_post_json(
                    &runtime.summary.aggregator_http_base_url,
                    "/v1/bars/range",
                    runtime.bearer_token.as_ref(),
                    &min_body,
                );
                let full_direct = raw_http_post_json(
                    &runtime.summary.aggregator_http_base_url,
                    "/v1/bars/range",
                    runtime.bearer_token.as_ref(),
                    &full_body,
                );
                let (min_direct, full_direct) =
                    futures_util::future::try_join(min_direct, full_direct)
                        .await
                        .map_err(|error| error.to_string())?;
                compare_semantic_values(
                    "aggregator.range[min]",
                    &canonical_aggregator_range_sdk(&min)?,
                    &min_direct,
                )?;
                compare_semantic_values(
                    "aggregator.range[full]",
                    &canonical_aggregator_range_sdk(&full)?,
                    &full_direct,
                )?;
                Ok::<(), String>(())
            }
            .await
            {
                Ok(()) => record_pass(
                    report,
                    "aggregator.range",
                    format!(
                        "rows={} close_end_ms={} direct_http_semantic_match=true",
                        range_rows_len(&min),
                        anchor_close_ms
                    ),
                ),
                Err(error) => record_fail(report, "aggregator.range", error),
            }
        }
        (Ok(min), Ok(full)) => record_fail(
            report,
            "aggregator.range",
            format!(
                "unexpected min/full range rows: min={} full={}",
                range_rows_len(&min),
                range_rows_len(&full)
            ),
        ),
        (Err(error), _) => record_fail(report, "aggregator.range", error.to_string()),
        (_, Err(error)) => record_fail(report, "aggregator.range", error.to_string()),
    }

    let predicate = format!("{target_pair}.close > 0");
    let search_min_request = SearchRequest {
        tf: Timeframe::M1,
        close_start: TimeInput::Ms(anchor_close_ms - 60 * 60_000),
        close_end: Some(TimeInput::Ms(anchor_close_ms)),
        cursor: None,
        predicate: predicate.clone(),
        evaluate_pair: Some(target_pair.clone()),
        metadata: Some(false),
        max_hits: Some(20),
        format: Some(HttpFormat::Json),
    };
    let search_full_request = SearchRequest {
        metadata: Some(true),
        ..search_min_request.clone()
    };

    match (
        client.search(&search_min_request).await,
        client.search(&search_full_request).await,
    ) {
        (Ok(min), Ok(full))
            if search_hits_len(&min) > 0 && search_hits_len(&min) == search_hits_len(&full) =>
        {
            match async {
                let min_body = aggregator_search_http_body(&search_min_request)?;
                let full_body = aggregator_search_http_body(&search_full_request)?;
                let min_direct = raw_http_post_json(
                    &runtime.summary.aggregator_http_base_url,
                    "/v1/bars/search",
                    runtime.bearer_token.as_ref(),
                    &min_body,
                );
                let full_direct = raw_http_post_json(
                    &runtime.summary.aggregator_http_base_url,
                    "/v1/bars/search",
                    runtime.bearer_token.as_ref(),
                    &full_body,
                );
                let (min_direct, full_direct) =
                    futures_util::future::try_join(min_direct, full_direct)
                        .await
                        .map_err(|error| error.to_string())?;
                compare_semantic_values(
                    "aggregator.search[min]",
                    &canonical_aggregator_search_sdk(&min)?,
                    &min_direct,
                )?;
                compare_semantic_values(
                    "aggregator.search[full]",
                    &canonical_aggregator_search_sdk(&full)?,
                    &full_direct,
                )?;
                Ok::<(), String>(())
            }
            .await
            {
                Ok(()) => record_pass(
                    report,
                    "aggregator.search",
                    format!(
                        "hits={} predicate={} direct_http_semantic_match=true",
                        search_hits_len(&min),
                        predicate
                    ),
                ),
                Err(error) => record_fail(report, "aggregator.search", error),
            }
        }
        (Ok(min), Ok(full)) => record_fail(
            report,
            "aggregator.search",
            format!(
                "unexpected min/full search hits: min={} full={}",
                search_hits_len(&min),
                search_hits_len(&full)
            ),
        ),
        (Err(error), _) => record_fail(report, "aggregator.search", error.to_string()),
        (_, Err(error)) => record_fail(report, "aggregator.search", error.to_string()),
    }

    let time_machine_min_request = TimeMachineRequest {
        tf: Timeframe::M1,
        close_start: TimeInput::Ms(anchor_close_ms - 20 * 60_000),
        close_end: Some(TimeInput::Ms(anchor_close_ms)),
        cursor: None,
        predicate: None,
        hits: Some(vec![anchor_close_ms - 120_000, anchor_close_ms - 60_000]),
        output_pairs: Some(vec![target_pair.clone()]),
        metadata: Some(false),
        before_bars: Some(2),
        after_bars: Some(2),
        max_hits: Some(10),
        overlap_mode: Some("clip".to_string()),
        format: Some(HttpFormat::Json),
    };
    let time_machine_full_request = TimeMachineRequest {
        metadata: Some(true),
        ..time_machine_min_request.clone()
    };

    match (
        client.time_machine(&time_machine_min_request).await,
        client.time_machine(&time_machine_full_request).await,
    ) {
        (Ok(min), Ok(full))
            if time_machine_rows_len(&min) > 0
                && time_machine_rows_len(&min) == time_machine_rows_len(&full) =>
        {
            match async {
                let min_body = aggregator_time_machine_http_body(&time_machine_min_request)?;
                let full_body = aggregator_time_machine_http_body(&time_machine_full_request)?;
                let min_direct = raw_http_post_json(
                    &runtime.summary.aggregator_http_base_url,
                    "/v1/bars/time-machine",
                    runtime.bearer_token.as_ref(),
                    &min_body,
                );
                let full_direct = raw_http_post_json(
                    &runtime.summary.aggregator_http_base_url,
                    "/v1/bars/time-machine",
                    runtime.bearer_token.as_ref(),
                    &full_body,
                );
                let (min_direct, full_direct) =
                    futures_util::future::try_join(min_direct, full_direct)
                        .await
                        .map_err(|error| error.to_string())?;
                compare_semantic_values(
                    "aggregator.time_machine[min]",
                    &canonical_aggregator_time_machine_sdk(&min)?,
                    &min_direct,
                )?;
                compare_semantic_values(
                    "aggregator.time_machine[full]",
                    &canonical_aggregator_time_machine_sdk(&full)?,
                    &full_direct,
                )?;
                Ok::<(), String>(())
            }
            .await
            {
                Ok(()) => record_pass(
                    report,
                    "aggregator.time_machine",
                    format!(
                        "rows={} direct_http_semantic_match=true",
                        time_machine_rows_len(&min)
                    ),
                ),
                Err(error) => record_fail(report, "aggregator.time_machine", error),
            }
        }
        (Ok(min), Ok(full)) => record_fail(
            report,
            "aggregator.time_machine",
            format!(
                "unexpected min/full time-machine rows: min={} full={}",
                time_machine_rows_len(&min),
                time_machine_rows_len(&full)
            ),
        ),
        (Err(error), _) => record_fail(report, "aggregator.time_machine", error.to_string()),
        (_, Err(error)) => record_fail(report, "aggregator.time_machine", error.to_string()),
    }

    let range_grpc_min_request = RangeGrpcRequest::from(&range_min_request);
    let search_grpc_min_request = SearchGrpcRequest::from(&search_min_request);
    let time_machine_grpc_min_request = TimeMachineGrpcRequest::from(&time_machine_min_request);

    if grpc_available {
        let range_grpc_full_request = RangeGrpcRequest::from(&range_full_request);
        match (
            client.range_grpc(&range_grpc_min_request).await,
            client.range_grpc(&range_grpc_full_request).await,
        ) {
            (Ok(min), Ok(full))
                if range_rows_len(&min) > 0 && range_rows_len(&min) == range_rows_len(&full) =>
            {
                match async {
                    let min_direct = raw_grpc_unary::<
                        aggregator_proto::RangeBarsRequestV1,
                        aggregator_proto::BarsRangeResponseV1,
                    >(
                        runtime
                            .summary
                            .aggregator_grpc_base_url
                            .as_deref()
                            .unwrap_or(""),
                        AGGREGATOR_RANGE_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        aggregator_range_grpc_proto(&range_grpc_min_request)?,
                    );
                    let full_direct = raw_grpc_unary::<
                        aggregator_proto::RangeBarsRequestV1,
                        aggregator_proto::BarsRangeResponseV1,
                    >(
                        runtime
                            .summary
                            .aggregator_grpc_base_url
                            .as_deref()
                            .unwrap_or(""),
                        AGGREGATOR_RANGE_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        aggregator_range_grpc_proto(&range_grpc_full_request)?,
                    );
                    let (min_direct, full_direct) =
                        futures_util::future::try_join(min_direct, full_direct)
                            .await
                            .map_err(|error| error.to_string())?;
                    compare_semantic_values(
                        "aggregator.range_grpc[min]",
                        &canonical_aggregator_range_sdk(&min)?,
                        &canonical_aggregator_range_grpc_raw(
                            &min_direct,
                            range_grpc_min_request.metadata.unwrap_or(false),
                        )?,
                    )?;
                    compare_semantic_values(
                        "aggregator.range_grpc[full]",
                        &canonical_aggregator_range_sdk(&full)?,
                        &canonical_aggregator_range_grpc_raw(
                            &full_direct,
                            range_grpc_full_request.metadata.unwrap_or(false),
                        )?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "aggregator.range_grpc",
                        format!(
                            "rows={} direct_grpc_semantic_match=true",
                            range_rows_len(&min)
                        ),
                    ),
                    Err(error) => record_fail(report, "aggregator.range_grpc", error),
                }
            }
            (Ok(min), Ok(full)) => record_fail(
                report,
                "aggregator.range_grpc",
                format!(
                    "unexpected min/full range rows: min={} full={}",
                    range_rows_len(&min),
                    range_rows_len(&full)
                ),
            ),
            (Err(error), _) => record_fail(report, "aggregator.range_grpc", error.to_string()),
            (_, Err(error)) => record_fail(report, "aggregator.range_grpc", error.to_string()),
        }

        let search_grpc_full_request = SearchGrpcRequest::from(&search_full_request);
        match (
            client.search_grpc(&search_grpc_min_request).await,
            client.search_grpc(&search_grpc_full_request).await,
        ) {
            (Ok(min), Ok(full))
                if search_hits_len(&min) > 0 && search_hits_len(&min) == search_hits_len(&full) =>
            {
                match async {
                    let min_direct = raw_grpc_unary::<
                        aggregator_proto::SearchBarsRequestV1,
                        aggregator_proto::BarsSearchResponseV1,
                    >(
                        runtime
                            .summary
                            .aggregator_grpc_base_url
                            .as_deref()
                            .unwrap_or(""),
                        AGGREGATOR_SEARCH_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        aggregator_search_grpc_proto(&search_grpc_min_request)?,
                    );
                    let full_direct = raw_grpc_unary::<
                        aggregator_proto::SearchBarsRequestV1,
                        aggregator_proto::BarsSearchResponseV1,
                    >(
                        runtime
                            .summary
                            .aggregator_grpc_base_url
                            .as_deref()
                            .unwrap_or(""),
                        AGGREGATOR_SEARCH_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        aggregator_search_grpc_proto(&search_grpc_full_request)?,
                    );
                    let (min_direct, full_direct) =
                        futures_util::future::try_join(min_direct, full_direct)
                            .await
                            .map_err(|error| error.to_string())?;
                    compare_semantic_values(
                        "aggregator.search_grpc[min]",
                        &canonical_aggregator_search_sdk(&min)?,
                        &canonical_aggregator_search_grpc_raw(
                            &min_direct,
                            search_grpc_min_request.metadata.unwrap_or(false),
                        )?,
                    )?;
                    compare_semantic_values(
                        "aggregator.search_grpc[full]",
                        &canonical_aggregator_search_sdk(&full)?,
                        &canonical_aggregator_search_grpc_raw(
                            &full_direct,
                            search_grpc_full_request.metadata.unwrap_or(false),
                        )?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "aggregator.search_grpc",
                        format!(
                            "hits={} direct_grpc_semantic_match=true",
                            search_hits_len(&min)
                        ),
                    ),
                    Err(error) => record_fail(report, "aggregator.search_grpc", error),
                }
            }
            (Ok(min), Ok(full)) => record_fail(
                report,
                "aggregator.search_grpc",
                format!(
                    "unexpected min/full search hits: min={} full={}",
                    search_hits_len(&min),
                    search_hits_len(&full)
                ),
            ),
            (Err(error), _) => record_fail(report, "aggregator.search_grpc", error.to_string()),
            (_, Err(error)) => record_fail(report, "aggregator.search_grpc", error.to_string()),
        }

        let time_machine_grpc_full_request =
            TimeMachineGrpcRequest::from(&time_machine_full_request);
        match (
            client
                .time_machine_grpc(&time_machine_grpc_min_request)
                .await,
            client
                .time_machine_grpc(&time_machine_grpc_full_request)
                .await,
        ) {
            (Ok(min), Ok(full))
                if time_machine_rows_len(&min) > 0
                    && time_machine_rows_len(&min) == time_machine_rows_len(&full) =>
            {
                match async {
                    let min_direct = raw_grpc_unary::<
                        aggregator_proto::TimeMachineBarsRequestV1,
                        aggregator_proto::BarsTimeMachineResponseV1,
                    >(
                        runtime
                            .summary
                            .aggregator_grpc_base_url
                            .as_deref()
                            .unwrap_or(""),
                        AGGREGATOR_TIME_MACHINE_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        aggregator_time_machine_grpc_proto(&time_machine_grpc_min_request)?,
                    );
                    let full_direct = raw_grpc_unary::<
                        aggregator_proto::TimeMachineBarsRequestV1,
                        aggregator_proto::BarsTimeMachineResponseV1,
                    >(
                        runtime
                            .summary
                            .aggregator_grpc_base_url
                            .as_deref()
                            .unwrap_or(""),
                        AGGREGATOR_TIME_MACHINE_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        aggregator_time_machine_grpc_proto(&time_machine_grpc_full_request)?,
                    );
                    let (min_direct, full_direct) =
                        futures_util::future::try_join(min_direct, full_direct)
                            .await
                            .map_err(|error| error.to_string())?;
                    compare_semantic_values(
                        "aggregator.time_machine_grpc[min]",
                        &canonical_aggregator_time_machine_sdk(&min)?,
                        &canonical_aggregator_time_machine_grpc_raw(
                            &min_direct,
                            time_machine_grpc_min_request.metadata.unwrap_or(false),
                        )?,
                    )?;
                    compare_semantic_values(
                        "aggregator.time_machine_grpc[full]",
                        &canonical_aggregator_time_machine_sdk(&full)?,
                        &canonical_aggregator_time_machine_grpc_raw(
                            &full_direct,
                            time_machine_grpc_full_request.metadata.unwrap_or(false),
                        )?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "aggregator.time_machine_grpc",
                        format!(
                            "rows={} direct_grpc_semantic_match=true",
                            time_machine_rows_len(&min)
                        ),
                    ),
                    Err(error) => record_fail(report, "aggregator.time_machine_grpc", error),
                }
            }
            (Ok(min), Ok(full)) => record_fail(
                report,
                "aggregator.time_machine_grpc",
                format!(
                    "unexpected min/full time-machine rows: min={} full={}",
                    time_machine_rows_len(&min),
                    time_machine_rows_len(&full)
                ),
            ),
            (Err(error), _) => {
                record_fail(report, "aggregator.time_machine_grpc", error.to_string())
            }
            (_, Err(error)) => {
                record_fail(report, "aggregator.time_machine_grpc", error.to_string())
            }
        }
    }

    match client
        .range_call(range_min_request.clone())
        .traverse()
        .await
    {
        Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => {
            record_pass(
                report,
                "aggregator.range_call",
                format!("pages_fetched={}", out.pages_fetched),
            );
        }
        Ok(out) => record_fail(
            report,
            "aggregator.range_call",
            format!(
                "unexpected traverse shape: pages_fetched={} pages={}",
                out.pages_fetched,
                out.pages.len()
            ),
        ),
        Err(error) => record_fail(report, "aggregator.range_call", error.to_string()),
    }

    match client
        .search_call(search_min_request.clone())
        .traverse()
        .await
    {
        Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => {
            record_pass(
                report,
                "aggregator.search_call",
                format!("pages_fetched={}", out.pages_fetched),
            );
        }
        Ok(out) => record_fail(
            report,
            "aggregator.search_call",
            format!(
                "unexpected traverse shape: pages_fetched={} pages={}",
                out.pages_fetched,
                out.pages.len()
            ),
        ),
        Err(error) => record_fail(report, "aggregator.search_call", error.to_string()),
    }

    match client
        .time_machine_call(time_machine_min_request.clone())
        .traverse()
        .await
    {
        Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => {
            record_pass(
                report,
                "aggregator.time_machine_call",
                format!("pages_fetched={}", out.pages_fetched),
            );
        }
        Ok(out) => record_fail(
            report,
            "aggregator.time_machine_call",
            format!(
                "unexpected traverse shape: pages_fetched={} pages={}",
                out.pages_fetched,
                out.pages.len()
            ),
        ),
        Err(error) => record_fail(report, "aggregator.time_machine_call", error.to_string()),
    }

    if grpc_available {
        match client
            .range_grpc_call(range_grpc_min_request)
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => {
                record_pass(
                    report,
                    "aggregator.range_grpc_call",
                    format!("pages_fetched={}", out.pages_fetched),
                );
            }
            Ok(out) => record_fail(
                report,
                "aggregator.range_grpc_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "aggregator.range_grpc_call", error.to_string()),
        }

        match client
            .search_grpc_call(search_grpc_min_request)
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => {
                record_pass(
                    report,
                    "aggregator.search_grpc_call",
                    format!("pages_fetched={}", out.pages_fetched),
                );
            }
            Ok(out) => record_fail(
                report,
                "aggregator.search_grpc_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "aggregator.search_grpc_call", error.to_string()),
        }

        match client
            .time_machine_grpc_call(time_machine_grpc_min_request)
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => {
                record_pass(
                    report,
                    "aggregator.time_machine_grpc_call",
                    format!("pages_fetched={}", out.pages_fetched),
                );
            }
            Ok(out) => record_fail(
                report,
                "aggregator.time_machine_grpc_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(
                report,
                "aggregator.time_machine_grpc_call",
                error.to_string(),
            ),
        }
    }
}

async fn run_phase_5_primitives_and_regime(runtime: &RuntimeConfig, report: &mut Report) {
    let primitives = &runtime.primitives;
    let regime = &runtime.regime;

    match primitives.docs_system().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "primitives.docs_system",
                format!(
                    "subsystem={} keys={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.docs_system",
            "docs_system was not a JSON object",
        ),
        Err(error) => record_fail(report, "primitives.docs_system", error.to_string()),
    }

    match primitives.docs_summary().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "primitives.docs_summary",
                format!(
                    "subsystem={} keys={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.docs_summary",
            "docs_summary was not a JSON object",
        ),
        Err(error) => record_fail(report, "primitives.docs_summary", error.to_string()),
    }

    match primitives.docs_taxonomy().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "primitives.docs_taxonomy",
                format!(
                    "keys={}",
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.docs_taxonomy",
            "docs_taxonomy was not a JSON object",
        ),
        Err(error) => record_fail(report, "primitives.docs_taxonomy", error.to_string()),
    }

    match primitives
        .docs_registry(&primitives_system::DocsRegistryRequest::default())
        .await
    {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "primitives.docs_registry",
                format!(
                    "keys={}",
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.docs_registry",
            "docs_registry was not a JSON object",
        ),
        Err(error) => record_fail(report, "primitives.docs_registry", error.to_string()),
    }

    match primitives.docs_endpoints().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "primitives.docs_endpoints",
                format!(
                    "keys={}",
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.docs_endpoints",
            "docs_endpoints was not a JSON object",
        ),
        Err(error) => record_fail(report, "primitives.docs_endpoints", error.to_string()),
    }

    match primitives.openapi().await {
        Ok(out) if out.get("openapi").is_some() => {
            record_pass(
                report,
                "primitives.openapi",
                format!(
                    "openapi={} paths={}",
                    json_str(&out, "openapi").unwrap_or(""),
                    out.get("paths")
                        .and_then(|value| value.as_object())
                        .map(|value| value.len())
                        .unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.openapi",
            "openapi document missing `openapi` key",
        ),
        Err(error) => record_fail(report, "primitives.openapi", error.to_string()),
    }

    let primitives_pairs_list_out = match primitives
        .pairs_list(&primitives_system::PairsListRequest {
            after_pair: None,
            limit: Some(5),
            enabled_only: Some(true),
        })
        .await
    {
        Ok(out) if !out.pairs.is_empty() => {
            record_pass(
                report,
                "primitives.pairs_list",
                format!("pairs={}", out.pairs.len()),
            );
            Some(out)
        }
        Ok(out) => {
            record_fail(
                report,
                "primitives.pairs_list",
                format!("pairs={}", out.pairs.len()),
            );
            None
        }
        Err(error) => {
            record_fail(report, "primitives.pairs_list", error.to_string());
            None
        }
    };

    let primitives_target_pair = primitives_pairs_list_out
        .as_ref()
        .and_then(|out| out.pairs.first())
        .cloned()
        .unwrap_or_else(|| "BTCUSDT".to_string());

    match primitives
        .pairs_status(&primitives_system::PairsStatusRequest {
            after_pair: None,
            limit: Some(1),
            pairs: Some(vec![primitives_target_pair.clone()]),
            filters: Some(vec!["status".to_string(), "readiness".to_string()]),
        })
        .await
    {
        Ok(out) if !out.pairs.is_empty() => {
            record_pass(
                report,
                "primitives.pairs_status",
                format!("pair={} rows={}", out.pairs[0].pair, out.pairs.len()),
            );
        }
        Ok(_) => record_fail(
            report,
            "primitives.pairs_status",
            "empty pairs status response",
        ),
        Err(error) => record_fail(report, "primitives.pairs_status", error.to_string()),
    }

    match primitives
        .files_downloads(&primitives_system::FilesDownloadsRequest {
            period: Some("day".to_string()),
            pairs: vec![primitives_target_pair.clone()],
            tfs: vec!["1m".to_string()],
            start_label_utc: None,
            end_label_utc: None,
            order: Some("desc".to_string()),
        })
        .await
    {
        Ok(out)
            if !out.rows.is_empty()
                && out.rows[0].url.starts_with("http")
                && !out.rows[0].expires_at_utc.trim().is_empty() =>
        {
            record_pass(
                report,
                "primitives.files_downloads",
                format!(
                    "rows={} first_pair={} expires_at_utc={}",
                    out.rows.len(),
                    out.rows[0].pair,
                    out.rows[0].expires_at_utc
                ),
            );
        }
        Ok(out) => record_fail(
            report,
            "primitives.files_downloads",
            format!("unexpected rows shape; rows={}", out.rows.len()),
        ),
        Err(error) => record_fail(report, "primitives.files_downloads", error.to_string()),
    }

    let primitives_latest_request = primitives_system::LatestRequest {
        pairs: vec![primitives_target_pair.clone(), "ETHUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Json),
    };

    let primitives_anchor_close_ms = match primitives.latest(&primitives_latest_request).await {
        Ok(out) if !out.rows.is_empty() => {
            let mode = primitive_mode_from_selectors_and_metadata(
                primitives_latest_request.family.as_deref(),
                primitives_latest_request.group.as_deref(),
                primitives_latest_request.metadata,
            );
            match async {
                let direct = raw_http_post_json(
                    &runtime.summary.primitives_http_base_url,
                    "/v1/outputs/latest",
                    runtime.bearer_token.as_ref(),
                    &primitives_latest_http_body(&primitives_latest_request)?,
                )
                .await?;
                compare_semantic_values(
                    "primitives.latest",
                    &canonical_primitive_latest_sdk(&out, mode)?,
                    &canonical_compute_latest_raw(direct, mode)?,
                )?;
                Ok::<i64, String>(out.close_end_ms)
            }
            .await
            {
                Ok(close_end_ms) => {
                    record_pass(
                        report,
                        "primitives.latest",
                        format!(
                            "rows={} missing_pairs={} kind={} close_end_ms={} direct_http_semantic_match=true",
                            out.rows.len(),
                            out.missing_pairs.len(),
                            mode.label(),
                            close_end_ms
                        ),
                    );
                    close_end_ms
                }
                Err(error) => {
                    record_fail(report, "primitives.latest", error);
                    0
                }
            }
        }
        Ok(out) => {
            record_fail(
                report,
                "primitives.latest",
                format!("latest returned rows={}", out.rows.len()),
            );
            0
        }
        Err(error) => {
            record_fail(report, "primitives.latest", error.to_string());
            0
        }
    };

    let primitives_projected_http_protobuf_request = primitives_system::LatestRequest {
        pairs: vec![primitives_target_pair.clone()],
        tf: Timeframe::M1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: Some(vec![PrimitiveProcessorFamily::MovingAverages]),
        group: Some(vec![PrimitiveProcessorGroup::Ema]),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Protobuf),
    };
    match primitives
        .latest(&primitives_projected_http_protobuf_request)
        .await
    {
        Err(SdkError::UnsupportedOrUnprovedUsage { .. }) => record_pass(
            report,
            "primitives.projected_http_protobuf_rejection",
            "projected HTTP protobuf request rejected before transport",
        ),
        Err(error) => record_fail(
            report,
            "primitives.projected_http_protobuf_rejection",
            error.to_string(),
        ),
        Ok(_) => record_fail(
            report,
            "primitives.projected_http_protobuf_rejection",
            "projected HTTP protobuf request unexpectedly succeeded",
        ),
    }

    let primitives_projected_grpc_request = primitives_system::LatestGrpcRequest {
        pairs: vec![primitives_target_pair.clone()],
        tf: Timeframe::M1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: Some(vec![PrimitiveProcessorFamily::MovingAverages]),
        group: Some(vec![PrimitiveProcessorGroup::Ema]),
        metadata: Some(false),
        diagnostics: Some(false),
    };
    match primitives
        .latest_grpc(&primitives_projected_grpc_request)
        .await
    {
        Err(SdkError::UnsupportedOrUnprovedUsage { .. }) => record_pass(
            report,
            "primitives.projected_grpc_rejection",
            "projected gRPC request rejected before transport",
        ),
        Err(error) => record_fail(
            report,
            "primitives.projected_grpc_rejection",
            error.to_string(),
        ),
        Ok(_) => record_fail(
            report,
            "primitives.projected_grpc_rejection",
            "projected gRPC request unexpectedly succeeded",
        ),
    }

    let primitives_latest_grpc_request =
        primitives_system::LatestGrpcRequest::from(&primitives_latest_request);
    match primitives
        .latest_grpc(&primitives_latest_grpc_request)
        .await
    {
        Ok(out) if !out.rows.is_empty() => {
            let mode = primitive_mode_from_selectors_and_metadata(
                primitives_latest_grpc_request.family.as_deref(),
                primitives_latest_grpc_request.group.as_deref(),
                primitives_latest_grpc_request.metadata,
            );
            match async {
                let direct = raw_grpc_unary::<
                    primitives_proto::LatestOutputsRequestV1,
                    primitives_proto::OutputsLatestResponseV1,
                >(
                    MathildePublicHosts::PRIMITIVES_GRPC,
                    OUTPUTS_LATEST_GPRC_PATH,
                    runtime.bearer_token.as_ref(),
                    primitives_latest_grpc_proto(&primitives_latest_grpc_request)?,
                )
                .await?;
                compare_semantic_values(
                    "primitives.latest_grpc",
                    &canonical_primitive_latest_sdk(&out, mode)?,
                    &canonical_primitive_latest_grpc_raw(
                        &direct,
                        mode,
                        primitives_latest_grpc_request.diagnostics.unwrap_or(false),
                    )?,
                )?;
                Ok::<(), String>(())
            }
            .await
            {
                Ok(()) => record_pass(
                    report,
                    "primitives.latest_grpc",
                    format!(
                        "rows={} missing_pairs={} kind={} close_end_ms={} direct_grpc_semantic_match=true",
                        out.rows.len(),
                        out.missing_pairs.len(),
                        mode.label(),
                        out.close_end_ms
                    ),
                ),
                Err(error) => record_fail(report, "primitives.latest_grpc", error),
            }
        }
        Ok(out) => record_fail(
            report,
            "primitives.latest_grpc",
            format!("latest grpc returned rows={}", out.rows.len()),
        ),
        Err(error) => record_fail(report, "primitives.latest_grpc", error.to_string()),
    }

    if primitives_anchor_close_ms > 0 {
        let primitives_range_request = primitives_system::RangeRequest {
            pairs: vec![primitives_target_pair.clone()],
            tf: Timeframe::M1,
            align_mode: Some(AlignMode::Exact),
            close_start: Some(TimeInput::Ms(primitives_anchor_close_ms - 10 * 60_000)),
            cursor: None,
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            limit: Some(5),
            family: Some(vec![PrimitiveProcessorFamily::MovingAverages]),
            group: Some(vec![PrimitiveProcessorGroup::Ema]),
            metadata: Some(false),
            diagnostics: Some(false),
            format: Some(HttpFormat::Json),
        };
        match primitives.range(&primitives_range_request).await {
            Ok(out) if !out.rows.is_empty() => {
                let mode = primitive_mode_from_selectors_and_metadata(
                    primitives_range_request.family.as_deref(),
                    primitives_range_request.group.as_deref(),
                    primitives_range_request.metadata,
                );
                match async {
                    let direct = raw_http_post_json(
                        &runtime.summary.primitives_http_base_url,
                        "/v1/outputs/range",
                        runtime.bearer_token.as_ref(),
                        &primitives_range_http_body(&primitives_range_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "primitives.range",
                        &canonical_primitive_range_sdk(&out, mode)?,
                        &canonical_compute_range_raw(direct, mode)?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "primitives.range",
                        format!(
                            "rows={} kind={} next_cursor={} direct_http_semantic_match=true",
                            out.rows.len(),
                            mode.label(),
                            out.next_cursor().unwrap_or("")
                        ),
                    ),
                    Err(error) => record_fail(report, "primitives.range", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "primitives.range",
                format!("range returned rows={}", out.rows.len()),
            ),
            Err(error) => record_fail(report, "primitives.range", error.to_string()),
        }

        let primitives_range_grpc_request = primitives_system::RangeGrpcRequest {
            pairs: vec![primitives_target_pair.clone()],
            tf: Timeframe::M1,
            align_mode: Some(AlignMode::Exact),
            close_start: Some(TimeInput::Ms(primitives_anchor_close_ms - 10 * 60_000)),
            cursor: None,
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            limit: Some(5),
            family: None,
            group: None,
            metadata: Some(true),
            diagnostics: Some(true),
        };
        match primitives.range_grpc(&primitives_range_grpc_request).await {
            Ok(out) if !out.rows.is_empty() => {
                let mode = primitive_mode_from_selectors_and_metadata(
                    primitives_range_grpc_request.family.as_deref(),
                    primitives_range_grpc_request.group.as_deref(),
                    primitives_range_grpc_request.metadata,
                );
                match async {
                    let direct = raw_grpc_unary::<
                        primitives_proto::RangeOutputsRequestV1,
                        primitives_proto::OutputsRangeResponseV1,
                    >(
                        MathildePublicHosts::PRIMITIVES_GRPC,
                        OUTPUTS_RANGE_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        primitives_range_grpc_proto(&primitives_range_grpc_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "primitives.range_grpc",
                        &canonical_primitive_range_sdk(&out, mode)?,
                        &canonical_primitive_range_grpc_raw(
                            &direct,
                            mode,
                            primitives_range_grpc_request.diagnostics.unwrap_or(false),
                        )?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "primitives.range_grpc",
                        format!(
                            "rows={} kind={} next_cursor={} direct_grpc_semantic_match=true",
                            out.rows.len(),
                            mode.label(),
                            out.next_cursor().unwrap_or("")
                        ),
                    ),
                    Err(error) => record_fail(report, "primitives.range_grpc", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "primitives.range_grpc",
                format!("range grpc returned rows={}", out.rows.len()),
            ),
            Err(error) => record_fail(report, "primitives.range_grpc", error.to_string()),
        }

        let primitives_predicate = format!("{primitives_target_pair}.c > 0");
        let primitives_search_request = primitives_system::SearchRequest {
            tf: Timeframe::M1,
            close_start: TimeInput::Ms(primitives_anchor_close_ms - 60 * 60_000),
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            cursor: None,
            predicate: primitives_predicate.clone(),
            evaluate_pair: Some(primitives_target_pair.clone()),
            family: None,
            group: None,
            metadata: Some(true),
            diagnostics: Some(true),
            max_hits: Some(5),
            format: Some(HttpFormat::Json),
        };
        match primitives.search(&primitives_search_request).await {
            Ok(out)
                if !out.hits.is_empty()
                    && out
                        .evaluated_rows
                        .as_ref()
                        .is_some_and(|rows| !rows.is_empty()) =>
            {
                let mode = primitive_mode_from_selectors_and_metadata(
                    primitives_search_request.family.as_deref(),
                    primitives_search_request.group.as_deref(),
                    primitives_search_request.metadata,
                );
                match async {
                    let direct = raw_http_post_json(
                        &runtime.summary.primitives_http_base_url,
                        "/v1/outputs/search",
                        runtime.bearer_token.as_ref(),
                        &primitives_search_http_body(&primitives_search_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "primitives.search",
                        &canonical_primitive_search_sdk(&out, mode)?,
                        &canonical_compute_search_raw(direct, mode)?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "primitives.search",
                        format!(
                            "hits={} evaluated_rows={} kind={} direct_http_semantic_match=true",
                            out.hits.len(),
                            out.evaluated_rows
                                .as_ref()
                                .map(|rows| rows.len())
                                .unwrap_or(0),
                            mode.label()
                        ),
                    ),
                    Err(error) => record_fail(report, "primitives.search", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "primitives.search",
                format!(
                    "search returned hits={} evaluated_rows={}",
                    out.hits.len(),
                    out.evaluated_rows
                        .as_ref()
                        .map(|rows| rows.len())
                        .unwrap_or(0)
                ),
            ),
            Err(error) => record_fail(report, "primitives.search", error.to_string()),
        }

        let primitives_search_grpc_request = primitives_system::SearchGrpcRequest {
            tf: Timeframe::M1,
            close_start: TimeInput::Ms(primitives_anchor_close_ms - 60 * 60_000),
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            cursor: None,
            predicate: primitives_predicate,
            evaluate_pair: Some(primitives_target_pair.clone()),
            family: None,
            group: None,
            metadata: Some(false),
            diagnostics: Some(false),
            max_hits: Some(5),
        };
        match primitives
            .search_grpc(&primitives_search_grpc_request)
            .await
        {
            Ok(out)
                if !out.hits.is_empty()
                    && out
                        .evaluated_rows
                        .as_ref()
                        .is_some_and(|rows| !rows.is_empty()) =>
            {
                let mode = primitive_mode_from_selectors_and_metadata(
                    primitives_search_grpc_request.family.as_deref(),
                    primitives_search_grpc_request.group.as_deref(),
                    primitives_search_grpc_request.metadata,
                );
                match async {
                    let direct = raw_grpc_unary::<
                        primitives_proto::SearchOutputsRequestV1,
                        primitives_proto::OutputsSearchResponseV1,
                    >(
                        MathildePublicHosts::PRIMITIVES_GRPC,
                        OUTPUTS_SEARCH_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        primitives_search_grpc_proto(&primitives_search_grpc_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "primitives.search_grpc",
                        &canonical_primitive_search_sdk(&out, mode)?,
                        &canonical_primitive_search_grpc_raw(
                            &direct,
                            mode,
                            primitives_search_grpc_request.diagnostics.unwrap_or(false),
                            primitives_search_grpc_request.evaluate_pair.is_some(),
                        )?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "primitives.search_grpc",
                        format!(
                            "hits={} evaluated_rows={} kind={} direct_grpc_semantic_match=true",
                            out.hits.len(),
                            out.evaluated_rows
                                .as_ref()
                                .map(|rows| rows.len())
                                .unwrap_or(0),
                            mode.label()
                        ),
                    ),
                    Err(error) => record_fail(report, "primitives.search_grpc", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "primitives.search_grpc",
                format!(
                    "search grpc returned hits={} evaluated_rows={}",
                    out.hits.len(),
                    out.evaluated_rows
                        .as_ref()
                        .map(|rows| rows.len())
                        .unwrap_or(0)
                ),
            ),
            Err(error) => record_fail(report, "primitives.search_grpc", error.to_string()),
        }

        let primitives_time_machine_request = primitives_system::TimeMachineRequest {
            tf: Timeframe::M1,
            close_start: TimeInput::Ms(primitives_anchor_close_ms - 20 * 60_000),
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            cursor: None,
            predicate: Some(format!("{primitives_target_pair}.c > 0")),
            hits: None,
            output_pairs: Some(vec![primitives_target_pair.clone()]),
            family: Some(vec![PrimitiveProcessorFamily::MovingAverages]),
            group: Some(vec![PrimitiveProcessorGroup::Ema]),
            metadata: Some(true),
            diagnostics: Some(false),
            before_bars: Some(2),
            after_bars: Some(2),
            max_hits: Some(10),
            overlap_mode: Some("merge".to_string()),
            format: Some(HttpFormat::Json),
        };
        match primitives
            .time_machine(&primitives_time_machine_request)
            .await
        {
            Ok(out) if !out.rows.is_empty() => {
                let mode = primitive_mode_from_selectors_and_metadata(
                    primitives_time_machine_request.family.as_deref(),
                    primitives_time_machine_request.group.as_deref(),
                    primitives_time_machine_request.metadata,
                );
                match async {
                    let direct = raw_http_post_json(
                        &runtime.summary.primitives_http_base_url,
                        "/v1/outputs/time-machine",
                        runtime.bearer_token.as_ref(),
                        &primitives_time_machine_http_body(&primitives_time_machine_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "primitives.time_machine",
                        &canonical_primitive_time_machine_sdk(&out, mode)?,
                        &canonical_compute_time_machine_raw(direct, mode)?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "primitives.time_machine",
                        format!(
                            "rows={} kind={} done={} direct_http_semantic_match=true",
                            out.rows.len(),
                            mode.label(),
                            out.done()
                        ),
                    ),
                    Err(error) => record_fail(report, "primitives.time_machine", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "primitives.time_machine",
                format!("time-machine returned rows={}", out.rows.len()),
            ),
            Err(error) => record_fail(report, "primitives.time_machine", error.to_string()),
        }

        let primitives_time_machine_grpc_request = primitives_system::TimeMachineGrpcRequest {
            tf: Timeframe::M1,
            close_start: TimeInput::Ms(primitives_anchor_close_ms - 20 * 60_000),
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            cursor: None,
            predicate: Some(format!("{primitives_target_pair}.c > 0")),
            hits: None,
            output_pairs: Some(vec![primitives_target_pair.clone()]),
            family: None,
            group: None,
            metadata: Some(true),
            diagnostics: Some(false),
            before_bars: Some(2),
            after_bars: Some(2),
            max_hits: Some(10),
            overlap_mode: Some("merge".to_string()),
        };
        match primitives
            .time_machine_grpc(&primitives_time_machine_grpc_request)
            .await
        {
            Ok(out) if !out.rows.is_empty() => {
                let mode = primitive_mode_from_selectors_and_metadata(
                    primitives_time_machine_grpc_request.family.as_deref(),
                    primitives_time_machine_grpc_request.group.as_deref(),
                    primitives_time_machine_grpc_request.metadata,
                );
                match async {
                    let direct = raw_grpc_unary::<
                        primitives_proto::TimeMachineOutputsRequestV1,
                        primitives_proto::OutputsTimeMachineResponseV1,
                    >(
                        MathildePublicHosts::PRIMITIVES_GRPC,
                        OUTPUTS_TIME_MACHINE_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        primitives_time_machine_grpc_proto(&primitives_time_machine_grpc_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "primitives.time_machine_grpc",
                        &canonical_primitive_time_machine_sdk(&out, mode)?,
                        &canonical_primitive_time_machine_grpc_raw(
                            &direct,
                            mode,
                            primitives_time_machine_grpc_request
                                .diagnostics
                                .unwrap_or(false),
                        )?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "primitives.time_machine_grpc",
                        format!(
                            "rows={} kind={} done={} direct_grpc_semantic_match=true",
                            out.rows.len(),
                            mode.label(),
                            out.done()
                        ),
                    ),
                    Err(error) => record_fail(report, "primitives.time_machine_grpc", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "primitives.time_machine_grpc",
                format!("time-machine grpc returned rows={}", out.rows.len()),
            ),
            Err(error) => record_fail(report, "primitives.time_machine_grpc", error.to_string()),
        }
    } else {
        for surface in [
            "primitives.range",
            "primitives.range_grpc",
            "primitives.search",
            "primitives.search_grpc",
            "primitives.time_machine",
            "primitives.time_machine_grpc",
        ] {
            record_fail(report, surface, "latest anchor was not established");
        }
    }

    match regime.docs_system().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "regime.docs_system",
                format!(
                    "subsystem={} keys={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "regime.docs_system",
            "docs_system was not a JSON object",
        ),
        Err(error) => record_fail(report, "regime.docs_system", error.to_string()),
    }

    match regime.docs_summary().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "regime.docs_summary",
                format!(
                    "subsystem={} keys={}",
                    json_str(&out, "subsystem").unwrap_or(""),
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "regime.docs_summary",
            "docs_summary was not a JSON object",
        ),
        Err(error) => record_fail(report, "regime.docs_summary", error.to_string()),
    }

    match regime.docs_taxonomy().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "regime.docs_taxonomy",
                format!(
                    "keys={}",
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "regime.docs_taxonomy",
            "docs_taxonomy was not a JSON object",
        ),
        Err(error) => record_fail(report, "regime.docs_taxonomy", error.to_string()),
    }

    match regime
        .docs_registry(&regime_system::DocsRegistryRequest::default())
        .await
    {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "regime.docs_registry",
                format!(
                    "keys={}",
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "regime.docs_registry",
            "docs_registry was not a JSON object",
        ),
        Err(error) => record_fail(report, "regime.docs_registry", error.to_string()),
    }

    match regime.docs_endpoints().await {
        Ok(out) if out.is_object() => {
            record_pass(
                report,
                "regime.docs_endpoints",
                format!(
                    "keys={}",
                    out.as_object().map(|value| value.len()).unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "regime.docs_endpoints",
            "docs_endpoints was not a JSON object",
        ),
        Err(error) => record_fail(report, "regime.docs_endpoints", error.to_string()),
    }

    match regime.openapi().await {
        Ok(out) if out.get("openapi").is_some() => {
            record_pass(
                report,
                "regime.openapi",
                format!(
                    "openapi={} paths={}",
                    json_str(&out, "openapi").unwrap_or(""),
                    out.get("paths")
                        .and_then(|value| value.as_object())
                        .map(|value| value.len())
                        .unwrap_or(0)
                ),
            );
        }
        Ok(_) => record_fail(
            report,
            "regime.openapi",
            "openapi document missing `openapi` key",
        ),
        Err(error) => record_fail(report, "regime.openapi", error.to_string()),
    }

    let regime_pairs_list_out = match regime
        .pairs_list(&regime_system::PairsListRequest {
            after_pair: None,
            limit: Some(5),
            enabled_only: Some(true),
        })
        .await
    {
        Ok(out) if !out.pairs.is_empty() => {
            record_pass(
                report,
                "regime.pairs_list",
                format!("pairs={}", out.pairs.len()),
            );
            Some(out)
        }
        Ok(out) => {
            record_fail(
                report,
                "regime.pairs_list",
                format!("pairs={}", out.pairs.len()),
            );
            None
        }
        Err(error) => {
            record_fail(report, "regime.pairs_list", error.to_string());
            None
        }
    };

    let regime_target_pair = regime_pairs_list_out
        .as_ref()
        .and_then(|out| out.pairs.first())
        .cloned()
        .unwrap_or_else(|| "BTCUSDT".to_string());

    match regime
        .pairs_status(&regime_system::PairsStatusRequest {
            after_pair: None,
            limit: Some(1),
            pairs: Some(vec![regime_target_pair.clone()]),
            filters: Some(vec!["status".to_string(), "readiness".to_string()]),
        })
        .await
    {
        Ok(out) if !out.pairs.is_empty() => {
            record_pass(
                report,
                "regime.pairs_status",
                format!("pair={} rows={}", out.pairs[0].pair, out.pairs.len()),
            );
        }
        Ok(_) => record_fail(report, "regime.pairs_status", "empty pairs status response"),
        Err(error) => record_fail(report, "regime.pairs_status", error.to_string()),
    }

    match regime
        .files_downloads(&regime_system::FilesDownloadsRequest {
            period: Some("day".to_string()),
            pairs: vec![regime_target_pair.clone()],
            tfs: vec!["1h".to_string()],
            start_label_utc: None,
            end_label_utc: None,
            order: Some("desc".to_string()),
        })
        .await
    {
        Ok(out)
            if !out.rows.is_empty()
                && out.rows[0].url.starts_with("http")
                && !out.rows[0].expires_at_utc.trim().is_empty() =>
        {
            record_pass(
                report,
                "regime.files_downloads",
                format!(
                    "rows={} first_pair={} expires_at_utc={}",
                    out.rows.len(),
                    out.rows[0].pair,
                    out.rows[0].expires_at_utc
                ),
            );
        }
        Ok(out) => record_fail(
            report,
            "regime.files_downloads",
            format!("unexpected rows shape; rows={}", out.rows.len()),
        ),
        Err(error) => record_fail(report, "regime.files_downloads", error.to_string()),
    }

    let regime_latest_request = regime_system::LatestRequest {
        pairs: vec![regime_target_pair.clone(), "ETHUSDT".to_string()],
        tf: Timeframe::H1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        secondary: Some(false),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Json),
    };
    let regime_anchor_close_ms = match regime.latest(&regime_latest_request).await {
        Ok(out) if !out.rows.is_empty() => {
            let mode = regime_mode_from_selectors_secondary_and_metadata(
                regime_latest_request.family.as_deref(),
                regime_latest_request.group.as_deref(),
                regime_latest_request.secondary,
                regime_latest_request.metadata,
            );
            match async {
                let direct = raw_http_post_json(
                    &runtime.summary.regime_http_base_url,
                    "/v1/outputs/latest",
                    runtime.bearer_token.as_ref(),
                    &regime_latest_http_body(&regime_latest_request)?,
                )
                .await?;
                compare_semantic_values(
                    "regime.latest",
                    &canonical_regime_latest_sdk(&out, mode)?,
                    &canonical_compute_latest_raw(direct, mode)?,
                )?;
                Ok::<i64, String>(out.close_end_ms)
            }
            .await
            {
                Ok(close_end_ms) => {
                    record_pass(
                        report,
                        "regime.latest",
                        format!(
                            "rows={} missing_pairs={} kind={} close_end_ms={} direct_http_semantic_match=true",
                            out.rows.len(),
                            out.missing_pairs.len(),
                            regime_output_kind(mode),
                            close_end_ms
                        ),
                    );
                    close_end_ms
                }
                Err(error) => {
                    record_fail(report, "regime.latest", error);
                    0
                }
            }
        }
        Ok(out) => {
            record_fail(
                report,
                "regime.latest",
                format!("latest returned rows={}", out.rows.len()),
            );
            0
        }
        Err(error) => {
            record_fail(report, "regime.latest", error.to_string());
            0
        }
    };

    let regime_projected_http_protobuf_request = regime_system::LatestRequest {
        pairs: vec![regime_target_pair.clone()],
        tf: Timeframe::H1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        secondary: Some(false),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Protobuf),
    };
    match regime.latest(&regime_projected_http_protobuf_request).await {
        Err(SdkError::UnsupportedOrUnprovedUsage { .. }) => record_pass(
            report,
            "regime.projected_http_protobuf_rejection",
            "projected HTTP protobuf request rejected before transport",
        ),
        Err(error) => record_fail(
            report,
            "regime.projected_http_protobuf_rejection",
            error.to_string(),
        ),
        Ok(_) => record_fail(
            report,
            "regime.projected_http_protobuf_rejection",
            "projected HTTP protobuf request unexpectedly succeeded",
        ),
    }

    let regime_projected_grpc_request = regime_system::LatestGrpcRequest {
        pairs: vec![regime_target_pair.clone()],
        tf: Timeframe::H1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        secondary: Some(false),
        metadata: Some(false),
        diagnostics: Some(false),
    };
    match regime.latest_grpc(&regime_projected_grpc_request).await {
        Err(SdkError::UnsupportedOrUnprovedUsage { .. }) => record_pass(
            report,
            "regime.projected_grpc_rejection",
            "projected gRPC request rejected before transport",
        ),
        Err(error) => record_fail(report, "regime.projected_grpc_rejection", error.to_string()),
        Ok(_) => record_fail(
            report,
            "regime.projected_grpc_rejection",
            "projected gRPC request unexpectedly succeeded",
        ),
    }

    let regime_non_h1_http_request = regime_system::LatestRequest {
        pairs: vec![regime_target_pair.clone()],
        tf: Timeframe::M1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        secondary: Some(true),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Json),
    };
    match regime.latest(&regime_non_h1_http_request).await {
        Err(SdkError::UnsupportedOrUnprovedUsage { message }) if message.contains("tf=1h") => {
            record_pass(
                report,
                "regime.non_h1_http_rejection",
                "non-h1 HTTP request rejected before transport",
            )
        }
        Err(error) => record_fail(report, "regime.non_h1_http_rejection", error.to_string()),
        Ok(_) => record_fail(
            report,
            "regime.non_h1_http_rejection",
            "non-h1 HTTP request unexpectedly succeeded",
        ),
    }

    let regime_non_h1_grpc_request = regime_system::LatestGrpcRequest {
        pairs: vec![regime_target_pair.clone()],
        tf: Timeframe::M1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        secondary: Some(true),
        metadata: Some(false),
        diagnostics: Some(false),
    };
    match regime.latest_grpc(&regime_non_h1_grpc_request).await {
        Err(SdkError::UnsupportedOrUnprovedUsage { message }) if message.contains("tf=1h") => {
            record_pass(
                report,
                "regime.non_h1_grpc_rejection",
                "non-h1 gRPC request rejected before transport",
            )
        }
        Err(error) => record_fail(report, "regime.non_h1_grpc_rejection", error.to_string()),
        Ok(_) => record_fail(
            report,
            "regime.non_h1_grpc_rejection",
            "non-h1 gRPC request unexpectedly succeeded",
        ),
    }

    let regime_latest_grpc_request = regime_system::LatestGrpcRequest {
        pairs: vec![regime_target_pair.clone(), "ETHUSDT".to_string()],
        tf: Timeframe::H1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        secondary: Some(true),
        metadata: Some(false),
        diagnostics: Some(false),
    };
    match regime.latest_grpc(&regime_latest_grpc_request).await {
        Ok(out) if !out.rows.is_empty() => {
            let mode = regime_mode_from_selectors_secondary_and_metadata(
                regime_latest_grpc_request.family.as_deref(),
                regime_latest_grpc_request.group.as_deref(),
                regime_latest_grpc_request.secondary,
                regime_latest_grpc_request.metadata,
            );
            match async {
                let direct = raw_grpc_unary::<
                    regime_proto::LatestOutputsRequestV1,
                    regime_proto::OutputsLatestResponseV1,
                >(
                    MathildePublicHosts::REGIME_GRPC,
                    OUTPUTS_LATEST_GPRC_PATH,
                    runtime.bearer_token.as_ref(),
                    regime_latest_grpc_proto(&regime_latest_grpc_request)?,
                )
                .await?;
                compare_semantic_values(
                    "regime.latest_grpc",
                    &canonical_regime_latest_sdk(&out, mode)?,
                    &canonical_regime_latest_grpc_raw(
                        &direct,
                        mode,
                        regime_latest_grpc_request.diagnostics.unwrap_or(false),
                    )?,
                )?;
                Ok::<(), String>(())
            }
            .await
            {
                Ok(()) => record_pass(
                    report,
                    "regime.latest_grpc",
                    format!(
                        "rows={} missing_pairs={} kind={} close_end_ms={} direct_grpc_semantic_match=true",
                        out.rows.len(),
                        out.missing_pairs.len(),
                        regime_output_kind(mode),
                        out.close_end_ms
                    ),
                ),
                Err(error) => record_fail(report, "regime.latest_grpc", error),
            }
        }
        Ok(out) => record_fail(
            report,
            "regime.latest_grpc",
            format!("latest grpc returned rows={}", out.rows.len()),
        ),
        Err(error) => record_fail(report, "regime.latest_grpc", error.to_string()),
    }

    if regime_anchor_close_ms > 0 {
        let regime_range_request = regime_system::RangeRequest {
            pairs: vec![regime_target_pair.clone()],
            tf: Timeframe::H1,
            align_mode: Some(AlignMode::Exact),
            close_start: Some(TimeInput::Ms(regime_anchor_close_ms - 10 * 3_600_000)),
            cursor: None,
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            limit: Some(5),
            family: None,
            group: None,
            secondary: Some(false),
            metadata: Some(false),
            diagnostics: Some(false),
            format: Some(HttpFormat::Json),
        };
        match regime.range(&regime_range_request).await {
            Ok(out) if !out.rows.is_empty() => {
                let mode = regime_mode_from_selectors_secondary_and_metadata(
                    regime_range_request.family.as_deref(),
                    regime_range_request.group.as_deref(),
                    regime_range_request.secondary,
                    regime_range_request.metadata,
                );
                match async {
                    let direct = raw_http_post_json(
                        &runtime.summary.regime_http_base_url,
                        "/v1/outputs/range",
                        runtime.bearer_token.as_ref(),
                        &regime_range_http_body(&regime_range_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "regime.range",
                        &canonical_regime_range_sdk(&out, mode)?,
                        &canonical_compute_range_raw(direct, mode)?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "regime.range",
                        format!(
                            "rows={} kind={} next_cursor={} direct_http_semantic_match=true",
                            out.rows.len(),
                            regime_output_kind(mode),
                            out.next_cursor().unwrap_or("")
                        ),
                    ),
                    Err(error) => record_fail(report, "regime.range", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "regime.range",
                format!("range returned rows={}", out.rows.len()),
            ),
            Err(error) => record_fail(report, "regime.range", error.to_string()),
        }

        let regime_range_grpc_request = regime_system::RangeGrpcRequest {
            pairs: vec![regime_target_pair.clone()],
            tf: Timeframe::H1,
            align_mode: Some(AlignMode::Exact),
            close_start: Some(TimeInput::Ms(regime_anchor_close_ms - 10 * 3_600_000)),
            cursor: None,
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            limit: Some(5),
            family: None,
            group: None,
            secondary: Some(true),
            metadata: Some(true),
            diagnostics: Some(true),
        };
        match regime.range_grpc(&regime_range_grpc_request).await {
            Ok(out) if !out.rows.is_empty() => {
                let mode = regime_mode_from_selectors_secondary_and_metadata(
                    regime_range_grpc_request.family.as_deref(),
                    regime_range_grpc_request.group.as_deref(),
                    regime_range_grpc_request.secondary,
                    regime_range_grpc_request.metadata,
                );
                match async {
                    let direct = raw_grpc_unary::<
                        regime_proto::RangeOutputsRequestV1,
                        regime_proto::OutputsRangeResponseV1,
                    >(
                        MathildePublicHosts::REGIME_GRPC,
                        OUTPUTS_RANGE_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        regime_range_grpc_proto(&regime_range_grpc_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "regime.range_grpc",
                        &canonical_regime_range_sdk(&out, mode)?,
                        &canonical_regime_range_grpc_raw(
                            &direct,
                            mode,
                            regime_range_grpc_request.diagnostics.unwrap_or(false),
                        )?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "regime.range_grpc",
                        format!(
                            "rows={} kind={} next_cursor={} direct_grpc_semantic_match=true",
                            out.rows.len(),
                            regime_output_kind(mode),
                            out.next_cursor().unwrap_or("")
                        ),
                    ),
                    Err(error) => record_fail(report, "regime.range_grpc", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "regime.range_grpc",
                format!("range grpc returned rows={}", out.rows.len()),
            ),
            Err(error) => record_fail(report, "regime.range_grpc", error.to_string()),
        }

        let regime_predicate = format!("{regime_target_pair}.c > 0");
        let regime_search_request = regime_system::SearchRequest {
            tf: Timeframe::H1,
            close_start: TimeInput::Ms(regime_anchor_close_ms - 24 * 3_600_000),
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            cursor: None,
            predicate: regime_predicate.clone(),
            evaluate_pair: Some(regime_target_pair.clone()),
            family: None,
            group: None,
            secondary: Some(true),
            metadata: Some(true),
            diagnostics: Some(true),
            max_hits: Some(5),
            format: Some(HttpFormat::Json),
        };
        match regime.search(&regime_search_request).await {
            Ok(out)
                if !out.hits.is_empty()
                    && out
                        .evaluated_rows
                        .as_ref()
                        .is_some_and(|rows| !rows.is_empty()) =>
            {
                let mode = regime_mode_from_selectors_secondary_and_metadata(
                    regime_search_request.family.as_deref(),
                    regime_search_request.group.as_deref(),
                    regime_search_request.secondary,
                    regime_search_request.metadata,
                );
                match async {
                    let direct = raw_http_post_json(
                        &runtime.summary.regime_http_base_url,
                        "/v1/outputs/search",
                        runtime.bearer_token.as_ref(),
                        &regime_search_http_body(&regime_search_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "regime.search",
                        &canonical_regime_search_sdk(&out, mode)?,
                        &canonical_compute_search_raw(direct, mode)?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "regime.search",
                        format!(
                            "hits={} evaluated_rows={} kind={} direct_http_semantic_match=true",
                            out.hits.len(),
                            out.evaluated_rows
                                .as_ref()
                                .map(|rows| rows.len())
                                .unwrap_or(0),
                            regime_output_kind(mode)
                        ),
                    ),
                    Err(error) => record_fail(report, "regime.search", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "regime.search",
                format!(
                    "search returned hits={} evaluated_rows={}",
                    out.hits.len(),
                    out.evaluated_rows
                        .as_ref()
                        .map(|rows| rows.len())
                        .unwrap_or(0)
                ),
            ),
            Err(error) => record_fail(report, "regime.search", error.to_string()),
        }

        let regime_search_grpc_request = regime_system::SearchGrpcRequest {
            tf: Timeframe::H1,
            close_start: TimeInput::Ms(regime_anchor_close_ms - 24 * 3_600_000),
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            cursor: None,
            predicate: regime_predicate,
            evaluate_pair: Some(regime_target_pair.clone()),
            family: None,
            group: None,
            secondary: Some(true),
            metadata: Some(false),
            diagnostics: Some(false),
            max_hits: Some(5),
        };
        match regime.search_grpc(&regime_search_grpc_request).await {
            Ok(out)
                if !out.hits.is_empty()
                    && out
                        .evaluated_rows
                        .as_ref()
                        .is_some_and(|rows| !rows.is_empty()) =>
            {
                let mode = regime_mode_from_selectors_secondary_and_metadata(
                    regime_search_grpc_request.family.as_deref(),
                    regime_search_grpc_request.group.as_deref(),
                    regime_search_grpc_request.secondary,
                    regime_search_grpc_request.metadata,
                );
                match async {
                    let direct = raw_grpc_unary::<
                        regime_proto::SearchOutputsRequestV1,
                        regime_proto::OutputsSearchResponseV1,
                    >(
                        MathildePublicHosts::REGIME_GRPC,
                        OUTPUTS_SEARCH_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        regime_search_grpc_proto(&regime_search_grpc_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "regime.search_grpc",
                        &canonical_regime_search_sdk(&out, mode)?,
                        &canonical_regime_search_grpc_raw(
                            &direct,
                            mode,
                            regime_search_grpc_request.diagnostics.unwrap_or(false),
                            regime_search_grpc_request.evaluate_pair.is_some(),
                        )?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "regime.search_grpc",
                        format!(
                            "hits={} evaluated_rows={} kind={} direct_grpc_semantic_match=true",
                            out.hits.len(),
                            out.evaluated_rows
                                .as_ref()
                                .map(|rows| rows.len())
                                .unwrap_or(0),
                            regime_output_kind(mode)
                        ),
                    ),
                    Err(error) => record_fail(report, "regime.search_grpc", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "regime.search_grpc",
                format!(
                    "search grpc returned hits={} evaluated_rows={}",
                    out.hits.len(),
                    out.evaluated_rows
                        .as_ref()
                        .map(|rows| rows.len())
                        .unwrap_or(0)
                ),
            ),
            Err(error) => record_fail(report, "regime.search_grpc", error.to_string()),
        }

        let regime_time_machine_request = regime_system::TimeMachineRequest {
            tf: Timeframe::H1,
            close_start: TimeInput::Ms(regime_anchor_close_ms - 24 * 3_600_000),
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            cursor: None,
            predicate: Some(format!("{regime_target_pair}.c > 0")),
            hits: None,
            output_pairs: Some(vec![regime_target_pair.clone()]),
            family: None,
            group: None,
            secondary: Some(false),
            metadata: Some(true),
            diagnostics: Some(false),
            before_bars: Some(2),
            after_bars: Some(2),
            max_hits: Some(10),
            overlap_mode: Some("merge".to_string()),
            format: Some(HttpFormat::Json),
        };
        match regime.time_machine(&regime_time_machine_request).await {
            Ok(out) if !out.rows.is_empty() => {
                let mode = regime_mode_from_selectors_secondary_and_metadata(
                    regime_time_machine_request.family.as_deref(),
                    regime_time_machine_request.group.as_deref(),
                    regime_time_machine_request.secondary,
                    regime_time_machine_request.metadata,
                );
                match async {
                    let direct = raw_http_post_json(
                        &runtime.summary.regime_http_base_url,
                        "/v1/outputs/time-machine",
                        runtime.bearer_token.as_ref(),
                        &regime_time_machine_http_body(&regime_time_machine_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "regime.time_machine",
                        &canonical_regime_time_machine_sdk(&out, mode)?,
                        &canonical_compute_time_machine_raw(direct, mode)?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "regime.time_machine",
                        format!(
                            "rows={} kind={} done={} direct_http_semantic_match=true",
                            out.rows.len(),
                            regime_output_kind(mode),
                            out.done()
                        ),
                    ),
                    Err(error) => record_fail(report, "regime.time_machine", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "regime.time_machine",
                format!("time-machine returned rows={}", out.rows.len()),
            ),
            Err(error) => record_fail(report, "regime.time_machine", error.to_string()),
        }

        let regime_time_machine_grpc_request = regime_system::TimeMachineGrpcRequest {
            tf: Timeframe::H1,
            close_start: TimeInput::Ms(regime_anchor_close_ms - 24 * 3_600_000),
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            cursor: None,
            predicate: Some(format!("{regime_target_pair}.c > 0")),
            hits: None,
            output_pairs: Some(vec![regime_target_pair.clone()]),
            family: None,
            group: None,
            secondary: Some(true),
            metadata: Some(true),
            diagnostics: Some(false),
            before_bars: Some(2),
            after_bars: Some(2),
            max_hits: Some(10),
            overlap_mode: Some("merge".to_string()),
        };
        match regime
            .time_machine_grpc(&regime_time_machine_grpc_request)
            .await
        {
            Ok(out) if !out.rows.is_empty() => {
                let mode = regime_mode_from_selectors_secondary_and_metadata(
                    regime_time_machine_grpc_request.family.as_deref(),
                    regime_time_machine_grpc_request.group.as_deref(),
                    regime_time_machine_grpc_request.secondary,
                    regime_time_machine_grpc_request.metadata,
                );
                match async {
                    let direct = raw_grpc_unary::<
                        regime_proto::TimeMachineOutputsRequestV1,
                        regime_proto::OutputsTimeMachineResponseV1,
                    >(
                        MathildePublicHosts::REGIME_GRPC,
                        OUTPUTS_TIME_MACHINE_GPRC_PATH,
                        runtime.bearer_token.as_ref(),
                        regime_time_machine_grpc_proto(&regime_time_machine_grpc_request)?,
                    )
                    .await?;
                    compare_semantic_values(
                        "regime.time_machine_grpc",
                        &canonical_regime_time_machine_sdk(&out, mode)?,
                        &canonical_regime_time_machine_grpc_raw(
                            &direct,
                            mode,
                            regime_time_machine_grpc_request
                                .diagnostics
                                .unwrap_or(false),
                        )?,
                    )?;
                    Ok::<(), String>(())
                }
                .await
                {
                    Ok(()) => record_pass(
                        report,
                        "regime.time_machine_grpc",
                        format!(
                            "rows={} kind={} done={} direct_grpc_semantic_match=true",
                            out.rows.len(),
                            regime_output_kind(mode),
                            out.done()
                        ),
                    ),
                    Err(error) => record_fail(report, "regime.time_machine_grpc", error),
                }
            }
            Ok(out) => record_fail(
                report,
                "regime.time_machine_grpc",
                format!("time-machine grpc returned rows={}", out.rows.len()),
            ),
            Err(error) => record_fail(report, "regime.time_machine_grpc", error.to_string()),
        }
    } else {
        for surface in [
            "regime.range",
            "regime.range_grpc",
            "regime.search",
            "regime.search_grpc",
            "regime.time_machine",
            "regime.time_machine_grpc",
        ] {
            record_fail(report, surface, "latest anchor was not established");
        }
    }
}

async fn run_phase_6_ws_downloads_pagination_and_parity(
    runtime: &RuntimeConfig,
    report: &mut Report,
) {
    let aggregator = &runtime.aggregator;
    let primitives = &runtime.primitives;
    let regime = &runtime.regime;

    let aggregator_target_pair = aggregator
        .pairs_list(&PairsListRequest {
            after_pair: None,
            limit: Some(1),
            enabled_only: Some(true),
        })
        .await
        .ok()
        .and_then(|out| out.pairs.first().cloned())
        .unwrap_or_else(|| "BTCUSDT".to_string());

    match aggregator
        .files_downloads(&FilesDownloadsRequest {
            period: Some("day".to_string()),
            pairs: vec![aggregator_target_pair.clone()],
            tfs: vec!["1m".to_string()],
            start_label_utc: None,
            end_label_utc: None,
            order: Some("desc".to_string()),
        })
        .await
    {
        Ok(out) if !out.rows.is_empty() => {
            let root = download_root_for("aggregator");
            if let Err(error) = fs::create_dir_all(&root) {
                record_fail(
                    report,
                    "aggregator.files_download_items",
                    format!("create_dir_all failed for {}: {error}", root.display()),
                );
            } else {
                match aggregator
                    .files_download_items(&[out.rows[0].clone()], Some(root.as_path()))
                    .await
                {
                    Ok(downloaded)
                        if !downloaded.is_empty()
                            && downloaded[0].bytes_written > 0
                            && Path::new(&downloaded[0].destination_path).exists() =>
                    {
                        record_pass(
                            report,
                            "aggregator.files_download_items",
                            format!(
                                "bytes_written={} path={}",
                                downloaded[0].bytes_written, downloaded[0].destination_path
                            ),
                        );
                    }
                    Ok(downloaded) => record_fail(
                        report,
                        "aggregator.files_download_items",
                        format!("unexpected download result count={}", downloaded.len()),
                    ),
                    Err(error) => {
                        record_fail(report, "aggregator.files_download_items", error.to_string())
                    }
                }
            }
        }
        Ok(_) => record_fail(
            report,
            "aggregator.files_download_items",
            "no downloadable rows available",
        ),
        Err(error) => record_fail(report, "aggregator.files_download_items", error.to_string()),
    }

    if runtime.summary.aggregator_ws_base_url.is_some() {
        let aggregator_ws_request = BarsWsSubscribeRequest {
            pairs: vec![aggregator_target_pair.clone()],
            tfs: vec![Timeframe::M1],
            metadata: Some(false),
            from_close: None,
            last_n_bars: Some(1),
            format: None,
        };

        match aggregator.connect_bars_ws(&aggregator_ws_request).await {
            Ok(mut connection) => {
                let started = tokio::time::Instant::now();
                let mut saw_payload = false;
                let mut saw_replay_done = false;
                let mut frame_kinds = Vec::new();
                loop {
                    let elapsed = started.elapsed();
                    if elapsed >= ws_replay_timeout() {
                        record_fail(
                            report,
                            "aggregator.connect_bars_ws",
                            format!(
                                "timed out waiting for replay rows + replay_done; observed={}",
                                frame_kinds.join(",")
                            ),
                        );
                        break;
                    }

                    match timeout(
                        ws_replay_timeout() - elapsed,
                        connection.next_frame(&aggregator_ws_request),
                    )
                    .await
                    {
                        Ok(Ok(Some(frame))) => {
                            frame_kinds.push(aggregator_bars_ws_frame_kind(&frame).to_string());
                            if aggregator_bars_ws_is_payload(&frame) {
                                saw_payload = true;
                            }
                            if let BarsWsInboundFrame::Meta(meta) = &frame {
                                if meta.is_replay_done() {
                                    saw_replay_done = true;
                                }
                            }
                            if saw_payload && saw_replay_done {
                                record_pass(
                                    report,
                                    "aggregator.connect_bars_ws",
                                    format!(
                                        "replay rows + replay_done observed within 60s; frames={}",
                                        frame_kinds.join(",")
                                    ),
                                );
                                break;
                            }
                        }
                        Ok(Ok(None)) => {
                            record_fail(
                                report,
                                "aggregator.connect_bars_ws",
                                "ws stream closed before replay rows + replay_done were observed",
                            );
                            break;
                        }
                        Ok(Err(error)) => {
                            record_fail(report, "aggregator.connect_bars_ws", error.to_string());
                            break;
                        }
                        Err(_) => {
                            record_fail(
                                report,
                                "aggregator.connect_bars_ws",
                                "timed out waiting for replay rows + replay_done",
                            );
                            break;
                        }
                    }
                }
            }
            Err(error) => record_fail(report, "aggregator.connect_bars_ws", error.to_string()),
        }

        let aggregator_swap_pair = if aggregator_target_pair == "ETHUSDT" {
            "BTCUSDT".to_string()
        } else {
            "ETHUSDT".to_string()
        };
        let aggregator_new_ws_request = BarsWsSubscribeRequest {
            pairs: vec![aggregator_swap_pair],
            tfs: vec![Timeframe::M1],
            metadata: Some(false),
            from_close: None,
            last_n_bars: Some(1),
            format: None,
        };
        match aggregator
            .connect_bars_ws_make_before_break(&aggregator_ws_request)
            .await
        {
            Ok(mut connection) => {
                if let Err(error) = connection.begin_swap(&aggregator_new_ws_request) {
                    record_fail(
                        report,
                        "aggregator.connect_bars_ws_make_before_break",
                        error.to_string(),
                    );
                } else {
                    match timeout(ws_timeout(), connection.next_frame()).await {
                        Ok(Ok(Some(frame))) => record_pass(
                            report,
                            "aggregator.connect_bars_ws_make_before_break",
                            format!(
                                "frame_kind={} swap_in_progress={} active_pairs={}",
                                aggregator_bars_ws_frame_kind(&frame),
                                connection.swap_in_progress(),
                                connection.active_request().pairs.join(",")
                            ),
                        ),
                        Ok(Ok(None)) => record_fail(
                            report,
                            "aggregator.connect_bars_ws_make_before_break",
                            "ws stream closed before first frame",
                        ),
                        Ok(Err(error)) => record_fail(
                            report,
                            "aggregator.connect_bars_ws_make_before_break",
                            error.to_string(),
                        ),
                        Err(_) => record_fail(
                            report,
                            "aggregator.connect_bars_ws_make_before_break",
                            "timed out waiting for make-before-break frame",
                        ),
                    }
                }
            }
            Err(error) => record_fail(
                report,
                "aggregator.connect_bars_ws_make_before_break",
                error.to_string(),
            ),
        }

        match aggregator
            .connect_bars_ws_recovering(&aggregator_ws_request, ExponentialBackoffConfig::default())
            .await
        {
            Ok(mut connection) => match timeout(ws_timeout(), connection.next_frame()).await {
                Ok(Ok(Some(frame))) => record_pass(
                    report,
                    "aggregator.connect_bars_ws_recovering",
                    format!(
                        "frame_kind={} next_attempt={} active_pairs={}",
                        aggregator_bars_ws_frame_kind(&frame),
                        connection.next_attempt(),
                        connection.active_request().pairs.join(",")
                    ),
                ),
                Ok(Ok(None)) => record_fail(
                    report,
                    "aggregator.connect_bars_ws_recovering",
                    "ws stream closed before first frame",
                ),
                Ok(Err(error)) => record_fail(
                    report,
                    "aggregator.connect_bars_ws_recovering",
                    error.to_string(),
                ),
                Err(_) => record_fail(
                    report,
                    "aggregator.connect_bars_ws_recovering",
                    "timed out waiting for recovering frame",
                ),
            },
            Err(error) => record_fail(
                report,
                "aggregator.connect_bars_ws_recovering",
                error.to_string(),
            ),
        }

        match aggregator.connect_messages_ws().await {
            Ok(mut connection) => {
                let subscribe = AggregatorMessagesWsSubscribeFrame {
                    id: "sdk-endpoint-test-aggregator".to_string(),
                    tfs: Some(vec![Timeframe::M1]),
                    predicate: format!("{aggregator_target_pair}.c > 0"),
                    message: "endpoint_test".to_string(),
                    payload: None,
                };
                let unsubscribe = AggregatorMessagesWsUnsubscribeFrame {
                    id: "sdk-endpoint-test-aggregator".to_string(),
                };
                if let Err(error) = connection.send_subscribe(&subscribe).await {
                    record_fail(report, "aggregator.connect_messages_ws", error.to_string());
                } else if let Err(error) = connection.send_unsubscribe(&unsubscribe).await {
                    record_fail(report, "aggregator.connect_messages_ws", error.to_string());
                } else {
                    match timeout(ws_timeout(), connection.next_frame()).await {
                        Ok(Ok(Some(frame))) => record_pass(
                            report,
                            "aggregator.connect_messages_ws",
                            format!("frame_kind={}", aggregator_messages_ws_frame_kind(&frame)),
                        ),
                        Ok(Ok(None)) => record_fail(
                            report,
                            "aggregator.connect_messages_ws",
                            "ws stream closed before first server frame",
                        ),
                        Ok(Err(error)) => {
                            record_fail(report, "aggregator.connect_messages_ws", error.to_string())
                        }
                        Err(_) => record_fail(
                            report,
                            "aggregator.connect_messages_ws",
                            "timed out waiting for messages ws frame",
                        ),
                    }
                }
            }
            Err(error) => record_fail(report, "aggregator.connect_messages_ws", error.to_string()),
        }

        match aggregator
            .connect_messages_ws_recovering(ExponentialBackoffConfig::default())
            .await
        {
            Ok(mut connection) => {
                let subscribe = AggregatorMessagesWsSubscribeFrame {
                    id: "sdk-endpoint-test-aggregator-recovering".to_string(),
                    tfs: Some(vec![Timeframe::M1]),
                    predicate: format!("{aggregator_target_pair}.c > 0"),
                    message: "endpoint_test_recovering".to_string(),
                    payload: None,
                };
                if let Err(error) = connection.send_subscribe(&subscribe).await {
                    record_fail(
                        report,
                        "aggregator.connect_messages_ws_recovering",
                        error.to_string(),
                    );
                } else {
                    match timeout(ws_timeout(), connection.next_frame()).await {
                        Ok(Ok(Some(frame))) => record_pass(
                            report,
                            "aggregator.connect_messages_ws_recovering",
                            format!(
                                "frame_kind={} active_subscriptions={} next_attempt={}",
                                aggregator_messages_ws_frame_kind(&frame),
                                connection.active_subscription_ids().len(),
                                connection.next_attempt()
                            ),
                        ),
                        Ok(Ok(None)) => record_fail(
                            report,
                            "aggregator.connect_messages_ws_recovering",
                            "ws stream closed before first server frame",
                        ),
                        Ok(Err(error)) => record_fail(
                            report,
                            "aggregator.connect_messages_ws_recovering",
                            error.to_string(),
                        ),
                        Err(_) => record_fail(
                            report,
                            "aggregator.connect_messages_ws_recovering",
                            "timed out waiting for recovering messages frame",
                        ),
                    }
                }
            }
            Err(error) => record_fail(
                report,
                "aggregator.connect_messages_ws_recovering",
                error.to_string(),
            ),
        }
    } else {
        for surface in [
            "aggregator.connect_bars_ws",
            "aggregator.connect_bars_ws_make_before_break",
            "aggregator.connect_bars_ws_recovering",
            "aggregator.connect_messages_ws",
            "aggregator.connect_messages_ws_recovering",
        ] {
            record_fail(
                report,
                surface,
                "public aggregator ws base url missing from runtime config",
            );
        }
    }

    if runtime.summary.aggregator_grpc_base_url.is_some() {
        let aggregator_latest_http_request = LatestRequest {
            pairs: vec![aggregator_target_pair.clone()],
            tf: Timeframe::M1,
            latest_mode: LatestMode::ExactWatermark,
            metadata: Some(false),
            format: Some(HttpFormat::Json),
        };
        let aggregator_latest_grpc_request =
            LatestGrpcRequest::from(&aggregator_latest_http_request);
        match (
            aggregator.latest(&aggregator_latest_http_request).await,
            aggregator
                .latest_grpc(&aggregator_latest_grpc_request)
                .await,
        ) {
            (Ok(http), Ok(grpc))
                if pair_from_latest_response(&http) == pair_from_latest_response(&grpc)
                    && close_end_from_latest_response(&http)
                        == close_end_from_latest_response(&grpc) =>
            {
                record_pass(
                    report,
                    "aggregator.latest_http_grpc_parity",
                    format!(
                        "pair={} close_end_ms={}",
                        pair_from_latest_response(&http)
                            .or_else(|| pair_from_latest_response(&grpc))
                            .unwrap_or("unknown"),
                        close_end_from_latest_response(&http)
                    ),
                );
            }
            (Ok(http), Ok(grpc)) => record_fail(
                report,
                "aggregator.latest_http_grpc_parity",
                format!(
                    "parity mismatch http_pair={:?} grpc_pair={:?} http_close_end_ms={} grpc_close_end_ms={}",
                    pair_from_latest_response(&http),
                    pair_from_latest_response(&grpc),
                    close_end_from_latest_response(&http),
                    close_end_from_latest_response(&grpc)
                ),
            ),
            (Err(http_error), Err(grpc_error)) => record_fail(
                report,
                "aggregator.latest_http_grpc_parity",
                format!("http={http_error}; grpc={grpc_error}"),
            ),
            (Err(error), _) => record_fail(
                report,
                "aggregator.latest_http_grpc_parity",
                format!("http={error}"),
            ),
            (_, Err(error)) => record_fail(
                report,
                "aggregator.latest_http_grpc_parity",
                format!("grpc={error}"),
            ),
        }
    } else {
        record_fail(
            report,
            "aggregator.latest_http_grpc_parity",
            "public aggregator gRPC base url missing from runtime config",
        );
    }

    let primitives_target_pair = primitives
        .pairs_list(&primitives_system::PairsListRequest {
            after_pair: None,
            limit: Some(1),
            enabled_only: Some(true),
        })
        .await
        .ok()
        .and_then(|out| out.pairs.first().cloned())
        .unwrap_or_else(|| "BTCUSDT".to_string());

    match primitives
        .files_downloads(&primitives_system::FilesDownloadsRequest {
            period: Some("day".to_string()),
            pairs: vec![primitives_target_pair.clone()],
            tfs: vec!["1m".to_string()],
            start_label_utc: None,
            end_label_utc: None,
            order: Some("desc".to_string()),
        })
        .await
    {
        Ok(out) if !out.rows.is_empty() => {
            let root = download_root_for("primitives");
            if let Err(error) = fs::create_dir_all(&root) {
                record_fail(
                    report,
                    "primitives.files_download_items",
                    format!("create_dir_all failed for {}: {error}", root.display()),
                );
            } else {
                match primitives
                    .files_download_items(&[out.rows[0].clone()], Some(root.as_path()))
                    .await
                {
                    Ok(downloaded)
                        if !downloaded.is_empty()
                            && downloaded[0].bytes_written > 0
                            && Path::new(&downloaded[0].destination_path).exists() =>
                    {
                        record_pass(
                            report,
                            "primitives.files_download_items",
                            format!(
                                "bytes_written={} path={}",
                                downloaded[0].bytes_written, downloaded[0].destination_path
                            ),
                        );
                    }
                    Ok(downloaded) => record_fail(
                        report,
                        "primitives.files_download_items",
                        format!("unexpected download result count={}", downloaded.len()),
                    ),
                    Err(error) => {
                        record_fail(report, "primitives.files_download_items", error.to_string())
                    }
                }
            }
        }
        Ok(_) => record_fail(
            report,
            "primitives.files_download_items",
            "no downloadable rows available",
        ),
        Err(error) => record_fail(report, "primitives.files_download_items", error.to_string()),
    }

    let primitives_projected_ws_request = OutputsWsSubscribeRequest {
        pairs: vec![primitives_target_pair.clone()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: Some(vec![PrimitiveProcessorFamily::MovingAverages]),
        group: Some(vec![PrimitiveProcessorGroup::Ema]),
        from_close: None,
        last_n_bars: Some(1),
        format: Some(OutputsWsFormat::Protobuf),
    };
    match primitives
        .connect_outputs_ws(&primitives_projected_ws_request)
        .await
    {
        Err(SdkError::UnsupportedOrUnprovedUsage { .. }) => record_pass(
            report,
            "primitives.projected_outputs_ws_protobuf_rejection",
            "projected outputs ws protobuf request rejected before transport",
        ),
        Err(error) => record_fail(
            report,
            "primitives.projected_outputs_ws_protobuf_rejection",
            error.to_string(),
        ),
        Ok(_) => record_fail(
            report,
            "primitives.projected_outputs_ws_protobuf_rejection",
            "projected outputs ws protobuf request unexpectedly succeeded",
        ),
    }

    let primitives_ws_request = OutputsWsSubscribeRequest {
        pairs: vec![primitives_target_pair.clone()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        from_close: None,
        last_n_bars: Some(1),
        format: Some(OutputsWsFormat::Json),
    };
    match primitives.connect_outputs_ws(&primitives_ws_request).await {
        Ok(mut connection) => {
            let started = tokio::time::Instant::now();
            let mut saw_payload = false;
            let mut saw_replay_done = false;
            let mut frame_kinds = Vec::new();
            loop {
                let elapsed = started.elapsed();
                if elapsed >= ws_replay_timeout() {
                    record_fail(
                        report,
                        "primitives.connect_outputs_ws",
                        format!(
                            "timed out waiting for replay rows + replay_done; observed={}",
                            frame_kinds.join(",")
                        ),
                    );
                    break;
                }

                match timeout(
                    ws_replay_timeout() - elapsed,
                    connection.next_frame(&primitives_ws_request),
                )
                .await
                {
                    Ok(Ok(Some(frame))) => {
                        frame_kinds.push(primitive_outputs_ws_frame_kind(&frame).to_string());
                        if primitive_outputs_ws_is_payload(&frame) {
                            saw_payload = true;
                        }
                        if let PrimitiveOutputsWsInboundFrame::Meta(meta) = &frame {
                            if meta.is_replay_done() {
                                saw_replay_done = true;
                            }
                        }
                        if saw_payload && saw_replay_done {
                            record_pass(
                                report,
                                "primitives.connect_outputs_ws",
                                format!(
                                    "replay rows + replay_done observed within 60s; frames={}",
                                    frame_kinds.join(",")
                                ),
                            );
                            break;
                        }
                    }
                    Ok(Ok(None)) => {
                        record_fail(
                            report,
                            "primitives.connect_outputs_ws",
                            "ws stream closed before replay rows + replay_done were observed",
                        );
                        break;
                    }
                    Ok(Err(error)) => {
                        record_fail(report, "primitives.connect_outputs_ws", error.to_string());
                        break;
                    }
                    Err(_) => {
                        record_fail(
                            report,
                            "primitives.connect_outputs_ws",
                            "timed out waiting for replay rows + replay_done",
                        );
                        break;
                    }
                }
            }
        }
        Err(error) => record_fail(report, "primitives.connect_outputs_ws", error.to_string()),
    }

    let primitives_swap_pair = if primitives_target_pair == "ETHUSDT" {
        "BTCUSDT".to_string()
    } else {
        "ETHUSDT".to_string()
    };
    let primitives_new_ws_request = OutputsWsSubscribeRequest {
        pairs: vec![primitives_swap_pair],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        from_close: None,
        last_n_bars: Some(1),
        format: Some(OutputsWsFormat::Json),
    };
    match primitives
        .connect_outputs_ws_make_before_break(&primitives_ws_request)
        .await
    {
        Ok(mut connection) => {
            if let Err(error) = connection.begin_swap(&primitives_new_ws_request) {
                record_fail(
                    report,
                    "primitives.connect_outputs_ws_make_before_break",
                    error.to_string(),
                );
            } else {
                match timeout(ws_timeout(), connection.next_frame()).await {
                    Ok(Ok(Some(frame))) => record_pass(
                        report,
                        "primitives.connect_outputs_ws_make_before_break",
                        format!(
                            "frame_kind={} swap_in_progress={} active_pairs={}",
                            primitive_outputs_ws_frame_kind(&frame),
                            connection.swap_in_progress(),
                            connection.active_request().pairs.join(",")
                        ),
                    ),
                    Ok(Ok(None)) => record_fail(
                        report,
                        "primitives.connect_outputs_ws_make_before_break",
                        "ws stream closed before first frame",
                    ),
                    Ok(Err(error)) => record_fail(
                        report,
                        "primitives.connect_outputs_ws_make_before_break",
                        error.to_string(),
                    ),
                    Err(_) => record_fail(
                        report,
                        "primitives.connect_outputs_ws_make_before_break",
                        "timed out waiting for make-before-break frame",
                    ),
                }
            }
        }
        Err(error) => record_fail(
            report,
            "primitives.connect_outputs_ws_make_before_break",
            error.to_string(),
        ),
    }

    match primitives
        .connect_outputs_ws_recovering(&primitives_ws_request, ExponentialBackoffConfig::default())
        .await
    {
        Ok(mut connection) => match timeout(ws_timeout(), connection.next_frame()).await {
            Ok(Ok(Some(frame))) => record_pass(
                report,
                "primitives.connect_outputs_ws_recovering",
                format!(
                    "frame_kind={} next_attempt={} active_pairs={}",
                    primitive_outputs_ws_frame_kind(&frame),
                    connection.next_attempt(),
                    connection.active_request().pairs.join(",")
                ),
            ),
            Ok(Ok(None)) => record_fail(
                report,
                "primitives.connect_outputs_ws_recovering",
                "ws stream closed before first frame",
            ),
            Ok(Err(error)) => record_fail(
                report,
                "primitives.connect_outputs_ws_recovering",
                error.to_string(),
            ),
            Err(_) => record_fail(
                report,
                "primitives.connect_outputs_ws_recovering",
                "timed out waiting for recovering frame",
            ),
        },
        Err(error) => record_fail(
            report,
            "primitives.connect_outputs_ws_recovering",
            error.to_string(),
        ),
    }

    match primitives.connect_messages_ws().await {
        Ok(mut connection) => {
            let subscribe = PrimitiveMessagesWsSubscribeFrame {
                id: "sdk-endpoint-test-primitives".to_string(),
                tfs: Some(vec![Timeframe::M1]),
                predicate: format!("{primitives_target_pair}.c > 0"),
                message: "endpoint_test".to_string(),
                payload: None,
            };
            let unsubscribe = PrimitiveMessagesWsUnsubscribeFrame {
                id: "sdk-endpoint-test-primitives".to_string(),
            };
            if let Err(error) = connection.send_subscribe(&subscribe).await {
                record_fail(report, "primitives.connect_messages_ws", error.to_string());
            } else if let Err(error) = connection.send_unsubscribe(&unsubscribe).await {
                record_fail(report, "primitives.connect_messages_ws", error.to_string());
            } else {
                match timeout(ws_timeout(), connection.next_frame()).await {
                    Ok(Ok(Some(frame))) => record_pass(
                        report,
                        "primitives.connect_messages_ws",
                        format!("frame_kind={}", primitive_messages_ws_frame_kind(&frame)),
                    ),
                    Ok(Ok(None)) => record_fail(
                        report,
                        "primitives.connect_messages_ws",
                        "ws stream closed before first server frame",
                    ),
                    Ok(Err(error)) => {
                        record_fail(report, "primitives.connect_messages_ws", error.to_string())
                    }
                    Err(_) => record_fail(
                        report,
                        "primitives.connect_messages_ws",
                        "timed out waiting for messages ws frame",
                    ),
                }
            }
        }
        Err(error) => record_fail(report, "primitives.connect_messages_ws", error.to_string()),
    }

    match primitives
        .connect_messages_ws_recovering(ExponentialBackoffConfig::default())
        .await
    {
        Ok(mut connection) => {
            let subscribe = PrimitiveMessagesWsSubscribeFrame {
                id: "sdk-endpoint-test-primitives-recovering".to_string(),
                tfs: Some(vec![Timeframe::M1]),
                predicate: format!("{primitives_target_pair}.c > 0"),
                message: "endpoint_test_recovering".to_string(),
                payload: None,
            };
            if let Err(error) = connection.send_subscribe(&subscribe).await {
                record_fail(
                    report,
                    "primitives.connect_messages_ws_recovering",
                    error.to_string(),
                );
            } else {
                match timeout(ws_timeout(), connection.next_frame()).await {
                    Ok(Ok(Some(frame))) => record_pass(
                        report,
                        "primitives.connect_messages_ws_recovering",
                        format!(
                            "frame_kind={} active_subscriptions={} next_attempt={}",
                            primitive_messages_ws_frame_kind(&frame),
                            connection.active_subscription_ids().len(),
                            connection.next_attempt()
                        ),
                    ),
                    Ok(Ok(None)) => record_fail(
                        report,
                        "primitives.connect_messages_ws_recovering",
                        "ws stream closed before first server frame",
                    ),
                    Ok(Err(error)) => record_fail(
                        report,
                        "primitives.connect_messages_ws_recovering",
                        error.to_string(),
                    ),
                    Err(_) => record_fail(
                        report,
                        "primitives.connect_messages_ws_recovering",
                        "timed out waiting for recovering messages frame",
                    ),
                }
            }
        }
        Err(error) => record_fail(
            report,
            "primitives.connect_messages_ws_recovering",
            error.to_string(),
        ),
    }

    let primitives_latest_http_request = primitives_system::LatestRequest {
        pairs: vec![primitives_target_pair.clone()],
        tf: Timeframe::M1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Json),
    };
    let primitives_latest_grpc_request =
        primitives_system::LatestGrpcRequest::from(&primitives_latest_http_request);
    match (
        primitives.latest(&primitives_latest_http_request).await,
        primitives
            .latest_grpc(&primitives_latest_grpc_request)
            .await,
    ) {
        (Ok(http), Ok(grpc))
            if http.close_end_ms == grpc.close_end_ms
                && http.rows.len() == grpc.rows.len()
                && http.missing_pairs == grpc.missing_pairs =>
        {
            record_pass(
                report,
                "primitives.latest_http_grpc_parity",
                format!(
                    "rows={} missing_pairs={} close_end_ms={}",
                    http.rows.len(),
                    http.missing_pairs.len(),
                    http.close_end_ms
                ),
            );
        }
        (Ok(http), Ok(grpc)) => record_fail(
            report,
            "primitives.latest_http_grpc_parity",
            format!(
                "parity mismatch http_rows={} grpc_rows={} http_missing_pairs={} grpc_missing_pairs={} http_close_end_ms={} grpc_close_end_ms={}",
                http.rows.len(),
                grpc.rows.len(),
                http.missing_pairs.len(),
                grpc.missing_pairs.len(),
                http.close_end_ms,
                grpc.close_end_ms
            ),
        ),
        (Err(http_error), Err(grpc_error)) => record_fail(
            report,
            "primitives.latest_http_grpc_parity",
            format!("http={http_error}; grpc={grpc_error}"),
        ),
        (Err(error), _) => record_fail(
            report,
            "primitives.latest_http_grpc_parity",
            format!("http={error}"),
        ),
        (_, Err(error)) => record_fail(
            report,
            "primitives.latest_http_grpc_parity",
            format!("grpc={error}"),
        ),
    }

    let primitives_anchor_close_ms = match primitives.latest(&primitives_latest_http_request).await
    {
        Ok(out) if !out.rows.is_empty() => out.close_end_ms,
        _ => 0,
    };
    if primitives_anchor_close_ms > 0 {
        let primitives_range_request = primitives_system::RangeRequest {
            pairs: vec![primitives_target_pair.clone()],
            tf: Timeframe::M1,
            align_mode: Some(AlignMode::Exact),
            close_start: Some(TimeInput::Ms(primitives_anchor_close_ms - 10 * 60_000)),
            cursor: None,
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            limit: Some(5),
            family: None,
            group: None,
            metadata: Some(false),
            diagnostics: Some(false),
            format: Some(HttpFormat::Json),
        };
        match primitives
            .range_call(primitives_range_request.clone())
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "primitives.range_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "primitives.range_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "primitives.range_call", error.to_string()),
        }

        let primitives_range_grpc_request = primitives_system::RangeGrpcRequest {
            pairs: vec![primitives_target_pair.clone()],
            tf: Timeframe::M1,
            align_mode: Some(AlignMode::Exact),
            close_start: Some(TimeInput::Ms(primitives_anchor_close_ms - 10 * 60_000)),
            cursor: None,
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            limit: Some(5),
            family: None,
            group: None,
            metadata: Some(false),
            diagnostics: Some(false),
        };
        match primitives
            .range_grpc_call(primitives_range_grpc_request)
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "primitives.range_grpc_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "primitives.range_grpc_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "primitives.range_grpc_call", error.to_string()),
        }

        let primitives_search_request = primitives_system::SearchRequest {
            tf: Timeframe::M1,
            close_start: TimeInput::Ms(primitives_anchor_close_ms - 60 * 60_000),
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            cursor: None,
            predicate: format!("{primitives_target_pair}.c > 0"),
            evaluate_pair: Some(primitives_target_pair.clone()),
            family: None,
            group: None,
            metadata: Some(true),
            diagnostics: Some(true),
            max_hits: Some(5),
            format: Some(HttpFormat::Json),
        };
        match primitives
            .search_call(primitives_search_request.clone())
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "primitives.search_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "primitives.search_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "primitives.search_call", error.to_string()),
        }

        let primitives_search_grpc_request = primitives_system::SearchGrpcRequest {
            tf: Timeframe::M1,
            close_start: TimeInput::Ms(primitives_anchor_close_ms - 60 * 60_000),
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            cursor: None,
            predicate: format!("{primitives_target_pair}.c > 0"),
            evaluate_pair: Some(primitives_target_pair.clone()),
            family: None,
            group: None,
            metadata: Some(false),
            diagnostics: Some(false),
            max_hits: Some(5),
        };
        match primitives
            .search_grpc_call(primitives_search_grpc_request)
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "primitives.search_grpc_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "primitives.search_grpc_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "primitives.search_grpc_call", error.to_string()),
        }

        let primitives_time_machine_request = primitives_system::TimeMachineRequest {
            tf: Timeframe::M1,
            close_start: TimeInput::Ms(primitives_anchor_close_ms - 20 * 60_000),
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            cursor: None,
            predicate: Some(format!("{primitives_target_pair}.c > 0")),
            hits: None,
            output_pairs: Some(vec![primitives_target_pair.clone()]),
            family: None,
            group: None,
            metadata: Some(true),
            diagnostics: Some(false),
            before_bars: Some(2),
            after_bars: Some(2),
            max_hits: Some(10),
            overlap_mode: Some("merge".to_string()),
            format: Some(HttpFormat::Json),
        };
        match primitives
            .time_machine_call(primitives_time_machine_request.clone())
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "primitives.time_machine_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "primitives.time_machine_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "primitives.time_machine_call", error.to_string()),
        }

        let primitives_time_machine_grpc_request = primitives_system::TimeMachineGrpcRequest {
            tf: Timeframe::M1,
            close_start: TimeInput::Ms(primitives_anchor_close_ms - 20 * 60_000),
            close_end: Some(TimeInput::Ms(primitives_anchor_close_ms)),
            cursor: None,
            predicate: Some(format!("{primitives_target_pair}.c > 0")),
            hits: None,
            output_pairs: Some(vec![primitives_target_pair.clone()]),
            family: None,
            group: None,
            metadata: Some(true),
            diagnostics: Some(false),
            before_bars: Some(2),
            after_bars: Some(2),
            max_hits: Some(10),
            overlap_mode: Some("merge".to_string()),
        };
        match primitives
            .time_machine_grpc_call(primitives_time_machine_grpc_request)
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "primitives.time_machine_grpc_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "primitives.time_machine_grpc_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(
                report,
                "primitives.time_machine_grpc_call",
                error.to_string(),
            ),
        }
    } else {
        for surface in [
            "primitives.range_call",
            "primitives.range_grpc_call",
            "primitives.search_call",
            "primitives.search_grpc_call",
            "primitives.time_machine_call",
            "primitives.time_machine_grpc_call",
        ] {
            record_fail(report, surface, "latest anchor was not established");
        }
    }

    let regime_target_pair = regime
        .pairs_list(&regime_system::PairsListRequest {
            after_pair: None,
            limit: Some(1),
            enabled_only: Some(true),
        })
        .await
        .ok()
        .and_then(|out| out.pairs.first().cloned())
        .unwrap_or_else(|| "BTCUSDT".to_string());

    match regime
        .files_downloads(&regime_system::FilesDownloadsRequest {
            period: Some("day".to_string()),
            pairs: vec![regime_target_pair.clone()],
            tfs: vec!["1h".to_string()],
            start_label_utc: None,
            end_label_utc: None,
            order: Some("desc".to_string()),
        })
        .await
    {
        Ok(out) if !out.rows.is_empty() => {
            let root = download_root_for("regime");
            if let Err(error) = fs::create_dir_all(&root) {
                record_fail(
                    report,
                    "regime.files_download_items",
                    format!("create_dir_all failed for {}: {error}", root.display()),
                );
            } else {
                match regime
                    .files_download_items(&[out.rows[0].clone()], Some(root.as_path()))
                    .await
                {
                    Ok(downloaded)
                        if !downloaded.is_empty()
                            && downloaded[0].bytes_written > 0
                            && Path::new(&downloaded[0].destination_path).exists() =>
                    {
                        record_pass(
                            report,
                            "regime.files_download_items",
                            format!(
                                "bytes_written={} path={}",
                                downloaded[0].bytes_written, downloaded[0].destination_path
                            ),
                        );
                    }
                    Ok(downloaded) => record_fail(
                        report,
                        "regime.files_download_items",
                        format!("unexpected download result count={}", downloaded.len()),
                    ),
                    Err(error) => {
                        record_fail(report, "regime.files_download_items", error.to_string())
                    }
                }
            }
        }
        Ok(_) => record_fail(
            report,
            "regime.files_download_items",
            "no downloadable rows available",
        ),
        Err(error) => record_fail(report, "regime.files_download_items", error.to_string()),
    }

    let regime_projected_ws_request = RegimeOutputsWsSubscribeRequest {
        pairs: vec![regime_target_pair.clone()],
        tfs: vec![Timeframe::H1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        secondary: Some(false),
        from_close: None,
        last_n_bars: Some(1),
        format: Some(RegimeOutputsWsFormat::Protobuf),
    };
    match regime
        .connect_outputs_ws(&regime_projected_ws_request)
        .await
    {
        Err(SdkError::UnsupportedOrUnprovedUsage { .. }) => record_pass(
            report,
            "regime.projected_outputs_ws_protobuf_rejection",
            "projected outputs ws protobuf request rejected before transport",
        ),
        Err(error) => record_fail(
            report,
            "regime.projected_outputs_ws_protobuf_rejection",
            error.to_string(),
        ),
        Ok(_) => record_fail(
            report,
            "regime.projected_outputs_ws_protobuf_rejection",
            "projected outputs ws protobuf request unexpectedly succeeded",
        ),
    }

    let regime_non_h1_ws_request = RegimeOutputsWsSubscribeRequest {
        pairs: vec![regime_target_pair.clone()],
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        secondary: Some(true),
        from_close: None,
        last_n_bars: Some(1),
        format: Some(RegimeOutputsWsFormat::Json),
    };
    match regime.connect_outputs_ws(&regime_non_h1_ws_request).await {
        Err(SdkError::UnsupportedOrUnprovedUsage { message }) if message.contains("tf=1h") => {
            record_pass(
                report,
                "regime.non_h1_outputs_ws_rejection",
                "non-h1 outputs ws request rejected before transport",
            )
        }
        Err(error) => record_fail(
            report,
            "regime.non_h1_outputs_ws_rejection",
            error.to_string(),
        ),
        Ok(_) => record_fail(
            report,
            "regime.non_h1_outputs_ws_rejection",
            "non-h1 outputs ws request unexpectedly succeeded",
        ),
    }

    let regime_ws_request = RegimeOutputsWsSubscribeRequest {
        pairs: vec![regime_target_pair.clone()],
        tfs: vec![Timeframe::H1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        secondary: Some(false),
        from_close: None,
        last_n_bars: Some(1),
        format: Some(RegimeOutputsWsFormat::Json),
    };
    match regime.connect_outputs_ws(&regime_ws_request).await {
        Ok(mut connection) => {
            match timeout(ws_timeout(), connection.next_frame(&regime_ws_request)).await {
                Ok(Ok(Some(frame))) => record_pass(
                    report,
                    "regime.connect_outputs_ws",
                    format!("frame_kind={}", regime_outputs_ws_frame_kind(&frame)),
                ),
                Ok(Ok(None)) => record_fail(
                    report,
                    "regime.connect_outputs_ws",
                    "ws stream closed before first frame",
                ),
                Ok(Err(error)) => {
                    record_fail(report, "regime.connect_outputs_ws", error.to_string())
                }
                Err(_) => record_fail(
                    report,
                    "regime.connect_outputs_ws",
                    "timed out waiting for first frame",
                ),
            }
        }
        Err(error) => record_fail(report, "regime.connect_outputs_ws", error.to_string()),
    }

    let regime_swap_pair = if regime_target_pair == "ETHUSDT" {
        "BTCUSDT".to_string()
    } else {
        "ETHUSDT".to_string()
    };
    let regime_new_ws_request = RegimeOutputsWsSubscribeRequest {
        pairs: vec![regime_swap_pair],
        tfs: vec![Timeframe::H1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        secondary: Some(false),
        from_close: None,
        last_n_bars: Some(1),
        format: Some(RegimeOutputsWsFormat::Json),
    };
    match regime
        .connect_outputs_ws_make_before_break(&regime_ws_request)
        .await
    {
        Ok(mut connection) => {
            if let Err(error) = connection.begin_swap(&regime_new_ws_request) {
                record_fail(
                    report,
                    "regime.connect_outputs_ws_make_before_break",
                    error.to_string(),
                );
            } else {
                match timeout(ws_timeout(), connection.next_frame()).await {
                    Ok(Ok(Some(frame))) => record_pass(
                        report,
                        "regime.connect_outputs_ws_make_before_break",
                        format!(
                            "frame_kind={} swap_in_progress={} active_pairs={}",
                            regime_outputs_ws_frame_kind(&frame),
                            connection.swap_in_progress(),
                            connection.active_request().pairs.join(",")
                        ),
                    ),
                    Ok(Ok(None)) => record_fail(
                        report,
                        "regime.connect_outputs_ws_make_before_break",
                        "ws stream closed before first frame",
                    ),
                    Ok(Err(error)) => record_fail(
                        report,
                        "regime.connect_outputs_ws_make_before_break",
                        error.to_string(),
                    ),
                    Err(_) => record_fail(
                        report,
                        "regime.connect_outputs_ws_make_before_break",
                        "timed out waiting for make-before-break frame",
                    ),
                }
            }
        }
        Err(error) => record_fail(
            report,
            "regime.connect_outputs_ws_make_before_break",
            error.to_string(),
        ),
    }

    match regime
        .connect_outputs_ws_recovering(&regime_ws_request, ExponentialBackoffConfig::default())
        .await
    {
        Ok(mut connection) => match timeout(ws_timeout(), connection.next_frame()).await {
            Ok(Ok(Some(frame))) => record_pass(
                report,
                "regime.connect_outputs_ws_recovering",
                format!(
                    "frame_kind={} next_attempt={} active_pairs={}",
                    regime_outputs_ws_frame_kind(&frame),
                    connection.next_attempt(),
                    connection.active_request().pairs.join(",")
                ),
            ),
            Ok(Ok(None)) => record_fail(
                report,
                "regime.connect_outputs_ws_recovering",
                "ws stream closed before first frame",
            ),
            Ok(Err(error)) => record_fail(
                report,
                "regime.connect_outputs_ws_recovering",
                error.to_string(),
            ),
            Err(_) => record_fail(
                report,
                "regime.connect_outputs_ws_recovering",
                "timed out waiting for recovering frame",
            ),
        },
        Err(error) => record_fail(
            report,
            "regime.connect_outputs_ws_recovering",
            error.to_string(),
        ),
    }

    match regime.connect_messages_ws().await {
        Ok(mut connection) => {
            let subscribe = RegimeMessagesWsSubscribeFrame {
                id: "sdk-endpoint-test-regime".to_string(),
                tfs: Some(vec![Timeframe::H1]),
                predicate: format!("{regime_target_pair}.c > 0"),
                message: "endpoint_test".to_string(),
                payload: None,
            };
            let unsubscribe = RegimeMessagesWsUnsubscribeFrame {
                id: "sdk-endpoint-test-regime".to_string(),
            };
            if let Err(error) = connection.send_subscribe(&subscribe).await {
                record_fail(report, "regime.connect_messages_ws", error.to_string());
            } else if let Err(error) = connection.send_unsubscribe(&unsubscribe).await {
                record_fail(report, "regime.connect_messages_ws", error.to_string());
            } else {
                match timeout(ws_timeout(), connection.next_frame()).await {
                    Ok(Ok(Some(frame))) => record_pass(
                        report,
                        "regime.connect_messages_ws",
                        format!("frame_kind={}", regime_messages_ws_frame_kind(&frame)),
                    ),
                    Ok(Ok(None)) => record_fail(
                        report,
                        "regime.connect_messages_ws",
                        "ws stream closed before first server frame",
                    ),
                    Ok(Err(error)) => {
                        record_fail(report, "regime.connect_messages_ws", error.to_string())
                    }
                    Err(_) => record_fail(
                        report,
                        "regime.connect_messages_ws",
                        "timed out waiting for messages ws frame",
                    ),
                }
            }
        }
        Err(error) => record_fail(report, "regime.connect_messages_ws", error.to_string()),
    }

    match regime
        .connect_messages_ws_recovering(ExponentialBackoffConfig::default())
        .await
    {
        Ok(mut connection) => {
            let subscribe = RegimeMessagesWsSubscribeFrame {
                id: "sdk-endpoint-test-regime-recovering".to_string(),
                tfs: Some(vec![Timeframe::H1]),
                predicate: format!("{regime_target_pair}.c > 0"),
                message: "endpoint_test_recovering".to_string(),
                payload: None,
            };
            if let Err(error) = connection.send_subscribe(&subscribe).await {
                record_fail(
                    report,
                    "regime.connect_messages_ws_recovering",
                    error.to_string(),
                );
            } else {
                match timeout(ws_timeout(), connection.next_frame()).await {
                    Ok(Ok(Some(frame))) => record_pass(
                        report,
                        "regime.connect_messages_ws_recovering",
                        format!(
                            "frame_kind={} active_subscriptions={} next_attempt={}",
                            regime_messages_ws_frame_kind(&frame),
                            connection.active_subscription_ids().len(),
                            connection.next_attempt()
                        ),
                    ),
                    Ok(Ok(None)) => record_fail(
                        report,
                        "regime.connect_messages_ws_recovering",
                        "ws stream closed before first server frame",
                    ),
                    Ok(Err(error)) => record_fail(
                        report,
                        "regime.connect_messages_ws_recovering",
                        error.to_string(),
                    ),
                    Err(_) => record_fail(
                        report,
                        "regime.connect_messages_ws_recovering",
                        "timed out waiting for recovering messages frame",
                    ),
                }
            }
        }
        Err(error) => record_fail(
            report,
            "regime.connect_messages_ws_recovering",
            error.to_string(),
        ),
    }

    let regime_latest_http_request = regime_system::LatestRequest {
        pairs: vec![regime_target_pair.clone()],
        tf: Timeframe::H1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        secondary: Some(true),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Json),
    };
    let regime_latest_grpc_request = regime_system::LatestGrpcRequest {
        pairs: vec![regime_target_pair.clone()],
        tf: Timeframe::H1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: None,
        group: None,
        secondary: Some(true),
        metadata: Some(false),
        diagnostics: Some(false),
    };
    match (
        regime.latest(&regime_latest_http_request).await,
        regime.latest_grpc(&regime_latest_grpc_request).await,
    ) {
        (Ok(http), Ok(grpc))
            if http.close_end_ms == grpc.close_end_ms
                && http.rows.len() == grpc.rows.len()
                && http.missing_pairs == grpc.missing_pairs =>
        {
            record_pass(
                report,
                "regime.latest_http_grpc_parity",
                format!(
                    "rows={} missing_pairs={} close_end_ms={}",
                    http.rows.len(),
                    http.missing_pairs.len(),
                    http.close_end_ms
                ),
            );
        }
        (Ok(http), Ok(grpc)) => record_fail(
            report,
            "regime.latest_http_grpc_parity",
            format!(
                "parity mismatch http_rows={} grpc_rows={} http_missing_pairs={} grpc_missing_pairs={} http_close_end_ms={} grpc_close_end_ms={}",
                http.rows.len(),
                grpc.rows.len(),
                http.missing_pairs.len(),
                grpc.missing_pairs.len(),
                http.close_end_ms,
                grpc.close_end_ms
            ),
        ),
        (Err(http_error), Err(grpc_error)) => record_fail(
            report,
            "regime.latest_http_grpc_parity",
            format!("http={http_error}; grpc={grpc_error}"),
        ),
        (Err(error), _) => record_fail(
            report,
            "regime.latest_http_grpc_parity",
            format!("http={error}"),
        ),
        (_, Err(error)) => record_fail(
            report,
            "regime.latest_http_grpc_parity",
            format!("grpc={error}"),
        ),
    }

    let regime_anchor_close_ms = match regime.latest(&regime_latest_http_request).await {
        Ok(out) if !out.rows.is_empty() => out.close_end_ms,
        _ => 0,
    };
    if regime_anchor_close_ms > 0 {
        let regime_range_request = regime_system::RangeRequest {
            pairs: vec![regime_target_pair.clone()],
            tf: Timeframe::H1,
            align_mode: Some(AlignMode::Exact),
            close_start: Some(TimeInput::Ms(regime_anchor_close_ms - 10 * 3_600_000)),
            cursor: None,
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            limit: Some(5),
            family: None,
            group: None,
            secondary: Some(false),
            metadata: Some(false),
            diagnostics: Some(false),
            format: Some(HttpFormat::Json),
        };
        match regime
            .range_call(regime_range_request.clone())
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "regime.range_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "regime.range_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "regime.range_call", error.to_string()),
        }

        let regime_range_grpc_request = regime_system::RangeGrpcRequest {
            pairs: vec![regime_target_pair.clone()],
            tf: Timeframe::H1,
            align_mode: Some(AlignMode::Exact),
            close_start: Some(TimeInput::Ms(regime_anchor_close_ms - 10 * 3_600_000)),
            cursor: None,
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            limit: Some(5),
            family: None,
            group: None,
            secondary: Some(true),
            metadata: Some(false),
            diagnostics: Some(false),
        };
        match regime
            .range_grpc_call(regime_range_grpc_request)
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "regime.range_grpc_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "regime.range_grpc_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "regime.range_grpc_call", error.to_string()),
        }

        let regime_search_request = regime_system::SearchRequest {
            tf: Timeframe::H1,
            close_start: TimeInput::Ms(regime_anchor_close_ms - 24 * 3_600_000),
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            cursor: None,
            predicate: format!("{regime_target_pair}.c > 0"),
            evaluate_pair: Some(regime_target_pair.clone()),
            family: None,
            group: None,
            secondary: Some(true),
            metadata: Some(true),
            diagnostics: Some(true),
            max_hits: Some(5),
            format: Some(HttpFormat::Json),
        };
        match regime
            .search_call(regime_search_request.clone())
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "regime.search_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "regime.search_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "regime.search_call", error.to_string()),
        }

        let regime_search_grpc_request = regime_system::SearchGrpcRequest {
            tf: Timeframe::H1,
            close_start: TimeInput::Ms(regime_anchor_close_ms - 24 * 3_600_000),
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            cursor: None,
            predicate: format!("{regime_target_pair}.c > 0"),
            evaluate_pair: Some(regime_target_pair.clone()),
            family: None,
            group: None,
            secondary: Some(true),
            metadata: Some(false),
            diagnostics: Some(false),
            max_hits: Some(5),
        };
        match regime
            .search_grpc_call(regime_search_grpc_request)
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "regime.search_grpc_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "regime.search_grpc_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "regime.search_grpc_call", error.to_string()),
        }

        let regime_time_machine_request = regime_system::TimeMachineRequest {
            tf: Timeframe::H1,
            close_start: TimeInput::Ms(regime_anchor_close_ms - 24 * 3_600_000),
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            cursor: None,
            predicate: Some(format!("{regime_target_pair}.c > 0")),
            hits: None,
            output_pairs: Some(vec![regime_target_pair.clone()]),
            family: None,
            group: None,
            secondary: Some(false),
            metadata: Some(true),
            diagnostics: Some(false),
            before_bars: Some(2),
            after_bars: Some(2),
            max_hits: Some(10),
            overlap_mode: Some("merge".to_string()),
            format: Some(HttpFormat::Json),
        };
        match regime
            .time_machine_call(regime_time_machine_request.clone())
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "regime.time_machine_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "regime.time_machine_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "regime.time_machine_call", error.to_string()),
        }

        let regime_time_machine_grpc_request = regime_system::TimeMachineGrpcRequest {
            tf: Timeframe::H1,
            close_start: TimeInput::Ms(regime_anchor_close_ms - 24 * 3_600_000),
            close_end: Some(TimeInput::Ms(regime_anchor_close_ms)),
            cursor: None,
            predicate: Some(format!("{regime_target_pair}.c > 0")),
            hits: None,
            output_pairs: Some(vec![regime_target_pair.clone()]),
            family: None,
            group: None,
            secondary: Some(true),
            metadata: Some(true),
            diagnostics: Some(false),
            before_bars: Some(2),
            after_bars: Some(2),
            max_hits: Some(10),
            overlap_mode: Some("merge".to_string()),
        };
        match regime
            .time_machine_grpc_call(regime_time_machine_grpc_request)
            .traverse()
            .await
        {
            Ok(out) if out.pages_fetched > 0 && !out.pages.is_empty() => record_pass(
                report,
                "regime.time_machine_grpc_call",
                format!(
                    "pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Ok(out) => record_fail(
                report,
                "regime.time_machine_grpc_call",
                format!(
                    "unexpected traverse shape: pages_fetched={} pages={}",
                    out.pages_fetched,
                    out.pages.len()
                ),
            ),
            Err(error) => record_fail(report, "regime.time_machine_grpc_call", error.to_string()),
        }
    } else {
        for surface in [
            "regime.range_call",
            "regime.range_grpc_call",
            "regime.search_call",
            "regime.search_grpc_call",
            "regime.time_machine_call",
            "regime.time_machine_grpc_call",
        ] {
            record_fail(report, surface, "latest anchor was not established");
        }
    }
}

async fn run(settings: &Settings) -> Result<Report, String> {
    load_dotenv_if_present(&settings.dotenv_path)?;

    let mut report = Report {
        title: "SDK Full Public Endpoint Verification Report".to_string(),
        execution_timestamp_utc: timestamp_for_report(),
        config: None,
        results: initial_case_results(),
        proved_observations: Vec::new(),
        failures: Vec::new(),
        skipped: Vec::new(),
        final_status: "full_public_endpoint_verification_failed".to_string(),
    };

    println!("[{BIN_NAME}] starting phase-6 ws, downloads, pagination, and parity verification");
    println!(
        "[{BIN_NAME}] loading environment from {}",
        settings.dotenv_path.display()
    );

    let runtime = match build_runtime_config() {
        Ok(runtime) => runtime,
        Err(error) => {
            record_fail(
                &mut report,
                "foundation.client_construction",
                format!("client foundation setup failed: {error}"),
            );
            return Ok(report);
        }
    };

    report.config = Some(runtime.summary.clone());
    report.proved_observations.push(
        "constructed `Aggregator` from checked-in public defaults with the shared bearer token"
            .to_string(),
    );
    report.proved_observations.push(
        "constructed `Intro`, `Primitives`, and `Regime` from checked-in public defaults without new environment variables".to_string(),
    );

    run_phase_3_foundation(&runtime, &mut report).await;
    run_phase_4_intro_and_aggregator(&runtime, &mut report).await;
    run_phase_5_primitives_and_regime(&runtime, &mut report).await;
    run_phase_6_ws_downloads_pagination_and_parity(&runtime, &mut report).await;
    report.final_status = if report.failures.is_empty() {
        "full_public_endpoint_verification_passed".to_string()
    } else {
        "full_public_endpoint_verification_failed".to_string()
    };

    Ok(report)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let settings = match parse_args() {
        Ok(Some(settings)) => settings,
        Ok(None) => return,
        Err(error) => {
            eprintln!("[{BIN_NAME}] argument_error: {error}");
            std::process::exit(1);
        }
    };

    let timestamp = timestamp_for_filename();
    let report = match run(&settings).await {
        Ok(report) => report,
        Err(error) => Report {
            title: "SDK Full Public Endpoint Verification Report".to_string(),
            execution_timestamp_utc: timestamp_for_report(),
            config: None,
            results: initial_case_results(),
            proved_observations: Vec::new(),
            failures: vec![error],
            skipped: Vec::new(),
            final_status: "full_public_endpoint_verification_failed".to_string(),
        },
    };

    match write_report(&report, &settings, &timestamp) {
        Ok(path) => {
            println!("[{BIN_NAME}] report_written={}", path.display());
            if report.failures.is_empty() {
                std::process::exit(0);
            }
            eprintln!("[{BIN_NAME}] failures_present");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("[{BIN_NAME}] report_write_failed: {error}");
            std::process::exit(1);
        }
    }
}
