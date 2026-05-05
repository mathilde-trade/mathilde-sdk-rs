use crate::core::auth::BearerToken;
use crate::core::config::IntroConfig;
use crate::core::error::SdkError;
use crate::systems::intro::intro;
use crate::transport::http::HttpTransport;

#[derive(Debug, Clone)]
pub struct Intro {
    http: HttpTransport,
}

impl Intro {
    pub fn new(config: IntroConfig) -> Result<Self, SdkError> {
        let http = config.require_http().clone();
        Ok(Self {
            http: HttpTransport::new(&http, config.bearer_token.clone()),
        })
    }

    pub fn client(bearer_token: Option<BearerToken>) -> Result<Self, SdkError> {
        Self::new(IntroConfig::mathilde_public_default(bearer_token)?)
    }

    pub async fn intro(&self) -> Result<serde_json::Value, SdkError> {
        intro::intro(&self.http).await
    }
}
