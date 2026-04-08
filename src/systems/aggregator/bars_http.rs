use crate::core::error::SdkError;
use crate::generated::aggregator::bars_proto::mathilde::feed::bars::v1::BarsLatestResponseV1;
use crate::systems::aggregator::types::{LatestBarsRequest, LatestBarsResponse};
use crate::systems::types::HttpFormat;
use crate::transport::http::HttpTransport;
use prost::Message;
use reqwest::Method;

pub async fn latest_bars(
    transport: &HttpTransport,
    request_body: &LatestBarsRequest,
) -> Result<LatestBarsResponse, SdkError> {
    let request = transport
        .request(Method::POST, "/v1/bars/latest")?
        .json(request_body);
    let response = transport.execute(request).await?;
    let response = transport.ensure_success(response).await?;

    if matches!(request_body.format, Some(HttpFormat::Protobuf)) {
        let body = response
            .bytes()
            .await
            .map_err(|source| SdkError::Decode { source })?;
        let proto = BarsLatestResponseV1::decode(body.as_ref())
            .map_err(|source| SdkError::contract_drift(format!("protobuf decode failed: {source}")))?;
        return LatestBarsResponse::from_proto(proto);
    }

    response
        .json::<LatestBarsResponse>()
        .await
        .map_err(|source| SdkError::Decode { source })
}
