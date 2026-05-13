# `mathilde-sdk-rs` — Global Inventory (GENERATED; DO NOT EDIT)

Generated: 2026-05-13T10:14:47Z
Protocol: `.dev/specs/SDK_INVENTORY_SYSTEM_SPEC_2026-05-05.md`

This file is generated from per-component inventories under `crates/*/docs/inventory.md`, `services/*/docs/inventory.md`, SDK module inventories under `src/*/docs/inventory.md`, and the runnable examples inventory at `examples/docs/inventory.md`.
If a crate does not have a top-level `docs/inventory.md`, this generator will also include module inventories under `crates/*/src/*/docs/inventory.md`.
If a file purpose is missing in a component inventory, this file will mark it as `INVENTORY GAP`.

## Components

- `module::sdk::core`: `src/core/docs/inventory.md`
- `module::sdk::examples`: `examples/docs/inventory.md`
- `module::sdk::streaming`: `src/streaming/docs/inventory.md`
- `module::sdk::systems`: `src/systems/docs/inventory.md`
- `module::sdk::tests`: `src/tests/docs/inventory.md`
- `module::sdk::transport`: `src/transport/docs/inventory.md`

---

## `src/core`

### Artifacts

- Inventory: `src/core/docs/inventory.md`

### Source Files

- `src/core/auth.rs`: bearer-token validation and HTTP auth-header helper.
- `src/core/config.rs`: typed SDK transport configuration surfaces and builders.
- `src/core/error.rs`: shared typed SDK error surface across transports and endpoint families.
- `src/core/mod.rs`: core module wiring and exports for shared SDK primitives.
- `src/core/pagination.rs`: shared pagination state-machine primitives, repeated-cursor guards, and explicit traversal-admission helpers.
- `src/core/time.rs`: shared `TimeInput` parsing and UTC-ms normalization logic.

---

## `examples`

### Artifacts

- Inventory: `examples/docs/inventory.md`

### Source Files

- `examples/documenation.rs`: public intro and documentation navigation example across aggregator, primitives, and regime.
- `examples/files.rs`: bounded file-listing and explicit selected-download example across all three systems.
- `examples/latest.rs`: latest closed-read example showing bars versus computed rows and getter-based processor field access.
- `examples/range.rs`: bounded range and bounded traverse example for fixed bars and computed outputs.
- `examples/search.rs`: hit-discovery example across all three systems, including reusable hit-only mode and evaluated-row mode.
- `examples/status.rs`: pairs list and readiness-status example across aggregator, primitives, and regime.
- `examples/time_machine.rs`: one-pass replay-context example using `predicate`, `before_bars`, `after_bars`, and offsets.
- `examples/transport.rs`: transport-equivalence example for one aggregator latest request over HTTP JSON, HTTP protobuf, and gRPC.
- `examples/workflows/bounded_recent_window.rs`: workflow example for aligning a bounded recent aggregator window with primitives computed rows.
- `examples/workflows/current_downside_state.rs`: workflow example for discovering downside-state hits in primitives and replaying shared hit timestamps in primitives and aggregator.
- `examples/workflows/current_grouped_regime_state.rs`: workflow example for current grouped regime-state discovery and direct regime replay context.
- `examples/workflows/due_diligence_review_packs.rs`: workflow example for intro-host due-diligence index and approved review-pack navigation across the two regime packs and two primitives family packs.
- `examples/workflows/measured_local_stress_context.rs`: workflow example for measured local stress discovery through primitives taxonomy, registry, search, and replay context.
- `examples/workflows/reproducible_monthly_research.rs`: workflow example for reproducible monthly slice downloads and local DuckDB joins.
- `examples/workflows/understanding_system.rs`: workflow example that mirrors the authoritative intro understanding workflow across intro, docs, registry, endpoints, and OpenAPI.
- `examples/ws.rs`: bounded WebSocket example across all three systems plus full make-before-break promotion proof for aggregator bars.

---

## `src/streaming`

### Artifacts

- Inventory: `src/streaming/docs/inventory.md`

### Source Files

- `src/streaming/make_before_break.rs`: shared make-before-break validation-window config.
- `src/streaming/mod.rs`: streaming module wiring and exports for shared WS coordination helpers.
- `src/streaming/subscription.rs`: shared WS recovery backoff config and reconnect state primitives.

---

## `src/systems`

### Artifacts

- Inventory: `src/systems/docs/inventory.md`

### Source Files

- `src/systems/aggregator/bars_grpc.rs`: aggregator public bars gRPC bindings.
- `src/systems/aggregator/bars_http.rs`: aggregator public bars HTTP bindings.
- `src/systems/aggregator/bars_pagination.rs`: aggregator explicit call-wrapper, pager, and traverse helpers layered on the one-page bars bindings.
- `src/systems/aggregator/bars_ws.rs`: aggregator public bars WS bindings, make-before-break, and managed recovery.
- `src/systems/aggregator/client.rs`: typed public client entrypoints for aggregator HTTP, gRPC, and WS surfaces.
- `src/systems/aggregator/docs.rs`: aggregator public documentation and OpenAPI bindings.
- `src/systems/aggregator/files.rs`: aggregator public file-download issuance bindings and typed local-download convenience.
- `src/systems/aggregator/messages_ws.rs`: aggregator public messages WS bindings and managed recovery.
- `src/systems/aggregator/mod.rs`: aggregator system module wiring and public exports.
- `src/systems/aggregator/pairs.rs`: aggregator public pairs discovery and status bindings.
- `src/systems/aggregator/types.rs`: aggregator-specific request, response, traversal-result, and WS frame types.
- `src/systems/helpers.rs`: shared public collection helpers for system-facing request ergonomics.
- `src/systems/intro/client.rs`: typed public client entrypoint for the dedicated intro host surface on `api.mathilde.dev`, including intro-root, legal-bundle, and due-diligence document reads.
- `src/systems/intro/intro.rs`: intro-host HTTP bindings for the root intro document, the deploy-owned `/v1/legal` JSON document bundle, and the approved `/v1/due-diligence` JSON document routes.
- `src/systems/intro/mod.rs`: intro system module wiring and public exports.
- `src/systems/mod.rs`: top-level system module wiring and exports.
- `src/systems/primitives/client.rs`: typed public client entrypoints for primitives HTTP, gRPC, and WS surfaces.
- `src/systems/primitives/docs.rs`: primitives public documentation and OpenAPI bindings.
- `src/systems/primitives/files.rs`: primitives public file-download issuance bindings and typed local-download convenience.
- `src/systems/primitives/messages_ws.rs`: primitives public messages WS bindings and managed recovery.
- `src/systems/primitives/mod.rs`: primitives system module wiring, generated contract re-exports, and public exports.
- `src/systems/primitives/outputs_grpc.rs`: primitives public outputs gRPC bindings.
- `src/systems/primitives/outputs_http.rs`: primitives public outputs HTTP bindings.
- `src/systems/primitives/outputs_pagination.rs`: primitives explicit call-wrapper, pager, and traverse helpers layered on the one-page outputs bindings.
- `src/systems/primitives/outputs_ws.rs`: primitives public outputs WS bindings, make-before-break, and managed recovery.
- `src/systems/primitives/pairs.rs`: primitives public pairs discovery and status bindings.
- `src/systems/primitives/types.rs`: primitives-specific request, response, traversal-result, and WS frame types.
- `src/systems/regime/client.rs`: typed public client entrypoints for regime HTTP, gRPC, and WS surfaces.
- `src/systems/regime/docs.rs`: regime public documentation and OpenAPI bindings.
- `src/systems/regime/files.rs`: regime public file-download issuance bindings and typed local-download convenience.
- `src/systems/regime/messages_ws.rs`: regime public messages WS bindings and managed recovery.
- `src/systems/regime/mod.rs`: regime system module wiring, generated contract re-exports, and public exports.
- `src/systems/regime/outputs_grpc.rs`: regime public outputs gRPC bindings.
- `src/systems/regime/outputs_http.rs`: regime public outputs HTTP bindings.
- `src/systems/regime/outputs_pagination.rs`: regime explicit call-wrapper, pager, and traverse helpers layered on the one-page outputs bindings.
- `src/systems/regime/outputs_ws.rs`: regime public outputs WS bindings, make-before-break, and managed recovery.
- `src/systems/regime/pairs.rs`: regime public pairs discovery and status bindings.
- `src/systems/regime/types.rs`: regime-specific request, response, traversal-result, and WS frame types.
- `src/systems/types.rs`: shared cross-system public enums and wire-facing labels.

---

## `src/tests`

### Artifacts

- Inventory: `src/tests/docs/inventory.md`

### Source Files

- `src/tests/grpc_test_support.rs`: shared gRPC frame decoding helpers for contract tests that inspect direct transport payloads.
- `src/tests/mod.rs`: top-level SDK test module wiring.
- `src/tests/test_aggregator_public_grpc_latest.rs`: contract tests for aggregator gRPC latest bars.
- `src/tests/test_aggregator_public_grpc_range.rs`: contract tests for aggregator gRPC range bars.
- `src/tests/test_aggregator_public_grpc_search.rs`: contract tests for aggregator gRPC search bars.
- `src/tests/test_aggregator_public_grpc_time_machine.rs`: contract tests for aggregator gRPC time-machine bars.
- `src/tests/test_aggregator_public_http_files_downloads.rs`: contract tests for aggregator HTTP file downloads.
- `src/tests/test_aggregator_public_http_latest.rs`: contract tests for aggregator HTTP latest bars and docs base behavior.
- `src/tests/test_aggregator_public_http_range.rs`: contract tests for aggregator HTTP bars range.
- `src/tests/test_aggregator_public_http_search.rs`: contract tests for aggregator HTTP bars search.
- `src/tests/test_aggregator_public_http_simple_discovery.rs`: contract tests for aggregator HTTP simple discovery and pairs surfaces.
- `src/tests/test_aggregator_public_http_time_machine.rs`: contract tests for aggregator HTTP bars time-machine.
- `src/tests/test_aggregator_public_ws_bars.rs`: contract tests for aggregator WS bars, make-before-break, and managed recovery.
- `src/tests/test_aggregator_public_ws_messages.rs`: contract tests for aggregator WS messages and managed recovery.
- `src/tests/test_core_pagination.rs`: contract tests for the shared pagination state machine and explicit traversal-admission guards.
- `src/tests/test_core_time.rs`: contract tests for shared time parsing and normalization.
- `src/tests/test_intro_public_http_intro.rs`: contract tests for the dedicated intro host system on `https://api.mathilde.dev`, including root-path intro behavior, `/v1/legal` and `/v1/due-diligence` route formation, bearer propagation, and ordered JSON preservation.
- `src/tests/test_primitives_docs.rs`: primitives docs selector serialization tests.
- `src/tests/test_primitives_outputs_grpc.rs`: primitives gRPC fail-closed projected selector contract tests.
- `src/tests/test_primitives_outputs_http.rs`: primitives HTTP fail-closed projected protobuf contract tests.
- `src/tests/test_primitives_outputs_pagination.rs`: primitives pagination admission tests for range, search, and time-machine calls.
- `src/tests/test_primitives_outputs_ws.rs`: primitives outputs WS projected protobuf fail-closed contract tests.
- `src/tests/test_primitives_public_grpc_latest.rs`: contract tests for primitives gRPC latest outputs request mapping and min-response decode.
- `src/tests/test_primitives_public_grpc_range.rs`: contract tests for primitives gRPC range outputs request mapping and metadata response decode.
- `src/tests/test_primitives_public_grpc_search.rs`: contract tests for primitives gRPC search outputs request mapping and evaluated-rows decode.
- `src/tests/test_primitives_public_grpc_time_machine.rs`: contract tests for primitives gRPC time-machine outputs request mapping and response decode.
- `src/tests/test_primitives_public_http_files_downloads.rs`: contract tests for primitives HTTP file downloads and authenticated local download behavior.
- `src/tests/test_primitives_public_http_latest.rs`: contract tests for primitives HTTP latest outputs, docs base behavior, and public-default config construction.
- `src/tests/test_primitives_public_http_range.rs`: contract tests for primitives HTTP range outputs, including projected JSON decode and projected protobuf fail-closed behavior.
- `src/tests/test_primitives_public_http_search.rs`: contract tests for primitives HTTP search outputs with typed metadata payloads.
- `src/tests/test_primitives_public_http_simple_discovery.rs`: contract tests for primitives HTTP docs, registry selector serialization, OpenAPI, and pairs discovery.
- `src/tests/test_primitives_public_http_time_machine.rs`: contract tests for primitives HTTP time-machine outputs with projected metadata payloads.
- `src/tests/test_primitives_public_ws_messages.rs`: contract tests for primitives messages WS control frames and recovering subscription replay.
- `src/tests/test_primitives_public_ws_outputs.rs`: contract tests for primitives outputs WS subscribe wiring and JSON row decode.
- `src/tests/test_primitives_types.rs`: primitives type, selector, and proto decode tests previously kept inline in runtime source files.
- `src/tests/test_public_surface_exports.rs`: contract tests for curated short-name exports and the absence of public internal module leaks in the system module surfaces.
- `src/tests/test_regime_docs.rs`: regime docs selector serialization tests.
- `src/tests/test_regime_outputs_grpc.rs`: regime gRPC fail-closed projected selector and non-`1h` contract tests.
- `src/tests/test_regime_outputs_http.rs`: regime HTTP fail-closed projected protobuf and non-`1h` contract tests.
- `src/tests/test_regime_outputs_pagination.rs`: regime pagination admission tests for range, search, and time-machine calls.
- `src/tests/test_regime_outputs_ws.rs`: regime outputs WS projected protobuf and non-`1h` fail-closed contract tests.
- `src/tests/test_regime_public_grpc_latest.rs`: contract tests for regime gRPC latest outputs request mapping and min-response decode.
- `src/tests/test_regime_public_grpc_range.rs`: contract tests for regime gRPC range outputs request mapping and metadata response decode.
- `src/tests/test_regime_public_grpc_search.rs`: contract tests for regime gRPC search outputs request mapping and evaluated-rows decode.
- `src/tests/test_regime_public_grpc_time_machine.rs`: contract tests for regime gRPC time-machine outputs request mapping and response decode.
- `src/tests/test_regime_public_http_files_downloads.rs`: contract tests for regime HTTP file downloads and authenticated local download behavior.
- `src/tests/test_regime_public_http_latest.rs`: contract tests for regime HTTP latest outputs, docs base behavior, and public-default config construction.
- `src/tests/test_regime_public_http_range.rs`: contract tests for regime HTTP range outputs, including projected JSON decode and projected protobuf fail-closed behavior.
- `src/tests/test_regime_public_http_search.rs`: contract tests for regime HTTP search outputs with typed metadata payloads.
- `src/tests/test_regime_public_http_simple_discovery.rs`: contract tests for regime HTTP docs, registry selector serialization, OpenAPI, and pairs discovery.
- `src/tests/test_regime_public_http_time_machine.rs`: contract tests for regime HTTP time-machine outputs with projected metadata payloads.
- `src/tests/test_regime_public_ws_messages.rs`: contract tests for regime messages WS control frames and recovering subscription replay.
- `src/tests/test_regime_public_ws_outputs.rs`: contract tests for regime outputs WS subscribe wiring, `secondary`, and JSON row decode.
- `src/tests/test_regime_types.rs`: regime type, selector, timeframe, and proto decode tests.
- `src/tests/test_systems_helpers.rs`: contract tests for shared systems helper collectors.
- `src/tests/test_transport_grpc.rs`: contract tests for shared gRPC transport behavior.

---

## `src/transport`

### Artifacts

- Inventory: `src/transport/docs/inventory.md`

### Source Files

- `src/transport/grpc.rs`: shared gRPC transport wrapper, channel handling, and bearer metadata injection.
- `src/transport/http.rs`: shared HTTP transport wrapper and auth-aware request builder.
- `src/transport/mod.rs`: transport module wiring and exports for HTTP, gRPC, and WS helpers.
- `src/transport/ws.rs`: shared WS upgrade URL normalization and bearer-auth upgrade headers.

---
