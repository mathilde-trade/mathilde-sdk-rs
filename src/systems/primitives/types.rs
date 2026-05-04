use crate::core::error::SdkError;
use crate::core::time::TimeInput;
use crate::generated::primitives::{
    OutputMetadata, OutputProcessDiagnostic, ProcessorFamily, ProcessorGroup, ProcessorOutputMin,
    ProcessorOutputWithMeta, ProcessorProjectedOutputMin, ProcessorProjectedOutputWithMeta,
    outputs_proto::mathilde::feed::outputs::v1 as proto,
};
use crate::systems::types::{AlignMode, HttpFormat, LatestMode, Timeframe};
use prost::Message;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrimitiveOutputMode {
    Min,
    WithMeta,
    ProjectedMin,
    ProjectedWithMeta,
}

impl PrimitiveOutputMode {
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

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveOutput {
    Min(ProcessorOutputMin),
    WithMeta(ProcessorOutputWithMeta),
    ProjectedMin(ProcessorProjectedOutputMin),
    ProjectedWithMeta(ProcessorProjectedOutputWithMeta),
}

impl PrimitiveOutput {
    pub fn is_projected(&self) -> bool {
        self.mode().is_projected()
    }

    pub fn has_metadata(&self) -> bool {
        self.mode().has_metadata()
    }

    pub fn diagnostics(&self) -> Option<&[OutputProcessDiagnostic]> {
        match self {
            Self::Min(output) => output.diagnostics.as_deref(),
            Self::WithMeta(output) => output.diagnostics.as_deref(),
            Self::ProjectedMin(output) => output.diagnostics.as_deref(),
            Self::ProjectedWithMeta(output) => output.diagnostics.as_deref(),
        }
    }

    pub fn metadata(&self) -> Option<&OutputMetadata> {
        match self {
            Self::Min(_) | Self::ProjectedMin(_) => None,
            Self::WithMeta(output) => Some(&output.metadata),
            Self::ProjectedWithMeta(output) => Some(&output.metadata),
        }
    }

    pub(crate) const fn mode(&self) -> PrimitiveOutputMode {
        match self {
            Self::Min(_) => PrimitiveOutputMode::Min,
            Self::WithMeta(_) => PrimitiveOutputMode::WithMeta,
            Self::ProjectedMin(_) => PrimitiveOutputMode::ProjectedMin,
            Self::ProjectedWithMeta(_) => PrimitiveOutputMode::ProjectedWithMeta,
        }
    }

    pub(crate) fn apply_diagnostics_gate(mut self, enabled: bool) -> Self {
        if enabled {
            return self;
        }

        match &mut self {
            Self::Min(output) => output.diagnostics = None,
            Self::WithMeta(output) => output.diagnostics = None,
            Self::ProjectedMin(output) => output.diagnostics = None,
            Self::ProjectedWithMeta(output) => output.diagnostics = None,
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
pub struct LatestOutputsRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub latest_mode: Option<LatestMode>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub format: Option<HttpFormat>,
}

impl LatestOutputsRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<PrimitiveOutputMode, SdkError> {
        infer_output_mode(self.family.as_deref(), self.group.as_deref(), self.metadata)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LatestOutputsGrpcRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub latest_mode: Option<LatestMode>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
}

impl LatestOutputsGrpcRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<PrimitiveOutputMode, SdkError> {
        infer_output_mode(self.family.as_deref(), self.group.as_deref(), self.metadata)
    }
}

impl From<&LatestOutputsRequest> for LatestOutputsGrpcRequest {
    fn from(value: &LatestOutputsRequest) -> Self {
        Self {
            pairs: value.pairs.clone(),
            tf: value.tf,
            latest_mode: value.latest_mode,
            family: value.family.clone(),
            group: value.group.clone(),
            metadata: value.metadata,
            diagnostics: value.diagnostics,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RangeOutputsRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub align_mode: Option<AlignMode>,
    pub close_start: Option<TimeInput>,
    pub cursor: Option<String>,
    pub close_end: Option<TimeInput>,
    pub limit: Option<i64>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub format: Option<HttpFormat>,
}

impl RangeOutputsRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<PrimitiveOutputMode, SdkError> {
        infer_output_mode(self.family.as_deref(), self.group.as_deref(), self.metadata)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct RangeOutputsGrpcRequest {
    pub pairs: Vec<String>,
    pub tf: Timeframe,
    pub align_mode: Option<AlignMode>,
    pub close_start: Option<TimeInput>,
    pub cursor: Option<String>,
    pub close_end: Option<TimeInput>,
    pub limit: Option<i64>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
}

impl RangeOutputsGrpcRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<PrimitiveOutputMode, SdkError> {
        infer_output_mode(self.family.as_deref(), self.group.as_deref(), self.metadata)
    }
}

impl From<&RangeOutputsRequest> for RangeOutputsGrpcRequest {
    fn from(value: &RangeOutputsRequest) -> Self {
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
            metadata: value.metadata,
            diagnostics: value.diagnostics,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SearchOutputsRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: String,
    pub evaluate_pair: Option<String>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub max_hits: Option<i64>,
    pub format: Option<HttpFormat>,
}

impl SearchOutputsRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<PrimitiveOutputMode, SdkError> {
        infer_output_mode(self.family.as_deref(), self.group.as_deref(), self.metadata)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SearchOutputsGrpcRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: String,
    pub evaluate_pair: Option<String>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub max_hits: Option<i64>,
}

impl SearchOutputsGrpcRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<PrimitiveOutputMode, SdkError> {
        infer_output_mode(self.family.as_deref(), self.group.as_deref(), self.metadata)
    }
}

impl From<&SearchOutputsRequest> for SearchOutputsGrpcRequest {
    fn from(value: &SearchOutputsRequest) -> Self {
        Self {
            tf: value.tf,
            close_start: value.close_start.clone(),
            close_end: value.close_end.clone(),
            cursor: value.cursor.clone(),
            predicate: value.predicate.clone(),
            evaluate_pair: value.evaluate_pair.clone(),
            family: value.family.clone(),
            group: value.group.clone(),
            metadata: value.metadata,
            diagnostics: value.diagnostics,
            max_hits: value.max_hits,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TimeMachineOutputsRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: Option<String>,
    pub hits: Option<Vec<i64>>,
    pub output_pairs: Option<Vec<String>>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub before_bars: Option<i64>,
    pub after_bars: Option<i64>,
    pub max_hits: Option<i64>,
    pub overlap_mode: Option<String>,
    pub format: Option<HttpFormat>,
}

impl TimeMachineOutputsRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<PrimitiveOutputMode, SdkError> {
        infer_output_mode(self.family.as_deref(), self.group.as_deref(), self.metadata)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TimeMachineOutputsGrpcRequest {
    pub tf: Timeframe,
    pub close_start: TimeInput,
    pub close_end: Option<TimeInput>,
    pub cursor: Option<String>,
    pub predicate: Option<String>,
    pub hits: Option<Vec<i64>>,
    pub output_pairs: Option<Vec<String>>,
    pub family: Option<Vec<ProcessorFamily>>,
    pub group: Option<Vec<ProcessorGroup>>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub before_bars: Option<i64>,
    pub after_bars: Option<i64>,
    pub max_hits: Option<i64>,
    pub overlap_mode: Option<String>,
}

impl TimeMachineOutputsGrpcRequest {
    pub fn validate(&self) -> Result<(), SdkError> {
        let _ = self.output_mode()?;
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> Result<PrimitiveOutputMode, SdkError> {
        infer_output_mode(self.family.as_deref(), self.group.as_deref(), self.metadata)
    }
}

impl From<&TimeMachineOutputsRequest> for TimeMachineOutputsGrpcRequest {
    fn from(value: &TimeMachineOutputsRequest) -> Self {
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
    pub group: Option<Vec<ProcessorGroup>>,
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
    pub group: Option<Vec<ProcessorGroup>>,
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
    pub group: Option<Vec<ProcessorGroup>>,
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
    pub group: Option<Vec<ProcessorGroup>>,
    pub metadata: Option<bool>,
    pub diagnostics: Option<bool>,
    pub before_bars: Option<i64>,
    pub after_bars: Option<i64>,
    pub max_hits: Option<i64>,
    pub overlap_mode: Option<String>,
    pub format: Option<HttpFormat>,
}

impl LatestOutputsRequest {
    pub(crate) fn normalize_http(&self) -> Result<NormalizedLatestOutputsRequest, SdkError> {
        self.validate()?;
        Ok(NormalizedLatestOutputsRequest {
            pairs: normalize_required_pair_values(&self.pairs, "latest outputs")?,
            tf: self.tf,
            latest_mode: self.latest_mode,
            family: normalize_family_selectors(self.family.as_deref()),
            group: normalize_group_selectors(self.group.as_deref()),
            metadata: self.metadata,
            diagnostics: self.diagnostics,
            format: self.format,
        })
    }
}

impl LatestOutputsGrpcRequest {
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
        })
    }
}

impl RangeOutputsRequest {
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
            group: normalize_group_selectors(self.group.as_deref()),
            metadata: self.metadata,
            diagnostics: self.diagnostics,
            format: self.format,
        })
    }
}

impl RangeOutputsGrpcRequest {
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
        })
    }
}

impl SearchOutputsRequest {
    pub(crate) fn normalize_http(&self) -> Result<NormalizedSearchOutputsRequest, SdkError> {
        self.validate()?;
        let predicate = normalize_required_string(&self.predicate, "search outputs predicate")?;
        Ok(NormalizedSearchOutputsRequest {
            tf: self.tf,
            close_start_ms: self.close_start.to_utc_ms()?,
            close_end_ms: self
                .close_end
                .as_ref()
                .map(TimeInput::to_utc_ms)
                .transpose()?,
            cursor: normalize_optional_string(self.cursor.as_deref()),
            predicate,
            evaluate_pair: normalize_optional_string(self.evaluate_pair.as_deref()),
            family: normalize_family_selectors(self.family.as_deref()),
            group: normalize_group_selectors(self.group.as_deref()),
            metadata: self.metadata,
            diagnostics: self.diagnostics,
            max_hits: self.max_hits,
            format: self.format,
        })
    }
}

impl SearchOutputsGrpcRequest {
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
        })
    }
}

impl TimeMachineOutputsRequest {
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
            group: normalize_group_selectors(self.group.as_deref()),
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

impl TimeMachineOutputsGrpcRequest {
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
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LatestOutputsPresentRow {
    pub output: PrimitiveOutput,
    pub age_ms: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LatestOutputsResponse {
    pub watermark_end_ms: i64,
    pub close_end_ms: i64,
    pub latest_mode: LatestMode,
    pub view: OutputView,
    pub rows: Vec<LatestOutputsPresentRow>,
    pub missing_pairs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RangeOutputsResponse {
    pub rows: Vec<PrimitiveOutput>,
    pub close_end_ms: i64,
    pub next_cursor: Option<String>,
}

impl RangeOutputsResponse {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RangeOutputsTraverseResult {
    pub pages: Vec<RangeOutputsResponse>,
    pub pages_fetched: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchOutputsResponse {
    pub hits: Vec<i64>,
    pub evaluated_rows: Option<Vec<PrimitiveOutput>>,
    pub next_cursor: Option<String>,
    pub done: bool,
    pub returned_hits: i64,
    pub effective_hits_limit: i64,
    pub truncated: bool,
    pub predicate_pairs: Vec<String>,
    pub predicate_normalized: String,
}

impl SearchOutputsResponse {
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor.as_deref()
    }

    pub fn done(&self) -> bool {
        self.done
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchOutputsTraverseResult {
    pub pages: Vec<SearchOutputsResponse>,
    pub pages_fetched: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeMachineOutputsRow {
    pub hit_close_ms: i64,
    pub offset: i64,
    pub output: PrimitiveOutput,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeMachineOutputsResponse {
    pub rows: Vec<TimeMachineOutputsRow>,
    pub next_cursor: Option<String>,
    pub done: bool,
    pub returned_hits: i64,
    pub effective_hits_limit: i64,
    pub truncated: bool,
    pub predicate_pairs: Vec<String>,
    pub predicate_normalized: Option<String>,
}

impl TimeMachineOutputsResponse {
    pub fn next_cursor(&self) -> Option<&str> {
        self.next_cursor.as_deref()
    }

    pub fn done(&self) -> bool {
        self.done
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeMachineOutputsTraverseResult {
    pub pages: Vec<TimeMachineOutputsResponse>,
    pub pages_fetched: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct LatestOutputsPresentRowWire<T> {
    output: T,
    age_ms: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct LatestOutputsResponseWire<T> {
    watermark_end_ms: i64,
    close_end_ms: i64,
    latest_mode: LatestMode,
    view: OutputView,
    rows: Vec<LatestOutputsPresentRowWire<T>>,
    missing_pairs: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct RangeOutputsResponseWire<T> {
    rows: Vec<T>,
    close_end_ms: i64,
    next_cursor: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct SearchOutputsResponseWire<T> {
    hits: Vec<i64>,
    evaluated_rows: Option<Vec<T>>,
    next_cursor: Option<String>,
    done: bool,
    returned_hits: i64,
    effective_hits_limit: i64,
    truncated: bool,
    predicate_pairs: Vec<String>,
    predicate_normalized: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct TimeMachineOutputsRowWire<T> {
    hit_close_ms: i64,
    offset: i64,
    output: T,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct TimeMachineOutputsResponseWire<T> {
    rows: Vec<TimeMachineOutputsRowWire<T>>,
    next_cursor: Option<String>,
    done: bool,
    returned_hits: i64,
    effective_hits_limit: i64,
    truncated: bool,
    predicate_pairs: Vec<String>,
    predicate_normalized: Option<String>,
}

impl PrimitiveOutput {
    fn from_min(output: ProcessorOutputMin, diagnostics_enabled: bool) -> Self {
        Self::Min(output).apply_diagnostics_gate(diagnostics_enabled)
    }

    fn from_with_meta(output: ProcessorOutputWithMeta, diagnostics_enabled: bool) -> Self {
        Self::WithMeta(output).apply_diagnostics_gate(diagnostics_enabled)
    }

    fn from_projected_min(output: ProcessorProjectedOutputMin, diagnostics_enabled: bool) -> Self {
        Self::ProjectedMin(output).apply_diagnostics_gate(diagnostics_enabled)
    }

    fn from_projected_with_meta(
        output: ProcessorProjectedOutputWithMeta,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::ProjectedWithMeta(output).apply_diagnostics_gate(diagnostics_enabled)
    }

    pub(crate) fn from_proto_row(
        value: proto::OutputRowV1,
        mode: PrimitiveOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        match mode {
            PrimitiveOutputMode::Min => Ok(Self::from_min(
                decode_proto_output_as::<ProcessorOutputMin>(
                    value,
                    "primitives outputs protobuf row",
                    false,
                )?,
                diagnostics_enabled,
            )),
            PrimitiveOutputMode::WithMeta => Ok(Self::from_with_meta(
                decode_proto_output_as::<ProcessorOutputWithMeta>(
                    value,
                    "primitives outputs full protobuf row",
                    true,
                )?,
                diagnostics_enabled,
            )),
            PrimitiveOutputMode::ProjectedMin | PrimitiveOutputMode::ProjectedWithMeta => {
                Err(SdkError::unsupported_or_unproved_usage(
                    "projected primitives protobuf/gRPC output decoding is not proved because unselected computed fields collapse to unset optional fields",
                ))
            }
        }
    }
}

impl LatestOutputsPresentRow {
    pub(crate) fn from_proto(
        value: proto::OutputsPresentRowV1,
        mode: PrimitiveOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        Ok(Self {
            output: PrimitiveOutput::from_proto_row(
                value.output.ok_or_else(|| {
                    SdkError::contract_drift("latest outputs protobuf row missing `output`")
                })?,
                mode,
                diagnostics_enabled,
            )?,
            age_ms: value.age_ms.ok_or_else(|| {
                SdkError::contract_drift("latest outputs protobuf row missing `age_ms`")
            })?,
        })
    }
}

impl LatestOutputsResponse {
    pub(crate) fn from_http_min(
        wire: LatestOutputsResponseWire<ProcessorOutputMin>,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        Self::from_http_typed(
            wire,
            PrimitiveOutputMode::Min,
            diagnostics_enabled,
            PrimitiveOutput::from_min,
        )
    }

    pub(crate) fn from_http_with_meta(
        wire: LatestOutputsResponseWire<ProcessorOutputWithMeta>,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        Self::from_http_typed(
            wire,
            PrimitiveOutputMode::WithMeta,
            diagnostics_enabled,
            PrimitiveOutput::from_with_meta,
        )
    }

    pub(crate) fn from_http_projected_min(
        wire: LatestOutputsResponseWire<ProcessorProjectedOutputMin>,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        Self::from_http_typed(
            wire,
            PrimitiveOutputMode::ProjectedMin,
            diagnostics_enabled,
            PrimitiveOutput::from_projected_min,
        )
    }

    pub(crate) fn from_http_projected_with_meta(
        wire: LatestOutputsResponseWire<ProcessorProjectedOutputWithMeta>,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        Self::from_http_typed(
            wire,
            PrimitiveOutputMode::ProjectedWithMeta,
            diagnostics_enabled,
            PrimitiveOutput::from_projected_with_meta,
        )
    }

    fn from_http_typed<T>(
        wire: LatestOutputsResponseWire<T>,
        mode: PrimitiveOutputMode,
        diagnostics_enabled: bool,
        wrap: fn(T, bool) -> PrimitiveOutput,
    ) -> Result<Self, SdkError> {
        ensure_latest_view_matches_mode(wire.view, mode, "latest outputs")?;
        Ok(Self {
            watermark_end_ms: wire.watermark_end_ms,
            close_end_ms: wire.close_end_ms,
            latest_mode: wire.latest_mode,
            view: wire.view,
            rows: wire
                .rows
                .into_iter()
                .map(|row| LatestOutputsPresentRow {
                    output: wrap(row.output, diagnostics_enabled),
                    age_ms: row.age_ms,
                })
                .collect(),
            missing_pairs: wire.missing_pairs,
        })
    }

    pub(crate) fn from_proto(
        response: proto::OutputsLatestResponseV1,
        mode: PrimitiveOutputMode,
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
                .map(|row| LatestOutputsPresentRow::from_proto(row, mode, diagnostics_enabled))
                .collect::<Result<Vec<_>, _>>()?,
            missing_pairs: response.missing_pairs,
        })
    }
}

impl RangeOutputsResponse {
    pub(crate) fn from_http_min(
        wire: RangeOutputsResponseWire<ProcessorOutputMin>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(wire, diagnostics_enabled, PrimitiveOutput::from_min)
    }

    pub(crate) fn from_http_with_meta(
        wire: RangeOutputsResponseWire<ProcessorOutputWithMeta>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(wire, diagnostics_enabled, PrimitiveOutput::from_with_meta)
    }

    pub(crate) fn from_http_projected_min(
        wire: RangeOutputsResponseWire<ProcessorProjectedOutputMin>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(
            wire,
            diagnostics_enabled,
            PrimitiveOutput::from_projected_min,
        )
    }

    pub(crate) fn from_http_projected_with_meta(
        wire: RangeOutputsResponseWire<ProcessorProjectedOutputWithMeta>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(
            wire,
            diagnostics_enabled,
            PrimitiveOutput::from_projected_with_meta,
        )
    }

    fn from_http_typed<T>(
        wire: RangeOutputsResponseWire<T>,
        diagnostics_enabled: bool,
        wrap: fn(T, bool) -> PrimitiveOutput,
    ) -> Self {
        Self {
            rows: wire
                .rows
                .into_iter()
                .map(|row| wrap(row, diagnostics_enabled))
                .collect(),
            close_end_ms: wire.close_end_ms,
            next_cursor: wire.next_cursor,
        }
    }

    pub(crate) fn from_proto(
        response: proto::OutputsRangeResponseV1,
        mode: PrimitiveOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        ensure_proto_output_mode_supported(mode, "range outputs protobuf/gRPC")?;
        Ok(Self {
            rows: response
                .rows
                .into_iter()
                .map(|row| PrimitiveOutput::from_proto_row(row, mode, diagnostics_enabled))
                .collect::<Result<Vec<_>, _>>()?,
            close_end_ms: response.close_end_ms,
            next_cursor: response.next_cursor,
        })
    }
}

impl SearchOutputsResponse {
    pub(crate) fn from_http_min(
        wire: SearchOutputsResponseWire<ProcessorOutputMin>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(wire, diagnostics_enabled, PrimitiveOutput::from_min)
    }

    pub(crate) fn from_http_with_meta(
        wire: SearchOutputsResponseWire<ProcessorOutputWithMeta>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(wire, diagnostics_enabled, PrimitiveOutput::from_with_meta)
    }

    pub(crate) fn from_http_projected_min(
        wire: SearchOutputsResponseWire<ProcessorProjectedOutputMin>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(
            wire,
            diagnostics_enabled,
            PrimitiveOutput::from_projected_min,
        )
    }

    pub(crate) fn from_http_projected_with_meta(
        wire: SearchOutputsResponseWire<ProcessorProjectedOutputWithMeta>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(
            wire,
            diagnostics_enabled,
            PrimitiveOutput::from_projected_with_meta,
        )
    }

    fn from_http_typed<T>(
        wire: SearchOutputsResponseWire<T>,
        diagnostics_enabled: bool,
        wrap: fn(T, bool) -> PrimitiveOutput,
    ) -> Self {
        Self {
            hits: wire.hits,
            evaluated_rows: wire.evaluated_rows.map(|rows| {
                rows.into_iter()
                    .map(|row| wrap(row, diagnostics_enabled))
                    .collect()
            }),
            next_cursor: wire.next_cursor,
            done: wire.done,
            returned_hits: wire.returned_hits,
            effective_hits_limit: wire.effective_hits_limit,
            truncated: wire.truncated,
            predicate_pairs: wire.predicate_pairs,
            predicate_normalized: wire.predicate_normalized,
        }
    }

    pub(crate) fn from_proto(
        response: proto::OutputsSearchResponseV1,
        mode: PrimitiveOutputMode,
        diagnostics_enabled: bool,
        evaluated_rows_enabled: bool,
    ) -> Result<Self, SdkError> {
        ensure_proto_output_mode_supported(mode, "search outputs protobuf/gRPC")?;
        let evaluated_rows = if evaluated_rows_enabled {
            Some(
                response
                    .evaluated_rows
                    .into_iter()
                    .map(|row| PrimitiveOutput::from_proto_row(row, mode, diagnostics_enabled))
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

impl TimeMachineOutputsRow {
    fn from_proto(
        value: proto::OutputsTimeMachineRowV1,
        mode: PrimitiveOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        Ok(Self {
            hit_close_ms: value.hit_close_ms,
            offset: value.offset,
            output: PrimitiveOutput::from_proto_row(
                value.output.ok_or_else(|| {
                    SdkError::contract_drift("time machine outputs protobuf row missing `output`")
                })?,
                mode,
                diagnostics_enabled,
            )?,
        })
    }
}

impl TimeMachineOutputsResponse {
    pub(crate) fn from_http_min(
        wire: TimeMachineOutputsResponseWire<ProcessorOutputMin>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(wire, diagnostics_enabled, PrimitiveOutput::from_min)
    }

    pub(crate) fn from_http_with_meta(
        wire: TimeMachineOutputsResponseWire<ProcessorOutputWithMeta>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(wire, diagnostics_enabled, PrimitiveOutput::from_with_meta)
    }

    pub(crate) fn from_http_projected_min(
        wire: TimeMachineOutputsResponseWire<ProcessorProjectedOutputMin>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(
            wire,
            diagnostics_enabled,
            PrimitiveOutput::from_projected_min,
        )
    }

    pub(crate) fn from_http_projected_with_meta(
        wire: TimeMachineOutputsResponseWire<ProcessorProjectedOutputWithMeta>,
        diagnostics_enabled: bool,
    ) -> Self {
        Self::from_http_typed(
            wire,
            diagnostics_enabled,
            PrimitiveOutput::from_projected_with_meta,
        )
    }

    fn from_http_typed<T>(
        wire: TimeMachineOutputsResponseWire<T>,
        diagnostics_enabled: bool,
        wrap: fn(T, bool) -> PrimitiveOutput,
    ) -> Self {
        Self {
            rows: wire
                .rows
                .into_iter()
                .map(|row| TimeMachineOutputsRow {
                    hit_close_ms: row.hit_close_ms,
                    offset: row.offset,
                    output: wrap(row.output, diagnostics_enabled),
                })
                .collect(),
            next_cursor: wire.next_cursor,
            done: wire.done,
            returned_hits: wire.returned_hits,
            effective_hits_limit: wire.effective_hits_limit,
            truncated: wire.truncated,
            predicate_pairs: wire.predicate_pairs,
            predicate_normalized: wire.predicate_normalized,
        }
    }

    pub(crate) fn from_proto(
        response: proto::OutputsTimeMachineResponseV1,
        mode: PrimitiveOutputMode,
        diagnostics_enabled: bool,
    ) -> Result<Self, SdkError> {
        ensure_proto_output_mode_supported(mode, "time-machine outputs protobuf/gRPC")?;
        Ok(Self {
            rows: response
                .rows
                .into_iter()
                .map(|row| TimeMachineOutputsRow::from_proto(row, mode, diagnostics_enabled))
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
    mode: PrimitiveOutputMode,
    diagnostics_enabled: bool,
) -> Result<Vec<LatestOutputsPresentRow>, SdkError> {
    match mode {
        PrimitiveOutputMode::Min => serde_json::from_str::<
            Vec<LatestOutputsPresentRowWire<ProcessorOutputMin>>,
        >(text)
        .map_err(|source| {
            SdkError::contract_drift(format!("outputs ws min JSON rows decode failed: {source}"))
        })
        .map(|rows| {
            rows.into_iter()
                .map(|row| LatestOutputsPresentRow {
                    output: PrimitiveOutput::from_min(row.output, diagnostics_enabled),
                    age_ms: row.age_ms,
                })
                .collect()
        }),
        PrimitiveOutputMode::WithMeta => serde_json::from_str::<
            Vec<LatestOutputsPresentRowWire<ProcessorOutputWithMeta>>,
        >(text)
        .map_err(|source| {
            SdkError::contract_drift(format!("outputs ws full JSON rows decode failed: {source}"))
        })
        .map(|rows| {
            rows.into_iter()
                .map(|row| LatestOutputsPresentRow {
                    output: PrimitiveOutput::from_with_meta(row.output, diagnostics_enabled),
                    age_ms: row.age_ms,
                })
                .collect()
        }),
        PrimitiveOutputMode::ProjectedMin => serde_json::from_str::<
            Vec<LatestOutputsPresentRowWire<ProcessorProjectedOutputMin>>,
        >(text)
        .map_err(|source| {
            SdkError::contract_drift(format!(
                "outputs ws projected-min JSON rows decode failed: {source}"
            ))
        })
        .map(|rows| {
            rows.into_iter()
                .map(|row| LatestOutputsPresentRow {
                    output: PrimitiveOutput::from_projected_min(row.output, diagnostics_enabled),
                    age_ms: row.age_ms,
                })
                .collect()
        }),
        PrimitiveOutputMode::ProjectedWithMeta => serde_json::from_str::<
            Vec<LatestOutputsPresentRowWire<ProcessorProjectedOutputWithMeta>>,
        >(text)
        .map_err(|source| {
            SdkError::contract_drift(format!(
                "outputs ws projected-full JSON rows decode failed: {source}"
            ))
        })
        .map(|rows| {
            rows.into_iter()
                .map(|row| LatestOutputsPresentRow {
                    output: PrimitiveOutput::from_projected_with_meta(
                        row.output,
                        diagnostics_enabled,
                    ),
                    age_ms: row.age_ms,
                })
                .collect()
        }),
    }
}

pub(crate) fn decode_latest_outputs_ws_proto(
    bytes: &[u8],
    mode: PrimitiveOutputMode,
    diagnostics_enabled: bool,
) -> Result<Vec<LatestOutputsPresentRow>, SdkError> {
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
        .map(|row| LatestOutputsPresentRow::from_proto(row, mode, diagnostics_enabled))
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

fn decode_proto_output_as<T>(
    value: proto::OutputRowV1,
    context: &'static str,
    require_metadata: bool,
) -> Result<T, SdkError>
where
    T: DeserializeOwned,
{
    let json = normalize_proto_output_json(value, context, require_metadata)?;
    serde_json::from_value(json)
        .map_err(|source| SdkError::contract_drift(format!("{context} decode failed: {source}")))
}

fn normalize_proto_output_json(
    value: proto::OutputRowV1,
    context: &'static str,
    require_metadata: bool,
) -> Result<serde_json::Value, SdkError> {
    let open_utc = value
        .open_utc
        .clone()
        .ok_or_else(|| SdkError::contract_drift(format!("{context} missing `open_utc`")))?;
    let close_utc = value
        .close_utc
        .clone()
        .ok_or_else(|| SdkError::contract_drift(format!("{context} missing `close_utc`")))?;
    let metadata = value.metadata.clone();
    if require_metadata && metadata.is_none() {
        return Err(SdkError::contract_drift(format!(
            "{context} missing `metadata`"
        )));
    }

    let mut json = serde_json::to_value(&value).map_err(|source| {
        SdkError::contract_drift(format!("{context} serialization failed: {source}"))
    })?;
    let object = json.as_object_mut().ok_or_else(|| {
        SdkError::contract_drift(format!("{context} did not serialize as a JSON object"))
    })?;
    object.insert("open_utc".to_string(), serde_json::Value::String(open_utc));
    object.insert(
        "close_utc".to_string(),
        serde_json::Value::String(close_utc),
    );

    match metadata {
        Some(metadata) => {
            object.insert(
                "metadata".to_string(),
                normalize_proto_metadata_json(metadata, context)?,
            );
        }
        None => {
            object.remove("metadata");
        }
    }

    Ok(json)
}

fn normalize_proto_metadata_json(
    value: proto::OutputMetadataV1,
    context: &'static str,
) -> Result<serde_json::Value, SdkError> {
    let mut json = serde_json::to_value(&value).map_err(|source| {
        SdkError::contract_drift(format!("{context} metadata serialization failed: {source}"))
    })?;
    let object = json.as_object_mut().ok_or_else(|| {
        SdkError::contract_drift(format!(
            "{context} metadata did not serialize as a JSON object"
        ))
    })?;

    match object.get_mut("tail_bar_provenance") {
        Some(tail) if tail.is_null() => {
            *tail = serde_json::Value::Object(serde_json::Map::new());
        }
        Some(tail) => {
            let tail_object = tail.as_object_mut().ok_or_else(|| {
                SdkError::contract_drift(format!(
                    "{context} metadata `tail_bar_provenance` did not serialize as a JSON object"
                ))
            })?;
            normalize_optional_vec_field(tail_object, "venues_expected");
            normalize_optional_vec_field(tail_object, "venues_with_trades");
        }
        None => {
            object.insert(
                "tail_bar_provenance".to_string(),
                serde_json::Value::Object(serde_json::Map::new()),
            );
        }
    }

    Ok(json)
}

fn normalize_optional_vec_field(
    object: &mut serde_json::Map<String, serde_json::Value>,
    field: &str,
) {
    let is_empty = object
        .get(field)
        .and_then(serde_json::Value::as_array)
        .is_some_and(|values| values.is_empty());
    if is_empty {
        object.insert(field.to_string(), serde_json::Value::Null);
    }
}

fn expected_output_view(mode: PrimitiveOutputMode) -> OutputView {
    if mode.has_metadata() {
        OutputView::Full
    } else {
        OutputView::Min
    }
}

fn ensure_latest_view_matches_mode(
    view: OutputView,
    mode: PrimitiveOutputMode,
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
    mode: PrimitiveOutputMode,
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

pub(crate) fn infer_output_mode(
    family: Option<&[ProcessorFamily]>,
    group: Option<&[ProcessorGroup]>,
    metadata: Option<bool>,
) -> Result<PrimitiveOutputMode, SdkError> {
    let projected = selector_values_present(family) || selector_values_present(group);
    let with_meta = metadata.unwrap_or(false);

    if selects_metadata_family(family) && !with_meta {
        return Err(SdkError::request_build(
            "family=metadata requires metadata=true",
        ));
    }

    Ok(match (projected, with_meta) {
        (false, false) => PrimitiveOutputMode::Min,
        (false, true) => PrimitiveOutputMode::WithMeta,
        (true, false) => PrimitiveOutputMode::ProjectedMin,
        (true, true) => PrimitiveOutputMode::ProjectedWithMeta,
    })
}

fn selector_values_present<T>(values: Option<&[T]>) -> bool {
    values.is_some_and(|values| !values.is_empty())
}

pub(crate) fn selects_metadata_family(values: Option<&[ProcessorFamily]>) -> bool {
    values.is_some_and(|values| values.contains(&ProcessorFamily::Metadata))
}
