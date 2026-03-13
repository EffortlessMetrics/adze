# ADR-003: Dual Runtime Strategy

## Status

Accepted

## Context

Adze maintains two separate runtime implementations:

1. **`runtime/`** (adze): The original runtime with Tree-sitter FFI bindings
2. **`runtime2/`** (adze-runtime): A pure-Rust GLR runtime with Tree-sitter-compatible API

This duality arose from the transition from Tree-sitter wrapper to pure-Rust implementation. The question was whether to:
- Keep both runtimes indefinitely
- Merge them into a single unified runtime
- Deprecate one in favor of the other

### Alternatives Considered

1. **Single Runtime (Tree-sitter FFI)**: Keep only `runtime/` with Tree-sitter bindings
2. **Single Runtime (Pure Rust)**: Replace everything with `runtime2/`
3. **Feature Flag Unification**: Merge both into one crate with feature flags
4. **Gradual Migration**: Maintain both during transition, then deprecate

## Decision

We maintain both runtimes with distinct purposes:

### `runtime/` - Legacy Tree-sitter Backend

```rust
// Uses Tree-sitter C library via FFI
// For users who need Tree-sitter compatibility
#[cfg(feature = "tree-sitter")]
pub use tree_sitter::Parser;
```

**Purpose**: Provides stable Tree-sitter integration for:
- Users with existing Tree-sitter grammar investments
- Projects requiring Tree-sitter's mature query system
- Compatibility with Tree-sitter ecosystem tools

**Status**: Maintenance mode - no new features, bug fixes only

### `runtime2/` - Pure-Rust GLR Backend

```rust
// Pure Rust GLR implementation
// Tree-sitter-compatible API
pub struct Parser {
    language: Option<Language>,
    // GLR-specific state
}
```

**Purpose**: Primary development target for:
- Pure-Rust builds without C toolchain
- WASM compilation
- GLR parsing for ambiguous grammars
- Future incremental parsing improvements

**Status**: Active development

### Migration Path

```
┌─────────────────────────────────────────────────────────────┐
│                    User Code                                 │
│              grammar::parse(input)                           │
└─────────────────────────────────────────────────────────────┘
                    │                   │
        ┌───────────┘                   └───────────┐
        ▼                                           ▼
┌───────────────────────┐             ┌───────────────────────┐
│      runtime/         │             │      runtime2/        │
│   (Tree-sitter FFI)   │             │   (Pure Rust GLR)     │
│                       │             │                       │
│  - Mature             │             │  - Active development │
│  - C required         │             │  - Pure Rust          │
│  - Full query system  │             │  - GLR support        │
└───────────────────────┘             └───────────────────────┘
```

Users select the backend via Cargo features:

```toml
# Default: Pure Rust backend
adze = "0.8"

# Tree-sitter backend (legacy)
adze = { version = "0.8", default-features = false, features = ["tree-sitter"] }
```

## Consequences

### Positive

- **Smooth Migration**: Users can migrate at their own pace
- **Risk Mitigation**: Pure-Rust runtime can mature while Tree-sitter remains stable
- **Clear Purpose**: Each runtime has well-defined use cases
- **Feature Comparison**: Both implementations validate the API design

### Negative

- **Maintenance Overhead**: Two codebases require testing and maintenance
- **API Drift Risk**: Runtimes may diverge if not carefully coordinated
- **User Confusion**: New users must understand which runtime to use
- **Duplicate Effort**: Some features must be implemented twice

### Neutral

- **Feature Parity Tracking**: We maintain a compatibility matrix
- **Documentation Burden**: Docs must cover both runtimes
- **Testing Complexity**: CI must test both backends

## Related

- Related ADRs: [ADR-001](001-pure-rust-glr-implementation.md), [ADR-006](006-tree-sitter-compatibility-layer.md)
- Reference: [runtime2/README.md](../../runtime2/README.md) - Pure Rust runtime docs
- Reference: [docs/status/KNOWN_RED.md](../status/KNOWN_RED.md) - Known issues per runtime
