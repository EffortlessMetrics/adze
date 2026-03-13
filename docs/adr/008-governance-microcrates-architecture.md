# ADR-008: Governance Microcrates Architecture

## Status

Accepted

## Context

The Adze project requires robust governance-as-code capabilities to ensure:
- **Test quality enforcement**: Preventing tests from being silently disabled
- **Concurrency policy**: Consistent thread pool and resource management
- **Feature flags**: Controlled feature rollout and deprecation
- **BDD scenario management**: Structured test fixtures and contracts

Initially, governance code was scattered across the main crates:
- `adze-runtime`: Mixed runtime logic with governance checks
- `adze-tool`: Build-time governance mixed with code generation
- `testing/`: Ad-hoc test utilities without clear boundaries

This created several problems:
1. **Circular dependencies**: Governance code needed runtime types, runtime needed governance
2. **Testing difficulty**: Hard to test governance logic in isolation
3. **Unclear ownership**: No clear module boundaries for governance features
4. **Dependency bloat**: Pulling in governance required pulling in everything

### Alternatives Considered

1. **Single governance crate**: One large crate for all governance code
   - Pros: Simple dependency management
   - Cons: Monolithic, hard to test in isolation, slow compilation

2. **Inline governance**: Keep governance in relevant crates
   - Pros: No new crates
   - Cons: Circular dependencies, unclear boundaries

3. **Microcrates with SRP**: Split into Single Responsibility Principle crates
   - Pros: Clear boundaries, isolated testing, minimal dependencies
   - Cons: More crates to manage

## Decision

We adopted a **governance microcrates architecture** with 47 specialized crates in the `crates/` directory, each following the Single Responsibility Principle (SRP).

### Crate Categories

#### BDD Testing Microcrates
| Crate | Purpose |
|-------|---------|
| `bdd-contract` | BDD scenario contracts and interfaces |
| `bdd-governance-contract` | Governance scenario contracts |
| `bdd-governance-core` | Core BDD governance implementation |
| `bdd-governance-fixtures` | Test fixtures for BDD scenarios |
| `bdd-grammar-analysis-core` | Grammar analysis BDD scenarios |
| `bdd-grammar-fixtures` | Grammar test fixtures |
| `bdd-grid-contract` | Grid testing contracts |
| `bdd-grid-core` | Grid testing implementation |
| `bdd-scenario-fixtures` | General scenario fixtures |

#### Concurrency Governance Microcrates
| Crate | Purpose |
|-------|---------|
| `concurrency-bounded-map-core` | Bounded map for concurrent access |
| `concurrency-caps-contract-core` | Resource caps contracts |
| `concurrency-caps-core` | Resource caps implementation |
| `concurrency-env-contract-core` | Environment config contracts |
| `concurrency-env-core` | Environment configuration |
| `concurrency-init-bootstrap-core` | Initialization bootstrap |
| `concurrency-init-bootstrap-policy-core` | Bootstrap policies |
| `concurrency-init-classifier-core` | Init classification |
| `concurrency-init-core` | Core initialization |
| `concurrency-init-rayon-core` | Rayon-specific init |
| `concurrency-map-core` | Concurrent map utilities |
| `concurrency-normalize-core` | Normalization utilities |
| `concurrency-parse-core` | Parse-time concurrency |
| `concurrency-plan-core` | Concurrency planning |

#### Feature & Runtime Governance Microcrates
| Crate | Purpose |
|-------|---------|
| `feature-policy-contract` | Feature flag contracts |
| `feature-policy-core` | Feature policy implementation |
| `governance-contract` | Core governance contracts |
| `governance-matrix-contract` | Governance matrix contracts |
| `governance-matrix-core` | Governance matrix implementation |
| `governance-matrix-core-impl` | Matrix implementation details |
| `governance-metadata` | Governance metadata handling |
| `governance-runtime-core` | Runtime governance |
| `governance-runtime-reporting` | Governance reporting |

#### Parser & Utility Microcrates
| Crate | Purpose |
|-------|---------|
| `parser-backend-core` | Parser backend abstraction |
| `parser-contract` | Parser contracts |
| `parser-feature-contract` | Parser feature contracts |
| `parser-governance-contract` | Parser governance contracts |
| `parsetable-metadata` | Parse table metadata |
| `runtime-governance` | Runtime governance integration |
| `runtime-governance-api` | Governance API surface |
| `runtime-governance-matrix` | Runtime governance matrix |
| `runtime2-governance` | Runtime2 governance |
| `stack-pool-core` | Stack pooling utilities |
| `linecol-core` | Line/column utilities |
| `common-syntax-core` | Shared syntax utilities |
| `ts-format-core` | Tree-sitter format utilities |
| `glr-versioning` | GLR version management |

### Naming Conventions

- **`*-contract`**: Interface/trait definitions only, no implementation
- **`*-core`**: Core implementation, minimal dependencies
- **`*-fixtures`**: Test data and fixtures
- **`*-impl`**: Internal implementation details

### Dependency Rules

```
contract crates ← core crates ← fixture crates
     ↑               ↑              ↑
     └───────────────┴──────────────┘
              (can depend on lower)
```

## Consequences

### Positive

- **Test isolation**: Each microcrate can be tested independently
- **Minimal dependencies**: Only pull what you need
- **Clear ownership**: Each crate has a single maintainer focus
- **Faster compilation**: Changed microcrate only triggers limited rebuild
- **Better architecture**: Forced clear boundaries between concerns
- **Reusable contracts**: Contract crates enable multiple implementations
- **Governance-as-code**: Policies are versioned, testable code

### Negative

- **Crate proliferation**: 47 microcrates increases cognitive load
- **Dependency management**: More `Cargo.toml` files to maintain
- **Version coordination**: Need to keep related crates in sync
- **Discovery difficulty**: Finding the right crate takes time
- **IDE overhead**: More crates can slow rust-analyzer

### Neutral

- **Workspace organization**: Microcrates excluded from main `cargo test` by default
- **CI strategy**: `microcrate-ci.yml` handles these separately
- **Documentation**: Each crate needs its own README

## Related

- Related ADRs: [ADR-002](002-workspace-structure.md)
- Reference: [crates/](../../crates/) - Microcrates directory
- Reference: [AGENTS.md](../../AGENTS.md) - Workspace organization
- Reference: [.github/workflows/microcrate-ci.yml](../../.github/workflows/microcrate-ci.yml)
