use crate::core::error::SdkError;
use crate::systems::aggregator::types::{FilesDownloadsRequest, FilesDownloadsResponse};
use crate::transport::http::HttpTransport;
use reqwest::Method;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct NormalizedFilesDownloadsRequest<'a> {
    period: &'a Option<String>,
    pairs: Vec<String>,
    tfs: &'a Vec<String>,
    start_label_utc: &'a Option<String>,
    end_label_utc: &'a Option<String>,
    order: &'a Option<String>,
}

fn parse_pairs_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub async fn files_downloads(
    transport: &HttpTransport,
    request: &FilesDownloadsRequest,
) -> Result<FilesDownloadsResponse, SdkError> {
    let normalized = NormalizedFilesDownloadsRequest {
        period: &request.period,
        pairs: parse_pairs_csv(&request.pairs),
        tfs: &request.tfs,
        start_label_utc: &request.start_label_utc,
        end_label_utc: &request.end_label_utc,
        order: &request.order,
    };

    let request = transport
        .request(Method::POST, "/v1/files/downloads")?
        .json(&normalized);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;
    response
        .json::<FilesDownloadsResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })
}
