use crate::core::error::SdkError;
use crate::systems::aggregator::types::{
    PublicDocResponse, PublicDocWithIndexResponse, PublicOpenApiDocument,
};
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

pub async fn docs_themes(transport: &HttpTransport) -> Result<PublicDocWithIndexResponse, SdkError> {
    let request = transport.request(Method::GET, "/v1/docs/themes")?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<PublicDocWithIndexResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn docs_endpoints(transport: &HttpTransport) -> Result<PublicDocResponse, SdkError> {
    let request = transport.request(Method::GET, "/v1/docs/endpoints")?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<PublicDocResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn openapi(transport: &HttpTransport) -> Result<PublicOpenApiDocument, SdkError> {
    let request = transport.request(Method::GET, "/openapi-public.json")?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<PublicOpenApiDocument>()
        .await
        .map_err(|source| SdkError::Decode { source })
}
