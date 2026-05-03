use crate::core::auth::BearerToken;
use crate::core::config::AggregatorConfig;
use crate::core::error::SdkError;
use crate::streaming::make_before_break::MakeBeforeBreakConfig;
use crate::streaming::subscription::ExponentialBackoffConfig;
use crate::systems::aggregator::bars_grpc;
use crate::systems::aggregator::bars_http;
use crate::systems::aggregator::bars_pagination::{
    RangeBarsCall, RangeBarsGrpcCall, SearchBarsCall, SearchBarsGrpcCall, TimeMachineBarsCall,
    TimeMachineBarsGrpcCall,
};
use crate::systems::aggregator::bars_ws;
use crate::systems::aggregator::docs;
use crate::systems::aggregator::files;
use crate::systems::aggregator::messages_ws;
use crate::systems::aggregator::pairs;
use crate::systems::aggregator::types::{
    DownloadedFile, FilesDownloadsRequest, FilesDownloadsResponse, FilesDownloadsRow,
    LatestBarsGrpcRequest, LatestBarsRequest, LatestBarsResponse, PairsListRequest,
    PairsListResponse, PairsStatusRequest, PairsStatusResponse, PublicOpenApiDocument,
    PublicPageDoc, PublicThemesCompiled, RangeBarsGrpcRequest, RangeBarsRequest, RangeBarsResponse,
    SearchBarsGrpcRequest, SearchBarsRequest, SearchBarsResponse, TimeMachineBarsGrpcRequest,
    TimeMachineBarsRequest, TimeMachineBarsResponse,
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
pub struct AggregatorClient {
    http: HttpTransport,
    grpc: Option<GrpcTransport>,
    ws: Option<WsTransport>,
}

impl AggregatorClient {
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

    pub fn mathilde_public_default(bearer_token: Option<BearerToken>) -> Result<Self, SdkError> {
        Self::new(AggregatorConfig::mathilde_public_default(bearer_token)?)
    }

    pub async fn docs_system(&self) -> Result<PublicPageDoc, SdkError> {
        docs::docs_system(&self.http).await
    }

    pub async fn docs_summary(&self) -> Result<PublicPageDoc, SdkError> {
        docs::docs_summary(&self.http).await
    }

    pub async fn docs_themes(&self) -> Result<PublicThemesCompiled, SdkError> {
        docs::docs_themes(&self.http).await
    }

    pub async fn docs_endpoints(&self) -> Result<PublicPageDoc, SdkError> {
        docs::docs_endpoints(&self.http).await
    }

    pub async fn openapi(&self) -> Result<PublicOpenApiDocument, SdkError> {
        docs::openapi(&self.http).await
    }

    pub async fn latest_bars(
        &self,
        request: &LatestBarsRequest,
    ) -> Result<LatestBarsResponse, SdkError> {
        bars_http::latest_bars(&self.http, request).await
    }

    pub async fn latest_bars_grpc(
        &self,
        request: &LatestBarsGrpcRequest,
    ) -> Result<LatestBarsResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        bars_grpc::latest_bars_grpc(grpc, request).await
    }

    pub async fn range_bars(
        &self,
        request: &RangeBarsRequest,
    ) -> Result<RangeBarsResponse, SdkError> {
        bars_http::range_bars(&self.http, request).await
    }

    pub async fn range_bars_grpc(
        &self,
        request: &RangeBarsGrpcRequest,
    ) -> Result<RangeBarsResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        bars_grpc::range_bars_grpc(grpc, request).await
    }

    pub fn range_bars_call(&self, request: RangeBarsRequest) -> RangeBarsCall<'_> {
        RangeBarsCall::new(self, request)
    }

    pub fn range_bars_grpc_call(&self, request: RangeBarsGrpcRequest) -> RangeBarsGrpcCall<'_> {
        RangeBarsGrpcCall::new(self, request)
    }

    pub async fn search_bars(
        &self,
        request: &SearchBarsRequest,
    ) -> Result<SearchBarsResponse, SdkError> {
        bars_http::search_bars(&self.http, request).await
    }

    pub async fn search_bars_grpc(
        &self,
        request: &SearchBarsGrpcRequest,
    ) -> Result<SearchBarsResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        bars_grpc::search_bars_grpc(grpc, request).await
    }

    pub fn search_bars_call(&self, request: SearchBarsRequest) -> SearchBarsCall<'_> {
        SearchBarsCall::new(self, request)
    }

    pub fn search_bars_grpc_call(&self, request: SearchBarsGrpcRequest) -> SearchBarsGrpcCall<'_> {
        SearchBarsGrpcCall::new(self, request)
    }

    pub async fn time_machine_bars(
        &self,
        request: &TimeMachineBarsRequest,
    ) -> Result<TimeMachineBarsResponse, SdkError> {
        bars_http::time_machine_bars(&self.http, request).await
    }

    pub async fn time_machine_bars_grpc(
        &self,
        request: &TimeMachineBarsGrpcRequest,
    ) -> Result<TimeMachineBarsResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        bars_grpc::time_machine_bars_grpc(grpc, request).await
    }

    pub fn time_machine_bars_call(
        &self,
        request: TimeMachineBarsRequest,
    ) -> TimeMachineBarsCall<'_> {
        TimeMachineBarsCall::new(self, request)
    }

    pub fn time_machine_bars_grpc_call(
        &self,
        request: TimeMachineBarsGrpcRequest,
    ) -> TimeMachineBarsGrpcCall<'_> {
        TimeMachineBarsGrpcCall::new(self, request)
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
