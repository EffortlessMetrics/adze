# ADR-002: Workspace Structure

## Status

Accepted

## Context

The Adze project has grown to 75 crates organized in a Cargo workspace. This structure was not accidental—it emerged from several competing requirements:

1. **Compilation Speed**: Large monolithic crates slow down incremental builds
2. **Dependency Management**: Different components have different dependency profiles
3. **Testing Isolation**: Unit tests should run against minimal dependencies
4. **Feature Gates**: Cargo features work best at crate boundaries
5. **Governance and Contracts**: We wanted to enforce architectural boundaries through type systems
6. **Parallel Development**: Multiple developers can work on different layers simultaneously

### Alternatives Considered

1. **Single Crate**: All code in one crate with feature flags
2. **3-5 Crates**: Traditional runtime/macro/tool split
3. **Microservices-style**: Each component in its own repository

## Decision

We adopted a layered workspace architecture with 75 crates organized into distinct layers:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          Application Layer                               │
│   cli/  │  playground/  │  lsp-generator/  │  wasm-demo/                │
└─────────────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────────┐
│                          Grammar Layer                                   │
│   grammars/python  │  grammars/javascript  │  grammars/go               │
└─────────────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────────┐
│                        Core Pipeline (7 crates)                          │
│   adze  │  adze-macro  │  adze-tool  │  adze-common  │  adze-ir        │
│   adze-glr-core  │  adze-tablegen                                    │
└─────────────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────────┐
│                        Runtime Layer                                     │
│   runtime/  │  runtime2/                                                │
└─────────────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────────┐
│                    Governance Layer (47 micro-crates)                    │
│   parser-contract  │  parser-governance-contract  │  bdd-*              │
│   concurrency-*  │  governance-*  │  feature-policy-*                   │
└─────────────────────────────────────────────────────────────────────────┘
```

### Core Pipeline (7 Crates)

These crates form the primary parser generation pipeline and are covered by the PR gate (`just ci-supported`):

| Crate | Purpose |
|-------|---------|
| [`adze`](../../runtime/) | Main runtime library with `Extract` trait |
| [`adze-macro`](../../macro/) | Proc-macro attributes for grammar definition |
| [`adze-tool`](../../tool/) | Build-time code generation |
| [`adze-common`](../../common/) | Shared grammar expansion logic |
| [`adze-ir`](../../ir/) | Grammar IR with GLR support |
| [`adze-glr-core`](../../glr-core/) | GLR parser generation |
| [`adze-tablegen`](../../tablegen/) | Table compression and FFI generation |

### Governance Micro-crates (47 Crates)

These enforce architectural contracts through type systems:

- **Contract crates**: Define traits and types for component boundaries
- **Core impl crates**: Provide standard implementations
- **BDD crates**: Behavior-driven development fixtures and scenarios
- **Concurrency crates**: Thread safety and parallelism contracts

### Workspace Exclusions

Some crates are intentionally excluded from default workspace commands:

```toml
exclude = [
  "runtime/fuzz",        # Fuzzing has different dependencies
  "tools/ts-bridge",     # Requires grammar features
  "crates/ts-c-harness", # External C/Tree-sitter dependencies
  "example",             # Mutually exclusive features
]
```

## Consequences

### Positive

- **Fast Incremental Builds**: Changing one crate only recompiles dependents
- **Clear Boundaries**: Crate structure enforces architectural separation
- **Parallel CI**: Different layers can be tested in parallel
- **Type-Safe Contracts**: Governance crates catch architectural violations at compile time
- **Feature Isolation**: Each crate can have focused feature gates
- **Independent Versioning**: Crates can be versioned independently if needed

### Negative

- **Workspace Complexity**: 75 crates is intimidating for new contributors
- **Dependency Graph**: Understanding the full dependency graph requires tooling
- **Publishing Overhead**: Releasing requires coordinating multiple crates
- **IDE Performance**: rust-analyzer must index many crates
- **Build Artifact Size**: More crates means more build metadata

### Neutral

- **Documentation Required**: [`AGENTS.md`](../../AGENTS.md) is essential for navigation
- **Tooling Investment**: Custom `justfile` recipes manage workspace operations
- **MSRV Coordination**: All crates must share the same minimum Rust version

## Related

- Related ADRs: [ADR-003](003-dual-runtime-strategy.md)
- Reference: [AGENTS.md](../../AGENTS.md) - Workspace organization
- Reference: [Cargo.toml](../../Cargo.toml) - Workspace members list
