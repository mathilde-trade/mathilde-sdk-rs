use crate::core::error::SdkError;
use crate::systems::regime::types::{
    DocsRegistryRequest, PublicOpenApiDocument, selector_family_names, selector_group_names,
};
use crate::transport::http::HttpTransport;
use reqwest::Method;

fn csv_param(values: Vec<String>) -> Option<String> {
    if values.is_empty() {
        None
    } else {
        Some(values.join(","))
    }
}

async fn get_json(transport: &HttpTransport, path: &str) -> Result<serde_json::Value, SdkError> {
    let request = transport.request(Method::GET, path)?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<serde_json::Value>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn docs_system(transport: &HttpTransport) -> Result<serde_json::Value, SdkError> {
    get_json(transport, "/v1/docs/system").await
}

pub async fn docs_summary(transport: &HttpTransport) -> Result<serde_json::Value, SdkError> {
    get_json(transport, "/v1/docs/summary").await
}

pub async fn docs_taxonomy(transport: &HttpTransport) -> Result<serde_json::Value, SdkError> {
    get_json(transport, "/v1/docs/taxonomy").await
}

pub async fn docs_registry(
    transport: &HttpTransport,
    request_body: &DocsRegistryRequest,
) -> Result<serde_json::Value, SdkError> {
    let family = csv_param(selector_family_names(request_body.family.as_deref()));
    let group = csv_param(selector_group_names(request_body.group.as_deref()));
    let query = [("family", family), ("group", group)];

    let request = transport
        .request(Method::GET, "/v1/docs/registry")?
        .query(&query);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<serde_json::Value>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn docs_endpoints(transport: &HttpTransport) -> Result<serde_json::Value, SdkError> {
    get_json(transport, "/v1/docs/endpoints").await
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
