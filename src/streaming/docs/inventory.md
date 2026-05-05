# `src/streaming` — Inventory (DRAFT)

Protocol: `.dev/specs/SDK_INVENTORY_SYSTEM_SPEC_2026-05-05.md`

This inventory describes only implemented module-level code and artifacts that
currently exist for `src/streaming`.

---

## 0) Artifacts

- Inventory (this file): `src/streaming/docs/inventory.md`

## 1) Source Files

- `src/streaming/mod.rs`: streaming module wiring and exports for shared WS coordination helpers.
- `src/streaming/make_before_break.rs`: shared make-before-break validation-window config.
- `src/streaming/subscription.rs`: shared WS recovery backoff config and reconnect state primitives.
