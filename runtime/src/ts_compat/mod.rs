//! Minimal Tree-sitter compatibility shims (edits, points, language wrapper).
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

//! Tree-sitter compatibility API
//!
//! This module provides a compatibility layer that mimics the Tree-sitter API,
//! allowing existing Tree-sitter code to work with adze with minimal changes.

use crate::parser_v4::{ParseNode, Parser as CoreParser};
use crate::pure_incremental::Edit as CoreEdit;
use crate::pure_parser;
use adze_glr_core::ParseTable;
use adze_ir::Grammar;
use std::sync::Arc;

/// An owned tree representation for ts_compat layer.
/// This provides the interface expected by ts_compat::Tree without lifetime constraints.
#[derive(Clone, Debug)]
pub(crate) struct OwnedCoreTree {
    /// The root parse node
    pub root: ParseNode,
    /// Source text that was parsed
    pub source: Vec<u8>,
    /// Number of parse errors
    pub error_count: usize,
}

impl OwnedCoreTree {
    /// Get the root symbol ID
    pub(crate) fn root_kind(&self) -> u16 {
        self.root.symbol.0
    }

    /// Get the error count
    pub(crate) fn error_count(&self) -> usize {
        self.error_count
    }
}

/// A position in a document, identified by row and column.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Point {
    pub row: u32,
    pub column: u32,
}

impl From<(u32, u32)> for Point {
    fn from((row, column): (u32, u32)) -> Self {
        Point { row, column }
    }
}

impl From<Point> for (u32, u32) {
    fn from(p: Point) -> Self {
        (p.row, p.column)
    }
}

/// An edit to a document.
#[derive(Clone, Debug, Default)]
pub struct InputEdit {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
    pub start_position: Point,
    pub old_end_position: Point,
    pub new_end_position: Point,
}

impl From<InputEdit> for CoreEdit {
    fn from(e: InputEdit) -> Self {
        CoreEdit {
            start_byte: e.start_byte,
            old_end_byte: e.old_end_byte,
            new_end_byte: e.new_end_byte,
            start_point: pure_parser::Point {
                row: e.start_position.row,
                column: e.start_position.column,
            },
            old_end_point: pure_parser::Point {
                row: e.old_end_position.row,
                column: e.old_end_position.column,
            },
            new_end_point: pure_parser::Point {
                row: e.new_end_position.row,
                column: e.new_end_position.column,
            },
        }
    }
}

/// A language definition containing grammar and parse tables.
#[derive(Clone, Debug)]
pub struct Language {
    pub name: String,
    pub grammar: Grammar,
    pub table: ParseTable,
}

impl Language {
    pub fn new(name: impl Into<String>, grammar: Grammar, table: ParseTable) -> Self {
        Self {
            name: name.into(),
            grammar,
            table,
        }
    }
}

/// A parser that can parse source code using a language.
pub struct Parser {
    core: Option<CoreParser>,
    lang: Option<Arc<Language>>,
}

impl Parser {
    /// Create a new parser.
    pub fn new() -> Self {
        Self {
            core: None,
            lang: None,
        }
    }

    /// Set the language for this parser.
    pub fn set_language(&mut self, lang: Arc<Language>) -> Result<(), String> {
        self.lang = Some(Arc::clone(&lang));
        self.core = Some(CoreParser::new(
            lang.grammar.clone(),
            lang.table.clone(),
            lang.name.clone(),
        ));
        Ok(())
    }

    /// Parse source code, optionally reusing an old tree for incremental parsing.
    ///
    /// Note: Incremental parsing is currently disabled and falls back to fresh parsing
    /// for consistency. The `old` parameter is accepted for API compatibility but ignored.
    pub fn parse(&mut self, source: &str, _old: Option<&Tree>) -> Option<Tree> {
        let core_parser = self.core.as_mut()?;
        let lang = self.lang.as_ref()?;

        // Use parse_tree() which returns an owned ParseNode
        match core_parser.parse_tree(source) {
            Ok(root) => Some(Tree {
                core: OwnedCoreTree {
                    root,
                    source: source.as_bytes().to_vec(),
                    error_count: 0, // TODO: track error count properly
                },
                last_edit: None,
                language: lang.clone(),
            }),
            Err(_) => None,
        }
    }

    /// Get the current language.
    pub fn language(&self) -> Option<&Arc<Language>> {
        self.lang.as_ref()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

/// A parsed syntax tree.
#[derive(Clone, Debug)]
pub struct Tree {
    pub(crate) core: OwnedCoreTree,
    pub(crate) last_edit: Option<CoreEdit>,
    pub(crate) language: Arc<Language>,
}

impl Tree {
    /// Apply an edit to this tree.
    pub fn edit(&mut self, edit: &InputEdit) {
        let core_edit = CoreEdit::from(edit.clone());
        // Store the edit for later incremental parsing
        // Note: parser_v4::Tree doesn't have apply_edit, edits are tracked separately
        self.last_edit = Some(core_edit);
    }

    /// Get the root node of this tree.
    pub fn root_node(&self) -> Node<'_> {
        Node::new(self, &self.core.root)
    }

    /// Get the root kind as a string.
    pub fn root_kind(&self) -> &str {
        self.kind_for_symbol(self.core.root_kind())
    }

    fn kind_for_symbol(&self, sym: u16) -> &str {
        // Try direct rule name mapping first
        if let Some(name) = self
            .language
            .grammar
            .rule_names
            .get(&adze_ir::SymbolId(sym))
        {
            return name.as_str();
        }
        // Fallback: if index_to_symbol is populated, prefer that
        if let Some(name) = self
            .language
            .table
            .index_to_symbol
            .get(sym as usize)
            .and_then(|sid| self.language.grammar.rule_names.get(sid))
        {
            return name.as_str();
        }
        "unknown"
    }

    /// Get the number of errors in this tree.
    pub fn error_count(&self) -> usize {
        self.core.error_count()
    }

    /// Check if the tree has errors.
    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }
}

/// A node in a syntax tree.
#[derive(Debug, Clone)]
pub struct Node<'a> {
    tree: &'a Tree,
    node: &'a ParseNode,
}

impl<'a> Node<'a> {
    fn new(tree: &'a Tree, node: &'a ParseNode) -> Self {
        Self { tree, node }
    }

    /// Convert byte position to Point (row, column)
    fn byte_to_point(source: &[u8], byte_pos: usize) -> Point {
        let mut row = 0;
        let mut column = 0;

        for (i, &byte) in source.iter().enumerate() {
            if i >= byte_pos {
                break;
            }
            if byte == b'\n' {
                row += 1;
                column = 0;
            } else {
                column += 1;
            }
        }

        Point { row, column }
    }

    /// Get the kind of this node as a string.
    pub fn kind(&self) -> &str {
        self.tree.kind_for_symbol(self.node.symbol.0)
    }

    /// Get the start byte of this node.
    pub fn start_byte(&self) -> usize {
        self.node.start_byte
    }

    /// Get the end byte of this node.
    pub fn end_byte(&self) -> usize {
        self.node.end_byte
    }

    /// Get the start position of this node.
    pub fn start_position(&self) -> Point {
        Self::byte_to_point(&self.tree.core.source, self.node.start_byte)
    }

    /// Get the end position of this node.
    pub fn end_position(&self) -> Point {
        Self::byte_to_point(&self.tree.core.source, self.node.end_byte)
    }

    /// Get the number of children.
    pub fn child_count(&self) -> usize {
        self.node.children.len()
    }

    /// Get a child by index.
    pub fn child(&self, index: usize) -> Option<Node<'a>> {
        self.node
            .children
            .get(index)
            .map(|child| Node::new(self.tree, child))
    }

    /// Check if this node is a named grammar node.
    pub fn is_named(&self) -> bool {
        self.tree
            .language
            .table
            .symbol_metadata
            .get(self.node.symbol.0 as usize)
            .map(|metadata| metadata.is_named)
            .unwrap_or_else(|| {
                !self
                    .tree
                    .language
                    .grammar
                    .tokens
                    .contains_key(&self.node.symbol)
            })
    }

    /// Get the number of named children.
    pub fn named_child_count(&self) -> usize {
        self.node
            .children
            .iter()
            .filter(|child| Node::new(self.tree, child).is_named())
            .count()
    }

    /// Get a named child by named-child index.
    pub fn named_child(&self, index: usize) -> Option<Node<'a>> {
        self.node
            .children
            .iter()
            .filter(|child| Node::new(self.tree, child).is_named())
            .nth(index)
            .map(|child| Node::new(self.tree, child))
    }

    /// Create a cursor rooted at this node.
    pub fn walk(&self) -> TreeCursor<'a> {
        TreeCursor::new(self.tree, self.node)
    }

    /// Get the field name attached to this node's edge from its parent.
    pub fn field_name(&self) -> Option<&str> {
        self.node.field_name.as_deref()
    }

    /// Get the field name for a child edge by child index.
    pub fn field_name_for_child(&self, index: usize) -> Option<&str> {
        self.node
            .children
            .get(index)
            .and_then(|child| child.field_name.as_deref())
    }

    /// Get the first child attached through the given field name.
    pub fn child_by_field_name(&self, field_name: &str) -> Option<Node<'a>> {
        self.node
            .children
            .iter()
            .find(|child| child.field_name.as_deref() == Some(field_name))
            .map(|child| Node::new(self.tree, child))
    }

    /// Check if this node is an error node.
    pub fn is_error(&self) -> bool {
        (self.node.symbol.0 == 0 && self.node.children.is_empty()) || self.tree.error_count() > 0
    }

    /// Check if this node is missing (was expected but not found).
    pub fn is_missing(&self) -> bool {
        self.node.start_byte == self.node.end_byte && self.is_error()
    }

    /// Get the byte range of this node.
    pub fn byte_range(&self) -> std::ops::Range<usize> {
        self.node.start_byte..self.node.end_byte
    }

    /// Get the text content of this node.
    pub fn utf8_text<'b>(&self, source: &'b [u8]) -> Result<&'b str, std::str::Utf8Error> {
        let range = self.byte_range();
        let slice = source.get(range).unwrap_or(&[]);
        std::str::from_utf8(slice)
    }

    /// Get the text content of this node as a string.
    pub fn text(&self, source: &[u8]) -> String {
        self.utf8_text(source).unwrap_or("").to_string()
    }
}

#[derive(Debug, Clone)]
struct CursorFrame<'a> {
    node: &'a ParseNode,
    child_index: usize,
}

/// A cursor for walking a syntax tree without allocating child vectors.
#[derive(Debug, Clone)]
pub struct TreeCursor<'a> {
    tree: &'a Tree,
    current: &'a ParseNode,
    parents: Vec<CursorFrame<'a>>,
}

impl<'a> TreeCursor<'a> {
    fn new(tree: &'a Tree, current: &'a ParseNode) -> Self {
        Self {
            tree,
            current,
            parents: Vec::new(),
        }
    }

    /// Get the cursor's current node.
    pub fn node(&self) -> Node<'a> {
        Node::new(self.tree, self.current)
    }

    /// Move to the first child of the current node.
    pub fn goto_first_child(&mut self) -> bool {
        let Some(child) = self.current.children.first() else {
            return false;
        };

        self.parents.push(CursorFrame {
            node: self.current,
            child_index: 0,
        });
        self.current = child;
        true
    }

    /// Move to the next sibling of the current node.
    pub fn goto_next_sibling(&mut self) -> bool {
        let Some(parent) = self.parents.last_mut() else {
            return false;
        };

        let next_index = parent.child_index + 1;
        let Some(next) = parent.node.children.get(next_index) else {
            return false;
        };

        parent.child_index = next_index;
        self.current = next;
        true
    }

    /// Move to the parent of the current node.
    pub fn goto_parent(&mut self) -> bool {
        let Some(parent) = self.parents.pop() else {
            return false;
        };

        self.current = parent.node;
        true
    }

    /// Get the field name attached to the current node's parent edge.
    pub fn field_name(&self) -> Option<&str> {
        self.current.field_name.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adze_glr_core::Action;
    use adze_ir::SymbolId;
    use std::collections::BTreeMap;

    fn empty_parse_table_language() -> Arc<Language> {
        Arc::new(Language::new(
            "ts_compat_empty_parse_table",
            Grammar::default(),
            ParseTable::default(),
        ))
    }

    fn accept_on_eof_language() -> Arc<Language> {
        let parse_table = ParseTable {
            symbol_to_index: BTreeMap::from([(SymbolId(0), 0)]),
            action_table: vec![vec![vec![Action::Accept]]],
            ..Default::default()
        };

        Arc::new(Language::new(
            "ts_compat_accept_on_eof",
            Grammar::default(),
            parse_table,
        ))
    }

    #[test]
    fn parse_ignores_old_tree_source() {
        let mut parser = Parser::new();
        parser.set_language(empty_parse_table_language()).unwrap();

        let old_tree = parser.parse("old", None).unwrap();
        let new_source = "incrementally updated";

        let reparsed = parser.parse(new_source, Some(&old_tree)).unwrap();
        assert_eq!(reparsed.core.source, new_source.as_bytes().to_vec());
        assert_ne!(reparsed.core.source, old_tree.core.source);
    }

    #[test]
    fn parse_returns_none_on_core_parse_error() {
        let mut parser = Parser::new();
        parser.set_language(accept_on_eof_language()).unwrap();

        let tree = parser.parse("any input", None);

        assert!(tree.is_none());
    }
}
