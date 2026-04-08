use crate::core::error::SdkError;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1::{
    BarsLatestResponseV1, BarsRangeResponseV1, BarsSearchResponseV1,
};
use crate::systems::aggregator::types::{
    LatestBarsRequest, LatestBarsResponse, RangeBarsRequest, RangeBarsResponse,
    SearchBarsRequest, SearchBarsResponse,
};
use crate::systems::types::HttpFormat;
use crate::transport::http::HttpTransport;
use prost::Message;
use reqwest::Method;

pub async fn latest_bars(
    transport: &HttpTransport,
    request_body: &LatestBarsRequest,
) -> Result<LatestBarsResponse, SdkError> {
    let request = transport
        .request(Method::POST, "/v1/bars/latest")?
        .json(request_body);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;

    if matches!(request_body.format, Some(HttpFormat::Protobuf)) {
        let body = response
            .bytes()
            .await
            .map_err(|source| SdkError::Decode { source })?;
        let proto = BarsLatestResponseV1::decode(body.as_ref()).map_err(|source| {
            SdkError::contract_drift(format!("protobuf decode failed: {source}"))
        })?;
        return LatestBarsResponse::from_proto(proto);
    }

    response
        .json::<LatestBarsResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn range_bars(
    transport: &HttpTransport,
    request_body: &RangeBarsRequest,
) -> Result<RangeBarsResponse, SdkError> {
    let normalized_request = request_body.normalize()?;
    let request = transport
        .request(Method::POST, "/v1/bars/range")?
        .json(&normalized_request);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;

    if matches!(normalized_request.format, Some(HttpFormat::Protobuf)) {
        let body = response
            .bytes()
            .await
            .map_err(|source| SdkError::Decode { source })?;
        let proto = BarsRangeResponseV1::decode(body.as_ref()).map_err(|source| {
            SdkError::contract_drift(format!("protobuf decode failed: {source}"))
        })?;
        return RangeBarsResponse::from_proto(proto, normalized_request.metadata.unwrap_or(false));
    }

    if normalized_request.metadata.unwrap_or(false) {
        return response
            .json::<crate::systems::aggregator::types::RangeBarsFullResponse>()
            .await
            .map(RangeBarsResponse::Full)
            .map_err(|source| SdkError::Decode { source });
    }

    response
        .json::<crate::systems::aggregator::types::RangeBarsMinResponse>()
        .await
        .map(RangeBarsResponse::Min)
        .map_err(|source| SdkError::Decode { source })
}

pub async fn search_bars(
    transport: &HttpTransport,
    request_body: &SearchBarsRequest,
) -> Result<SearchBarsResponse, SdkError> {
    let normalized_request = request_body.normalize()?;
    let request = transport
        .request(Method::POST, "/v1/bars/search")?
        .json(&normalized_request);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;

    if matches!(normalized_request.format, Some(HttpFormat::Protobuf)) {
        let body = response
            .bytes()
            .await
            .map_err(|source| SdkError::Decode { source })?;
        let proto = BarsSearchResponseV1::decode(body.as_ref()).map_err(|source| {
            SdkError::contract_drift(format!("protobuf decode failed: {source}"))
        })?;
        return SearchBarsResponse::from_proto(
            proto,
            normalized_request.metadata.unwrap_or(false),
        );
    }

    if normalized_request.metadata.unwrap_or(false) {
        return response
            .json::<crate::systems::aggregator::types::SearchBarsFullResponse>()
            .await
            .map(SearchBarsResponse::Full)
            .map_err(|source| SdkError::Decode { source });
    }

    response
        .json::<crate::systems::aggregator::types::SearchBarsMinResponse>()
        .await
        .map(SearchBarsResponse::Min)
        .map_err(|source| SdkError::Decode { source })
}
