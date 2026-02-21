# ADR 0001: Arena Allocator for Parse Trees

**Status**: Proposed
**Date**: 2025-01-20
**Authors**: adze maintainers
**Supersedes**: None
**Related**: v0.8.0 Performance Contract, Benchmarking Infrastructure

## Context

Current parse tree construction in adze allocates each tree node individually using `Box<TreeNode>` or similar heap allocations. For a large parse tree with thousands of nodes, this results in:

- **High allocation overhead**: Each node requires a separate `malloc()` call
- **Poor cache locality**: Nodes scattered across the heap
- **Allocator contention**: General allocator must handle many small allocations
- **Fragmentation**: Small allocations increase memory fragmentation

### Current Performance Baseline (v0.8.0-corrected)

From our corrected benchmarks:
- Small (50 ops): 46 µs → ~1.1M ops/sec
- Medium (200 ops): 224 µs → ~900K ops/sec
- Large (1000 ops): 1.18 ms → ~850K ops/sec

**Performance degradation** with scale suggests allocation overhead is a bottleneck.

### Goals from Performance Contract

- **≥50% reduction in allocation count**
- **≥30% reduction in peak memory usage**
- **≥20% speedup on large fixtures**

## Decision

We will implement a **typed arena allocator** for parse tree nodes with the following design:

### 1. Core Architecture

```rust
pub struct TreeArena {
    chunks: Vec<Chunk>,
    current_chunk: usize,
    current_offset: usize,
}

struct Chunk {
    data: Vec<TreeNode>,
    capacity: usize,
}

pub struct NodeHandle {
    chunk_idx: u32,
    node_idx: u32,
}
```

**Key Design Decisions**:

- **Typed arena**: Only allocate `TreeNode` types (not generic)
- **Chunked allocation**: Grow in chunks to avoid large contiguous allocations
- **Handle-based references**: Indirect pointers for lifetime safety
- **Reset capability**: Reuse arena across multiple parses

### 2. Lifetime Management

```rust
pub struct Tree<'arena> {
    root: NodeHandle,
    arena: &'arena TreeArena,
}

impl<'arena> Tree<'arena> {
    pub fn root(&self) -> TreeNodeRef<'arena> {
        self.arena.get(self.root)
    }
}
```

**Rationale**:
- Lifetime ties tree to arena
- Prevents use-after-free (tree can't outlive arena)
- Zero-cost abstraction (lifetimes erased at runtime)

### 3. Parser Integration

```rust
pub struct Parser {
    arena: TreeArena,
    // ... other fields
}

impl Parser {
    pub fn parse<'a>(&'a mut self, input: &str) -> Result<Tree<'a>> {
        self.arena.reset();  // Reuse for next parse
        // Build tree using arena...
        Ok(Tree { root, arena: &self.arena })
    }
}
```

**Benefits**:
- Parser owns arena (single allocation per parser)
- `reset()` reuses memory across parses
- Incremental parsing can selectively reset regions

### 4. Memory Layout

```
Chunk 0: [Node0][Node1][Node2]...[NodeN]
Chunk 1: [Node0][Node1][Node2]...[NodeM]
Chunk 2: [Node0][Node1]...
```

- **Initial chunk size**: 1024 nodes (~64KB for typical node size)
- **Growth strategy**: Exponential (2x previous chunk)
- **Maximum chunk size**: 64K nodes (~4MB) to avoid fragmentation

## Alternatives Considered

### 1. Bumpalo / Generic Arena

**Pros**: Battle-tested, flexible
**Cons**:
- Unsafe API requires careful usage
- Generic design has overhead we don't need
- No typed guarantees

**Decision**: Custom typed arena provides better ergonomics and safety for our specific use case.

### 2. Reference-Counted Nodes (Rc<TreeNode>)

**Pros**: Simple lifetime management
**Cons**:
- Reference counting overhead on every access
- No cache locality benefits
- Increased memory per node (refcount + pointer)

**Decision**: Arena provides better performance without runtime overhead.

### 3. Intrusive List (Nodes own next pointers)

**Pros**: Zero indirection
**Cons**:
- Complex tree structure modifications
- No ability to reuse memory
- Limits incremental parsing

**Decision**: Handle-based approach provides flexibility for future optimizations.

## Consequences

### Positive

✅ **Allocation Efficiency**: 1 allocation per chunk vs 1 per node
✅ **Cache Locality**: Nodes stored contiguously in memory
✅ **Reuse**: Arena can be reset and reused across parses
✅ **Memory Predictability**: Known upper bound based on input size
✅ **Safety**: Lifetimes prevent use-after-free at compile time
✅ **Benchmarkable**: Clear before/after comparison

### Negative

⚠️ **API Complexity**: Introduces lifetime parameter to `Tree<'arena>`
⚠️ **Migration Effort**: Existing code must be updated to use arena
⚠️ **Memory Footprint**: May use more memory than strictly necessary (chunk granularity)
⚠️ **FFI Implications**: Arena lifetime complicates C API boundaries

### Mitigations

- **API Complexity**: Provide wrapper types for common cases
- **Migration**: Phased rollout with feature flag
- **Memory Footprint**: Configurable chunk sizes, reset() for reuse
- **FFI**: Separate `OwnedTree` type for FFI that clones data

## Implementation Plan

### Phase 1: Core Arena ✅ COMPLETED (2025-01-20)

- [x] Implement `TreeArena` with chunk management
- [x] Implement `NodeHandle` and dereference logic
- [x] Add comprehensive unit tests (18/18 passing, Miri/ASan clean)
- [x] Benchmark allocation counts (99%+ reduction achieved)
- [x] Document API and usage
- [x] Verify 3.7x-5.0x speedup vs Box allocation

**Results**: Far exceeded all targets (99% allocation reduction, 3.7x-5.0x speedup)

**References**:
- Implementation: `runtime/src/arena_allocator.rs`
- Tests: `runtime/tests/arena_allocator_test.rs`
- Benchmarks: `benchmarks/benches/arena_vs_box_allocation.rs`
- Documentation: `docs/ARENA_ALLOCATOR.md`, `docs/guides/ARENA_ALLOCATOR_GUIDE.md`

### Phase 2: Parser Integration (IN PROGRESS - Week 2-3)

- [ ] Update `Parser` to own arena
- [ ] Modify tree construction to use arena
- [ ] Update `Tree<'arena>` type with lifetime parameter
- [ ] Update Node API for arena-allocated data
- [ ] Integration tests with existing grammars
- [ ] Measure end-to-end parsing performance

**Specifications**:
- Detailed spec: `docs/specs/PARSER_ARENA_INTEGRATION_SPEC.md`
- Implementation plan: `docs/implementation-plans/PARSER_ARENA_INTEGRATION.md`

### Phase 3: Measurement & Optimization (Week 4)

- [ ] Run benchmarks vs v0.8.0-corrected baseline
- [ ] Validate ≥50% reduction in allocations (end-to-end)
- [ ] Validate ≥20% speedup target (end-to-end)
- [ ] Tune chunk sizes based on real workloads
- [ ] Profile and optimize hot paths

### Phase 4: Documentation & Stabilization (Week 5)

- [ ] Update all API documentation
- [ ] Complete migration guide for consumers
- [ ] Performance analysis report
- [ ] Prepare for v0.8.0 release

## Success Criteria

Measured against v0.8.0-corrected baseline:

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Allocation Count | ≥50% reduction | Valgrind/massif on benchmarks |
| Peak Memory | ≥30% reduction | Heaptrack on large fixture |
| Parse Time | ≥20% speedup | Criterion benchmark comparison |
| Memory Reuse | Arena reset < 1µs | Microbenchmark |

**Acceptance**: All targets must be met on arithmetic grammar; 2/4 on complex grammars.

## References

- [Performance Contract v0.8.0](../contracts/V0.8.0_PERFORMANCE_CONTRACT.md)
- [Benchmarking Guide](../guides/PERFORMANCE_BENCHMARKING.md)
- [Baseline v0.8.0-corrected](../../baselines/v0.8.0-corrected.json)
- [Typed Arena Pattern](https://docs.rs/typed-arena/)
- [Rust Lifetimes Guide](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2025-01-20 | Use typed arena over generic | Type safety, ergonomics, performance |
| 2025-01-20 | Handle-based over raw pointers | Safety, flexibility for incremental parsing |
| 2025-01-20 | Chunked growth strategy | Balance between allocation overhead and fragmentation |
| 2025-01-20 | Parser owns arena | Natural reuse, clear ownership |
| 2025-01-20 | Tree borrows arena with lifetime | Compile-time safety, zero overhead |

## Review & Approval

**Status**: Phase 1 Complete and Validated ✅

**Phase 1**:
- **Implemented**: 2025-01-20
- **Verified**: Miri/ASan clean, 18/18 tests passing
- **Benchmarked**: 3.7x-5.0x speedup, 99%+ allocation reduction

**Phase 2**:
- **Status**: Specification complete, implementation in progress
- **Timeline**: Week 2-3 (2025-01-21 onwards)
