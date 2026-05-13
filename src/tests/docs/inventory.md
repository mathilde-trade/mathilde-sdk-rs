# `src/tests` — Inventory (DRAFT)

Protocol: `.dev/specs/SDK_INVENTORY_SYSTEM_SPEC_2026-05-05.md`

This inventory describes only implemented module-level code and artifacts that
currently exist for `src/tests`.

---

## 0) Artifacts

- Inventory (this file): `src/tests/docs/inventory.md`

## 1) Source Files

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
