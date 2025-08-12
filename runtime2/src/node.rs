//! Node representation for syntax tree nodes

use crate::Language;
use std::fmt;

/// A node in the syntax tree
#[derive(Clone, Copy)]
pub struct Node<'tree> {
    /// Internal node data (placeholder)
    _data: &'tree (),
    /// Language reference
    _language: Option<&'tree Language>,
}

impl<'tree> Node<'tree> {
    /// Create a new node (internal use)
    pub(crate) fn new(_node: &'tree impl std::any::Any, _language: Option<&'tree Language>) -> Self {
        Self {
            _data: &(),
            _language,
        }
    }

    /// Get the node's symbol type
    pub fn kind(&self) -> &str {
        "placeholder"
    }

    /// Get the node's symbol ID
    pub fn kind_id(&self) -> u16 {
        0
    }

    /// Get the node's byte range
    pub fn byte_range(&self) -> std::ops::Range<usize> {
        0..0
    }

    /// Get the node's start byte
    pub fn start_byte(&self) -> usize {
        self.byte_range().start
    }

    /// Get the node's end byte
    pub fn end_byte(&self) -> usize {
        self.byte_range().end
    }

    /// Get the node's start position
    pub fn start_position(&self) -> Point {
        Point { row: 0, column: 0 }
    }

    /// Get the node's end position
    pub fn end_position(&self) -> Point {
        Point { row: 0, column: 0 }
    }

    /// Check if this node is named (visible in the tree)
    pub fn is_named(&self) -> bool {
        true
    }

    /// Check if this node is missing (error recovery)
    pub fn is_missing(&self) -> bool {
        false
    }

    /// Check if this node is an error node
    pub fn is_error(&self) -> bool {
        false
    }

    /// Get the number of children
    pub fn child_count(&self) -> usize {
        0
    }

    /// Get the number of named children
    pub fn named_child_count(&self) -> usize {
        0
    }

    /// Get a child by index
    pub fn child(&self, index: usize) -> Option<Node<'tree>> {
        let _ = index;
        None
    }

    /// Get a named child by index
    pub fn named_child(&self, index: usize) -> Option<Node<'tree>> {
        let _ = index;
        None
    }

    /// Get a child by field name
    pub fn child_by_field_name(&self, field_name: &str) -> Option<Node<'tree>> {
        let _ = field_name;
        None
    }

    /// Get the parent node
    pub fn parent(&self) -> Option<Node<'tree>> {
        None
    }

    /// Get the next sibling
    pub fn next_sibling(&self) -> Option<Node<'tree>> {
        None
    }

    /// Get the previous sibling
    pub fn prev_sibling(&self) -> Option<Node<'tree>> {
        None
    }

    /// Get the next named sibling
    pub fn next_named_sibling(&self) -> Option<Node<'tree>> {
        None
    }

    /// Get the previous named sibling
    pub fn prev_named_sibling(&self) -> Option<Node<'tree>> {
        None
    }

    /// Get the UTF-8 text of this node
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