# ADR-001: Pure-Rust GLR Implementation

## Status

Accepted

## Context

Adze (formerly rust-sitter) was originally designed to wrap Tree-sitter, a C-based parser generator. While Tree-sitter is excellent for many use cases, several issues motivated building a pure-Rust GLR implementation:

1. **Build Complexity**: Tree-sitter requires a C toolchain (gcc/clang, make) which complicates cross-compilation and Windows support
2. **WASM Compilation**: Compiling Tree-sitter to WebAssembly requires additional tooling and produces larger binaries
3. **Ambiguous Grammar Support**: Tree-sitter uses a deterministic LR parser, which cannot handle ambiguous grammars common in languages like C++ and JavaScript
4. **Rust Ecosystem Integration**: Users expect pure-Rust crates that work seamlessly with Cargo without external dependencies
5. **Debugging Difficulty**: Debugging parser issues across the Rust/C boundary is challenging

### Alternatives Considered

1. **Continue with Tree-sitter FFI**: Maintain C bindings and accept the complexity
2. **Use existing Rust parser generators**: (lalrpop, nom, chumsky) - but none provide Tree-sitter-compatible incremental parsing
3. **Port Tree-sitter to Rust**: Full port of Tree-sitter's codebase - significant effort

## Decision

We implemented a pure-Rust GLR (Generalized LR) parser that:

1. **GLR Parsing Algorithm**: Uses a graph-structured stack (GSS) to handle ambiguous grammars by forking and merging parse paths
2. **Tree-sitter Compatible Output**: Generates ABI-compatible parse tables and language structures
3. **Modular Architecture**: Separates concerns into distinct crates:
   - [`adze-ir`](../../ir/): Grammar intermediate representation with GLR support
   - [`adze-glr-core`](../../glr-core/): GLR parsing engine with LR(1) automaton
   - [`adze-tablegen`](../../tablegen/): Parse table compression and FFI generation

### Key Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Grammar Definition                        │
│              #[adze::grammar] proc-macros                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    adze-ir                                   │
│  - Grammar IR with multiple rules per LHS                    │
│  - Precedence, associativity, field mappings                 │
│  - Grammar optimization passes                               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    adze-glr-core                             │
│  - FIRST/FOLLOW set computation                              │
│  - LR(1) automaton with conflict detection                   │
│  - GLR fork/merge capabilities                               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    adze-tablegen                             │
│  - Tree-sitter compatible table compression                  │
│  - FFI-compatible TSLanguage generation                      │
│  - Deterministic output for reproducible builds              │
└─────────────────────────────────────────────────────────────┘
```

## Consequences

### Positive

- **No C Toolchain Required**: Pure Rust builds work on any platform Rust supports
- **WASM Support**: Clean compilation to WebAssembly without Emscripten
- **Ambiguous Grammar Handling**: GLR can parse languages like C++ that have inherent ambiguities
- **Better Error Messages**: Full control over error reporting and recovery
- **Easier Debugging**: All parsing logic is in Rust, enabling better tooling integration
- **Smaller Binaries**: No C runtime overhead

### Negative

- **Implementation Complexity**: GLR is more complex than deterministic LR parsing
- **Performance Overhead**: GLR parsing can be slower for unambiguous grammars due to forking
- **Maintenance Burden**: We own the entire parsing stack rather than leveraging Tree-sitter's maturity
- **Feature Parity**: Some advanced Tree-sitter features (query system, full incremental parsing) are still in progress

### Neutral

- **Tree-sitter Interop**: We maintain a bridge (`ts-bridge`) for importing existing Tree-sitter grammars
- **API Compatibility**: The runtime API mirrors Tree-sitter's for familiarity

## Related

- Related ADRs: [ADR-003](003-dual-runtime-strategy.md), [ADR-006](006-tree-sitter-compatibility-layer.md)
- Implementation: [docs/archive/implementation/PURE_RUST_IMPLEMENTATION.md](../archive/implementation/PURE_RUST_IMPLEMENTATION.md)
