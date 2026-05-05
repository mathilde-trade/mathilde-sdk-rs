# mathilde-sdk-rs

`mathilde-sdk-rs` is the public Rust SDK for the public MATHILDE surface. It binds public endpoints only, stays thin and contract-faithful across HTTP, gRPC, and WebSocket, and keeps request, traversal, and recovery behavior explicit. MATHILDE measures, not predicts, and this SDK is a client-contract layer rather than a trading or opinion layer.

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
- [Live examples and workflows](examples/README.md)

## What This Is

This SDK is a thin public client-contract layer for MATHILDE systems with a proved public surface. Today that means `intro`, `aggregator`, `primitives`, and `regime`.

The current public scope exposes typed entrypoints for:

- the top-level intro document on `api.mathilde.dev`
- public docs and OpenAPI discovery
- public pair-state and pair-list discovery
- public file-download issuance and explicit local materialization
- bars query and streaming surfaces for `aggregator`
- outputs query and streaming surfaces for `primitives` and `regime`
- shared time, traversal, and recovery conventions where the public contract proves them

The SDK is transport-aware, but endpoint-faithful. It keeps the public query shapes visible instead of flattening them into one generic read abstraction.

## What This Is Not

This SDK is not a trading framework, not a prediction layer, not a private or admin client, and not a wrapper that hides transport policy behind unproved convenience.

In particular, it does not claim:

- hidden traversal by default
- hidden retry or recovery by default
- private-system coverage because a public feed exists
- local-time guessing for time inputs
- local predicate-language ownership beyond the proven transport surface

If a behavior is not proved by current code, generated contract artifacts, specs, or tests, this README does not claim it.

## Supported Public Surfaces

The SDK currently binds four public system surfaces:

- `Intro`
- `Aggregator`
- `Primitives`
- `Regime`

Those systems share a common navigation pattern:

- `Intro` for the top-level public MATHILDE map
- docs endpoints for system-specific understanding
- OpenAPI for the exact HTTP wire contract
- query or streaming shapes matched to the concrete question

### Intro

**What it is:**
The public host-root introduction document for the wider MATHILDE platform. It is the top-level navigation starting point for humans and LLMs moving across the public documentation space.

**When to use it:**
Appropriate when the question is which public system exists here, where its public documentation lives, and which public surface should be inspected next.

**When not to use it:**
Not the schema authority for subsystem routes, request shapes, or system-specific invariants. Those belong to each subsystem's docs and OpenAPI surfaces.

### Docs Pages

**What it is:**
Public authored documentation reads for the current public system surface. These are the first system-specific entrypoint when the question is conceptual understanding rather than wire format.

**When to use it:**
Appropriate when the question is what a system does, how its endpoint families are organized, and which public concepts matter before integration.

**Current docs surfaces:**

- `aggregator`: `docs_system`, `docs_summary`, `docs_themes`, `docs_endpoints`
- `primitives`: `docs_system`, `docs_summary`, `docs_taxonomy`, `docs_registry`, `docs_endpoints`
- `regime`: `docs_system`, `docs_summary`, `docs_taxonomy`, `docs_registry`, `docs_endpoints`

For `primitives` and `regime`, taxonomy and registry are also the public authority for output-definition discovery and selector filtering. Full family/group descriptions belong there, not duplicated in this README.

**When not to use it:**
Not the exact schema authority when the wire contract itself is the question.

### OpenAPI

**What it is:**
The public OpenAPI document for the current HTTP surface.

**When to use it:**
Appropriate when the question is the exact route, body, parameter, or response schema contract. This is the schema authority for the public HTTP surface, not just a convenience document mirror.

**When not to use it:**
Not the main conceptual explanation of why a surface exists or which question shape it answers.

### Pair Status

**What it is:**
The richer public pair-readiness and pair-state discovery surface.

**When to use it:**
Appropriate when the question is which pairs are available, ready, or currently visible through the public feed state, especially when runtime state, history, frontier, counts, readiness, or coverage matter more than names alone.

**When not to use it:**
Not a historical bars read and not a substitute for pair-list enumeration when only names are needed.

### Pair List

**What it is:**
A lighter public pair catalogue surface.

**When to use it:**
Appropriate when the question is simply which public pairs exist for the current surface and a lighter paginated catalogue is preferable to the full nested pair-state view.

**When not to use it:**
Not appropriate when richer pair-state details or bars data are required.

### File Downloads

**What it is:**
The public export-oriented file flow for parquet windows across one or more pairs and timeframes.

**When to use it:**
Appropriate when the task is export retrieval rather than query-shaped reads. The public flow is explicit:

- `files_downloads` issues rows with signed URLs and `expires_at_utc`
- `files_download_items` optionally materializes selected returned rows to local parquet files with bearer auth

**When not to use it:**
Not a substitute for query-shaped reads, pair-state inspection, or hidden internal file-management endpoints.

### `latest`

**What it is:**
The current stable closed snapshot question.

**When to use it:**
Appropriate when the question is "where is the stable public edge now?" and the newest aligned closed read is the required output.

**When not to use it:**
Not appropriate for bounded history, predicate-first discovery, or local context around hits.

### `range`

**What it is:**
The bounded historical slice question.

**When to use it:**
Appropriate for reproducible historical windows, backfill, or paged historical extraction.

**When not to use it:**
Not the correct surface when the primary question is "when did this condition become true?" or "what happened around these hits?"

### `search`

**What it is:**
The event-discovery question over stable historical closes.

**When to use it:**
Appropriate when the main question is "at which closes did this condition become true?" and hits are the required output rather than a full bounded slice first.

**When not to use it:**
Not a full history dump and not a context-window replay surface.

### `time_machine`

**What it is:**
The hit-context question.

**When to use it:**
Appropriate when hit timestamps are already known, or when the system should find hits and then return nearby context.

**When not to use it:**
Not a general replacement for bounded range reads.

### Bars WS

**What it is:**
The live bars streaming shape for `aggregator`.

**When to use it:**
Appropriate when the question is live and continuous and the subscription set can stay fixed for the life of the socket.

**When not to use it:**
Not appropriate when in-band subscribe and unsubscribe changes are required on one connection.

### Outputs WS

**What it is:**
The live outputs streaming shape for `primitives` and `regime`.

**When to use it:**
Appropriate when the question is live computed-state delivery and the subscription set can stay fixed for the life of the socket. This is the output analogue of bars WS, with make-before-break and opt-in recovering wrappers in the current SDK surface.

**When not to use it:**
Not appropriate for in-band subscription churn on one connection, historical range extraction, or hidden gap-free continuity claims.

### Messages WS

**What it is:**
A live predicate-triggered messages stream with mutable subscriptions.

**When to use it:**
Appropriate for connection-local rules that emit live message events when their predicates evaluate true.

**When not to use it:**
Not a bars stream, not an outputs stream, and not a historical replay surface.

## Core Conventions

The SDK keeps a small set of shared public conventions explicit.

Method naming stays query-shaped and transport-aware:

- system facades expose `latest`, `range`, `search`, and `time_machine`
- transport-equivalent gRPC methods append `_grpc`

HTTP and gRPC request shapes stay aligned where the public contract allows it. The main default difference is that HTTP request structs may expose `format` while gRPC does not.

Pair collections stay typed at the SDK boundary. When a public HTTP endpoint uses CSV on the wire, the SDK performs that normalization explicitly rather than exposing raw CSV as the main caller-facing shape.

Mixed millisecond and UTC-string time inputs use `TimeInput`. The current proved accepted forms are:

- UTC milliseconds
- RFC3339 strings with trailing `Z`
- compact UTC strings in `YYYY-MM-DD:HH:MM[:SS]`

Offset forms and ambiguous local-time strings are rejected rather than guessed.

Traversal is explicit and additive. One-page methods stay unchanged, and the cursor-aware families add call wrappers and pagers on top. Search and `time_machine` traversal require explicit `close_end`. Range traversal can freeze an omitted `close_end` from the first page rather than pretending the window was open-ended and stable by default.

Traversal choice stays explicit:

- `traverse()` when all fetched pages should be materialized together
- `pager()` when page-by-page consumption is preferred and lower memory pressure matters
- `search` and `time_machine` paging/traversal require explicit `close_end`
- larger windows should keep `close_end` explicit and keep `limit` or `max_hits` bounded sensibly
- `traverse()` is implemented as `pager()` plus page collection, so its memory cost grows with the full fetched page set

WebSocket behavior is also explicit:

- bars WS uses one immutable subscription per connection
- outputs WS uses one immutable subscription per connection
- bars or outputs subscription changes require reconnect or make-before-break
- messages WS uses in-band `subscribe` and `unsubscribe`
- managed reconnect behavior is opt-in through the recovering wrappers

Predicate surfaces are transport-visible, not locally redefined by the SDK. The SDK accepts predicate strings on the public request surface and returns normalized predicate strings where the public response or frame contract proves them. This README does not claim local parser ownership for the full predicate grammar.

For output systems, selector meaning is delegated to public taxonomy and registry surfaces. The README explains how selectors participate in the shared mechanics, but full family/group descriptions remain outside this file.

## Installation

Crate entry for `Cargo.toml`:

```toml
[dependencies]
mathilde-sdk-rs = { path = "." }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The smallest public-default construction path is:

```rust
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::Aggregator;

let client = Aggregator::client(Some(BearerToken::new("feed_public_token")?))?;
```

The equivalent public intro-root construction path is:

```rust
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::intro::Intro;

let client = Intro::client(Some(BearerToken::new("feed_public_token")?))?;
```

The equivalent checked-in public-default construction for computed outputs is:

```rust
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::primitives::Primitives;

let client = Primitives::client(Some(BearerToken::new("feed_public_token")?))?;
```

The typed config path remains available for explicit transport overrides:

```rust
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::core::config::{
    AggregatorConfig, GrpcTransportConfig, HttpTransportConfig, WsTransportConfig,
};
use mathilde_sdk_rs::systems::aggregator::Aggregator;

let client = Aggregator::new(AggregatorConfig {
    http: HttpTransportConfig::new("http://127.0.0.1:18182")?,
    grpc: Some(GrpcTransportConfig::new("http://127.0.0.1:18092")?),
    ws: Some(WsTransportConfig::new("ws://127.0.0.1:18182")?),
    bearer_token: Some(BearerToken::new("feed_public_token")?),
})?;
```

## Quick Start

A minimal aligned bar snapshot can start with `latest`:

```rust
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::aggregator::{Aggregator, LatestRequest};
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::types::{HttpFormat, LatestMode, Timeframe};

let client = Aggregator::client(Some(BearerToken::new("feed_public_token")?))?;

let out = client
    .latest(&LatestRequest {
        pairs: pairs(["BTCUSDT", "ETHUSDT"]),
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    })
    .await?;

println!("rows={}", out.rows.len());
println!("close_end_ms={}", out.close_end_ms);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Selector-driven computed outputs use the same client shape with a different request and payload surface:

```rust
use mathilde_sdk_rs::core::auth::BearerToken;
use mathilde_sdk_rs::systems::primitives::{
    LatestRequest, Primitives, ProcessorFamily, ProcessorGroup,
};
use mathilde_sdk_rs::systems::types::{HttpFormat, LatestMode, Timeframe};

let client = Primitives::client(Some(BearerToken::new("feed_public_token")?))?;

let out = client
    .latest(&LatestRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: Some(vec![ProcessorFamily::MovingAverages]),
        group: Some(vec![ProcessorGroup::Ema]),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Json),
    })
    .await?;

println!("pair={}", out.rows[0].row.pair);
if let Some(value) = out.rows[0].row.computed.f64("ma_ema_p20") {
    println!("ma_ema_p20={value}");
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Endpoint And Transport Matrix

This matrix is mechanism-first. Each row names the question shape or discovery surface first, then shows which current system bindings implement it.

| Shape          | Current system bindings                                   | HTTP                                                                                                                         | gRPC                | WS                                                           | Cursor              | Managed recovery                 | Important limit                                                                                                               |
| -------------- | --------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- | ------------------- | ------------------------------------------------------------ | ------------------- | -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| Intro          | `Intro`                                                   | `intro`                                                                                                                      | No                  | No                                                           | No                  | No                               | Host-root public intro document; not subsystem schema authority                                                               |
| Docs pages     | `Aggregator`, `Primitives`, `Regime`                      | `docs_system`, `docs_summary`, plus system-specific `docs_themes` or `docs_taxonomy` / `docs_registry`, and `docs_endpoints` | No                  | No                                                           | No                  | No                               | Docs for conceptual system understanding; OpenAPI for the exact wire contract                                                 |
| OpenAPI        | `Aggregator`, `Primitives`, `Regime`                      | `openapi`                                                                                                                    | No                  | No                                                           | No                  | No                               | HTTP schema authority only                                                                                                    |
| Pair status    | `Aggregator`, `Primitives`, `Regime`                      | `pairs_status`                                                                                                               | No                  | No                                                           | Paging-style fields | No                               | Richer pair-state surface, not bars or outputs history                                                                        |
| Pair list      | `Aggregator`, `Primitives`, `Regime`                      | `pairs_list`                                                                                                                 | No                  | No                                                           | Paging-style fields | No                               | Lighter catalogue surface than pair status                                                                                    |
| File downloads | `Aggregator`, `Primitives`, `Regime`                      | `files_downloads`                                                                                                            | No                  | No                                                           | No                  | No                               | Two-step export flow; `files_download_items` is the explicit local materialization helper, not a separate public query family |
| `latest`       | `Aggregator` bars, `Primitives` outputs, `Regime` outputs | `latest`                                                                                                                     | `latest_grpc`       | No                                                           | No                  | No                               | `Regime` is `tf=1h` only; projected output protobuf/gRPC paths fail closed when the request is projected                      |
| `range`        | `Aggregator` bars, `Primitives` outputs, `Regime` outputs | `range`                                                                                                                      | `range_grpc`        | No                                                           | Yes                 | No                               | Traversal is explicit; omitted `close_end` may be frozen from the first page; `Regime` is `tf=1h` only                        |
| `search`       | `Aggregator` bars, `Primitives` outputs, `Regime` outputs | `search`                                                                                                                     | `search_grpc`       | No                                                           | Yes                 | No                               | Predicate-first discovery; traversal requires explicit `close_end`; `Regime` is `tf=1h` only                                  |
| `time_machine` | `Aggregator` bars, `Primitives` outputs, `Regime` outputs | `time_machine`                                                                                                               | `time_machine_grpc` | No                                                           | Yes                 | No                               | Context around hits; traversal requires explicit `close_end`; `Regime` is `tf=1h` only                                        |
| Bars WS        | `Aggregator`                                              | No                                                                                                                           | No                  | `connect_bars_ws`, `connect_bars_ws_make_before_break`       | N/A                 | `connect_bars_ws_recovering`     | Immutable subscription; changing the set requires reconnect or make-before-break; no gap-free continuity claim                |
| Outputs WS     | `Primitives`, `Regime`                                    | No                                                                                                                           | No                  | `connect_outputs_ws`, `connect_outputs_ws_make_before_break` | N/A                 | `connect_outputs_ws_recovering`  | Immutable subscription; projected protobuf WS fails closed for projected requests; `Regime` is `tf=1h` only                   |
| Messages WS    | `Aggregator`, `Primitives`, `Regime`                      | No                                                                                                                           | No                  | `connect_messages_ws`                                        | N/A                 | `connect_messages_ws_recovering` | Mutable subscribe/unsubscribe stream; not bars WS, not outputs WS, not historical replay                                      |

## Which Endpoint To Use

### Platform Map

`Intro` is the correct surface when the first question is where the public platform starts and which public system should be inspected next.

**Current binding:**

- `Intro`: `intro`

**What not to infer:**

- `Intro` is not the schema authority for subsystem routes
- `Intro` does not replace subsystem docs or OpenAPI

### System Explanation Before Wire Details

Docs pages are the correct surface when the question is what a system does, how its public endpoint families are organized, and which public concepts matter before integration.

**Current bindings:**

- `Aggregator`: `docs_system`, `docs_summary`, `docs_themes`, `docs_endpoints`
- `Primitives`: `docs_system`, `docs_summary`, `docs_taxonomy`, `docs_registry`, `docs_endpoints`
- `Regime`: `docs_system`, `docs_summary`, `docs_taxonomy`, `docs_registry`, `docs_endpoints`

**What not to infer:**

- docs pages are not the exact wire-schema authority
- selector catalogs for `primitives` and `regime` belong in taxonomy/registry, not in this README

### Exact HTTP Wire Contract

OpenAPI is the correct surface when the question is the exact route, request body, parameter, or response schema contract.

**Current bindings:**

- `Aggregator`: `openapi`
- `Primitives`: `openapi`
- `Regime`: `openapi`

**What not to infer:**

- OpenAPI is not the main conceptual overview surface
- OpenAPI does not replace the system docs when the question is explanatory

### Export Files Rather Than Query-Shaped Reads

The file flow is the correct surface when the task is export-oriented retrieval rather than snapshot, bounded history, hit discovery, or hit context.

**Current bindings:**

- `Aggregator`: `files_downloads`, `files_download_items`
- `Primitives`: `files_downloads`, `files_download_items`
- `Regime`: `files_downloads`, `files_download_items`

**What not to infer:**

- this is a two-step export flow, not a historical query family
- `files_download_items` materializes returned rows; it is not a hidden public file-management API

### Current Stable Snapshot

`latest` is the correct surface when the question is the current stable closed snapshot for the selected pairs and timeframe.

**Current bindings:**

- `Aggregator`: `latest`, `latest_grpc`
- `Primitives`: `latest`, `latest_grpc`
- `Regime`: `latest`, `latest_grpc`

**What not to infer:**

- `latest` is not a streaming surface
- `latest` is not a substitute for bounded history
- `Regime` currently remains `tf=1h` only

### Bounded Historical Slice

`range` is the correct surface when the question is which rows belong to a concrete bounded historical window on a fixed grid.

**Current bindings:**

- `Aggregator`: `range`, `range_grpc`
- `Primitives`: `range`, `range_grpc`
- `Regime`: `range`, `range_grpc`

**What not to infer:**

- traversal is explicit, not automatic
- a cursor is paging state, not a different query family
- range is not the right surface when the primary question is event discovery
- `Regime` currently remains `tf=1h` only

### Condition Hit Discovery

`search` is the correct surface when the predicate itself is the primary question and hit discovery across a historical window is required.

**Current bindings:**

- `Aggregator`: `search`, `search_grpc`
- `Primitives`: `search`, `search_grpc`
- `Regime`: `search`, `search_grpc`

**What not to infer:**

- `search` is not a full bounded history dump
- `search` is not the same shape as messages WS
- traversal requires explicit `close_end`
- `Regime` currently remains `tf=1h` only

### Hit Context

`time_machine` is the correct surface when hit timestamps are already known, or when the system should discover hits and then return nearby context.

**Current bindings:**

- `Aggregator`: `time_machine`, `time_machine_grpc`
- `Primitives`: `time_machine`, `time_machine_grpc`
- `Regime`: `time_machine`, `time_machine_grpc`

**What not to infer:**

- this is a context surface, not a general history replacement
- it does not imply automatic traversal
- `Regime` currently remains `tf=1h` only

### Live Bars With One Fixed Subscription

Bars WS is the correct surface when the question is live bar delivery for one immutable subscription set.

**Current bindings:**

- `Aggregator`: `connect_bars_ws`, `connect_bars_ws_make_before_break`, `connect_bars_ws_recovering`

**What not to infer:**

- bars WS does not support in-band unsubscribe
- changing the subscription set requires reconnect or make-before-break
- managed recovery does not prove gap-free continuity

### Live Computed Outputs With One Fixed Subscription

Outputs WS is the correct surface when the question is live computed-state delivery for one immutable subscription set.

**Current bindings:**

- `Primitives`: `connect_outputs_ws`, `connect_outputs_ws_make_before_break`, `connect_outputs_ws_recovering`
- `Regime`: `connect_outputs_ws`, `connect_outputs_ws_make_before_break`, `connect_outputs_ws_recovering`

**What not to infer:**

- outputs WS is not messages WS
- projected protobuf outputs WS paths fail closed for projected requests
- `Regime` currently remains `tf=1h` only

### Live Predicate-Triggered Notifications With Mutable Subscriptions

Messages WS is the correct surface when the question is live rule-based notification and connection-local subscribe and unsubscribe control are required.

**Current bindings:**

- `Aggregator`: `connect_messages_ws`, `connect_messages_ws_recovering`
- `Primitives`: `connect_messages_ws`, `connect_messages_ws_recovering`
- `Regime`: `connect_messages_ws`, `connect_messages_ws_recovering`

**What not to infer:**

- messages WS is not the same shape as bars WS or outputs WS
- replay/backfill is not the contract of this surface
- active subscribe state is connection-local and must be re-established after reconnect

## Examples

The examples below stay shape-first. Query, traversal, and generic streaming examples default to `Aggregator` where the public shape is easiest to show there. Output systems are not left implicit: a selector-bearing `Primitives` example is included explicitly, and `Regime` follows the same output shape while adding `secondary` and remaining `tf=1h` only.

Each snippet assumes a previously constructed client of the matching system type, as shown in `Installation` and `Quick Start`.

### Latest Bars

This example answers the question: what is the current stable closed snapshot for these pairs?

```rust
use mathilde_sdk_rs::systems::aggregator::LatestRequest;
use mathilde_sdk_rs::systems::types::{HttpFormat, LatestMode, Timeframe};

let out = client
    .latest(&LatestRequest {
        pairs: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: LatestMode::ExactWatermark,
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    })
    .await?;

println!("rows={}", out.rows.len());
println!("close_end_ms={}", out.close_end_ms);
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Latest Primitives Outputs

This example shows the same `latest` shape on a `Primitives` client when selector-bearing outputs are required.

```rust
use mathilde_sdk_rs::systems::primitives::{
    LatestRequest, ProcessorFamily, ProcessorGroup,
};
use mathilde_sdk_rs::systems::types::{HttpFormat, LatestMode, Timeframe};

let out = client
    .latest(&LatestRequest {
        pairs: vec!["BTCUSDT".to_string()],
        tf: Timeframe::M1,
        latest_mode: Some(LatestMode::ExactWatermark),
        family: Some(vec![ProcessorFamily::MovingAverages]),
        group: Some(vec![ProcessorGroup::Ema]),
        metadata: Some(false),
        diagnostics: Some(false),
        format: Some(HttpFormat::Json),
    })
    .await?;

println!("pair={}", out.rows[0].row.pair);
if let Some(value) = out.rows[0].row.computed.f64("ma_ema_p20") {
    println!("ma_ema_p20={value}");
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Range Bars

This example answers the question: what are the closed bars for this bounded historical interval?

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::RangeRequest;
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::types::{AlignMode, HttpFormat, Timeframe};

let out = client
    .range(&RangeRequest {
        pairs: pairs(["BTCUSDT", "ETHUSDT"]),
        tf: Timeframe::M1,
        align_mode: Some(AlignMode::Exact),
        close_start: Some(TimeInput::Utc("2026-02-02T00:00:00Z".to_string())),
        cursor: None,
        close_end: Some(TimeInput::Utc("2026-02-02T06:00:00Z".to_string())),
        limit: Some(1000),
        metadata: Some(false),
        format: Some(HttpFormat::Json),
    })
    .await?;

println!("rows={}", out.rows.len());
println!("next_cursor={:?}", out.next_cursor);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Full traversal is explicit when the whole bounded range is required:

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::RangeRequest;
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::types::{AlignMode, HttpFormat, Timeframe};

let request = RangeRequest {
    pairs: pairs(["BTCUSDT"]),
    tf: Timeframe::M1,
    align_mode: Some(AlignMode::Exact),
    close_start: Some(TimeInput::Utc("2026-02-02T00:00:00Z".to_string())),
    cursor: None,
    close_end: Some(TimeInput::Utc("2026-02-02T06:00:00Z".to_string())),
    limit: Some(1000),
    metadata: Some(false),
    format: Some(HttpFormat::Json),
};

let out = client.range_call(request).traverse().await?;
println!("pages_fetched={}", out.pages_fetched);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`traverse()` calls `pager()` internally and materializes every fetched page into `out.pages`. For larger windows, explicit `close_end` and a sensible `limit` keep the page count bounded more intentionally.

Manual continuation remains explicit through the pager:

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::RangeRequest;
use mathilde_sdk_rs::systems::helpers::pairs;
use mathilde_sdk_rs::systems::types::{AlignMode, HttpFormat, Timeframe};

let request = RangeRequest {
    pairs: pairs(["BTCUSDT"]),
    tf: Timeframe::M1,
    align_mode: Some(AlignMode::Exact),
    close_start: Some(TimeInput::Utc("2026-02-02T00:00:00Z".to_string())),
    cursor: None,
    close_end: None,
    limit: Some(1000),
    metadata: Some(false),
    format: Some(HttpFormat::Json),
};

let mut pager = client.range_call(request).pager()?;

while let Some(page) = pager.next().await? {
    println!("page rows={}", page.rows.len());
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

`pager()` is the lower-memory continuation path because it processes one page at a time instead of collecting the full page set in memory. For large `range` windows, explicit `close_end` and a sensible `limit` remain the safer default, even though omitted `close_end` is still a distinct range contract.

### Search Bars

This example answers the question: at which stable closes did this predicate become true?

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::SearchRequest;
use mathilde_sdk_rs::systems::types::{HttpFormat, Timeframe};

let out = client
    .search(&SearchRequest {
        tf: Timeframe::M1,
        close_start: TimeInput::Utc("2026-02-02T00:00:00Z".to_string()),
        close_end: Some(TimeInput::Utc("2026-02-02T06:00:00Z".to_string())),
        cursor: None,
        predicate: "BTCUSDT.c > ETHUSDT.c * 1.5 && ETHUSDT.v > 10".to_string(),
        evaluate_pair: Some("BTCUSDT".to_string()),
        metadata: Some(false),
        max_hits: Some(500),
        format: Some(HttpFormat::Json),
    })
    .await?;

println!("hits={}", out.hits.len());
println!("next_cursor={:?}", out.next_cursor);
println!("predicate_normalized={}", out.predicate_normalized);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Full traversal across every search page keeps `close_end` explicit:

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::SearchRequest;
use mathilde_sdk_rs::systems::types::{HttpFormat, Timeframe};

let request = SearchRequest {
    tf: Timeframe::M1,
    close_start: TimeInput::Utc("2026-02-02T00:00:00Z".to_string()),
    close_end: Some(TimeInput::Utc("2026-02-02T06:00:00Z".to_string())),
    cursor: None,
    predicate: "BTCUSDT.c > ETHUSDT.c * 1.5 && ETHUSDT.v > 10".to_string(),
    evaluate_pair: Some("BTCUSDT".to_string()),
    metadata: Some(false),
    max_hits: Some(500),
    format: Some(HttpFormat::Json),
};

let out = client.search_call(request.clone()).traverse().await?;
println!("pages_fetched={}", out.pages_fetched);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`traverse()` materializes every fetched search page in memory. `search` traversal and paging require explicit `close_end`; for larger windows, `max_hits` should stay bounded sensibly and `client.search_call(request).pager()?` is the lower-memory continuation path.

### Time-Machine Bars

This example answers the question: what did the local bars context look like around these hits?

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::TimeMachineRequest;
use mathilde_sdk_rs::systems::types::{HttpFormat, Timeframe};

let out = client
    .time_machine(&TimeMachineRequest {
        tf: Timeframe::M1,
        close_start: TimeInput::Utc("2026-02-02T00:00:00Z".to_string()),
        close_end: Some(TimeInput::Utc("2026-02-02T02:00:00Z".to_string())),
        cursor: None,
        predicate: Some("BTCUSDT.c > ETHUSDT.c * 1.5".to_string()),
        hits: None,
        output_pairs: Some(vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]),
        metadata: Some(false),
        before_bars: Some(10),
        after_bars: Some(10),
        max_hits: Some(100),
        overlap_mode: Some("merge".to_string()),
        format: Some(HttpFormat::Json),
    })
    .await?;

println!("rows={}", out.rows.len());
println!("next_cursor={:?}", out.next_cursor);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Full traversal across every `time_machine` page keeps `close_end` explicit:

```rust
use mathilde_sdk_rs::core::time::TimeInput;
use mathilde_sdk_rs::systems::aggregator::TimeMachineRequest;
use mathilde_sdk_rs::systems::types::{HttpFormat, Timeframe};

let request = TimeMachineRequest {
    tf: Timeframe::M1,
    close_start: TimeInput::Utc("2026-02-02T00:00:00Z".to_string()),
    close_end: Some(TimeInput::Utc("2026-02-02T02:00:00Z".to_string())),
    cursor: None,
    predicate: Some("BTCUSDT.c > ETHUSDT.c * 1.5".to_string()),
    hits: None,
    output_pairs: Some(vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]),
    metadata: Some(false),
    before_bars: Some(10),
    after_bars: Some(10),
    max_hits: Some(100),
    overlap_mode: Some("merge".to_string()),
    format: Some(HttpFormat::Json),
};

let out = client.time_machine_call(request.clone()).traverse().await?;
println!("pages_fetched={}", out.pages_fetched);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`traverse()` materializes every fetched `time_machine` page in memory. `time_machine` traversal and paging require explicit `close_end`; for larger windows, `max_hits` and the context window settings should stay bounded sensibly and `client.time_machine_call(request).pager()?` is the lower-memory continuation path.

### Bars WS

This example covers live bars for one fixed subscription set.

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
        BarsWsInboundFrame::JsonRows(rows) => {
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

The recovering wrapper makes reconnect behavior explicit:

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

This example covers live predicate-triggered messages from one mutable subscription set.

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

The recovering wrapper makes reconnect-plus-replay behavior explicit:

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

HTTP covers the public HTTP contract. For `aggregator` bars, that includes JSON and protobuf where the public contract proves both. For `primitives` and `regime`, projected HTTP requests with `format=protobuf` are intentionally rejected before transport when projected omitted-vs-selected semantics are not locally proved.

gRPC mirrors the current unary query families. Caller-facing request shapes stay aligned with HTTP where possible, minus the HTTP-only `format` field. For `primitives` and `regime`, projected gRPC output requests are intentionally rejected before transport.

WS is not one generic stream surface. The current SDK exposes three different live shapes with different contracts:

- bars WS: immutable subscription per connection
- outputs WS: immutable subscription per connection
- messages WS: in-band subscribe and unsubscribe

For output systems, projected protobuf WS requests are also intentionally rejected before transport when the request is projected.

## WS Recovery And Limits

WS recovery is opt-in. The raw WS connection types remain thin and do not hide reconnect policy.

Managed bars recovery reconnects with the same subscribe request using bounded exponential backoff. Make-before-break is a separate coordination surface. This proves reconnect behavior, not gap-free continuity after disconnect.

Managed outputs recovery is also opt-in. The current proved output recovery behavior is reconnect-with-the-same-request on the primitives outputs surface. This README does not claim stronger continuity semantics for outputs recovery than the current shared contract proves.

Managed messages recovery reconnects and replays active subscribe state. This is a different contract from bars WS and outputs WS because the active subscription set is mutable and connection-local.

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
- the SDK guesses local time inputs
- the SDK owns the full predicate-language parser contract
- bars or outputs recovery is gap-free
- the SDK hides all transport policy by default
- projected protobuf paths are supported for projected output requests
- `regime` supports timeframes beyond `1h`
- every MATHILDE subsystem is already bound here

Also note the current public config contract: `AggregatorConfig` always requires HTTP config, even if a caller plans to use only gRPC or WS.

## Further Reading

- [Live examples and workflows](examples/README.md)
- [Validation report: full public endpoint verification](bin/endpoint_test/endpoint_test_20260505T090037Z.md)
- [Validation report: live public surface checks](bin/sdk_live_public_surface_check/sdk_live_public_surface_check_20260505T090054Z.md)
