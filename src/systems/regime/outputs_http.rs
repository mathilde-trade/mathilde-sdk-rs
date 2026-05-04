use crate::core::error::SdkError;
use crate::generated::regime::{
    ProcessorOutputMin, ProcessorOutputWithMeta, ProcessorProjectedOutputMin,
    ProcessorProjectedOutputWithMeta, outputs_proto::mathilde::feed::outputs::v1 as proto,
};
use crate::systems::regime::types::{
    LatestOutputsRequest, LatestOutputsResponse, LatestOutputsResponseWire, RangeOutputsRequest,
    RangeOutputsResponse, RangeOutputsResponseWire, RegimeOutputMode, SearchOutputsRequest,
    SearchOutputsResponse, SearchOutputsResponseWire, TimeMachineOutputsRequest,
    TimeMachineOutputsResponse, TimeMachineOutputsResponseWire, diagnostics_enabled,
};
use crate::systems::types::HttpFormat;
use crate::transport::http::HttpTransport;
use prost::Message;
use reqwest::Method;

pub async fn latest_outputs(
    transport: &HttpTransport,
    request_body: &LatestOutputsRequest,
) -> Result<LatestOutputsResponse, SdkError> {
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
        return LatestOutputsResponse::from_proto(proto, output_mode, diagnostics_enabled);
    }

    match output_mode {
        RegimeOutputMode::Min => response
            .json::<LatestOutputsResponseWire<ProcessorOutputMin>>()
            .await
            .map_err(|source| SdkError::Decode { source })
            .and_then(|wire| LatestOutputsResponse::from_http_min(wire, diagnostics_enabled)),
        RegimeOutputMode::WithMeta => response
            .json::<LatestOutputsResponseWire<ProcessorOutputWithMeta>>()
            .await
            .map_err(|source| SdkError::Decode { source })
            .and_then(|wire| LatestOutputsResponse::from_http_with_meta(wire, diagnostics_enabled)),
        RegimeOutputMode::ProjectedMin => response
            .json::<LatestOutputsResponseWire<ProcessorProjectedOutputMin>>()
            .await
            .map_err(|source| SdkError::Decode { source })
            .and_then(|wire| {
                LatestOutputsResponse::from_http_projected_min(wire, diagnostics_enabled)
            }),
        RegimeOutputMode::ProjectedWithMeta => response
            .json::<LatestOutputsResponseWire<ProcessorProjectedOutputWithMeta>>()
            .await
            .map_err(|source| SdkError::Decode { source })
            .and_then(|wire| {
                LatestOutputsResponse::from_http_projected_with_meta(wire, diagnostics_enabled)
            }),
    }
}

pub async fn range_outputs(
    transport: &HttpTransport,
    request_body: &RangeOutputsRequest,
) -> Result<RangeOutputsResponse, SdkError> {
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
        return RangeOutputsResponse::from_proto(proto, output_mode, diagnostics_enabled);
    }

    Ok(match output_mode {
        RegimeOutputMode::Min => RangeOutputsResponse::from_http_min(
            response
                .json::<RangeOutputsResponseWire<ProcessorOutputMin>>()
                .await
                .map_err(|source| SdkError::Decode { source })?,
            diagnostics_enabled,
        ),
        RegimeOutputMode::WithMeta => RangeOutputsResponse::from_http_with_meta(
            response
                .json::<RangeOutputsResponseWire<ProcessorOutputWithMeta>>()
                .await
                .map_err(|source| SdkError::Decode { source })?,
            diagnostics_enabled,
        ),
        RegimeOutputMode::ProjectedMin => RangeOutputsResponse::from_http_projected_min(
            response
                .json::<RangeOutputsResponseWire<ProcessorProjectedOutputMin>>()
                .await
                .map_err(|source| SdkError::Decode { source })?,
            diagnostics_enabled,
        ),
        RegimeOutputMode::ProjectedWithMeta => RangeOutputsResponse::from_http_projected_with_meta(
            response
                .json::<RangeOutputsResponseWire<ProcessorProjectedOutputWithMeta>>()
                .await
                .map_err(|source| SdkError::Decode { source })?,
            diagnostics_enabled,
        ),
    })
}

pub async fn search_outputs(
    transport: &HttpTransport,
    request_body: &SearchOutputsRequest,
) -> Result<SearchOutputsResponse, SdkError> {
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
        return SearchOutputsResponse::from_proto(
            proto,
            output_mode,
            diagnostics_enabled,
            request_body.evaluate_pair.is_some(),
        );
    }

    Ok(match output_mode {
        RegimeOutputMode::Min => SearchOutputsResponse::from_http_min(
            response
                .json::<SearchOutputsResponseWire<ProcessorOutputMin>>()
                .await
                .map_err(|source| SdkError::Decode { source })?,
            diagnostics_enabled,
        ),
        RegimeOutputMode::WithMeta => SearchOutputsResponse::from_http_with_meta(
            response
                .json::<SearchOutputsResponseWire<ProcessorOutputWithMeta>>()
                .await
                .map_err(|source| SdkError::Decode { source })?,
            diagnostics_enabled,
        ),
        RegimeOutputMode::ProjectedMin => SearchOutputsResponse::from_http_projected_min(
            response
                .json::<SearchOutputsResponseWire<ProcessorProjectedOutputMin>>()
                .await
                .map_err(|source| SdkError::Decode { source })?,
            diagnostics_enabled,
        ),
        RegimeOutputMode::ProjectedWithMeta => {
            SearchOutputsResponse::from_http_projected_with_meta(
                response
                    .json::<SearchOutputsResponseWire<ProcessorProjectedOutputWithMeta>>()
                    .await
                    .map_err(|source| SdkError::Decode { source })?,
                diagnostics_enabled,
            )
        }
    })
}

pub async fn time_machine_outputs(
    transport: &HttpTransport,
    request_body: &TimeMachineOutputsRequest,
) -> Result<TimeMachineOutputsResponse, SdkError> {
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
        return TimeMachineOutputsResponse::from_proto(proto, output_mode, diagnostics_enabled);
    }

    Ok(match output_mode {
        RegimeOutputMode::Min => TimeMachineOutputsResponse::from_http_min(
            response
                .json::<TimeMachineOutputsResponseWire<ProcessorOutputMin>>()
                .await
                .map_err(|source| SdkError::Decode { source })?,
            diagnostics_enabled,
        ),
        RegimeOutputMode::WithMeta => TimeMachineOutputsResponse::from_http_with_meta(
            response
                .json::<TimeMachineOutputsResponseWire<ProcessorOutputWithMeta>>()
                .await
                .map_err(|source| SdkError::Decode { source })?,
            diagnostics_enabled,
        ),
        RegimeOutputMode::ProjectedMin => TimeMachineOutputsResponse::from_http_projected_min(
            response
                .json::<TimeMachineOutputsResponseWire<ProcessorProjectedOutputMin>>()
                .await
                .map_err(|source| SdkError::Decode { source })?,
            diagnostics_enabled,
        ),
        RegimeOutputMode::ProjectedWithMeta => {
            TimeMachineOutputsResponse::from_http_projected_with_meta(
                response
                    .json::<TimeMachineOutputsResponseWire<ProcessorProjectedOutputWithMeta>>()
                    .await
                    .map_err(|source| SdkError::Decode { source })?,
                diagnostics_enabled,
            )
        }
    })
}

fn ensure_http_format_supported(
    output_mode: RegimeOutputMode,
    format: Option<HttpFormat>,
    context: &'static str,
) -> Result<(), SdkError> {
    if output_mode.is_projected() && matches!(format, Some(HttpFormat::Protobuf)) {
        return Err(SdkError::unsupported_or_unproved_usage(format!(
            "{context} projected HTTP protobuf requests are rejected by regime v1"
        )));
    }
    Ok(())
}
