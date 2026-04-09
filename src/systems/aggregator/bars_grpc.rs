use crate::core::error::SdkError;
use crate::systems::aggregator::types::{
    LatestBarsGrpcRequest, LatestBarsResponse, RangeBarsGrpcRequest, RangeBarsResponse,
    SearchBarsGrpcRequest, SearchBarsResponse, TimeMachineBarsGrpcRequest,
    TimeMachineBarsResponse,
};
use crate::transport::grpc::GrpcTransport;
use tonic::client::Grpc;
use tonic::codec::ProstCodec;
use tonic::codegen::http::uri::PathAndQuery;

const LATEST_BARS_PATH: &str = "/mathilde.feed.bars.v1.BarsServiceV1/LatestBars";
const RANGE_BARS_PATH: &str = "/mathilde.feed.bars.v1.BarsServiceV1/RangeBars";
const SEARCH_BARS_PATH: &str = "/mathilde.feed.bars.v1.BarsServiceV1/SearchBars";
const TIME_MACHINE_BARS_PATH: &str = "/mathilde.feed.bars.v1.BarsServiceV1/TimeMachineBars";

pub async fn latest_bars_grpc(
    transport: &GrpcTransport,
    request: &LatestBarsGrpcRequest,
) -> Result<LatestBarsResponse, SdkError> {
    let path = PathAndQuery::from_static(LATEST_BARS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel());
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    LatestBarsResponse::from_proto(response.into_inner())
}

pub async fn range_bars_grpc(
    transport: &GrpcTransport,
    request: &RangeBarsGrpcRequest,
) -> Result<RangeBarsResponse, SdkError> {
    let path = PathAndQuery::from_static(RANGE_BARS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel());
    let metadata = request.metadata.unwrap_or(false);
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    RangeBarsResponse::from_proto(response.into_inner(), metadata)
}

pub async fn search_bars_grpc(
    transport: &GrpcTransport,
    request: &SearchBarsGrpcRequest,
) -> Result<SearchBarsResponse, SdkError> {
    let path = PathAndQuery::from_static(SEARCH_BARS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel());
    let metadata = request.metadata.unwrap_or(false);
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    SearchBarsResponse::from_proto(response.into_inner(), metadata)
}

pub async fn time_machine_bars_grpc(
    transport: &GrpcTransport,
    request: &TimeMachineBarsGrpcRequest,
) -> Result<TimeMachineBarsResponse, SdkError> {
    let path = PathAndQuery::from_static(TIME_MACHINE_BARS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel());
    let metadata = request.metadata.unwrap_or(false);
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    TimeMachineBarsResponse::from_proto(response.into_inner(), metadata)
}
