use crate::core::error::SdkError;
use crate::systems::aggregator::types::{FilesDownloadsRequest, FilesDownloadsResponse};
use crate::transport::http::HttpTransport;
use reqwest::Method;

pub async fn files_downloads(
    transport: &HttpTransport,
    request: &FilesDownloadsRequest,
) -> Result<FilesDownloadsResponse, SdkError> {
    let request = transport
        .request(Method::POST, "/v1/files/downloads")?
        .json(request);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<FilesDownloadsResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })
}
