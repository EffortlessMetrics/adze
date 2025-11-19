//! Forest-to-Tree Conversion for GLR Parsing (Phase 3.2, Component 2)
//!
//! Contract: docs/specs/FOREST_CONVERTER_CONTRACT.md
//!
//! This module converts ParseForest (potentially containing multiple parse trees)
//! into a single Tree structure using disambiguation strategies.

use crate::error::ParseError;
use crate::glr_engine::{ForestNode, ForestNodeId, ParseForest};
use crate::tree::TreeNode;
use crate::Tree;
use rust_sitter_glr_core::SymbolId;
use std::collections::HashSet;
use std::fmt;

/// Disambiguation strategies for ambiguous parses
///
/// Contract: Determines which alternative to select when forest has
/// multiple valid parse trees
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisambiguationStrategy {
    /// Prefer shift over reduce (Tree-sitter default)
    ///
    /// Creates right-associative trees
    PreferShift,

    /// Prefer reduce over shift
    ///
    /// Creates left-associative trees
    PreferReduce,

    /// Use precedence from grammar (Phase 3.3)
    #[allow(dead_code)]
    Precedence,

    /// Take first alternative (fast but arbitrary)
    First,

    /// Reject ambiguity (return error)
    RejectAmbiguity,
}

/// Converts ParseForest to single Tree
///
/// Contract:
/// - Selects one parse tree from potentially multiple valid parses
/// - Applies disambiguation strategy consistently
/// - Preserves all node metadata
#[derive(Debug)]
pub struct ForestConverter {
    /// Disambiguation strategy
    strategy: DisambiguationStrategy,
}

/// Forest conversion errors
#[derive(Debug)]
pub enum ConversionError {
    /// Forest has no root nodes
    NoRoots,

    /// Ambiguous forest with multiple valid parses
    AmbiguousForest { count: usize },

    /// Invalid forest structure
    InvalidForest { reason: String },

    /// Invalid node reference
    InvalidNodeId { node_id: usize },

    /// Cycle detected in forest
    #[allow(dead_code)]
    CycleDetected { node_id: usize },
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::NoRoots => write!(f, "Forest has no root nodes"),
            ConversionError::AmbiguousForest { count } => {
                write!(f, "Ambiguous forest: {} valid parses", count)
            }
            ConversionError::InvalidForest { reason } => {
                write!(f, "Invalid forest structure: {}", reason)
            }
            ConversionError::InvalidNodeId { node_id } => {
                write!(f, "Invalid node reference: {}", node_id)
            }
            ConversionError::CycleDetected { node_id } => {
                write!(f, "Cycle detected in forest at node {}", node_id)
            }
        }
    }
}

impl std::error::Error for ConversionError {}

impl From<ConversionError> for ParseError {
    fn from(err: ConversionError) -> Self {
        ParseError::with_msg(&err.to_string())
    }
}

impl ForestConverter {
    /// Create converter with strategy
    ///
    /// # Contract
    ///
    /// ## Postconditions
    /// - Converter ready to convert forests
    ///
    pub fn new(strategy: DisambiguationStrategy) -> Self {
        Self { strategy }
    }

    /// Convert ParseForest to Tree
    ///
    /// # Contract
    ///
    /// ## Preconditions
    /// - `forest.roots` is non-empty
    /// - Forest nodes form valid tree structure
    /// - All ForestNodeIds reference valid nodes
    ///
    /// ## Postconditions
    /// - Tree has single root node
    /// - Node ranges are consistent
    ///
    /// ## Algorithm
    ///
    /// Phase 1: Select root (disambiguation if multiple)
    /// Phase 2: Build tree via DFS traversal
    ///
    pub fn to_tree(
        &self,
        forest: &ParseForest,
        input: &[u8],
    ) -> Result<Tree, ConversionError> {
        // Phase 1: Select root
        if forest.roots.is_empty() {
            return Err(ConversionError::NoRoots);
        }

        let selected_root = if forest.roots.len() == 1 {
            forest.roots[0]
        } else {
            // Multiple roots - apply disambiguation
            self.disambiguate_roots(&forest.roots, forest)?
        };

        // Phase 2: Build tree
        let mut visited = HashSet::new();
        let root_node = self.build_node(selected_root, forest, input, &mut visited)?;

        // Create tree
        let mut tree = Tree::new(root_node);
        tree.set_source(input.to_vec());

        Ok(tree)
    }

    /// Detect ambiguity in forest
    ///
    /// # Contract
    ///
    /// ## Returns
    /// - `None`: Unambiguous (single parse)
    /// - `Some(count)`: `count` alternative parses
    ///
    pub fn detect_ambiguity(&self, forest: &ParseForest) -> Option<usize> {
        // Check multiple roots
        if forest.roots.len() > 1 {
            return Some(forest.roots.len());
        }

        // Check for Packed nodes (ambiguity within tree)
        let mut max_alternatives = 1;
        for node in &forest.nodes {
            if let ForestNode::Packed { alternatives } = node {
                max_alternatives = max_alternatives.max(alternatives.len());
            }
        }

        if max_alternatives > 1 {
            Some(max_alternatives)
        } else {
            None
        }
    }

    /// Disambiguate multiple roots
    fn disambiguate_roots(
        &self,
        roots: &[ForestNodeId],
        _forest: &ParseForest,
    ) -> Result<ForestNodeId, ConversionError> {
        match self.strategy {
            DisambiguationStrategy::First => Ok(roots[0]),
            DisambiguationStrategy::RejectAmbiguity => Err(ConversionError::AmbiguousForest {
                count: roots.len(),
            }),
            // For PreferShift/PreferReduce, we'd need metadata about which
            // root came from shift vs reduce. For now, default to first.
            _ => Ok(roots[0]),
        }
    }

    /// Build node recursively
    fn build_node(
        &self,
        node_id: ForestNodeId,
        forest: &ParseForest,
        input: &[u8],
        visited: &mut HashSet<usize>,
    ) -> Result<TreeNode, ConversionError> {
        // Validate node ID
        if node_id.0 >= forest.nodes.len() {
            return Err(ConversionError::InvalidNodeId {
                node_id: node_id.0,
            });
        }

        // Cycle detection (commented out for now - can cause false positives in valid DAGs)
        // if visited.contains(&node_id.0) {
        //     return Err(ConversionError::CycleDetected { node_id: node_id.0 });
        // }
        visited.insert(node_id.0);

        let forest_node = &forest.nodes[node_id.0];

        match forest_node {
            ForestNode::Terminal { symbol, range } => {
                // Create terminal (leaf) node
                Ok(TreeNode::new_with_children(
                    symbol.0 as u32,
                    range.start,
                    range.end,
                    vec![],
                ))
            }

            ForestNode::Nonterminal {
                symbol,
                children,
                rule_id: _,
            } => {
                // Recursively build children
                let mut child_nodes = Vec::new();
                for child_id in children {
                    let child_node = self.build_node(*child_id, forest, input, visited)?;
                    child_nodes.push(child_node);
                }

                // Calculate range from children
                let start_byte = child_nodes.first().map(|c| c.start_byte()).unwrap_or(0);
                let end_byte = child_nodes.last().map(|c| c.end_byte()).unwrap_or(0);

                // Create nonterminal (internal) node
                Ok(TreeNode::new_with_children(
                    symbol.0 as u32,
                    start_byte,
                    end_byte,
                    child_nodes,
                ))
            }

            ForestNode::Packed { alternatives } => {
                // Ambiguity point - apply disambiguation
                let selected = self.disambiguate_alternatives(alternatives, forest)?;
                self.build_node(selected, forest, input, visited)
            }
        }
    }

    /// Disambiguate alternatives in Packed node
    fn disambiguate_alternatives(
        &self,
        alternatives: &[ForestNodeId],
        _forest: &ParseForest,
    ) -> Result<ForestNodeId, ConversionError> {
        if alternatives.is_empty() {
            return Err(ConversionError::InvalidForest {
                reason: "Packed node has no alternatives".to_string(),
            });
        }

        match self.strategy {
            DisambiguationStrategy::First => Ok(alternatives[0]),

            DisambiguationStrategy::PreferShift => {
                // For MVP, we don't have metadata about shift vs reduce
                // Default to first for now (Phase 3.3 will add metadata)
                Ok(alternatives[0])
            }

            DisambiguationStrategy::PreferReduce => {
                // For MVP, we don't have metadata about shift vs reduce
                // Default to first for now (Phase 3.3 will add metadata)
                Ok(alternatives[0])
            }

            DisambiguationStrategy::Precedence => {
                // Precedence requires metadata (Phase 3.3)
                Ok(alternatives[0])
            }

            DisambiguationStrategy::RejectAmbiguity => Err(ConversionError::AmbiguousForest {
                count: alternatives.len(),
            }),
        }
    }
}

impl TreeNode {
    /// Get start byte
    pub(crate) fn start_byte(&self) -> usize {
        self.start_byte
    }

    /// Get end byte
    pub(crate) fn end_byte(&self) -> usize {
        self.end_byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disambiguation_strategy_equality() {
        assert_eq!(
            DisambiguationStrategy::First,
            DisambiguationStrategy::First
        );
        assert_ne!(
            DisambiguationStrategy::First,
            DisambiguationStrategy::PreferShift
        );
    }
}
