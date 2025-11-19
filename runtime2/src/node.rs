//! Node representation for syntax tree nodes
//!
//! Contract: docs/specs/NODE_API_CONTRACT.md

use crate::{Language, tree::TreeNode};
use std::fmt;

/// A node in the syntax tree
///
/// Provides read-only access to tree node data with lifetime tied to parent Tree.
///
/// # Contract
///
/// - Nodes are Copy and read-only (no mutation)
/// - Lifetime `'tree` tied to parent Tree
/// - Child access is always safe (returns None if out of bounds)
/// - Byte ranges are always valid (start <= end)
///
/// See: docs/specs/NODE_API_CONTRACT.md
#[derive(Clone, Copy)]
pub struct Node<'tree> {
    /// Reference to internal tree node data
    data: &'tree TreeNode,
    /// Language reference for symbol metadata
    language: Option<&'tree Language>,
}

impl<'tree> Node<'tree> {
    /// Create a new node (internal use)
    ///
    /// # Contract
    ///
    /// - `node` must be a valid TreeNode with valid ranges
    /// - `language` is optional (GLR mode may not have Language)
    pub(crate) fn new(
        node: &'tree TreeNode,
        language: Option<&'tree Language>,
    ) -> Self {
        Self {
            data: node,
            language,
        }
    }

    /// Get the node's symbol type
    ///
    /// # Phase 1: Returns "unknown" (no symbol name resolution yet)
    /// # Phase 2: Will use language.symbol_metadata for actual names
    pub fn kind(&self) -> &str {
        "unknown"
    }

    /// Get the node's symbol ID
    ///
    /// # Contract
    ///
    /// - Returns `data.symbol as u16`
    /// - Maps to grammar symbol IDs
    pub fn kind_id(&self) -> u16 {
        self.data.symbol as u16
    }

    /// Get the node's byte range
    ///
    /// # Contract
    ///
    /// - Returns `data.start_byte..data.end_byte`
    /// - Range is always valid: start <= end
    /// - Measured in bytes, not characters
    pub fn byte_range(&self) -> std::ops::Range<usize> {
        self.data.start_byte..self.data.end_byte
    }

    /// Get the node's start byte
    pub fn start_byte(&self) -> usize {
        self.data.start_byte
    }

    /// Get the node's end byte
    pub fn end_byte(&self) -> usize {
        self.data.end_byte
    }

    /// Get the node's start position
    ///
    /// # Phase 1: Returns dummy (0, 0)
    /// # Phase 2: Will calculate from byte positions
    pub fn start_position(&self) -> Point {
        Point { row: 0, column: 0 }
    }

    /// Get the node's end position
    ///
    /// # Phase 1: Returns dummy (0, 0)
    /// # Phase 2: Will calculate from byte positions
    pub fn end_position(&self) -> Point {
        Point { row: 0, column: 0 }
    }

    /// Check if this node is named (visible in the tree)
    ///
    /// # Phase 1: Always returns true
    /// # Phase 2: Will use symbol_metadata.visible
    pub fn is_named(&self) -> bool {
        true
    }

    /// Check if this node is missing (error recovery)
    ///
    /// # Contract
    ///
    /// - Returns false (error recovery not implemented)
    pub fn is_missing(&self) -> bool {
        false
    }

    /// Check if this node is an error node
    ///
    /// # Contract
    ///
    /// - Returns false (error nodes not implemented)
    pub fn is_error(&self) -> bool {
        false
    }

    /// Get the number of children
    ///
    /// # Contract
    ///
    /// - Returns `data.children.len()`
    /// - Includes both named and anonymous children
    /// - Returns 0 for terminal nodes
    pub fn child_count(&self) -> usize {
        self.data.children.len()
    }

    /// Get the number of named children
    ///
    /// # Phase 1: Returns child_count() (no filtering)
    /// # Phase 2: Will filter by symbol_metadata.visible
    pub fn named_child_count(&self) -> usize {
        self.child_count()
    }

    /// Get a child by index
    ///
    /// # Contract
    ///
    /// - Returns Some(child) if index < child_count()
    /// - Returns None if index out of bounds
    /// - Child inherits parent's language
    pub fn child(&self, index: usize) -> Option<Node<'tree>> {
        self.data.children.get(index).map(|child| Node {
            data: child,
            language: self.language,
        })
    }

    /// Get a named child by index
    ///
    /// # Phase 1: Same as child(index) (no filtering)
    /// # Phase 2: Will skip unnamed children
    pub fn named_child(&self, index: usize) -> Option<Node<'tree>> {
        self.child(index)
    }

    /// Get a child by field name
    ///
    /// # Contract
    ///
    /// - Returns None (field access not implemented)
    pub fn child_by_field_name(&self, field_name: &str) -> Option<Node<'tree>> {
        let _ = field_name;
        None
    }

    /// Get the parent node
    ///
    /// # Contract
    ///
    /// - Returns None (parent links not stored)
    pub fn parent(&self) -> Option<Node<'tree>> {
        None
    }

    /// Get the next sibling
    ///
    /// # Contract
    ///
    /// - Returns None (sibling links not stored)
    pub fn next_sibling(&self) -> Option<Node<'tree>> {
        None
    }

    /// Get the previous sibling
    ///
    /// # Contract
    ///
    /// - Returns None (sibling links not stored)
    pub fn prev_sibling(&self) -> Option<Node<'tree>> {
        None
    }

    /// Get the next named sibling
    ///
    /// # Contract
    ///
    /// - Returns None (sibling links not stored)
    pub fn next_named_sibling(&self) -> Option<Node<'tree>> {
        None
    }

    /// Get the previous named sibling
    ///
    /// # Contract
    ///
    /// - Returns None (sibling links not stored)
    pub fn prev_named_sibling(&self) -> Option<Node<'tree>> {
        None
    }

    /// Get the UTF-8 text of this node
    ///
    /// # Contract
    ///
    /// - Extracts source[self.byte_range()]
    /// - Validates UTF-8 and returns error if invalid
    /// - Lifetime 'a independent of 'tree
    pub fn utf8_text<'a>(&self, source: &'a [u8]) -> Result<&'a str, std::str::Utf8Error> {
        let range = self.byte_range();
        std::str::from_utf8(&source[range])
    }
}

impl fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Node {{ kind: {}, range: {:?} }}",
            self.kind(),
            self.byte_range()
        )
    }
}

/// A point in the source text
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    /// Zero-indexed row
    pub row: usize,
    /// Zero-indexed column (in bytes)
    pub column: usize,
}

impl Point {
    /// Create a new point
    pub const fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.row + 1, self.column + 1)
    }
}
