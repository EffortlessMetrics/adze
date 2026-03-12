# adze-parser-feature-profile-core

Single-responsibility microcrate for parser feature-profile state and backend
resolution policy.

This crate provides:

- `ParserFeatureProfile`: snapshot of feature flags (`pure-rust`, tree-sitter variants, `glr`)
- `ParserFeatureProfile::resolve_backend`: deterministic backend selection
- `Display` formatting for profile diagnostics
- `ParserBackend` re-export for ergonomic consumers
