//! Minimal Tree-sitter compatibility shims (edits, points, language wrapper).
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

//! Tree-sitter compatibility API
//!
//! This module provides a compatibility layer that mimics the Tree-sitter API,
//! allowing existing Tree-sitter code to work with rust-sitter with minimal changes.

use crate::parser_v4::{Parser as CoreParser, Tree as CoreTree};
use crate::pure_incremental::Edit as CoreEdit;
use crate::pure_parser;
use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::Grammar;
use std::sync::Arc;

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
#[derive(Clone)]
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
    pub fn parse(&mut self, source: &str, old: Option<&Tree>) -> Option<Tree> {
        let core_parser = self.core.as_mut()?;
        let lang = self.lang.as_ref()?;

        match old {
            #[cfg(feature = "incremental_glr")]
            Some(old_tree) if old_tree.last_edit.is_some() => {
                // Try incremental parsing
                if let Some(edit) = &old_tree.last_edit {
                    // TODO: Implement incremental parsing in v4
                    // For now, always fall back to fresh parse
                    let _ = edit; // Suppress unused warning
                }
                // Fall back to fresh parse
                match core_parser.parse(source) {
                    Ok(t) => Some(Tree {
                        core: t,
                        last_edit: None,
                        language: lang.clone(),
                    }),
                    Err(_) => None,
                }
            }
            _ => {
                // Fresh parse
                match core_parser.parse(source) {
                    Ok(t) => Some(Tree {
                        core: t,
                        last_edit: None,
                        language: lang.clone(),
                    }),
                    Err(_) => None,
                }
            }
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
#[derive(Clone)]
pub struct Tree {
    pub(crate) core: CoreTree,
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
        Node {
            tree: &self.core,
            _index: 0,
        }
    }

    /// Get the root kind as a string.
    pub fn root_kind(&self) -> &str {
        let sym = self.core.root_kind();
        // Try direct rule name mapping first
        if let Some(name) = self
            .language
            .grammar
            .rule_names
            .get(&rust_sitter_ir::SymbolId(sym))
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
pub struct Node<'a> {
    tree: &'a CoreTree,
    _index: usize,
}

impl<'a> Node<'a> {
    /// Get the kind of this node as a string.
    pub fn kind(&self) -> &str {
        // TODO: Implement actual node kind lookup
        "node"
    }

    /// Get the start byte of this node.
    pub fn start_byte(&self) -> usize {
        // TODO: Implement actual position lookup
        0
    }

    /// Get the end byte of this node.
    pub fn end_byte(&self) -> usize {
        // TODO: Implement actual position lookup
        0
    }

    /// Get the start position of this node.
    pub fn start_position(&self) -> Point {
        Point { row: 0, column: 0 }
    }

    /// Get the end position of this node.
    pub fn end_position(&self) -> Point {
        Point { row: 0, column: 0 }
    }

    /// Get the number of children.
    pub fn child_count(&self) -> usize {
        // TODO: Implement actual child count
        0
    }

    /// Get a child by index.
    pub fn child(&self, index: usize) -> Option<Node<'a>> {
        if index < self.child_count() {
            Some(Node {
                tree: self.tree,
                _index: index + 1,
            })
        } else {
            None
        }
    }

    /// Check if this node is an error node.
    pub fn is_error(&self) -> bool {
        // TODO: Implement actual error check
        false
    }

    /// Check if this node is missing (was expected but not found).
    pub fn is_missing(&self) -> bool {
        // TODO: Implement actual missing check
        false
    }
}
