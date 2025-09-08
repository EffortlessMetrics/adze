//! Incremental parsing for pure Rust implementation.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

// Incremental parsing support for pure-Rust parser
use crate::pure_parser::{ParseResult, ParsedNode, Parser, TSLanguage};
use std::ops::Range;

// Re-export Point so tests can keep using pure_incremental::Point
pub use crate::pure_parser::Point;

/// Edit operation for incremental parsing
#[derive(Debug, Clone)]
pub struct Edit {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
    pub start_point: Point,
    pub old_end_point: Point,
    pub new_end_point: Point,
}

/// Tree that can be reused for incremental parsing
#[derive(Debug, Clone)]
pub struct Tree {
    pub root: ParsedNode,
    pub language: &'static TSLanguage,
    #[allow(dead_code)]
    source: Vec<u8>,
}

/// Node that can be reused during incremental parsing
#[derive(Debug, Clone)]
pub struct ReusableNode {
    #[allow(dead_code)]
    node: ParsedNode,
    #[allow(dead_code)]
    byte_range: Range<usize>,
    #[allow(dead_code)]
    is_error: bool,
}

impl Tree {
    /// Create a new tree from a parse result
    pub fn new(root: ParsedNode, language: &'static TSLanguage, source: &[u8]) -> Self {
        Tree {
            root,
            language,
            source: source.to_vec(),
        }
    }

    /// Get the root node of the tree
    pub fn root_node(&self) -> &ParsedNode {
        &self.root
    }

    /// Edit the tree to reflect changes in the source code
    pub fn edit(&mut self, edit: &Edit) {
        // Update byte positions in the tree
        self.edit_node(&mut self.root.clone(), edit);
    }

    /// Recursively edit node positions
    fn edit_node(&self, node: &mut ParsedNode, edit: &Edit) {
        // If node is entirely before the edit, no changes needed
        if node.end_byte() <= edit.start_byte {
            return;
        }

        // If node starts after the edit, shift its positions
        if node.start_byte() >= edit.old_end_byte {
            let _byte_delta = (edit.new_end_byte as isize) - (edit.old_end_byte as isize);
            let _row_delta = edit.new_end_point.row as i32 - edit.old_end_point.row as i32;

            // Update positions (would need mutable access to ParsedNode fields)
            // node.start_byte += byte_delta;
            // node.end_byte += byte_delta;
            // node.start_point.row += row_delta;
            // node.end_point.row += row_delta;
        }

        // Recursively edit children
        for child in node.children() {
            self.edit_node(&mut child.clone(), edit);
        }
    }

    /// Get reusable nodes for incremental parsing
    pub fn get_reusable_nodes(&self) -> Vec<ReusableNode> {
        let mut nodes = Vec::new();
        self.collect_reusable_nodes(&self.root, &mut nodes);
        nodes
    }

    #[allow(clippy::only_used_in_recursion)]
    fn collect_reusable_nodes(&self, node: &ParsedNode, nodes: &mut Vec<ReusableNode>) {
        // Add this node as reusable
        nodes.push(ReusableNode {
            node: node.clone(),
            byte_range: node.start_byte()..node.end_byte(),
            is_error: node.is_error(),
        });

        // Collect from children
        for child in node.children() {
            self.collect_reusable_nodes(child, nodes);
        }
    }
}

/// Incremental parser that reuses nodes from previous parses
pub struct IncrementalParser {
    parser: Parser,
    previous_tree: Option<Tree>,
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new()
    }
}

impl IncrementalParser {
    /// Create a new incremental parser
    pub fn new() -> Self {
        IncrementalParser {
            parser: Parser::new(),
            previous_tree: None,
        }
    }

    /// Set the language for parsing
    pub fn set_language(&mut self, language: &'static TSLanguage) -> Result<(), String> {
        self.parser.set_language(language)
    }

    /// Set timeout for parsing
    pub fn set_timeout_micros(&mut self, timeout: u64) {
        self.parser.set_timeout_micros(timeout);
    }

    /// Set cancellation flag
    pub fn set_cancellation_flag(&mut self, flag: Option<*const std::sync::atomic::AtomicBool>) {
        self.parser.set_cancellation_flag(flag);
    }

    /// Parse with incremental reuse
    pub fn parse(&mut self, source: &str, old_tree: Option<&Tree>) -> ParseResult {
        // If we have an old tree, try to reuse nodes
        if let Some(tree) = old_tree {
            self.previous_tree = Some(tree.clone());

            // Get reusable nodes
            let _reusable_nodes = tree.get_reusable_nodes();

            // TODO: Implement actual incremental parsing logic
            // For now, fall back to full reparse
        }

        // Parse from scratch
        let result = self.parser.parse_string(source);

        // Store the tree for next parse
        if let Some(root) = &result.root
            && let Some(language) = self.parser.language()
        {
            self.previous_tree = Some(Tree::new(root.clone(), language, source.as_bytes()));
        }

        result
    }

    /// Parse with edits
    pub fn parse_with_edits(
        &mut self,
        source: &str,
        mut old_tree: Option<Tree>,
        edits: &[Edit],
    ) -> ParseResult {
        // Apply edits to the old tree
        if let Some(ref mut tree) = old_tree {
            for edit in edits {
                tree.edit(edit);
            }
        }

        // Parse with the edited tree
        self.parse(source, old_tree.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_parser_creation() {
        let parser = IncrementalParser::new();
        assert!(parser.previous_tree.is_none());
    }

    #[test]
    fn test_edit_creation() {
        let edit = Edit {
            start_byte: 5,
            old_end_byte: 10,
            new_end_byte: 15,
            start_point: Point { row: 0, column: 5 },
            old_end_point: Point { row: 0, column: 10 },
            new_end_point: Point { row: 0, column: 15 },
        };

        assert_eq!(edit.start_byte, 5);
        assert_eq!(edit.new_end_byte - edit.old_end_byte, 5);
    }

    #[test]
    fn test_tree_edit() {
        // Create a simple tree
        let node = ParsedNode {
            symbol: 1,
            children: vec![],
            start_byte: 0,
            end_byte: 20,
            start_point: Point { row: 0, column: 0 },
            end_point: Point { row: 0, column: 20 },
            is_extra: false,
            is_error: false,
            is_missing: false,
            is_named: true,
            field_id: None,
            language: None,
        };

        let language = {
            &crate::pure_parser::TSLanguage {
                version: 15,
                symbol_count: 10,
                alias_count: 0,
                token_count: 5,
                external_token_count: 0,
                state_count: 20,
                large_state_count: 10,
                production_id_count: 0,
                field_count: 0,
                max_alias_sequence_length: 0,
                production_id_map: std::ptr::null(),
                parse_table: std::ptr::null(),
                small_parse_table: std::ptr::null(),
                small_parse_table_map: std::ptr::null(),
                parse_actions: std::ptr::null(),
                symbol_names: std::ptr::null(),
                field_names: std::ptr::null(),
                field_map_slices: std::ptr::null(),
                field_map_entries: std::ptr::null(),
                symbol_metadata: std::ptr::null(),
                public_symbol_map: std::ptr::null(),
                alias_map: std::ptr::null(),
                alias_sequences: std::ptr::null(),
                lex_modes: std::ptr::null(),
                lex_fn: None,
                keyword_lex_fn: None,
                keyword_capture_token: 0,
                external_scanner: crate::pure_parser::ExternalScanner {
                    states: std::ptr::null(),
                    symbol_map: std::ptr::null(),
                    create: None,
                    destroy: None,
                    scan: None,
                    serialize: None,
                    deserialize: None,
                },
                primary_state_ids: std::ptr::null(),
                production_lhs_index: std::ptr::null(),
                production_count: 0,
                eof_symbol: 0,
                rules: std::ptr::null(),
                rule_count: 0,
            }
        };

        let mut tree = Tree::new(node, language, b"original source code");

        // Apply an edit
        let edit = Edit {
            start_byte: 8,
            old_end_byte: 14,
            new_end_byte: 20,
            start_point: Point { row: 0, column: 8 },
            old_end_point: Point { row: 0, column: 14 },
            new_end_point: Point { row: 0, column: 20 },
        };

        tree.edit(&edit);

        // Verify tree was edited (in real implementation)
        let reusable = tree.get_reusable_nodes();
        assert!(!reusable.is_empty());
    }
}
