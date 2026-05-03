use crate::core::auth::{BearerToken, apply_bearer_auth};
use crate::core::config::HttpTransportConfig;
use crate::core::error::SdkError;
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, RequestBuilder, Response};
use std::path::Path;
use tokio::fs::{File, create_dir_all};
use tokio::io::AsyncWriteExt;
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

    pub fn request_absolute(
        &self,
        method: Method,
        absolute_url: impl AsRef<str>,
    ) -> Result<RequestBuilder, SdkError> {
        let raw = absolute_url.as_ref();
        let url =
            Url::parse(raw).map_err(|source| SdkError::invalid_url(raw.to_string(), source))?;
        self.ensure_same_origin(&url)?;
        let headers = apply_bearer_auth(HeaderMap::new(), self.bearer_token.as_ref())?;
        Ok(self.client.request(method, url).headers(headers))
    }

    pub fn has_bearer_token(&self) -> bool {
        self.bearer_token.is_some()
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

    pub async fn download_to_path(
        &self,
        request: RequestBuilder,
        destination_path: &Path,
    ) -> Result<u64, SdkError> {
        if let Some(parent) = destination_path.parent() {
            create_dir_all(parent).await.map_err(SdkError::io)?;
        }

        let response = self.execute(request).await?;
        let mut response = self.ensure_success(response).await?;
        let mut file = File::create(destination_path).await.map_err(SdkError::io)?;
        let mut bytes_written: u64 = 0;

        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|source| SdkError::Decode { source })?
        {
            file.write_all(&chunk).await.map_err(SdkError::io)?;
            bytes_written += chunk.len() as u64;
        }

        file.flush().await.map_err(SdkError::io)?;
        Ok(bytes_written)
    }

    fn ensure_same_origin(&self, url: &Url) -> Result<(), SdkError> {
        if self.base_url.scheme() == url.scheme()
            && self.base_url.host_str() == url.host_str()
            && self.base_url.port_or_known_default() == url.port_or_known_default()
        {
            return Ok(());
        }

        Err(SdkError::request_build(format!(
            "absolute download URL origin `{}` does not match configured http origin `{}`",
            url.origin().ascii_serialization(),
            self.base_url.origin().ascii_serialization()
        )))
    }
}
