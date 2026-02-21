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
        Node::new(self, 0)
    }

    /// Get the root kind as a string.
    pub fn root_kind(&self) -> &str {
        let sym = self.core.root_kind();
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
///
/// Note: Current implementation represents the root node only, as parser_v4
/// does not expose detailed tree structure. Node metadata is inferred from
/// the tree's overall properties and position within the source.
#[derive(Debug, Clone)]
pub struct Node<'a> {
    tree: &'a Tree,
    /// Index/position within the tree (root = 0)
    index: usize,
    /// Cached byte range for this node (start, end)
    byte_range: Option<(usize, usize)>,
    /// Cached position range for this node
    position_range: Option<(Point, Point)>,
}

impl<'a> Node<'a> {
    /// Create a new node with computed metadata
    fn new(tree: &'a Tree, index: usize) -> Self {
        let (byte_range, position_range) = Self::compute_ranges(tree, index);
        Self {
            tree,
            index,
            byte_range,
            position_range,
        }
    }

    /// Compute byte and position ranges for this node based on its index
    #[allow(clippy::type_complexity)]
    fn compute_ranges(
        tree: &Tree,
        index: usize,
    ) -> (Option<(usize, usize)>, Option<(Point, Point)>) {
        if index == 0 {
            // Root node covers the entire source
            let byte_end = tree.core.source.len();
            let end_position = Self::byte_to_point(&tree.core.source, byte_end);
            (
                Some((0, byte_end)),
                Some((Point { row: 0, column: 0 }, end_position)),
            )
        } else {
            // Non-root nodes: In current implementation, no children are exposed
            // Return None to indicate this node doesn't have valid ranges
            (None, None)
        }
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
        if self.index == 0 {
            // Root node - return the actual root kind
            self.tree.root_kind()
        } else {
            // Non-root nodes are not exposed by current parser_v4 implementation
            "unknown"
        }
    }

    /// Get the start byte of this node.
    pub fn start_byte(&self) -> usize {
        self.byte_range.map(|(start, _)| start).unwrap_or(0)
    }

    /// Get the end byte of this node.
    pub fn end_byte(&self) -> usize {
        self.byte_range.map(|(_, end)| end).unwrap_or(0)
    }

    /// Get the start position of this node.
    pub fn start_position(&self) -> Point {
        self.position_range
            .map(|(start, _)| start)
            .unwrap_or_default()
    }

    /// Get the end position of this node.
    pub fn end_position(&self) -> Point {
        self.position_range.map(|(_, end)| end).unwrap_or_default()
    }

    /// Get the number of children.
    pub fn child_count(&self) -> usize {
        if self.index == 0 {
            // Root node: parser_v4 doesn't expose children, but we can infer
            // that a successful parse with content has at least structure
            if !self.tree.core.source.is_empty() && self.tree.error_count() == 0 {
                // Estimate: non-trivial content likely has some structure
                // This is a heuristic since actual children aren't exposed
                0 // Conservative: return 0 until full tree structure is available
            } else {
                0
            }
        } else {
            // Non-root nodes don't exist in current implementation
            0
        }
    }

    /// Get a child by index.
    pub fn child(&self, index: usize) -> Option<Node<'a>> {
        if index < self.child_count() {
            // Current implementation doesn't expose actual children
            // Return None to indicate child access is not available
            None
        } else {
            None
        }
    }

    /// Check if this node is an error node.
    pub fn is_error(&self) -> bool {
        if self.index == 0 {
            // Root node: check if the entire tree has errors
            self.tree.error_count() > 0
        } else {
            // Non-root nodes: no specific error information available
            false
        }
    }

    /// Check if this node is missing (was expected but not found).
    pub fn is_missing(&self) -> bool {
        if self.index == 0 {
            // Root node: check if parse failed completely (empty source with errors)
            self.tree.core.source.is_empty() && self.tree.error_count() > 0
        } else {
            // Non-root nodes: no specific missing information available
            false
        }
    }

    /// Get the byte range of this node.
    pub fn byte_range(&self) -> std::ops::Range<usize> {
        let (start, end) = self.byte_range.unwrap_or((0, 0));
        start..end
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
