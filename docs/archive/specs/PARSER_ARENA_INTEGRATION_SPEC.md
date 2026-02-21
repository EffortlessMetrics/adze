# Parser Arena Integration Specification

**Version**: 1.0
**Status**: Draft
**Related**: ADR-0001, ARENA_ALLOCATOR_SPEC.md, v0.8.0 Performance Contract

## Overview

This specification defines the integration of the arena allocator into adze's parser and tree types, completing Phase 2 of the arena allocator implementation.

## Goals

1. **Zero-copy parsing**: Parse trees reference arena-allocated nodes directly
2. **Memory reuse**: Parser arena persists across parse calls
3. **Lifetime safety**: Compile-time prevention of tree/arena lifetime mismatches
4. **API compatibility**: Minimize breaking changes to existing parser API
5. **Performance**: Maintain arena allocator's 3.7x-5.0x speedup in real parsing

## Non-Goals

- ❌ Supporting both arena and Box allocation simultaneously (feature flag only)
- ❌ Thread-safe shared arena (each parser owns its arena)
- ❌ Incremental parsing integration (Phase 3)
- ❌ C FFI compatibility (requires separate owned tree type)

## Design Decisions

### 1. Parser Owns Arena

```rust
pub struct Parser {
    arena: TreeArena,
    // ... existing fields
}
```

**Rationale**:
- Single allocation per parser instance
- Natural fit for reuse across parse calls
- Clear ownership semantics

**Alternative rejected**: Thread-local arena (complex, unclear ownership)

### 2. Tree Borrows Arena Reference

```rust
pub struct Tree<'arena> {
    root: NodeHandle,
    arena: &'arena TreeArena,
    // ... existing fields
}
```

**Rationale**:
- Lifetime ties tree to parser/arena
- Prevents use-after-free at compile time
- Zero runtime overhead

**Alternative rejected**: `Rc<TreeArena>` (runtime overhead, unclear when to free)

### 3. Parse Method Signature

```rust
impl Parser {
    pub fn parse<'a>(&'a mut self, input: &str) -> Result<Tree<'a>> {
        self.arena.reset();
        // ... build tree
        Ok(Tree { root, arena: &self.arena, ... })
    }
}
```

**Rationale**:
- `&mut self` ensures exclusive access during parse
- Return type `Tree<'a>` ties tree lifetime to parser borrow
- Automatic reset ensures clean state

## API Contract

### Parser Type

```rust
pub struct Parser {
    arena: TreeArena,
    language: Language,
    // ... existing fields
}

impl Parser {
    /// Create a new parser
    pub fn new() -> Self;

    /// Set the language for parsing
    pub fn set_language(&mut self, language: Language);

    /// Parse input text
    ///
    /// # Lifetime
    ///
    /// The returned tree borrows the parser's arena.
    /// The tree is valid until the next `parse()` call or parser drop.
    ///
    /// # Performance
    ///
    /// - First parse: Allocates arena chunks as needed
    /// - Subsequent parses: Reuses arena memory (zero allocations if input size ≤ previous)
    pub fn parse<'a>(&'a mut self, input: &str) -> Result<Tree<'a>>;

    /// Get arena metrics
    pub fn arena_metrics(&self) -> ArenaMetrics;
}
```

### Tree Type

```rust
pub struct Tree<'arena> {
    root: NodeHandle,
    arena: &'arena TreeArena,
    source: &'arena str,
}

impl<'arena> Tree<'arena> {
    /// Get the root node
    pub fn root_node(&self) -> Node<'arena>;

    /// Walk the tree
    pub fn walk(&self) -> TreeCursor<'arena>;

    /// Get node by handle
    pub fn get_node(&self, handle: NodeHandle) -> Node<'arena>;
}
```

### Node Type

```rust
pub struct Node<'arena> {
    handle: NodeHandle,
    arena: &'arena TreeArena,
}

impl<'arena> Node<'arena> {
    /// Get node kind/symbol
    pub fn kind(&self) -> &str;

    /// Get child nodes
    pub fn children(&self) -> impl Iterator<Item = Node<'arena>>;

    /// Get named children
    pub fn named_children(&self) -> impl Iterator<Item = Node<'arena>>;

    /// Access node data
    pub fn data(&self) -> &TreeNodeData;
}
```

## Behavioral Specifications

### Spec 1: Parser Creation and Setup

**Given**: New Parser
**When**: User creates parser and sets language
**Then**:
- Parser has arena with default capacity (1024 nodes)
- Arena is empty (len == 0)
- Language is set correctly

**Test**:
```rust
#[test]
fn spec_1_parser_creation() {
    let mut parser = Parser::new();
    parser.set_language(Language::arithmetic());

    // Arena exists but is empty
    assert_eq!(parser.arena_metrics().len(), 0);
    assert!(parser.arena_metrics().capacity() > 0);
}
```

### Spec 2: Single Parse

**Given**: Parser with language set
**When**: User calls `parse(input)`
**Then**:
- Tree is constructed using arena
- Tree lifetime tied to parser borrow
- Arena contains exactly the nodes in the tree

**Test**:
```rust
#[test]
fn spec_2_single_parse() {
    let mut parser = Parser::new();
    parser.set_language(Language::arithmetic());

    let input = "1 + 2";
    let tree = parser.parse(input).unwrap();

    // Tree is valid
    assert!(tree.root_node().is_some());

    // Arena has nodes
    let metrics = parser.arena_metrics();
    assert!(metrics.len() > 0);
}
```

### Spec 3: Multiple Parses with Memory Reuse

**Given**: Parser that has parsed input once
**When**: User calls `parse()` again with same-sized input
**Then**:
- Arena is reset before parsing
- No new chunk allocations (reuses existing capacity)
- Second parse succeeds

**Test**:
```rust
#[test]
fn spec_3_memory_reuse() {
    let mut parser = Parser::new();
    parser.set_language(Language::arithmetic());

    // First parse
    let input1 = "1 + 2 * 3";
    {
        let _tree1 = parser.parse(input1).unwrap();
    }
    let capacity_after_first = parser.arena_metrics().capacity();

    // Second parse (same size)
    let input2 = "4 - 5 / 6";
    {
        let _tree2 = parser.parse(input2).unwrap();
    }

    // No new allocations
    assert_eq!(parser.arena_metrics().capacity(), capacity_after_first);
}
```

### Spec 4: Lifetime Safety (Compile-Time)

**Given**: Parser and parsed tree
**When**: User tries to use tree after parser is dropped
**Then**: Compilation error

**Test**:
```rust
// This should NOT compile
/*
#[test]
fn spec_4_lifetime_safety() {
    let tree = {
        let mut parser = Parser::new();
        parser.set_language(Language::arithmetic());
        parser.parse("1 + 2").unwrap()
    }; // parser dropped here

    // Compilation error: parser doesn't live long enough
    let _root = tree.root_node();
}
*/
```

### Spec 5: Arena Growth for Large Inputs

**Given**: Parser with small initial capacity
**When**: User parses large input
**Then**:
- Arena allocates new chunks as needed
- All nodes are accessible
- Parse completes successfully

**Test**:
```rust
#[test]
fn spec_5_arena_growth() {
    let mut parser = Parser::with_arena_capacity(10);
    parser.set_language(Language::arithmetic());

    // Large input requiring >10 nodes
    let input = "1 + 2 - 3 * 4 / 5 + 6 - 7 * 8 / 9 + 10";
    let tree = parser.parse(input).unwrap();

    assert!(tree.root_node().is_some());
    assert!(parser.arena_metrics().len() > 10);
    assert!(parser.arena_metrics().num_chunks() > 1);
}
```

### Spec 6: Tree Invalidation After Next Parse

**Given**: Tree from first parse
**When**: User calls `parse()` again
**Then**:
- First tree becomes invalid (arena reset)
- Must not access first tree after second parse

**Test**:
```rust
#[test]
fn spec_6_tree_invalidation() {
    let mut parser = Parser::new();
    parser.set_language(Language::arithmetic());

    let input1 = "1 + 2";
    let tree1 = parser.parse(input1).unwrap();
    let root1 = tree1.root_node();

    // This is safe: tree1 still borrows parser
    assert!(root1.is_some());

    // But we can't parse again while tree1 exists
    // because parse() needs &mut self

    // drop(tree1); // Explicit drop to end borrow
    // let tree2 = parser.parse("3 + 4").unwrap();
}
```

### Spec 7: Performance - No Regression vs Box Baseline

**Given**: Parser with arena allocator
**When**: Parsing benchmark inputs
**Then**:
- Parse time improvements align with arena benchmark (3.7x-5.0x)
- No performance regression in actual parsing workload

**Test**: (Benchmark, not unit test)
```rust
#[bench]
fn bench_parse_with_arena(b: &mut Bencher) {
    let mut parser = Parser::new();
    parser.set_language(Language::arithmetic());

    b.iter(|| {
        let tree = parser.parse(LARGE_INPUT).unwrap();
        black_box(tree);
    });
}
```

## Migration Strategy

### Phase 2.1: Feature Flag (Week 1)

Enable via Cargo feature during development:

```toml
[features]
default = ["arena-allocator"]
arena-allocator = []
box-allocator = []  # Legacy
```

### Phase 2.2: Parallel Implementation (Week 1)

Maintain both implementations during transition:

```rust
#[cfg(feature = "arena-allocator")]
pub struct Parser {
    arena: TreeArena,
    // ...
}

#[cfg(feature = "box-allocator")]
pub struct Parser {
    // ... old Box-based implementation
}
```

### Phase 2.3: Update Call Sites (Week 2)

Update example grammars and tests:

```rust
// Before
let tree = parser.parse(input)?;
let root = tree.root();

// After (same API, just lifetime parameter)
let tree = parser.parse(input)?;
let root = tree.root_node();
```

### Phase 2.4: Validation (Week 2)

- [ ] All existing tests pass
- [ ] No performance regression
- [ ] Miri/ASan clean
- [ ] Benchmarks show expected improvement

## Implementation Checklist

### Core Integration

- [ ] Add `arena: TreeArena` field to Parser
- [ ] Update Parser::new() to initialize arena
- [ ] Add lifetime parameter to Tree: `Tree<'arena>`
- [ ] Update parse() signature: `pub fn parse<'a>(&'a mut self, ...) -> Tree<'a>`
- [ ] Modify tree construction to use arena.alloc()
- [ ] Update Node type to reference arena-allocated data

### API Updates

- [ ] TreeNodeData struct for arena-allocated node data
- [ ] Update TreeCursor to work with handles
- [ ] Update TreeWalker for arena-based traversal
- [ ] Add arena metrics methods to Parser

### Testing

- [ ] Unit tests for all 7 specifications
- [ ] Integration tests with example grammars
- [ ] Benchmark arena-integrated parser vs v0.8.0-corrected
- [ ] Miri verification
- [ ] ASan verification

### Documentation

- [ ] Update Parser API docs with lifetime examples
- [ ] Migration guide for existing code
- [ ] Performance comparison vs Box-based
- [ ] Troubleshooting guide for common lifetime errors

## Success Criteria

### Functional

- ✅ All existing parser tests pass with arena integration
- ✅ New arena integration tests pass (Specs 1-7)
- ✅ Example grammars parse correctly
- ✅ No memory leaks (Valgrind)
- ✅ No undefined behavior (Miri)

### Performance

Measured against v0.8.0-corrected baseline:

| Metric | Target | Measurement |
|--------|--------|-------------|
| Parse Time | ≥20% speedup | Criterion benchmark |
| Allocation Count | ≥50% reduction | Valgrind/massif |
| Memory Reuse | Reset < 1µs | Microbenchmark |

### API Quality

- ✅ Lifetime errors are clear and actionable
- ✅ Common patterns well-documented
- ✅ Migration path is straightforward
- ✅ No unnecessary API breakage

## Risk Mitigation

### Risk: Lifetime Complexity

**Impact**: Users struggle with lifetime parameters
**Mitigation**:
- Comprehensive examples in docs
- Helper functions for common patterns
- Clear compiler error messages

### Risk: Performance Regression

**Impact**: Real-world parsing slower despite arena improvements
**Mitigation**:
- Benchmark each step
- Profile with perf/flamegraph
- A/B test with feature flag

### Risk: Breaking Changes

**Impact**: Existing code breaks
**Mitigation**:
- Feature flag during transition
- Migration guide
- Deprecation warnings before removal

## References

- [ADR-0001: Arena Allocator](../adr/0001-arena-allocator-for-parse-trees.md)
- [Arena Allocator Spec](ARENA_ALLOCATOR_SPEC.md)
- [Arena Allocator Guide](../guides/ARENA_ALLOCATOR_GUIDE.md)
- [v0.8.0 Performance Contract](../contracts/V0.8.0_PERFORMANCE_CONTRACT.md)
