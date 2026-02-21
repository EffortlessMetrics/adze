# Parser Arena Integration Implementation Plan

**Phase**: 2 of 4 (Arena Allocator Rollout)
**Status**: Planning
**Timeline**: 2 weeks
**Dependencies**: Phase 1 (Arena Allocator Core) ✅

## Executive Summary

Integrate the arena allocator into adze's parser and tree types, enabling zero-copy parsing with memory reuse across parse sessions while maintaining lifetime safety.

## Current State Analysis

### Existing Architecture

From `runtime/src/lib.rs` analysis:

```rust
// Current (GLR) parser is parser_v4
#[cfg(feature = "glr")]
pub use super::parser_v4::*;

// Trees are defined in multiple places:
// - parser_v4.rs: GLR parser tree
// - pure_incremental.rs: Incremental parsing tree
// - ts_compat/mod.rs: Tree-sitter compatibility tree
```

### Current Tree Allocation

```rust
// Current approach (parser_v4.rs)
pub struct Tree {
    root: Box<TreeNode>,  // Individual heap allocation
    // ...
}

struct TreeNode {
    children: Vec<Box<TreeNode>>,  // More allocations
    // ...
}
```

**Problems**:
- Each node = 1 allocation
- Poor cache locality
- No memory reuse

### Target Architecture

```rust
pub struct Parser {
    arena: TreeArena,  // Single arena per parser
    // ... existing fields
}

pub struct Tree<'arena> {
    root: NodeHandle,  // Handle into arena
    arena: &'arena TreeArena,  // Borrowed reference
    // ... existing fields
}
```

## Implementation Roadmap

### Week 1: Core Integration

#### Day 1: Design and Planning

- [x] Create PARSER_ARENA_INTEGRATION_SPEC.md
- [ ] Review spec with stakeholders
- [ ] Update ADR-0001 with Phase 2 details
- [ ] Create test plan

**Deliverables**: Approved spec, updated ADR

#### Day 2: TreeNodeData Design

Goal: Define arena-allocated node data structure

```rust
// runtime/src/tree_node_data.rs
pub struct TreeNodeData {
    symbol: SymbolId,
    start_byte: usize,
    end_byte: usize,
    children: SmallVec<[NodeHandle; 4]>,  // Inline for small node counts
    named_children_count: u16,
    is_named: bool,
    is_extra: bool,
}
```

**Tasks**:
- [ ] Design TreeNodeData struct
- [ ] Implement Copy/Clone if possible (or optimize layout)
- [ ] Add tests for node data creation
- [ ] Benchmark node data size (keep ≤64 bytes)

**Deliverables**: `runtime/src/tree_node_data.rs`, tests

#### Day 3: Update Parser Type

Goal: Add arena field to Parser

```rust
// runtime/src/parser_v4.rs (or new parser_v5.rs)

pub struct Parser {
    arena: TreeArena,
    language: Language,
    // ... existing fields
}

impl Parser {
    pub fn new() -> Self {
        Self {
            arena: TreeArena::new(),
            // ... existing initialization
        }
    }

    pub fn with_arena_capacity(capacity: usize) -> Self {
        Self {
            arena: TreeArena::with_capacity(capacity),
            // ... existing initialization
        }
    }

    pub fn arena_metrics(&self) -> ArenaMetrics {
        ArenaMetrics {
            len: self.arena.len(),
            capacity: self.arena.capacity(),
            num_chunks: self.arena.num_chunks(),
            memory_usage: self.arena.memory_usage(),
        }
    }
}
```

**Tasks**:
- [ ] Add arena field to Parser
- [ ] Update Parser::new()
- [ ] Add with_arena_capacity() constructor
- [ ] Add arena_metrics() accessor
- [ ] Update all Parser tests

**Deliverables**: Updated Parser type, passing tests

#### Day 4: Update Tree Type

Goal: Add lifetime parameter and arena reference

```rust
// runtime/src/parser_v4.rs

pub struct Tree<'arena> {
    root: NodeHandle,
    arena: &'arena TreeArena,
    source: &'arena str,
    // ... existing fields (updated for handles)
}

impl<'arena> Tree<'arena> {
    pub fn root_node(&self) -> Node<'arena> {
        Node {
            handle: self.root,
            arena: self.arena,
        }
    }

    pub fn walk(&self) -> TreeCursor<'arena> {
        TreeCursor::new(self)
    }
}
```

**Tasks**:
- [ ] Add 'arena lifetime parameter to Tree
- [ ] Replace Box<TreeNode> with NodeHandle
- [ ] Add arena reference field
- [ ] Update Tree methods to return Node<'arena>
- [ ] Update Tree tests (expect compilation errors, fix them)

**Deliverables**: Updated Tree type, API changes documented

#### Day 5: Update Parse Method

Goal: Modify parse() to use arena and return Tree<'a>

```rust
impl Parser {
    pub fn parse<'a>(&'a mut self, input: &str) -> Result<Tree<'a>> {
        // Reset arena for fresh parse
        self.arena.reset();

        // Parse using existing logic, but allocate into arena
        let root_handle = self.parse_internal(input)?;

        Ok(Tree {
            root: root_handle,
            arena: &self.arena,
            source: input,
            // ...
        })
    }

    fn parse_internal(&mut self, input: &str) -> Result<NodeHandle> {
        // Existing parse logic, modified to use arena.alloc()
        // Instead of: Box::new(TreeNode { ... })
        // Use: self.arena.alloc(TreeNodeData { ... })
    }
}
```

**Tasks**:
- [ ] Update parse() signature to return Tree<'a>
- [ ] Add arena.reset() at start of parse
- [ ] Modify tree construction to use arena.alloc()
- [ ] Update parse tests
- [ ] Verify lifetime constraints work as expected

**Deliverables**: Working parse() method, passing tests

### Week 2: Node API and Optimization

#### Day 6: Node Type Implementation

Goal: Create Node<'arena> wrapper for arena-allocated nodes

```rust
// runtime/src/node.rs

pub struct Node<'arena> {
    handle: NodeHandle,
    arena: &'arena TreeArena,
}

impl<'arena> Node<'arena> {
    pub fn kind(&self) -> &str {
        let data = self.arena.get(self.handle);
        // Lookup symbol name from data
    }

    pub fn children(&self) -> impl Iterator<Item = Node<'arena>> + 'arena {
        let data = self.arena.get(self.handle);
        data.children.iter().map(move |&child_handle| {
            Node {
                handle: child_handle,
                arena: self.arena,
            }
        })
    }

    pub fn named_children(&self) -> impl Iterator<Item = Node<'arena>> + 'arena {
        self.children().filter(|n| n.is_named())
    }

    pub fn is_named(&self) -> bool {
        self.arena.get(self.handle).is_named
    }

    // ... other node methods
}
```

**Tasks**:
- [ ] Implement Node<'arena> struct
- [ ] Add accessor methods (kind, children, parent, etc.)
- [ ] Implement iterator for children
- [ ] Add convenience methods (child_by_field_name, etc.)
- [ ] Write comprehensive Node tests

**Deliverables**: `runtime/src/node.rs`, tests

#### Day 7: TreeCursor Update

Goal: Update TreeCursor to work with arena handles

```rust
pub struct TreeCursor<'arena> {
    current: NodeHandle,
    stack: Vec<NodeHandle>,
    arena: &'arena TreeArena,
}

impl<'arena> TreeCursor<'arena> {
    pub fn goto_first_child(&mut self) -> bool {
        let data = self.arena.get(self.current);
        if let Some(&first_child) = data.children.first() {
            self.stack.push(self.current);
            self.current = first_child;
            true
        } else {
            false
        }
    }

    pub fn goto_next_sibling(&mut self) -> bool {
        // Navigate to next sibling using handles
    }

    pub fn goto_parent(&mut self) -> bool {
        if let Some(parent) = self.stack.pop() {
            self.current = parent;
            true
        } else {
            false
        }
    }
}
```

**Tasks**:
- [ ] Update TreeCursor to use NodeHandle
- [ ] Modify navigation methods (goto_first_child, etc.)
- [ ] Add arena reference
- [ ] Update TreeCursor tests
- [ ] Benchmark cursor performance

**Deliverables**: Updated TreeCursor, passing tests

#### Day 8-9: Integration Testing

Goal: Verify integration with example grammars

**Tasks**:
- [ ] Update arithmetic grammar example
- [ ] Update optional grammar example
- [ ] Update repetition grammar example
- [ ] Run all integration tests
- [ ] Fix any edge cases discovered
- [ ] Verify no test regressions

**Deliverables**: All example grammars work, all tests pass

#### Day 10: Performance Benchmarking

Goal: Measure real-world performance improvements

**Benchmark Plan**:

1. **Parse Time Benchmark**
   ```rust
   // benchmarks/benches/parse_with_arena.rs
   fn bench_arithmetic_small(b: &mut Bencher) {
       let mut parser = Parser::new();
       parser.set_language(Language::arithmetic());

       b.iter(|| {
           let tree = parser.parse(ARITH_SMALL).unwrap();
           black_box(tree);
       });
   }
   ```

2. **Memory Reuse Benchmark**
   ```rust
   fn bench_parse_reuse(b: &mut Bencher) {
       let mut parser = Parser::new();
       parser.set_language(Language::arithmetic());

       // Warm up arena
       parser.parse(ARITH_MEDIUM).unwrap();

       b.iter(|| {
           let tree = parser.parse(ARITH_MEDIUM).unwrap();
           black_box(tree);
       });
   }
   ```

**Tasks**:
- [ ] Create parse_with_arena benchmark
- [ ] Run baseline comparison vs v0.8.0-corrected
- [ ] Measure allocation counts (Valgrind)
- [ ] Profile hot paths (perf/flamegraph)
- [ ] Document results

**Deliverables**: Benchmark results, performance analysis

#### Day 11: Safety Verification

Goal: Verify memory safety and correctness

**Tasks**:
- [ ] Run Miri on all arena-integrated tests
- [ ] Run ASan on integration tests
- [ ] Valgrind memory leak check
- [ ] Test lifetime error messages (compile-fail tests)
- [ ] Document safety guarantees

**Deliverables**: Clean Miri/ASan/Valgrind, safety report

#### Day 12: Documentation

Goal: Complete user-facing documentation

**Tasks**:
- [ ] Update Parser API docs with lifetime examples
- [ ] Create migration guide (Box → Arena)
- [ ] Update CLAUDE.md with arena integration notes
- [ ] Add troubleshooting section for common errors
- [ ] Create examples showing typical usage patterns

**Deliverables**: Complete documentation

## Testing Strategy

### Unit Tests (Day-by-Day)

Each component gets comprehensive unit tests:

- TreeNodeData: Construction, field access, small/large node variants
- Parser: Creation, arena initialization, metrics
- Tree: Lifetime safety, root access, arena reference
- Node: Navigation, accessors, iterators
- TreeCursor: Navigation, position tracking

### Integration Tests (Day 8-9)

Test full parse workflows:

```rust
#[test]
fn integration_parse_arithmetic() {
    let mut parser = Parser::new();
    parser.set_language(Language::arithmetic());

    let input = "1 + 2 * 3 - 4 / 5";
    let tree = parser.parse(input).unwrap();

    let root = tree.root_node();
    assert_eq!(root.kind(), "expression");
    assert!(root.child_count() > 0);
}

#[test]
fn integration_multiple_parses() {
    let mut parser = Parser::new();
    parser.set_language(Language::arithmetic());

    for input in ARITHMETIC_INPUTS {
        let tree = parser.parse(input).unwrap();
        assert!(tree.root_node().is_some());
    }

    // Verify memory reuse
    assert!(parser.arena_metrics().num_chunks() < 10);
}
```

### Compile-Fail Tests (Day 11)

Verify lifetime errors are caught:

```rust
// tests/compile-fail/tree_outlives_parser.rs
fn main() {
    let tree = {
        let mut parser = Parser::new();
        parser.parse("1 + 2").unwrap()
    }; // Should fail: parser dropped

    let _ = tree.root_node();
}
```

### Performance Tests (Day 10)

Benchmark suite:

- Parse time vs baseline (expect ≥20% improvement)
- Allocation count (expect ≥50% reduction)
- Memory reuse (expect no allocations after warmup)
- Cursor navigation (expect no regression)

## Risk Mitigation

### Risk 1: Lifetime Errors Too Complex

**Symptom**: Users struggle with `Tree<'arena>` lifetimes

**Mitigation**:
- Add helper functions for common patterns
- Extensive documentation with examples
- Consider wrapper types for simpler use cases

### Risk 2: Performance Regression

**Symptom**: Arena overhead negates allocation improvements

**Mitigation**:
- Profile at each step
- Benchmark each commit
- Optimize hot paths (NodeHandle dereference, etc.)

### Risk 3: API Breaking Changes

**Symptom**: Existing code breaks extensively

**Mitigation**:
- Feature flag during development
- Deprecation warnings before removal
- Migration guide with examples

### Risk 4: Integration Bugs

**Symptom**: Subtle bugs in tree construction

**Mitigation**:
- Comprehensive test coverage
- Fuzzing with various grammars
- Golden test comparison (outputs match Box version)

## Success Criteria

### Functional Requirements

- [ ] All existing parser tests pass
- [ ] All example grammars parse correctly
- [ ] Tree navigation API works identically
- [ ] No memory leaks (Valgrind clean)
- [ ] No undefined behavior (Miri clean)

### Performance Requirements

Measured against v0.8.0-corrected baseline:

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Parse Time | ≥20% speedup | Criterion benchmark |
| Allocation Count | ≥50% reduction | Valgrind/massif |
| Memory Reuse | Arena reset < 1µs | Microbenchmark |
| No Regression | Node access ≤5% slower | Microbenchmark |

### Quality Requirements

- [ ] API documentation complete
- [ ] Migration guide written
- [ ] Troubleshooting guide created
- [ ] Examples demonstrate common patterns

## Rollout Plan

### Phase 2.1: Development (Week 1-2)

- Feature flag: `arena-allocator` (default enabled for testing)
- All new code behind feature flag
- Box implementation remains as fallback

### Phase 2.2: Validation (Week 3)

- Extended testing period
- Performance validation
- User feedback on API

### Phase 2.3: Stabilization (Week 4)

- Make arena-allocator default
- Deprecate Box implementation
- Update CI to test both paths

### Phase 2.4: Cleanup (v0.9.0)

- Remove Box implementation
- Remove feature flag
- Arena is the only implementation

## Dependencies

### Completed

- ✅ Phase 1: Arena Allocator Core
- ✅ Arena allocator tests (18/18 passing)
- ✅ Arena benchmark (3.7x-5.0x speedup verified)
- ✅ Safety verification (Miri/ASan clean)

### Required

- Parser type refactoring
- Tree type lifetime parameter
- Node API redesign
- TreeCursor update

### Optional

- Incremental parsing integration (Phase 3)
- C FFI compatibility layer (separate)
- Thread-local arena optimization (future)

## References

- [PARSER_ARENA_INTEGRATION_SPEC.md](../specs/PARSER_ARENA_INTEGRATION_SPEC.md)
- [ADR-0001: Arena Allocator](../adr/0001-arena-allocator-for-parse-trees.md)
- [ARENA_ALLOCATOR_SPEC.md](../specs/ARENA_ALLOCATOR_SPEC.md)
- [v0.8.0 Performance Contract](../contracts/V0.8.0_PERFORMANCE_CONTRACT.md)
