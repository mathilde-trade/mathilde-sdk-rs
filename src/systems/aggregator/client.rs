use crate::core::config::AggregatorConfig;
use crate::core::error::SdkError;
use crate::systems::aggregator::bars_http;
use crate::systems::aggregator::docs;
use crate::systems::aggregator::types::{LatestBarsRequest, LatestBarsResponse, PublicDocResponse};
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

    pub async fn latest_bars(
        &self,
        request: &LatestBarsRequest,
    ) -> Result<LatestBarsResponse, SdkError> {
        bars_http::latest_bars(&self.http, request).await
    }
}
