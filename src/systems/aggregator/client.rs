use crate::core::config::AggregatorConfig;
use crate::core::error::SdkError;
use crate::systems::aggregator::bars_http;
use crate::systems::aggregator::docs;
use crate::systems::aggregator::files;
use crate::systems::aggregator::pairs;
use crate::systems::aggregator::types::{
    FilesDownloadsRequest, FilesDownloadsResponse, LatestBarsRequest, LatestBarsResponse,
    PairsListRequest, PairsListResponse, PairsStatusRequest, PairsStatusResponse, RangeBarsRequest,
    RangeBarsResponse, PublicDocResponse, PublicDocWithIndexResponse, PublicOpenApiDocument,
    SearchBarsRequest, SearchBarsResponse, TimeMachineBarsRequest,
    TimeMachineBarsResponse,
};
use crate::transport::http::HttpTransport;

#[derive(Debug, Clone)]
pub struct AggregatorClient {
    http: HttpTransport,
}

impl AggregatorClient {
    pub fn new(config: AggregatorConfig) -> Result<Self, SdkError> {
        let http = config.require_http()?.clone();
        Ok(Self {
            http: HttpTransport::new(&http, config.bearer_token.clone()),
        })
    }

    pub async fn docs_system(&self) -> Result<PublicDocResponse, SdkError> {
        docs::docs_system(&self.http).await
    }

    pub async fn docs_themes(&self) -> Result<PublicDocWithIndexResponse, SdkError> {
        docs::docs_themes(&self.http).await
    }

    pub async fn docs_endpoints(&self) -> Result<PublicDocResponse, SdkError> {
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

    pub async fn range_bars(
        &self,
        request: &RangeBarsRequest,
    ) -> Result<RangeBarsResponse, SdkError> {
        bars_http::range_bars(&self.http, request).await
    }

    pub async fn search_bars(
        &self,
        request: &SearchBarsRequest,
    ) -> Result<SearchBarsResponse, SdkError> {
        bars_http::search_bars(&self.http, request).await
    }

    pub async fn time_machine_bars(
        &self,
        request: &TimeMachineBarsRequest,
    ) -> Result<TimeMachineBarsResponse, SdkError> {
        bars_http::time_machine_bars(&self.http, request).await
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
}
