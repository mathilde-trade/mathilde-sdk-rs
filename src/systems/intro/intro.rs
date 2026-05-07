use crate::core::error::SdkError;
use crate::transport::http::HttpTransport;
use reqwest::Method;

async fn get_json_document(
    transport: &HttpTransport,
    path: &str,
) -> Result<serde_json::Value, SdkError> {
    let request = transport.request(Method::GET, path)?;
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<serde_json::Value>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn intro(transport: &HttpTransport) -> Result<serde_json::Value, SdkError> {
    // The public intro contract is the host root. The service may redirect to /v1/intro.
    get_json_document(transport, "/").await
}

pub async fn due_diligence(transport: &HttpTransport) -> Result<serde_json::Value, SdkError> {
    // The due-diligence index is a deploy-owned intro-host JSON surface.
    get_json_document(transport, "/v1/due-diligence").await
}

pub async fn due_diligence_regime_kalman_local_trend_state(
    transport: &HttpTransport,
) -> Result<serde_json::Value, SdkError> {
    // Approved regime review packs live on the same intro host.
    get_json_document(
        transport,
        "/v1/due-diligence/regime/kalman_local_trend_state",
    )
    .await
}

pub async fn due_diligence_regime_flow_absorption_elasticity_state(
    transport: &HttpTransport,
) -> Result<serde_json::Value, SdkError> {
    // Approved regime review packs live on the same intro host.
    get_json_document(
        transport,
        "/v1/due-diligence/regime/flow_absorption_elasticity_state",
    )
    .await
}

pub async fn due_diligence_primitives_correlation(
    transport: &HttpTransport,
) -> Result<serde_json::Value, SdkError> {
    // Approved primitives family review packs live on the same intro host.
    get_json_document(transport, "/v1/due-diligence/primitives/correlation").await
}

pub async fn due_diligence_primitives_drawdown(
    transport: &HttpTransport,
) -> Result<serde_json::Value, SdkError> {
    // Approved primitives family review packs live on the same intro host.
    get_json_document(transport, "/v1/due-diligence/primitives/drawdown").await
}
