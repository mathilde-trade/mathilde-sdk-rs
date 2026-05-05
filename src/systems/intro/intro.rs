use crate::core::error::SdkError;
use crate::transport::http::HttpTransport;
use reqwest::Method;

pub async fn intro(transport: &HttpTransport) -> Result<serde_json::Value, SdkError> {
    // The public intro contract is the host root. The service may redirect to /v1/intro.
    let request = transport.request(Method::GET, "/")?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<serde_json::Value>()
        .await
        .map_err(|source| SdkError::Decode { source })
}
