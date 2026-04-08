use crate::core::auth::{apply_bearer_auth, BearerToken};
use crate::core::config::HttpTransportConfig;
use crate::core::error::SdkError;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, RequestBuilder, Response};
use url::Url;

#[derive(Debug, Clone)]
pub struct HttpTransport {
    client: Client,
    base_url: Url,
    bearer_token: Option<BearerToken>,
}

impl HttpTransport {
    pub fn new(config: &HttpTransportConfig, bearer_token: Option<BearerToken>) -> Self {
        Self {
            client: Client::new(),
            base_url: config.base_url.clone(),
            bearer_token,
        }
    }

    pub fn endpoint_url(&self, path: &str) -> Result<Url, SdkError> {
        let normalized = path.trim_start_matches('/');
        self.base_url
            .join(normalized)
            .map_err(|source| SdkError::invalid_url(path.to_string(), source))
    }

    pub fn request(&self, method: Method, path: &str) -> Result<RequestBuilder, SdkError> {
        let url = self.endpoint_url(path)?;
        let headers = apply_bearer_auth(HeaderMap::new(), self.bearer_token.as_ref())?;
        Ok(self.client.request(method, url).headers(headers))
    }

    pub async fn execute(&self, request: RequestBuilder) -> Result<Response, SdkError> {
        request
            .send()
            .await
            .map_err(|source| SdkError::Transport { source })
    }

    pub async fn ensure_success(&self, response: Response) -> Result<Response, SdkError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let body = response
            .text()
            .await
            .map_err(|source| SdkError::Decode { source })?;
        Err(SdkError::HttpStatus {
            status: status.as_u16(),
            body,
        })
    }
}
