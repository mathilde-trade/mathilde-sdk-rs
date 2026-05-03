use crate::core::auth::BearerToken;
use crate::core::error::SdkError;
use url::Url;

#[derive(Debug, Clone, Copy, Default)]
pub struct MathildePublicHosts;

impl MathildePublicHosts {
    pub const AGGREGATOR_HTTP: &'static str = "https://aggregator.api.mathilde.dev";
    pub const AGGREGATOR_GRPC: &'static str = "https://aggregator.grpc.mathilde.dev";
    pub const PRIMITIVES_HTTP: &'static str = "https://primitives.api.mathilde.dev";
    pub const PRIMITIVES_GRPC: &'static str = "https://primitives.grpc.mathilde.dev";
    pub const REGIME_HTTP: &'static str = "https://regime.api.mathilde.dev";
    pub const REGIME_GRPC: &'static str = "https://regime.grpc.mathilde.dev";
    pub const INTRO: &'static str = "https://api.mathilde.dev";
}

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

fn derive_ws_url_from_http_base(http_base_url: &str) -> Result<Url, SdkError> {
    let mut ws_url = Url::parse(http_base_url)
        .map_err(|source| SdkError::invalid_url(http_base_url.to_string(), source))?;
    ws_url
        .set_scheme("wss")
        .map_err(|_| SdkError::contract_drift("failed to derive wss url from https base"))?;
    Ok(ws_url)
}

impl AggregatorConfig {
    pub fn mathilde_public_default(bearer_token: Option<BearerToken>) -> Result<Self, SdkError> {
        Ok(Self {
            http: Some(HttpTransportConfig::new(
                MathildePublicHosts::AGGREGATOR_HTTP,
            )?),
            grpc: Some(GrpcTransportConfig::new(
                MathildePublicHosts::AGGREGATOR_GRPC,
            )?),
            ws: Some(WsTransportConfig {
                base_url: derive_ws_url_from_http_base(MathildePublicHosts::AGGREGATOR_HTTP)?,
            }),
            bearer_token,
        })
    }

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
