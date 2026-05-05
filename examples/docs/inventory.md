# `examples` — Inventory (DRAFT)

Protocol: `.dev/specs/SDK_INVENTORY_SYSTEM_SPEC_2026-05-05.md`

This inventory describes only implemented runnable example code and artifacts
that currently exist for `examples`.

---

## 0) Artifacts

- Inventory (this file): `examples/docs/inventory.md`

## 1) Source Files

- `examples/documenation.rs`: public intro and documentation navigation example across aggregator, primitives, and regime.
- `examples/files.rs`: bounded file-listing and explicit selected-download example across all three systems.
- `examples/latest.rs`: latest closed-read example showing bars versus computed rows and getter-based processor field access.
- `examples/range.rs`: bounded range and bounded traverse example for fixed bars and computed outputs.
- `examples/search.rs`: hit-discovery example across all three systems, including reusable hit-only mode and evaluated-row mode.
- `examples/status.rs`: pairs list and readiness-status example across aggregator, primitives, and regime.
- `examples/time_machine.rs`: one-pass replay-context example using `predicate`, `before_bars`, `after_bars`, and offsets.
- `examples/transport.rs`: transport-equivalence example for one aggregator latest request over HTTP JSON, HTTP protobuf, and gRPC.
- `examples/ws.rs`: bounded WebSocket example across all three systems plus full make-before-break promotion proof for aggregator bars.
- `examples/workflows/bounded_recent_window.rs`: workflow example for aligning a bounded recent aggregator window with primitives computed rows.
- `examples/workflows/current_downside_state.rs`: workflow example for discovering downside-state hits in primitives and replaying shared hit timestamps in primitives and aggregator.
- `examples/workflows/current_grouped_regime_state.rs`: workflow example for current grouped regime-state discovery and direct regime replay context.
- `examples/workflows/measured_local_stress_context.rs`: workflow example for measured local stress discovery through primitives taxonomy, registry, search, and replay context.
- `examples/workflows/reproducible_monthly_research.rs`: workflow example for reproducible monthly slice downloads and local DuckDB joins.
- `examples/workflows/understanding_system.rs`: workflow example that mirrors the authoritative intro understanding workflow across intro, docs, registry, endpoints, and OpenAPI.
