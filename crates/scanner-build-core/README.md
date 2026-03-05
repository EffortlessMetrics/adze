# adze-scanner-build-core

Core SRP microcrate for build-time external scanner discovery and registration code generation.

This crate owns:

- scanner source discovery (`scanner.c`, `scanner.cc/.cpp`, `scanner.rs`, and named variants)
- build integration for C/C++ scanners via `cc`
- Rust registration stub generation for Rust scanners
- a `build_scanner()` helper for `build.rs` wiring

It is re-exported by `adze-tool` as `adze_tool::scanner_build` for backward compatibility.
