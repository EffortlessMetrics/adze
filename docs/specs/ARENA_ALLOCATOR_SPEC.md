# Arena Allocator Specification

**Version**: 1.0
**Status**: Draft
**Related**: ADR-0001, v0.8.0 Performance Contract

## Overview

This specification defines the behavior, API, and performance characteristics of rust-sitter's arena allocator for parse tree nodes.

## Goals

1. **Performance**: Reduce allocation overhead by ≥50%
2. **Safety**: Prevent use-after-free through compile-time lifetime checks
3. **Ergonomics**: Minimal API surface with clear ownership semantics
4. **Reusability**: Support arena reset for parse session reuse

## Non-Goals

- ❌ Generic arena (only `TreeNode` allocation)
- ❌ Thread-safe concurrent allocation
- ❌ Arbitrary object lifetimes (tied to arena)
- ❌ Deallocation of individual nodes

## API Contract

### Core Types

```rust
/// Typed arena for allocating parse tree nodes
pub struct TreeArena {
    chunks: Vec<Chunk>,
    current_chunk: usize,
    current_offset: usize,
}

/// Opaque handle to a node in the arena
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeHandle {
    chunk_idx: u32,
    node_idx: u32,
}

/// Reference to a node in the arena
pub struct TreeNodeRef<'arena> {
    node: &'arena TreeNode,
}
```

### TreeArena Methods

#### Construction

```rust
impl TreeArena {
    /// Create a new arena with default capacity (1024 nodes)
    ///
    /// # Postconditions
    /// - Arena has one chunk allocated
    /// - Chunk capacity is INITIAL_CHUNK_SIZE
    /// - current_chunk is 0, current_offset is 0
    pub fn new() -> Self;

    /// Create arena with specific initial capacity
    ///
    /// # Preconditions
    /// - initial_capacity > 0
    ///
    /// # Postconditions
    /// - Arena has one chunk allocated with given capacity
    pub fn with_capacity(initial_capacity: usize) -> Self;
}
```

#### Allocation

```rust
impl TreeArena {
    /// Allocate a new tree node
    ///
    /// # Preconditions
    /// - Arena is not corrupted
    ///
    /// # Postconditions
    /// - Returns unique NodeHandle
    /// - Node is stored in arena
    /// - Handle remains valid until arena is reset or dropped
    ///
    /// # Performance
    /// - Amortized O(1) allocation
    /// - May allocate new chunk (O(n) where n = new chunk size)
    pub fn alloc(&mut self, node: TreeNode) -> NodeHandle;

    /// Get reference to node
    ///
    /// # Preconditions
    /// - handle was returned by alloc() on this arena
    /// - arena has not been reset since allocation
    ///
    /// # Postconditions
    /// - Returns reference with arena's lifetime
    ///
    /// # Panics
    /// - If handle is invalid (debug builds)
    ///
    /// # Performance
    /// - O(1) lookup
    pub fn get(&self, handle: NodeHandle) -> TreeNodeRef<'_>;

    /// Get mutable reference to node
    ///
    /// # Preconditions
    /// - Same as get()
    /// - Exclusive access to arena (&mut self)
    ///
    /// # Performance
    /// - O(1) lookup
    pub fn get_mut(&mut self, handle: NodeHandle) -> TreeNodeRefMut<'_>;
}
```

#### Reset and Reuse

```rust
impl TreeArena {
    /// Reset arena for reuse
    ///
    /// # Postconditions
    /// - All previous NodeHandles are invalidated
    /// - current_offset reset to 0
    /// - Chunks are retained (no deallocation)
    ///
    /// # Safety
    /// - Caller must ensure no references to nodes exist
    /// - Lifetime system enforces this at compile time
    ///
    /// # Performance
    /// - O(1) - just resets offsets
    pub fn reset(&mut self);

    /// Clear arena and free all chunks except first
    ///
    /// # Postconditions
    /// - Only first chunk remains (with initial capacity)
    /// - All handles invalidated
    ///
    /// # Performance
    /// - O(chunks) - deallocates excess chunks
    pub fn clear(&mut self);
}
```

#### Metrics

```rust
impl TreeArena {
    /// Get total number of nodes allocated
    pub fn len(&self) -> usize;

    /// Check if arena is empty
    pub fn is_empty(&self) -> bool;

    /// Get total capacity (across all chunks)
    pub fn capacity(&self) -> usize;

    /// Get number of chunks
    pub fn num_chunks(&self) -> usize;

    /// Get memory usage in bytes
    pub fn memory_usage(&self) -> usize;
}
```

### Tree Integration

```rust
/// Parse tree with arena-allocated nodes
pub struct Tree<'arena> {
    root: NodeHandle,
    arena: &'arena TreeArena,
}

impl<'arena> Tree<'arena> {
    /// Get root node reference
    pub fn root(&self) -> TreeNodeRef<'arena>;

    /// Walk tree depth-first
    pub fn walk(&self) -> TreeWalker<'arena>;

    /// Get node by handle
    pub fn get_node(&self, handle: NodeHandle) -> TreeNodeRef<'arena>;
}
```

## Behavioral Specifications

### Spec 1: Basic Allocation

**Given**: A new `TreeArena`
**When**: User calls `alloc(node)`
**Then**:
- Returns valid `NodeHandle`
- `get(handle)` returns reference to allocated node
- Node data matches input

**Test**:
```rust
#[test]
fn test_basic_allocation() {
    let mut arena = TreeArena::new();
    let node = TreeNode::new(/* ... */);
    let handle = arena.alloc(node.clone());
    let retrieved = arena.get(handle);
    assert_eq!(*retrieved, node);
}
```

### Spec 2: Multiple Allocations

**Given**: `TreeArena` with capacity N
**When**: User allocates N+1 nodes
**Then**:
- All allocations succeed
- New chunk is allocated automatically
- All handles remain valid
- `num_chunks() == 2`

**Test**:
```rust
#[test]
fn test_chunk_growth() {
    let mut arena = TreeArena::with_capacity(2);
    let h1 = arena.alloc(TreeNode::new(1));
    let h2 = arena.alloc(TreeNode::new(2));
    assert_eq!(arena.num_chunks(), 1);

    let h3 = arena.alloc(TreeNode::new(3));
    assert_eq!(arena.num_chunks(), 2);

    // All handles still valid
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.get(h3).value(), 3);
}
```

### Spec 3: Arena Reset

**Given**: Arena with N allocated nodes
**When**: User calls `reset()`
**Then**:
- `len() == 0`
- `capacity()` unchanged
- Previous handles are logically invalid (unsafe to use)
- New allocations reuse memory

**Test**:
```rust
#[test]
fn test_arena_reset() {
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::new(1));
    arena.alloc(TreeNode::new(2));
    let initial_capacity = arena.capacity();

    arena.reset();

    assert_eq!(arena.len(), 0);
    assert_eq!(arena.capacity(), initial_capacity);
    assert_eq!(arena.num_chunks(), 1); // Chunks retained
}
```

### Spec 4: Lifetime Safety

**Given**: Arena and allocated tree
**When**: User tries to use tree after arena is dropped
**Then**: Compilation error (lifetime violation)

**Test**:
```rust
// This should NOT compile
#[test]
fn test_lifetime_safety() {
    let tree = {
        let mut arena = TreeArena::new();
        let root = arena.alloc(TreeNode::new(1));
        Tree { root, arena: &arena }
    }; // arena dropped here

    // Compilation error: arena does not live long enough
    let _root = tree.root();
}
```

### Spec 5: Performance - Allocation Count

**Given**: Input with N nodes
**When**: Parsing creates parse tree
**Then**: Number of allocations ≤ log₂(N) + 1 (one per chunk)

**Test**:
```rust
#[test]
fn test_allocation_count() {
    let allocations = count_allocations(|| {
        let mut arena = TreeArena::new();
        for i in 0..10_000 {
            arena.alloc(TreeNode::new(i));
        }
    });

    // With default chunk size 1024, expect ~10 chunk allocations
    // vs 10,000 with individual Box allocation
    assert!(allocations < 20);
}
```

### Spec 6: Performance - Cache Locality

**Given**: Tree with N nodes allocated sequentially
**When**: Traversing tree depth-first
**Then**: Cache miss rate < 10% (most nodes in same cache line)

**Measurement**: Use `perf stat -e cache-misses` on benchmark

### Spec 7: Memory Reuse

**Given**: Arena after parsing input of size N
**When**: `reset()` called and parsing input of size M
**Then**:
- If M ≤ N: No new allocations
- If M > N: Only allocate additional chunks needed

**Test**:
```rust
#[test]
fn test_memory_reuse() {
    let mut arena = TreeArena::new();

    // First parse
    for i in 0..1000 {
        arena.alloc(TreeNode::new(i));
    }
    let capacity_after_first = arena.capacity();

    arena.reset();

    // Second parse (same size)
    for i in 0..1000 {
        arena.alloc(TreeNode::new(i));
    }

    // No new allocations
    assert_eq!(arena.capacity(), capacity_after_first);
}
```

## Error Handling

### Invalid Handle Access

**Behavior**: Debug builds panic, release builds return undefined reference

**Rationale**: Handles should never be invalid if API is used correctly. Lifetime system prevents most errors at compile time.

**Implementation**:
```rust
pub fn get(&self, handle: NodeHandle) -> TreeNodeRef<'_> {
    debug_assert!(self.is_valid_handle(handle), "Invalid node handle");
    unsafe { self.get_unchecked(handle) }
}
```

### Out of Memory

**Behavior**: Allocation returns `None` or panics

**Rationale**: Parse tree allocation failure is unrecoverable

**Implementation**: Configurable via feature flag:
- `panic-on-oom` (default): Panic
- `fallible-alloc`: Return `Option<NodeHandle>`

## Performance Characteristics

### Time Complexity

| Operation | Amortized | Worst Case |
|-----------|-----------|------------|
| `alloc()` | O(1) | O(n) when allocating new chunk |
| `get()` | O(1) | O(1) |
| `reset()` | O(1) | O(1) |
| `clear()` | O(chunks) | O(chunks) |

### Space Complexity

- **Per-node overhead**: 0 bytes (no individual allocation metadata)
- **Arena overhead**: O(chunks) = O(log N) for N nodes
- **Fragmentation**: At most (chunk_size - 1) nodes wasted per chunk

### Growth Strategy

```rust
fn next_chunk_size(&self, current_size: usize) -> usize {
    min(current_size * 2, MAX_CHUNK_SIZE)
}
```

- **Initial**: 1,024 nodes (~64KB)
- **Maximum**: 65,536 nodes (~4MB)
- **Rationale**: Balance between allocation frequency and fragmentation

## Thread Safety

**Not thread-safe**: Arena does not implement `Sync` or `Send` for mutable operations.

**Rationale**: Single-threaded parsing is the primary use case. Multi-threaded parsing can use one arena per thread.

**Future**: Consider `Send` for arena ownership transfer between threads.

## Memory Safety

### Invariants

1. **Valid handles**: All `NodeHandle` values returned by `alloc()` refer to valid memory
2. **Stable addresses**: Node addresses do not change after allocation (within chunk)
3. **Lifetime correctness**: References cannot outlive arena

### Verification

- **Miri**: Run all tests under Miri to detect UB
- **ASan**: Build with AddressSanitizer for use-after-free detection
- **Valgrind**: Check for memory leaks and invalid accesses

## Testing Strategy

### Unit Tests

- [ ] Basic allocation and retrieval
- [ ] Chunk growth and multi-chunk allocation
- [ ] Reset and memory reuse
- [ ] Edge cases (capacity boundaries, large allocations)

### Property Tests

- [ ] Handles remain valid across chunk growth
- [ ] Reset enables same capacity reuse
- [ ] No memory leaks (Valgrind verification)

### Integration Tests

- [ ] Parse trees use arena correctly
- [ ] Multiple parse sessions reuse arena
- [ ] Incremental parsing with selective reset

### Benchmark Tests

- [ ] Allocation count vs baseline (≥50% reduction)
- [ ] Parse time vs baseline (≥20% speedup)
- [ ] Memory usage vs baseline (≥30% reduction)

## Migration Path

### Phase 1: Feature Flag

Enable via Cargo feature:
```toml
[features]
arena-allocator = []
```

### Phase 2: Parallel Implementation

Maintain both implementations:
```rust
#[cfg(feature = "arena-allocator")]
pub struct Tree<'arena> { /* arena-based */ }

#[cfg(not(feature = "arena-allocator"))]
pub struct Tree { /* box-based */ }
```

### Phase 3: Default Switchover

After validation, make arena-allocator default:
```toml
[features]
default = ["arena-allocator"]
box-allocator = [] # Opt-out
```

### Phase 4: Deprecation

Remove box-allocator after one major version.

## Acceptance Criteria

- [ ] All API methods implemented and documented
- [ ] All behavioral specs pass
- [ ] Miri and ASan clean
- [ ] Performance targets met (≥50% alloc, ≥20% speed)
- [ ] Integration tests with arithmetic grammar pass
- [ ] Documentation and migration guide complete

## References

- [ADR-0001: Arena Allocator Decision](../adr/0001-arena-allocator-for-parse-trees.md)
- [Performance Contract v0.8.0](../contracts/V0.8.0_PERFORMANCE_CONTRACT.md)
- [Rust Lifetime Docs](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)
- [typed-arena crate](https://docs.rs/typed-arena/)
