# Arena Allocator Quick Reference

> **Production Ready** - v0.8.0+

The arena allocator provides **3.7x-5.0x faster** parse tree allocation with **99%+ fewer allocations** compared to Box-based allocation.

## Quick Example

```rust
use rust_sitter::arena_allocator::{TreeArena, TreeNode};

let mut arena = TreeArena::new();

// Allocate nodes
let child1 = arena.alloc(TreeNode::leaf(1));
let child2 = arena.alloc(TreeNode::leaf(2));
let parent = arena.alloc(TreeNode::branch(vec![child1, child2]));

// Access nodes
assert_eq!(arena.get(child1).value(), 1);

// Reuse for next parse
arena.reset();
```

## Performance at a Glance

| Metric | Result | Target |
|--------|--------|--------|
| Speedup | **3.7x-5.0x** | ≥20% (1.2x) |
| Allocation Reduction | **99%+** | ≥50% |
| Memory Reuse | **Zero-cost reset** | N/A |

### Benchmark Results (10,000 nodes)

- **Arena**: 80.7 µs, ~10 allocations
- **Box**: 401 µs, 10,000 allocations
- **Speedup**: 5.0x

## When to Use

✅ **Use arena allocation when:**
- Parsing files (most common case)
- Building ASTs or parse trees
- Need predictable performance
- Want to reuse memory across parses

❌ **Consider alternatives when:**
- Nodes need individual lifetimes
- Tree must outlive parser
- Need to incrementally drop subtrees

## Key API

```rust
// Create arena
let mut arena = TreeArena::new();

// Allocate node → returns NodeHandle
let handle = arena.alloc(node);

// Access node → returns TreeNodeRef<'_>
let node = arena.get(handle);

// Reset for reuse
arena.reset();

// Metrics
arena.len()           // Node count
arena.capacity()      // Total capacity
arena.num_chunks()    // Chunk count
arena.memory_usage()  // Bytes used
```

## Safety Guarantees

✅ **Miri verified** - No undefined behavior
✅ **ASan verified** - No memory errors
✅ **Lifetime safe** - Compile-time prevention of use-after-free
✅ **Handle validation** - Debug assertions catch invalid handles

## Documentation

- **Full Guide**: [docs/guides/ARENA_ALLOCATOR_GUIDE.md](guides/ARENA_ALLOCATOR_GUIDE.md)
- **Design Rationale**: [docs/adr/0001-arena-allocator-for-parse-trees.md](adr/0001-arena-allocator-for-parse-trees.md)
- **Specification**: [docs/specs/ARENA_ALLOCATOR_SPEC.md](specs/ARENA_ALLOCATOR_SPEC.md)
- **Benchmark Results**: [benchmarks/results/arena_vs_box_summary.md](../benchmarks/results/arena_vs_box_summary.md)

## Testing

```bash
# Run tests
cargo test -p rust-sitter arena_allocator

# Memory safety
cargo +nightly miri test -p rust-sitter --test arena_allocator_test

# Benchmarks
cargo bench --bench arena_vs_box_allocation
```

## Status

- ✅ **Phase 1**: Core implementation (v0.8.0)
- 🚧 **Phase 2**: Parser integration (in progress)
- ⏳ **Phase 3**: Default in v0.9.0
- ⏳ **Phase 4**: Stabilize in v1.0.0
