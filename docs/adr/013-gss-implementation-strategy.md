# ADR 013: GSS Implementation Strategy

**Status**: Accepted
**Date**: 2025-03-13
**Authors**: adze maintainers
**Related**: ADR-001 (Pure-Rust GLR Implementation), ADR-003 (Dual Runtime Strategy)

## Context

The Graph-Structured Stack (GSS) is a fundamental data structure for GLR parsing, enabling efficient fork and merge operations when multiple parse paths exist. The Adze project maintains two competing GSS implementations:

1. **Rc-based Implementation** ([`glr-core/src/gss.rs`](../../glr-core/src/gss.rs))
   - Uses `Rc<StackNode>` for shared ownership
   - Nodes are individually heap-allocated
   - Supports structural sharing through reference counting

2. **Arena-based Implementation** ([`glr-core/src/gss_arena.rs`](../../glr-core/src/gss_arena.rs))
   - Uses `typed_arena::Arena` for bulk allocation
   - All nodes allocated from a single arena
   - Lifetime-bound references instead of Rc

3. **Persistent Stack** ([`glr-core/src/stack.rs`](../../glr-core/src/stack.rs))
   - Hybrid approach with small vector optimization
   - Uses `Arc<StackNode>` for shared tails
   - Spills to new nodes when head capacity exceeded

### Problem Statement

Each implementation has distinct performance characteristics:
- Rc-based: Simple ownership, but allocation overhead per node
- Arena-based: Minimal allocation overhead, but lifetime complexity
- Persistent stack: Balance of both, but more complex invariant maintenance

## Decision

We adopt a **tiered strategy** where each implementation serves a specific use case:

### Primary Implementation: Persistent Stack ([`stack.rs`](../../glr-core/src/stack.rs))

The persistent stack with small vector optimization serves as the default GSS implementation for production parsing:

```rust
pub struct StackNode {
    pub state: u16,
    pub symbol: Option<u16>,
    pub head: Vec<u16>,  // Small vector optimization (4 pairs = 8 entries)
    pub tail: Option<Arc<StackNode>>,
}
```

**Rationale**:
- Small vector optimization avoids allocation for shallow stacks
- Arc-based sharing enables efficient forking without copying
- Well-defined invariants documented in code comments

### Secondary Implementation: Arena GSS ([`gss_arena.rs`](../../glr-core/src/gss_arena.rs))

The arena-based implementation is reserved for:
- High-throughput parsing scenarios with deep parse stacks
- Memory-constrained environments
- Batch parsing where arena can be reused

### Legacy Implementation: Rc GSS ([`gss.rs`](../../glr-core/src/gss.rs))

The Rc-based implementation is maintained for:
- Backward compatibility with existing code
- Scenarios where lifetime management is impractical
- Testing and debugging (simpler mental model)

### Selection Guidelines

| Scenario | Recommended Implementation |
|----------|---------------------------|
| Default parsing | Persistent Stack (`stack.rs`) |
| Deep stacks (>100 levels) | Arena GSS (`gss_arena.rs`) |
| Memory-constrained | Arena GSS (`gss_arena.rs`) |
| Quick prototyping | Rc GSS (`gss.rs`) |
| Testing infrastructure | Any (configurable) |

## Consequences

### Positive

- **Performance flexibility**: Different workloads can use optimal implementation
- **Memory efficiency**: Small vector optimization reduces allocation for common cases
- **Backward compatibility**: Existing code continues to work with Rc-based implementation
- **Clear selection criteria**: Guidelines help developers choose appropriate implementation

### Negative

- **Maintenance burden**: Three implementations require ongoing maintenance
- **API complexity**: Consumers must understand tradeoffs to select correctly
- **Testing overhead**: All implementations must pass identical test suites
- **Code duplication**: Similar logic implemented across three files

### Neutral

- The persistent stack implementation is the default for the production runtime2
- Arena implementation requires lifetime annotations that propagate through the codebase
- Future consolidation may be possible once performance characteristics are fully understood

## Related

- Related ADRs: [ADR-001](001-pure-rust-glr-implementation.md), [ADR-003](003-dual-runtime-strategy.md)
- Evidence: [`glr-core/src/gss.rs`](../../glr-core/src/gss.rs), [`glr-core/src/gss_arena.rs`](../../glr-core/src/gss_arena.rs), [`glr-core/src/stack.rs`](../../glr-core/src/stack.rs)
- See also: [`glr-core/src/perf_optimizations.rs`](../../glr-core/src/perf_optimizations.rs) for stack pooling
