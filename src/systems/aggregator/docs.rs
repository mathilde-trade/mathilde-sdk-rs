use crate::core::error::SdkError;
use crate::systems::aggregator::types::{
    PublicOpenApiDocument, PublicPageDoc, PublicThemesCompiled,
};
use crate::transport::http::HttpTransport;
use reqwest::Method;

pub async fn docs_system(transport: &HttpTransport) -> Result<PublicPageDoc, SdkError> {
    let request = transport.request(Method::GET, "/v1/docs/system")?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<PublicPageDoc>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn docs_summary(transport: &HttpTransport) -> Result<PublicPageDoc, SdkError> {
    let request = transport.request(Method::GET, "/v1/docs/summary")?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<PublicPageDoc>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn docs_themes(transport: &HttpTransport) -> Result<PublicThemesCompiled, SdkError> {
    let request = transport.request(Method::GET, "/v1/docs/themes")?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<PublicThemesCompiled>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn docs_endpoints(transport: &HttpTransport) -> Result<PublicPageDoc, SdkError> {
    let request = transport.request(Method::GET, "/v1/docs/endpoints")?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<PublicPageDoc>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn openapi(transport: &HttpTransport) -> Result<PublicOpenApiDocument, SdkError> {
    let request = transport.request(Method::GET, "/openapi.json")?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<PublicOpenApiDocument>()
        .await
        .map_err(|source| SdkError::Decode { source })
}
