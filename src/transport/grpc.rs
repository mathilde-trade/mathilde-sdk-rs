use crate::core::auth::BearerToken;
use crate::core::config::GrpcTransportConfig;
use crate::core::error::SdkError;
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, Endpoint};

#[derive(Debug, Clone)]
pub struct GrpcTransport {
    channel: Channel,
    #[cfg(test)]
    endpoint_uri: String,
    bearer_token: Option<BearerToken>,
}

impl GrpcTransport {
    pub fn new(
        config: &GrpcTransportConfig,
        bearer_token: Option<BearerToken>,
    ) -> Result<Self, SdkError> {
        let endpoint_uri = Self::endpoint_uri(config);
        let channel = Endpoint::new(endpoint_uri.clone())
            .map_err(SdkError::grpc_transport)?
            .connect_lazy();

        Ok(Self {
            channel,
            #[cfg(test)]
            endpoint_uri,
            bearer_token,
        })
    }

    pub fn endpoint_uri(config: &GrpcTransportConfig) -> String {
        match config.base_url.scheme() {
            "http" | "https" => config.base_url.as_str().trim_end_matches('/').to_string(),
            _ => config.base_url.as_str().to_string(),
        }
    }

    #[cfg(test)]
    pub fn endpoint(&self) -> &str {
        &self.endpoint_uri
    }

    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    pub fn apply_bearer<T>(
        &self,
        mut request: tonic::Request<T>,
    ) -> Result<tonic::Request<T>, SdkError> {
        if let Some(token) = self.bearer_token.as_ref() {
            let value = MetadataValue::try_from(format!("Bearer {}", token.as_str()))
                .map_err(|_| SdkError::grpc_metadata("invalid grpc bearer token metadata"))?;
            request.metadata_mut().insert("authorization", value);
        }
        Ok(request)
    }
}
