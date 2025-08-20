//! Subtree representation and manipulation.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

// Subtree representation with dynamic precedence support

use rust_sitter_ir::SymbolId;
use std::sync::Arc;

/// Node information for a subtree
#[derive(Debug, Clone)]
pub struct SubtreeNode {
    /// Symbol ID for this node
    pub symbol_id: SymbolId,

    /// Whether this node is an error node
    pub is_error: bool,

    /// Byte range in source text
    pub byte_range: std::ops::Range<usize>,
}

/// A child edge with optional field information
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ChildEdge {
    /// The child subtree
    pub subtree: Arc<Subtree>,

    /// Field ID for this child (u16::MAX means no field)
    pub field_id: u16,
}

/// Constant representing "no field" for a child edge
pub const FIELD_NONE: u16 = u16::MAX;

impl ChildEdge {
    /// Create a new ChildEdge
    pub fn new(subtree: Arc<Subtree>, field_id: u16) -> Self {
        Self { subtree, field_id }
    }

    /// Create a ChildEdge without a field
    pub fn new_without_field(subtree: Arc<Subtree>) -> Self {
        Self {
            subtree,
            field_id: FIELD_NONE,
        }
    }
}

/// A subtree in the parse tree, potentially with dynamic precedence
#[derive(Debug, Clone)]
pub struct Subtree {
    /// The tree node data
    pub node: SubtreeNode,

    /// Dynamic precedence value for this subtree
    /// Set by prec.dynamic(n) annotations in the grammar
    pub dynamic_prec: i32,

    /// Child subtrees with optional field information
    pub children: Vec<ChildEdge>,
}

impl Subtree {
    /// Create a new subtree with the given node and children (no field info)
    pub fn new(node: SubtreeNode, children: Vec<Arc<Subtree>>) -> Self {
        // Convert to ChildEdge with no field
        let children_with_fields = children
            .into_iter()
            .map(|subtree| ChildEdge {
                subtree,
                field_id: FIELD_NONE,
            })
            .collect::<Vec<_>>();

        // Propagate dynamic precedence upward (max of children)
        let max_child_prec = children_with_fields
            .iter()
            .map(|c| c.subtree.dynamic_prec)
            .max()
            .unwrap_or(0);

        Self {
            node,
            dynamic_prec: max_child_prec,
            children: children_with_fields,
        }
    }

    /// Create a new subtree with field information for children
    pub fn new_with_fields(node: SubtreeNode, children: Vec<ChildEdge>) -> Self {
        // Propagate dynamic precedence upward (max of children)
        let max_child_prec = children
            .iter()
            .map(|c| c.subtree.dynamic_prec)
            .max()
            .unwrap_or(0);

        Self {
            node,
            dynamic_prec: max_child_prec,
            children,
        }
    }

    /// Create a new subtree with explicit dynamic precedence (no field info)
    pub fn with_dynamic_prec(
        node: SubtreeNode,
        children: Vec<Arc<Subtree>>,
        dynamic_prec: i32,
    ) -> Self {
        // Convert to ChildEdge with no field
        let children_with_fields = children
            .into_iter()
            .map(|subtree| ChildEdge {
                subtree,
                field_id: FIELD_NONE,
            })
            .collect::<Vec<_>>();

        // Take max of explicit precedence and children's precedence
        let max_child_prec = children_with_fields
            .iter()
            .map(|c| c.subtree.dynamic_prec)
            .max()
            .unwrap_or(0);

        Self {
            node,
            dynamic_prec: dynamic_prec.max(max_child_prec),
            children: children_with_fields,
        }
    }

    /// Create a new subtree with explicit dynamic precedence and field info
    pub fn with_dynamic_prec_and_fields(
        node: SubtreeNode,
        children: Vec<ChildEdge>,
        dynamic_prec: i32,
    ) -> Self {
        // Take max of explicit precedence and children's precedence
        let max_child_prec = children
            .iter()
            .map(|c| c.subtree.dynamic_prec)
            .max()
            .unwrap_or(0);

        Self {
            node,
            dynamic_prec: dynamic_prec.max(max_child_prec),
            children,
        }
    }

    /// Get the symbol ID for this subtree
    pub fn symbol(&self) -> u16 {
        self.node.symbol_id.0
    }

    /// Check if this subtree is in error
    pub fn is_error(&self) -> bool {
        self.node.is_error
    }

    /// Get the byte range for this subtree
    pub fn byte_range(&self) -> std::ops::Range<usize> {
        self.node.byte_range.clone()
    }
}
