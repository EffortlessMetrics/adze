# Node<'arena> Specification

**Version**: 1.0
**Status**: Draft
**Related**: PARSER_ARENA_INTEGRATION_SPEC.md, TREE_NODE_DATA_SPEC.md, ADR-0001

## Overview

This specification defines the `Node<'arena>` type, which provides a safe, ergonomic API for accessing arena-allocated parse tree nodes. Node is a lightweight handle wrapper that borrows the arena and provides access to TreeNodeData without exposing internal implementation details.

## Goals

1. **Safe arena access**: Lifetime-bound references prevent use-after-free
2. **Ergonomic API**: Natural tree traversal without manual handle management
3. **Zero-cost abstraction**: Node is compile-time only, no runtime overhead
4. **Tree-sitter compatibility**: API matches tree-sitter Node interface
5. **Iterator support**: Efficient child iteration without allocations

## Non-Goals

- ❌ Mutable node access (trees are immutable after parsing)
- ❌ Parent pointers (use TreeCursor for bidirectional traversal)
- ❌ Node ownership (Node always borrows arena)
- ❌ Serialization (handled at Tree level)

## Design

### Type Definition

```rust
/// A node in the parse tree
///
/// Node is a lightweight wrapper around a NodeHandle that provides
/// safe access to arena-allocated TreeNodeData. The lifetime parameter
/// ties the node to the arena, preventing use-after-free.
///
/// # Lifetime
///
/// The `'arena` lifetime ensures nodes cannot outlive the arena
/// they reference. This is enforced at compile time with zero
/// runtime overhead.
///
/// # Size
///
/// Node is 16 bytes on 64-bit systems:
/// - NodeHandle: 8 bytes (u32 + u32)
/// - &'arena TreeArena: 8 bytes (pointer)
///
/// # Copy Semantics
///
/// Node implements Copy because it only contains:
/// - A handle (Copy)
/// - A reference (Copy)
///
/// This allows efficient passing and duplication without clone().
#[derive(Copy, Clone, Debug)]
pub struct Node<'arena> {
    handle: NodeHandle,
    arena: &'arena TreeArena,
}
```

### Core API

```rust
impl<'arena> Node<'arena> {
    /// Create a new node from handle and arena reference
    ///
    /// # Safety
    ///
    /// Internal use only. Handle must be valid for the arena.
    pub(crate) fn new(handle: NodeHandle, arena: &'arena TreeArena) -> Self;

    /// Get the node's symbol/kind ID
    pub fn symbol(&self) -> u16;

    /// Get the node's byte range in the source
    ///
    /// Returns (start_byte, end_byte) tuple.
    pub fn byte_range(&self) -> (u32, u32);

    /// Get start byte position
    pub fn start_byte(&self) -> u32;

    /// Get end byte position
    pub fn end_byte(&self) -> u32;

    /// Check if this is a named node
    ///
    /// Named nodes appear in the grammar explicitly.
    /// Anonymous nodes are punctuation/keywords.
    pub fn is_named(&self) -> bool;

    /// Check if this node is missing (error recovery)
    pub fn is_missing(&self) -> bool;

    /// Check if this node is extra (trivia)
    pub fn is_extra(&self) -> bool;

    /// Check if this node contains errors
    pub fn has_error(&self) -> bool;

    /// Get child count
    pub fn child_count(&self) -> usize;

    /// Get named child count
    pub fn named_child_count(&self) -> usize;

    /// Get child by index
    ///
    /// Returns None if index >= child_count().
    pub fn child(&self, index: usize) -> Option<Node<'arena>>;

    /// Get named child by index
    ///
    /// Returns None if index >= named_child_count().
    pub fn named_child(&self, index: usize) -> Option<Node<'arena>>;

    /// Get field ID if this node has one
    pub fn field_id(&self) -> Option<u16>;

    /// Iterate over all children
    pub fn children(&self) -> NodeChildren<'arena>;

    /// Iterate over named children only
    pub fn named_children(&self) -> NamedChildren<'arena>;

    /// Get direct access to underlying TreeNodeData
    ///
    /// For advanced use cases that need raw data access.
    pub fn data(&self) -> &TreeNodeData;
}
```

### Iterator Types

```rust
/// Iterator over all children of a node
pub struct NodeChildren<'arena> {
    handles: &'arena [NodeHandle],
    arena: &'arena TreeArena,
    index: usize,
}

impl<'arena> Iterator for NodeChildren<'arena> {
    type Item = Node<'arena>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.handles.len() {
            let handle = self.handles[self.index];
            self.index += 1;
            Some(Node::new(handle, self.arena))
        } else {
            None
        }
    }
}

/// Iterator over named children only
pub struct NamedChildren<'arena> {
    inner: NodeChildren<'arena>,
}

impl<'arena> Iterator for NamedChildren<'arena> {
    type Item = Node<'arena>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find(|node| node.is_named())
    }
}
```

## Behavioral Specifications

### Spec 1: Node Creation and Access

**Given**: Arena with allocated TreeNodeData
**When**: User creates Node from handle and arena
**Then**:
- Node provides access to underlying data
- Symbol, byte range, flags are accessible
- Node is Copy

**Test**:
```rust
#[test]
fn spec_1_node_creation() {
    let mut arena = TreeArena::new();
    let data = TreeNodeData::leaf(42, 10, 20);
    let handle = arena.alloc(data);

    let node = Node::new(handle, &arena);

    assert_eq!(node.symbol(), 42);
    assert_eq!(node.byte_range(), (10, 20));
    assert_eq!(node.start_byte(), 10);
    assert_eq!(node.end_byte(), 20);
}
```

### Spec 2: Node is Copy

**Given**: Node instance
**When**: Node is assigned to another variable
**Then**:
- No clone() required
- Both variables access same underlying data
- Original node still usable

**Test**:
```rust
#[test]
fn spec_2_node_is_copy() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNodeData::leaf(1, 0, 10));

    let node1 = Node::new(handle, &arena);
    let node2 = node1; // Copy, not move

    // Both usable
    assert_eq!(node1.symbol(), 1);
    assert_eq!(node2.symbol(), 1);
}
```

### Spec 3: Child Access

**Given**: Node with children
**When**: User accesses children by index or iterator
**Then**:
- child(i) returns Some(Node) for valid indices
- child(i) returns None for out-of-bounds
- children() iterator yields all children
- Child count matches data

**Test**:
```rust
#[test]
fn spec_3_child_access() {
    let mut arena = TreeArena::new();

    // Create children
    let child1 = arena.alloc(TreeNodeData::leaf(1, 0, 5));
    let child2 = arena.alloc(TreeNodeData::leaf(2, 5, 10));

    // Create parent
    let parent_data = TreeNodeData::branch(10, 0, 10, vec![child1, child2]);
    let parent_handle = arena.alloc(parent_data);

    let parent = Node::new(parent_handle, &arena);

    // Test child access
    assert_eq!(parent.child_count(), 2);
    assert!(parent.child(0).is_some());
    assert!(parent.child(1).is_some());
    assert!(parent.child(2).is_none());

    // Test iterator
    let children: Vec<_> = parent.children().collect();
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].symbol(), 1);
    assert_eq!(children[1].symbol(), 2);
}
```

### Spec 4: Named Children

**Given**: Node with mix of named and anonymous children
**When**: User requests named children
**Then**:
- named_child_count() returns count of named children only
- named_child(i) returns ith named child
- named_children() iterator yields only named children

**Test**:
```rust
#[test]
fn spec_4_named_children() {
    let mut arena = TreeArena::new();

    // Named child (is_named flag set)
    let mut named = TreeNodeData::leaf(1, 0, 5);
    named.set_named(true);
    let named_handle = arena.alloc(named);

    // Anonymous child
    let anon = TreeNodeData::leaf(2, 5, 6);
    let anon_handle = arena.alloc(anon);

    // Parent with both
    let parent_data = TreeNodeData::branch(
        10, 0, 6,
        vec![named_handle, anon_handle]
    );
    let parent_handle = arena.alloc(parent_data);

    let parent = Node::new(parent_handle, &arena);

    assert_eq!(parent.child_count(), 2);
    assert_eq!(parent.named_child_count(), 1);

    let named_child = parent.named_child(0).unwrap();
    assert_eq!(named_child.symbol(), 1);
    assert!(named_child.is_named());

    // Second named child doesn't exist
    assert!(parent.named_child(1).is_none());
}
```

### Spec 5: Node Flags

**Given**: Node with various flags set
**When**: User checks flag states
**Then**:
- is_named() reflects flag state
- is_missing() reflects flag state
- is_extra() reflects flag state
- has_error() reflects flag state

**Test**:
```rust
#[test]
fn spec_5_node_flags() {
    let mut arena = TreeArena::new();

    let mut data = TreeNodeData::leaf(1, 0, 10);
    data.set_named(true);
    data.set_missing(true);
    data.set_extra(false);
    data.set_has_error(true);

    let handle = arena.alloc(data);
    let node = Node::new(handle, &arena);

    assert!(node.is_named());
    assert!(node.is_missing());
    assert!(!node.is_extra());
    assert!(node.has_error());
}
```

### Spec 6: Data Access

**Given**: Node
**When**: User calls .data()
**Then**:
- Returns reference to underlying TreeNodeData
- Data matches what was allocated
- Reference is read-only

**Test**:
```rust
#[test]
fn spec_6_data_access() {
    let mut arena = TreeArena::new();

    let data = TreeNodeData::leaf(42, 100, 200);
    let handle = arena.alloc(data.clone());

    let node = Node::new(handle, &arena);
    let retrieved_data = node.data();

    assert_eq!(retrieved_data.symbol(), 42);
    assert_eq!(retrieved_data.byte_range(), (100, 200));
}
```

### Spec 7: Lifetime Safety (Compile-Time)

**Given**: Node created from arena reference
**When**: Arena reference is dropped
**Then**: Compilation error (node cannot outlive arena)

**Test**:
```rust
// This should NOT compile
/*
#[test]
fn spec_7_lifetime_safety() {
    let node = {
        let mut arena = TreeArena::new();
        let handle = arena.alloc(TreeNodeData::leaf(1, 0, 10));
        Node::new(handle, &arena)
    }; // arena dropped here

    // Compilation error: arena doesn't live long enough
    let _ = node.symbol();
}
*/
```

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| Node creation | O(1) | Just copies handle and reference |
| symbol() | O(1) | Direct field access |
| byte_range() | O(1) | Direct field access |
| child(i) | O(1) | Array index + Node creation |
| children() | O(1) to create | Iterator is lazy |
| children().collect() | O(n) | Visits all n children |
| named_children() | O(n) | Filters all children |
| is_named() | O(1) | Bit check |

## Memory Layout

Node size: **16 bytes** (64-bit systems)

```
┌─────────────────────┬─────────────────────┐
│   NodeHandle (8B)   │  &TreeArena (8B)    │
├──────────┬──────────┼─────────────────────┤
│ chunk_idx│ node_idx │   arena pointer     │
│  (4B)    │  (4B)    │      (8B)           │
└──────────┴──────────┴─────────────────────┘
```

## Integration Points

### Tree<'arena>

```rust
impl<'arena> Tree<'arena> {
    pub fn root_node(&self) -> Node<'arena> {
        Node::new(self.root, self.arena)
    }

    pub fn get_node(&self, handle: NodeHandle) -> Node<'arena> {
        Node::new(handle, self.arena)
    }
}
```

### TreeCursor<'arena>

```rust
impl<'arena> TreeCursor<'arena> {
    pub fn node(&self) -> Node<'arena> {
        Node::new(self.current, self.arena)
    }
}
```

## Migration from Old Node

Old API (Box-based):
```rust
let root = tree.root();
let child = root.child(0);
let symbol = child.symbol();
```

New API (Arena-based):
```rust
let root = tree.root_node();
let child = root.child(0).unwrap(); // Now returns Option
let symbol = child.symbol();
```

**Breaking Changes**:
- `child(i)` returns `Option<Node>` instead of panicking
- Node is Copy, not Clone
- No mutable access to nodes

## Implementation Checklist

- [ ] Define Node<'arena> struct
- [ ] Implement new(handle, arena) constructor
- [ ] Implement symbol() and byte_range() accessors
- [ ] Implement flag methods (is_named, is_missing, etc.)
- [ ] Implement child access (child, named_child)
- [ ] Implement NodeChildren iterator
- [ ] Implement NamedChildren iterator
- [ ] Implement children() and named_children()
- [ ] Implement data() accessor
- [ ] Write comprehensive test suite (7 specs)
- [ ] Add Debug formatting
- [ ] Document all public APIs
- [ ] Verify size with mem::size_of
