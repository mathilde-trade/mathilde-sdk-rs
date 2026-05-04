# `mathilde-sdk-rs` — Global Inventory (GENERATED; DO NOT EDIT)

Generated: 2026-05-04T11:58:01Z
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
- `src/systems/intro/client.rs`: typed public client entrypoint for the dedicated intro root surface on `api.mathilde.dev`.
- `src/systems/intro/intro.rs`: intro root HTTP binding that calls the host root and decodes the ordered JSON intro document.
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

- `src/tests/contract/mod.rs`: contract-test module wiring.
- `src/tests/contract/test_aggregator_public_grpc_latest.rs`: contract tests for aggregator gRPC latest bars.
- `src/tests/contract/test_aggregator_public_grpc_range.rs`: contract tests for aggregator gRPC range bars.
- `src/tests/contract/test_aggregator_public_grpc_search.rs`: contract tests for aggregator gRPC search bars.
- `src/tests/contract/test_aggregator_public_grpc_time_machine.rs`: contract tests for aggregator gRPC time-machine bars.
- `src/tests/contract/test_aggregator_public_http_files_downloads.rs`: contract tests for aggregator HTTP file downloads.
- `src/tests/contract/test_aggregator_public_http_latest.rs`: contract tests for aggregator HTTP latest bars and docs base behavior.
- `src/tests/contract/test_aggregator_public_http_range.rs`: contract tests for aggregator HTTP bars range.
- `src/tests/contract/test_aggregator_public_http_search.rs`: contract tests for aggregator HTTP bars search.
- `src/tests/contract/test_aggregator_public_http_simple_discovery.rs`: contract tests for aggregator HTTP simple discovery and pairs surfaces.
- `src/tests/contract/test_aggregator_public_http_time_machine.rs`: contract tests for aggregator HTTP bars time-machine.
- `src/tests/contract/test_aggregator_public_ws_bars.rs`: contract tests for aggregator WS bars, make-before-break, and managed recovery.
- `src/tests/contract/test_aggregator_public_ws_messages.rs`: contract tests for aggregator WS messages and managed recovery.
- `src/tests/contract/test_core_pagination.rs`: contract tests for the shared pagination state machine and explicit traversal-admission guards.
- `src/tests/contract/test_core_time.rs`: contract tests for shared time parsing and normalization.
- `src/tests/contract/test_intro_public_http_intro.rs`: contract tests for the dedicated intro root system on `https://api.mathilde.dev`, including root-path request behavior and ordered JSON preservation.
- `src/tests/contract/test_primitives_docs.rs`: relocated primitives docs selector serialization tests.
- `src/tests/contract/test_primitives_outputs_grpc.rs`: relocated primitives gRPC fail-closed projected selector contract tests.
- `src/tests/contract/test_primitives_outputs_http.rs`: relocated primitives HTTP fail-closed projected protobuf contract tests.
- `src/tests/contract/test_primitives_outputs_pagination.rs`: relocated primitives pagination admission tests for range, search, and time-machine calls.
- `src/tests/contract/test_primitives_outputs_ws.rs`: relocated primitives outputs WS projected protobuf fail-closed contract tests.
- `src/tests/contract/test_primitives_public_grpc_latest.rs`: contract tests for primitives gRPC latest outputs request mapping and min-response decode.
- `src/tests/contract/test_primitives_public_grpc_range.rs`: contract tests for primitives gRPC range outputs request mapping and metadata response decode.
- `src/tests/contract/test_primitives_public_grpc_search.rs`: contract tests for primitives gRPC search outputs request mapping and evaluated-rows decode.
- `src/tests/contract/test_primitives_public_grpc_time_machine.rs`: contract tests for primitives gRPC time-machine outputs request mapping and response decode.
- `src/tests/contract/test_primitives_public_http_files_downloads.rs`: contract tests for primitives HTTP file downloads and authenticated local download behavior.
- `src/tests/contract/test_primitives_public_http_latest.rs`: contract tests for primitives HTTP latest outputs, docs base behavior, and public-default config construction.
- `src/tests/contract/test_primitives_public_http_range.rs`: contract tests for primitives HTTP range outputs, including projected JSON decode and projected protobuf fail-closed behavior.
- `src/tests/contract/test_primitives_public_http_search.rs`: contract tests for primitives HTTP search outputs with typed metadata payloads.
- `src/tests/contract/test_primitives_public_http_simple_discovery.rs`: contract tests for primitives HTTP docs, registry selector serialization, openapi, and pairs discovery.
- `src/tests/contract/test_primitives_public_http_time_machine.rs`: contract tests for primitives HTTP time-machine outputs with projected metadata payloads.
- `src/tests/contract/test_primitives_public_ws_messages.rs`: contract tests for primitives messages WS control frames and recovering subscription replay.
- `src/tests/contract/test_primitives_public_ws_outputs.rs`: contract tests for primitives outputs WS subscribe wiring and JSON row decode.
- `src/tests/contract/test_primitives_types.rs`: relocated primitives type, selector, and proto decode tests previously kept inline in runtime source files.
- `src/tests/contract/test_regime_docs.rs`: relocated regime docs selector serialization tests.
- `src/tests/contract/test_regime_outputs_grpc.rs`: relocated regime gRPC fail-closed projected selector and non-`1h` contract tests.
- `src/tests/contract/test_regime_outputs_http.rs`: relocated regime HTTP fail-closed projected protobuf and non-`1h` contract tests.
- `src/tests/contract/test_regime_outputs_pagination.rs`: relocated regime pagination admission tests for range, search, and time-machine calls.
- `src/tests/contract/test_regime_outputs_ws.rs`: relocated regime outputs WS projected protobuf and non-`1h` fail-closed contract tests.
- `src/tests/contract/test_regime_public_grpc_latest.rs`: contract tests for regime gRPC latest outputs request mapping and min-response decode.
- `src/tests/contract/test_regime_public_grpc_range.rs`: contract tests for regime gRPC range outputs request mapping and metadata response decode.
- `src/tests/contract/test_regime_public_grpc_search.rs`: contract tests for regime gRPC search outputs request mapping and evaluated-rows decode.
- `src/tests/contract/test_regime_public_grpc_time_machine.rs`: contract tests for regime gRPC time-machine outputs request mapping and response decode.
- `src/tests/contract/test_regime_public_http_files_downloads.rs`: contract tests for regime HTTP file downloads and authenticated local download behavior.
- `src/tests/contract/test_regime_public_http_latest.rs`: contract tests for regime HTTP latest outputs, docs base behavior, and public-default config construction.
- `src/tests/contract/test_regime_public_http_range.rs`: contract tests for regime HTTP range outputs, including projected JSON decode and projected protobuf fail-closed behavior.
- `src/tests/contract/test_regime_public_http_search.rs`: contract tests for regime HTTP search outputs with typed metadata payloads.
- `src/tests/contract/test_regime_public_http_simple_discovery.rs`: contract tests for regime HTTP docs, registry selector serialization, openapi, and pairs discovery.
- `src/tests/contract/test_regime_public_http_time_machine.rs`: contract tests for regime HTTP time-machine outputs with projected metadata payloads.
- `src/tests/contract/test_regime_public_ws_messages.rs`: contract tests for regime messages WS control frames and recovering subscription replay.
- `src/tests/contract/test_regime_public_ws_outputs.rs`: contract tests for regime outputs WS subscribe wiring, `secondary`, and JSON row decode.
- `src/tests/contract/test_regime_types.rs`: relocated regime type, selector, timeframe, and proto decode tests.
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
