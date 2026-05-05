use crate::core::error::SdkError;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1::{
    BarsLatestResponseV1, BarsRangeResponseV1, BarsSearchResponseV1, BarsTimeMachineResponseV1,
};
use crate::systems::aggregator::types::{
    LatestRequest, LatestResponse, RangeRequest, RangeResponse, SearchRequest, SearchResponse,
    TimeMachineRequest, TimeMachineResponse,
};
use crate::systems::types::HttpFormat;
use crate::transport::http::HttpTransport;
use prost::Message;
use reqwest::Method;

pub async fn latest_bars(
    transport: &HttpTransport,
    request_body: &LatestRequest,
) -> Result<LatestResponse, SdkError> {
    let normalized_request = request_body.normalize()?;
    let request = transport
        .request(Method::POST, "/v1/bars/latest")?
        .json(&normalized_request);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;

    if matches!(normalized_request.format, Some(HttpFormat::Protobuf)) {
        let body = response
            .bytes()
            .await
            .map_err(|source| SdkError::Decode { source })?;
        let proto = BarsLatestResponseV1::decode(body.as_ref()).map_err(|source| {
            SdkError::contract_drift(format!("protobuf decode failed: {source}"))
        })?;
        return LatestResponse::from_proto(proto);
    }

    response
        .json::<LatestResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })
}

pub async fn range_bars(
    transport: &HttpTransport,
    request_body: &RangeRequest,
) -> Result<RangeResponse, SdkError> {
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
        return RangeResponse::from_proto(proto, normalized_request.metadata.unwrap_or(false));
    }

    let response = response
        .json::<RangeResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })?;
    response.validate_metadata(normalized_request.metadata.unwrap_or(false))?;
    Ok(response)
}

pub async fn search_bars(
    transport: &HttpTransport,
    request_body: &SearchRequest,
) -> Result<SearchResponse, SdkError> {
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
        return SearchResponse::from_proto(proto, normalized_request.metadata.unwrap_or(false));
    }

    let response = response
        .json::<SearchResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })?;
    response.validate_metadata(normalized_request.metadata.unwrap_or(false))?;
    Ok(response)
}

pub async fn time_machine_bars(
    transport: &HttpTransport,
    request_body: &TimeMachineRequest,
) -> Result<TimeMachineResponse, SdkError> {
    let normalized_request = request_body.normalize()?;
    let request = transport
        .request(Method::POST, "/v1/bars/time-machine")?
        .json(&normalized_request);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;

    if matches!(normalized_request.format, Some(HttpFormat::Protobuf)) {
        let body = response
            .bytes()
            .await
            .map_err(|source| SdkError::Decode { source })?;
        let proto = BarsTimeMachineResponseV1::decode(body.as_ref()).map_err(|source| {
            SdkError::contract_drift(format!("protobuf decode failed: {source}"))
        })?;
        return TimeMachineResponse::from_proto(
            proto,
            normalized_request.metadata.unwrap_or(false),
        );
    }

    let response = response
        .json::<TimeMachineResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })?;
    response.validate_metadata(normalized_request.metadata.unwrap_or(false))?;
    Ok(response)
}
