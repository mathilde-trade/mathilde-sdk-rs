# Live Examples And Workflows

`examples/` contains runnable public-SDK examples. They stay close to the
proved SDK surface, keep transport and endpoint behavior explicit, and show the
smallest useful flow for each public question shape.

## Index

- [What This Is](#what-this-is)
- [What This Is Not](#what-this-is-not)
- [How To Run](#how-to-run)
- [Basic Examples](#basic-examples)
- [Workflow Examples](#workflow-examples)
- [Download Outputs](#download-outputs)

## What This Is

These examples are the public usage layer for the current SDK surface.

They are useful when the question is:

- which public method answers this question shape
- how bearer auth is passed in a real run
- how bars differ from computed-output rows
- how `family` and `group` selectors narrow compute views
- when `search` is better than `time_machine`
- how file downloads and local reproducible slices are materialized

The examples stay intentionally small:

- one bearer declaration near the beginning
- one client per system surface used
- one compact flow per file
- minimal inline comments at the main user decision points

## What This Is Not

These examples are not benchmarks, not private-system integrations, not hidden
retry wrappers, and not opinion or trading flows.

They do not claim:

- prediction
- hidden paging or traversal
- hidden `.env` loading by `cargo run`
- private endpoint coverage
- transport equivalence where the public contract differs

If a behavior is not proved by the current SDK surface, these examples do not
claim it.

## How To Run

All examples use `EXAMPLE_BEARER_TOKEN`.

```bash
export EXAMPLE_BEARER_TOKEN='...'
cargo run --example latest
```

`cargo run` does not load `.env` automatically. Export the bearer first or pass
it inline for one command.

Workflow example targets are also runnable directly, for example:

```bash
cargo run --example current_downside_state
```

## Basic Examples

### Documentation

File: [documenation.rs](documenation.rs)

What it shows:

- `Intro`
- public docs reads for `Aggregator`, `Primitives`, and `Regime`
- filtered registry reads for the compute systems

Why it is useful:

- use this first when the question is understanding rather than retrieval
- it shows where taxonomy and registry belong in the public navigation flow

What it is not:

- not a retrieval example
- not the exact wire-contract authority

### Status

File: [status.rs](status.rs)

What it shows:

- `pairs_list(...)`
- `pairs_status(...)`
- one compact readiness summary per system

Why it is useful:

- use this when the first question is which pairs are present or ready before
  making data requests

What it is not:

- not a history read
- not a files or replay example

### Files

File: [files.rs](files.rs)

What it shows:

- `files_downloads(...)`
- explicit `files_download_items(...)`
- one bounded listing request per system

Why it is useful:

- use this when the task is export retrieval rather than query-shaped reads
- it keeps download selection explicit instead of downloading everything in one
  silent step

What it is not:

- not a hidden bulk downloader
- not a local research join example

### WebSocket

File: [ws.rs](ws.rs)

What it shows:

- bounded Aggregator bars WS
- bounded Primitives outputs WS
- bounded Regime outputs WS
- Aggregator make-before-break with full promotion proof

Why it is useful:

- use this when the question is live delivery shape, replay/meta ordering, or
  make-before-break behavior

What it is not:

- not a historical extraction example
- not a persistent production consumer

### Latest

File: [latest.rs](latest.rs)

What it shows:

- one latest Aggregator bar
- one latest Primitives row
- one latest Regime row
- processor fields through `row.computed.f64(...)`

Why it is useful:

- use this when the question is the newest stable closed read now
- it shows the difference between fixed bars and compute fields clearly

What it is not:

- not a bounded history window
- not a hit-discovery example

### Range

File: [range.rs](range.rs)

What it shows:

- one bounded page from `range(...)`
- computed-field access on range rows
- one bounded `range_call(...).traverse()` example

Why it is useful:

- use this when the question is reproducible bounded history
- it shows when traversal is appropriate and why it must stay bounded

What it is not:

- not the best first surface for hit discovery
- not a live stream

### Search

File: [search.rs](search.rs)

What it shows:

- bounded `search(...)` requests across all three systems
- `evaluate_pair` and `evaluated_rows`
- hit-only discovery when `evaluate_pair` is omitted

Why it is useful:

- use this when the first need is timestamps where a condition became true
- it is the cleaner first step when the same hits will be reused later

What it is not:

- not the replay step itself
- not a general bounded history surface

### Time Machine

File: [time_machine.rs](time_machine.rs)

What it shows:

- one-pass replay with `predicate`
- `before_bars` and `after_bars`
- offset interpretation around each hit

Why it is useful:

- use this when one system can both find the hits and return the local context
  in one pass

What it is not:

- not always the best first step when hits must be reused across multiple
  replay calls

### Transport

File: [transport.rs](transport.rs)

What it shows:

- one Aggregator latest request over HTTP JSON
- the same request over HTTP protobuf
- the same request over gRPC
- the `LatestGrpcRequest::from(&latest_request)` adapter

Why it is useful:

- use this when the question is transport equivalence rather than endpoint
  meaning

What it is not:

- not a full transport benchmark
- not a complete cross-system matrix

## Workflow Examples

The workflow examples live under [`worflows/`](worflows). The directory name is
kept as currently implemented.

These files preserve the authoritative workflow content from
`https://api.mathilde.dev` and turn it into runnable SDK examples.

### Understanding The System

File: [understanding_system.rs](worflows/understanding_system.rs)

What it shows:

- the top-level navigation flow from `Intro`
- system docs, taxonomy, registry, endpoints, and OpenAPI reads

Why it is useful:

- use this first when the question is which public surface should be used next

What it is not:

- not a data retrieval flow

### Bounded Recent Window

File: [bounded_recent_window.rs](worflows/bounded_recent_window.rs)

What it shows:

- bounded recent Aggregator bars
- bounded recent Primitives outputs
- local alignment by close time

Why it is useful:

- use this when the question is a recent stable window, not a similarity search

What it is not:

- not a hit-discovery workflow
- not an offline export workflow

### Current Downside State

File: [current_downside_state.rs](worflows/current_downside_state.rs)

What it shows:

- current Primitives downside row
- broad downside predicate built from measured fields
- `search` as reusable hit discovery
- `time_machine(hits=...)` on both Primitives and Aggregator

Why it is useful:

- use this when computed-state hits must be reused across output replay and bar
  replay

What it is not:

- not a one-pass replay example
- not a prediction claim

### Measured Local Stress Context

File: [measured_local_stress_context.rs](worflows/measured_local_stress_context.rs)

What it shows:

- Primitives taxonomy and filtered registry grounding
- a concrete local-stress predicate on measured fields
- reusable hit discovery plus replay

Why it is useful:

- use this when the predicate is defined by compute fields and the same matched
  moments must be replayed across systems

What it is not:

- not a generic bars-only workflow

### Current Grouped Regime State

File: [current_grouped_regime_state.rs](worflows/current_grouped_regime_state.rs)

What it shows:

- current grouped BTC regime row
- a coarse grouped-state predicate built from that row
- one-pass Regime `time_machine(predicate=...)`

Why it is useful:

- use this when grouped-state similarity is needed only inside the Regime
  surface and the matched timestamps are not reused elsewhere

What it is not:

- not a cross-system reusable-hit workflow

### Reproducible Monthly Research

File: [reproducible_monthly_research.rs](worflows/reproducible_monthly_research.rs)

What it shows:

- monthly Aggregator files discovery
- monthly Primitives files discovery
- explicit parquet download into a timestamped example-local run directory
- local DuckDB join
- a reproducible manifest and joined parquet slice

Why it is useful:

- use this when the task is offline auditable research, not interactive latest
  retrieval

What it is not:

- not a live snapshot flow
- not a hidden warehouse pipeline

## Download Outputs

Examples that write files keep them inside this repository so users can inspect
or remove them easily after testing.

Current download roots:

- `files.rs`:
  - `examples/downloads/files/...`
- `reproducible_monthly_research.rs`:
  - `examples/downloads/reproducible_monthly_research_<utc-timestamp>/...`

The monthly research workflow also writes:

- a local DuckDB database
- the exact join SQL
- a joined parquet file
- a manifest describing the slice and its file identities
