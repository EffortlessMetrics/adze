//! Typed arena allocator utilities for parse-tree style node graphs.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use std::mem;

/// Default initial chunk size (1024 nodes ~= 64KB for typical node size)
const DEFAULT_CHUNK_SIZE: usize = 1024;

/// Maximum chunk size to avoid large contiguous allocations
const MAX_CHUNK_SIZE: usize = 65536;

/// Arena allocator for parse tree nodes.
#[derive(Debug)]
pub struct TreeArena {
    chunks: Vec<Chunk>,
    current_chunk_idx: usize,
    current_offset: usize,
}

#[derive(Debug)]
struct Chunk {
    data: Vec<TreeNode>,
    capacity: usize,
}

impl Chunk {
    fn new(capacity: usize) -> Self {
        Chunk {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    fn is_full(&self) -> bool {
        self.data.len() >= self.capacity
    }

    fn alloc(&mut self, node: TreeNode) -> usize {
        let idx = self.data.len();
        self.data.push(node);
        idx
    }

    fn get(&self, idx: usize) -> &TreeNode {
        &self.data[idx]
    }

    fn get_mut(&mut self, idx: usize) -> &mut TreeNode {
        &mut self.data[idx]
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

/// Opaque handle to a node in the arena.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeHandle {
    chunk_idx: u32,
    node_idx: u32,
}

impl NodeHandle {
    /// Create a new node handle.
    #[must_use]
    pub fn new(chunk_idx: u32, node_idx: u32) -> Self {
        NodeHandle {
            chunk_idx,
            node_idx,
        }
    }

    fn chunk_idx(&self) -> usize {
        self.chunk_idx as usize
    }

    fn node_idx(&self) -> usize {
        self.node_idx as usize
    }
}

impl TreeArena {
    /// Create a new arena with default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHUNK_SIZE)
    }

    /// Create arena with specific initial capacity.
    pub fn with_capacity(initial_capacity: usize) -> Self {
        assert!(initial_capacity > 0, "Capacity must be > 0");

        let chunks = vec![Chunk::new(initial_capacity)];

        TreeArena {
            chunks,
            current_chunk_idx: 0,
            current_offset: 0,
        }
    }

    /// Allocate a new tree node.
    pub fn alloc(&mut self, node: TreeNode) -> NodeHandle {
        if self.chunks[self.current_chunk_idx].is_full() {
            self.allocate_new_chunk();
        }

        let node_idx = self.chunks[self.current_chunk_idx].alloc(node);
        self.current_offset = node_idx + 1;

        NodeHandle {
            chunk_idx: self.current_chunk_idx as u32,
            node_idx: node_idx as u32,
        }
    }

    /// Get immutable reference to node.
    pub fn get(&self, handle: NodeHandle) -> TreeNodeRef<'_> {
        debug_assert!(self.is_valid_handle(handle), "Invalid node handle");

        let chunk = &self.chunks[handle.chunk_idx()];
        let node = chunk.get(handle.node_idx());

        TreeNodeRef { node }
    }

    /// Get mutable reference to node.
    pub fn get_mut(&mut self, handle: NodeHandle) -> TreeNodeRefMut<'_> {
        debug_assert!(self.is_valid_handle(handle), "Invalid node handle");

        let chunk = &mut self.chunks[handle.chunk_idx()];
        let node = chunk.get_mut(handle.node_idx());

        TreeNodeRefMut { node }
    }

    /// Reset arena for reuse.
    pub fn reset(&mut self) {
        for chunk in &mut self.chunks {
            chunk.clear();
        }
        self.current_chunk_idx = 0;
        self.current_offset = 0;
    }

    /// Clear arena and free excess chunks.
    pub fn clear(&mut self) {
        self.chunks.truncate(1);
        self.chunks[0].clear();
        self.current_chunk_idx = 0;
        self.current_offset = 0;
    }

    /// Get total number of allocated nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        let mut total = 0;
        for (i, chunk) in self.chunks.iter().enumerate() {
            if i < self.current_chunk_idx {
                total += chunk.capacity;
            } else if i == self.current_chunk_idx {
                total += self.current_offset;
            }
        }
        total
    }

    /// Check if arena is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.current_chunk_idx == 0 && self.current_offset == 0
    }

    /// Get total capacity across all chunks.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.chunks.iter().map(|c| c.capacity).sum()
    }

    /// Get number of chunks.
    #[must_use]
    pub fn num_chunks(&self) -> usize {
        self.chunks.len()
    }

    /// Get approximate memory usage in bytes.
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        let node_size = mem::size_of::<TreeNode>();
        self.capacity() * node_size
    }

    fn allocate_new_chunk(&mut self) {
        let current_capacity = self.chunks[self.current_chunk_idx].capacity;
        let new_capacity = (current_capacity * 2).min(MAX_CHUNK_SIZE);

        self.chunks.push(Chunk::new(new_capacity));
        self.current_chunk_idx += 1;
        self.current_offset = 0;
    }

    fn is_valid_handle(&self, handle: NodeHandle) -> bool {
        let chunk_idx = handle.chunk_idx();
        let node_idx = handle.node_idx();

        if chunk_idx >= self.chunks.len() {
            return false;
        }

        node_idx < self.chunks[chunk_idx].data.len()
    }

    /// Get current metrics snapshot.
    #[must_use]
    pub fn metrics(&self) -> ArenaMetrics {
        ArenaMetrics::from_arena(self)
    }
}

impl Default for TreeArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable reference to a tree node.
pub struct TreeNodeRef<'arena> {
    node: &'arena TreeNode,
}

impl<'arena> TreeNodeRef<'arena> {
    /// Get the underlying node reference.
    pub fn get_ref(&self) -> &'arena TreeNode {
        self.node
    }

    /// Get the underlying node reference (backwards-compatible alias).
    #[allow(clippy::wrong_self_convention, clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'arena TreeNode {
        self.get_ref()
    }

    /// Get node symbol value.
    #[must_use]
    pub fn value(&self) -> i32 {
        self.node.symbol()
    }

    /// Get node symbol.
    #[must_use]
    pub fn symbol(&self) -> i32 {
        self.node.symbol()
    }

    /// Check if this is a branch node.
    #[must_use]
    pub fn is_branch(&self) -> bool {
        matches!(self.node.kind, TreeNodeKind::Branch { .. })
    }

    /// Check if this is a leaf node.
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        matches!(self.node.kind, TreeNodeKind::Leaf { .. })
    }

    /// Get child handles for this node.
    #[must_use]
    pub fn children(&self) -> &[NodeHandle] {
        self.node.children()
    }
}

impl<'arena> std::ops::Deref for TreeNodeRef<'arena> {
    type Target = TreeNode;

    fn deref(&self) -> &Self::Target {
        self.node
    }
}

/// Mutable reference to a tree node.
pub struct TreeNodeRefMut<'arena> {
    node: &'arena mut TreeNode,
}

impl<'arena> TreeNodeRefMut<'arena> {
    /// Set the value of a leaf node.
    pub fn set_value(&mut self, value: i32) {
        if let TreeNodeKind::Leaf { symbol: ref mut v } = self.node.kind {
            *v = value;
        }
    }
}

impl<'arena> std::ops::Deref for TreeNodeRefMut<'arena> {
    type Target = TreeNode;

    fn deref(&self) -> &Self::Target {
        self.node
    }
}

impl<'arena> std::ops::DerefMut for TreeNodeRefMut<'arena> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.node
    }
}

/// A node in the parse tree.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeNode {
    kind: TreeNodeKind,
}

#[derive(Clone, Debug, PartialEq)]
enum TreeNodeKind {
    Leaf {
        symbol: i32,
    },
    Branch {
        symbol: i32,
        children: Vec<NodeHandle>,
    },
}

impl TreeNode {
    /// Create a leaf node with a value.
    #[must_use]
    pub fn leaf(value: i32) -> Self {
        TreeNode {
            kind: TreeNodeKind::Leaf { symbol: value },
        }
    }

    /// Create a branch node with children.
    #[must_use]
    pub fn branch(children: Vec<NodeHandle>) -> Self {
        TreeNode {
            kind: TreeNodeKind::Branch {
                symbol: 0,
                children,
            },
        }
    }

    /// Create a branch node with symbol and children.
    #[must_use]
    pub fn branch_with_symbol(symbol: i32, children: Vec<NodeHandle>) -> Self {
        TreeNode {
            kind: TreeNodeKind::Branch { symbol, children },
        }
    }

    /// Get symbol value.
    #[must_use]
    pub fn value(&self) -> i32 {
        self.symbol()
    }

    /// Get symbol id.
    #[must_use]
    pub fn symbol(&self) -> i32 {
        match self.kind {
            TreeNodeKind::Leaf { symbol } | TreeNodeKind::Branch { symbol, .. } => symbol,
        }
    }

    /// Check if this is a leaf.
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        matches!(self.kind, TreeNodeKind::Leaf { .. })
    }

    /// Check if this is a branch.
    #[must_use]
    pub fn is_branch(&self) -> bool {
        matches!(self.kind, TreeNodeKind::Branch { .. })
    }

    /// Get child handles for this node.
    #[must_use]
    pub fn children(&self) -> &[NodeHandle] {
        match &self.kind {
            TreeNodeKind::Leaf { .. } => &[],
            TreeNodeKind::Branch { children, .. } => children,
        }
    }
}

/// Arena metrics snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArenaMetrics {
    len: usize,
    capacity: usize,
    num_chunks: usize,
    memory_usage: usize,
}

impl ArenaMetrics {
    fn from_arena(arena: &TreeArena) -> Self {
        Self {
            len: arena.len(),
            capacity: arena.capacity(),
            num_chunks: arena.num_chunks(),
            memory_usage: arena.memory_usage(),
        }
    }

    /// Get number of allocated nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if arena is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get total capacity across all chunks.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get number of chunks.
    #[must_use]
    pub fn num_chunks(&self) -> usize {
        self.num_chunks
    }

    /// Get approximate memory usage in bytes.
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        self.memory_usage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_arena() {
        let arena = TreeArena::new();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        assert_eq!(arena.num_chunks(), 1);
    }

    #[test]
    fn test_basic_allocation() {
        let mut arena = TreeArena::new();
        let handle = arena.alloc(TreeNode::leaf(42));

        assert_eq!(arena.len(), 1);
        assert_eq!(arena.get(handle).value(), 42);
    }
}
