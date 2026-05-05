use crate::core::auth::BearerToken;
use crate::core::config::AggregatorConfig;
use crate::core::error::SdkError;
use crate::streaming::make_before_break::MakeBeforeBreakConfig;
use crate::streaming::subscription::ExponentialBackoffConfig;
use crate::systems::aggregator::bars_grpc;
use crate::systems::aggregator::bars_http;
use crate::systems::aggregator::bars_pagination::{
    RangeCall, RangeGrpcCall, SearchCall, SearchGrpcCall, TimeMachineCall, TimeMachineGrpcCall,
};
use crate::systems::aggregator::bars_ws;
use crate::systems::aggregator::docs;
use crate::systems::aggregator::files;
use crate::systems::aggregator::messages_ws;
use crate::systems::aggregator::pairs;
use crate::systems::aggregator::types::{
    DownloadedFile, FilesDownloadsRequest, FilesDownloadsResponse, FilesDownloadsRow,
    LatestGrpcRequest, LatestRequest, LatestResponse, PairsListRequest, PairsListResponse,
    PairsStatusRequest, PairsStatusResponse, PublicOpenApiDocument, RangeGrpcRequest, RangeRequest,
    RangeResponse, SearchGrpcRequest, SearchRequest, SearchResponse, TimeMachineGrpcRequest,
    TimeMachineRequest, TimeMachineResponse,
};
use crate::systems::aggregator::{
    BarsWsConnection, BarsWsMakeBeforeBreak, BarsWsSubscribeRequest, MessagesWsConnection,
    RecoveringBarsWsConnection, RecoveringMessagesWsConnection,
};
use crate::transport::grpc::GrpcTransport;
use crate::transport::http::HttpTransport;
use crate::transport::ws::WsTransport;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Aggregator {
    http: HttpTransport,
    grpc: Option<GrpcTransport>,
    ws: Option<WsTransport>,
}

impl Aggregator {
    pub fn new(config: AggregatorConfig) -> Result<Self, SdkError> {
        let http = config.require_http().clone();
        let grpc = config
            .grpc
            .as_ref()
            .map(|grpc| GrpcTransport::new(grpc, config.bearer_token.clone()))
            .transpose()?;
        let ws = config
            .ws
            .as_ref()
            .map(|ws| WsTransport::new(ws, config.bearer_token.as_ref()));

        Ok(Self {
            http: HttpTransport::new(&http, config.bearer_token.clone()),
            grpc,
            ws,
        })
    }

    pub fn client(bearer_token: Option<BearerToken>) -> Result<Self, SdkError> {
        Self::new(AggregatorConfig::mathilde_public_default(bearer_token)?)
    }

    pub async fn docs_system(&self) -> Result<serde_json::Value, SdkError> {
        docs::docs_system(&self.http).await
    }

    pub async fn docs_summary(&self) -> Result<serde_json::Value, SdkError> {
        docs::docs_summary(&self.http).await
    }

    pub async fn docs_themes(&self) -> Result<serde_json::Value, SdkError> {
        docs::docs_themes(&self.http).await
    }

    pub async fn docs_endpoints(&self) -> Result<serde_json::Value, SdkError> {
        docs::docs_endpoints(&self.http).await
    }

    pub async fn openapi(&self) -> Result<PublicOpenApiDocument, SdkError> {
        docs::openapi(&self.http).await
    }

    pub async fn latest(&self, request: &LatestRequest) -> Result<LatestResponse, SdkError> {
        bars_http::latest_bars(&self.http, request).await
    }

    pub async fn latest_grpc(
        &self,
        request: &LatestGrpcRequest,
    ) -> Result<LatestResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        bars_grpc::latest_bars_grpc(grpc, request).await
    }

    pub async fn range(&self, request: &RangeRequest) -> Result<RangeResponse, SdkError> {
        bars_http::range_bars(&self.http, request).await
    }

    pub async fn range_grpc(&self, request: &RangeGrpcRequest) -> Result<RangeResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        bars_grpc::range_bars_grpc(grpc, request).await
    }

    pub fn range_call(&self, request: RangeRequest) -> RangeCall<'_> {
        RangeCall::new(self, request)
    }

    pub fn range_grpc_call(&self, request: RangeGrpcRequest) -> RangeGrpcCall<'_> {
        RangeGrpcCall::new(self, request)
    }

    pub async fn search(&self, request: &SearchRequest) -> Result<SearchResponse, SdkError> {
        bars_http::search_bars(&self.http, request).await
    }

    pub async fn search_grpc(
        &self,
        request: &SearchGrpcRequest,
    ) -> Result<SearchResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        bars_grpc::search_bars_grpc(grpc, request).await
    }

    pub fn search_call(&self, request: SearchRequest) -> SearchCall<'_> {
        SearchCall::new(self, request)
    }

    pub fn search_grpc_call(&self, request: SearchGrpcRequest) -> SearchGrpcCall<'_> {
        SearchGrpcCall::new(self, request)
    }

    pub async fn time_machine(
        &self,
        request: &TimeMachineRequest,
    ) -> Result<TimeMachineResponse, SdkError> {
        bars_http::time_machine_bars(&self.http, request).await
    }

    pub async fn time_machine_grpc(
        &self,
        request: &TimeMachineGrpcRequest,
    ) -> Result<TimeMachineResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        bars_grpc::time_machine_bars_grpc(grpc, request).await
    }

    pub fn time_machine_call(&self, request: TimeMachineRequest) -> TimeMachineCall<'_> {
        TimeMachineCall::new(self, request)
    }

    pub fn time_machine_grpc_call(
        &self,
        request: TimeMachineGrpcRequest,
    ) -> TimeMachineGrpcCall<'_> {
        TimeMachineGrpcCall::new(self, request)
    }

    pub async fn connect_bars_ws(
        &self,
        request: &BarsWsSubscribeRequest,
    ) -> Result<BarsWsConnection, SdkError> {
        let ws = self
            .ws
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("ws"))?;
        bars_ws::BarsWsConnection::connect(ws, request).await
    }

    pub async fn connect_bars_ws_make_before_break(
        &self,
        request: &BarsWsSubscribeRequest,
    ) -> Result<BarsWsMakeBeforeBreak, SdkError> {
        let ws = self
            .ws
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("ws"))?;
        bars_ws::BarsWsMakeBeforeBreak::connect(ws, request, MakeBeforeBreakConfig::default()).await
    }

    pub async fn connect_bars_ws_recovering(
        &self,
        request: &BarsWsSubscribeRequest,
        config: ExponentialBackoffConfig,
    ) -> Result<RecoveringBarsWsConnection, SdkError> {
        let ws = self
            .ws
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("ws"))?;
        bars_ws::RecoveringBarsWsConnection::connect(ws, request, config).await
    }

    pub async fn connect_messages_ws(&self) -> Result<MessagesWsConnection, SdkError> {
        let ws = self
            .ws
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("ws"))?;
        messages_ws::MessagesWsConnection::connect(ws).await
    }

    pub async fn connect_messages_ws_recovering(
        &self,
        config: ExponentialBackoffConfig,
    ) -> Result<RecoveringMessagesWsConnection, SdkError> {
        let ws = self
            .ws
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("ws"))?;
        messages_ws::RecoveringMessagesWsConnection::connect(ws, config).await
    }

    pub async fn pairs_status(
        &self,
        request: &PairsStatusRequest,
    ) -> Result<PairsStatusResponse, SdkError> {
        pairs::pairs_status(&self.http, request).await
    }

    pub async fn pairs_list(
        &self,
        request: &PairsListRequest,
    ) -> Result<PairsListResponse, SdkError> {
        pairs::pairs_list(&self.http, request).await
    }

    pub async fn files_downloads(
        &self,
        request: &FilesDownloadsRequest,
    ) -> Result<FilesDownloadsResponse, SdkError> {
        files::files_downloads(&self.http, request).await
    }

    pub async fn files_download_items(
        &self,
        items: &[FilesDownloadsRow],
        destination_root: Option<&Path>,
    ) -> Result<Vec<DownloadedFile>, SdkError> {
        files::files_download_items(&self.http, items, destination_root).await
    }
}
