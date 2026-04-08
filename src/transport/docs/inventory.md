# `src/transport` — Inventory (DRAFT)

Protocol: `.dev/specs/SDK_INVENTORY_SYSTEM_SPEC_2026-04-08.md`

This inventory describes only implemented module-level code and artifacts that
currently exist for `src/transport`.

---

## 0) Artifacts

- Inventory (this file): `src/transport/docs/inventory.md`

## 1) Source Files

- `src/transport/mod.rs`: transport module wiring and exports for HTTP, gRPC, and WS helpers.
- `src/transport/http.rs`: shared HTTP transport wrapper and auth-aware request builder.
- `src/transport/grpc.rs`: shared gRPC transport wrapper, channel handling, and bearer metadata injection.
- `src/transport/ws.rs`: shared WS upgrade URL normalization and bearer-auth upgrade headers.
