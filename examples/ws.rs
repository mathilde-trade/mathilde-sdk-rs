// Example: public ws subscriptions and make-before-break
//
// What this example does:
// 1. Builds one authenticated client per public system surface.
// 2. Opens one small JSON WS subscription for Aggregator bars.
// 3. Opens one small JSON WS subscription for Primitives outputs.
// 4. Opens one small JSON WS subscription for Regime outputs.
// 5. Starts one Aggregator make-before-break swap and proves the promoted request takes over.
//
// This example keeps the flow minimal:
// - one bearer declaration at the beginning
// - one client per system
// - one bounded replay request per stream
// - one compact print summary per received frame

use std::env;
use std::error::Error;
use std::time::Duration;

use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{
    Aggregator, BarsWsFormat, BarsWsInboundFrame, BarsWsMakeBeforeBreak, BarsWsSubscribeRequest,
};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::primitives::{
    OutputsWsFormat as PrimitivesOutputsWsFormat,
    OutputsWsInboundFrame as PrimitivesOutputsWsInboundFrame,
    OutputsWsSubscribeRequest as PrimitivesOutputsWsSubscribeRequest, Primitives,
};
use mathilde_sdk_rs::systems::regime::{
    OutputsWsFormat as RegimeOutputsWsFormat, OutputsWsInboundFrame as RegimeOutputsWsInboundFrame,
    OutputsWsSubscribeRequest as RegimeOutputsWsSubscribeRequest, Regime,
};
use mathilde_sdk_rs::systems::types::Timeframe;
use tokio::time::timeout;

const FRAME_TIMEOUT_SECS: u64 = 20;
const MAX_FRAMES_PER_STREAM: usize = 3;
const MAX_MBB_FRAMES_UNTIL_PROMOTED: usize = 12;

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn Error>> {
    // Read the example bearer once and reuse it across all clients.
    let bearer = read_example_bearer()?;

    // Build one client per public system surface.
    let aggregator = Aggregator::client(Some(bearer.clone()))?;
    let primitives = Primitives::client(Some(bearer.clone()))?;
    let regime = Regime::client(Some(bearer))?;

    let aggregator_request = BarsWsSubscribeRequest {
        pairs: pairs(["BTCUSDT"]),
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        from_close: None,
        last_n_bars: Some(2),
        format: Some(BarsWsFormat::Json),
    };

    let primitives_request = PrimitivesOutputsWsSubscribeRequest {
        pairs: pairs(["BTCUSDT"]),
        tfs: vec![Timeframe::M1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        from_close: None,
        last_n_bars: Some(2),
        format: Some(PrimitivesOutputsWsFormat::Json),
    };

    let regime_request = RegimeOutputsWsSubscribeRequest {
        pairs: pairs(["BTCUSDT"]),
        tfs: vec![Timeframe::H1],
        metadata: Some(false),
        diagnostics: Some(false),
        family: None,
        group: None,
        secondary: Some(false),
        from_close: None,
        last_n_bars: Some(2),
        format: Some(RegimeOutputsWsFormat::Json),
    };

    // Read one short bounded frame slice from each public WS surface.
    let mut aggregator_ws = aggregator.connect_bars_ws(&aggregator_request).await?;
    read_aggregator_frames(
        "aggregator ws",
        &mut aggregator_ws,
        &aggregator_request,
        MAX_FRAMES_PER_STREAM,
    )
    .await?;

    let mut primitives_ws = primitives.connect_outputs_ws(&primitives_request).await?;
    read_primitives_frames(
        "primitives ws",
        &mut primitives_ws,
        &primitives_request,
        MAX_FRAMES_PER_STREAM,
    )
    .await?;

    let mut regime_ws = regime.connect_outputs_ws(&regime_request).await?;
    read_regime_frames(
        "regime ws",
        &mut regime_ws,
        &regime_request,
        MAX_FRAMES_PER_STREAM,
    )
    .await?;

    // Start one make-before-break swap on Aggregator and keep reading the active stream.
    let mut aggregator_mbb = aggregator
        .connect_bars_ws_make_before_break(&aggregator_request)
        .await?;
    let aggregator_swap_request = BarsWsSubscribeRequest {
        metadata: Some(true),
        ..aggregator_request.clone()
    };

    println!(
        "aggregator mbb active_pairs={} swap_in_progress={}",
        aggregator_mbb.active_request().pairs.len(),
        aggregator_mbb.swap_in_progress()
    );

    aggregator_mbb.begin_swap(&aggregator_swap_request)?;
    println!(
        "aggregator mbb requested_swap metadata={:?} swap_in_progress={}",
        aggregator_swap_request.metadata,
        aggregator_mbb.swap_in_progress()
    );

    read_aggregator_mbb_until_promoted(
        "aggregator mbb",
        &mut aggregator_mbb,
        &aggregator_swap_request,
        MAX_MBB_FRAMES_UNTIL_PROMOTED,
    )
    .await?;

    println!(
        "aggregator mbb active_metadata={:?} swap_in_progress={}",
        aggregator_mbb.active_request().metadata,
        aggregator_mbb.swap_in_progress()
    );

    Ok(())
}

fn read_example_bearer() -> Result<BearerToken, Box<dyn Error>> {
    match env::var("EXAMPLE_BEARER_TOKEN") {
        Ok(raw) => Ok(BearerToken::new(raw)?),
        Err(env::VarError::NotPresent) => Err(
            "EXAMPLE_BEARER_TOKEN is not exported for this process; cargo run does not load .env automatically"
                .into(),
        ),
        Err(error) => Err(Box::new(error)),
    }
}

async fn read_aggregator_frames(
    label: &str,
    connection: &mut mathilde_sdk_rs::systems::aggregator::BarsWsConnection,
    request: &BarsWsSubscribeRequest,
    max_frames: usize,
) -> Result<(), Box<dyn Error>> {
    for frame_index in 0..max_frames {
        let frame = timeout(
            Duration::from_secs(FRAME_TIMEOUT_SECS),
            connection.next_frame(request),
        )
        .await??;

        let Some(frame) = frame else {
            println!("{label} stream_closed");
            break;
        };

        let should_stop = print_aggregator_frame(label, frame_index, &frame);
        if should_stop {
            break;
        }
    }

    Ok(())
}

async fn read_primitives_frames(
    label: &str,
    connection: &mut mathilde_sdk_rs::systems::primitives::OutputsWsConnection,
    request: &PrimitivesOutputsWsSubscribeRequest,
    max_frames: usize,
) -> Result<(), Box<dyn Error>> {
    for frame_index in 0..max_frames {
        let frame = timeout(
            Duration::from_secs(FRAME_TIMEOUT_SECS),
            connection.next_frame(request),
        )
        .await??;

        let Some(frame) = frame else {
            println!("{label} stream_closed");
            break;
        };

        let should_stop = print_primitives_frame(label, frame_index, &frame);
        if should_stop {
            break;
        }
    }

    Ok(())
}

async fn read_regime_frames(
    label: &str,
    connection: &mut mathilde_sdk_rs::systems::regime::OutputsWsConnection,
    request: &RegimeOutputsWsSubscribeRequest,
    max_frames: usize,
) -> Result<(), Box<dyn Error>> {
    for frame_index in 0..max_frames {
        let frame = timeout(
            Duration::from_secs(FRAME_TIMEOUT_SECS),
            connection.next_frame(request),
        )
        .await??;

        let Some(frame) = frame else {
            println!("{label} stream_closed");
            break;
        };

        let should_stop = print_regime_frame(label, frame_index, &frame);
        if should_stop {
            break;
        }
    }

    Ok(())
}

async fn read_aggregator_mbb_until_promoted(
    label: &str,
    connection: &mut BarsWsMakeBeforeBreak,
    promoted_request: &BarsWsSubscribeRequest,
    max_frames: usize,
) -> Result<(), Box<dyn Error>> {
    for frame_index in 0..max_frames {
        // The active stream can be quiet while the candidate replay is validating,
        // so treat bounded timeouts as poll ticks instead of immediate failure.
        let frame = match timeout(
            Duration::from_secs(FRAME_TIMEOUT_SECS),
            connection.next_frame(),
        )
        .await
        {
            Ok(result) => result?,
            Err(_) => {
                println!(
                    "{label} frame={frame_index} poll_timeout swap_in_progress={}",
                    connection.swap_in_progress()
                );

                if !connection.swap_in_progress()
                    && connection.active_request().metadata == promoted_request.metadata
                {
                    println!(
                        "{label} swap_promoted active_metadata={:?} swap_in_progress={}",
                        connection.active_request().metadata,
                        connection.swap_in_progress()
                    );
                    return Ok(());
                }

                continue;
            }
        };

        let Some(frame) = frame else {
            println!("{label} stream_closed");
            break;
        };

        print_aggregator_frame(label, frame_index, &frame);

        if matches!(frame, BarsWsInboundFrame::Error(_)) {
            return Err("aggregator make-before-break returned an error frame".into());
        }

        // Promotion is proved only when the pending swap is gone and the active
        // request now matches the requested metadata shape.
        if !connection.swap_in_progress()
            && connection.active_request().metadata == promoted_request.metadata
        {
            println!(
                "{label} swap_promoted active_metadata={:?} swap_in_progress={}",
                connection.active_request().metadata,
                connection.swap_in_progress()
            );
            return Ok(());
        }
    }

    Err("aggregator make-before-break was not promoted within the bounded frame budget".into())
}

fn print_aggregator_frame(label: &str, frame_index: usize, frame: &BarsWsInboundFrame) -> bool {
    match frame {
        BarsWsInboundFrame::Meta(meta) => {
            println!(
                "{label} frame={frame_index} meta phase={:?} missing_pairs={} event={:?}",
                meta.phase,
                meta.missing_pairs.len(),
                meta.event
            );
            false
        }
        BarsWsInboundFrame::Error(error) => {
            println!(
                "{label} frame={frame_index} error kind={} error={}",
                error.kind, error.error
            );
            true
        }
        BarsWsInboundFrame::JsonRows(rows) | BarsWsInboundFrame::ProtobufRows(rows) => {
            let first = rows.first();
            println!(
                "{label} frame={frame_index} rows={} first_pair={:?} first_close_utc={:?}",
                rows.len(),
                first.map(|row| row.pair.as_str()),
                first.map(|row| row.close_utc.as_str())
            );
            true
        }
    }
}

fn print_primitives_frame(
    label: &str,
    frame_index: usize,
    frame: &PrimitivesOutputsWsInboundFrame,
) -> bool {
    match frame {
        PrimitivesOutputsWsInboundFrame::Meta(meta) => {
            println!(
                "{label} frame={frame_index} meta phase={:?} missing_pairs={} event={:?}",
                meta.phase,
                meta.missing_pairs.len(),
                meta.event
            );
            false
        }
        PrimitivesOutputsWsInboundFrame::Error(error) => {
            println!(
                "{label} frame={frame_index} error kind={} error={}",
                error.kind, error.error
            );
            true
        }
        PrimitivesOutputsWsInboundFrame::JsonRows(rows)
        | PrimitivesOutputsWsInboundFrame::ProtobufRows(rows) => {
            let first = rows.first();
            println!(
                "{label} frame={frame_index} rows={} first_pair={:?} first_close_utc={:?} first_age_ms={:?}",
                rows.len(),
                first.map(|row| row.row.pair.as_str()),
                first.map(|row| row.row.close_utc.as_str()),
                first.map(|row| row.age_ms)
            );
            true
        }
    }
}

fn print_regime_frame(
    label: &str,
    frame_index: usize,
    frame: &RegimeOutputsWsInboundFrame,
) -> bool {
    match frame {
        RegimeOutputsWsInboundFrame::Meta(meta) => {
            println!(
                "{label} frame={frame_index} meta phase={:?} missing_pairs={} event={:?}",
                meta.phase,
                meta.missing_pairs.len(),
                meta.event
            );
            false
        }
        RegimeOutputsWsInboundFrame::Error(error) => {
            println!(
                "{label} frame={frame_index} error kind={} error={}",
                error.kind, error.error
            );
            true
        }
        RegimeOutputsWsInboundFrame::JsonRows(rows)
        | RegimeOutputsWsInboundFrame::ProtobufRows(rows) => {
            let first = rows.first();
            println!(
                "{label} frame={frame_index} rows={} first_pair={:?} first_close_utc={:?} first_age_ms={:?}",
                rows.len(),
                first.map(|row| row.row.pair.as_str()),
                first.map(|row| row.row.close_utc.as_str()),
                first.map(|row| row.age_ms)
            );
            true
        }
    }
}
