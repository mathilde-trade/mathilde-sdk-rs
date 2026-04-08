use crate::core::error::SdkError;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BearerToken(String);

impl BearerToken {
    pub fn new(raw: impl Into<String>) -> Result<Self, SdkError> {
        let raw = raw.into();
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(SdkError::invalid_auth_token(
                "bearer token must not be empty",
            ));
        }
        if trimmed.chars().any(char::is_whitespace) {
            return Err(SdkError::invalid_auth_token(
                "bearer token must not contain whitespace",
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_authorization_value(&self) -> Result<HeaderValue, SdkError> {
        HeaderValue::from_str(&format!("Bearer {}", self.0)).map_err(|_| {
            SdkError::invalid_auth_token("bearer token cannot be encoded as an authorization header")
        })
    }
}

pub fn apply_bearer_auth(
    mut headers: HeaderMap,
    token: Option<&BearerToken>,
) -> Result<HeaderMap, SdkError> {
    if let Some(token) = token {
        headers.insert(AUTHORIZATION, token.as_authorization_value()?);
    }
    Ok(headers)
}
