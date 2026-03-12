# adze-common-type-ops-core

Single-responsibility helpers for transforming containerized Rust `syn::Type` values.

This microcrate owns type-centric operations that were previously mixed into
`adze-common-syntax-core`:

- `try_extract_inner_type`
- `filter_inner_type`
- `wrap_leaf_type`

`adze-common-syntax-core` now acts as the syntax-parsing entrypoint and re-exports
these helpers for compatibility.
