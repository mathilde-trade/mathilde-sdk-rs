# `mathilde-sdk-rs` — Global Inventory (GENERATED; DO NOT EDIT)

Generated: 2026-05-03T13:42:49Z
Protocol: `.dev/specs/SDK_INVENTORY_SYSTEM_SPEC_2026-04-08.md`

This file is generated from per-component inventories under `crates/*/docs/inventory.md`, `services/*/docs/inventory.md`, and SDK module inventories under `src/*/docs/inventory.md`.
If a crate does not have a top-level `docs/inventory.md`, this generator will also include module inventories under `crates/*/src/*/docs/inventory.md`.
If a file purpose is missing in a component inventory, this file will mark it as `INVENTORY GAP`.

## Components

- `module::sdk::core`: `src/core/docs/inventory.md`
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
- `src/core/client.rs`: reserved placeholder for a future top-level SDK facade; no implemented runtime code yet.
- `src/core/config.rs`: typed SDK transport configuration surfaces and builders.
- `src/core/error.rs`: shared typed SDK error surface across transports and endpoint families.
- `src/core/mod.rs`: core module wiring and exports for shared SDK primitives.
- `src/core/pagination.rs`: shared pagination state-machine primitives, repeated-cursor guards, and explicit traversal-admission helpers.
- `src/core/time.rs`: shared `TimeInput` parsing and UTC-ms normalization logic.

---

## `src/streaming`

### Artifacts

- Inventory: `src/streaming/docs/inventory.md`

### Source Files

- `src/streaming/make_before_break.rs`: shared make-before-break validation-window config.
- `src/streaming/mod.rs`: streaming module wiring and exports for shared WS coordination helpers.
- `src/streaming/replay.rs`: shared replay placeholder module for future reconnect and gap-handling coordination.
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
- `src/systems/mod.rs`: top-level system module wiring and exports.
- `src/systems/types.rs`: shared cross-system public enums and wire-facing labels.

---

## `src/tests`

### Artifacts

- Inventory: `src/tests/docs/inventory.md`

### Source Files

- `src/tests/contract/mod.rs`: contract-test module wiring.
- `src/tests/contract/test_aggregator_public_grpc_v6_latest.rs`: contract tests for aggregator gRPC latest bars.
- `src/tests/contract/test_aggregator_public_grpc_v7_range.rs`: contract tests for aggregator gRPC range bars.
- `src/tests/contract/test_aggregator_public_grpc_v8_search.rs`: contract tests for aggregator gRPC search bars.
- `src/tests/contract/test_aggregator_public_grpc_v9_time_machine.rs`: contract tests for aggregator gRPC time-machine bars.
- `src/tests/contract/test_aggregator_public_http_v0.rs`: contract tests for aggregator HTTP latest bars and docs base behavior.
- `src/tests/contract/test_aggregator_public_http_v1_simple_discovery.rs`: contract tests for aggregator HTTP simple discovery and pairs surfaces.
- `src/tests/contract/test_aggregator_public_http_v2_files_downloads.rs`: contract tests for aggregator HTTP file downloads.
- `src/tests/contract/test_aggregator_public_http_v3_bars_range.rs`: contract tests for aggregator HTTP bars range.
- `src/tests/contract/test_aggregator_public_http_v4_bars_search.rs`: contract tests for aggregator HTTP bars search.
- `src/tests/contract/test_aggregator_public_http_v5_bars_time_machine.rs`: contract tests for aggregator HTTP bars time-machine.
- `src/tests/contract/test_aggregator_public_ws_v10_bars.rs`: contract tests for aggregator WS bars, make-before-break, and managed recovery.
- `src/tests/contract/test_aggregator_public_ws_v11_messages.rs`: contract tests for aggregator WS messages and managed recovery.
- `src/tests/contract/test_core_pagination.rs`: contract tests for the shared pagination state machine and explicit traversal-admission guards.
- `src/tests/contract/test_core_time.rs`: contract tests for shared time parsing and normalization.
- `src/tests/contract/test_systems_helpers.rs`: contract tests for shared systems helper collectors.
- `src/tests/contract/test_transport_grpc.rs`: contract tests for shared gRPC transport behavior.
- `src/tests/integration/mod.rs`: integration-test module wiring placeholder.
- `src/tests/mod.rs`: top-level SDK test module wiring.

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
