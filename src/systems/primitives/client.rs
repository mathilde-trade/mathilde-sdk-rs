use crate::core::auth::BearerToken;
use crate::core::config::PrimitivesConfig;
use crate::core::error::SdkError;
use crate::streaming::make_before_break::MakeBeforeBreakConfig;
use crate::streaming::subscription::ExponentialBackoffConfig;
use crate::systems::primitives::docs;
use crate::systems::primitives::files;
use crate::systems::primitives::messages_ws;
use crate::systems::primitives::outputs_grpc;
use crate::systems::primitives::outputs_http;
use crate::systems::primitives::outputs_pagination::{
    RangeOutputsCall, RangeOutputsGrpcCall, SearchOutputsCall, SearchOutputsGrpcCall,
    TimeMachineOutputsCall, TimeMachineOutputsGrpcCall,
};
use crate::systems::primitives::outputs_ws;
use crate::systems::primitives::pairs;
use crate::systems::primitives::types::{
    DocsRegistryRequest, DownloadedFile, FilesDownloadsRequest, FilesDownloadsResponse,
    FilesDownloadsRow, LatestOutputsGrpcRequest, LatestOutputsRequest, LatestOutputsResponse,
    PairsListRequest, PairsListResponse, PairsStatusRequest, PairsStatusResponse,
    PublicOpenApiDocument, RangeOutputsGrpcRequest, RangeOutputsRequest, RangeOutputsResponse,
    SearchOutputsGrpcRequest, SearchOutputsRequest, SearchOutputsResponse,
    TimeMachineOutputsGrpcRequest, TimeMachineOutputsRequest, TimeMachineOutputsResponse,
};
use crate::systems::primitives::{
    MessagesWsConnection, OutputsWsConnection, OutputsWsMakeBeforeBreak, OutputsWsSubscribeRequest,
    RecoveringMessagesWsConnection, RecoveringOutputsWsConnection,
};
use crate::transport::grpc::GrpcTransport;
use crate::transport::http::HttpTransport;
use crate::transport::ws::WsTransport;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Primitives {
    pub(crate) http: HttpTransport,
    pub(crate) grpc: Option<GrpcTransport>,
    pub(crate) ws: Option<WsTransport>,
}

impl Primitives {
    pub fn new(config: PrimitivesConfig) -> Result<Self, SdkError> {
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
        Self::new(PrimitivesConfig::mathilde_public_default(bearer_token)?)
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

    pub async fn latest(
        &self,
        request: &LatestOutputsRequest,
    ) -> Result<LatestOutputsResponse, SdkError> {
        outputs_http::latest_outputs(&self.http, request).await
    }

    pub async fn latest_grpc(
        &self,
        request: &LatestOutputsGrpcRequest,
    ) -> Result<LatestOutputsResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        outputs_grpc::latest_outputs_grpc(grpc, request).await
    }

    pub async fn range(
        &self,
        request: &RangeOutputsRequest,
    ) -> Result<RangeOutputsResponse, SdkError> {
        outputs_http::range_outputs(&self.http, request).await
    }

    pub async fn range_grpc(
        &self,
        request: &RangeOutputsGrpcRequest,
    ) -> Result<RangeOutputsResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        outputs_grpc::range_outputs_grpc(grpc, request).await
    }

    pub fn range_call(&self, request: RangeOutputsRequest) -> RangeOutputsCall<'_> {
        RangeOutputsCall::new(self, request)
    }

    pub fn range_grpc_call(&self, request: RangeOutputsGrpcRequest) -> RangeOutputsGrpcCall<'_> {
        RangeOutputsGrpcCall::new(self, request)
    }

    pub async fn search(
        &self,
        request: &SearchOutputsRequest,
    ) -> Result<SearchOutputsResponse, SdkError> {
        outputs_http::search_outputs(&self.http, request).await
    }

    pub async fn search_grpc(
        &self,
        request: &SearchOutputsGrpcRequest,
    ) -> Result<SearchOutputsResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        outputs_grpc::search_outputs_grpc(grpc, request).await
    }

    pub fn search_call(&self, request: SearchOutputsRequest) -> SearchOutputsCall<'_> {
        SearchOutputsCall::new(self, request)
    }

    pub fn search_grpc_call(&self, request: SearchOutputsGrpcRequest) -> SearchOutputsGrpcCall<'_> {
        SearchOutputsGrpcCall::new(self, request)
    }

    pub async fn time_machine(
        &self,
        request: &TimeMachineOutputsRequest,
    ) -> Result<TimeMachineOutputsResponse, SdkError> {
        outputs_http::time_machine_outputs(&self.http, request).await
    }

    pub async fn time_machine_grpc(
        &self,
        request: &TimeMachineOutputsGrpcRequest,
    ) -> Result<TimeMachineOutputsResponse, SdkError> {
        let grpc = self
            .grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))?;
        outputs_grpc::time_machine_outputs_grpc(grpc, request).await
    }

    pub fn time_machine_call(
        &self,
        request: TimeMachineOutputsRequest,
    ) -> TimeMachineOutputsCall<'_> {
        TimeMachineOutputsCall::new(self, request)
    }

    pub fn time_machine_grpc_call(
        &self,
        request: TimeMachineOutputsGrpcRequest,
    ) -> TimeMachineOutputsGrpcCall<'_> {
        TimeMachineOutputsGrpcCall::new(self, request)
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
