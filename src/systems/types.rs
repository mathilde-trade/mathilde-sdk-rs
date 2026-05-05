use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Timeframe {
    #[serde(rename = "1m")]
    M1,
    #[serde(rename = "5m")]
    M5,
    #[serde(rename = "15m")]
    M15,
    #[serde(rename = "30m")]
    M30,
    #[serde(rename = "1h")]
    H1,
    #[serde(rename = "4h")]
    H4,
    #[serde(rename = "6h")]
    H6,
    #[serde(rename = "12h")]
    H12,
    #[serde(rename = "1d")]
    D1,
}

impl Timeframe {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::M1 => "1m",
            Self::M5 => "5m",
            Self::M15 => "15m",
            Self::M30 => "30m",
            Self::H1 => "1h",
            Self::H4 => "4h",
            Self::H6 => "6h",
            Self::H12 => "12h",
            Self::D1 => "1d",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpFormat {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "protobuf")]
    Protobuf,
}

impl HttpFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Protobuf => "protobuf",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LatestMode {
    #[serde(rename = "exact_watermark")]
    ExactWatermark,
    #[serde(rename = "latest_available_le_watermark")]
    LatestAvailableLeWatermark,
}

impl LatestMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactWatermark => "exact_watermark",
            Self::LatestAvailableLeWatermark => "latest_available_le_watermark",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlignMode {
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "floor")]
    Floor,
}

impl AlignMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Floor => "floor",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BarsView {
    #[serde(rename = "min")]
    Min,
    #[serde(rename = "full")]
    Full,
}

impl BarsView {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Min => "min",
            Self::Full => "full",
        }
    }
}
