use crate::core::auth::BearerToken;
use crate::systems::aggregator::{Aggregator, LatestRequest, RangeGrpcRequest, RangeRequest};
use crate::systems::helpers::pairs;
use crate::systems::types::{LatestMode, Timeframe};

#[tokio::test]
pub async fn my_test() -> Result<(), Box<dyn std::error::Error>> {
    let agg = Aggregator::client(Some(BearerToken::new(
        "9720f73c-7256-4a09-82bd-a2448b8770c8.540a03d2e13542ccbd1ee3d166ecf71d",
    )?))?;

    let latest_request = LatestRequest {
        pairs: pairs(["BTCUSDT", "ETHUSDT"]),
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        format: None,
        metadata: None,
    };

    let range_request = RangeRequest {
        pairs: pairs(["BTCUSDT", "ETHUSDT"]),
        tf: Timeframe::M5,
        align_mode: None,
        format: Some(crate::systems::types::HttpFormat::Protobuf),
        metadata: None,
        close_start: Some("2026-02-02T00:00:00Z".into()),
        close_end: None,
        limit: Some(1000),
        cursor: None,
    };

    let grpc_request = RangeGrpcRequest::from(&range_request);

    let latest = agg.latest(&latest_request).await?;

    for output in latest.rows {
        let bar = &output;
        let metadta = &bar.metadata;
        println!("Output: {:?}", output.age_ms);
        if let Some(metadata) = metadta {
            println!("Metadata: {:?}", metadata);
        }
        println!(
            "Open: {}, High: {}, Low: {}, Close: {}",
            bar.o, bar.h, bar.l, bar.c
        );
        if bar.o > bar.c {
            println!("Price went down in this bar.");
        } else if bar.o < bar.c {
            println!("Price went up in this bar.");
        } else {
            println!("Price stayed the same in this bar.");
        }
    }
    let range = agg.range(&range_request).await?;
    for output in range.rows {
        let bar = &output;
        let metadta = &bar.metadata;
        println!("Output: {:?}", output.age_ms);
        if let Some(metadata) = metadta {
            println!("Metadata: {:?}", metadata);
        }
        println!(
            "Open: {}, High: {}, Low: {}, Close: {}",
            bar.o, bar.h, bar.l, bar.c
        );
        if bar.o > bar.c {
            println!("Price went down in this bar.");
        } else if bar.o < bar.c {
            println!("Price went up in this bar.");
        } else {
            println!("Price stayed the same in this bar.");
        }
    }
    let range_t = agg.range_grpc_call(grpc_request).traverse().await?;
    println!("pages_fetched={}", range_t.pages_fetched);
    let mut count = 0;
    for page in range_t.pages {
        for bar in page.rows {
            println!(
                "{} {} o={} h={} l={} c={}",
                bar.pair, bar.close_utc, bar.o, bar.h, bar.l, bar.c
            );
            count += 1;
        }
    }
    println!("total bars={}", count);

    Ok(())
}
