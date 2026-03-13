# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Adze project.

## What is an ADR?

An ADR is a document that captures an important architectural decision made along with its context and consequences. ADRs help future maintainers understand the "why" behind architectural choices.

## ADR Format

Each ADR follows this structure:

- **Title**: A short noun phrase describing the decision
- **Status**: Proposed, Accepted, Deprecated, or Superseded
- **Context**: The issue motivating this decision
- **Decision**: The change being proposed or made
- **Consequences**: What becomes easier or harder as a result

## Index

| Number | Title | Status | Date |
|--------|-------|--------|------|
| [000](000-template.md) | ADR Template | - | - |
| [001](001-pure-rust-glr-implementation.md) | Pure-Rust GLR Implementation | Accepted | 2024-01-15 |
| [002](002-workspace-structure.md) | Workspace Structure | Accepted | 2024-01-15 |
| [003](003-dual-runtime-strategy.md) | Dual Runtime Strategy | Accepted | 2024-02-01 |
| [004](004-grammar-definition-via-macros.md) | Grammar Definition via Macros | Accepted | 2024-01-15 |
| [005](005-incremental-parsing-architecture.md) | Incremental Parsing Architecture | Accepted | 2024-03-01 |
| [006](006-tree-sitter-compatibility-layer.md) | Tree-sitter Compatibility Layer | Accepted | 2024-02-15 |
| [007](007-bdd-framework-for-parser-testing.md) | BDD Framework for Parser Testing | Accepted | 2025-03-13 |
| [008](008-governance-microcrates-architecture.md) | Governance Microcrates Architecture | Accepted | 2025-03-13 |
| [009](009-symbol-registry-unification.md) | Symbol Registry Unification | Accepted | 2025-03-13 |
| [010](010-external-scanner-architecture.md) | External Scanner Architecture | Accepted | 2025-03-13 |
| [011](011-parse-table-binary-format.md) | Parse Table Binary Format (Postcard) | Accepted | 2025-03-13 |
| [012](012-performance-baseline-management.md) | Performance Baseline Management | Accepted | 2025-03-13 |
| [013](013-gss-implementation-strategy.md) | GSS Implementation Strategy | Accepted | 2025-03-13 |
| [014](014-parse-table-compression-strategy.md) | Parse Table Compression Strategy | Accepted | 2025-03-13 |
| [015](015-disambiguation-strategy.md) | Disambiguation Strategy for Ambiguous Parses | Accepted | 2025-03-13 |
| [016](016-error-handling-strategy.md) | Error Handling Strategy | Accepted | 2025-03-13 |
| [017](017-memory-management-strategy.md) | Memory Management and Allocation Strategy | Accepted | 2025-03-13 |
| [018](018-grammar-optimization-pipeline.md) | Grammar Optimization Pipeline | Accepted | 2025-03-13 |
| [019](019-contract-first-development-methodology.md) | Contract-First Development Methodology | Accepted | 2026-03-13 |
| [020](020-direct-forest-splicing-algorithm.md) | Direct Forest Splicing Algorithm | Accepted | 2026-03-13 |
| [021](021-feature-flag-and-backend-selection.md) | Feature Flag and Backend Selection Strategy | Accepted | 2026-03-13 |
| [022](022-telemetry-and-performance-monitoring.md) | Telemetry and Performance Monitoring Strategy | Accepted | 2026-03-13 |
| [023](023-forest-to-tree-conversion-strategy.md) | Forest-to-Tree Conversion Strategy | Accepted | 2026-03-13 |
| [024](024-abi-compatibility-strategy.md) | ABI Compatibility Strategy | Accepted | 2026-03-13 |
| [025](025-parse-table-validation-strategy.md) | Parse Table Validation Strategy | Accepted | 2026-03-13 |
| [026](026-proc-macro-attribute-design.md) | Proc-Macro Attribute Design and Expansion | Accepted | 2026-03-13 |

## Creating a New ADR

1. Copy `000-template.md` to a new file with the next sequential number
2. Fill in the sections following the template
3. Update this index with the new ADR
4. Submit for review

## Related Documentation

- [Documentation Index](../INDEX.md) - Master documentation index
- [Navigation Guide](../NAVIGATION.md) - Reading paths and cross-references
- [Quick Reference](../QUICK_REFERENCE.md) - One-page cheat sheet
- [Testing Guide](../testing/TESTING_GUIDE.md) - Comprehensive testing strategies
- [Contributor Guide](../contributing/CONTRIBUTOR_GUIDE.md) - Complete contributor onboarding
- [AGENTS.md](../../AGENTS.md) - Project overview and development guidelines
- [docs/archive/implementation/](../archive/implementation/) - Implementation details
- [docs/tutorials/](../tutorials/) - Getting started guides
