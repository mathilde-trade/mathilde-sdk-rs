use crate::core::error::SdkError;
use crate::systems::aggregator::types::PublicDocResponse;
use crate::transport::http::HttpTransport;
use reqwest::Method;

pub async fn docs_system(transport: &HttpTransport) -> Result<PublicDocResponse, SdkError> {
    let request = transport.request(Method::GET, "/v1/docs/system")?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<PublicDocResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })
}
