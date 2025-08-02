// Incremental parsing support for the pure-Rust Tree-sitter runtime
// This module provides efficient reparsing of edited documents

use crate::parser_v2::{ParseNode, ParserV2};
use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::Grammar;
use std::ops::Range;

/// Represents an edit to a document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    /// The byte range that was replaced
    pub old_range: Range<usize>,
    /// The new text that replaces the old range
    pub new_text: String,
}

impl Edit {
    /// Create a new edit
    pub fn new(start: usize, old_end: usize, new_text: String) -> Self {
        Edit {
            old_range: start..old_end,
            new_text,
        }
    }

    /// Create an edit with text
    pub fn with_text(old_range: Range<usize>, new_text: String) -> Self {
        Edit {
            old_range,
            new_text,
        }
    }

    /// Get the new length after replacement
    pub fn new_length(&self) -> usize {
        self.new_text.len()
    }

    /// Get the change in length
    pub fn delta(&self) -> isize {
        self.new_text.len() as isize - self.old_range.len() as isize
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
        let root_clone = self.root.clone();
        self.collect_ranges(&root_clone, 0);
    }

    /// Recursively collect node ranges
    fn collect_ranges(&mut self, node: &ParseNode, mut offset: usize) -> usize {
        let node_id = self.node_ranges.len();
        let start = offset;

        if node.children.is_empty() {
            // Leaf node - use token length
            let len = node.symbol.0 as usize; // Simplified - would need actual token length
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

        // Update text with the actual new text from the edit
        let prefix = &self.text[..edit.old_range.start];
        let suffix = &self.text[edit.old_range.end..];
        self.text = format!("{}{}{}", prefix, &edit.new_text, suffix);

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
            let root = self.parser.parse(tokens.to_vec())?;
            let text = tokens
                .iter()
                .map(|t| t.text.clone())
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                .collect::<String>();
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

        let total_edit_size: usize = edits
            .iter()
            .map(|e| e.old_range.len() + e.new_length())
            .sum();

        let text_size = old_tree.text.len();

        // If edits affect less than 10% of the text, try incremental parsing
        if total_edit_size < text_size / 10 {
            // TODO: Implement actual subtree reuse logic
            // For now, fall back to full reparse
        }

        // Full reparse
        let root = self.parser.parse(tokens.to_vec())?;
        let text = tokens
            .iter()
            .map(|t| t.text.clone())
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .collect::<String>();
        Ok(IncrementalTree::new(root, text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_ir::SymbolId;

    #[test]
    fn test_edit_application() {
        // Create a simple tree
        let root = ParseNode {
            symbol: SymbolId(0),
            rule_id: None,
            children: vec![
                ParseNode {
                    symbol: SymbolId(1),
                    rule_id: None,
                    children: vec![],
                    start_byte: 0,
                    end_byte: 5,
                    text: Some(b"hello".to_vec()),
                },
                ParseNode {
                    symbol: SymbolId(2),
                    rule_id: None,
                    children: vec![],
                    start_byte: 6,
                    end_byte: 11,
                    text: Some(b"world".to_vec()),
                },
            ],
            start_byte: 0,
            end_byte: 11,
            text: None,
        };

        let mut tree = IncrementalTree::new(root, "hello world".to_string());

        // Replace "hello" with "hi"
        let edit = Edit::new(0, 5, "hi".to_string());
        let affected = tree.apply_edit(&edit);

        assert_eq!(tree.text, "hi world");
        assert!(!affected.is_empty());
    }

    #[test]
    fn test_edit_delta() {
        // Insertion
        let edit = Edit::new(5, 5, "abc".to_string());
        assert_eq!(edit.delta(), 3);

        // Deletion
        let edit = Edit::new(5, 10, "".to_string());
        assert_eq!(edit.delta(), -5);

        // Replacement (same size)
        let edit = Edit::new(5, 10, "hello".to_string());
        assert_eq!(edit.delta(), 0);
    }

    #[test]
    fn test_affected_nodes() {
        let root = ParseNode {
            symbol: SymbolId(0),
            rule_id: None,
            children: vec![
                ParseNode {
                    symbol: SymbolId(1),
                    rule_id: None,
                    children: vec![],
                    start_byte: 0,
                    end_byte: 4,
                    text: Some(b"abcd".to_vec()),
                },
                ParseNode {
                    symbol: SymbolId(2),
                    rule_id: None,
                    children: vec![],
                    start_byte: 4,
                    end_byte: 8,
                    text: Some(b"efgh".to_vec()),
                },
                ParseNode {
                    symbol: SymbolId(3),
                    rule_id: None,
                    children: vec![],
                    start_byte: 8,
                    end_byte: 12,
                    text: Some(b"ijkl".to_vec()),
                },
            ],
            start_byte: 0,
            end_byte: 12,
            text: None,
        };

        let mut tree = IncrementalTree::new(root, "abcdefghijkl".to_string());

        // Edit in the middle
        let edit = Edit::new(4, 8, "XY".to_string());
        let affected = tree.apply_edit(&edit);

        // Should affect middle nodes
        assert!(affected.len() > 0);
    }
}
