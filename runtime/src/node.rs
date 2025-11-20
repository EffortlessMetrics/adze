//! Arena-allocated parse tree node
//!
//! This module provides the `Node<'arena>` type, a lightweight wrapper
//! around arena-allocated `TreeNodeData`. Node provides a safe, ergonomic
//! API for traversing parse trees without manual handle management.
//!
//! # Example
//!
//! ```ignore
//! use rust_sitter::parser_v4::Parser;
//!
//! let mut parser = Parser::new(grammar, parse_table, "example".to_string());
//! let tree = parser.parse("1 + 2")?;
//!
//! // Get root node
//! let root = tree.root_node();
//!
//! // Traverse children
//! for child in root.children() {
//!     println!("Child symbol: {}", child.symbol());
//! }
//! ```
//!
//! Related: docs/specs/NODE_ARENA_SPEC.md

use crate::arena_allocator::{NodeHandle, TreeArena, TreeNodeRef};
use crate::tree_node_data::TreeNodeData;

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
    /// This method will be fully implemented in Day 5 when parse() integration
    /// allocates TreeNodeData in the arena. For Day 4, we're establishing the
    /// type signatures.
    pub fn data(&self) -> &'arena TreeNodeData {
        // TODO(Phase 2 Day 5): Implement when TreeArena stores TreeNodeData
        // For now, we need to integrate TreeNodeData allocation into TreeArena
        // Current TreeArena stores TreeNode (demo type), not TreeNodeData
        unimplemented!("data() will be implemented in Day 5 parse() integration")
    }

    /// Get the node's symbol/kind ID
    ///
    /// # Performance
    ///
    /// O(1) - delegates to TreeNodeData::symbol().
    pub fn symbol(&self) -> u16 {
        self.data().symbol()
    }

    /// Get the node's byte range in the source
    ///
    /// Returns (start_byte, end_byte) tuple.
    ///
    /// # Performance
    ///
    /// O(1) - delegates to TreeNodeData::byte_range().
    pub fn byte_range(&self) -> (u32, u32) {
        self.data().byte_range()
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
        self.data().is_named()
    }

    /// Check if this node is missing (error recovery)
    ///
    /// Missing nodes are inserted by the parser during error recovery.
    ///
    /// # Performance
    ///
    /// O(1) - bit check in flags.
    pub fn is_missing(&self) -> bool {
        self.data().is_missing()
    }

    /// Check if this node is extra (trivia)
    ///
    /// Extra nodes are comments, whitespace, etc.
    ///
    /// # Performance
    ///
    /// O(1) - bit check in flags.
    pub fn is_extra(&self) -> bool {
        self.data().is_extra()
    }

    /// Check if this node contains errors
    ///
    /// Returns true if this node or any descendant has an error.
    ///
    /// # Performance
    ///
    /// O(1) - bit check in flags.
    pub fn has_error(&self) -> bool {
        self.data().is_error()
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
        self.data().child_count()
    }

    /// Get named child count
    ///
    /// Returns the number of named children only.
    ///
    /// # Performance
    ///
    /// O(1) - direct field access via data().
    pub fn named_child_count(&self) -> usize {
        self.data().named_child_count() as usize
    }

    /// Get child by index
    ///
    /// Returns None if index >= child_count().
    ///
    /// # Performance
    ///
    /// O(1) - array index + Node creation.
    pub fn child(&self, index: usize) -> Option<Node<'arena>> {
        self.data()
            .child(index)
            .map(|handle| Node::new(handle, self.arena))
    }

    /// Get named child by index
    ///
    /// Returns the ith named child, skipping anonymous children.
    /// Returns None if index >= named_child_count().
    ///
    /// # Performance
    ///
    /// O(n) where n = number of children (must filter by is_named).
    pub fn named_child(&self, index: usize) -> Option<Node<'arena>> {
        let mut named_count = 0;
        for child_handle in self.data().children() {
            let child_node = Node::new(*child_handle, self.arena);
            if child_node.is_named() {
                if named_count == index {
                    return Some(child_node);
                }
                named_count += 1;
            }
        }
        None
    }

    /// Get field ID if this node has one
    ///
    /// Field IDs are used for named fields in the grammar.
    ///
    /// # Performance
    ///
    /// O(1) - direct field access via data().
    pub fn field_id(&self) -> Option<u16> {
        self.data().field_id()
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
        // TODO(Phase 2 Day 5): Implement when TreeArena stores TreeNodeData
        unimplemented!("children() will be implemented in Day 5 parse() integration")
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
        let handle = arena.alloc(TreeNodeData::leaf(1, 0, 10));
        let node = Node::new(handle, &arena);

        takes_copy(node);
    }
}
