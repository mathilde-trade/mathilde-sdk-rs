use crate::core::error::SdkError;
use crate::generated::primitives::outputs_proto::mathilde::feed::outputs::v1 as proto;
use crate::systems::primitives::types::{
    LatestOutputsHttpResponseWire, LatestRequest, LatestResponse, PrimitiveOutputMode,
    RangeOutputsResponseWire, RangeRequest, RangeResponse, SearchOutputsResponseWire,
    SearchRequest, SearchResponse, TimeMachineOutputsResponseWire, TimeMachineRequest,
    TimeMachineResponse, diagnostics_enabled,
};
use crate::systems::types::HttpFormat;
use crate::transport::http::HttpTransport;
use prost::Message;
use reqwest::Method;

pub async fn latest_outputs(
    transport: &HttpTransport,
    request_body: &LatestRequest,
) -> Result<LatestResponse, SdkError> {
    let output_mode = request_body.output_mode()?;
    let normalized_request = request_body.normalize_http()?;
    ensure_http_format_supported(output_mode, normalized_request.format, "latest outputs")?;
    let diagnostics_enabled = diagnostics_enabled(request_body.diagnostics);
    let request = transport
        .request(Method::POST, "/v1/outputs/latest")?
        .json(&normalized_request);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;

    if matches!(normalized_request.format, Some(HttpFormat::Protobuf)) {
        let body = response
            .bytes()
            .await
            .map_err(|source| SdkError::Decode { source })?;
        let proto = proto::OutputsLatestResponseV1::decode(body.as_ref()).map_err(|source| {
            SdkError::contract_drift(format!("latest outputs protobuf decode failed: {source}"))
        })?;
        return LatestResponse::from_proto(proto, output_mode, diagnostics_enabled);
    }

    response
        .json::<LatestOutputsHttpResponseWire>()
        .await
        .map_err(|source| SdkError::Decode { source })
        .and_then(|wire| LatestResponse::from_http(wire, output_mode, diagnostics_enabled))
}

pub async fn range_outputs(
    transport: &HttpTransport,
    request_body: &RangeRequest,
) -> Result<RangeResponse, SdkError> {
    let output_mode = request_body.output_mode()?;
    let normalized_request = request_body.normalize_http()?;
    ensure_http_format_supported(output_mode, normalized_request.format, "range outputs")?;
    let diagnostics_enabled = diagnostics_enabled(request_body.diagnostics);
    let request = transport
        .request(Method::POST, "/v1/outputs/range")?
        .json(&normalized_request);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;

    if matches!(normalized_request.format, Some(HttpFormat::Protobuf)) {
        let body = response
            .bytes()
            .await
            .map_err(|source| SdkError::Decode { source })?;
        let proto = proto::OutputsRangeResponseV1::decode(body.as_ref()).map_err(|source| {
            SdkError::contract_drift(format!("range outputs protobuf decode failed: {source}"))
        })?;
        return RangeResponse::from_proto(proto, output_mode, diagnostics_enabled);
    }

    response
        .json::<RangeOutputsResponseWire>()
        .await
        .map_err(|source| SdkError::Decode { source })
        .and_then(|wire| RangeResponse::from_http(wire, output_mode, diagnostics_enabled))
}

pub async fn search_outputs(
    transport: &HttpTransport,
    request_body: &SearchRequest,
) -> Result<SearchResponse, SdkError> {
    let output_mode = request_body.output_mode()?;
    let normalized_request = request_body.normalize_http()?;
    ensure_http_format_supported(output_mode, normalized_request.format, "search outputs")?;
    let diagnostics_enabled = diagnostics_enabled(request_body.diagnostics);
    let request = transport
        .request(Method::POST, "/v1/outputs/search")?
        .json(&normalized_request);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;

    if matches!(normalized_request.format, Some(HttpFormat::Protobuf)) {
        let body = response
            .bytes()
            .await
            .map_err(|source| SdkError::Decode { source })?;
        let proto = proto::OutputsSearchResponseV1::decode(body.as_ref()).map_err(|source| {
            SdkError::contract_drift(format!("search outputs protobuf decode failed: {source}"))
        })?;
        return SearchResponse::from_proto(
            proto,
            output_mode,
            diagnostics_enabled,
            request_body.evaluate_pair.is_some(),
        );
    }

    response
        .json::<SearchOutputsResponseWire>()
        .await
        .map_err(|source| SdkError::Decode { source })
        .and_then(|wire| SearchResponse::from_http(wire, output_mode, diagnostics_enabled))
}

pub async fn time_machine_outputs(
    transport: &HttpTransport,
    request_body: &TimeMachineRequest,
) -> Result<TimeMachineResponse, SdkError> {
    let output_mode = request_body.output_mode()?;
    let normalized_request = request_body.normalize_http()?;
    ensure_http_format_supported(
        output_mode,
        normalized_request.format,
        "time machine outputs",
    )?;
    let diagnostics_enabled = diagnostics_enabled(request_body.diagnostics);
    let request = transport
        .request(Method::POST, "/v1/outputs/time-machine")?
        .json(&normalized_request);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;

    if matches!(normalized_request.format, Some(HttpFormat::Protobuf)) {
        let body = response
            .bytes()
            .await
            .map_err(|source| SdkError::Decode { source })?;
        let proto =
            proto::OutputsTimeMachineResponseV1::decode(body.as_ref()).map_err(|source| {
                SdkError::contract_drift(format!(
                    "time machine outputs protobuf decode failed: {source}"
                ))
            })?;
        return TimeMachineResponse::from_proto(proto, output_mode, diagnostics_enabled);
    }

    response
        .json::<TimeMachineOutputsResponseWire>()
        .await
        .map_err(|source| SdkError::Decode { source })
        .and_then(|wire| TimeMachineResponse::from_http(wire, output_mode, diagnostics_enabled))
}

fn ensure_http_format_supported(
    output_mode: PrimitiveOutputMode,
    format: Option<HttpFormat>,
    context: &'static str,
) -> Result<(), SdkError> {
    if output_mode.is_projected() && matches!(format, Some(HttpFormat::Protobuf)) {
        return Err(SdkError::unsupported_or_unproved_usage(format!(
            "{context} selector-present HTTP protobuf requests are rejected by primitives v1"
        )));
    }
    Ok(())
}
