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
- `src/tests/contract/test_aggregator_public_http_latest.rs`: contract tests for aggregator HTTP latest bars and docs base behavior.
- `src/tests/contract/test_aggregator_public_http_simple_discovery.rs`: contract tests for aggregator HTTP simple discovery and pairs surfaces.
- `src/tests/contract/test_aggregator_public_http_files_downloads.rs`: contract tests for aggregator HTTP file downloads.
- `src/tests/contract/test_aggregator_public_http_range.rs`: contract tests for aggregator HTTP bars range.
- `src/tests/contract/test_aggregator_public_http_search.rs`: contract tests for aggregator HTTP bars search.
- `src/tests/contract/test_aggregator_public_http_time_machine.rs`: contract tests for aggregator HTTP bars time-machine.
- `src/tests/contract/test_aggregator_public_grpc_latest.rs`: contract tests for aggregator gRPC latest bars.
- `src/tests/contract/test_aggregator_public_grpc_range.rs`: contract tests for aggregator gRPC range bars.
- `src/tests/contract/test_aggregator_public_grpc_search.rs`: contract tests for aggregator gRPC search bars.
- `src/tests/contract/test_aggregator_public_grpc_time_machine.rs`: contract tests for aggregator gRPC time-machine bars.
- `src/tests/contract/test_aggregator_public_ws_bars.rs`: contract tests for aggregator WS bars, make-before-break, and managed recovery.
- `src/tests/contract/test_aggregator_public_ws_messages.rs`: contract tests for aggregator WS messages and managed recovery.
- `src/tests/contract/test_primitives_public_http_latest.rs`: contract tests for primitives HTTP latest outputs, docs base behavior, and public-default config construction.
- `src/tests/contract/test_primitives_public_http_simple_discovery.rs`: contract tests for primitives HTTP docs, registry selector serialization, openapi, and pairs discovery.
- `src/tests/contract/test_primitives_public_http_files_downloads.rs`: contract tests for primitives HTTP file downloads and authenticated local download behavior.
- `src/tests/contract/test_primitives_public_http_range.rs`: contract tests for primitives HTTP range outputs, including projected JSON decode and projected protobuf fail-closed behavior.
- `src/tests/contract/test_primitives_public_http_search.rs`: contract tests for primitives HTTP search outputs with typed metadata payloads.
- `src/tests/contract/test_primitives_public_http_time_machine.rs`: contract tests for primitives HTTP time-machine outputs with projected metadata payloads.
- `src/tests/contract/test_primitives_public_grpc_latest.rs`: contract tests for primitives gRPC latest outputs request mapping and min-response decode.
- `src/tests/contract/test_primitives_public_grpc_range.rs`: contract tests for primitives gRPC range outputs request mapping and metadata response decode.
- `src/tests/contract/test_primitives_public_grpc_search.rs`: contract tests for primitives gRPC search outputs request mapping and evaluated-rows decode.
- `src/tests/contract/test_primitives_public_grpc_time_machine.rs`: contract tests for primitives gRPC time-machine outputs request mapping and response decode.
- `src/tests/contract/test_primitives_public_ws_outputs.rs`: contract tests for primitives outputs WS subscribe wiring and JSON row decode.
- `src/tests/contract/test_primitives_public_ws_messages.rs`: contract tests for primitives messages WS control frames and recovering subscription replay.
- `src/tests/contract/test_primitives_types.rs`: relocated primitives type, selector, and proto decode tests previously kept inline in runtime source files.
- `src/tests/contract/test_primitives_outputs_http.rs`: relocated primitives HTTP fail-closed projected protobuf contract tests.
- `src/tests/contract/test_primitives_outputs_grpc.rs`: relocated primitives gRPC fail-closed projected selector contract tests.
- `src/tests/contract/test_primitives_outputs_ws.rs`: relocated primitives outputs WS projected protobuf fail-closed contract tests.
- `src/tests/contract/test_primitives_outputs_pagination.rs`: relocated primitives pagination admission tests for range, search, and time-machine calls.
- `src/tests/contract/test_primitives_docs.rs`: relocated primitives docs selector serialization tests.
- `src/tests/contract/test_core_pagination.rs`: contract tests for the shared pagination state machine and explicit traversal-admission guards.
- `src/tests/contract/test_core_time.rs`: contract tests for shared time parsing and normalization.
- `src/tests/contract/test_systems_helpers.rs`: contract tests for shared systems helper collectors.
- `src/tests/contract/test_transport_grpc.rs`: contract tests for shared gRPC transport behavior.
- `src/tests/integration/mod.rs`: integration-test module wiring placeholder.
