//! Subtree representation and manipulation.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

// Subtree representation with dynamic precedence support

use rust_sitter_ir::SymbolId;
use smallvec::SmallVec;
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

    /// Alternative parse trees for ambiguous nodes
    /// Empty = single parse, non-empty = ambiguity pack
    pub alternatives: SmallVec<[Arc<Subtree>; 2]>,
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
            alternatives: SmallVec::new(),
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
            alternatives: SmallVec::new(),
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
            alternatives: SmallVec::new(),
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
            alternatives: SmallVec::new(),
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

    /// Check if this subtree has ambiguous alternatives
    pub fn is_ambiguous(&self) -> bool {
        !self.alternatives.is_empty()
    }

    /// Check if this subtree has alternatives
    pub fn has_alts(&self) -> bool {
        !self.alternatives.is_empty()
    }

    /// Get all alternatives (not including the primary tree)
    pub fn alternatives_iter(&self) -> impl Iterator<Item = &Arc<Subtree>> {
        self.alternatives.iter()
    }

    /// Merge two subtrees with the same top, preserving all alternatives
    pub fn merge_ambiguous(mut self, other: Arc<Subtree>) -> Self {
        // If the other tree also has alternatives, merge them all
        if !other.alternatives.is_empty() {
            for alt in &other.alternatives {
                if !self.alternatives.iter().any(|a| Arc::ptr_eq(a, alt)) {
                    self.alternatives.push(alt.clone());
                }
            }
        }

        // Add the other tree itself as an alternative (if not already present)
        // Need to check by pointer equality since we're moving other
        let other_ptr = Arc::as_ptr(&other);
        if !self
            .alternatives
            .iter()
            .any(|a| Arc::as_ptr(a) == other_ptr)
        {
            // Keep the highest dynamic precedence before moving
            self.dynamic_prec = self.dynamic_prec.max(other.dynamic_prec);
            self.alternatives.push(other);
        } else {
            // Still update precedence even if not adding
            self.dynamic_prec = self.dynamic_prec.max(other.dynamic_prec);
        }

        self
    }

    /// Create a new subtree with the given alternative
    pub fn with_alts(mut self, alt: Arc<Subtree>) -> Self {
        if !Arc::ptr_eq(&Arc::new(self.clone()), &alt) {
            self.alternatives.push(alt);
        }
        self
    }

    /// Add an alternative to this subtree (deduplicating by pointer)
    pub fn push_alt(mut self, alt: Arc<Subtree>) -> Self {
        let alt_ptr = Arc::as_ptr(&alt);
        if !self.alternatives.iter().any(|a| Arc::as_ptr(a) == alt_ptr) {
            self.dynamic_prec = self.dynamic_prec.max(alt.dynamic_prec);
            self.alternatives.push(alt);
        }
        self
    }

    /// Concatenate alternatives from two subtrees (deduplicating)
    pub fn concat_alts(mut self, other: Arc<Subtree>) -> Self {
        // First add the other tree as an alternative
        self = self.push_alt(other.clone());

        // Then add all of its alternatives
        for alt in &other.alternatives {
            if !self.alternatives.iter().any(|a| Arc::ptr_eq(a, alt)) {
                self.alternatives.push(alt.clone());
            }
        }

        self
    }
}
