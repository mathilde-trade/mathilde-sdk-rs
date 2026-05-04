use crate::core::error::SdkError;
use crate::systems::regime::types::{
    PairsListRequest, PairsListResponse, PairsStatusRequest, PairsStatusResponse,
    normalize_optional_pair_values,
};
use crate::transport::http::HttpTransport;
use reqwest::Method;

fn csv_string_param(values: Option<&[String]>) -> Option<String> {
    normalize_optional_pair_values(values).map(|values| values.join(","))
}

fn csv_vec_param(values: Option<&[String]>) -> Option<String> {
    let joined = values?
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(",");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

pub async fn pairs_status(
    transport: &HttpTransport,
    request: &PairsStatusRequest,
) -> Result<PairsStatusResponse, SdkError> {
    let pairs = csv_string_param(request.pairs.as_deref());
    let filters = csv_vec_param(request.filters.as_deref());

    let query = [
        ("after_pair", request.after_pair.clone()),
        ("limit", request.limit.map(|value| value.to_string())),
        ("pairs", pairs),
        ("filters", filters),
    ];

    let request = transport
        .request(Method::GET, "/v1/pairs/status")?
        .query(&query);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<PairsStatusResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn pairs_list(
    transport: &HttpTransport,
    request: &PairsListRequest,
) -> Result<PairsListResponse, SdkError> {
    let query = [
        ("after_pair", request.after_pair.clone()),
        ("limit", request.limit.map(|value| value.to_string())),
        (
            "enabled_only",
            request.enabled_only.map(|value| value.to_string()),
        ),
    ];

    let request = transport
        .request(Method::GET, "/v1/pairs/list")?
        .query(&query);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<PairsListResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })
}
