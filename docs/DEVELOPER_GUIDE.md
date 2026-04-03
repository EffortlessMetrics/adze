# Developer Guide - Adze

> **Doc status:** Up to date for Adze 0.8.0-dev.

## Prerequisites

### System Requirements
- **Rust 1.92.0+** (2024 edition support)
- **Node.js**: Required for `tree-sitter` CLI compatibility (legacy/C backend only)
- **C Compiler**: Required for `tree-sitter` C integration
- **just**: Command runner (optional but recommended)

## Maintenance Lanes

Adze uses a "Support Lane" model to keep the core green while allowing experimental features to evolve.

### 🟢 Supported Lane (Must be Green)
These crates are the core product. CI enforces passing tests and lints on every PR.
- `adze` (core runtime)
- `adze-macro`
- `adze-tool`
- `adze-common`
- `adze-ir`
- `adze-glr-core`
- `adze-tablegen`

### 🟡 Experimental/Community Lane (Best Effort)
These crates are useful but may break during major refactors.
- `grammars/*` (Python, JS, Go examples)
- `example/` (Arithmetic demo)
- `runtime2` (alternative runtime path)
- `cli/`
- `playground/`

To run the supported gate locally:
```bash
just ci-supported
```

## Quick Commands

### Core Development
```bash
# Run tests for core crates only (fast)
just test

# Run strict linting
just clippy

# Format code
just fmt
```

### Full Workspace
```bash
# Build everything (including experimental)
cargo build --workspace

# Run all tests (may require heavy resources)
cargo test --workspace
```

### Grammar Development
```bash
# Build a specific grammar
cargo build -p adze-python

# Snapshot testing
cargo test -p adze-example
cargo insta review
```

### Debugging
If you need to inspect generated parsers:
```bash
export ADZE_EMIT_ARTIFACTS=true
cargo build -p adze-example
# Check target/debug/build/*/out/grammar_*/
```

## Release Process

1. **Verify State**: Ensure `just ci-supported` passes.
2. **Update Docs**: Check [`docs/status/FRICTION_LOG.md`](./status/FRICTION_LOG.md) and [`CHANGELOG.md`](../CHANGELOG.md).
3. **Bump Version**: Update `version` in `Cargo.toml` files (workspace members).
4. **Tag**: `git tag v0.8.0`
5. **Publish**: `cargo publish` (scripted in CI).
6. **Release surface configuration**: choose `RELEASE_SURFACE_MODE` (`fixed`/`auto`) and optional `RELEASE_CRATE_FILE` override as needed.
7. **Release surface strictness**: decide whether to run `strict_publish_surface` (fixed mode only) in the GitHub Release workflow when publishing, or `STRICT_PUBLISH_SURFACE=true` for local helper runs.
8. Optionally set workflow dispatch inputs `release_surface_mode` and `release_crate_file` for one-off releases.

## Code Standards

- **Formatting**: `rustfmt` is enforced.
- **Lints**: `clippy` warnings are errors in the supported lane.
- **Safety**: Unsafe code must be documented with `// SAFETY:` comments.
- **Testing**: New features must have corresponding tests in `tests/` or unit tests.

## Troubleshooting

### "Too many open files" during tests
The full workspace test suite opens many files. Increase your ulimit or run tests per-crate.

### "Memory limit exceeded"
The GLR table generation can be memory intensive for huge grammars. Try `cargo test --release` to use optimized table generation.
