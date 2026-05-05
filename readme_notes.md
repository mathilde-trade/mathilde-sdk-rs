# README Notes

Deferred README updates from the runtime-performance remediation:

- Add guidance near the `range`, `search`, and `time_machine` traversal examples
  that `traverse()` materializes all fetched pages in memory.
- Add guidance near the same examples that `pager()` is the lower-memory
  streaming path for large windows.
- Add a short note in traversal examples that large windows should prefer an
  explicit `close_end` and a sensible `limit` to avoid excessive page counts.

Intentional constraint:

- `README.md` was not modified during this remediation because README edits were
  explicitly deferred by the user.
