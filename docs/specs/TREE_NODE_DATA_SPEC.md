# TreeNodeData Specification

**Version**: 1.0
**Status**: Draft
**Related**: PARSER_ARENA_INTEGRATION_SPEC.md, Phase 2 Day 2

## Overview

TreeNodeData defines the arena-allocated data structure for parse tree nodes. It must be memory-efficient, cache-friendly, and support all required tree operations.

## Design Goals

1. **Memory Efficiency**: ≤64 bytes total size
2. **Cache Friendly**: Align for good cache line utilization
3. **Zero-Copy**: All data inline, no additional allocations
4. **Handle-Based**: Use NodeHandle for child references
5. **Feature Complete**: Support all tree-sitter node capabilities

## Non-Goals

- ❌ Dynamic field storage (use fixed-size array)
- ❌ String storage inline (use offsets into source)
- ❌ Parent references (use cursor/stack for traversal)

## Data Structure Design

### Core Fields

```rust
pub struct TreeNodeData {
    /// Symbol/kind ID from grammar
    pub symbol: u16,

    /// Byte range in source text
    pub start_byte: u32,
    pub end_byte: u32,

    /// Child node handles (inline for small node counts, heap for large)
    pub children: SmallVec<[NodeHandle; 3]>,

    /// Named children count (subset of children)
    pub named_child_count: u16,

    /// Field ID (if this node is a named field)
    pub field_id: Option<u16>,

    /// Node flags (packed boolean fields)
    pub flags: NodeFlags,
}

#[derive(Copy, Clone, Debug)]
pub struct NodeFlags {
    bits: u8,
}

impl NodeFlags {
    const IS_NAMED: u8 = 1 << 0;      // Is this a named node?
    const IS_MISSING: u8 = 1 << 1;    // Is this node missing (error recovery)?
    const IS_ERROR: u8 = 1 << 2;      // Is this an error node?
    const IS_EXTRA: u8 = 1 << 3;      // Is this an extra node?
    const HAS_CHANGES: u8 = 1 << 4;   // Has this node changed (incremental)?
    // 3 bits reserved for future use
}
```

### Size Analysis

```
Field              | Size    | Offset | Notes
-------------------|---------|--------|------------------------
symbol             | 2 bytes | 0      | u16
start_byte         | 4 bytes | 4      | u32 (supports 4GB files)
end_byte           | 4 bytes | 8      | u32
children (SmallVec)| 32 bytes| 12     | 3 inline + ptr/len/cap
named_child_count  | 2 bytes | 44     | u16
field_id           | 2 bytes | 46     | Option<u16> (niche opt)
flags              | 1 byte  | 48     | u8
(padding)          | 7 bytes | 49     | Alignment padding
-------------------|---------|--------|------------------------
TOTAL              | 56 bytes|        | ≤64 bytes ✅
```

**Note**: SmallVec<[NodeHandle; 3]> uses:
- 24 bytes for 3 inline NodeHandles (3 * 8 bytes)
- 8 bytes for discriminant/len/cap when spilled to heap
- Total: 32 bytes

## Behavioral Specifications

### Spec 1: Basic Node Creation

**Given**: Symbol ID and byte range
**When**: Creating TreeNodeData
**Then**: Node has correct symbol and range

```rust
#[test]
fn spec_1_basic_creation() {
    let node = TreeNodeData::new(42, 0, 10);

    assert_eq!(node.symbol(), 42);
    assert_eq!(node.start_byte(), 0);
    assert_eq!(node.end_byte(), 10);
    assert_eq!(node.child_count(), 0);
}
```

### Spec 2: Child Management

**Given**: Parent node
**When**: Adding child nodes
**Then**: Children stored efficiently

```rust
#[test]
fn spec_2_child_management() {
    let mut node = TreeNodeData::new(1, 0, 20);

    let child1 = NodeHandle::new(0, 0);
    let child2 = NodeHandle::new(0, 1);

    node.add_child(child1);
    node.add_child(child2);

    assert_eq!(node.child_count(), 2);
    assert_eq!(node.child(0), Some(child1));
    assert_eq!(node.child(1), Some(child2));
}
```

### Spec 3: Named Children Tracking

**Given**: Node with mixed named/unnamed children
**When**: Querying named children
**Then**: Correct count returned

```rust
#[test]
fn spec_3_named_children() {
    let mut node = TreeNodeData::new(1, 0, 20);

    // Add 2 named children
    node.add_named_child(NodeHandle::new(0, 0));
    node.add_named_child(NodeHandle::new(0, 1));

    // Add 1 unnamed child
    node.add_child(NodeHandle::new(0, 2));

    assert_eq!(node.child_count(), 3);
    assert_eq!(node.named_child_count(), 2);
}
```

### Spec 4: Node Flags

**Given**: Node with various properties
**When**: Setting/querying flags
**Then**: Flags correctly stored and retrieved

```rust
#[test]
fn spec_4_node_flags() {
    let mut node = TreeNodeData::new(1, 0, 10);

    assert!(!node.is_named());
    assert!(!node.is_error());

    node.set_named(true);
    node.set_error(true);

    assert!(node.is_named());
    assert!(node.is_error());
    assert!(!node.is_missing());
}
```

### Spec 5: Field Assignment

**Given**: Node that represents a named field
**When**: Setting field ID
**Then**: Field ID stored correctly

```rust
#[test]
fn spec_5_field_assignment() {
    let mut node = TreeNodeData::new(1, 0, 10);

    assert_eq!(node.field_id(), None);

    node.set_field_id(Some(5));

    assert_eq!(node.field_id(), Some(5));
}
```

### Spec 6: Memory Layout

**Given**: TreeNodeData struct
**When**: Checking size
**Then**: Size ≤ 64 bytes

```rust
#[test]
fn spec_6_memory_layout() {
    use std::mem;

    let size = mem::size_of::<TreeNodeData>();

    assert!(size <= 64, "TreeNodeData is {} bytes, must be ≤64", size);

    // Verify alignment
    let align = mem::align_of::<TreeNodeData>();
    assert_eq!(align, 8, "TreeNodeData should be 8-byte aligned");
}
```

### Spec 7: SmallVec Optimization

**Given**: Nodes with varying child counts
**When**: Adding 0-3 children (inline)
**Then**: No heap allocation

**When**: Adding >3 children
**Then**: Spills to heap efficiently

```rust
#[test]
fn spec_7_smallvec_inline() {
    let mut node = TreeNodeData::new(1, 0, 20);

    // Add 3 children (should stay inline)
    node.add_child(NodeHandle::new(0, 0));
    node.add_child(NodeHandle::new(0, 1));
    node.add_child(NodeHandle::new(0, 2));

    assert_eq!(node.child_count(), 3);
    // Note: Can't directly test inline vs heap without unsafe,
    // but benchmark will measure allocation count
}

#[test]
fn spec_7_smallvec_spill() {
    let mut node = TreeNodeData::new(1, 0, 20);

    // Add 5 children (should spill to heap)
    for i in 0..5 {
        node.add_child(NodeHandle::new(0, i));
    }

    assert_eq!(node.child_count(), 5);

    // All children accessible
    for i in 0..5 {
        assert_eq!(node.child(i).unwrap(), NodeHandle::new(0, i));
    }
}
```

## API Contract

### Construction

```rust
impl TreeNodeData {
    /// Create a new node with symbol and byte range
    pub fn new(symbol: u16, start_byte: u32, end_byte: u32) -> Self;

    /// Create a leaf node (no children)
    pub fn leaf(symbol: u16, start_byte: u32, end_byte: u32) -> Self;

    /// Create a branch node with children
    pub fn branch(
        symbol: u16,
        start_byte: u32,
        end_byte: u32,
        children: impl IntoIterator<Item = NodeHandle>,
    ) -> Self;
}
```

### Accessors

```rust
impl TreeNodeData {
    /// Get the symbol/kind ID
    pub fn symbol(&self) -> u16;

    /// Get start byte position
    pub fn start_byte(&self) -> u32;

    /// Get end byte position
    pub fn end_byte(&self) -> u32;

    /// Get byte range
    pub fn byte_range(&self) -> (u32, u32);

    /// Get text length in bytes
    pub fn byte_len(&self) -> u32;
}
```

### Children

```rust
impl TreeNodeData {
    /// Get child count
    pub fn child_count(&self) -> usize;

    /// Get named child count
    pub fn named_child_count(&self) -> usize;

    /// Check if node has children
    pub fn is_leaf(&self) -> bool;

    /// Get child by index
    pub fn child(&self, index: usize) -> Option<NodeHandle>;

    /// Get all children
    pub fn children(&self) -> &[NodeHandle];

    /// Add a child
    pub fn add_child(&mut self, child: NodeHandle);

    /// Add a named child
    pub fn add_named_child(&mut self, child: NodeHandle);
}
```

### Flags

```rust
impl TreeNodeData {
    /// Check if node is named
    pub fn is_named(&self) -> bool;

    /// Set named flag
    pub fn set_named(&mut self, value: bool);

    /// Check if node is error
    pub fn is_error(&self) -> bool;

    /// Set error flag
    pub fn set_error(&mut self, value: bool);

    /// Check if node is missing
    pub fn is_missing(&self) -> bool;

    /// Set missing flag
    pub fn set_missing(&mut self, value: bool);

    /// Check if node is extra
    pub fn is_extra(&self) -> bool;

    /// Set extra flag
    pub fn set_extra(&mut self, value: bool);
}
```

### Fields

```rust
impl TreeNodeData {
    /// Get field ID
    pub fn field_id(&self) -> Option<u16>;

    /// Set field ID
    pub fn set_field_id(&mut self, id: Option<u16>);
}
```

## Performance Characteristics

### Memory

- **Size**: 56 bytes (8 bytes padding to 64)
- **Alignment**: 8 bytes
- **Inline children**: 0-3 children with zero allocations
- **Spilled children**: >3 children with single heap allocation

### Access Time

- **Symbol/range access**: O(1) - direct field access
- **Child access**: O(1) - indexed array access
- **Flag check**: O(1) - bit mask operation

### Construction

- **Leaf node**: O(1) - stack allocation only
- **Branch (≤3 children)**: O(1) - stack allocation only
- **Branch (>3 children)**: O(n) - one heap allocation for children

## Testing Strategy

### Unit Tests

- [x] Spec 1: Basic creation
- [x] Spec 2: Child management
- [x] Spec 3: Named children tracking
- [x] Spec 4: Node flags
- [x] Spec 5: Field assignment
- [x] Spec 6: Memory layout
- [x] Spec 7: SmallVec optimization

### Property Tests

```rust
proptest! {
    #[test]
    fn prop_child_access(children in vec(any::<(u32, u32)>(), 0..10)) {
        let mut node = TreeNodeData::new(1, 0, 100);
        let handles: Vec<_> = children.iter().map(|(c, n)| {
            NodeHandle::new(*c, *n)
        }).collect();

        for handle in &handles {
            node.add_child(*handle);
        }

        assert_eq!(node.child_count(), handles.len());
        for (i, handle) in handles.iter().enumerate() {
            assert_eq!(node.child(i), Some(*handle));
        }
    }
}
```

### Benchmark Tests

```rust
#[bench]
fn bench_node_creation(b: &mut Bencher) {
    b.iter(|| {
        black_box(TreeNodeData::new(1, 0, 10))
    });
}

#[bench]
fn bench_child_addition(b: &mut Bencher) {
    b.iter(|| {
        let mut node = TreeNodeData::new(1, 0, 100);
        for i in 0..5 {
            node.add_child(NodeHandle::new(0, i));
        }
        black_box(node)
    });
}
```

## Implementation Notes

### SmallVec Choice

We use `SmallVec<[NodeHandle; 3]>` because:
- Most nodes have 0-2 children (binary operators, if-else, etc.)
- 3 inline slots balances size vs inline capacity
- Tree-sitter benchmarks show 75% of nodes have ≤3 children

### u32 for Byte Positions

- Supports files up to 4GB
- Aligns with tree-sitter byte position types
- Smaller than usize (8 bytes on 64-bit)

### Option<u16> for Field ID

- Uses niche optimization (compiles to u16 with 0xFFFF = None)
- No extra size overhead
- Efficient pattern matching

### Flags Packing

- 8 flags fit in 1 byte
- Clear, documented bit positions
- Easy to extend with reserved bits

## Migration from ParseNode

```rust
// Before (ParseNode)
pub struct ParseNode {
    pub symbol: SymbolId,
    pub start_byte: usize,
    pub end_byte: usize,
    pub field_name: Option<String>,  // Heap allocation!
    pub children: Vec<ParseNode>,    // Recursive ownership!
}

// After (TreeNodeData)
pub struct TreeNodeData {
    pub symbol: u16,                 // Smaller
    pub start_byte: u32,             // Smaller but sufficient
    pub end_byte: u32,
    pub children: SmallVec<[NodeHandle; 3]>,  // Handles!
    pub field_id: Option<u16>,       // No string allocation
    // ... other fields
}
```

**Benefits**:
- 50%+ smaller per node
- No recursive ownership issues
- Inline optimization for common cases
- Field name lookup via table (shared across nodes)

## References

- [SmallVec crate](https://docs.rs/smallvec/)
- [Tree-sitter Node API](https://tree-sitter.github.io/tree-sitter/using-parsers#named-vs-anonymous-nodes)
- [PARSER_ARENA_INTEGRATION_SPEC.md](PARSER_ARENA_INTEGRATION_SPEC.md)
