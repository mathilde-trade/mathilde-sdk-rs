# mathilde-sdk-rs

`mathilde-sdk-rs` is the public Rust SDK for MATHILDE feed access. It binds
public HTTP, gRPC, and WebSocket contracts faithfully, keeps request and
transport behavior explicit, and does not hide unproved convenience semantics.
MATHILDE measures, not predicts, and this SDK is a public client-contract
layer, not an opinion or trading layer.

> This SDK is the official public contract binding. It prioritizes semantic fidelity and explicit behavior over convenience abstraction. Users who want higher-level ergonomic or opinionated workflows are free to build additional wrappers on top.

## Index

- [What This Is](#what-this-is)
- [What This Is Not](#what-this-is-not)
- [Supported Public Surfaces](#supported-public-surfaces)
- [Core Conventions](#core-conventions)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Endpoint And Transport Matrix](#endpoint-and-transport-matrix)
- [Which Endpoint To Use](#which-endpoint-to-use)
- [Examples](#examples)
- [Transport Notes](#transport-notes)
- [WS Recovery And Limits](#ws-recovery-and-limits)
- [What Not To Infer](#what-not-to-infer)
- [Further Reading](#further-reading)

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

### Docs Pages

What it is:
Public authored documentation reads for the current public `aggregator`
surface.

When to use it:
Use docs pages when the question is about subsystem explanation, theme-level
interpretation, or endpoint-family overview before integration. The current
public docs surfaces are:

- `docs_system` for the canonical public system document
- `docs_summary` for the short public subsystem entry summary
- `docs_themes` for the compiled public themes corpus
- `docs_endpoints` for the public machine-first endpoint selection guide

When not to use it:
Do not use docs pages as the exact schema authority when you need the wire
contract itself.

### OpenAPI

What it is:
The public OpenAPI document for the current HTTP surface.

When to use it:
Use OpenAPI when the question is about the exact route, body, parameter, and
response schema contract. This is the schema authority for the public HTTP
surface, not just a convenience document mirror.

When not to use it:
Do not treat it as the main conceptual explanation of why a surface exists or
which question shape it answers.

### Pair Status

What it is:
The public pair-readiness and pair-state discovery surface with nested status
blocks.

When to use it:
Use pair status when the question is which pairs are available, ready, or
currently visible through the public feed state. This is the richer pair-state
surface when you need runtime state, history, frontier, counts, readiness, or
coverage rather than just names.

When not to use it:
Do not use pair status as a historical bars read or as a substitute for pair
list enumeration when you only need names.

### Pair List

What it is:
A lightweight public pair catalogue surface.

When to use it:
Use pair list when the question is simply which public pairs exist for the
current surface and you want a lighter paginated catalogue rather than the
full nested pair-state view.

When not to use it:
Do not use pair list when you need richer pair-state details or bars data.

### File Downloads

What it is:
The public batch download request surface for exported parquet files across one
or more pairs and timeframes.

When to use it:
Use file downloads when the task is export-oriented retrieval and you want one
unified flat list of signed download URLs instead of calling internal file
discovery and URL endpoints separately. The public surface here is
`files_downloads`, which returns signed URLs and `expires_at_utc`, not file
bytes. The SDK also exposes `files_download_items` as an explicit convenience
layer that downloads selected returned rows to local parquet files with bearer
auth.

When not to use it:
Do not use file downloads as a substitute for direct bars querying or pair
state inspection. Do not assume the full internal files family is public
through this SDK surface.

### Latest

What it is:
The current stable closed snapshot for one or more pairs on one timeframe.

When to use it:
Use `latest` when the question is "where is the stable edge now?" and you want
the newest aligned closed read the public surface is prepared to serve.

When not to use it:
Do not use it for bounded history, predicate-first discovery, or local context
around hits.

### Range

What it is:
A bounded historical interval of closed bars on a fixed grid.

When to use it:
Use `range` when you need a reproducible historical window, backfill, or
paged historical extraction.

When not to use it:
Do not use it when the real question is "when did this condition become true?"
or "what happened around these hits?"

### Search

What it is:
A predicate-first discovery surface over stable historical bars.

When to use it:
Use `search` when the main question is "at which closes did this condition
become true?"

When not to use it:
Do not use it as a full history dump or as a context-window replay surface.

### Time machine

What it is:
A context surface that returns bars before and after selected hit points.

When to use it:
Use `time-machine` when you already know hit timestamps, or when you want the
system to find hits and then return nearby context.

When not to use it:
Do not use it as a general replacement for bounded range reads.

### Bars WS

What it is:
A live bars stream with one immutable subscription per connection.

When to use it:
Use bars WS when the question is live and continuous and the subscription set
can stay fixed for the life of the socket.

When not to use it:
Do not use it if you need in-band subscribe and unsubscribe changes on one
connection.

### Messages WS

What it is:
A live predicate-triggered messages stream with mutable subscriptions.

When to use it:
Use messages WS when you want connection-local rules that emit live message
events when their predicates evaluate true.

When not to use it:
Do not use it as a bars stream or as a historical replay surface.

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

For pair-set request fields, the SDK also exposes one small shared collector:

```rust
use mathilde_sdk_rs::systems::helpers::pairs;
```

Use it when you want to construct `Vec<String>` pair sets without spelling
`.to_string()` on every element yourself. It is only a collector. It does not
parse CSV, trim whitespace, deduplicate, or validate emptiness.

Both forms are valid at the public request boundary:

- helper form for shorter call sites
- direct `Vec<String>` construction when you want the shape to stay fully explicit

Helper form:

```rust
pairs: pairs(["BTCUSDT", "ETHUSDT"])
```

Direct vector form:

```rust
pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]
```

Mixed UTC-string and millisecond inputs use `TimeInput`:

```rust
use mathilde_sdk_rs::core::time::TimeInput;

let start = TimeInput::Utc("2026-02-02T00:00:00Z".to_string());
let end = TimeInput::Ms(1770003600000);
```

Traversal is explicit and additive. The one-page methods stay unchanged, and
cursor-aware endpoint families also expose call wrappers with explicit
continuation helpers:

- one page: `client.range_bars(&request).await?`
- one page through wrapper: `client.range_bars_call(request.clone()).send().await?`
- full traversal: `client.range_bars_call(request).traverse().await?`
- manual paging: `client.range_bars_call(request).pager()?.next().await?`

The same explicit call-wrapper pattern exists for `search_bars` and
`time_machine_bars`, including their gRPC equivalents.

Range traversal may freeze an omitted `close_end` from the first page. Search
and time-machine traversal or pager use require explicit `close_end`.

WebSocket conventions are explicit:

- bars WS uses one immutable subscription per connection
- bars subscription changes require reconnect
- messages WS uses in-band `subscribe` and `unsubscribe`
- managed WS recovery is opt-in

Predicate-based surfaces are also explicit.

A predicate is a boolean expression evaluated on stable bars. It lets you ask
whether a condition is true at a given close instead of asking for all rows in
a window first and filtering them yourself.

The current public predicate language supports both compact and long field
aliases for the core bar fields. Code-read evidence from the upstream parser
proves aliases such as:

- `.o` or `.open`
- `.h` or `.high`
- `.l` or `.low`
- `.c` or `.close`
- `.v` or `.volume`
- `.n` or `.trades`
- `.quote_v` or `.quote_volume`
- `.vw` or `.vwap`

The same parser also proves additional derived and metadata-backed field names,
including:

- `coverage_ratio`
- `covered_1m_count`
- `expected_1m_count`
- `inputs_source_counts_frontier`
- `inputs_source_counts_api`
- `inputs_source_counts_synthetic`
- `inputs_source_counts_fix_data`
- `frontier_5s_inputs_coverage_ratio`
- `frontier_5s_expected`
- `frontier_5s_synth_n`
- `frontier_5s_synth_ratio`
- `frontier_5s_trade_n`
- `frontier_5s_trade_ratio`
- `harmonized_at_ms`
- `source`
- `process`
- `venues_expected`
- `venues_with_trades`

Not every predicate surface allows every proved field. In particular,
`connect_messages_ws` uses the same shared predicate language, but its runtime
policy remains restricted to websocket-supported numeric bar fields and rejects
metadata-backed fields.

Typical examples are:

```text
BTCUSDT.close > BTCUSDT.open
BTCUSDT.close > ETHUSDT.close * 1.02
BTCUSDT.high > BTCUSDT.low && BTCUSDT.volume >= 100
BTCUSDT.coverage_ratio >= 0.99 && BTCUSDT.expected_1m_count >= 60
has(BTCUSDT.venues_expected, "binance")
(BTCUSDT.close > BTCUSDT.open) && (ETHUSDT.close > ETHUSDT.open) && BTCUSDT.quote_volume >= 1000
```

Predicates are currently used by these public surfaces:

- `search_bars`
- `search_bars_grpc`
- `time_machine_bars` in predicate mode
- `time_machine_bars_grpc` in predicate mode
- `connect_messages_ws`
- `connect_messages_ws_recovering`

Use predicate surfaces when the primary question is about events or conditions.
Do not use them when the primary question is simply "show me the full bounded
history."

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
mathilde-sdk-rs = { path = "." }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The smallest public-default construction path is:

```rust
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::AggregatorClient;

let client =
    AggregatorClient::mathilde_public_default(Some(BearerToken::new("feed_public_token")?))?;
```

If you need explicit transport overrides, the typed config path remains
available:

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
use mathilde_sdk_rs::systems::aggregator::{AggregatorClient, LatestBarsRequest, LatestBarsResponse};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::types::{HttpFormat, LatestMode, Timeframe};

let client =
    AggregatorClient::mathilde_public_default(Some(BearerToken::new("feed_public_token")?))?;

let out = client
    .latest_bars(&LatestBarsRequest {
        pairs: pairs(["BTCUSDT", "ETHUSDT"]),
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
| Docs              | `docs_system`, `docs_summary`, `docs_themes`, `docs_endpoints`, `openapi` | No                       | No                                  | No                                          | No                               | Public documentation and OpenAPI reads |
| Discovery         | `pairs_status`, `pairs_list`, `files_downloads`           | No                       | No                                  | `pairs_status` supports paging-style fields | No                               | Public pair and file discovery         |
| Latest bars       | `latest_bars`                                             | `latest_bars_grpc`       | No                                  | No                                          | No                               | Current aligned bar snapshot           |
| Range bars        | `range_bars`                                              | `range_bars_grpc`        | `connect_bars_ws`                   | Yes                                         | `connect_bars_ws_recovering`     | Windowed bars reads and bars stream    |
| Search bars       | `search_bars`                                             | `search_bars_grpc`       | No                                  | Yes                                         | No                               | Predicate-driven hit search            |
| Time-machine bars | `time_machine_bars`                                       | `time_machine_bars_grpc` | No                                  | Yes                                         | No                               | Context around hits                    |
| Bars WS helpers   | No                                                        | No                       | `connect_bars_ws_make_before_break` | N/A                                         | `connect_bars_ws_recovering`     | Immutable subscription per connection  |
| Messages WS       | No                                                        | No                       | `connect_messages_ws`               | N/A                                         | `connect_messages_ws_recovering` | In-band subscribe and unsubscribe      |

## Which Endpoint To Use

### `latest`

This shape answers the question: what is the current stable closed snapshot for
these pairs and this timeframe?

Use it when you need the newest aligned read point that the public surface is
prepared to serve as stable truth.

Do not use it when you need a historical interval, predicate hit discovery, or
context around hits.

Current aggregator binding:

- `latest_bars`
- `latest_bars_grpc`

What not to infer:

- this is not a streaming surface
- this is not a generic "newest thing written" fetch
- this is not a substitute for historical range extraction

### `range`

This shape answers the question: what are the closed bars for this bounded
historical interval?

Use it when you need a concrete time window on a fixed grid, reproducible
backfill, or paged historical extraction.

Do not use it when your real question is "when did this become true?" or "what
was the local context around those hits?"

Current aggregator binding:

- `range_bars`
- `range_bars_grpc`

What not to infer:

- traversal is explicit, not automatic
- a cursor is paging state, not a different query family
- range is not a search surface

### `search`

This shape answers the question: at which stable closes did this condition
become true?

Use it when you need event discovery across a historical window and the
predicate itself is the primary question.

Here the predicate is the contract center. You provide a boolean condition on
stable bars, and the surface tells you which closes satisfied it.

Typical predicate examples are:

- `BTCUSDT.close > BTCUSDT.open`
- `BTCUSDT.close > ETHUSDT.close * 1.02`
- `BTCUSDT.coverage_ratio >= 0.99 && BTCUSDT.expected_1m_count >= 60`

Do not use it when you need the full bounded dataset or when you already know
the hit points and only want nearby context.

Current aggregator binding:

- `search_bars`
- `search_bars_grpc`
- `connect_messages_ws` for streaming predicate-triggered messages rather than
  historical hit search

What not to infer:

- search is not a full history dump
- search answers "when did this happen?", not "show me all bars"
- min and full views may not imply byte-identical cursor encoding

### `time-machine`

This shape answers the question: what did the local context look like around
those hit points?

Use it when you already have hit timestamps, or when you want the system to
find hits and then return bars before and after those moments.

This family supports two modes:

- hits mode: you provide explicit hit timestamps
- predicate mode: you provide a predicate and let the system discover hits
  before returning context around them

That means time-machine can answer both:

- "show me context around these known hit points"
- "find where this became true, then show me the nearby bars"

Do not use it as a general replacement for bounded range reads.

Current aggregator binding:

- `time_machine_bars`
- `time_machine_bars_grpc`

What not to infer:

- this is a context surface, not a general-purpose history surface
- it does not imply automatic traversal
- it is not the first-pass event-discovery surface when `search` already fits

### Bars WS

This shape answers the question: how do I consume live bars for one fixed
subscription set?

Use it when you need a live bars stream and are willing to treat the
subscription as immutable for the lifetime of the socket.

Do not use it if you need in-band subscribe and unsubscribe changes on the same
connection.

Current aggregator binding:

- `connect_bars_ws`
- `connect_bars_ws_make_before_break`
- `connect_bars_ws_recovering`

What not to infer:

- bars WS does not support in-band unsubscribe
- changing the subscription set requires reconnect
- managed recovery does not currently prove gap-free continuity

### Messages WS

This shape answers the question: how do I receive live predicate-triggered
messages from mutable subscriptions?

Use it when you want rule-based live notifications and connection-local
subscribe and unsubscribe control.

Here the predicate is a live rule. Instead of returning historical hits, the
stream emits subscribed acknowledgements, live message frames, heartbeats, and
errors for the active subscription set.

Do not use it as a bars stream or as a historical replay surface.

Current aggregator binding:

- `connect_messages_ws`
- `connect_messages_ws_recovering`

What not to infer:

- messages WS is not the same model as bars WS
- replay/backfill is not part of this surface
- subscribe state is connection-local and must be re-established after
  reconnect

## Examples

### Latest Bars

This example answers the question: what is the current stable closed snapshot
for these pairs?

```rust
use mathilde_sdk_rs::systems::aggregator::{LatestBarsRequest, LatestBarsResponse};
use mathilde_sdk_rs::systems::types::{HttpFormat, LatestMode, Timeframe};

let out = client
    .latest_bars(&LatestBarsRequest {
        pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
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

This example answers the question: what are the closed bars for this bounded
historical interval?

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::{RangeBarsRequest, RangeBarsResponse};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::types::{AlignMode, HttpFormat, Timeframe};

let out = client
    .range_bars(&RangeBarsRequest {
        pairs: pairs(["BTCUSDT", "ETHUSDT"]),
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

Full traversal is explicit when you want the whole bounded range:

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::RangeBarsRequest;
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::types::{AlignMode, HttpFormat, Timeframe};

let request = RangeBarsRequest {
    pairs: pairs(["BTCUSDT"]),
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

let out = client.range_bars_call(request).traverse().await?;
println!("pages_fetched={}", out.pages_fetched);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Manual continuation is also explicit through the pager:

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::{RangeBarsRequest, RangeBarsResponse};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::types::{AlignMode, HttpFormat, Timeframe};

let request = RangeBarsRequest {
    pairs: pairs(["BTCUSDT"]),
    tf: Timeframe::M1,
    align_mode: Some(AlignMode::Exact),
    close_start: Some(TimeInput::Utc("2026-02-02T00:00:00Z".to_string())),
    cursor: None,
    close_end: None,
    limit: Some(1000),
    exclude_sources: None,
    metadata: Some(false),
    format: Some(HttpFormat::Json),
};

let mut pager = client.range_bars_call(request).pager()?;

while let Some(page) = pager.next().await? {
    match page {
        RangeBarsResponse::Min(r) => println!("page rows={}", r.rows.len()),
        RangeBarsResponse::Full(r) => println!("page rows={}", r.rows.len()),
    }
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Search Bars

This example answers the question: at which stable closes did this predicate
become true?

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

If you want full traversal across every search page, keep `close_end`
explicit and use:

```rust
let out = client.search_bars_call(request).traverse().await?;
println!("pages_fetched={}", out.pages_fetched);
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Time-Machine Bars

This example answers the question: what did the local bars context look like
around these hits?

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

If you want full traversal across every time-machine page, keep `close_end`
explicit and use:

```rust
let out = client.time_machine_bars_call(request).traverse().await?;
println!("pages_fetched={}", out.pages_fetched);
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Bars WS

This example answers the question: how do I consume live bars for one fixed
subscription set?

```rust
use mathilde_sdk_rs::systems::aggregator::{
    BarsWsFormat, BarsWsInboundFrame, BarsWsSubscribeRequest,
};
use mathilde_sdk_rs::systems::types::Timeframe;

let request = BarsWsSubscribeRequest {
    pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
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
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::types::Timeframe;

let request = BarsWsSubscribeRequest {
    pairs: pairs(["BTCUSDT"]),
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

This example answers the question: how do I receive live predicate-triggered
messages from one mutable subscription set?

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
