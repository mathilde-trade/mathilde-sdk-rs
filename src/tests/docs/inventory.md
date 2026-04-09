# `src/tests` — Inventory (DRAFT)

Protocol: `.dev/specs/SDK_INVENTORY_SYSTEM_SPEC_2026-04-08.md`

This inventory describes only implemented module-level code and artifacts that
currently exist for `src/tests`.

---

## 0) Artifacts

- Inventory (this file): `src/tests/docs/inventory.md`

## 1) Source Files

- `src/tests/mod.rs`: top-level SDK test module wiring.
- `src/tests/contract/mod.rs`: contract-test module wiring.
- `src/tests/contract/test_aggregator_public_http_v0.rs`: contract tests for aggregator HTTP latest bars and docs base behavior.
- `src/tests/contract/test_aggregator_public_http_v1_simple_discovery.rs`: contract tests for aggregator HTTP simple discovery and pairs surfaces.
- `src/tests/contract/test_aggregator_public_http_v2_files_downloads.rs`: contract tests for aggregator HTTP file downloads.
- `src/tests/contract/test_aggregator_public_http_v3_bars_range.rs`: contract tests for aggregator HTTP bars range.
- `src/tests/contract/test_aggregator_public_http_v4_bars_search.rs`: contract tests for aggregator HTTP bars search.
- `src/tests/contract/test_aggregator_public_http_v5_bars_time_machine.rs`: contract tests for aggregator HTTP bars time-machine.
- `src/tests/contract/test_aggregator_public_grpc_v6_latest.rs`: contract tests for aggregator gRPC latest bars.
- `src/tests/contract/test_aggregator_public_grpc_v7_range.rs`: contract tests for aggregator gRPC range bars.
- `src/tests/contract/test_aggregator_public_grpc_v8_search.rs`: contract tests for aggregator gRPC search bars.
- `src/tests/contract/test_aggregator_public_grpc_v9_time_machine.rs`: contract tests for aggregator gRPC time-machine bars.
- `src/tests/contract/test_aggregator_public_ws_v10_bars.rs`: contract tests for aggregator WS bars, make-before-break, and managed recovery.
- `src/tests/contract/test_aggregator_public_ws_v11_messages.rs`: contract tests for aggregator WS messages and managed recovery.
- `src/tests/contract/test_core_time.rs`: contract tests for shared time parsing and normalization.
- `src/tests/contract/test_systems_helpers.rs`: contract tests for shared systems helper collectors.
- `src/tests/contract/test_transport_grpc.rs`: contract tests for shared gRPC transport behavior.
- `src/tests/integration/mod.rs`: integration-test module wiring placeholder.
