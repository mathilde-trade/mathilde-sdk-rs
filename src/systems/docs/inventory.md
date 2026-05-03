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
