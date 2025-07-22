// Incremental parsing support for the pure-Rust Tree-sitter runtime
// This module provides efficient reparsing of edited documents

use crate::parser_v2::{ParseNode, ParserV2};
use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::{Grammar, SymbolId};
use std::ops::Range;

/// Represents an edit to a document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    /// The byte range that was replaced
    pub old_range: Range<usize>,
    /// The new length after replacement
    pub new_length: usize,
}

impl Edit {
    /// Create a new edit
    pub fn new(start: usize, old_end: usize, new_end: usize) -> Self {
        Edit {
            old_range: start..old_end,
            new_length: new_end - start,
        }
    }

    /// Get the change in length
    pub fn delta(&self) -> isize {
        self.new_length as isize - self.old_range.len() as isize
    }
}

/// A parse tree that can be incrementally updated
#[derive(Debug, Clone)]
pub struct IncrementalTree {
    /// The root node of the parse tree
    pub root: ParseNode,
    /// The input text that was parsed
    pub text: String,
    /// Byte ranges of nodes for efficient edit mapping
    node_ranges: Vec<(usize, Range<usize>)>, // (node_id, byte_range)
}

impl IncrementalTree {
    /// Create a new incremental tree
    pub fn new(root: ParseNode, text: String) -> Self {
        let mut tree = IncrementalTree {
            root,
            text,
            node_ranges: Vec::new(),
        };
        tree.compute_node_ranges();
        tree
    }

    /// Compute byte ranges for all nodes in the tree
    fn compute_node_ranges(&mut self) {
        self.node_ranges.clear();
        self.collect_ranges(&self.root, 0);
    }

    /// Recursively collect node ranges
    fn collect_ranges(&mut self, node: &ParseNode, mut offset: usize) -> usize {
        let node_id = self.node_ranges.len();
        let start = offset;
        
        if node.children.is_empty() {
            // Leaf node - use token length
            let len = node.symbol.0; // Simplified - would need actual token length
            self.node_ranges.push((node_id, start..start + len));
            offset + len
        } else {
            // Internal node - sum of children
            for child in &node.children {
                offset = self.collect_ranges(child, offset);
            }
            self.node_ranges.push((node_id, start..offset));
            offset
        }
    }

    /// Apply an edit to the tree and return affected nodes
    pub fn apply_edit(&mut self, edit: &Edit) -> Vec<usize> {
        // Find all nodes that intersect with the edit range
        let mut affected = Vec::new();
        
        for (node_id, range) in &self.node_ranges {
            if range.end > edit.old_range.start && range.start < edit.old_range.end {
                affected.push(*node_id);
            }
        }

        // Update text
        let prefix = &self.text[..edit.old_range.start];
        let suffix = &self.text[edit.old_range.end..];
        let new_text = self.text[edit.old_range.start..edit.old_range.start + edit.new_length].to_string();
        self.text = format!("{}{}{}", prefix, new_text, suffix);

        // Adjust node ranges after the edit
        let delta = edit.delta();
        if delta != 0 {
            for (_, range) in &mut self.node_ranges {
                if range.start >= edit.old_range.end {
                    range.start = (range.start as isize + delta) as usize;
                    range.end = (range.end as isize + delta) as usize;
                } else if range.end > edit.old_range.end {
                    range.end = (range.end as isize + delta) as usize;
                }
            }
        }

        affected
    }
}

/// Incremental parser that reuses unchanged subtrees
pub struct IncrementalParser {
    parser: ParserV2,
}

impl IncrementalParser {
    /// Create a new incremental parser
    pub fn new(grammar: Grammar, table: ParseTable) -> Self {
        IncrementalParser {
            parser: ParserV2::new(grammar, table),
        }
    }

    /// Parse with an optional old tree to reuse
    pub fn parse_incremental(
        &mut self,
        tokens: &[crate::parser_v2::Token],
        old_tree: Option<&IncrementalTree>,
        edits: &[Edit],
    ) -> Result<IncrementalTree, crate::parser_v2::ParseError> {
        if let Some(old_tree) = old_tree {
            // Try to reuse parts of the old tree
            self.parse_with_reuse(tokens, old_tree, edits)
        } else {
            // No old tree - parse from scratch
            let root = self.parser.parse(tokens)?;
            let text = tokens.iter()
                .map(|t| &t.text[..])
                .collect::<Vec<_>>()
                .join("");
            Ok(IncrementalTree::new(root, text))
        }
    }

    /// Parse while trying to reuse unchanged subtrees
    fn parse_with_reuse(
        &mut self,
        tokens: &[crate::parser_v2::Token],
        old_tree: &IncrementalTree,
        edits: &[Edit],
    ) -> Result<IncrementalTree, crate::parser_v2::ParseError> {
        // For now, implement a simple strategy:
        // If edits are small and localized, try to reuse unaffected subtrees
        
        let total_edit_size: usize = edits.iter()
            .map(|e| e.old_range.len() + e.new_length)
            .sum();
        
        let text_size = old_tree.text.len();
        
        // If edits affect less than 10% of the text, try incremental parsing
        if total_edit_size < text_size / 10 {
            // TODO: Implement actual subtree reuse logic
            // For now, fall back to full reparse
        }

        // Full reparse
        let root = self.parser.parse(tokens)?;
        let text = tokens.iter()
            .map(|t| &t.text[..])
            .collect::<Vec<_>>()
            .join("");
        Ok(IncrementalTree::new(root, text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser_v2::Token;

    #[test]
    fn test_edit_application() {
        // Create a simple tree
        let root = ParseNode {
            symbol: SymbolId(0),
            children: vec![
                ParseNode {
                    symbol: SymbolId(1),
                    children: vec![],
                },
                ParseNode {
                    symbol: SymbolId(2),
                    children: vec![],
                },
            ],
        };

        let mut tree = IncrementalTree::new(root, "hello world".to_string());
        
        // Replace "hello" with "hi"
        let edit = Edit::new(0, 5, 2);
        let affected = tree.apply_edit(&edit);
        
        assert_eq!(tree.text, "hi world");
        assert!(!affected.is_empty());
    }

    #[test]
    fn test_edit_delta() {
        // Insertion
        let edit = Edit::new(5, 5, 8);
        assert_eq!(edit.delta(), 3);

        // Deletion
        let edit = Edit::new(5, 10, 5);
        assert_eq!(edit.delta(), -5);

        // Replacement (same size)
        let edit = Edit::new(5, 10, 10);
        assert_eq!(edit.delta(), 0);
    }

    #[test]
    fn test_affected_nodes() {
        let root = ParseNode {
            symbol: SymbolId(0),
            children: vec![
                ParseNode {
                    symbol: SymbolId(1),
                    children: vec![],
                },
                ParseNode {
                    symbol: SymbolId(2),
                    children: vec![],
                },
                ParseNode {
                    symbol: SymbolId(3),
                    children: vec![],
                },
            ],
        };

        let mut tree = IncrementalTree::new(root, "abcdefghijkl".to_string());
        
        // Edit in the middle
        let edit = Edit::new(4, 8, 6);
        let affected = tree.apply_edit(&edit);
        
        // Should affect middle nodes
        assert!(affected.len() > 0);
    }
}