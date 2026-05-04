// use crate::core::auth::BearerToken;
// use crate::systems::helpers::pairs;
// use crate::systems::primitives::{
//     LatestOutputsResponse, PrimitiveOutput, Primitives, ProcessorFamily,
//     ProcessorProjectedOutputMin,
// };
// use crate::systems::types::Timeframe;

// pub async fn my_test() {


//     let pr = Primitives::client(Some(BearerToken::new("feed_public_token")?))?;
//     let latest_req = pr
//         .latest(&crate::systems::primitives::LatestOutputsRequest {
//             pairs: pairs(["BTCUSDT"]),
//             tf: Timeframe::M1,
//             latest_mode: None,
//             family: Some(vec![ProcessorFamily::MovingAverages]),
//             group: None,
//             metadata: None,
//             diagnostics: None,
//             format: None,
//         })
//         .await?;

//     if let PrimitiveOutput::ProjectedMin(row) = &latest_req.rows[0].output {
//         println!("Projected output: {:?}", row.);
//     } else {
//         println!("Output is not projected");
//     }
// }
