# Microcrate Guide

Adze follows a **single-responsibility-principle (SRP) microcrate** architecture. Each crate owns one concern, keeps a narrow public API, and declares its dependencies explicitly. This page catalogues every workspace crate, grouped by layer.

## Core pipeline crates

These crates form the main grammar-to-parser pipeline:

| Crate | Path | Responsibility |
|---|---|---|
| `adze-macro` | `macro/` | Proc-macro attributes (`#[adze::grammar]`, `#[adze::leaf]`, etc.) |
| `adze-common` | `common/` | Shared grammar expansion logic used by both the macro and the build tool |
| `adze-ir` | `ir/` | Grammar intermediate representation, normalization, validation, and optimization |
| `adze-glr-core` | `glr-core/` | FIRST/FOLLOW sets, LR(1) item sets, canonical collection, conflict detection |
| `adze-tablegen` | `tablegen/` | Table compression and static `Language` struct generation (Tree-sitter ABI) |
| `adze-tool` | `tool/` | Build-time driver (`build_parsers()`), code emission, visualization |
| `adze` | `runtime/` | Runtime library: `Extract` trait, error recovery, visitor, serialization |
| `adze-runtime2` | `runtime2/` | Production GLR runtime: `Parser`, `Tree`, forest builder, incremental editing |

## Concurrency crates

Bounded-concurrency infrastructure to prevent resource exhaustion:

| Crate | Path | Responsibility |
|---|---|---|
| `concurrency-caps-core` | `crates/concurrency-caps-core/` | Core cap values and initialization |
| `concurrency-caps-contract-core` | `crates/concurrency-caps-contract-core/` | Trait contracts for cap providers |
| `concurrency-env-core` | `crates/concurrency-env-core/` | Read caps from environment variables |
| `concurrency-env-contract-core` | `crates/concurrency-env-contract-core/` | Contracts for env-based cap sources |
| `concurrency-init-core` | `crates/concurrency-init-core/` | Thread-pool initialization |
| `concurrency-init-rayon-core` | `crates/concurrency-init-rayon-core/` | Rayon pool initialization with caps |
| `concurrency-init-bootstrap-core` | `crates/concurrency-init-bootstrap-core/` | Bootstrap-time cap detection |
| `concurrency-init-bootstrap-policy-core` | `crates/concurrency-init-bootstrap-policy-core/` | Policy for bootstrap caps |
| `concurrency-init-classifier-core` | `crates/concurrency-init-classifier-core/` | Classifies system into cap tiers |
| `concurrency-map-core` | `crates/concurrency-map-core/` | Concurrent map utilities |
| `concurrency-bounded-map-core` | `crates/concurrency-bounded-map-core/` | Size-bounded concurrent map |
| `concurrency-plan-core` | `crates/concurrency-plan-core/` | Concurrency execution plans |
| `concurrency-normalize-core` | `crates/concurrency-normalize-core/` | Normalize caps across sources |
| `concurrency-parse-core` | `crates/concurrency-parse-core/` | Parse concurrency config strings |

## Governance and BDD crates

Quality-assurance infrastructure for feature tracking and behavioral contracts:

| Crate | Path | Responsibility |
|---|---|---|
| `bdd-contract` | `crates/bdd-contract/` | Shared BDD scenario and phase contracts |
| `bdd-grammar-analysis-core` | `crates/bdd-grammar-analysis-core/` | Grammar-level BDD analysis |
| `bdd-grammar-fixtures` | `crates/bdd-grammar-fixtures/` | Test fixtures for grammar BDD |
| `bdd-governance-contract` | `crates/bdd-governance-contract/` | Governance BDD contracts |
| `bdd-governance-core` | `crates/bdd-governance-core/` | Governance BDD logic |
| `bdd-governance-fixtures` | `crates/bdd-governance-fixtures/` | Governance test fixtures |
| `bdd-scenario-fixtures` | `crates/bdd-scenario-fixtures/` | Shared BDD scenario fixtures |
| `bdd-grid-contract` | `crates/bdd-grid-contract/` | Grid/matrix BDD contracts |
| `bdd-grid-core` | `crates/bdd-grid-core/` | Grid BDD evaluation logic |
| `governance-contract` | `crates/governance-contract/` | Core governance trait contracts |
| `governance-metadata` | `crates/governance-metadata/` | Governance progress metadata types |
| `parser-feature-profile-snapshot-core` | `crates/parser-feature-profile-snapshot-core/` | Parser feature profile snapshot value object |
| `governance-matrix-contract` | `crates/governance-matrix-contract/` | Feature matrix contracts |
| `governance-matrix-core` | `crates/governance-matrix-core/` | Feature matrix evaluation |
| `governance-matrix-core-impl` | `crates/governance-matrix-core-impl/` | Feature matrix implementation |
| `governance-runtime-core` | `crates/governance-runtime-core/` | Runtime governance checks |
| `governance-runtime-reporting` | `crates/governance-runtime-reporting/` | Governance reports |
| `runtime-governance` | `crates/runtime-governance/` | Runtime governance integration |
| `runtime-governance-api` | `crates/runtime-governance-api/` | Public governance API |
| `runtime-governance-matrix` | `crates/runtime-governance-matrix/` | Runtime feature matrix |
| `runtime2-governance` | `crates/runtime2-governance/` | GLR runtime governance |
| `feature-policy-contract` | `crates/feature-policy-contract/` | Feature-flag policy contracts |
| `feature-policy-core` | `crates/feature-policy-core/` | Feature-flag policy logic |
| `parser-contract` | `crates/parser-contract/` | Parser trait contracts |
| `parser-governance-contract` | `crates/parser-governance-contract/` | Parser governance contracts |
| `parser-feature-contract` | `crates/parser-feature-contract/` | Parser feature contracts |
| `parser-backend-core` | `crates/parser-backend-core/` | Backend abstraction |

## Utility and supporting crates

| Crate | Path | Responsibility |
|---|---|---|
| `parsetable-metadata` | `crates/parsetable-metadata/` | Parse-table metadata types |
| `ts-format-core` | `crates/ts-format-core/` | Tree-sitter format utilities |
| `stack-pool-core` | `crates/stack-pool-core/` | Stack-based object pooling |
| `glr-versioning` | `crates/glr-versioning/` | GLR version tracking |
| `glr-test-support` | `glr-test-support/` | Test helpers for GLR crates |
| `linecol-core` | `crates/linecol-core/` | Line/column position computation |
| `common-syntax-core` | `crates/common-syntax-core/` | Shared syntax utilities |

## Application and tooling crates

| Crate | Path | Responsibility |
|---|---|---|
| `adze-cli` | `cli/` | Command-line interface |
| `lsp-generator` | `lsp-generator/` | LSP server code generation |
| `playground` | `playground/` | Interactive grammar playground |
| `wasm-demo` | `wasm-demo/` | Browser-based WASM demo |
| `ts-bridge` | `tools/ts-bridge/` | Extracts parse tables from compiled Tree-sitter grammars |

## Test and example crates

| Crate | Path | Responsibility |
|---|---|---|
| `example` | `example/` | Example grammars (arithmetic, optionals, repetitions, etc.) |
| `golden-tests` | `golden-tests/` | Tree-sitter compatibility verification |
| `testing` | `testing/` | Shared test utilities |
| `test-mini` | `test-mini/` | Minimal integration smoke tests |
| `benchmarks` | `benchmarks/` | Performance benchmarks |
| `downstream-demo` | `samples/downstream-demo/` | Demonstrates downstream consumption |
| `xtask` | `xtask/` | Workspace automation tasks |

## Grammar crates

| Crate | Path | Language |
|---|---|---|
| `adze-javascript` | `grammars/javascript/` | JavaScript grammar |
| `adze-python` | `grammars/python/` | Python grammar (with external scanner) |
| `adze-python-simple` | `grammars/python-simple/` | Simplified Python grammar |
| `adze-go` | `grammars/go/` | Go grammar |
| `test-vec-wrapper` | `grammars/test-vec-wrapper/` | Test helper grammar |

## How crates relate

The dependency graph is intentionally acyclic and layered. A quick rule of thumb:

- **Contract crates** (`*-contract`) define traits. They have no logic and no dependencies beyond `std`.
- **Core crates** (`*-core`) implement those traits. They depend only on their contract crate.
- **Integration crates** (e.g., `runtime-governance`) wire cores together behind a façade.
- **Application crates** (`cli`, `playground`, `tool`) sit at the top and pull in whatever they need.

This keeps compile times low, enforces API boundaries, and makes it straightforward to swap implementations.

## Adding a new microcrate

1. `cargo init crates/my-new-core --lib`
2. Add it to the `[workspace] members` list in the root `Cargo.toml`.
3. Give it a narrow, descriptive name following the `<domain>-<layer>-core` convention.
4. If it defines a public API contract, split the trait into a `<domain>-<layer>-contract` crate first.
5. Wire it into the integration crate that needs it.
