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

/// A subtree in the parse tree, potentially with dynamic precedence
#[derive(Debug, Clone)]
pub struct Subtree {
    /// The tree node data
    pub node: SubtreeNode,
    
    /// Dynamic precedence value for this subtree
    /// Set by prec.dynamic(n) annotations in the grammar
    pub dynamic_prec: i32,
    
    /// Child subtrees
    pub children: Vec<Arc<Subtree>>,
}

impl Subtree {
    /// Create a new subtree with the given node and children
    pub fn new(node: SubtreeNode, children: Vec<Arc<Subtree>>) -> Self {
        // Propagate dynamic precedence upward (max of children)
        let max_child_prec = children
            .iter()
            .map(|c| c.dynamic_prec)
            .max()
            .unwrap_or(0);
        
        Self {
            node,
            dynamic_prec: max_child_prec,
            children,
        }
    }
    
    /// Create a new subtree with explicit dynamic precedence
    pub fn with_dynamic_prec(node: SubtreeNode, children: Vec<Arc<Subtree>>, dynamic_prec: i32) -> Self {
        // Take max of explicit precedence and children's precedence
        let max_child_prec = children
            .iter()
            .map(|c| c.dynamic_prec)
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
}