# SRP Microcrates

This directory contains single-responsibility principle (SRP) support modules that implement governance-as-code for the Adze parser toolchain. During the 0.9 release work, transitional façade crates are being retired or moved into owner submodules before release.

## Overview

Each microcrate follows the single-responsibility principle, focusing on one specific concern:

- **Contract crates** define traits and types for inter-module communication
- **Core crates** provide concrete implementations
- **Fixture crates** supply test data and scenarios

This modular architecture enables:

- Independent testing and validation
- Clear dependency boundaries
- Feature-gated compilation
- Contract-based verification

## Categories

### BDD Framework

Behavior-driven development infrastructure for scenario tracking and progress reporting.

| Crate | Purpose |
|-------|---------|
| [`bdd-governance-contract`](bdd-governance-contract) | Governance contracts for BDD progress tracking |
| [`bdd-governance-core`](bdd-governance-core) | Core implementation of BDD governance |
| [`bdd-governance-fixtures`](bdd-governance-fixtures) | Test fixtures for BDD governance scenarios |
| [`bdd-grammar-fixtures`](bdd-grammar-fixtures) | Test fixtures for grammar BDD scenarios |
| [`bdd-grid-core`](bdd-grid-core) | Core implementation of BDD scenario grids |
| [`bdd-scenario-fixtures`](bdd-scenario-fixtures) | General BDD scenario test fixtures |

### Concurrency (11 crates)

Thread pool management, concurrency caps, and parallel execution policies.

| Crate | Purpose |
|-------|---------|
| [`concurrency-caps-contract-core`](concurrency-caps-contract-core) | Contracts for concurrency cap definitions |
| [`concurrency-caps-core`](concurrency-caps-core) | Core concurrency cap implementations |
| [`concurrency-env-contract-core`](concurrency-env-contract-core) | Environment-based concurrency contract |
| [`concurrency-env-core`](concurrency-env-core) | Environment variable parsing for concurrency |
| [`concurrency-init-bootstrap-core`](concurrency-init-bootstrap-core) | Bootstrap initialization for concurrency |
| [`concurrency-init-core`](concurrency-init-core) | Rayon global thread-pool initialization |
| [`concurrency-init-rayon-core`](concurrency-init-rayon-core) | Rayon-specific initialization utilities |
| [`concurrency-map-core`](concurrency-map-core) | Concurrent map data structures |
| [`concurrency-normalize-core`](concurrency-normalize-core) | Normalization of concurrency configurations |
| [`concurrency-parse-core`](concurrency-parse-core) | Parsing of concurrency specifications |
| [`concurrency-plan-core`](concurrency-plan-core) | Concurrency planning and scheduling |

### Governance (7 crates)

Parser backend selection, metadata management, and policy enforcement.

| Crate | Purpose |
|-------|---------|
| [`governance-contract`](governance-contract) | Shared governance contracts for parser backend selection |
| [`governance-matrix-contract`](governance-matrix-contract) | Governance matrix contract definitions |
| [`governance-matrix-core`](governance-matrix-core) | Core governance matrix implementation |
| [`governance-matrix-core-impl`](governance-matrix-core-impl) | Concrete governance matrix implementations |
| [`governance-metadata`](governance-metadata) | Metadata structures for governance |
| [`governance-runtime-core`](governance-runtime-core) | Runtime governance core functionality |
| [`governance-runtime-reporting`](governance-runtime-reporting) | Governance reporting utilities |

### Feature Policy

Feature flag management and policy enforcement.

| Crate | Purpose |
|-------|---------|
| [`feature-policy-core`](feature-policy-core) | Core parser feature-policy implementation |

### Runtime Governance (4 crates)

Runtime-facing governance helpers and progress reporting.

| Crate | Purpose |
|-------|---------|
| [`runtime-governance`](runtime-governance) | Runtime-facing governance helpers |
| [`runtime-governance-api`](runtime-governance-api) | Runtime governance API definitions |
| [`runtime-governance-matrix`](runtime-governance-matrix) | Runtime governance matrix implementation |
| [`runtime2-governance`](runtime2-governance) | Governance for runtime2 (production GLR) |

### Utilities (5 crates)

Shared utilities, metadata, and support structures.

| Crate | Purpose |
|-------|---------|
| [`linecol-core`](linecol-core) | Line/column byte-position tracking |
| [`parsetable-metadata`](parsetable-metadata) | Parse table metadata structures |
| [`ts-c-harness`](ts-c-harness) | Tree-sitter C FFI test harness *(excluded from workspace)* |

## Dependency Graph

The dependency graph is intentionally not maintained by hand during the 0.9
microcrate transition. Use these source-of-truth commands instead:

```bash
cargo metadata --format-version 1 --no-deps
cargo run -q -p xtask -- check-package-boundary
```

## Feature Flag Matrix

All crates support standard governance features for parser backend selection:

| Feature | Description |
|---------|-------------|
| `pure-rust` | Enable pure-Rust GLR backend |
| `tree-sitter-standard` | Enable standard Tree-sitter backend |
| `tree-sitter-c2rust` | Enable c2rust Tree-sitter backend |
| `glr` | Enable GLR parsing (implies `pure-rust`) |
| `strict_api` | Deny unreachable public items |
| `strict_docs` | Deny missing documentation |

### Feature Propagation

Features propagate through the dependency chain:

```text
governance-contract
  └── bdd-governance-contract
```

Enabling `glr` on `governance-contract` automatically enables `pure-rust` and propagates down the chain.

## Test Coverage Summary

The microcrate list is actively shrinking during the 0.9 SRP owner-module
transition. See
[MICROCRATE_TEST_COVERAGE.md](../docs/status/MICROCRATE_TEST_COVERAGE.md)
for the current tracked crate count and coverage analysis.

## Quick Start

### Adding a Dependency

Add microcrates to your `Cargo.toml` using workspace dependencies:

```toml
[dependencies]
adze-governance-contract = { workspace = true }
```

### Enabling Features

Select your parser backend via feature flags:

```toml
[dependencies]
adze-governance-contract = { workspace = true, features = ["glr"] }
```

### Using BDD Progress Tracking

```rust
use adze_bdd_contract::{bdd_progress_report, BddPhase};

// Generate a progress report for the Core phase
let report = bdd_progress_report(BddPhase::Core);
println!("{}", report);
```

### Using Governance Contracts

```rust
use adze_governance_contract::{ParserBackend, ParserFeatureProfile};

// Get the current feature profile
let profile = ParserFeatureProfile::current();

// Select the appropriate backend
let backend = profile.preferred_backend();
```

### Using Concurrency Initialization

```rust
use adze_concurrency_init_core::init_concurrency_caps;

// Initialize concurrency caps (idempotent)
init_concurrency_caps();
```

### Using Line/Column Tracking

```rust
use adze_linecol_core::LineCol;

let lc = LineCol::at_position(b"hello\nworld", 8);
assert_eq!(lc.line, 1);
assert_eq!(lc.column(8), 2);
```

## Crate Naming Conventions

| Suffix | Meaning |
|--------|---------|
| `-contract` | Trait and type definitions only |
| `-core` | Concrete implementations |
| `-fixtures` | Test data and scenarios |
| `-impl` | Specific implementations of contracts |

## Architecture Principles

1. **Single Responsibility**: Each crate has one clear purpose
2. **Contract-First**: Traits defined in `-contract` crates
3. **Feature-Gated**: All crates support standard feature flags
4. **Test Coverage**: 100% BDD + property test coverage
5. **Documentation**: All public APIs documented with `//!` comments

## Related Documentation

- [MICROCRATE_TEST_COVERAGE.md](../docs/status/MICROCRATE_TEST_COVERAGE.md) - Detailed test coverage analysis
- [API_STABILITY.md](../docs/status/API_STABILITY.md) - API stability guarantees
- [AGENTS.md](../AGENTS.md) - Development guidelines
