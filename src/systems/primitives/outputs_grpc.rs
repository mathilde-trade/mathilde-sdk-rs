use crate::core::error::SdkError;
use crate::systems::primitives::types::{
    LatestOutputsGrpcRequest, LatestOutputsResponse, PrimitiveOutputMode, RangeOutputsGrpcRequest,
    RangeOutputsResponse, SearchOutputsGrpcRequest, SearchOutputsResponse,
    TimeMachineOutputsGrpcRequest, TimeMachineOutputsResponse, diagnostics_enabled,
};
use crate::transport::grpc::GrpcTransport;
use tonic::client::Grpc;
use tonic::codec::ProstCodec;
use tonic::codegen::http::uri::PathAndQuery;

const LATEST_OUTPUTS_PATH: &str = "/mathilde.feed.outputs.v1.OutputsServiceV1/LatestOutputs";
const RANGE_OUTPUTS_PATH: &str = "/mathilde.feed.outputs.v1.OutputsServiceV1/RangeOutputs";
const SEARCH_OUTPUTS_PATH: &str = "/mathilde.feed.outputs.v1.OutputsServiceV1/SearchOutputs";
const TIME_MACHINE_OUTPUTS_PATH: &str =
    "/mathilde.feed.outputs.v1.OutputsServiceV1/TimeMachineOutputs";

pub async fn latest_outputs_grpc(
    transport: &GrpcTransport,
    request: &LatestOutputsGrpcRequest,
) -> Result<LatestOutputsResponse, SdkError> {
    let output_mode = request.output_mode()?;
    ensure_grpc_mode_supported(output_mode, "latest outputs")?;
    let diagnostics_enabled = diagnostics_enabled(request.diagnostics);
    let path = PathAndQuery::from_static(LATEST_OUTPUTS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel());
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    LatestOutputsResponse::from_proto(response.into_inner(), output_mode, diagnostics_enabled)
}

pub async fn range_outputs_grpc(
    transport: &GrpcTransport,
    request: &RangeOutputsGrpcRequest,
) -> Result<RangeOutputsResponse, SdkError> {
    let output_mode = request.output_mode()?;
    ensure_grpc_mode_supported(output_mode, "range outputs")?;
    let diagnostics_enabled = diagnostics_enabled(request.diagnostics);
    let path = PathAndQuery::from_static(RANGE_OUTPUTS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel());
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    RangeOutputsResponse::from_proto(response.into_inner(), output_mode, diagnostics_enabled)
}

pub async fn search_outputs_grpc(
    transport: &GrpcTransport,
    request: &SearchOutputsGrpcRequest,
) -> Result<SearchOutputsResponse, SdkError> {
    let output_mode = request.output_mode()?;
    ensure_grpc_mode_supported(output_mode, "search outputs")?;
    let diagnostics_enabled = diagnostics_enabled(request.diagnostics);
    let evaluated_rows_enabled = request.evaluate_pair.is_some();
    let path = PathAndQuery::from_static(SEARCH_OUTPUTS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel());
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    SearchOutputsResponse::from_proto(
        response.into_inner(),
        output_mode,
        diagnostics_enabled,
        evaluated_rows_enabled,
    )
}

pub async fn time_machine_outputs_grpc(
    transport: &GrpcTransport,
    request: &TimeMachineOutputsGrpcRequest,
) -> Result<TimeMachineOutputsResponse, SdkError> {
    let output_mode = request.output_mode()?;
    ensure_grpc_mode_supported(output_mode, "time machine outputs")?;
    let diagnostics_enabled = diagnostics_enabled(request.diagnostics);
    let path = PathAndQuery::from_static(TIME_MACHINE_OUTPUTS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel());
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    TimeMachineOutputsResponse::from_proto(response.into_inner(), output_mode, diagnostics_enabled)
}

fn ensure_grpc_mode_supported(
    output_mode: PrimitiveOutputMode,
    context: &'static str,
) -> Result<(), SdkError> {
    if output_mode.is_projected() {
        return Err(SdkError::unsupported_or_unproved_usage(format!(
            "{context} projected gRPC decoding is not proved because protobuf leaves unselected computed fields unset"
        )));
    }
    Ok(())
}
