# Arena Allocator Implementation Plan

**Status**: Ready to Implement
**Target**: v0.8.0
**Prerequisites**: ADR-0001, ARENA_ALLOCATOR_SPEC completed

## Current State Analysis

### Existing Code
- File exists: `runtime/src/arena_allocator.rs`
- **Status**: Stub implementation, NOT in use
- **Issues with current impl**:
  - Uses `RefCell` (runtime borrow checking overhead)
  - `ArenaRef` copies values (defeats caching benefits)
  - Interior mutability complicates lifetime safety
  - Unsafe `TypedArena` without proper lifetime tracking

### Usage Analysis
```bash
$ grep -r "Arena::" runtime/src/*.rs | grep -v arena_allocator
# No results - arena not used anywhere
```

**Conclusion**: Safe to replace with production-ready implementation.

## Implementation Strategy

### Phase 1: Core Arena (Day 1-2)

#### 1.1 Replace arena_allocator.rs
Replace stub with production implementation matching ADR-0001.

**File**: `runtime/src/arena_allocator.rs`

**Core Types**:
```rust
pub struct TreeArena {
    chunks: Vec<Chunk>,
    current_chunk_idx: usize,
    current_offset: usize,
}

struct Chunk {
    data: Vec<TreeNode>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeHandle {
    chunk_idx: u32,
    node_idx: u32,
}

pub struct TreeNodeRef<'arena> {
    node: &'arena TreeNode,
}
```

**Key Methods**:
- `TreeArena::new()` - default capacity
- `TreeArena::with_capacity(usize)` - custom initial capacity
- `alloc(&mut self, TreeNode) -> NodeHandle` - allocate node
- `get(&self, NodeHandle) -> TreeNodeRef<'_>` - immutable access
- `get_mut(&mut self, NodeHandle) -> TreeNodeRefMut<'_>` - mutable access
- `reset(&mut self)` - reuse arena
- `clear(&mut self)` - free excess chunks

#### 1.2 Test Suite
**File**: `runtime/src/arena_allocator.rs` (tests module)

Tests from spec:
- [x] Basic allocation and retrieval
- [x] Multi-chunk growth
- [x] Handle validity across chunks
- [x] Reset and reuse
- [x] Clear behavior
- [x] Metrics (len, capacity, num_chunks)

**Property tests** (if time permits):
- Random allocation/access patterns
- Stress test with large node counts

#### 1.3 Safety Verification
```bash
# Run under Miri
cargo +nightly miri test -p rust-sitter arena_allocator

# Build with ASan
RUSTFLAGS="-Z sanitizer=address" cargo test -p rust-sitter arena_allocator

# Check with Valgrind
valgrind --leak-check=full cargo test -p rust-sitter arena_allocator
```

### Phase 2: Tree Integration (Day 3-4)

#### 2.1 Define Tree Types
**File**: `runtime/src/tree.rs` or integrate into existing tree module

```rust
pub struct Tree<'arena> {
    root: NodeHandle,
    arena: &'arena TreeArena,
}

impl<'arena> Tree<'arena> {
    pub fn root(&self) -> TreeNodeRef<'arena> {
        self.arena.get(self.root)
    }

    pub fn walk(&self) -> TreeWalker<'arena> {
        TreeWalker::new(self)
    }
}
```

#### 2.2 Update Parser
Identify parser entry point (need to find current tree construction).

**Search for**:
```bash
grep -r "TreeNode" runtime/src/*.rs | grep -i "new\|alloc\|build"
```

**Update to**:
```rust
pub struct Parser {
    arena: TreeArena,
    // ... other fields
}

impl Parser {
    pub fn parse<'a>(&'a mut self, input: &str) -> Result<Tree<'a>> {
        self.arena.reset();

        // Build tree using arena.alloc() instead of Box::new()
        let root_handle = self.build_tree_with_arena(input)?;

        Ok(Tree {
            root: root_handle,
            arena: &self.arena,
        })
    }

    fn build_tree_with_arena(&mut self, input: &str) -> Result<NodeHandle> {
        // TODO: Update tree construction logic
    }
}
```

#### 2.3 Integration Tests
**File**: `runtime/tests/arena_integration_test.rs`

- Parse with arithmetic grammar
- Verify tree structure correct
- Test multiple parses (arena reuse)
- Incremental parsing compatibility

### Phase 3: Benchmarking (Day 5)

#### 3.1 Allocation Count Benchmark
**File**: `benchmarks/benches/arena_vs_box_allocation.rs`

```rust
fn bench_box_allocation(c: &mut Criterion) {
    c.bench_function("box_alloc_1000", |b| {
        b.iter(|| {
            let nodes: Vec<_> = (0..1000)
                .map(|i| Box::new(TreeNode::new(i)))
                .collect();
            black_box(nodes);
        });
    });
}

fn bench_arena_allocation(c: &mut Criterion) {
    c.bench_function("arena_alloc_1000", |b| {
        let mut arena = TreeArena::new();
        b.iter(|| {
            arena.reset();
            let handles: Vec<_> = (0..1000)
                .map(|i| arena.alloc(TreeNode::new(i)))
                .collect();
            black_box(handles);
        });
    });
}
```

**Measure with**:
```bash
# Allocation count
valgrind --tool=massif cargo bench --bench arena_vs_box_allocation

# Parse through counting
cargo bench --bench arena_vs_box_allocation | grep "alloc"
```

#### 3.2 Parse Performance Benchmark
**File**: Update `benchmarks/benches/glr_performance_real.rs`

Add feature flag to toggle arena:
```rust
#[cfg(feature = "arena-allocator")]
fn benchmark_with_arena(c: &mut Criterion) {
    // existing benchmark logic
}

#[cfg(not(feature = "arena-allocator"))]
fn benchmark_without_arena(c: &mut Criterion) {
    // baseline
}
```

**Run comparison**:
```bash
# Baseline (without arena)
cargo bench -p rust-sitter-benchmarks --bench glr_performance_real

# With arena
cargo bench -p rust-sitter-benchmarks --bench glr_performance_real --features arena-allocator

# Compare
cargo xtask compare-baseline v0.8.0-corrected --threshold 5
```

#### 3.3 Memory Profile
```bash
# Baseline
heaptrack cargo bench --bench glr_performance_real

# With arena
heaptrack cargo bench --bench glr_performance_real --features arena-allocator

# Analyze
heaptrack_gui heaptrack.cargo.*.gz
```

### Phase 4: Documentation & Polish (Day 6)

#### 4.1 API Documentation
Add rustdoc to all public items:
- Module-level docs explaining usage
- Examples for common patterns
- Performance characteristics
- Safety guarantees

#### 4.2 Migration Guide
**File**: `docs/guides/ARENA_MIGRATION_GUIDE.md`

Content:
- Why arena allocation?
- API changes (lifetime parameters)
- Code examples (before/after)
- Performance improvements
- Troubleshooting

#### 4.3 Performance Report
**File**: `docs/reports/ARENA_ALLOCATOR_PERFORMANCE.md`

Include:
- Allocation count comparison (table)
- Parse time comparison (graphs from Criterion)
- Memory usage comparison (heaptrack screenshots)
- Analysis of results
- Conclusions and recommendations

## Success Criteria Checklist

- [ ] Core arena implementation passes all spec tests
- [ ] Miri, ASan, Valgrind clean
- [ ] Parser integration works with arithmetic grammar
- [ ] ≥50% reduction in allocation count (measured)
- [ ] ≥20% speedup in parse time (measured)
- [ ] ≥30% reduction in peak memory (measured)
- [ ] All benchmarks updated with arena feature flag
- [ ] Documentation complete (API docs, migration guide, perf report)
- [ ] CI tests pass
- [ ] Code reviewed and approved

## Risk Mitigation

### Risk: Lifetime complexity breaks existing code
**Mitigation**: Feature flag allows gradual rollout
```toml
[features]
arena-allocator = []
```

### Risk: Performance targets not met
**Mitigation**:
1. Profile to find bottlenecks
2. Tune chunk sizes
3. Consider handle caching
4. Document actual improvements even if below target

### Risk: FFI compatibility issues
**Mitigation**: Create `OwnedTree` wrapper for C API
```rust
pub struct OwnedTree {
    data: Vec<TreeNode>,
    // ... tree structure
}

impl From<Tree<'_>> for OwnedTree {
    fn from(tree: Tree<'_>) -> Self {
        // Clone tree data out of arena
    }
}
```

## Timeline Estimate

| Phase | Days | Tasks |
|-------|------|-------|
| Phase 1: Core Arena | 2 | Implement + test core types |
| Phase 2: Integration | 2 | Update parser, tree types |
| Phase 3: Benchmarking | 1 | Measure improvements |
| Phase 4: Documentation | 1 | Docs, reports, review |
| **Total** | **6** | **Full implementation** |

## Next Actions

1. ✅ ADR created
2. ✅ Spec written
3. ✅ Implementation plan drafted
4. ⏭️ Start Phase 1: Implement core arena
5. ⏭️ Write TDD tests for arena
6. ⏭️ Implement arena methods
7. ⏭️ Run safety checks (Miri/ASan)
8. ⏭️ Continue with Phase 2...

## References

- [ADR-0001](../adr/0001-arena-allocator-for-parse-trees.md)
- [Arena Allocator Spec](../specs/ARENA_ALLOCATOR_SPEC.md)
- [Performance Contract v0.8.0](../contracts/V0.8.0_PERFORMANCE_CONTRACT.md)
- [Benchmarking Guide](../guides/PERFORMANCE_BENCHMARKING.md)
