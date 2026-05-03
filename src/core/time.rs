use crate::core::error::SdkError;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum TimeInput {
    Ms(i64),
    Utc(String),
}

impl TimeInput {
    pub fn to_utc_ms(&self) -> Result<i64, SdkError> {
        match self {
            Self::Ms(value) => validate_utc_ms(*value),
            Self::Utc(value) => parse_utc_string_to_ms(value),
        }
    }
}

impl From<i64> for TimeInput {
    fn from(value: i64) -> Self {
        Self::Ms(value)
    }
}

impl From<String> for TimeInput {
    fn from(value: String) -> Self {
        Self::Utc(value)
    }
}

impl From<&str> for TimeInput {
    fn from(value: &str) -> Self {
        Self::Utc(value.to_string())
    }
}

pub fn validate_utc_ms(value: i64) -> Result<i64, SdkError> {
    Utc.timestamp_millis_opt(value)
        .single()
        .map(|_| value)
        .ok_or_else(|| SdkError::invalid_time_input(format!("invalid utc ms timestamp: {value}")))
}

pub fn parse_utc_string_to_ms(value: &str) -> Result<i64, SdkError> {
    if value.ends_with('Z') {
        if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
            return Ok(parsed.with_timezone(&Utc).timestamp_millis());
        }
    }

    let time_portion = value.split_once('T').map(|(_, rest)| rest);
    if time_portion.is_some()
        && (value.contains('+') || time_portion.is_some_and(|rest| rest.contains('-')))
    {
        return Err(SdkError::invalid_time_input(format!(
            "unsupported utc time input `{value}`"
        )));
    }

    if let Ok(parsed) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d:%H:%M") {
        return Ok(Utc.from_utc_datetime(&parsed).timestamp_millis());
    }

    if let Ok(parsed) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d:%H:%M:%S") {
        return Ok(Utc.from_utc_datetime(&parsed).timestamp_millis());
    }

    Err(SdkError::invalid_time_input(format!(
        "unsupported utc time input `{value}`"
    )))
}
