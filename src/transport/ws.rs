use crate::core::auth::BearerToken;
use crate::core::config::WsTransportConfig;
use crate::core::error::SdkError;
use reqwest::header::{AUTHORIZATION, HeaderMap};
use url::Url;

#[derive(Debug, Clone)]
pub struct WsTransport {
    base_url: Url,
    bearer_token: Option<BearerToken>,
}

impl WsTransport {
    pub fn new(config: &WsTransportConfig, bearer_token: Option<&BearerToken>) -> Self {
        Self {
            base_url: config.base_url.clone(),
            bearer_token: bearer_token.cloned(),
        }
    }

    pub fn endpoint_url(&self, path: &str) -> Result<Url, SdkError> {
        let mut url = self
            .base_url
            .join(path)
            .map_err(|_| SdkError::request_build(format!("invalid ws path `{path}`")))?;
        normalize_ws_scheme(&mut url)?;
        Ok(url)
    }

    pub fn upgrade_headers(&self) -> Result<HeaderMap, SdkError> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.bearer_token {
            headers.insert(AUTHORIZATION, token.as_authorization_value()?);
        }
        Ok(headers)
    }
}

fn normalize_ws_scheme(url: &mut Url) -> Result<(), SdkError> {
    match url.scheme() {
        "http" => url
            .set_scheme("ws")
            .map_err(|_| SdkError::request_build("failed to convert http base URL to ws"))?,
        "https" => url
            .set_scheme("wss")
            .map_err(|_| SdkError::request_build("failed to convert https base URL to wss"))?,
        "ws" | "wss" => {}
        other => {
            return Err(SdkError::request_build(format!(
                "unsupported ws base URL scheme `{other}`"
            )));
        }
    }
    Ok(())
}
