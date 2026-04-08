use thiserror::Error;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("invalid url `{input}`: {source}")]
    InvalidUrl {
        input: String,
        #[source]
        source: url::ParseError,
    },
    #[error("missing required transport config: {transport}")]
    MissingTransportConfig { transport: &'static str },
    #[error("invalid auth token: {message}")]
    InvalidAuthToken { message: &'static str },
    #[error("invalid time input: {message}")]
    InvalidTimeInput { message: String },
    #[error("request build failed: {message}")]
    RequestBuild { message: String },
    #[error("transport error: {source}")]
    Transport {
        #[source]
        source: reqwest::Error,
    },
    #[error("grpc transport error: {source}")]
    GrpcTransport {
        #[source]
        source: tonic::transport::Error,
    },
    #[error("grpc status {code}: {message}")]
    GrpcStatus {
        code: tonic::Code,
        message: String,
    },
    #[error("grpc metadata error: {message}")]
    GrpcMetadata { message: String },
    #[error("ws transport error: {message}")]
    WsTransport { message: String },
    #[error("http status {status}: {body}")]
    HttpStatus { status: u16, body: String },
    #[error("decode error: {source}")]
    Decode {
        #[source]
        source: reqwest::Error,
    },
    #[error("contract drift: {message}")]
    ContractDrift { message: String },
}

impl SdkError {
    pub fn invalid_url(input: String, source: url::ParseError) -> Self {
        Self::InvalidUrl { input, source }
    }

    pub fn missing_transport_config(transport: &'static str) -> Self {
        Self::MissingTransportConfig { transport }
    }

    pub fn invalid_auth_token(message: &'static str) -> Self {
        Self::InvalidAuthToken { message }
    }

    pub fn invalid_time_input(message: impl Into<String>) -> Self {
        Self::InvalidTimeInput {
            message: message.into(),
        }
    }

    pub fn request_build(message: impl Into<String>) -> Self {
        Self::RequestBuild {
            message: message.into(),
        }
    }

    pub fn grpc_transport(source: tonic::transport::Error) -> Self {
        Self::GrpcTransport { source }
    }

    pub fn grpc_status(status: tonic::Status) -> Self {
        Self::GrpcStatus {
            code: status.code(),
            message: status.message().to_string(),
        }
    }

    pub fn grpc_metadata(message: impl Into<String>) -> Self {
        Self::GrpcMetadata {
            message: message.into(),
        }
    }

    pub fn ws_transport(message: impl Into<String>) -> Self {
        Self::WsTransport {
            message: message.into(),
        }
    }

    pub fn contract_drift(message: impl Into<String>) -> Self {
        Self::ContractDrift {
            message: message.into(),
        }
    }
}
