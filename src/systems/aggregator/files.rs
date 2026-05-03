use crate::core::error::SdkError;
use crate::systems::aggregator::types::{
    DownloadedFile, FilesDownloadsRequest, FilesDownloadsResponse, FilesDownloadsRow,
    normalize_required_pair_values,
};
use crate::transport::http::HttpTransport;
use reqwest::Method;
use serde::Serialize;
use std::path::{Path, PathBuf};

const DEFAULT_DOWNLOAD_ROOT: &str = "/tmp/mathilde";

#[derive(Debug, Serialize)]
struct NormalizedFilesDownloadsRequest<'a> {
    period: &'a Option<String>,
    pairs: Vec<String>,
    tfs: &'a Vec<String>,
    start_label_utc: &'a Option<String>,
    end_label_utc: &'a Option<String>,
    order: &'a Option<String>,
}

pub async fn files_downloads(
    transport: &HttpTransport,
    request: &FilesDownloadsRequest,
) -> Result<FilesDownloadsResponse, SdkError> {
    let normalized = NormalizedFilesDownloadsRequest {
        period: &request.period,
        pairs: normalize_required_pair_values(&request.pairs, "files downloads")?,
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

fn sanitize_path_component(raw: &str) -> String {
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn destination_path_for_row(root: &Path, row: &FilesDownloadsRow) -> PathBuf {
    root.join(sanitize_path_component(&row.period))
        .join(sanitize_path_component(&row.pair))
        .join(sanitize_path_component(&row.tf))
        .join(format!(
            "{}.parquet",
            sanitize_path_component(&row.label_utc)
        ))
}

pub async fn files_download_items(
    transport: &HttpTransport,
    items: &[FilesDownloadsRow],
    destination_root: Option<&Path>,
) -> Result<Vec<DownloadedFile>, SdkError> {
    if !transport.has_bearer_token() {
        return Err(SdkError::unsupported_or_unproved_usage(
            "files_download_items requires bearer auth configured on the client",
        ));
    }

    let root = destination_root.unwrap_or_else(|| Path::new(DEFAULT_DOWNLOAD_ROOT));
    let mut downloaded = Vec::with_capacity(items.len());

    for row in items {
        let destination_path = destination_path_for_row(root, row);
        let request = transport.request_absolute(Method::GET, &row.url)?;
        let bytes_written = transport
            .download_to_path(request, &destination_path)
            .await?;

        downloaded.push(DownloadedFile {
            row: row.clone(),
            destination_path: destination_path.to_string_lossy().into_owned(),
            bytes_written,
        });
    }

    Ok(downloaded)
}
