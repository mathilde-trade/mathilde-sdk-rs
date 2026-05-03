# `src/core` — Inventory (DRAFT)

Protocol: `.dev/specs/SDK_INVENTORY_SYSTEM_SPEC_2026-04-08.md`

This inventory describes only implemented module-level code and artifacts that
currently exist for `src/core`.

---

## 0) Artifacts

- Inventory (this file): `src/core/docs/inventory.md`

## 1) Source Files

- `src/core/mod.rs`: core module wiring and exports for shared SDK primitives.
- `src/core/auth.rs`: bearer-token validation and HTTP auth-header helper.
- `src/core/client.rs`: shared base client helpers for HTTP request execution.
- `src/core/config.rs`: typed SDK transport configuration surfaces and builders.
- `src/core/error.rs`: shared typed SDK error surface across transports and endpoint families.
- `src/core/pagination.rs`: shared pagination state-machine primitives, repeated-cursor guards, and explicit traversal-admission helpers.
- `src/core/time.rs`: shared `TimeInput` parsing and UTC-ms normalization logic.
