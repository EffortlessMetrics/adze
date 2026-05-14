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

No standalone concurrency microcrates remain. Concurrency caps, environment
contracts, normalization, planning, and bounded-map helpers now live under the
runtime owner module `adze::concurrency_caps`.

## Governance and BDD crates

Quality-assurance infrastructure for feature tracking and behavioral contracts:

| Crate | Path | Responsibility |
|---|---|---|
| `bdd-contract` | `crates/bdd-contract/` | Shared BDD scenario and phase contracts |
| `bdd-grammar-fixtures` | `crates/bdd-grammar-fixtures/` | Test fixtures and grammar-level BDD analysis |
| `bdd-governance-core` | `crates/bdd-governance-core/` | Governance BDD snapshots/matrix composition |
| `bdd-governance-reporting-core` | `crates/bdd-governance-reporting-core/` | Profile-aware governance report/status formatting |
| `bdd-grid-contract` | `crates/bdd-grid-contract/` | Grid/matrix BDD contracts |
| `governance-runtime-core` | `crates/governance-runtime-core/` | Runtime governance checks |
| `feature-policy-contract` | `crates/feature-policy-contract/` | Feature-flag policy contracts |
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
| `error-location-core` | `crates/error-location-core/` | Shared parse error location type and offset conversion |

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
- **Integration crates** wire cores together behind an owner-facing module or crate.
- **Application crates** (`cli`, `playground`, `tool`) sit at the top and pull in whatever they need.

This keeps compile times low, enforces API boundaries, and makes it straightforward to swap implementations.

## Adding a new microcrate

1. `cargo init crates/my-new-core --lib`
2. Add it to the `[workspace] members` list in the root `Cargo.toml`.
3. Give it a narrow, descriptive name following the `<domain>-<layer>-core` convention.
4. If it defines a public API contract, split the trait into a `<domain>-<layer>-contract` crate first.
5. Wire it into the integration crate that needs it.
