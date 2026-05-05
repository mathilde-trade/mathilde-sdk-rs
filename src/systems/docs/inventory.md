# `src/systems` — Inventory (DRAFT)

Protocol: `.dev/specs/SDK_INVENTORY_SYSTEM_SPEC_2026-04-08.md`

This inventory describes only implemented module-level code and artifacts that
currently exist for `src/systems`.

---

## 0) Artifacts

- Inventory (this file): `src/systems/docs/inventory.md`

## 1) Source Files

- `src/systems/helpers.rs`: shared public collection helpers for system-facing request ergonomics.
- `src/systems/mod.rs`: top-level system module wiring and exports.
- `src/systems/types.rs`: shared cross-system public enums and wire-facing labels.
- `src/systems/aggregator/mod.rs`: aggregator system module wiring and public exports.
- `src/systems/aggregator/client.rs`: typed public client entrypoints for aggregator HTTP, gRPC, and WS surfaces.
- `src/systems/aggregator/docs.rs`: aggregator public documentation and OpenAPI bindings.
- `src/systems/aggregator/pairs.rs`: aggregator public pairs discovery and status bindings.
- `src/systems/aggregator/files.rs`: aggregator public file-download issuance bindings and typed local-download convenience.
- `src/systems/aggregator/bars_http.rs`: aggregator public bars HTTP bindings.
- `src/systems/aggregator/bars_grpc.rs`: aggregator public bars gRPC bindings.
- `src/systems/aggregator/bars_pagination.rs`: aggregator explicit call-wrapper, pager, and traverse helpers layered on the one-page bars bindings.
- `src/systems/aggregator/bars_ws.rs`: aggregator public bars WS bindings, make-before-break, and managed recovery.
- `src/systems/aggregator/messages_ws.rs`: aggregator public messages WS bindings and managed recovery.
- `src/systems/aggregator/types.rs`: aggregator-specific request, response, traversal-result, and WS frame types.
- `src/systems/intro/mod.rs`: intro system module wiring and public exports.
- `src/systems/intro/client.rs`: typed public client entrypoint for the dedicated intro root surface on `api.mathilde.dev`.
- `src/systems/intro/intro.rs`: intro root HTTP binding that calls the host root and decodes the ordered JSON intro document.
- `src/systems/primitives/mod.rs`: primitives system module wiring, generated contract re-exports, and public exports.
- `src/systems/primitives/client.rs`: typed public client entrypoints for primitives HTTP, gRPC, and WS surfaces.
- `src/systems/primitives/docs.rs`: primitives public documentation and OpenAPI bindings.
- `src/systems/primitives/pairs.rs`: primitives public pairs discovery and status bindings.
- `src/systems/primitives/files.rs`: primitives public file-download issuance bindings and typed local-download convenience.
- `src/systems/primitives/outputs_http.rs`: primitives public outputs HTTP bindings.
- `src/systems/primitives/outputs_grpc.rs`: primitives public outputs gRPC bindings.
- `src/systems/primitives/outputs_pagination.rs`: primitives explicit call-wrapper, pager, and traverse helpers layered on the one-page outputs bindings.
- `src/systems/primitives/outputs_ws.rs`: primitives public outputs WS bindings, make-before-break, and managed recovery.
- `src/systems/primitives/messages_ws.rs`: primitives public messages WS bindings and managed recovery.
- `src/systems/primitives/types.rs`: primitives-specific request, response, traversal-result, and WS frame types.
- `src/systems/regime/mod.rs`: regime system module wiring, generated contract re-exports, and public exports.
- `src/systems/regime/client.rs`: typed public client entrypoints for regime HTTP, gRPC, and WS surfaces.
- `src/systems/regime/docs.rs`: regime public documentation and OpenAPI bindings.
- `src/systems/regime/pairs.rs`: regime public pairs discovery and status bindings.
- `src/systems/regime/files.rs`: regime public file-download issuance bindings and typed local-download convenience.
- `src/systems/regime/outputs_http.rs`: regime public outputs HTTP bindings.
- `src/systems/regime/outputs_grpc.rs`: regime public outputs gRPC bindings.
- `src/systems/regime/outputs_pagination.rs`: regime explicit call-wrapper, pager, and traverse helpers layered on the one-page outputs bindings.
- `src/systems/regime/outputs_ws.rs`: regime public outputs WS bindings, make-before-break, and managed recovery.
- `src/systems/regime/messages_ws.rs`: regime public messages WS bindings and managed recovery.
- `src/systems/regime/types.rs`: regime-specific request, response, traversal-result, and WS frame types.
