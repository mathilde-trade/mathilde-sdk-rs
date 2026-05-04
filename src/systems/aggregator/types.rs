use crate::core::error::SdkError;
use crate::core::time::TimeInput;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1 as proto;
use crate::systems::types::{AlignMode, BarsView, HttpFormat, LatestMode, Timeframe};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Default)]
pub struct PairsStatusRequest {
    pub after_pair: Option<String>,
    pub limit: Option<i64>,
    pub pairs: Option<Vec<String>>,
    pub filters: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct PairsListRequest {
    pub after_pair: Option<String>,
    pub limit: Option<i64>,
    pub enabled_only: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairStatusBootstrap {
    pub done: bool,
    pub harmonized: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairStatusStatusBlock {
    pub enabled: bool,
    pub run_state: String,
    pub last_error: Option<String>,
    pub initial_date_utc: String,
    pub bootstrap: PairStatusBootstrap,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PairStatusSeedQuality {
    pub coverage_p50: Option<f64>,
    pub coverage_p95: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PairStatusHistoryBlock {
    pub seed_enabled: Option<bool>,
    pub seed_done: Option<bool>,
    pub seed_state: Option<String>,
    pub seed_target_end_utc: Option<String>,
    pub seed_cursor_end_utc: Option<String>,
    pub seed_last_error: Option<String>,
    pub seed_done_rule: Option<String>,
    pub seed_quality: PairStatusSeedQuality,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairStatusFrontierBlock {
    pub frontier_subscribed: bool,
    pub frontier_subscribed_at_utc: Option<String>,
    pub frontier_t0_pair_utc: Option<String>,
    pub frontier_last_status_update_utc: Option<String>,
    pub frontier_last_finalized_e_utc: Option<String>,
    pub frontier_enabled_venues_n: i64,
    pub frontier_connected_venues_n: i64,
    pub frontier_last_error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairStatusCountsBlock {
    #[serde(rename = "1m")]
    pub m1: i64,
    #[serde(rename = "5m")]
    pub m5: i64,
    #[serde(rename = "15m")]
    pub m15: i64,
    #[serde(rename = "30m")]
    pub m30: i64,
    #[serde(rename = "1h")]
    pub h1: i64,
    #[serde(rename = "4h")]
    pub h4: i64,
    #[serde(rename = "6h")]
    pub h6: i64,
    #[serde(rename = "12h")]
    pub h12: i64,
    #[serde(rename = "1d")]
    pub d1: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairStatusReadinessCell {
    pub ready: bool,
    pub ready_at_utc: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairStatusReadinessBlock {
    pub m1: PairStatusReadinessCell,
    pub m5: PairStatusReadinessCell,
    pub m15: PairStatusReadinessCell,
    pub m30: PairStatusReadinessCell,
    pub h1: PairStatusReadinessCell,
    pub h4: PairStatusReadinessCell,
    pub h6: PairStatusReadinessCell,
    pub h12: PairStatusReadinessCell,
    pub d1: PairStatusReadinessCell,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PairStatusRow {
    pub pair: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PairStatusStatusBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<PairStatusHistoryBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontier: Option<PairStatusFrontierBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counts: Option<PairStatusCountsBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readiness: Option<PairStatusReadinessBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PairsStatusResponse {
    pub pairs: Vec<PairStatusRow>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairsListResponse {
    pub pairs: Vec<String>,
    pub next_after_pair: Option<String>,
}

pub type PublicOpenApiDocument = serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FilesDownloadsRequest {
    pub period: Option<String>,
    pub pairs: Vec<String>,
    pub tfs: Vec<String>,
    pub start_label_utc: Option<String>,
    pub end_label_utc: Option<String>,
    pub order: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FilesDownloadsRow {
    pub period: String,
    pub pair: String,
    pub tf: String,
    pub label_utc: String,
    pub url: String,
    pub expires_at_utc: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FilesDownloadsResponse {
    pub rows: Vec<FilesDownloadsRow>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DownloadedFile {
    pub row: FilesDownloadsRow,
    pub destination_path: String,
    pub bytes_written: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LatestBarsRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub latest_mode: LatestMode,
    pub metadata: Option<bool>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct NormalizedLatestBarsRequest {
    pub pairs: String,
    pub tf: Timeframe,
    pub latest_mode: LatestMode,
    pub metadata: Option<bool>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LatestBarsGrpcRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub latest_mode: LatestMode,
    pub metadata: Option<bool>,
}

impl LatestBarsRequest {
    pub fn normalize(&self) -> Result<NormalizedLatestBarsRequest, SdkError> {
        Ok(NormalizedLatestBarsRequest {
            pairs: join_required_pair_values_csv(&self.pairs, "latest bars")?,
            tf: self.tf,
            latest_mode: self.latest_mode,
            metadata: self.metadata,
            format: self.format,
        })
    }
}

impl LatestBarsGrpcRequest {
    #[allow(dead_code)]
    pub(crate) fn to_proto(&self) -> Result<proto::LatestBarsRequestV1, SdkError> {
        Ok(proto::LatestBarsRequestV1 {
            pairs: normalize_required_pair_values(&self.pairs, "latest bars")?,
            tf: self.tf.as_str().to_string(),
            latest_mode: self.latest_mode.as_str().to_string(),
            metadata: self.metadata.unwrap_or(false),
        })
    }
}

impl From<&LatestBarsRequest> for LatestBarsGrpcRequest {
    fn from(value: &LatestBarsRequest) -> Self {
        Self {
            pairs: value.pairs.clone(),
            tf: value.tf,
            latest_mode: value.latest_mode,
            metadata: value.metadata,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RangeBarsRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub align_mode: Option<AlignMode>,
    pub close_start: Option<TimeInput>,
    pub cursor: Option<String>,
    pub close_end: Option<TimeInput>,
    pub limit: Option<i64>,
    pub metadata: Option<bool>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RangeBarsGrpcRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub align_mode: Option<AlignMode>,
    pub close_start: Option<TimeInput>,
    pub cursor: Option<String>,
    pub close_end: Option<TimeInput>,
    pub limit: Option<i64>,
    pub metadata: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct NormalizedRangeBarsRequest {
    pub pairs: String,
    pub tf: Timeframe,
    pub align_mode: Option<AlignMode>,
    #[serde(rename = "close_start_ms")]
    pub close_start_ms: Option<i64>,
    pub cursor: Option<String>,
    #[serde(rename = "close_end_ms")]
    pub close_end_ms: Option<i64>,
    pub limit: Option<i64>,
    pub metadata: Option<bool>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct NormalizedRangeBarsGrpcRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub align_mode: Option<AlignMode>,
    pub close_start_ms: i64,
    pub cursor: Option<String>,
    pub close_end_ms: i64,
    pub limit: Option<i64>,
    pub metadata: bool,
}

impl RangeBarsRequest {
    pub fn normalize(&self) -> Result<NormalizedRangeBarsRequest, SdkError> {
        Ok(NormalizedRangeBarsRequest {
            pairs: join_required_pair_values_csv(&self.pairs, "range bars")?,
            tf: self.tf,
            align_mode: self.align_mode,
            close_start_ms: self
                .close_start
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?,
            cursor: self.cursor.clone(),
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?,
            limit: self.limit,
            metadata: self.metadata,
            format: self.format,
        })
    }
}

impl RangeBarsGrpcRequest {
    #[allow(dead_code)]
    pub(crate) fn normalize(&self) -> Result<NormalizedRangeBarsGrpcRequest, SdkError> {
        Ok(NormalizedRangeBarsGrpcRequest {
            pairs: normalize_required_pair_values(&self.pairs, "range bars")?,
            tf: self.tf,
            align_mode: self.align_mode,
            close_start_ms: self
                .close_start
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?
                .unwrap_or(0),
            cursor: self.cursor.clone(),
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?
                .unwrap_or(0),
            limit: self.limit,
            metadata: self.metadata.unwrap_or(false),
        })
    }

    #[allow(dead_code)]
    pub(crate) fn to_proto(&self) -> Result<proto::RangeBarsRequestV1, SdkError> {
        let normalized = self.normalize()?;
        Ok(proto::RangeBarsRequestV1 {
            pairs: normalize_pair_values(&normalized.pairs),
            tf: normalized.tf.as_str().to_string(),
            close_end_ms: normalized.close_end_ms,
            cursor: normalized.cursor,
            limit: normalized.limit,
            metadata: normalized.metadata,
            close_start_ms: normalized.close_start_ms,
            align_mode: normalized
                .align_mode
                .map(|align_mode| align_mode.as_str().to_string()),
        })
    }
}

impl From<&RangeBarsRequest> for RangeBarsGrpcRequest {
    fn from(value: &RangeBarsRequest) -> Self {
        Self {
            pairs: value.pairs.clone(),
            tf: value.tf,
            align_mode: value.align_mode,
            close_start: value.close_start.clone(),
            cursor: value.cursor.clone(),
            close_end: value.close_end.clone(),
            limit: value.limit,
            metadata: value.metadata,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SearchBarsRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: String,
    pub evaluate_pair: Option<String>,
    pub metadata: Option<bool>,
    pub max_hits: Option<i64>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SearchBarsGrpcRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: String,
    pub evaluate_pair: Option<String>,
    pub metadata: Option<bool>,
    pub max_hits: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct NormalizedSearchBarsRequest {
    pub tf: Timeframe,
    #[serde(rename = "close_start_ms")]
    pub close_start_ms: i64,
    #[serde(rename = "close_end_ms")]
    pub close_end_ms: Option<i64>,
    pub cursor: Option<String>,
    pub predicate: String,
    pub evaluate_pair: Option<String>,
    pub metadata: Option<bool>,
    pub max_hits: Option<i64>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct NormalizedSearchBarsGrpcRequest {
    pub tf: Timeframe,
    pub close_start_ms: i64,
    pub close_end_ms: i64,
    pub cursor: Option<String>,
    pub predicate: String,
    pub evaluate_pair: Option<String>,
    pub metadata: bool,
    pub max_hits: Option<i64>,
}

impl SearchBarsRequest {
    pub fn normalize(&self) -> Result<NormalizedSearchBarsRequest, SdkError> {
        Ok(NormalizedSearchBarsRequest {
            tf: self.tf,
            close_start_ms: self.close_start.to_utc_ms()?,
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?,
            cursor: self.cursor.clone(),
            predicate: self.predicate.clone(),
            evaluate_pair: self.evaluate_pair.clone(),
            metadata: self.metadata,
            max_hits: self.max_hits,
            format: self.format,
        })
    }
}

impl SearchBarsGrpcRequest {
    #[allow(dead_code)]
    pub(crate) fn normalize(&self) -> Result<NormalizedSearchBarsGrpcRequest, SdkError> {
        Ok(NormalizedSearchBarsGrpcRequest {
            tf: self.tf,
            close_start_ms: self.close_start.to_utc_ms()?,
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?
                .unwrap_or(0),
            cursor: self.cursor.clone(),
            predicate: self.predicate.trim().to_string(),
            evaluate_pair: self.evaluate_pair.clone(),
            metadata: self.metadata.unwrap_or(false),
            max_hits: self.max_hits,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn to_proto(&self) -> Result<proto::SearchBarsRequestV1, SdkError> {
        let normalized = self.normalize()?;
        Ok(proto::SearchBarsRequestV1 {
            tf: normalized.tf.as_str().to_string(),
            close_start_ms: normalized.close_start_ms,
            close_end_ms: normalized.close_end_ms,
            cursor: normalized.cursor,
            predicate: normalized.predicate,
            evaluate_pair: normalized.evaluate_pair,
            metadata: normalized.metadata,
            max_hits: normalized.max_hits,
        })
    }
}

impl From<&SearchBarsRequest> for SearchBarsGrpcRequest {
    fn from(value: &SearchBarsRequest) -> Self {
        Self {
            tf: value.tf,
            close_start: value.close_start.clone(),
            close_end: value.close_end.clone(),
            cursor: value.cursor.clone(),
            predicate: value.predicate.clone(),
            evaluate_pair: value.evaluate_pair.clone(),
            metadata: value.metadata,
            max_hits: value.max_hits,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TimeMachineBarsRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: Option<String>,
    pub hits: Option<Vec<i64>>,
    pub output_pairs: Option<Vec<String>>,
    pub metadata: Option<bool>,
    pub before_bars: Option<i64>,
    pub after_bars: Option<i64>,
    pub max_hits: Option<i64>,
    pub overlap_mode: Option<String>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TimeMachineBarsGrpcRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: Option<String>,
    pub hits: Option<Vec<i64>>,
    pub output_pairs: Option<Vec<String>>,
    pub metadata: Option<bool>,
    pub before_bars: Option<i64>,
    pub after_bars: Option<i64>,
    pub max_hits: Option<i64>,
    pub overlap_mode: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct NormalizedTimeMachineBarsRequest {
    pub tf: Timeframe,
    #[serde(rename = "close_start_ms")]
    pub close_start_ms: i64,
    #[serde(rename = "close_end_ms")]
    pub close_end_ms: Option<i64>,
    pub cursor: Option<String>,
    pub predicate: Option<String>,
    pub hits: Option<Vec<i64>>,
    pub output_pairs: Option<Vec<String>>,
    pub metadata: Option<bool>,
    pub before_bars: Option<i64>,
    pub after_bars: Option<i64>,
    pub max_hits: Option<i64>,
    pub overlap_mode: Option<String>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct NormalizedTimeMachineBarsGrpcRequest {
    pub tf: Timeframe,
    pub close_start_ms: i64,
    pub close_end_ms: i64,
    pub cursor: Option<String>,
    pub predicate: Option<String>,
    pub hits: Option<Vec<i64>>,
    pub output_pairs: Option<Vec<String>>,
    pub metadata: bool,
    pub before_bars: Option<i64>,
    pub after_bars: Option<i64>,
    pub max_hits: Option<i64>,
    pub overlap_mode: Option<String>,
}

impl TimeMachineBarsRequest {
    pub fn normalize(&self) -> Result<NormalizedTimeMachineBarsRequest, SdkError> {
        Ok(NormalizedTimeMachineBarsRequest {
            tf: self.tf,
            close_start_ms: self.close_start.to_utc_ms()?,
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?,
            cursor: self.cursor.clone(),
            predicate: self.predicate.clone(),
            hits: self.hits.clone(),
            output_pairs: self.output_pairs.clone(),
            metadata: self.metadata,
            before_bars: self.before_bars,
            after_bars: self.after_bars,
            max_hits: self.max_hits,
            overlap_mode: self.overlap_mode.clone(),
            format: self.format,
        })
    }
}

impl TimeMachineBarsGrpcRequest {
    #[allow(dead_code)]
    pub(crate) fn normalize(&self) -> Result<NormalizedTimeMachineBarsGrpcRequest, SdkError> {
        Ok(NormalizedTimeMachineBarsGrpcRequest {
            tf: self.tf,
            close_start_ms: self.close_start.to_utc_ms()?,
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?
                .unwrap_or(0),
            cursor: self.cursor.clone(),
            predicate: self.predicate.clone(),
            hits: self.hits.clone(),
            output_pairs: self.output_pairs.clone(),
            metadata: self.metadata.unwrap_or(false),
            before_bars: self.before_bars,
            after_bars: self.after_bars,
            max_hits: self.max_hits,
            overlap_mode: self.overlap_mode.clone(),
        })
    }

    #[allow(dead_code)]
    pub(crate) fn to_proto(&self) -> Result<proto::TimeMachineBarsRequestV1, SdkError> {
        let normalized = self.normalize()?;
        Ok(proto::TimeMachineBarsRequestV1 {
            tf: normalized.tf.as_str().to_string(),
            close_start_ms: normalized.close_start_ms,
            close_end_ms: normalized.close_end_ms,
            cursor: normalized.cursor,
            predicate: normalized.predicate,
            hits: normalized.hits.unwrap_or_default(),
            output_pairs: normalized.output_pairs.unwrap_or_default(),
            metadata: normalized.metadata,
            before_bars: normalized.before_bars,
            after_bars: normalized.after_bars,
            max_hits: normalized.max_hits,
            overlap_mode: normalized.overlap_mode,
        })
    }
}

impl From<&TimeMachineBarsRequest> for TimeMachineBarsGrpcRequest {
    fn from(value: &TimeMachineBarsRequest) -> Self {
        Self {
            tf: value.tf,
            close_start: value.close_start.clone(),
            close_end: value.close_end.clone(),
            cursor: value.cursor.clone(),
            predicate: value.predicate.clone(),
            hits: value.hits.clone(),
            output_pairs: value.output_pairs.clone(),
            metadata: value.metadata,
            before_bars: value.before_bars,
            after_bars: value.after_bars,
            max_hits: value.max_hits,
            overlap_mode: value.overlap_mode.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Bar {
    pub pair: String,
    pub tf: Timeframe,
    pub open_ms: i64,
    pub close_ms: i64,
    pub open_utc: String,
    pub close_utc: String,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
    pub quote_v: Option<f64>,
    pub taker_known_v: Option<f64>,
    pub taker_signed_v: Option<f64>,
    pub taker_known_quote_v: Option<f64>,
    pub taker_signed_quote_v: Option<f64>,
    pub taker_known_n: Option<i64>,
    pub taker_signed_n: Option<i64>,
    pub vw: Option<f64>,
    pub n: Option<i64>,
    pub coverage_ratio: Option<f64>,
    pub at_ms: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LatestBarsPresentRow {
    #[serde(flatten)]
    pub bar: Bar,
    pub age_ms: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BarMetadata {
    pub source: String,
    pub process: Option<String>,
    pub venues_expected: Option<Vec<String>>,
    pub venues_with_trades: Option<Vec<String>>,
    pub ingested_at_ms: Option<i64>,
    pub ingested_at_utc: Option<String>,
    pub target_ingested_at_ms: Option<i64>,
    pub target_ingested_at_utc: Option<String>,
    pub built_at_ms: Option<i64>,
    pub built_at_utc: Option<String>,
    pub committed_at_ms: Option<i64>,
    pub committed_at_utc: Option<String>,
    pub harmonized_at_ms: Option<i64>,
    pub harmonized_at_utc: Option<String>,
    pub recomputed_at_ms: Option<i64>,
    pub recomputed_at_utc: Option<String>,
    pub recomputed_reason: Option<String>,
    pub covered_1m_count: Option<i64>,
    pub expected_1m_count: Option<i64>,
    pub coverage_ratio: Option<f64>,
    pub inputs_source_counts_frontier: Option<i64>,
    pub inputs_source_counts_api: Option<i64>,
    pub inputs_source_counts_synthetic: Option<i64>,
    pub inputs_source_counts_fix_data: Option<i64>,
    pub frontier_5s_inputs_coverage_ratio: Option<f64>,
    pub frontier_5s_expected: Option<i64>,
    pub frontier_5s_synth_n: Option<i64>,
    pub frontier_5s_synth_ratio: Option<f64>,
    pub frontier_5s_trade_n: Option<i64>,
    pub frontier_5s_trade_ratio: Option<f64>,
    pub age_ms: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BarWithMetadata {
    pub pair: String,
    pub tf: Timeframe,
    pub open_ms: i64,
    pub close_ms: i64,
    pub open_utc: String,
    pub close_utc: String,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
    pub quote_v: Option<f64>,
    pub taker_known_v: Option<f64>,
    pub taker_signed_v: Option<f64>,
    pub taker_known_quote_v: Option<f64>,
    pub taker_signed_quote_v: Option<f64>,
    pub taker_known_n: Option<i64>,
    pub taker_signed_n: Option<i64>,
    pub vw: Option<f64>,
    pub n: Option<i64>,
    pub coverage_ratio: Option<f64>,
    pub at_ms: Option<i64>,
    pub metadata: BarMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LatestBarsWithMetadataPresentRow {
    #[serde(flatten)]
    pub bar: BarWithMetadata,
    pub age_ms: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RangeBarsMinResponse {
    pub rows: Vec<Bar>,
    pub close_end_ms: i64,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RangeBarsFullResponse {
    pub rows: Vec<BarWithMetadata>,
    pub close_end_ms: i64,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RangeBarsResponse {
    Min(RangeBarsMinResponse),
    Full(RangeBarsFullResponse),
}

impl RangeBarsResponse {
    pub fn next_cursor(&self) -> Option<&str> {
        match self {
            Self::Min(response) => response.next_cursor.as_deref(),
            Self::Full(response) => response.next_cursor.as_deref(),
        }
    }

    pub fn close_end_ms(&self) -> i64 {
        match self {
            Self::Min(response) => response.close_end_ms,
            Self::Full(response) => response.close_end_ms,
        }
    }

    pub fn done(&self) -> bool {
        self.next_cursor().is_none()
    }

    pub fn from_proto(
        response: proto::BarsRangeResponseV1,
        metadata: bool,
    ) -> Result<Self, SdkError> {
        if metadata {
            Ok(Self::Full(RangeBarsFullResponse {
                rows: response
                    .rows
                    .into_iter()
                    .map(BarWithMetadata::from_proto)
                    .collect::<Result<Vec<_>, _>>()?,
                close_end_ms: response.close_end_ms,
                next_cursor: response.next_cursor,
            }))
        } else {
            Ok(Self::Min(RangeBarsMinResponse {
                rows: response
                    .rows
                    .into_iter()
                    .map(Bar::from_proto)
                    .collect::<Result<Vec<_>, _>>()?,
                close_end_ms: response.close_end_ms,
                next_cursor: response.next_cursor,
            }))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangeBarsTraverseResult {
    pub pages: Vec<RangeBarsResponse>,
    pub pages_fetched: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SearchBarsMinResponse {
    pub hits: Vec<i64>,
    pub evaluated_rows: Option<Vec<Bar>>,
    pub next_cursor: Option<String>,
    pub done: bool,
    pub returned_hits: i64,
    pub effective_hits_limit: i64,
    pub truncated: bool,
    pub predicate_pairs: Vec<String>,
    pub predicate_normalized: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SearchBarsFullResponse {
    pub hits: Vec<i64>,
    pub evaluated_rows: Option<Vec<BarWithMetadata>>,
    pub next_cursor: Option<String>,
    pub done: bool,
    pub returned_hits: i64,
    pub effective_hits_limit: i64,
    pub truncated: bool,
    pub predicate_pairs: Vec<String>,
    pub predicate_normalized: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchBarsResponse {
    Min(SearchBarsMinResponse),
    Full(SearchBarsFullResponse),
}

impl SearchBarsResponse {
    pub fn next_cursor(&self) -> Option<&str> {
        match self {
            Self::Min(response) => response.next_cursor.as_deref(),
            Self::Full(response) => response.next_cursor.as_deref(),
        }
    }

    pub fn done(&self) -> bool {
        match self {
            Self::Min(response) => response.done,
            Self::Full(response) => response.done,
        }
    }

    pub fn from_proto(
        response: proto::BarsSearchResponseV1,
        metadata: bool,
    ) -> Result<Self, SdkError> {
        if metadata {
            Ok(Self::Full(SearchBarsFullResponse {
                hits: response.hits,
                evaluated_rows: if response.evaluated_rows.is_empty() {
                    None
                } else {
                    Some(
                        response
                            .evaluated_rows
                            .into_iter()
                            .map(BarWithMetadata::from_proto)
                            .collect::<Result<Vec<_>, _>>()?,
                    )
                },
                next_cursor: response.next_cursor,
                done: response.done,
                returned_hits: response.returned_hits,
                effective_hits_limit: response.effective_hits_limit,
                truncated: response.truncated,
                predicate_pairs: response.predicate_pairs,
                predicate_normalized: response.predicate_normalized,
            }))
        } else {
            Ok(Self::Min(SearchBarsMinResponse {
                hits: response.hits,
                evaluated_rows: if response.evaluated_rows.is_empty() {
                    None
                } else {
                    Some(
                        response
                            .evaluated_rows
                            .into_iter()
                            .map(Bar::from_proto)
                            .collect::<Result<Vec<_>, _>>()?,
                    )
                },
                next_cursor: response.next_cursor,
                done: response.done,
                returned_hits: response.returned_hits,
                effective_hits_limit: response.effective_hits_limit,
                truncated: response.truncated,
                predicate_pairs: response.predicate_pairs,
                predicate_normalized: response.predicate_normalized,
            }))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchBarsTraverseResult {
    pub pages: Vec<SearchBarsResponse>,
    pub pages_fetched: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TimeMachineBarsMinRow {
    pub hit_close_ms: i64,
    pub offset: i64,
    pub bar: Bar,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TimeMachineBarsFullRow {
    pub hit_close_ms: i64,
    pub offset: i64,
    pub bar: BarWithMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TimeMachineBarsMinResponse {
    pub rows: Vec<TimeMachineBarsMinRow>,
    pub next_cursor: Option<String>,
    pub done: bool,
    pub returned_hits: i64,
    pub effective_hits_limit: i64,
    pub truncated: bool,
    pub predicate_pairs: Vec<String>,
    pub predicate_normalized: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TimeMachineBarsFullResponse {
    pub rows: Vec<TimeMachineBarsFullRow>,
    pub next_cursor: Option<String>,
    pub done: bool,
    pub returned_hits: i64,
    pub effective_hits_limit: i64,
    pub truncated: bool,
    pub predicate_pairs: Vec<String>,
    pub predicate_normalized: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeMachineBarsResponse {
    Min(TimeMachineBarsMinResponse),
    Full(TimeMachineBarsFullResponse),
}

impl TimeMachineBarsResponse {
    pub fn next_cursor(&self) -> Option<&str> {
        match self {
            Self::Min(response) => response.next_cursor.as_deref(),
            Self::Full(response) => response.next_cursor.as_deref(),
        }
    }

    pub fn done(&self) -> bool {
        match self {
            Self::Min(response) => response.done,
            Self::Full(response) => response.done,
        }
    }

    pub fn from_proto(
        response: proto::BarsTimeMachineResponseV1,
        metadata: bool,
    ) -> Result<Self, SdkError> {
        if metadata {
            Ok(Self::Full(TimeMachineBarsFullResponse {
                rows: response
                    .rows
                    .into_iter()
                    .map(TimeMachineBarsFullRow::from_proto)
                    .collect::<Result<Vec<_>, _>>()?,
                next_cursor: response.next_cursor,
                done: response.done,
                returned_hits: response.returned_hits,
                effective_hits_limit: response.effective_hits_limit,
                truncated: response.truncated,
                predicate_pairs: response.predicate_pairs,
                predicate_normalized: response.predicate_normalized,
            }))
        } else {
            Ok(Self::Min(TimeMachineBarsMinResponse {
                rows: response
                    .rows
                    .into_iter()
                    .map(TimeMachineBarsMinRow::from_proto)
                    .collect::<Result<Vec<_>, _>>()?,
                next_cursor: response.next_cursor,
                done: response.done,
                returned_hits: response.returned_hits,
                effective_hits_limit: response.effective_hits_limit,
                truncated: response.truncated,
                predicate_pairs: response.predicate_pairs,
                predicate_normalized: response.predicate_normalized,
            }))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeMachineBarsTraverseResult {
    pub pages: Vec<TimeMachineBarsResponse>,
    pub pages_fetched: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LatestBarsMinResponse {
    pub watermark_end_ms: i64,
    pub close_end_ms: i64,
    pub latest_mode: LatestMode,
    pub view: BarsView,
    pub rows: Vec<LatestBarsPresentRow>,
    pub missing_pairs: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LatestBarsFullResponse {
    pub watermark_end_ms: i64,
    pub close_end_ms: i64,
    pub latest_mode: LatestMode,
    pub view: BarsView,
    pub rows: Vec<LatestBarsWithMetadataPresentRow>,
    pub missing_pairs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LatestBarsResponse {
    Min(LatestBarsMinResponse),
    Full(LatestBarsFullResponse),
}

impl<'de> serde::Deserialize<'de> for LatestBarsResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let view = value
            .get("view")
            .cloned()
            .ok_or_else(|| serde::de::Error::custom("latest bars response missing `view`"))?;
        let view: BarsView = serde_json::from_value(view).map_err(serde::de::Error::custom)?;

        match view {
            BarsView::Min => serde_json::from_value::<LatestBarsMinResponse>(value)
                .map(Self::Min)
                .map_err(serde::de::Error::custom),
            BarsView::Full => serde_json::from_value::<LatestBarsFullResponse>(value)
                .map(Self::Full)
                .map_err(serde::de::Error::custom),
        }
    }
}

impl LatestBarsResponse {
    pub fn from_proto(response: proto::BarsLatestResponseV1) -> Result<Self, SdkError> {
        let view = proto::BarsViewV1::try_from(response.view).map_err(|_| {
            SdkError::contract_drift("latest bars protobuf response has invalid view")
        })?;

        match view {
            proto::BarsViewV1::Min => Ok(Self::Min(LatestBarsMinResponse {
                watermark_end_ms: response.watermark_end_ms,
                close_end_ms: response.close_end_ms,
                latest_mode: LatestMode::from_proto(&response.latest_mode)?,
                view: BarsView::Min,
                rows: response
                    .rows
                    .into_iter()
                    .map(LatestBarsPresentRow::from_proto)
                    .collect::<Result<Vec<_>, _>>()?,
                missing_pairs: response.missing_pairs,
            })),
            proto::BarsViewV1::Full => Ok(Self::Full(LatestBarsFullResponse {
                watermark_end_ms: response.watermark_end_ms,
                close_end_ms: response.close_end_ms,
                latest_mode: LatestMode::from_proto(&response.latest_mode)?,
                view: BarsView::Full,
                rows: response
                    .rows
                    .into_iter()
                    .map(LatestBarsWithMetadataPresentRow::from_proto)
                    .collect::<Result<Vec<_>, _>>()?,
                missing_pairs: response.missing_pairs,
            })),
            proto::BarsViewV1::Unspecified => Err(SdkError::contract_drift(
                "latest bars protobuf response has unspecified view",
            )),
        }
    }
}

pub(crate) fn normalize_pair_values(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.trim())
        .filter(|pair| !pair.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub(crate) fn normalize_required_pair_values(
    values: &[String],
    context: &'static str,
) -> Result<Vec<String>, SdkError> {
    let normalized = normalize_pair_values(values);
    if normalized.is_empty() {
        return Err(SdkError::request_build(format!(
            "{context} requires at least one pair"
        )));
    }
    Ok(normalized)
}

pub(crate) fn join_required_pair_values_csv(
    values: &[String],
    context: &'static str,
) -> Result<String, SdkError> {
    Ok(normalize_required_pair_values(values, context)?.join(","))
}

pub(crate) fn join_optional_pair_values_csv(values: Option<&[String]>) -> Option<String> {
    let joined = normalize_pair_values(values?).join(",");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

impl LatestBarsPresentRow {
    fn from_proto(value: proto::BarsPresentRowV1) -> Result<Self, SdkError> {
        Ok(Self {
            bar: Bar::from_proto(value.bar.ok_or_else(|| {
                SdkError::contract_drift("latest bars protobuf row missing `bar`")
            })?)?,
            age_ms: value.age_ms.ok_or_else(|| {
                SdkError::contract_drift("latest bars protobuf row missing `age_ms`")
            })?,
        })
    }
}

impl LatestBarsWithMetadataPresentRow {
    fn from_proto(value: proto::BarsPresentRowV1) -> Result<Self, SdkError> {
        Ok(Self {
            bar: BarWithMetadata::from_proto(value.bar.ok_or_else(|| {
                SdkError::contract_drift("latest bars protobuf row missing `bar`")
            })?)?,
            age_ms: value.age_ms.ok_or_else(|| {
                SdkError::contract_drift("latest bars protobuf row missing `age_ms`")
            })?,
        })
    }
}

impl TimeMachineBarsMinRow {
    fn from_proto(value: proto::BarsTimeMachineRowV1) -> Result<Self, SdkError> {
        Ok(Self {
            hit_close_ms: value.hit_close_ms,
            offset: value.offset,
            bar: Bar::from_proto(value.bar.ok_or_else(|| {
                SdkError::contract_drift("time-machine protobuf row missing `bar`")
            })?)?,
        })
    }
}

impl TimeMachineBarsFullRow {
    fn from_proto(value: proto::BarsTimeMachineRowV1) -> Result<Self, SdkError> {
        Ok(Self {
            hit_close_ms: value.hit_close_ms,
            offset: value.offset,
            bar: BarWithMetadata::from_proto(value.bar.ok_or_else(|| {
                SdkError::contract_drift("time-machine full protobuf row missing `bar`")
            })?)?,
        })
    }
}

impl Bar {
    pub(crate) fn from_proto(value: proto::BarRowV1) -> Result<Self, SdkError> {
        Ok(Self {
            pair: value.pair,
            tf: Timeframe::from_proto(&value.tf)?,
            open_ms: value.s_ms,
            close_ms: value.e_ms,
            open_utc: value
                .s_utc
                .ok_or_else(|| SdkError::contract_drift("bar protobuf row missing `s_utc`"))?,
            close_utc: value
                .e_utc
                .ok_or_else(|| SdkError::contract_drift("bar protobuf row missing `e_utc`"))?,
            o: value.o,
            h: value.h,
            l: value.l,
            c: value.c,
            v: value.v,
            quote_v: value.quote_v,
            taker_known_v: value.taker_known_v,
            taker_signed_v: value.taker_signed_v,
            taker_known_quote_v: value.taker_known_quote_v,
            taker_signed_quote_v: value.taker_signed_quote_v,
            taker_known_n: value.taker_known_n,
            taker_signed_n: value.taker_signed_n,
            vw: value.vw,
            n: value.n,
            coverage_ratio: value.coverage_ratio,
            at_ms: value.at_ms,
        })
    }
}

impl BarWithMetadata {
    pub(crate) fn from_proto(value: proto::BarRowV1) -> Result<Self, SdkError> {
        Ok(Self {
            pair: value.pair,
            tf: Timeframe::from_proto(&value.tf)?,
            open_ms: value.s_ms,
            close_ms: value.e_ms,
            open_utc: value
                .s_utc
                .ok_or_else(|| SdkError::contract_drift("bar protobuf row missing `s_utc`"))?,
            close_utc: value
                .e_utc
                .ok_or_else(|| SdkError::contract_drift("bar protobuf row missing `e_utc`"))?,
            o: value.o,
            h: value.h,
            l: value.l,
            c: value.c,
            v: value.v,
            quote_v: value.quote_v,
            taker_known_v: value.taker_known_v,
            taker_signed_v: value.taker_signed_v,
            taker_known_quote_v: value.taker_known_quote_v,
            taker_signed_quote_v: value.taker_signed_quote_v,
            taker_known_n: value.taker_known_n,
            taker_signed_n: value.taker_signed_n,
            vw: value.vw,
            n: value.n,
            coverage_ratio: value.coverage_ratio,
            at_ms: value.at_ms,
            metadata: BarMetadata::from_proto(value.metadata.ok_or_else(|| {
                SdkError::contract_drift("latest bars full protobuf row missing `metadata`")
            })?),
        })
    }
}

impl BarMetadata {
    fn from_proto(value: proto::BarMetadataV1) -> Self {
        Self {
            source: value.source,
            process: value.process,
            venues_expected: if value.venues_expected.is_empty() {
                None
            } else {
                Some(value.venues_expected)
            },
            venues_with_trades: if value.venues_with_trades.is_empty() {
                None
            } else {
                Some(value.venues_with_trades)
            },
            ingested_at_ms: value.ingested_at_ms,
            ingested_at_utc: value.ingested_at_utc,
            target_ingested_at_ms: value.target_ingested_at_ms,
            target_ingested_at_utc: value.target_ingested_at_utc,
            built_at_ms: value.built_at_ms,
            built_at_utc: value.built_at_utc,
            committed_at_ms: value.committed_at_ms,
            committed_at_utc: value.committed_at_utc,
            harmonized_at_ms: value.harmonized_at_ms,
            harmonized_at_utc: value.harmonized_at_utc,
            recomputed_at_ms: value.recomputed_at_ms,
            recomputed_at_utc: value.recomputed_at_utc,
            recomputed_reason: value.recomputed_reason,
            covered_1m_count: value.covered_1m_count,
            expected_1m_count: value.expected_1m_count,
            coverage_ratio: value.coverage_ratio,
            inputs_source_counts_frontier: value.inputs_source_counts_frontier,
            inputs_source_counts_api: value.inputs_source_counts_api,
            inputs_source_counts_synthetic: value.inputs_source_counts_synthetic,
            inputs_source_counts_fix_data: value.inputs_source_counts_fix_data,
            frontier_5s_inputs_coverage_ratio: value.frontier_5s_inputs_coverage_ratio,
            frontier_5s_expected: value.frontier_5s_expected,
            frontier_5s_synth_n: value.frontier_5s_synth_n,
            frontier_5s_synth_ratio: value.frontier_5s_synth_ratio,
            frontier_5s_trade_n: value.frontier_5s_trade_n,
            frontier_5s_trade_ratio: value.frontier_5s_trade_ratio,
            age_ms: value.age_ms,
        }
    }
}

impl Timeframe {
    fn from_proto(value: &str) -> Result<Self, SdkError> {
        match value {
            "1m" => Ok(Self::M1),
            "5m" => Ok(Self::M5),
            "15m" => Ok(Self::M15),
            "30m" => Ok(Self::M30),
            "1h" => Ok(Self::H1),
            "4h" => Ok(Self::H4),
            "6h" => Ok(Self::H6),
            "12h" => Ok(Self::H12),
            "1d" => Ok(Self::D1),
            other => Err(SdkError::contract_drift(format!(
                "unsupported timeframe `{other}`"
            ))),
        }
    }
}

impl LatestMode {
    fn from_proto(value: &str) -> Result<Self, SdkError> {
        match value {
            "exact_watermark" => Ok(Self::ExactWatermark),
            "latest_available_le_watermark" => Ok(Self::LatestAvailableLeWatermark),
            other => Err(SdkError::contract_drift(format!(
                "unsupported latest_mode `{other}`"
            ))),
        }
    }
}
