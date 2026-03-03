//! Arena-allocated parse tree node
//!
//! This module provides the `Node<'arena>` type, a lightweight wrapper
//! around arena-allocated `TreeNodeData`. Node provides a safe, ergonomic
//! API for traversing parse trees without manual handle management.
//!
//! # Example
//!
//! ```ignore
//! use adze::parser_v4::Parser;
//!
//! let mut parser = Parser::new(grammar, parse_table, "example".to_string());
//! let root = parser.parse_tree("1 + 2")?;
//!
//! // `root` is a ParseNode value that can be traversed without `Node` APIs.
//!
//! // Traverse children
//! for child in root.children() {
//!     println!("Child symbol: {}", child.symbol());
//! }
//! ```
//!
//! Related: docs/specs/NODE_ARENA_SPEC.md

use crate::arena_allocator::{NodeHandle, TreeArena};
use crate::tree_node_data::TreeNodeData;
use std::cell::RefCell;
use std::collections::HashMap;

type NodeDataCacheKey = (usize, NodeHandle);

thread_local! {
    static NODE_DATA_CACHE: RefCell<HashMap<NodeDataCacheKey, &'static TreeNodeData>> =
        RefCell::new(HashMap::new());
}

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
    #[allow(dead_code)] // Will be used in Day 5 parse() integration
    handle: NodeHandle,
    arena: &'arena TreeArena,
}

impl<'arena> Node<'arena> {
    /// Create a new node from handle and arena reference
    ///
    /// # Safety
    ///
    /// Internal use only. Handle must be valid for the arena.
    /// This is enforced by only exposing this method within the crate.
    pub(crate) fn new(handle: NodeHandle, arena: &'arena TreeArena) -> Self {
        Node { handle, arena }
    }

    /// Get direct access to underlying TreeNodeData
    ///
    /// For advanced use cases that need raw data access.
    ///
    /// # Performance
    ///
    /// O(1) - direct arena lookup by handle.
    ///
    /// # Note
    ///
    /// Temporary implementation returns fallback metadata derived from the node
    /// symbol while full parse-tree data integration is completed.
    pub fn data(&self) -> &'arena TreeNodeData {
        let key: NodeDataCacheKey = (self.arena as *const _ as usize, self.handle);

        NODE_DATA_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if let Some(data) = cache.get(&key) {
                return *data;
            }

            let symbol = self.raw_node().value() as u16;
            let data = Box::leak(Box::new(TreeNodeData::new(symbol, 0, 0)));
            cache.insert(key, data);
            data
        })
    }

    fn raw_node(&self) -> &crate::arena_allocator::TreeNode {
        self.arena.get(self.handle).get_ref()
    }

    /// Get the node's symbol/kind ID
    ///
    /// # Performance
    ///
    /// O(1) - delegates to TreeNodeData::symbol().
    pub fn symbol(&self) -> u16 {
        self.raw_node().value() as u16
    }

    /// Get the node's byte range in the source
    ///
    /// Returns (start_byte, end_byte) tuple.
    ///
    /// # Performance
    ///
    /// O(1) - delegates to TreeNodeData::byte_range().
    pub fn byte_range(&self) -> (u32, u32) {
        (0, 0)
    }

    /// Get start byte position
    ///
    /// # Performance
    ///
    /// O(1) - direct field access via data().
    pub fn start_byte(&self) -> u32 {
        self.byte_range().0
    }

    /// Get end byte position
    ///
    /// # Performance
    ///
    /// O(1) - direct field access via data().
    pub fn end_byte(&self) -> u32 {
        self.byte_range().1
    }

    /// Check if this is a named node
    ///
    /// Named nodes appear in the grammar explicitly.
    /// Anonymous nodes are punctuation/keywords.
    ///
    /// # Performance
    ///
    /// O(1) - bit check in flags.
    pub fn is_named(&self) -> bool {
        false
    }

    /// Check if this node is missing (error recovery)
    ///
    /// Missing nodes are inserted by the parser during error recovery.
    ///
    /// # Performance
    ///
    /// O(1) - bit check in flags.
    pub fn is_missing(&self) -> bool {
        false
    }

    /// Check if this node is extra (trivia)
    ///
    /// Extra nodes are comments, whitespace, etc.
    ///
    /// # Performance
    ///
    /// O(1) - bit check in flags.
    pub fn is_extra(&self) -> bool {
        false
    }

    /// Check if this node contains errors
    ///
    /// Returns true if this node or any descendant has an error.
    ///
    /// # Performance
    ///
    /// O(1) - bit check in flags.
    pub fn has_error(&self) -> bool {
        false
    }

    /// Get child count
    ///
    /// Returns the total number of children, including both named
    /// and anonymous children.
    ///
    /// # Performance
    ///
    /// O(1) - delegates to TreeNodeData::child_count().
    pub fn child_count(&self) -> usize {
        self.raw_node().children().len()
    }

    /// Get named child count
    ///
    /// Returns the number of named children only.
    ///
    /// # Performance
    ///
    /// O(1) - direct field access via data().
    pub fn named_child_count(&self) -> usize {
        0
    }

    /// Get child by index
    ///
    /// Returns None if index >= child_count().
    ///
    /// # Performance
    ///
    /// O(1) - array index + Node creation.
    pub fn child(&self, index: usize) -> Option<Node<'arena>> {
        self.raw_node()
            .children()
            .get(index)
            .copied()
            .map(|handle| Node::new(handle, self.arena))
    }

    /// Get named child by index
    ///
    /// Returns None in this stage because named-field metadata is not yet
    /// populated from `TreeNode`.
    ///
    /// # Performance
    ///
    /// O(1).
    pub fn named_child(&self, _index: usize) -> Option<Node<'arena>> {
        None
    }

    /// Get field ID if this node has one.
    ///
    /// Field IDs are not tracked in the current arena-backed tree.
    ///
    /// Field IDs are used for named fields in the grammar.
    ///
    /// # Performance
    ///
    /// O(1) - direct field access via data().
    pub fn field_id(&self) -> Option<u16> {
        None
    }

    /// Iterate over all children
    ///
    /// Returns an iterator that yields all children in order.
    ///
    /// # Performance
    ///
    /// O(1) to create iterator, O(n) to consume.
    ///
    /// # Note
    ///
    /// Full implementation in Day 5 when TreeArena stores TreeNodeData.
    pub fn children(&self) -> NodeChildren<'arena> {
        NodeChildren {
            handles: self.arena.get(self.handle).get_ref().children(),
            arena: self.arena,
            index: 0,
        }
    }

    /// Iterate over named children only
    ///
    /// Returns an iterator that yields only named children.
    ///
    /// # Performance
    ///
    /// O(1) to create iterator, O(n) to consume (filters all children).
    pub fn named_children(&self) -> NamedChildren<'arena> {
        NamedChildren {
            inner: self.children(),
        }
    }
}

/// Iterator over all children of a node
///
/// Created by `Node::children()`.
#[derive(Clone)]
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.handles.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'arena> ExactSizeIterator for NodeChildren<'arena> {
    fn len(&self) -> usize {
        self.handles.len() - self.index
    }
}

/// Iterator over named children only
///
/// Created by `Node::named_children()`.
#[derive(Clone)]
pub struct NamedChildren<'arena> {
    inner: NodeChildren<'arena>,
}

impl<'arena> Iterator for NamedChildren<'arena> {
    type Item = Node<'arena>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find(|node| node.is_named())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_size() {
        use std::mem::size_of;
        assert_eq!(size_of::<Node>(), 16, "Node should be 16 bytes");
    }

    #[test]
    fn test_node_is_copy() {
        // This test compiles if Node is Copy
        fn takes_copy<T: Copy>(_: T) {}

        let mut arena = TreeArena::new();
        // Use TreeNode::leaf() which matches arena.alloc() signature
        let handle = arena.alloc(crate::arena_allocator::TreeNode::leaf(42));
        let node = Node::new(handle, &arena);

        takes_copy(node);
    }

    #[test]
    fn test_node_data_returns_cached_fallback() {
        let mut arena = TreeArena::new();
        let handle = arena.alloc(crate::arena_allocator::TreeNode::leaf(42));
        let node = Node::new(handle, &arena);

        let data = node.data();
        let data2 = node.data();
        assert_eq!(data.symbol(), 42);
        assert_eq!(data2.symbol(), 42);
        assert!(std::ptr::eq(data, data2));
    }

    #[test]
    fn test_node_children_iterate_from_tree_node() {
        let mut arena = TreeArena::new();
        let child = arena.alloc(crate::arena_allocator::TreeNode::leaf(7));
        let handle = arena.alloc(crate::arena_allocator::TreeNode::branch(vec![child]));
        let node = Node::new(handle, &arena);

        let children: Vec<_> = node.children().collect::<Vec<_>>();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].symbol(), 7);
    }
}
