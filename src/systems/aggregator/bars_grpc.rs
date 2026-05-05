use crate::core::error::SdkError;
use crate::systems::aggregator::types::{
    LatestGrpcRequest, LatestResponse, RangeGrpcRequest, RangeResponse, SearchGrpcRequest,
    SearchResponse, TimeMachineGrpcRequest, TimeMachineResponse,
};
use crate::transport::grpc::GrpcTransport;
use tonic::client::Grpc;
use tonic::codec::CompressionEncoding;
use tonic::codec::ProstCodec;
use tonic::codegen::http::uri::PathAndQuery;

const LATEST_BARS_PATH: &str = "/mathilde.feed.bars.v1.BarsServiceV1/LatestBars";
const RANGE_BARS_PATH: &str = "/mathilde.feed.bars.v1.BarsServiceV1/RangeBars";
const SEARCH_BARS_PATH: &str = "/mathilde.feed.bars.v1.BarsServiceV1/SearchBars";
const TIME_MACHINE_BARS_PATH: &str = "/mathilde.feed.bars.v1.BarsServiceV1/TimeMachineBars";

pub async fn latest_bars_grpc(
    transport: &GrpcTransport,
    request: &LatestGrpcRequest,
) -> Result<LatestResponse, SdkError> {
    let path = PathAndQuery::from_static(LATEST_BARS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel())
        .accept_compressed(CompressionEncoding::Gzip)
        .accept_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Gzip);
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    LatestResponse::from_proto(response.into_inner())
}

pub async fn range_bars_grpc(
    transport: &GrpcTransport,
    request: &RangeGrpcRequest,
) -> Result<RangeResponse, SdkError> {
    let path = PathAndQuery::from_static(RANGE_BARS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel())
        .accept_compressed(CompressionEncoding::Gzip)
        .accept_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Gzip);
    let metadata = request.metadata.unwrap_or(false);
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    RangeResponse::from_proto(response.into_inner(), metadata)
}

pub async fn search_bars_grpc(
    transport: &GrpcTransport,
    request: &SearchGrpcRequest,
) -> Result<SearchResponse, SdkError> {
    let path = PathAndQuery::from_static(SEARCH_BARS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel())
        .accept_compressed(CompressionEncoding::Gzip)
        .accept_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Gzip);
    let metadata = request.metadata.unwrap_or(false);
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    SearchResponse::from_proto(response.into_inner(), metadata)
}

pub async fn time_machine_bars_grpc(
    transport: &GrpcTransport,
    request: &TimeMachineGrpcRequest,
) -> Result<TimeMachineResponse, SdkError> {
    let path = PathAndQuery::from_static(TIME_MACHINE_BARS_PATH);
    let codec = ProstCodec::default();
    let mut grpc = Grpc::new(transport.channel())
        .accept_compressed(CompressionEncoding::Gzip)
        .accept_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Zstd)
        .send_compressed(CompressionEncoding::Gzip);
    let metadata = request.metadata.unwrap_or(false);
    grpc.ready().await.map_err(SdkError::grpc_transport)?;
    let request = transport.apply_bearer(tonic::Request::new(request.to_proto()?))?;

    let response = grpc
        .unary(request, path, codec)
        .await
        .map_err(SdkError::grpc_status)?;

    TimeMachineResponse::from_proto(response.into_inner(), metadata)
}
