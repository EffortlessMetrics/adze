# Adze Quick Reference

**Last updated:** 2026-03-13 | **Version:** 0.8.0-dev (RC Quality) | **MSRV:** 1.92.0

---

## Project Status at a Glance

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Version | 0.8.0-dev | 0.8.0 | 🟡 RC Quality |
| Test Count | 2,460+ | 2,500+ | 🟡 Near target |
| Feature Matrix | 11/12 pass | 12/12 pass | 🟡 1 expected failure |
| API Stability | 54 stable APIs | 100+ APIs | 🟡 Expanding |
| Clippy / Fmt | ✅ Clean | ✅ Clean | 🟢 Pass |
| Security Audit | 0 vulns | 0 vulns | 🟢 Pass |
| WASM Compatibility | ✅ | ✅ | 🟢 Pass |

---

## Essential Commands

### PR Gate (MUST PASS)

```bash
just ci-supported
```

### Development

| Task | Command |
|------|---------|
| Build all | `cargo build` |
| Build crate | `cargo build -p adze` |
| Test core | `just test` |
| Test all | `cargo t2` (2 threads) |
| Test crate | `cargo test -p <crate>` |
| Format check | `just fmt` |
| Clippy | `just clippy` |
| Format fix | `cargo fmt --all` |

### Snapshots

| Task | Command |
|------|---------|
| Review snapshots | `cargo insta review` |
| Quick review | `just snap` |

### Advanced

| Task | Command |
|------|---------|
| Feature matrix | `just matrix` |
| Mutation testing | `just mutate` |
| MSRV check | `just check-msrv` |

---

## Core Pipeline Crates

| Crate | Path | Purpose |
|-------|------|---------|
| `adze` | `runtime/` | Main runtime, Extract trait |
| `adze-macro` | `macro/` | Proc-macro attributes |
| `adze-tool` | `tool/` | Build-time code generation |
| `adze-common` | `common/` | Shared grammar expansion |
| `adze-ir` | `ir/` | Grammar IR with GLR support |
| `adze-glr-core` | `glr-core/` | GLR parser generation |
| `adze-tablegen` | `tablegen/` | Table compression, FFI |

---

## Key Links

### Documentation

| Resource | Link |
|----------|------|
| Master Index | [docs/INDEX.md](./INDEX.md) |
| Navigation Guide | [docs/NAVIGATION.md](./NAVIGATION.md) |
| Getting Started | [tutorials/getting-started.md](./tutorials/getting-started.md) |
| Grammar Author's Guide | [guides/GRAMMAR_AUTHORS_GUIDE.md](./guides/GRAMMAR_AUTHORS_GUIDE.md) |
| Integration Guide | [guides/INTEGRATION_GUIDE.md](./guides/INTEGRATION_GUIDE.md) |
| API Reference | [reference/api.md](./reference/api.md) |
| Architecture | [explanations/architecture.md](./explanations/architecture.md) |
| Testing Guide | [testing/TESTING_GUIDE.md](./testing/TESTING_GUIDE.md) |
| Contributor Guide | [contributing/CONTRIBUTOR_GUIDE.md](./contributing/CONTRIBUTOR_GUIDE.md) |

### Status & Planning

| Resource | Link |
|----------|------|
| Current Priorities | [status/NOW_NEXT_LATER.md](./status/NOW_NEXT_LATER.md) |
| Technical Roadmap | [roadmap/TECHNICAL_ROADMAP.md](./roadmap/TECHNICAL_ROADMAP.md) |
| Vision & Strategy | [vision/VISION_AND_STRATEGY.md](./vision/VISION_AND_STRATEGY.md) |
| API Stability | [status/API_STABILITY.md](./status/API_STABILITY.md) |
| Friction Log | [status/FRICTION_LOG.md](./status/FRICTION_LOG.md) |

### Architecture

| Resource | Link |
|----------|------|
| ADR Index | [adr/INDEX.md](./adr/INDEX.md) |
| GLR Internals | [explanations/glr-internals.md](./explanations/glr-internals.md) |
| Incremental Theory | [explanations/incremental-parsing-theory.md](./explanations/incremental-parsing-theory.md) |

### Root Files

| Resource | Link |
|----------|------|
| Project Conventions | [AGENTS.md](../AGENTS.md) |
| FAQ | [FAQ.md](../FAQ.md) |
| Contributing | [CONTRIBUTING.md](../CONTRIBUTING.md) |

---

## Workspace Dependencies

```toml
# Common workspace dependencies (use { workspace = true })
tree-sitter, serde, serde_json, proptest, insta, criterion
thiserror, syn, quote, proc-macro2, anyhow, tempfile
indexmap, bincode, clap, rayon, rustc-hash, smallvec, regex
```

---

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `RUST_TEST_THREADS` | 2 | Test concurrency |
| `RAYON_NUM_THREADS` | 4 | Rayon thread pool |
| `CARGO_BUILD_JOBS` | 4 (CI: 2) | Parallel builds |
| `TOKIO_WORKER_THREADS` | 2 | Tokio workers |
| `ADZE_EMIT_ARTIFACTS` | unset | Output grammar files |
| `ADZE_LOG_PERFORMANCE` | unset | GLR performance logs |

---

## Verification Checklist

Before submitting PR:

```bash
# 1. Format
cargo fmt --all

# 2. Lint
just clippy

# 3. Test
just test

# 4. Full PR gate
just ci-supported

# 5. Snapshots (if changed)
cargo insta review
```

---

## Architecture Decision Records

| ADR | Title |
|-----|-------|
| [001](./adr/001-pure-rust-glr-implementation.md) | Pure-Rust GLR Implementation |
| [002](./adr/002-workspace-structure.md) | Workspace Structure |
| [003](./adr/003-dual-runtime-strategy.md) | Dual Runtime Strategy |
| [004](./adr/004-grammar-definition-via-macros.md) | Grammar Definition via Macros |
| [005](./adr/005-incremental-parsing-architecture.md) | Incremental Parsing Architecture |
| [006](./adr/006-tree-sitter-compatibility-layer.md) | Tree-sitter Compatibility Layer |
| [007](./adr/007-bdd-framework-for-parser-testing.md) | BDD Framework for Parser Testing |
| [008](./adr/008-governance-microcrates-architecture.md) | Governance Microcrates Architecture |
| [009](./adr/009-symbol-registry-unification.md) | Symbol Registry Unification |
| [010](./adr/010-external-scanner-architecture.md) | External Scanner Architecture |
| [011](./adr/011-parse-table-binary-format.md) | Parse Table Binary Format |
| [012](./adr/012-performance-baseline-management.md) | Performance Baseline Management |
| [013](./adr/013-gss-implementation-strategy.md) | GSS Implementation Strategy |
| [014](./adr/014-parse-table-compression-strategy.md) | Parse Table Compression Strategy |
| [015](./adr/015-disambiguation-strategy.md) | Disambiguation Strategy |
| [016](./adr/016-error-handling-strategy.md) | Error Handling Strategy |
| [017](./adr/017-memory-management-strategy.md) | Memory Management Strategy |
| [018](./adr/018-grammar-optimization-pipeline.md) | Grammar Optimization Pipeline |

See [ADR Index](./adr/INDEX.md) for complete list and details.

---

## Dual Runtime Strategy

| Runtime | Status | Purpose |
|---------|--------|---------|
| `runtime/` (adze) | Maintenance | Tree-sitter FFI compatibility |
| `runtime2/` (adze-runtime) | Active | Pure-Rust GLR, WASM, future |

---

## Common Patterns

### Grammar Definition

```rust
#[adze::grammar("calc")]
pub mod calc {
    #[adze::language]
    pub enum Expr {
        Number(i32),
        Add(Box<Expr>, Box<Expr>),
    }
}
```

### Error Types

```rust
// Library crates: use thiserror
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("description: {0}")]
    Variant(String),
}

// Application crates: use anyhow
use anyhow::Result;
```

### Collections

| Use Case | Type |
|----------|------|
| Hot paths | `rustc_hash::FxHashMap` |
| Small collections | `smallvec::SmallVec` |
| Ordered maps | `indexmap::IndexMap` |

---

## Workspace Lints

```toml
# Enforced via [workspace.lints.rust]
unsafe_op_in_unsafe_fn = "deny"
unused_must_use = "deny"
missing_docs = "warn"
unused_extern_crates = "deny"
```

---

## Test Patterns

| Type | Tool | Example |
|------|------|---------|
| Snapshot | `insta` | `insta::assert_snapshot!()` |
| Property | `proptest` | `proptest! { ... }` |
| Feature-gated | `#[cfg(feature = "test-api")]` | Test helpers |

---

## Quick Reading Paths

| Role | Start | Next |
|------|-------|------|
| New User | [Getting Started](./tutorials/getting-started.md) | [API Reference](./reference/api.md) |
| Contributor | [Contributor Guide](./contributing/CONTRIBUTOR_GUIDE.md) | [Now/Next/Later](./status/NOW_NEXT_LATER.md) |
| Architect | [Architecture](./explanations/architecture.md) | [ADRs](./adr/INDEX.md) |
| Integrator | [API Reference](./reference/api.md) | [API Stability](./status/API_STABILITY.md) |
| Tester | [Testing Guide](./testing/TESTING_GUIDE.md) | [Test Strategy](./explanations/test-strategy.md) |

---

## Key Files

| File | Purpose |
|------|---------|
| `justfile` | Development recipes |
| `rust-toolchain.toml` | Toolchain pinning |
| `Cargo.toml` | Workspace root (75 members) |
| `.githooks/pre-commit` | Pre-commit checks |
| `docs/status/KNOWN_RED.md` | CI exclusions |

---

## CI Workflows

| Workflow | Purpose |
|----------|---------|
| `ci.yml` | Main CI with `ci-supported` job (PR gate) |
| `pure-rust-ci.yml` | Pure-Rust implementation |
| `core-tests.yml` | Core crate testing |
| `golden-tests.yml` | Tree-sitter parity |
| `benchmarks.yml` | Performance benchmarks |

---

*For complete documentation, see [INDEX.md](./INDEX.md) or [NAVIGATION.md](./NAVIGATION.md).*
