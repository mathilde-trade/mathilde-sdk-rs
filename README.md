# mathilde-sdk-rs

`mathilde-sdk-rs` is the public Rust SDK for MATHILDE feed access. It binds
public HTTP, gRPC, and WebSocket contracts faithfully, keeps request and
transport behavior explicit, and does not hide unproved convenience semantics.
MATHILDE measures, not predicts, and this SDK is a public client-contract
layer, not an opinion or trading layer.

## What This Is

This SDK is a thin public-surface binding for MATHILDE systems that have a
proved public contract. Today that means the public `aggregator` feed surface.

At the current implemented scope, the SDK gives you:

- public documentation and OpenAPI reads
- public discovery surfaces for pairs and downloadable files
- bars reads over HTTP and gRPC
- bars streaming and messages streaming over WebSocket
- typed request and response surfaces
- explicit shared conventions for time inputs, request shapes, and WS recovery

The SDK is transport-aware, but it is still endpoint-faithful. It does not try
to turn every endpoint into the same abstraction if the upstream contract does
not justify that.

## What This Is Not

This SDK is not:

- a trading framework
- a prediction layer
- a private or admin client
- a hidden traversal engine
- a hidden retry layer by default
- a claim that every source-system behavior is covered just because the public
  feed exists

If a behavior is not proved by the current code, specs, and tests, the README
does not claim it.

## Supported Public Surfaces

The current implemented public surface is the `aggregator` feed client.

There are three broad families of work a caller can do with it.

First, you can inspect the public documentation and discovery layer. That
includes top-level docs pages, the public OpenAPI document, pair status and
pair lists, and file-download discovery.

Second, you can query bars as bounded request/response surfaces. Those are the
latest, range, search, and time-machine families. They exist over HTTP, and
the bars families also exist over gRPC.

Third, you can consume streaming public surfaces over WebSocket. Bars streaming
and messages streaming are both implemented, but they do not use the same
subscription model. Bars is immutable per connection. Messages supports
in-band subscribe and unsubscribe.

## Core Conventions

The SDK uses a small number of explicit conventions across the public surface.

Bars-family method names keep `bars` as the suffix:

- `latest_bars`
- `range_bars`
- `search_bars`
- `time_machine_bars`

Transport-equivalent gRPC methods append `_grpc`:

- `latest_bars_grpc`
- `range_bars_grpc`
- `search_bars_grpc`
- `time_machine_bars_grpc`

HTTP and gRPC request shapes stay aligned where possible. The main default
difference is that HTTP request structs may expose `format`, while gRPC does
not.

Bars-family pair lists use CSV strings:

```rust
pairs: "BTCUSDT,ETHUSDT".to_string()
```

Mixed UTC-string and millisecond inputs use `TimeInput`:

```rust
use mathilde_sdk_rs::core::time::TimeInput;

let start = TimeInput::Utc("2026-02-02T00:00:00Z".to_string());
let end = TimeInput::Ms(1770003600000);
```

Traversal is manual by default. If an endpoint exposes a cursor, the SDK does
not automatically walk pages for you.

WebSocket conventions are explicit:

- bars WS uses one immutable subscription per connection
- bars subscription changes require reconnect
- messages WS uses in-band `subscribe` and `unsubscribe`
- managed WS recovery is opt-in

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
mathilde-sdk-rs = { path = "." }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The client is created from typed transport config:

```rust
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::core::config::{
    AggregatorConfig, GrpcTransportConfig, HttpTransportConfig, WsTransportConfig,
};
use mathilde_sdk_rs::systems::aggregator::AggregatorClient;

let client = AggregatorClient::new(AggregatorConfig {
    http: Some(HttpTransportConfig::new("http://127.0.0.1:18182")?),
    grpc: Some(GrpcTransportConfig::new("http://127.0.0.1:18092")?),
    ws: Some(WsTransportConfig::new("ws://127.0.0.1:18182")?),
    bearer_token: Some(BearerToken::new("feed_public_token")?),
})?;
```

## Quick Start

If you want one aligned bar snapshot quickly, `latest_bars` is the smallest
place to start:

```rust
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::core::config::{AggregatorConfig, HttpTransportConfig};
use mathilde_sdk_rs::systems::aggregator::{AggregatorClient, LatestBarsRequest, LatestBarsResponse};
use mathilde_sdk_rs::systems::types::{HttpFormat, LatestMode, Timeframe};

let client = AggregatorClient::new(AggregatorConfig {
    http: Some(HttpTransportConfig::new("http://127.0.0.1:18182")?),
    grpc: None,
    ws: None,
    bearer_token: Some(BearerToken::new("feed_public_token")?),
})?;

let out = client
    .latest_bars(&LatestBarsRequest {
        pairs: "BTCUSDT,ETHUSDT".to_string(),
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        exclude_sources: None,
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    })
    .await?;

match out {
    LatestBarsResponse::Min(r) => {
        println!("rows={}", r.rows.len());
        println!("close_end_ms={}", r.close_end_ms);
    }
    LatestBarsResponse::Full(_) => unreachable!(),
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Endpoint And Transport Matrix

| Family            | HTTP                                                      | gRPC                     | WS                                  | Cursor                                      | Managed recovery                 | Notes                                  |
| ----------------- | --------------------------------------------------------- | ------------------------ | ----------------------------------- | ------------------------------------------- | -------------------------------- | -------------------------------------- |
| Docs              | `docs_system`, `docs_themes`, `docs_endpoints`, `openapi` | No                       | No                                  | No                                          | No                               | Public documentation and OpenAPI reads |
| Discovery         | `pairs_status`, `pairs_list`, `files_downloads`           | No                       | No                                  | `pairs_status` supports paging-style fields | No                               | Public pair and file discovery         |
| Latest bars       | `latest_bars`                                             | `latest_bars_grpc`       | No                                  | No                                          | No                               | Current aligned bar snapshot           |
| Range bars        | `range_bars`                                              | `range_bars_grpc`        | `connect_bars_ws`                   | Yes                                         | `connect_bars_ws_recovering`     | Windowed bars reads and bars stream    |
| Search bars       | `search_bars`                                             | `search_bars_grpc`       | No                                  | Yes                                         | No                               | Predicate-driven hit search            |
| Time-machine bars | `time_machine_bars`                                       | `time_machine_bars_grpc` | No                                  | Yes                                         | No                               | Context around hits                    |
| Bars WS helpers   | No                                                        | No                       | `connect_bars_ws_make_before_break` | N/A                                         | `connect_bars_ws_recovering`     | Immutable subscription per connection  |
| Messages WS       | No                                                        | No                       | `connect_messages_ws`               | N/A                                         | `connect_messages_ws_recovering` | In-band subscribe and unsubscribe      |

## Which Endpoint To Use

If you need the current aligned bar for one or more pairs, use `latest_bars`
or `latest_bars_grpc`.

If you need a bounded historical window on a fixed grid, use `range_bars` or
`range_bars_grpc`.

If you need to search a window for timestamps where a predicate is true, use
`search_bars` or `search_bars_grpc`.

If you already know hits, or you want context before and after hits, use
`time_machine_bars` or `time_machine_bars_grpc`.

If you want bars as a stream, use bars WS. If you need to swap subscriptions
without dropping the active stream immediately, use the make-before-break bars
helper. If you want reconnect on disconnect, use the recovering bars wrapper.

If you want predicate-triggered streaming messages with mutable subscriptions,
use messages WS. If you want reconnect plus subscription replay, use the
recovering messages wrapper.

## Examples

### Latest Bars

```rust
use mathilde_sdk_rs::systems::aggregator::{LatestBarsRequest, LatestBarsResponse};
use mathilde_sdk_rs::systems::types::{HttpFormat, LatestMode, Timeframe};

let out = client
    .latest_bars(&LatestBarsRequest {
        pairs: "BTCUSDT,ETHUSDT".to_string(),
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        exclude_sources: None,
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    })
    .await?;

match out {
    LatestBarsResponse::Min(r) => {
        println!("rows={}", r.rows.len());
        println!("close_end_ms={}", r.close_end_ms);
    }
    LatestBarsResponse::Full(_) => unreachable!(),
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Range Bars

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::{RangeBarsRequest, RangeBarsResponse};
use mathilde_sdk_rs::systems::types::{AlignMode, HttpFormat, Timeframe};

let out = client
    .range_bars(&RangeBarsRequest {
        pairs: "BTCUSDT,ETHUSDT".to_string(),
        tf: Timeframe::M1,
        align_mode: Some(AlignMode::Exact),
        close_start: Some(TimeInput::Utc("2026-02-02T00:00:00Z".to_string())),
        cursor: None,
        close_end: Some(TimeInput::Utc("2026-02-02T06:00:00Z".to_string())),
        limit: Some(1000),
        exclude_sources: None,
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    })
    .await?;

match out {
    RangeBarsResponse::Min(r) => {
        println!("rows={}", r.rows.len());
        println!("next_cursor={:?}", r.next_cursor);
    }
    RangeBarsResponse::Full(_) => unreachable!(),
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Manual cursor continuation remains explicit:

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::{RangeBarsRequest, RangeBarsResponse};
use mathilde_sdk_rs::systems::types::{AlignMode, HttpFormat, Timeframe};

let mut request = RangeBarsRequest {
    pairs: "BTCUSDT".to_string(),
    tf: Timeframe::M1,
    align_mode: Some(AlignMode::Exact),
    close_start: Some(TimeInput::Utc("2026-02-02T00:00:00Z".to_string())),
    cursor: None,
    close_end: Some(TimeInput::Utc("2026-02-02T06:00:00Z".to_string())),
    limit: Some(1000),
    exclude_sources: None,
    metadata: Some(false),
    format: Some(HttpFormat::Json),
};

let page_1 = client.range_bars(&request).await?;

let next_cursor = match &page_1 {
    RangeBarsResponse::Min(r) => r.next_cursor.clone(),
    RangeBarsResponse::Full(r) => r.next_cursor.clone(),
};

if let Some(next_cursor) = next_cursor {
    request.cursor = Some(next_cursor);
    let _page_2 = client.range_bars(&request).await?;
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Search Bars

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::{SearchBarsRequest, SearchBarsResponse};
use mathilde_sdk_rs::systems::types::{ExcludeSource, HttpFormat, Timeframe};

let out = client
    .search_bars(&SearchBarsRequest {
        tf: Timeframe::M1,
        close_start: TimeInput::Utc("2026-02-02T00:00:00Z".to_string()),
        close_end: Some(TimeInput::Utc("2026-02-02T06:00:00Z".to_string())),
        cursor: None,
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5 && ETHUSDT.v > 10".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        exclude_sources: Some(vec![ExcludeSource::NoTradeFill]),
        metadata: Some(false),
        max_hits: Some(500),
        format: Some(HttpFormat::Json),
    })
    .await?;

match out {
    SearchBarsResponse::Min(r) => {
        println!("hits={}", r.hits.len());
        println!("next_cursor={:?}", r.next_cursor);
        println!("predicate_normalized={}", r.predicate_normalized);
    }
    SearchBarsResponse::Full(_) => unreachable!(),
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Time-Machine Bars

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::{TimeMachineBarsRequest, TimeMachineBarsResponse};
use mathilde_sdk_rs::systems::types::{ExcludeSource, HttpFormat, Timeframe};

let out = client
    .time_machine_bars(&TimeMachineBarsRequest {
        tf: Timeframe::M1,
        close_start: TimeInput::Utc("2026-02-02T00:00:00Z".to_string()),
        close_end: Some(TimeInput::Utc("2026-02-02T02:00:00Z".to_string())),
        cursor: None,
        predicate: Some("BTCUSDT.c > ETHUSDT.c * 1.5".to_string()),
        hits: None,
        output_pairs: Some(vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]),
        exclude_sources: Some(vec![ExcludeSource::NoTradeFill]),
        metadata: Some(false),
        before_bars: Some(10),
        after_bars: Some(10),
        max_hits: Some(100),
        overlap_mode: Some("merge".to_string()),
        format: Some(HttpFormat::Json),
    })
    .await?;

match out {
    TimeMachineBarsResponse::Min(r) => {
        println!("rows={}", r.rows.len());
        println!("next_cursor={:?}", r.next_cursor);
    }
    TimeMachineBarsResponse::Full(_) => unreachable!(),
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Bars WS

```rust
use mathilde_sdk_rs::systems::aggregator::{
    BarsWsFormat, BarsWsInboundFrame, BarsWsSubscribeRequest,
};
use mathilde_sdk_rs::systems::types::Timeframe;

let request = BarsWsSubscribeRequest {
    pairs: "BTCUSDT,ETHUSDT".to_string(),
    tfs: vec![Timeframe::M1],
    metadata: Some(false),
    from_close: None,
    last_n_bars: Some(100),
    format: Some(BarsWsFormat::Json),
};

let mut stream = client.connect_bars_ws(&request).await?;

while let Some(frame) = stream.next_frame(&request).await? {
    match frame {
        BarsWsInboundFrame::Meta(meta) => {
            println!("phase={:?} close_ms={:?}", meta.phase, meta.close_ms);
        }
        BarsWsInboundFrame::JsonRowsMin(rows) => {
            println!("rows={}", rows.len());
            break;
        }
        BarsWsInboundFrame::Error(err) => {
            println!("ws error: {} {}", err.kind, err.error);
            break;
        }
        _ => {}
    }
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

If you need reconnect on disconnect, use the recovering wrapper explicitly:

```rust
use std::time::Duration;
use mathilde_sdk_rs::streaming::subscription::ExponentialBackoffConfig;
use mathilde_sdk_rs::systems::aggregator::BarsWsSubscribeRequest;
use mathilde_sdk_rs::systems::types::Timeframe;

let request = BarsWsSubscribeRequest {
    pairs: "BTCUSDT".to_string(),
    tfs: vec![Timeframe::M1],
    metadata: Some(false),
    from_close: None,
    last_n_bars: Some(1),
    format: None,
};

let _stream = client
    .connect_bars_ws_recovering(
        &request,
        ExponentialBackoffConfig {
            initial_delay: Duration::from_millis(250),
            multiplier: 2,
            max_delay: Duration::from_secs(10),
            max_attempts: None,
            jitter_ratio: 0.2,
        },
    )
    .await?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Messages WS

```rust
use mathilde_sdk_rs::systems::aggregator::{
    MessagesWsServerFrame, MessagesWsSubscribeFrame, MessagesWsUnsubscribeFrame,
};
use mathilde_sdk_rs::systems::types::Timeframe;

let mut stream = client.connect_messages_ws().await?;

stream
    .send_subscribe(&MessagesWsSubscribeFrame {
        id: "rule_1".to_string(),
        tfs: Some(vec![Timeframe::M1]),
        predicate: "BTCUSDT.c > 0".to_string(),
        message: "rule triggered".to_string(),
        payload: Some(serde_json::json!({"strategy":"alpha"})),
    })
    .await?;

if let Some(frame) = stream.next_frame().await? {
    match frame {
        MessagesWsServerFrame::Subscribed(frame) => {
            println!("subscribed={}", frame.id);
        }
        MessagesWsServerFrame::Message(frame) => {
            println!("message={} close_ms={}", frame.message, frame.close_ms);
        }
        MessagesWsServerFrame::Heartbeat(frame) => {
            println!("heartbeat at_ms={}", frame.at_ms);
        }
        MessagesWsServerFrame::Error(frame) => {
            println!("error={} {}", frame.kind, frame.error);
        }
    }
}

stream
    .send_unsubscribe(&MessagesWsUnsubscribeFrame {
        id: "rule_1".to_string(),
    })
    .await?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

If you need reconnect plus replay of active subscriptions, use the recovering
wrapper explicitly:

```rust
use std::time::Duration;
use mathilde_sdk_rs::streaming::subscription::ExponentialBackoffConfig;

let _stream = client
    .connect_messages_ws_recovering(ExponentialBackoffConfig {
        initial_delay: Duration::from_millis(250),
        multiplier: 2,
        max_delay: Duration::from_secs(10),
        max_attempts: None,
        jitter_ratio: 0.2,
    })
    .await?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Transport Notes

HTTP covers the public HTTP contract. For bars endpoints, that includes JSON
and protobuf where the upstream public contract exposes both.

gRPC is transport-equivalent to the bars HTTP families that exist today. The
caller-facing request shape stays aligned with HTTP where possible, minus the
HTTP-only `format` field.

WS is not one generic stream surface. Bars and messages are intentionally
different:

- bars: immutable subscription per connection
- messages: in-band subscribe and unsubscribe

## WS Recovery And Limits

WS recovery is opt-in. The raw WS connections remain thin and do not hide
reconnect policy.

Managed bars recovery reconnects with the same subscribe request using bounded
exponential backoff. It is proved locally and live at the current scope, but
it is reconnect-only today. This does not mean gap-free continuity after a
disconnect.

Managed messages recovery reconnects and replays active subscribe state. It is
proved locally and live at the current scope for reconnect plus subscribe-state
replay.

The default shared backoff policy currently implemented is:

- initial delay: `250ms`
- multiplier: `2x`
- max delay: `10s`
- max attempts: `None`
- jitter ratio: `0.2`

## What Not To Infer

Do not infer from this README that:

- the SDK exposes private or admin surfaces
- traversal is automatic
- bars WS recovery is gap-free
- the SDK hides all transport policy by default
- local time parsing is supported for time inputs
- every MATHILDE subsystem is already bound here

Also note one current construction limit from code: `AggregatorClient::new(...)`
currently requires HTTP config even if a caller plans to use only gRPC or WS.
That is current behavior, not a wider transport design claim.

## Further Reading

- [Global inventory](inventory.md)
- [Core inventory](src/core/docs/inventory.md)
- [Transport inventory](src/transport/docs/inventory.md)
- [Streaming inventory](src/streaming/docs/inventory.md)
- [Systems inventory](src/systems/docs/inventory.md)
- [Tests inventory](src/tests/docs/inventory.md)
- [Cross-endpoint consistency spec](.dev/specs/SDK_CROSS_ENDPOINT_CONSISTENCY_SPEC_2026-04-08.md)
- [Shared WS recovery spec](.dev/specs/SDK_SHARED_WS_RECOVERY_SPEC_2026-04-08.md)
