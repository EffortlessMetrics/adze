# SRP Microcrates

This directory contains **47 single-responsibility principle (SRP) microcrates** that implement governance-as-code for the Adze parser toolchain.

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

### BDD Framework (9 crates)

Behavior-driven development infrastructure for scenario tracking and progress reporting.

| Crate | Purpose |
|-------|---------|
| [`bdd-contract`](bdd-contract) | Shared BDD contracts for scenario tracking and feature-matrix summaries |
| [`bdd-governance-contract`](bdd-governance-contract) | Governance contracts for BDD progress tracking |
| [`bdd-governance-core`](bdd-governance-core) | Core implementation of BDD governance |
| [`bdd-governance-fixtures`](bdd-governance-fixtures) | Test fixtures for BDD governance scenarios |
| [`bdd-grammar-analysis-core`](bdd-grammar-analysis-core) | BDD scenario core for grammar analysis |
| [`bdd-grammar-fixtures`](bdd-grammar-fixtures) | Test fixtures for grammar BDD scenarios |
| [`bdd-grid-contract`](bdd-grid-contract) | BDD scenario grid contracts |
| [`bdd-grid-core`](bdd-grid-core) | Core implementation of BDD scenario grids |
| [`bdd-scenario-fixtures`](bdd-scenario-fixtures) | General BDD scenario test fixtures |

### Concurrency (14 crates)

Thread pool management, concurrency caps, and parallel execution policies.

| Crate | Purpose |
|-------|---------|
| [`concurrency-bounded-map-core`](concurrency-bounded-map-core) | Bounded concurrent map implementation |
| [`concurrency-caps-contract-core`](concurrency-caps-contract-core) | Contracts for concurrency cap definitions |
| [`concurrency-caps-core`](concurrency-caps-core) | Core concurrency cap implementations |
| [`concurrency-env-contract-core`](concurrency-env-contract-core) | Environment-based concurrency contract |
| [`concurrency-env-core`](concurrency-env-core) | Environment variable parsing for concurrency |
| [`concurrency-init-bootstrap-core`](concurrency-init-bootstrap-core) | Bootstrap initialization for concurrency |
| [`concurrency-init-bootstrap-policy-core`](concurrency-init-bootstrap-policy-core) | Policy for bootstrap initialization |
| [`concurrency-init-classifier-core`](concurrency-init-classifier-core) | Classification of initialization contexts |
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

### Parser Contracts (4 crates)

Parser backend abstraction and feature negotiation.

| Crate | Purpose |
|-------|---------|
| [`parser-backend-core`](parser-backend-core) | Core parser backend abstractions |
| [`parser-contract`](parser-contract) | Shared contracts for parser backend selection |
| [`parser-feature-contract`](parser-feature-contract) | Parser feature negotiation contracts |
| [`parser-governance-contract`](parser-governance-contract) | Governance contracts for parser backends |

### Feature Policy (2 crates)

Feature flag management and policy enforcement.

| Crate | Purpose |
|-------|---------|
| [`feature-policy-contract`](feature-policy-contract) | Feature policy contract definitions |
| [`feature-policy-core`](feature-policy-core) | Core parser feature-policy implementation |

### Runtime Governance (4 crates)

Runtime-facing governance helpers and progress reporting.

| Crate | Purpose |
|-------|---------|
| [`runtime-governance`](runtime-governance) | Runtime-facing governance helpers |
| [`runtime-governance-api`](runtime-governance-api) | Runtime governance API definitions |
| [`runtime-governance-matrix`](runtime-governance-matrix) | Runtime governance matrix implementation |
| [`runtime2-governance`](runtime2-governance) | Governance for runtime2 (production GLR) |

### Utilities (7 crates)

Shared utilities, metadata, and support structures.

| Crate | Purpose |
|-------|---------|
| [`common-syntax-core`](common-syntax-core) | Common syntax utilities |
| [`glr-versioning`](glr-versioning) | GLR versioning support |
| [`linecol-core`](linecol-core) | Line/column byte-position tracking |
| [`parsetable-metadata`](parsetable-metadata) | Parse table metadata structures |
| [`stack-pool-core`](stack-pool-core) | Stack-allocated pool utilities |
| [`ts-c-harness`](ts-c-harness) | Tree-sitter C FFI test harness *(excluded from workspace)* |
| [`ts-format-core`](ts-format-core) | Tree-sitter formatting utilities |

## Dependency Graph

```text
                                    ┌─────────────────┐
                                    │  bdd-grid-core  │
                                    └────────┬────────┘
                                             │
                                             ▼
                                    ┌─────────────────┐
                                    │ bdd-grid-contract│
                                    └────────┬────────┘
                                             │
                                             ▼
┌──────────────────┐                ┌─────────────────┐
│  bdd-grid-core   │                │  bdd-contract   │
└──────────────────┘                └────────┬────────┘
                                             │
                                             ▼
                                    ┌─────────────────────┐
                                    │ bdd-governance-core │
                                    └────────┬────────────┘
                                             │
                                             ▼
                                    ┌─────────────────────────┐
                                    │ bdd-governance-contract │
                                    └────────┬────────────────┘
                                             │
                                             ▼
                                    ┌───────────────────────────┐
                                    │ parser-governance-contract │
                                    └────────┬──────────────────┘
                                             │
                    ┌────────────────────────┼────────────────────────┐
                    │                        │                        │
                    ▼                        ▼                        ▼
           ┌────────────────┐      ┌──────────────────┐    ┌─────────────────┐
           │ governance-     │      │ parser-feature-  │    │ feature-policy- │
           │ contract        │      │ contract         │    │ contract        │
           └────────┬───────┘      └──────────────────┘    └─────────────────┘
                    │
                    ▼
           ┌────────────────┐
           │ parser-contract │
           └────────────────┘

Concurrency Stack:
┌───────────────────────────────────────────────────────────────┐
│                    concurrency-init-core                       │
│  ┌─────────────────────┬─────────────────────┬──────────────┐ │
│  │ concurrency-env-core│concurrency-init-    │concurrency-  │ │
│  │                     │bootstrap-core       │init-rayon-core│
│  └─────────────────────┴─────────────────────┴──────────────┘ │
└───────────────────────────────────────────────────────────────┘
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
parser-contract
  └── governance-contract
        └── parser-governance-contract
              └── bdd-governance-contract
```

Enabling `glr` on `parser-contract` automatically enables `pure-rust` and propagates down the chain.

## Test Coverage Summary

All 47 microcrates have comprehensive test coverage:

| Category | Count | BDD Tests | Property Tests | Contract Lock |
|----------|-------|-----------|----------------|---------------|
| BDD Framework | 9 |✓| ✓ | 6/9 |
| Concurrency | 14 | ✓ | ✓ | 14/14 |
| Governance | 7 | ✓ | ✓ | 7/7 |
| Parser Contracts | 4 | ✓ | ✓ | 4/4 |
| Feature Policy | 2 | ✓ | ✓ | 2/2 |
| Runtime Governance | 4 | ✓ | ✓ | 4/4 |
| Utilities | 7 | ✓ | ✓ | 6/7 |

**Total: 100% BDD + Property coverage across all 47 crates**

See [MICROCRATE_TEST_COVERAGE.md](../docs/status/MICROCRATE_TEST_COVERAGE.md) for detailed coverage analysis.

## Quick Start

### Adding a Dependency

Add microcrates to your `Cargo.toml` using workspace dependencies:

```toml
[dependencies]
adze-governance-contract = { workspace = true }
adze-parser-contract = { workspace = true }
```

### Enabling Features

Select your parser backend via feature flags:

```toml
[dependencies]
adze-parser-contract = { workspace = true, features = ["glr"] }
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
