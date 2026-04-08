use crate::core::error::SdkError;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1 as proto;
use crate::systems::types::{BarsView, ExcludeSource, HttpFormat, LatestMode, Timeframe};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PublicDocResponse {
    pub slug: String,
    pub kind: String,
    pub title: String,
    pub format: String,
    pub content: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PublicDocIndexRow {
    pub title: String,
    pub anchor: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct PublicDocWithIndexResponse {
    pub slug: String,
    pub kind: String,
    pub title: String,
    pub format: String,
    pub content: String,
    pub index: Vec<PublicDocIndexRow>,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LatestBarsRequest {
    pub pairs: String,
    pub tf: Timeframe,
    pub latest_mode: LatestMode,
    pub exclude_sources: Option<Vec<ExcludeSource>>,
    pub metadata: Option<bool>,
    pub format: Option<HttpFormat>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ExcludedSourceCount {
    pub source: String,
    pub count: i64,
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
    pub metadata: BarMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LatestBarsWithMetadataPresentRow {
    #[serde(flatten)]
    pub bar: BarWithMetadata,
    pub age_ms: i64,
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
    pub excluded_sources: Option<Vec<ExcludeSource>>,
    pub excluded_rows_total: Option<i64>,
    pub excluded_rows_by_source: Option<Vec<ExcludedSourceCount>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LatestBarsFullResponse {
    pub watermark_end_ms: i64,
    pub close_end_ms: i64,
    pub latest_mode: LatestMode,
    pub view: BarsView,
    pub rows: Vec<LatestBarsWithMetadataPresentRow>,
    pub missing_pairs: Vec<String>,
    pub excluded_sources: Option<Vec<ExcludeSource>>,
    pub excluded_rows_total: Option<i64>,
    pub excluded_rows_by_source: Option<Vec<ExcludedSourceCount>>,
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
                excluded_sources: Some(ExcludeSource::vec_from_proto(response.excluded_sources)?),
                excluded_rows_total: response.excluded_rows_total,
                excluded_rows_by_source: Some(
                    response
                        .excluded_rows_by_source
                        .into_iter()
                        .map(ExcludedSourceCount::from_proto)
                        .collect(),
                ),
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
                excluded_sources: Some(ExcludeSource::vec_from_proto(response.excluded_sources)?),
                excluded_rows_total: response.excluded_rows_total,
                excluded_rows_by_source: Some(
                    response
                        .excluded_rows_by_source
                        .into_iter()
                        .map(ExcludedSourceCount::from_proto)
                        .collect(),
                ),
            })),
            proto::BarsViewV1::Unspecified => Err(SdkError::contract_drift(
                "latest bars protobuf response has unspecified view",
            )),
        }
    }
}

impl ExcludedSourceCount {
    fn from_proto(value: proto::ExcludedSourceCountV1) -> Self {
        Self {
            source: value.source,
            count: value.count,
        }
    }
}

impl LatestBarsPresentRow {
    fn from_proto(value: proto::BarsPresentRowV1) -> Result<Self, SdkError> {
        Ok(Self {
            bar: Bar::from_proto(value.bar.ok_or_else(|| {
                SdkError::contract_drift("latest bars protobuf row missing `bar`")
            })?)?,
            age_ms: value.age_ms.unwrap_or(0),
        })
    }
}

impl LatestBarsWithMetadataPresentRow {
    fn from_proto(value: proto::BarsPresentRowV1) -> Result<Self, SdkError> {
        Ok(Self {
            bar: BarWithMetadata::from_proto(value.bar.ok_or_else(|| {
                SdkError::contract_drift("latest bars protobuf row missing `bar`")
            })?)?,
            age_ms: value.age_ms.unwrap_or(0),
        })
    }
}

impl Bar {
    fn from_proto(value: proto::BarRowV1) -> Result<Self, SdkError> {
        Ok(Self {
            pair: value.pair,
            tf: Timeframe::from_proto(&value.tf)?,
            open_ms: value.s_ms,
            close_ms: value.e_ms,
            open_utc: value.s_utc.unwrap_or_default(),
            close_utc: value.e_utc.unwrap_or_default(),
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
        })
    }
}

impl BarWithMetadata {
    fn from_proto(value: proto::BarRowV1) -> Result<Self, SdkError> {
        Ok(Self {
            pair: value.pair,
            tf: Timeframe::from_proto(&value.tf)?,
            open_ms: value.s_ms,
            close_ms: value.e_ms,
            open_utc: value.s_utc.unwrap_or_default(),
            close_utc: value.e_utc.unwrap_or_default(),
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

impl ExcludeSource {
    fn from_proto(value: String) -> Result<Self, SdkError> {
        match value.as_str() {
            "api" => Ok(Self::Api),
            "fix-data" => Ok(Self::FixData),
            "frontier" => Ok(Self::Frontier),
            "aggregate" => Ok(Self::Aggregate),
            "synthetic" => Ok(Self::Synthetic),
            "no_trade_fill" => Ok(Self::NoTradeFill),
            other => Err(SdkError::contract_drift(format!(
                "unsupported exclude_source `{other}`"
            ))),
        }
    }

    fn vec_from_proto(values: Vec<String>) -> Result<Vec<Self>, SdkError> {
        values.into_iter().map(Self::from_proto).collect()
    }
}
