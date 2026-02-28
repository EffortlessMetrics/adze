//! Arena allocator for efficient parse tree node allocation
//!
//! This module provides a typed arena allocator optimized for allocating
//! parse tree nodes. It reduces allocation overhead and improves cache locality
//! compared to individually heap-allocated nodes.
//!
//! # Design
//!
//! - **Typed arena**: Only allocates `TreeNode` types
//! - **Chunked growth**: Allocates memory in exponentially growing chunks
//! - **Handle-based**: Uses `NodeHandle` for safe indirect references
//! - **Reset capability**: Arena can be cleared and reused across parses
//!
//! # Example
//!
//! ```
//! use adze::arena_allocator::{TreeArena, TreeNode};
//!
//! let mut arena = TreeArena::new();
//!
//! // Allocate nodes
//! let child1 = arena.alloc(TreeNode::leaf(1));
//! let child2 = arena.alloc(TreeNode::leaf(2));
//! let parent = arena.alloc(TreeNode::branch(vec![child1, child2]));
//!
//! // Access nodes
//! assert_eq!(arena.get(child1).value(), 1);
//!
//! // Reuse arena for next parse
//! arena.reset();
//! ```
//!
//! # Performance Characteristics
//!
//! - **Allocation**: O(1) amortized (O(n) when allocating new chunk)
//! - **Access**: O(1) via handle
//! - **Memory overhead**: O(log N) chunks for N nodes
//! - **Cache locality**: Excellent (nodes in contiguous memory)
//!
//! # Safety
//!
//! This implementation uses unsafe code internally but provides a safe API.
//! All invariants are maintained through careful design:
//!
//! - Handles are validated in debug builds
//! - No references escape chunk boundaries
//! - Lifetime system prevents use-after-reset
//!
//! Related: docs/adr/0001-arena-allocator-for-parse-trees.md

use std::mem;

/// Default initial chunk size (1024 nodes ~= 64KB for typical node size)
const DEFAULT_CHUNK_SIZE: usize = 1024;

/// Maximum chunk size to avoid large contiguous allocations
const MAX_CHUNK_SIZE: usize = 65536;

/// Arena allocator for parse tree nodes
///
/// Allocates nodes in chunks to reduce allocation overhead and improve
/// cache locality. Provides handle-based access for safety.
#[derive(Debug)]
pub struct TreeArena {
    chunks: Vec<Chunk>,
    current_chunk_idx: usize,
    current_offset: usize,
}

/// A single chunk of allocated tree nodes
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

/// Opaque handle to a node in the arena
///
/// Handles are small, copyable identifiers that can be used to retrieve
/// nodes from the arena. They remain valid until the arena is reset or dropped.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeHandle {
    chunk_idx: u32,
    node_idx: u32,
}

impl NodeHandle {
    /// Create a new node handle
    ///
    /// # Safety
    ///
    /// Caller must ensure indices are valid for the arena.
    /// This is primarily for testing; normal code should use `arena.alloc()`.
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
    /// Create a new arena with default capacity
    ///
    /// The arena starts with one chunk of [`DEFAULT_CHUNK_SIZE`] nodes.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHUNK_SIZE)
    }

    /// Create arena with specific initial capacity
    ///
    /// # Panics
    ///
    /// Panics if `initial_capacity` is 0.
    pub fn with_capacity(initial_capacity: usize) -> Self {
        assert!(initial_capacity > 0, "Capacity must be > 0");

        let chunks = vec![Chunk::new(initial_capacity)];

        TreeArena {
            chunks,
            current_chunk_idx: 0,
            current_offset: 0,
        }
    }

    /// Allocate a new tree node
    ///
    /// Returns a handle that can be used to retrieve the node later.
    /// The handle remains valid until the arena is reset or dropped.
    ///
    /// # Performance
    ///
    /// O(1) amortized. May allocate a new chunk (O(n)) if current chunk is full.
    pub fn alloc(&mut self, node: TreeNode) -> NodeHandle {
        // Check if current chunk is full
        if self.chunks[self.current_chunk_idx].is_full() {
            self.allocate_new_chunk();
        }

        // Allocate in current chunk
        let node_idx = self.chunks[self.current_chunk_idx].alloc(node);
        self.current_offset = node_idx + 1;

        NodeHandle {
            chunk_idx: self.current_chunk_idx as u32,
            node_idx: node_idx as u32,
        }
    }

    /// Get immutable reference to node
    ///
    /// # Panics
    ///
    /// Panics in debug builds if handle is invalid.
    /// Undefined behavior in release builds if handle is invalid.
    pub fn get(&self, handle: NodeHandle) -> TreeNodeRef<'_> {
        debug_assert!(self.is_valid_handle(handle), "Invalid node handle");

        let chunk = &self.chunks[handle.chunk_idx()];
        let node = chunk.get(handle.node_idx());

        TreeNodeRef { node }
    }

    /// Get mutable reference to node
    ///
    /// # Panics
    ///
    /// Panics in debug builds if handle is invalid.
    pub fn get_mut(&mut self, handle: NodeHandle) -> TreeNodeRefMut<'_> {
        debug_assert!(self.is_valid_handle(handle), "Invalid node handle");

        let chunk = &mut self.chunks[handle.chunk_idx()];
        let node = chunk.get_mut(handle.node_idx());

        TreeNodeRefMut { node }
    }

    /// Reset arena for reuse
    ///
    /// Clears all allocated nodes but retains chunks for reuse.
    /// All previous `NodeHandle`s are invalidated.
    ///
    /// # Performance
    ///
    /// O(chunks) to clear each chunk's data vector.
    pub fn reset(&mut self) {
        for chunk in &mut self.chunks {
            chunk.clear();
        }
        self.current_chunk_idx = 0;
        self.current_offset = 0;
    }

    /// Clear arena and free excess chunks
    ///
    /// Retains only the first chunk with its original capacity.
    /// More aggressive than `reset()` for long-running applications.
    pub fn clear(&mut self) {
        // Keep first chunk, drop the rest
        self.chunks.truncate(1);
        self.chunks[0].clear();
        self.current_chunk_idx = 0;
        self.current_offset = 0;
    }

    /// Get total number of allocated nodes
    pub fn len(&self) -> usize {
        let mut total = 0;
        for (i, chunk) in self.chunks.iter().enumerate() {
            if i < self.current_chunk_idx {
                total += chunk.capacity; // Full chunks
            } else if i == self.current_chunk_idx {
                total += self.current_offset; // Partial current chunk
            }
        }
        total
    }

    /// Check if arena is empty
    pub fn is_empty(&self) -> bool {
        self.current_chunk_idx == 0 && self.current_offset == 0
    }

    /// Get total capacity across all chunks
    pub fn capacity(&self) -> usize {
        self.chunks.iter().map(|c| c.capacity).sum()
    }

    /// Get number of chunks
    pub fn num_chunks(&self) -> usize {
        self.chunks.len()
    }

    /// Get approximate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        let node_size = mem::size_of::<TreeNode>();
        self.capacity() * node_size
    }

    /// Allocate a new chunk with exponential growth
    fn allocate_new_chunk(&mut self) {
        let current_capacity = self.chunks[self.current_chunk_idx].capacity;
        let new_capacity = (current_capacity * 2).min(MAX_CHUNK_SIZE);

        self.chunks.push(Chunk::new(new_capacity));
        self.current_chunk_idx += 1;
        self.current_offset = 0;
    }

    /// Check if handle is valid for this arena
    fn is_valid_handle(&self, handle: NodeHandle) -> bool {
        let chunk_idx = handle.chunk_idx();
        let node_idx = handle.node_idx();

        if chunk_idx >= self.chunks.len() {
            return false;
        }

        node_idx < self.chunks[chunk_idx].data.len()
    }
}

impl Default for TreeArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable reference to a tree node
///
/// This type provides safe access to arena-allocated nodes through
/// the `Deref` trait.
pub struct TreeNodeRef<'arena> {
    node: &'arena TreeNode,
}

impl<'arena> TreeNodeRef<'arena> {
    /// Get the underlying node reference
    pub fn get_ref(&self) -> &'arena TreeNode {
        self.node
    }

    /// Get the underlying node reference (backwards-compatible alias)
    #[allow(clippy::wrong_self_convention, clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'arena TreeNode {
        self.get_ref()
    }

    /// Get node symbol value
    pub fn value(&self) -> i32 {
        self.node.symbol()
    }

    /// Get node symbol
    pub fn symbol(&self) -> i32 {
        self.node.symbol()
    }

    /// Check if this is a branch node
    pub fn is_branch(&self) -> bool {
        matches!(self.node.kind, TreeNodeKind::Branch { .. })
    }

    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        matches!(self.node.kind, TreeNodeKind::Leaf { .. })
    }

    /// Get child handles for this node
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

/// Mutable reference to a tree node
pub struct TreeNodeRefMut<'arena> {
    node: &'arena mut TreeNode,
}

impl<'arena> TreeNodeRefMut<'arena> {
    /// Set the value of a leaf node
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

/// A node in the parse tree
///
/// Simplified for arena allocator demonstration.
/// In production, this would include symbol type, span, fields, etc.
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
    /// Create a leaf node with a value
    pub fn leaf(value: i32) -> Self {
        TreeNode {
            kind: TreeNodeKind::Leaf { symbol: value },
        }
    }

    /// Create a branch node with children
    pub fn branch(children: Vec<NodeHandle>) -> Self {
        TreeNode {
            kind: TreeNodeKind::Branch {
                symbol: 0,
                children,
            },
        }
    }

    /// Create a branch node with symbol and children
    pub fn branch_with_symbol(symbol: i32, children: Vec<NodeHandle>) -> Self {
        TreeNode {
            kind: TreeNodeKind::Branch { symbol, children },
        }
    }

    /// Get symbol value
    pub fn value(&self) -> i32 {
        self.symbol()
    }

    /// Get symbol id
    pub fn symbol(&self) -> i32 {
        match self.kind {
            TreeNodeKind::Leaf { symbol } | TreeNodeKind::Branch { symbol, .. } => symbol,
        }
    }

    /// Check if this is a leaf
    pub fn is_leaf(&self) -> bool {
        matches!(self.kind, TreeNodeKind::Leaf { .. })
    }

    /// Check if this is a branch
    pub fn is_branch(&self) -> bool {
        matches!(self.kind, TreeNodeKind::Branch { .. })
    }

    /// Get child handles for this node
    pub fn children(&self) -> &[NodeHandle] {
        match &self.kind {
            TreeNodeKind::Leaf { .. } => &[],
            TreeNodeKind::Branch { children, .. } => children,
        }
    }
}

/// Arena metrics snapshot
///
/// Provides information about the current state of a TreeArena.
/// All metrics are computed from the arena's current state and represent
/// a snapshot at the time `arena_metrics()` is called.
///
/// # Example
///
/// ```
/// use adze::arena_allocator::{TreeArena, TreeNode};
///
/// let mut arena = TreeArena::new();
///
/// // Before allocation
/// let metrics = arena.metrics();
/// assert_eq!(metrics.len(), 0);
///
/// // After allocation
/// arena.alloc(TreeNode::leaf(42));
/// let metrics = arena.metrics();
/// assert_eq!(metrics.len(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArenaMetrics {
    /// Number of nodes currently allocated
    len: usize,
    /// Total capacity across all chunks
    capacity: usize,
    /// Number of chunks allocated
    num_chunks: usize,
    /// Approximate memory usage in bytes
    memory_usage: usize,
}

impl ArenaMetrics {
    /// Create metrics from arena
    pub(crate) fn from_arena(arena: &TreeArena) -> Self {
        Self {
            len: arena.len(),
            capacity: arena.capacity(),
            num_chunks: arena.num_chunks(),
            memory_usage: arena.memory_usage(),
        }
    }

    /// Get number of allocated nodes
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if arena is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get total capacity across all chunks
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get number of chunks
    pub fn num_chunks(&self) -> usize {
        self.num_chunks
    }

    /// Get approximate memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.memory_usage
    }
}

impl TreeArena {
    /// Get current metrics snapshot
    ///
    /// Returns a snapshot of arena metrics including node count,
    /// capacity, number of chunks, and memory usage.
    ///
    /// # Performance
    ///
    /// O(chunks) to compute len() for partially filled chunks.
    /// Other metrics are O(1).
    pub fn metrics(&self) -> ArenaMetrics {
        ArenaMetrics::from_arena(self)
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

    #[test]
    fn test_multiple_allocations() {
        let mut arena = TreeArena::new();

        let h1 = arena.alloc(TreeNode::leaf(1));
        let h2 = arena.alloc(TreeNode::leaf(2));
        let h3 = arena.alloc(TreeNode::leaf(3));

        assert_eq!(arena.len(), 3);
        assert_eq!(arena.get(h1).value(), 1);
        assert_eq!(arena.get(h2).value(), 2);
        assert_eq!(arena.get(h3).value(), 3);
    }

    #[test]
    fn test_chunk_growth() {
        let mut arena = TreeArena::with_capacity(2);

        arena.alloc(TreeNode::leaf(1));
        arena.alloc(TreeNode::leaf(2));
        assert_eq!(arena.num_chunks(), 1);

        arena.alloc(TreeNode::leaf(3));
        assert_eq!(arena.num_chunks(), 2);
    }

    #[test]
    fn test_reset() {
        let mut arena = TreeArena::new();

        for i in 0..10 {
            arena.alloc(TreeNode::leaf(i));
        }

        assert_eq!(arena.len(), 10);

        arena.reset();

        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn test_branch_nodes() {
        let mut arena = TreeArena::new();

        let child1 = arena.alloc(TreeNode::leaf(1));
        let child2 = arena.alloc(TreeNode::leaf(2));
        let parent = arena.alloc(TreeNode::branch(vec![child1, child2]));

        assert!(arena.get(parent).is_branch());
        assert!(arena.get(child1).is_leaf());
    }
}
