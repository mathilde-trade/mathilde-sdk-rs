mod client;
mod docs;
mod files;
mod messages_ws;
mod outputs_grpc;
mod outputs_http;
mod outputs_pagination;
mod outputs_ws;
mod pairs;
mod types;

pub use crate::generated::regime::{
    OutputBarsMetadata, OutputMetadata, OutputProcessDiagnostic, ProcessorFamily, ProcessorGroup,
};
pub use client::Regime;
pub use messages_ws::{
    MessagesWsClientFrame, MessagesWsConnection, MessagesWsErrorFrame, MessagesWsHeartbeatFrame,
    MessagesWsMessageFrame, MessagesWsServerFrame, MessagesWsSubscribeFrame,
    MessagesWsSubscribedFrame, MessagesWsUnsubscribeFrame, RecoveringMessagesWsConnection,
};

pub use outputs_pagination::{
    RangeCall, RangeGrpcCall, RangeGrpcPager, RangePager, SearchCall, SearchGrpcCall,
    SearchGrpcPager, SearchPager, TimeMachineCall, TimeMachineGrpcCall, TimeMachineGrpcPager,
    TimeMachinePager,
};
pub use outputs_ws::{
    NormalizedOutputsWsSubscribeRequest, OutputsWsConnection, OutputsWsErrorFrame, OutputsWsFormat,
    OutputsWsInboundFrame, OutputsWsMakeBeforeBreak, OutputsWsMetaFrame, OutputsWsPhase,
    OutputsWsSubscribeRequest, RecoveringOutputsWsConnection,
};
pub use types::{
    ComputedFields, DocsRegistryRequest, DownloadedFile, FilesDownloadsRequest,
    FilesDownloadsResponse, FilesDownloadsRow, LatestGrpcRequest, LatestPresentRow, LatestRequest,
    LatestResponse, OutputRow, OutputView, PairStatusBootstrap, PairStatusHistoryBlock,
    PairStatusReadinessBlock, PairStatusReadinessCell, PairStatusRow, PairStatusStatusBlock,
    PairsListRequest, PairsListResponse, PairsStatusRequest, PairsStatusResponse,
    PublicOpenApiDocument, RangeGrpcRequest, RangeRequest, RangeResponse, RangeTraverseResult,
    SearchGrpcRequest, SearchRequest, SearchResponse, SearchTraverseResult, TimeMachineGrpcRequest,
    TimeMachineRequest, TimeMachineResponse, TimeMachineRow, TimeMachineTraverseResult,
};

#[allow(unused_imports)]
pub(crate) mod raw {
    pub use crate::generated::regime::raw::{
        OutputBarsMetadata, OutputMetadata, OutputProcessDiagnostic, ProcessorFamily,
        ProcessorGroup,
    };
}

#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use outputs_grpc::latest_outputs_grpc;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use outputs_http::latest_outputs;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use types::{
    RegimeOutputMode, diagnostics_enabled, selector_family_names, selector_group_names,
};
