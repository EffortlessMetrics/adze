# Runtime2 status (bounded)

**Last updated:** 2026-04-26

## Recommended support tier

`runtime2/` is currently an **experimental proving ground**.

It is intentionally outside the supported PR gate (`just ci-supported`) and should be treated as a place to iterate on runtime APIs and GLR runtime mechanics, not as the default stable runtime contract for users.

## What is explicitly proven right now

The runtime2 smoke test `smoke_language_builder_constructs_minimal_language` proves one bounded behavior:

- a minimal `Language` can be constructed via `Language::builder()` with parse table + metadata,
- symbol/field metadata can be queried from that object (`symbol_name`, `field_name`, `is_visible`).

This is a construction/API sanity proof only; it does **not** claim parser correctness, full forest disambiguation guarantees, or production-readiness.

## Scope boundary

This status does **not** promote runtime2 to stable.

Graduation from experimental should require:
- inclusion in a required CI lane,
- a small, explicitly documented contract suite (parser + tree semantics),
- sustained green results over normal development cadence.
