use crate::core::auth::BearerToken;
use crate::core::config::GrpcTransportConfig;
use crate::core::error::SdkError;
use crate::transport::grpc::GrpcTransport;
use tonic::Code;
use url::Url;

#[tokio::test]
async fn test_grpc_transport_new_trims_trailing_slash_in_endpoint() {
    let config =
        GrpcTransportConfig::new("http://127.0.0.1:50051/").expect("valid grpc transport config");

    let transport = GrpcTransport::new(&config, None).expect("grpc transport");

    assert_eq!(transport.endpoint(), "http://127.0.0.1:50051");
}

#[tokio::test]
async fn test_grpc_transport_apply_bearer_inserts_authorization_metadata() {
    let config =
        GrpcTransportConfig::new("http://127.0.0.1:50051").expect("valid grpc transport config");
    let token = BearerToken::new("feed_public_token").expect("valid token");
    let transport = GrpcTransport::new(&config, Some(token)).expect("grpc transport");

    let request = tonic::Request::new(());
    let request = transport
        .apply_bearer(request)
        .expect("grpc bearer metadata");

    let authorization = request
        .metadata()
        .get("authorization")
        .expect("authorization metadata");

    assert_eq!(
        authorization.to_str().expect("metadata string"),
        "Bearer feed_public_token"
    );
}

#[tokio::test]
async fn test_grpc_transport_apply_bearer_without_token_leaves_metadata_empty() {
    let config =
        GrpcTransportConfig::new("http://127.0.0.1:50051").expect("valid grpc transport config");
    let transport = GrpcTransport::new(&config, None).expect("grpc transport");

    let request = transport
        .apply_bearer(tonic::Request::new(()))
        .expect("request without bearer");

    assert!(request.metadata().get("authorization").is_none());
}

#[tokio::test]
async fn test_grpc_transport_new_preserves_non_http_scheme_verbatim() {
    let config = GrpcTransportConfig {
        base_url: Url::parse("ftp://127.0.0.1:50051").expect("ftp url parses"),
    };

    let transport = GrpcTransport::new(&config, None).expect("grpc transport");

    assert_eq!(transport.endpoint(), "ftp://127.0.0.1:50051/");
}

#[test]
fn test_sdk_error_grpc_status_maps_code_and_message() {
    let error = SdkError::grpc_status(tonic::Status::new(Code::PermissionDenied, "forbidden"));

    match error {
        SdkError::GrpcStatus { code, message } => {
            assert_eq!(code, Code::PermissionDenied);
            assert_eq!(message, "forbidden");
        }
        other => panic!("expected grpc status error, got {other:?}"),
    }
}
