use crate::core::auth::BearerToken;
use crate::core::error::SdkError;
use url::Url;

#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    pub base_url: Url,
}

impl HttpTransportConfig {
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, SdkError> {
        let raw = base_url.as_ref();
        let base_url =
            Url::parse(raw).map_err(|source| SdkError::invalid_url(raw.to_string(), source))?;
        Ok(Self { base_url })
    }
}

#[derive(Debug, Clone)]
pub struct GrpcTransportConfig {
    pub base_url: Url,
}

impl GrpcTransportConfig {
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, SdkError> {
        let raw = base_url.as_ref();
        let base_url =
            Url::parse(raw).map_err(|source| SdkError::invalid_url(raw.to_string(), source))?;
        Ok(Self { base_url })
    }
}

#[derive(Debug, Clone)]
pub struct WsTransportConfig {
    pub base_url: Url,
}

impl WsTransportConfig {
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, SdkError> {
        let raw = base_url.as_ref();
        let base_url =
            Url::parse(raw).map_err(|source| SdkError::invalid_url(raw.to_string(), source))?;
        Ok(Self { base_url })
    }
}

#[derive(Debug, Clone, Default)]
pub struct AggregatorConfig {
    pub http: Option<HttpTransportConfig>,
    pub grpc: Option<GrpcTransportConfig>,
    pub ws: Option<WsTransportConfig>,
    pub bearer_token: Option<BearerToken>,
}

impl AggregatorConfig {
    pub fn require_http(&self) -> Result<&HttpTransportConfig, SdkError> {
        self.http
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("http"))
    }

    pub fn require_grpc(&self) -> Result<&GrpcTransportConfig, SdkError> {
        self.grpc
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("grpc"))
    }

    pub fn require_ws(&self) -> Result<&WsTransportConfig, SdkError> {
        self.ws
            .as_ref()
            .ok_or_else(|| SdkError::missing_transport_config("ws"))
    }
}
