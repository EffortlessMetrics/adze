# Node API Contract Specification

**Version**: 1.0
**Status**: Draft
**Phase**: 3.3 Component 1
**Related**: PHASE_3.3_INTEGRATION_TESTING.md, TREE_API_CONTRACT.md

---

## Purpose

Define the contract for the `Node<'tree>` API that provides read-only access to syntax tree nodes.
This API must support tree traversal, child access, and metadata queries required for:

1. User validation tests (arithmetic example)
2. Typed AST extraction (Extract trait)
3. Editor tooling (LSP, syntax highlighting)
4. Tree-sitter API compatibility

## Problem Statement

**Current State**:
- `Tree` contains `TreeNode` with full data (symbol, range, children)
- `Node<'tree>` API is stubbed out with placeholders
- `Node::new()` discards `TreeNode` reference and stores `&()`
- All `Node` methods return dummy values (child_count() → 0, child() → None)

**Impact**:
- 4/10 arithmetic tests failing with "Root should have children"
- Users cannot navigate trees
- No way to validate parse tree structure

**Root Cause**:
```rust
// Current broken implementation
pub(crate) fn new(_node: &'tree impl std::any::Any, _language: Option<&'tree Language>) -> Self {
    Self {
        _data: &(),  // ← Throws away the TreeNode reference!
        _language,
    }
}
```

---

## API Contract

### Node Structure

```rust
pub struct Node<'tree> {
    /// Reference to internal tree node data
    data: &'tree TreeNode,
    /// Language for symbol metadata
    language: Option<&'tree Language>,
}
```

**Invariants**:
- `data` lifetime tied to parent `Tree`
- Node is read-only (Copy + no mutation)
- Language is optional (GLR mode may not have Language)

### Core Methods

#### 1. Node Metadata

```rust
/// Get the node's symbol ID
pub fn kind_id(&self) -> u16;
```

**Contract**:
- Returns `self.data.symbol as u16`
- Always succeeds
- Maps to grammar symbol IDs

```rust
/// Get the node's symbol name
pub fn kind(&self) -> &str;
```

**Contract**:
- Returns symbol name from `language.symbol_metadata[symbol_id]`
- Falls back to "unknown" if language is None
- Always returns valid UTF-8

#### 2. Position Information

```rust
/// Get the node's byte range in source
pub fn byte_range(&self) -> std::ops::Range<usize>;
```

**Contract**:
- Returns `self.data.start_byte..self.data.end_byte`
- Range is always valid: `start <= end`
- Measured in bytes, not characters

```rust
/// Get start/end byte shortcuts
pub fn start_byte(&self) -> usize;
pub fn end_byte(&self) -> usize;
```

**Contract**:
- Convenience methods for `byte_range().start` / `.end`
- Same guarantees as `byte_range()`

#### 3. Child Access

```rust
/// Get the number of children
pub fn child_count(&self) -> usize;
```

**Contract**:
- Returns `self.data.children.len()`
- Includes both named and anonymous children
- Returns 0 for terminal nodes

```rust
/// Get a child by index
pub fn child(&self, index: usize) -> Option<Node<'tree>>;
```

**Contract**:
- Returns `Some(Node { data: &children[index], language })` if `index < child_count()`
- Returns `None` if index out of bounds
- Child inherits parent's language

```rust
/// Get the number of named children
pub fn named_child_count(&self) -> usize;
```

**Contract**:
- Returns count of children where `symbol_metadata[child.symbol].visible == true`
- Falls back to `child_count()` if language is None
- Always `<= child_count()`

```rust
/// Get a named child by index
pub fn named_child(&self, index: usize) -> Option<Node<'tree>>;
```

**Contract**:
- Skip unnamed children, return nth named child
- Returns None if index exceeds named child count
- Uses symbol_metadata to determine visibility

#### 4. Node Classification

```rust
/// Check if this node is named (visible in the tree)
pub fn is_named(&self) -> bool;
```

**Contract**:
- Returns `symbol_metadata[self.kind_id()].visible`
- Falls back to `true` if language is None
- Determines whether node appears in "named" APIs

```rust
/// Check if this node is missing (error recovery)
pub fn is_missing(&self) -> bool;
```

**Contract**:
- Returns false for now (error recovery not implemented)
- Future: check if node was inserted by error recovery

```rust
/// Check if this node is an error node
pub fn is_error(&self) -> bool;
```

**Contract**:
- Returns false for now (error nodes not implemented)
- Future: check if node represents a parse error

#### 5. Text Extraction

```rust
/// Get the UTF-8 text of this node
pub fn utf8_text<'a>(&self, source: &'a [u8]) -> Result<&'a str, std::str::Utf8Error>;
```

**Contract**:
- Extracts `source[self.byte_range()]`
- Validates UTF-8 and returns error if invalid
- Lifetime `'a` independent of `'tree`

---

## Phase 1 Implementation (MVP)

**Scope**: Minimum required for arithmetic tests

Implement:
- ✅ `kind_id()` → `data.symbol as u16`
- ✅ `byte_range()` → `data.start_byte..data.end_byte`
- ✅ `child_count()` → `data.children.len()`
- ✅ `child(index)` → `data.children.get(index)`

Defer:
- ❌ `kind()` → return "unknown" for now (no symbol names)
- ❌ `named_child_count()` → return `child_count()` (no visibility filtering)
- ❌ `is_named()` → return `true` (assume all named)

**Rationale**: Arithmetic tests only need child access, not symbol names or visibility

## Phase 2 Implementation (Full API)

**Scope**: Complete Tree-sitter compatibility

Add:
- Symbol name resolution via Language
- Named child filtering via symbol_metadata
- Field-based child access
- Tree navigation (parent, siblings)

---

## Test Coverage

### Unit Tests

```rust
#[test]
fn test_node_child_count() {
    let tree = Tree::new(TreeNode::new_with_children(
        4, 0, 5,
        vec![
            TreeNode::leaf(1, 0, 1),
            TreeNode::leaf(2, 1, 2),
            TreeNode::leaf(1, 2, 3),
        ],
    ));
    let root = tree.root_node();
    assert_eq!(root.child_count(), 3);
}

#[test]
fn test_node_child_access() {
    // Same tree as above
    let root = tree.root_node();
    let child0 = root.child(0).expect("First child exists");
    assert_eq!(child0.kind_id(), 1);
    assert_eq!(child0.byte_range(), 0..1);
}

#[test]
fn test_node_out_of_bounds() {
    let tree = Tree::new(TreeNode::leaf(1, 0, 2));
    let root = tree.root_node();
    assert_eq!(root.child(0), None); // No children
}
```

### Integration Tests

Use existing arithmetic tests:
- `test_basic_subtraction`: Verify root has 3 children
- `test_precedence`: Verify tree structure matches precedence
- All other tests should pass with working child access

---

## Success Criteria

**Phase 1 Complete When**:
- ✅ All 6 arithmetic parsing tests pass (currently failing 4 pass)
- ✅ `child_count()` returns correct values
- ✅ `child(index)` returns correct child nodes
- ✅ Unit tests cover basic child access

**Phase 2 Complete When**:
- ✅ Symbol names available via `kind()`
- ✅ Named child filtering working
- ✅ Field access working
- ✅ Tree navigation (parent/sibling) working

---

## Non-Goals (Out of Scope)

- Tree mutation (nodes are read-only)
- Error node creation (no error recovery yet)
- Position tracking beyond bytes (rows/columns deferred)
- Tree cursors (advanced iteration pattern)

---

## Dependencies

**Requires**:
- `Tree` and `TreeNode` structures (already implemented)
- `Language` with `symbol_metadata` (available in GLR mode)

**Blocks**:
- Arithmetic example completion
- Typed AST extraction
- Editor tooling integration

---

## Open Questions

1. **Q**: How to handle missing Language in pure-Rust mode?
   **A**: Fall back to generic behavior (all children named, kind_id only)

2. **Q**: Should `Node` implement Debug with tree structure?
   **A**: Yes, but keep it simple: `Node { kind: X, range: Y }`

3. **Q**: Performance of `named_child()` with filtering?
   **A**: Acceptable for now, optimize later if needed

---

**Status**: Ready for implementation
**Next**: Implement Phase 1 MVP in `runtime2/src/node.rs`
**Validation**: Run `cargo test -p rust-sitter-runtime --example arithmetic`
