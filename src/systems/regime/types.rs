use crate::core::error::SdkError;
use crate::core::time::TimeInput;
use crate::generated::regime::{
    OutputBarsMetadata, OutputMetadata, OutputProcessDiagnostic, PROCESSOR_FIELD_NAMES,
    ProcessorFamily, ProcessorGroup,
    outputs_proto::mathilde::feed::outputs::v1 as proto,
    processor_field_support::{collect_numeric_computed_fields, is_known_processor_field},
};
use crate::systems::types::{AlignMode, HttpFormat, LatestMode, Timeframe};
use prost::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegimeOutputMode {
    Min,
    WithMeta,
    ProjectedMin,
    ProjectedWithMeta,
}

impl RegimeOutputMode {
    pub(crate) const fn is_projected(self) -> bool {
        matches!(self, Self::ProjectedMin | Self::ProjectedWithMeta)
    }

    pub(crate) const fn has_metadata(self) -> bool {
        matches!(self, Self::WithMeta | Self::ProjectedWithMeta)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputView {
    Min,
    Full,
}

impl OutputView {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Min => "min",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, serde::Serialize)]
#[serde(transparent)]
pub struct ComputedFields(pub(crate) serde_json::Map<String, serde_json::Value>);

impl ComputedFields {
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }

    pub fn f64(&self, key: &str) -> Option<f64> {
        self.get(key)?.as_f64()
    }

    pub fn i64(&self, key: &str) -> Option<i64> {
        self.get(key)?.as_i64()
    }

    pub fn bool(&self, key: &str) -> Option<bool> {
        self.get(key)?.as_bool()
    }

    pub fn str_value(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &serde_json::Value)> {
        self.0.iter().map(|(key, value)| (key.as_str(), value))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn into_inner(self) -> serde_json::Map<String, serde_json::Value> {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct OutputRow {
    pub pair: String,
    pub tf: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<OutputMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<OutputProcessDiagnostic>>,
    #[serde(skip_serializing_if = "ComputedFields::is_empty")]
    pub computed: ComputedFields,
}

impl OutputRow {
    pub fn computed(&self) -> &ComputedFields {
        &self.computed
    }

    pub(crate) fn ensure_metadata_shape(
        &self,
        metadata_required: bool,
        context: &'static str,
    ) -> Result<(), SdkError> {
        match (metadata_required, self.metadata.is_some()) {
            (true, false) => Err(SdkError::contract_drift(format!(
                "{context} missing `metadata`"
            ))),
            (false, true) => Err(SdkError::contract_drift(format!(
                "{context} unexpectedly included `metadata`"
            ))),
            _ => Ok(()),
        }
    }

    pub(crate) fn apply_diagnostics_gate(mut self, enabled: bool) -> Self {
        if !enabled {
            self.diagnostics = None;
        }
        self
    }
}

pub type PublicOpenApiDocument = serde_json::Value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct DocsRegistryRequest {
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
}

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
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairStatusStatusBlock {
    pub enabled: bool,
    pub run_state: String,
    pub last_error: Option<String>,
    pub initial_date_utc: String,
    pub bootstrap: PairStatusBootstrap,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairStatusHistoryBlock {
    pub seed_enabled: Option<bool>,
    pub seed_done: Option<bool>,
    pub seed_state: Option<String>,
    pub seed_last_error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairStatusReadinessCell {
    pub ready: bool,
    pub ready_at_utc: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PairStatusReadinessBlock {
    pub h1: PairStatusReadinessCell,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct PairStatusRow {
    pub pair: String,
    pub status: Option<PairStatusStatusBlock>,
    pub history: Option<PairStatusHistoryBlock>,
    pub readiness: Option<PairStatusReadinessBlock>,
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
pub struct LatestRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub latest_mode: Option<LatestMode>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub format: Option<HttpFormat>,
}

impl LatestRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        ensure_supported_regime_tf(self.tf, "latest outputs")?;
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<RegimeOutputMode, SdkError> {
        infer_output_mode(
            self.family.as_deref(),
            self.group.as_deref(),
            self.secondary,
            self.metadata,
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LatestGrpcRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub latest_mode: Option<LatestMode>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
}

impl LatestGrpcRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        ensure_supported_regime_tf(self.tf, "latest outputs")?;
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<RegimeOutputMode, SdkError> {
        infer_output_mode(
            self.family.as_deref(),
            self.group.as_deref(),
            self.secondary,
            self.metadata,
        )
    }
}

impl From<&LatestRequest> for LatestGrpcRequest {
    fn from(value: &LatestRequest) -> Self {
        Self {
            pairs: value.pairs.clone(),
            tf: value.tf,
            latest_mode: value.latest_mode,
            family: value.family.clone(),
            group: value.group.clone(),
            secondary: value.secondary,
            metadata: value.metadata,
            diagnostics: value.diagnostics,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RangeRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub align_mode: Option<AlignMode>,
    pub close_start: Option<TimeInput>,
    pub cursor: Option<String>,
    pub close_end: Option<TimeInput>,
    pub limit: Option<i64>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub format: Option<HttpFormat>,
}

impl RangeRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        ensure_supported_regime_tf(self.tf, "range outputs")?;
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<RegimeOutputMode, SdkError> {
        infer_output_mode(
            self.family.as_deref(),
            self.group.as_deref(),
            self.secondary,
            self.metadata,
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RangeGrpcRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub align_mode: Option<AlignMode>,
    pub close_start: Option<TimeInput>,
    pub cursor: Option<String>,
    pub close_end: Option<TimeInput>,
    pub limit: Option<i64>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
}

impl RangeGrpcRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        ensure_supported_regime_tf(self.tf, "range outputs")?;
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<RegimeOutputMode, SdkError> {
        infer_output_mode(
            self.family.as_deref(),
            self.group.as_deref(),
            self.secondary,
            self.metadata,
        )
    }
}

impl From<&RangeRequest> for RangeGrpcRequest {
    fn from(value: &RangeRequest) -> Self {
        Self {
            pairs: value.pairs.clone(),
            tf: value.tf,
            align_mode: value.align_mode,
            close_start: value.close_start.clone(),
            cursor: value.cursor.clone(),
            close_end: value.close_end.clone(),
            limit: value.limit,
            family: value.family.clone(),
            group: value.group.clone(),
            secondary: value.secondary,
            metadata: value.metadata,
            diagnostics: value.diagnostics,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SearchRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: String,
    pub evaluate_pair: Option<String>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub max_hits: Option<i64>,
    pub format: Option<HttpFormat>,
}

impl SearchRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        ensure_supported_regime_tf(self.tf, "search outputs")?;
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<RegimeOutputMode, SdkError> {
        infer_output_mode(
            self.family.as_deref(),
            self.group.as_deref(),
            self.secondary,
            self.metadata,
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SearchGrpcRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: String,
    pub evaluate_pair: Option<String>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub max_hits: Option<i64>,
}

impl SearchGrpcRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        ensure_supported_regime_tf(self.tf, "search outputs")?;
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<RegimeOutputMode, SdkError> {
        infer_output_mode(
            self.family.as_deref(),
            self.group.as_deref(),
            self.secondary,
            self.metadata,
        )
    }
}

impl From<&SearchRequest> for SearchGrpcRequest {
    fn from(value: &SearchRequest) -> Self {
        Self {
            tf: value.tf,
            close_start: value.close_start.clone(),
            close_end: value.close_end.clone(),
            cursor: value.cursor.clone(),
            predicate: value.predicate.clone(),
            evaluate_pair: value.evaluate_pair.clone(),
            family: value.family.clone(),
            group: value.group.clone(),
            secondary: value.secondary,
            metadata: value.metadata,
            diagnostics: value.diagnostics,
            max_hits: value.max_hits,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TimeMachineRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: Option<String>,
    pub hits: Option<Vec<i64>>,
    pub output_pairs: Option<Vec<String>>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub before_bars: Option<i64>,
    pub after_bars: Option<i64>,
    pub max_hits: Option<i64>,
    pub overlap_mode: Option<String>,
    pub format: Option<HttpFormat>,
}

impl TimeMachineRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        ensure_supported_regime_tf(self.tf, "time machine outputs")?;
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<RegimeOutputMode, SdkError> {
        infer_output_mode(
            self.family.as_deref(),
            self.group.as_deref(),
            self.secondary,
            self.metadata,
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TimeMachineGrpcRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: Option<String>,
    pub hits: Option<Vec<i64>>,
    pub output_pairs: Option<Vec<String>>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub before_bars: Option<i64>,
    pub after_bars: Option<i64>,
    pub max_hits: Option<i64>,
    pub overlap_mode: Option<String>,
}

impl TimeMachineGrpcRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        ensure_supported_regime_tf(self.tf, "time machine outputs")?;
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<RegimeOutputMode, SdkError> {
        infer_output_mode(
            self.family.as_deref(),
            self.group.as_deref(),
            self.secondary,
            self.metadata,
        )
    }
}

impl From<&TimeMachineRequest> for TimeMachineGrpcRequest {
    fn from(value: &TimeMachineRequest) -> Self {
        Self {
            tf: value.tf,
            close_start: value.close_start.clone(),
            close_end: value.close_end.clone(),
            cursor: value.cursor.clone(),
            predicate: value.predicate.clone(),
            hits: value.hits.clone(),
            output_pairs: value.output_pairs.clone(),
            family: value.family.clone(),
            group: value.group.clone(),
            secondary: value.secondary,
            metadata: value.metadata,
            diagnostics: value.diagnostics,
            before_bars: value.before_bars,
            after_bars: value.after_bars,
            max_hits: value.max_hits,
            overlap_mode: value.overlap_mode.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub(crate) struct NormalizedLatestOutputsRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub latest_mode: Option<LatestMode>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<String>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub(crate) struct NormalizedRangeOutputsRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub align_mode: Option<AlignMode>,
    #[serde(rename = "close_start_ms")]
    pub close_start_ms: Option<i64>,
    pub cursor: Option<String>,
    #[serde(rename = "close_end_ms")]
    pub close_end_ms: Option<i64>,
    pub limit: Option<i64>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<String>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub(crate) struct NormalizedSearchOutputsRequest {
    pub tf: Timeframe,
    #[serde(rename = "close_start_ms")]
    pub close_start_ms: i64,
    #[serde(rename = "close_end_ms")]
    pub close_end_ms: Option<i64>,
    pub cursor: Option<String>,
    pub predicate: String,
    pub evaluate_pair: Option<String>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<String>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub max_hits: Option<i64>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub(crate) struct NormalizedTimeMachineOutputsRequest {
    pub tf: Timeframe,
    #[serde(rename = "close_start_ms")]
    pub close_start_ms: i64,
    #[serde(rename = "close_end_ms")]
    pub close_end_ms: Option<i64>,
    pub cursor: Option<String>,
    pub predicate: Option<String>,
    pub hits: Option<Vec<i64>>,
    pub output_pairs: Option<Vec<String>>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<String>>,
    pub secondary: Option<bool>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub before_bars: Option<i64>,
    pub after_bars: Option<i64>,
    pub max_hits: Option<i64>,
    pub overlap_mode: Option<String>,
    pub format: Option<HttpFormat>,
}

impl LatestRequest {
    pub(crate) fn normalize_http(&self) -> Result<NormalizedLatestOutputsRequest, SdkError> {
        self.validate()?;
        Ok(NormalizedLatestOutputsRequest {
            pairs: normalize_required_pair_values(&self.pairs, "latest outputs")?,
            tf: self.tf,
            latest_mode: self.latest_mode,
            family: normalize_family_selectors(self.family.as_deref()),
            group: normalize_group_selector_names(self.group.as_deref()),
            secondary: self.secondary,
            metadata: self.metadata,
            diagnostics: self.diagnostics,
            format: self.format,
        })
    }
}

impl LatestGrpcRequest {
    pub(crate) fn to_proto(&self) -> Result<proto::LatestOutputsRequestV1, SdkError> {
        self.validate()?;
        Ok(proto::LatestOutputsRequestV1 {
            pairs: normalize_required_pair_values(&self.pairs, "latest outputs")?,
            tf: self.tf.as_str().to_string(),
            latest_mode: self
                .latest_mode
                .unwrap_or(LatestMode::ExactWatermark)
                .as_str()
                .to_string(),
            exclude_sources: Vec::new(),
            metadata: self.metadata.unwrap_or(false),
            family: selector_family_names(self.family.as_deref()),
            group: selector_group_names(self.group.as_deref()),
            diagnostics: self.diagnostics,
            secondary: self.secondary.unwrap_or(false),
        })
    }
}

impl RangeRequest {
    pub(crate) fn normalize_http(&self) -> Result<NormalizedRangeOutputsRequest, SdkError> {
        self.validate()?;
        Ok(NormalizedRangeOutputsRequest {
            pairs: normalize_required_pair_values(&self.pairs, "range outputs")?,
            tf: self.tf,
            align_mode: self.align_mode,
            close_start_ms: self
                .close_start
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?,
            cursor: normalize_optional_string(self.cursor.as_deref()),
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?,
            limit: self.limit,
            family: normalize_family_selectors(self.family.as_deref()),
            group: normalize_group_selector_names(self.group.as_deref()),
            secondary: self.secondary,
            metadata: self.metadata,
            diagnostics: self.diagnostics,
            format: self.format,
        })
    }
}

impl RangeGrpcRequest {
    pub(crate) fn to_proto(&self) -> Result<proto::RangeOutputsRequestV1, SdkError> {
        self.validate()?;
        Ok(proto::RangeOutputsRequestV1 {
            pairs: normalize_required_pair_values(&self.pairs, "range outputs")?,
            tf: self.tf.as_str().to_string(),
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?
                .unwrap_or(0),
            cursor: normalize_optional_string(self.cursor.as_deref()),
            limit: self.limit,
            exclude_sources: Vec::new(),
            metadata: self.metadata.unwrap_or(false),
            close_start_ms: self
                .close_start
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?
                .unwrap_or(0),
            align_mode: self.align_mode.map(|mode| mode.as_str().to_string()),
            family: selector_family_names(self.family.as_deref()),
            group: selector_group_names(self.group.as_deref()),
            diagnostics: self.diagnostics,
            secondary: self.secondary.unwrap_or(false),
        })
    }
}

impl SearchRequest {
    pub(crate) fn normalize_http(&self) -> Result<NormalizedSearchOutputsRequest, SdkError> {
        self.validate()?;
        Ok(NormalizedSearchOutputsRequest {
            tf: self.tf,
            close_start_ms: self.close_start.to_utc_ms()?,
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?,
            cursor: normalize_optional_string(self.cursor.as_deref()),
            predicate: normalize_required_string(&self.predicate, "search outputs predicate")?,
            evaluate_pair: normalize_optional_string(self.evaluate_pair.as_deref()),
            family: normalize_family_selectors(self.family.as_deref()),
            group: normalize_group_selector_names(self.group.as_deref()),
            secondary: self.secondary,
            metadata: self.metadata,
            diagnostics: self.diagnostics,
            max_hits: self.max_hits,
            format: self.format,
        })
    }
}

impl SearchGrpcRequest {
    pub(crate) fn to_proto(&self) -> Result<proto::SearchOutputsRequestV1, SdkError> {
        self.validate()?;
        Ok(proto::SearchOutputsRequestV1 {
            tf: self.tf.as_str().to_string(),
            close_start_ms: self.close_start.to_utc_ms()?,
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?
                .unwrap_or(0),
            cursor: normalize_optional_string(self.cursor.as_deref()),
            predicate: normalize_required_string(&self.predicate, "search outputs predicate")?,
            evaluate_pair: normalize_optional_string(self.evaluate_pair.as_deref()),
            exclude_sources: Vec::new(),
            metadata: self.metadata.unwrap_or(false),
            max_hits: self.max_hits,
            family: selector_family_names(self.family.as_deref()),
            group: selector_group_names(self.group.as_deref()),
            diagnostics: self.diagnostics,
            secondary: self.secondary.unwrap_or(false),
        })
    }
}

impl TimeMachineRequest {
    pub(crate) fn normalize_http(&self) -> Result<NormalizedTimeMachineOutputsRequest, SdkError> {
        self.validate()?;
        Ok(NormalizedTimeMachineOutputsRequest {
            tf: self.tf,
            close_start_ms: self.close_start.to_utc_ms()?,
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?,
            cursor: normalize_optional_string(self.cursor.as_deref()),
            predicate: self
                .predicate
                .as_deref()
                .map(|value| normalize_required_string(value, "time machine predicate"))
                .transpose()?,
            hits: self.hits.clone(),
            output_pairs: normalize_optional_pair_values(self.output_pairs.as_deref()),
            family: normalize_family_selectors(self.family.as_deref()),
            group: normalize_group_selector_names(self.group.as_deref()),
            secondary: self.secondary,
            metadata: self.metadata,
            diagnostics: self.diagnostics,
            before_bars: self.before_bars,
            after_bars: self.after_bars,
            max_hits: self.max_hits,
            overlap_mode: normalize_optional_string(self.overlap_mode.as_deref()),
            format: self.format,
        })
    }
}

impl TimeMachineGrpcRequest {
    pub(crate) fn to_proto(&self) -> Result<proto::TimeMachineOutputsRequestV1, SdkError> {
        self.validate()?;
        Ok(proto::TimeMachineOutputsRequestV1 {
            tf: self.tf.as_str().to_string(),
            close_start_ms: self.close_start.to_utc_ms()?,
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?
                .unwrap_or(0),
            cursor: normalize_optional_string(self.cursor.as_deref()),
            predicate: self
                .predicate
                .as_deref()
                .map(|value| normalize_required_string(value, "time machine predicate"))
                .transpose()?,
            hits: self.hits.clone().unwrap_or_default(),
            output_pairs: normalize_optional_pair_values(self.output_pairs.as_deref())
                .unwrap_or_default(),
            exclude_sources: Vec::new(),
            metadata: self.metadata.unwrap_or(false),
            before_bars: self.before_bars,
            after_bars: self.after_bars,
            max_hits: self.max_hits,
            overlap_mode: normalize_optional_string(self.overlap_mode.as_deref()),
            family: selector_family_names(self.family.as_deref()),
            group: selector_group_names(self.group.as_deref()),
            diagnostics: self.diagnostics,
            secondary: self.secondary.unwrap_or(false),
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct LatestPresentRow {
    pub row: OutputRow,
    pub age_ms: i64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct LatestResponse {
    pub watermark_end_ms: i64,
    pub close_end_ms: i64,
    pub latest_mode: LatestMode,
    pub view: OutputView,
    pub rows: Vec<LatestPresentRow>,
    pub missing_pairs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RangeResponse {
    pub rows: Vec<OutputRow>,
    pub close_end_ms: i64,
    pub next_cursor: Option<String>,
}

impl RangeResponse {
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor.as_deref()
    }

    pub fn close_end_ms(&self) -> i64 {
        self.close_end_ms
    }

    pub fn done(&self) -> bool {
        self.next_cursor.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct RangeTraverseResult {
    pub pages: Vec<RangeResponse>,
    pub pages_fetched: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SearchResponse {
    pub hits: Vec<i64>,
    pub evaluated_rows: Option<Vec<OutputRow>>,
    pub next_cursor: Option<String>,
    pub done: bool,
    pub returned_hits: i64,
    pub effective_hits_limit: i64,
    pub truncated: bool,
    pub predicate_pairs: Vec<String>,
    pub predicate_normalized: String,
}

impl SearchResponse {
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor.as_deref()
    }

    pub fn done(&self) -> bool {
        self.done
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SearchTraverseResult {
    pub pages: Vec<SearchResponse>,
    pub pages_fetched: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TimeMachineRow {
    pub hit_close_ms: i64,
    pub offset: i64,
    pub row: OutputRow,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TimeMachineResponse {
    pub rows: Vec<TimeMachineRow>,
    pub next_cursor: Option<String>,
    pub done: bool,
    pub returned_hits: i64,
    pub effective_hits_limit: i64,
    pub truncated: bool,
    pub predicate_pairs: Vec<String>,
    pub predicate_normalized: Option<String>,
}

impl TimeMachineResponse {
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor.as_deref()
    }

    pub fn done(&self) -> bool {
        self.done
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct TimeMachineTraverseResult {
    pub pages: Vec<TimeMachineResponse>,
    pub pages_fetched: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct OutputRowWire {
    pair: String,
    tf: String,
    open_ms: i64,
    close_ms: i64,
    open_utc: String,
    close_utc: String,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: f64,
    quote_v: Option<f64>,
    taker_known_v: Option<f64>,
    taker_signed_v: Option<f64>,
    taker_known_quote_v: Option<f64>,
    taker_signed_quote_v: Option<f64>,
    taker_known_n: Option<i64>,
    taker_signed_n: Option<i64>,
    vw: Option<f64>,
    n: Option<i64>,
    #[serde(default)]
    metadata: Option<OutputMetadata>,
    #[serde(default)]
    diagnostics: Option<Vec<OutputProcessDiagnostic>>,
    #[serde(flatten)]
    computed: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct LatestOutputsHttpPresentRowWire {
    #[serde(flatten)]
    row: OutputRowWire,
    age_ms: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct LatestOutputsWsPresentRowWire {
    #[serde(flatten)]
    row: OutputRowWire,
    #[serde(default)]
    age_ms: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct LatestOutputsHttpResponseWire {
    watermark_end_ms: i64,
    close_end_ms: i64,
    latest_mode: LatestMode,
    view: OutputView,
    rows: Vec<LatestOutputsHttpPresentRowWire>,
    missing_pairs: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct RangeOutputsResponseWire {
    rows: Vec<OutputRowWire>,
    close_end_ms: i64,
    next_cursor: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct SearchOutputsResponseWire {
    hits: Vec<i64>,
    evaluated_rows: Option<Vec<OutputRowWire>>,
    next_cursor: Option<String>,
    done: bool,
    returned_hits: i64,
    effective_hits_limit: i64,
    truncated: bool,
    predicate_pairs: Vec<String>,
    predicate_normalized: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct TimeMachineOutputsRowWire {
    hit_close_ms: i64,
    offset: i64,
    #[serde(rename = "output")]
    row: OutputRowWire,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct TimeMachineOutputsResponseWire {
    rows: Vec<TimeMachineOutputsRowWire>,
    next_cursor: Option<String>,
    done: bool,
    returned_hits: i64,
    effective_hits_limit: i64,
    truncated: bool,
    predicate_pairs: Vec<String>,
    predicate_normalized: Option<String>,
}

impl OutputRowWire {
    fn into_public(
        self,
        metadata_required: bool,
        diagnostics_enabled: bool,
        context: &'static str,
    ) -> Result<OutputRow, SdkError> {
        validate_computed_fields(&self.computed, context)?;
        let row = OutputRow {
            pair: self.pair,
            tf: self.tf,
            open_ms: self.open_ms,
            close_ms: self.close_ms,
            open_utc: self.open_utc,
            close_utc: self.close_utc,
            o: self.o,
            h: self.h,
            l: self.l,
            c: self.c,
            v: self.v,
            quote_v: self.quote_v,
            taker_known_v: self.taker_known_v,
            taker_signed_v: self.taker_signed_v,
            taker_known_quote_v: self.taker_known_quote_v,
            taker_signed_quote_v: self.taker_signed_quote_v,
            taker_known_n: self.taker_known_n,
            taker_signed_n: self.taker_signed_n,
            vw: self.vw,
            n: self.n,
            metadata: self.metadata,
            diagnostics: self.diagnostics,
            computed: ComputedFields(self.computed),
        }
        .apply_diagnostics_gate(diagnostics_enabled);
        row.ensure_metadata_shape(metadata_required, context)?;
        Ok(row)
    }
}

impl OutputRow {
    pub(crate) fn from_proto(
        value: proto::OutputRowV1,
        mode: RegimeOutputMode,
        diagnostics_enabled: bool,
        context: &'static str,
    ) -> Result<Self, SdkError> {
        let computed = collect_numeric_computed_fields(&value, context)?;
        validate_computed_fields(&computed, context)?;

        let row = OutputRow {
            pair: value.pair,
            tf: value.tf,
            open_ms: value.open_ms,
            close_ms: value.close_ms,
            open_utc: value
                .open_utc
                .ok_or_else(|| SdkError::contract_drift(format!("{context} missing `open_utc`")))?,
            close_utc: value.close_utc.ok_or_else(|| {
                SdkError::contract_drift(format!("{context} missing `close_utc`"))
            })?,
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
            metadata: value.metadata.map(output_metadata_from_proto),
            diagnostics: Some(
                value
                    .diagnostics
                    .into_iter()
                    .map(output_process_diagnostic_from_proto)
                    .collect(),
            ),
            computed: ComputedFields(computed),
        }
        .apply_diagnostics_gate(diagnostics_enabled);
        row.ensure_metadata_shape(mode.has_metadata(), context)?;
        Ok(row)
    }
}

impl LatestPresentRow {
    pub(crate) fn from_proto(
        value: proto::OutputsPresentRowV1,
        mode: RegimeOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        Ok(Self {
            row: OutputRow::from_proto(
                value.output.ok_or_else(|| {
                    SdkError::contract_drift("latest outputs protobuf row missing `output`")
                })?,
                mode,
                diagnostics_enabled,
                "latest outputs protobuf row",
            )?,
            age_ms: value.age_ms.ok_or_else(|| {
                SdkError::contract_drift("latest outputs protobuf row missing `age_ms`")
            })?,
        })
    }
}

impl LatestResponse {
    #[doc(hidden)]
    pub fn from_grpc_proto(
        response: proto::OutputsLatestResponseV1,
        metadata_required: bool,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        let mode = if metadata_required {
            RegimeOutputMode::WithMeta
        } else {
            RegimeOutputMode::Min
        };
        Self::from_proto(response, mode, diagnostics_enabled)
    }

    pub(crate) fn from_http(
        wire: LatestOutputsHttpResponseWire,
        mode: RegimeOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        ensure_latest_view_matches_mode(wire.view, mode, "latest outputs")?;
        let metadata_required = mode.has_metadata();
        Ok(Self {
            watermark_end_ms: wire.watermark_end_ms,
            close_end_ms: wire.close_end_ms,
            latest_mode: wire.latest_mode,
            view: wire.view,
            rows: wire
                .rows
                .into_iter()
                .map(|row| {
                    Ok(LatestPresentRow {
                        row: row.row.into_public(
                            metadata_required,
                            diagnostics_enabled,
                            "latest outputs response row",
                        )?,
                        age_ms: row.age_ms,
                    })
                })
                .collect::<Result<Vec<_>, SdkError>>()?,
            missing_pairs: wire.missing_pairs,
        })
    }

    pub(crate) fn from_proto(
        response: proto::OutputsLatestResponseV1,
        mode: RegimeOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        ensure_proto_output_mode_supported(mode, "latest outputs protobuf/gRPC")?;
        let view = OutputView::from_proto(response.view)?;
        ensure_latest_view_matches_mode(view, mode, "latest outputs")?;
        Ok(Self {
            watermark_end_ms: response.watermark_end_ms,
            close_end_ms: response.close_end_ms,
            latest_mode: latest_mode_from_proto(&response.latest_mode)?,
            view,
            rows: response
                .rows
                .into_iter()
                .map(|row| LatestPresentRow::from_proto(row, mode, diagnostics_enabled))
                .collect::<Result<Vec<_>, _>>()?,
            missing_pairs: response.missing_pairs,
        })
    }
}

impl RangeResponse {
    #[doc(hidden)]
    pub fn from_grpc_proto(
        response: proto::OutputsRangeResponseV1,
        metadata_required: bool,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        let mode = if metadata_required {
            RegimeOutputMode::WithMeta
        } else {
            RegimeOutputMode::Min
        };
        Self::from_proto(response, mode, diagnostics_enabled)
    }

    pub(crate) fn from_http(
        wire: RangeOutputsResponseWire,
        mode: RegimeOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        let metadata_required = mode.has_metadata();
        Ok(Self {
            rows: wire
                .rows
                .into_iter()
                .map(|row| {
                    row.into_public(metadata_required, diagnostics_enabled, "range outputs row")
                })
                .collect::<Result<Vec<_>, _>>()?,
            close_end_ms: wire.close_end_ms,
            next_cursor: wire.next_cursor,
        })
    }

    pub(crate) fn from_proto(
        response: proto::OutputsRangeResponseV1,
        mode: RegimeOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        ensure_proto_output_mode_supported(mode, "range outputs protobuf/gRPC")?;
        Ok(Self {
            rows: response
                .rows
                .into_iter()
                .map(|row| {
                    OutputRow::from_proto(row, mode, diagnostics_enabled, "range outputs row")
                })
                .collect::<Result<Vec<_>, _>>()?,
            close_end_ms: response.close_end_ms,
            next_cursor: response.next_cursor,
        })
    }
}

impl SearchResponse {
    #[doc(hidden)]
    pub fn from_grpc_proto(
        response: proto::OutputsSearchResponseV1,
        metadata_required: bool,
        diagnostics_enabled: bool,
        evaluated_rows_enabled: bool,
    ) -> Result<Self, SdkError> {
        let mode = if metadata_required {
            RegimeOutputMode::WithMeta
        } else {
            RegimeOutputMode::Min
        };
        Self::from_proto(response, mode, diagnostics_enabled, evaluated_rows_enabled)
    }

    pub(crate) fn from_http(
        wire: SearchOutputsResponseWire,
        mode: RegimeOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        let metadata_required = mode.has_metadata();
        Ok(Self {
            hits: wire.hits,
            evaluated_rows: wire
                .evaluated_rows
                .map(|rows| {
                    rows.into_iter()
                        .map(|row| {
                            row.into_public(
                                metadata_required,
                                diagnostics_enabled,
                                "search outputs evaluated row",
                            )
                        })
                        .collect::<Result<Vec<_>, SdkError>>()
                })
                .transpose()?,
            next_cursor: wire.next_cursor,
            done: wire.done,
            returned_hits: wire.returned_hits,
            effective_hits_limit: wire.effective_hits_limit,
            truncated: wire.truncated,
            predicate_pairs: wire.predicate_pairs,
            predicate_normalized: wire.predicate_normalized,
        })
    }

    pub(crate) fn from_proto(
        response: proto::OutputsSearchResponseV1,
        mode: RegimeOutputMode,
        diagnostics_enabled: bool,
        evaluated_rows_enabled: bool,
    ) -> Result<Self, SdkError> {
        ensure_proto_output_mode_supported(mode, "search outputs protobuf/gRPC")?;
        let evaluated_rows = if evaluated_rows_enabled {
            Some(
                response
                    .evaluated_rows
                    .into_iter()
                    .map(|row| {
                        OutputRow::from_proto(
                            row,
                            mode,
                            diagnostics_enabled,
                            "search outputs evaluated row",
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )
        } else if response.evaluated_rows.is_empty() {
            None
        } else {
            return Err(SdkError::contract_drift(
                "search outputs protobuf response returned `evaluated_rows` without `evaluate_pair` in the request",
            ));
        };

        Ok(Self {
            hits: response.hits,
            evaluated_rows,
            next_cursor: response.next_cursor,
            done: response.done,
            returned_hits: response.returned_hits,
            effective_hits_limit: response.effective_hits_limit,
            truncated: response.truncated,
            predicate_pairs: response.predicate_pairs,
            predicate_normalized: response.predicate_normalized,
        })
    }
}

impl TimeMachineRow {
    fn from_proto(
        value: proto::OutputsTimeMachineRowV1,
        mode: RegimeOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        Ok(Self {
            hit_close_ms: value.hit_close_ms,
            offset: value.offset,
            row: OutputRow::from_proto(
                value.output.ok_or_else(|| {
                    SdkError::contract_drift("time machine outputs protobuf row missing `output`")
                })?,
                mode,
                diagnostics_enabled,
                "time machine outputs row",
            )?,
        })
    }
}

impl TimeMachineResponse {
    #[doc(hidden)]
    pub fn from_grpc_proto(
        response: proto::OutputsTimeMachineResponseV1,
        metadata_required: bool,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        let mode = if metadata_required {
            RegimeOutputMode::WithMeta
        } else {
            RegimeOutputMode::Min
        };
        Self::from_proto(response, mode, diagnostics_enabled)
    }

    pub(crate) fn from_http(
        wire: TimeMachineOutputsResponseWire,
        mode: RegimeOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        let metadata_required = mode.has_metadata();
        Ok(Self {
            rows: wire
                .rows
                .into_iter()
                .map(|row| {
                    Ok(TimeMachineRow {
                        hit_close_ms: row.hit_close_ms,
                        offset: row.offset,
                        row: row.row.into_public(
                            metadata_required,
                            diagnostics_enabled,
                            "time machine outputs row",
                        )?,
                    })
                })
                .collect::<Result<Vec<_>, SdkError>>()?,
            next_cursor: wire.next_cursor,
            done: wire.done,
            returned_hits: wire.returned_hits,
            effective_hits_limit: wire.effective_hits_limit,
            truncated: wire.truncated,
            predicate_pairs: wire.predicate_pairs,
            predicate_normalized: wire.predicate_normalized,
        })
    }

    pub(crate) fn from_proto(
        response: proto::OutputsTimeMachineResponseV1,
        mode: RegimeOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        ensure_proto_output_mode_supported(mode, "time-machine outputs protobuf/gRPC")?;
        Ok(Self {
            rows: response
                .rows
                .into_iter()
                .map(|row| TimeMachineRow::from_proto(row, mode, diagnostics_enabled))
                .collect::<Result<Vec<_>, _>>()?,
            next_cursor: response.next_cursor,
            done: response.done,
            returned_hits: response.returned_hits,
            effective_hits_limit: response.effective_hits_limit,
            truncated: response.truncated,
            predicate_pairs: response.predicate_pairs,
            predicate_normalized: response.predicate_normalized,
        })
    }
}

impl OutputView {
    fn from_proto(value: i32) -> Result<Self, SdkError> {
        match proto::OutputsViewV1::try_from(value) {
            Ok(proto::OutputsViewV1::Min) => Ok(Self::Min),
            Ok(proto::OutputsViewV1::Full) => Ok(Self::Full),
            _ => Err(SdkError::contract_drift(format!(
                "unsupported outputs view `{value}`"
            ))),
        }
    }
}

pub(crate) fn diagnostics_enabled(value: Option<bool>) -> bool {
    value.unwrap_or(false)
}

pub(crate) fn decode_latest_outputs_ws_json(
    text: &str,
    mode: RegimeOutputMode,
    diagnostics_enabled: bool,
) -> Result<Vec<LatestPresentRow>, SdkError> {
    let rows =
        serde_json::from_str::<Vec<LatestOutputsWsPresentRowWire>>(text).map_err(|source| {
            SdkError::contract_drift(format!("outputs ws JSON rows decode failed: {source}"))
        })?;
    let metadata_required = mode.has_metadata();
    rows.into_iter()
        .map(|row| {
            Ok(LatestPresentRow {
                row: row.row.into_public(
                    metadata_required,
                    diagnostics_enabled,
                    "outputs ws row",
                )?,
                age_ms: row.age_ms,
            })
        })
        .collect()
}

pub(crate) fn decode_latest_outputs_ws_proto(
    bytes: &[u8],
    mode: RegimeOutputMode,
    diagnostics_enabled: bool,
) -> Result<Vec<LatestPresentRow>, SdkError> {
    ensure_proto_output_mode_supported(mode, "outputs ws protobuf rows")?;
    let payload = proto::OutputsRowsPayloadV1::decode(bytes).map_err(|source| {
        SdkError::contract_drift(format!(
            "outputs ws protobuf payload decode failed: {source}"
        ))
    })?;
    let view = OutputView::from_proto(payload.view)?;
    let expected = expected_output_view(mode);
    if view != expected {
        return Err(SdkError::contract_drift(format!(
            "outputs ws protobuf payload view `{}` did not match request mode `{}`",
            view.as_str(),
            expected.as_str()
        )));
    }

    payload
        .rows
        .into_iter()
        .map(|row| LatestPresentRow::from_proto(row, mode, diagnostics_enabled))
        .collect()
}

fn latest_mode_from_proto(value: &str) -> Result<LatestMode, SdkError> {
    match value {
        "exact_watermark" => Ok(LatestMode::ExactWatermark),
        "latest_available_le_watermark" => Ok(LatestMode::LatestAvailableLeWatermark),
        other => Err(SdkError::contract_drift(format!(
            "unsupported latest_mode `{other}`"
        ))),
    }
}

fn validate_computed_fields(
    computed: &serde_json::Map<String, serde_json::Value>,
    context: &'static str,
) -> Result<(), SdkError> {
    for (key, value) in computed {
        if !is_known_processor_field(key.as_str()) && !PROCESSOR_FIELD_NAMES.contains(&key.as_str())
        {
            return Err(SdkError::contract_drift(format!(
                "{context} contained unknown computed field `{key}`"
            )));
        }

        if !(value.is_null() || value.is_number()) {
            return Err(SdkError::contract_drift(format!(
                "{context} computed field `{key}` must be null or numeric"
            )));
        }
    }

    Ok(())
}

fn output_process_diagnostic_from_proto(
    value: proto::OutputProcessDiagnosticV1,
) -> OutputProcessDiagnostic {
    OutputProcessDiagnostic {
        indicator: value.indicator,
        message: value.message,
    }
}

fn output_metadata_from_proto(value: proto::OutputMetadataV1) -> OutputMetadata {
    OutputMetadata {
        source: value.source,
        process: value.process,
        bars_input_n: value.bars_input_n,
        recompute_upstream_lifeline_id: value.recompute_upstream_lifeline_id,
        recompute_upstream_reason: value.recompute_upstream_reason,
        recomputed_at_ms: value.recomputed_at_ms,
        recomputed_at_utc: value.recomputed_at_utc,
        computed_at_ms: value.computed_at_ms,
        computed_at_utc: value.computed_at_utc,
        tail_bar_provenance: value
            .tail_bar_provenance
            .map(output_bars_metadata_from_proto)
            .unwrap_or_default(),
    }
}

fn output_bars_metadata_from_proto(value: proto::OutputBarsMetadataV1) -> OutputBarsMetadata {
    OutputBarsMetadata {
        source: value.source,
        venues_expected: repeated_strings_or_none(value.venues_expected),
        venues_with_trades: repeated_strings_or_none(value.venues_with_trades),
        ingested_at_ms: value.ingested_at_ms,
        ingested_at_utc: value.ingested_at_utc,
        target_ingested_at_ms: value.target_ingested_at_ms,
        target_ingested_at_utc: value.target_ingested_at_utc,
        committed_at_ms: value.committed_at_ms,
        committed_at_utc: value.committed_at_utc,
        harmonized_at_ms: value.harmonized_at_ms,
        harmonized_at_utc: value.harmonized_at_utc,
        frontier_5s_expected: value.frontier_5s_expected,
        frontier_5s_synth_n: value.frontier_5s_synth_n,
        frontier_5s_synth_ratio: value.frontier_5s_synth_ratio,
        frontier_5s_trade_n: value.frontier_5s_trade_n,
        frontier_5s_trade_ratio: value.frontier_5s_trade_ratio,
        process: value.process,
        built_at_ms: value.built_at_ms,
        built_at_utc: value.built_at_utc,
        covered_1m_count: value.covered_1m_count,
        expected_1m_count: value.expected_1m_count,
        coverage_ratio: value.coverage_ratio,
        inputs_source_counts_frontier: value.inputs_source_counts_frontier,
        inputs_source_counts_api: value.inputs_source_counts_api,
        inputs_source_counts_synthetic: value.inputs_source_counts_synthetic,
        inputs_source_counts_fix_data: value.inputs_source_counts_fix_data,
        frontier_5s_inputs_coverage_ratio: value.frontier_5s_inputs_coverage_ratio,
        recomputed_at_ms: value.recomputed_at_ms,
        recomputed_at_utc: value.recomputed_at_utc,
        recomputed_reason: value.recomputed_reason,
    }
}

fn repeated_strings_or_none(values: Vec<String>) -> Option<Vec<String>> {
    (!values.is_empty()).then_some(values)
}

fn expected_output_view(mode: RegimeOutputMode) -> OutputView {
    if mode.has_metadata() {
        OutputView::Full
    } else {
        OutputView::Min
    }
}

fn ensure_latest_view_matches_mode(
    view: OutputView,
    mode: RegimeOutputMode,
    context: &'static str,
) -> Result<(), SdkError> {
    let expected = expected_output_view(mode);
    if view == expected {
        return Ok(());
    }

    Err(SdkError::contract_drift(format!(
        "{context} response view `{}` did not match request mode `{}`",
        view.as_str(),
        expected.as_str()
    )))
}

pub(crate) fn ensure_proto_output_mode_supported(
    mode: RegimeOutputMode,
    context: &'static str,
) -> Result<(), SdkError> {
    if mode.is_projected() {
        return Err(SdkError::unsupported_or_unproved_usage(format!(
            "{context} is not proved for projected selector mode because protobuf leaves unselected computed fields unset"
        )));
    }
    Ok(())
}

pub(crate) fn normalize_pair_values(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
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

pub(crate) fn normalize_optional_pair_values(values: Option<&[String]>) -> Option<Vec<String>> {
    let normalized = normalize_pair_values(values?);
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn normalize_required_string(value: &str, context: &'static str) -> Result<String, SdkError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(SdkError::request_build(format!(
            "{context} must not be blank"
        )));
    }
    Ok(normalized.to_string())
}

pub(crate) fn normalize_optional_string(value: Option<&str>) -> Option<String> {
    let normalized = value?.trim();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_string())
    }
}

pub(crate) fn normalize_family_selectors(
    values: Option<&[ProcessorFamily]>,
) -> Option<Vec<ProcessorFamily>> {
    let values = values?;
    if values.is_empty() {
        None
    } else {
        Some(values.to_vec())
    }
}

pub(crate) fn normalize_group_selectors(
    values: Option<&[ProcessorGroup]>,
) -> Option<Vec<ProcessorGroup>> {
    let values = values?;
    if values.is_empty() {
        None
    } else {
        Some(values.to_vec())
    }
}

pub(crate) fn normalize_group_selector_names(
    values: Option<&[ProcessorGroup]>,
) -> Option<Vec<String>> {
    let values = values?;
    if values.is_empty() {
        None
    } else {
        Some(selector_group_names(Some(values)))
    }
}

pub(crate) fn selector_family_names(values: Option<&[ProcessorFamily]>) -> Vec<String> {
    values
        .unwrap_or(&[])
        .iter()
        .map(|value| value.canonical_name().to_string())
        .collect()
}

pub(crate) fn selector_group_names(values: Option<&[ProcessorGroup]>) -> Vec<String> {
    values
        .unwrap_or(&[])
        .iter()
        .map(|value| value.canonical_name().to_string())
        .collect()
}

pub(crate) fn ensure_supported_regime_tf(
    tf: Timeframe,
    context: &'static str,
) -> Result<(), SdkError> {
    if tf != Timeframe::H1 {
        return Err(SdkError::unsupported_or_unproved_usage(format!(
            "{context} supports tf=1h only in regime v1"
        )));
    }
    Ok(())
}

pub(crate) fn infer_output_mode(
    family: Option<&[ProcessorFamily]>,
    group: Option<&[ProcessorGroup]>,
    secondary: Option<bool>,
    metadata: Option<bool>,
) -> Result<RegimeOutputMode, SdkError> {
    if selects_metadata_family(family) && !metadata.unwrap_or(false) {
        return Err(SdkError::request_build(
            "family=metadata requires metadata=true",
        ));
    }

    let projected = selector_values_present(family)
        || selector_values_present(group)
        || !secondary.unwrap_or(false);
    let with_meta = metadata.unwrap_or(false);

    Ok(match (projected, with_meta) {
        (false, false) => RegimeOutputMode::Min,
        (false, true) => RegimeOutputMode::WithMeta,
        (true, false) => RegimeOutputMode::ProjectedMin,
        (true, true) => RegimeOutputMode::ProjectedWithMeta,
    })
}

fn selector_values_present<T>(values: Option<&[T]>) -> bool {
    values.is_some_and(|values| !values.is_empty())
}

fn selects_metadata_family(family: Option<&[ProcessorFamily]>) -> bool {
    family
        .unwrap_or(&[])
        .iter()
        .any(|value| value.canonical_name() == "metadata")
}
