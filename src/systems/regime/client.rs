use crate::core::auth::BearerToken;
use crate::core::config::RegimeConfig;
use crate::core::error::SdkError;
use crate::streaming::make_before_break::MakeBeforeBreakConfig;
use crate::streaming::subscription::ExponentialBackoffConfig;
use crate::systems::regime::docs;
use crate::systems::regime::files;
use crate::systems::regime::messages_ws;
use crate::systems::regime::outputs_grpc;
use crate::systems::regime::outputs_http;
use crate::systems::regime::outputs_pagination::{
    RangeCall, RangeGrpcCall, SearchCall, SearchGrpcCall, TimeMachineCall, TimeMachineGrpcCall,
};
use crate::systems::regime::outputs_ws;
use crate::systems::regime::pairs;
use crate::systems::regime::types::{
    DocsRegistryRequest, DownloadedFile, FilesDownloadsRequest, FilesDownloadsResponse,
    FilesDownloadsRow, LatestGrpcRequest, LatestRequest, LatestResponse, PairsListRequest,
    PairsListResponse, PairsStatusRequest, PairsStatusResponse, PublicOpenApiDocument,
    RangeGrpcRequest, RangeRequest, RangeResponse, SearchGrpcRequest, SearchRequest,
    SearchResponse, TimeMachineGrpcRequest, TimeMachineRequest, TimeMachineResponse,
};
use crate::systems::regime::{
    MessagesWsConnection, OutputsWsConnection, OutputsWsMakeBeforeBreak, OutputsWsSubscribeRequest,
    RecoveringMessagesWsConnection, RecoveringOutputsWsConnection,
};
use crate::transport::grpc::GrpcTransport;
use crate::transport::http::HttpTransport;
use crate::transport::ws::WsTransport;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Regime {
    pub(crate) http: HttpTransport,
    pub(crate) grpc: Option<GrpcTransport>,
    pub(crate) ws: Option<WsTransport>,
}

impl Regime {
    pub fn new(config: RegimeConfig) -> Result<Self, SdkError> {
        let http = HttpTransport::new(&config.http, config.bearer_token.clone());
        let grpc = config
            .grpc
            .as_ref()
            .map(|grpc| GrpcTransport::new(grpc, config.bearer_token.clone()))
            .transpose()?;
        let ws = config
            .ws
            .as_ref()
            .map(|ws| WsTransport::new(ws, config.bearer_token.as_ref()));

        Ok(Self { http, grpc, ws })
    }

    pub fn client(bearer_token: Option<BearerToken>) -> Result<Self, SdkError> {
        Self::new(RegimeConfig::mathilde_public_default(bearer_token)?)
    }

    pub async fn docs_system(&self) -> Result<serde_json::Value, SdkError> {
        docs::docs_system(&self.http).await
    }

    pub async fn docs_summary(&self) -> Result<serde_json::Value, SdkError> {
        docs::docs_summary(&self.http).await
    }

    pub async fn docs_taxonomy(&self) -> Result<serde_json::Value, SdkError> {
        docs::docs_taxonomy(&self.http).await
    }

    pub async fn docs_registry(
        &self,
        request: &DocsRegistryRequest,
    ) -> Result<serde_json::Value, SdkError> {
        docs::docs_registry(&self.http, request).await
    }

    pub async fn docs_endpoints(&self) -> Result<serde_json::Value, SdkError> {
        docs::docs_endpoints(&self.http).await
    }

    pub async fn openapi(&self) -> Result<PublicOpenApiDocument, SdkError> {
        docs::openapi(&self.http).await
    }

    pub async fn latest(&self, request: &LatestRequest) -> Result<LatestResponse, SdkError> {
        outputs_http::latest_outputs(&self.http, request).await
    }

    pub async fn latest_grpc(
        &self,
        request: &LatestGrpcRequest,
    ) -> Result<LatestResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        outputs_grpc::latest_outputs_grpc(grpc, request).await
    }

    pub async fn range(&self, request: &RangeRequest) -> Result<RangeResponse, SdkError> {
        outputs_http::range_outputs(&self.http, request).await
    }

    pub async fn range_grpc(&self, request: &RangeGrpcRequest) -> Result<RangeResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        outputs_grpc::range_outputs_grpc(grpc, request).await
    }

    pub fn range_call(&self, request: RangeRequest) -> RangeCall<'_> {
        RangeCall::new(self, request)
    }

    pub fn range_grpc_call(&self, request: RangeGrpcRequest) -> RangeGrpcCall<'_> {
        RangeGrpcCall::new(self, request)
    }

    pub async fn search(&self, request: &SearchRequest) -> Result<SearchResponse, SdkError> {
        outputs_http::search_outputs(&self.http, request).await
    }

    pub async fn search_grpc(
        &self,
        request: &SearchGrpcRequest,
    ) -> Result<SearchResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        outputs_grpc::search_outputs_grpc(grpc, request).await
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
        outputs_http::time_machine_outputs(&self.http, request).await
    }

    pub async fn time_machine_grpc(
        &self,
        request: &TimeMachineGrpcRequest,
    ) -> Result<TimeMachineResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        outputs_grpc::time_machine_outputs_grpc(grpc, request).await
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

    pub async fn connect_outputs_ws(
        &self,
        request: &OutputsWsSubscribeRequest,
    ) -> Result<OutputsWsConnection, SdkError> {
        let ws = self
            .ws
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("ws"))?;
        outputs_ws::OutputsWsConnection::connect(ws, request).await
    }

    pub async fn connect_outputs_ws_make_before_break(
        &self,
        request: &OutputsWsSubscribeRequest,
    ) -> Result<OutputsWsMakeBeforeBreak, SdkError> {
        let ws = self
            .ws
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("ws"))?;
        outputs_ws::OutputsWsMakeBeforeBreak::connect(ws, request, MakeBeforeBreakConfig::default())
            .await
    }

    pub async fn connect_outputs_ws_recovering(
        &self,
        request: &OutputsWsSubscribeRequest,
        config: ExponentialBackoffConfig,
    ) -> Result<RecoveringOutputsWsConnection, SdkError> {
        let ws = self
            .ws
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("ws"))?;
        outputs_ws::RecoveringOutputsWsConnection::connect(ws, request, config).await
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
