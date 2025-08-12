//! GLR runtime for rust-sitter parsers with Tree-sitter API compatibility
//!
//! This crate provides a runtime that mimics Tree-sitter's API while using
//! GLR parsing internally to handle ambiguous grammars.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod error;
pub mod external_scanner;
pub mod language;
pub mod node;
pub mod parser;
pub mod tree;

// Re-exports for convenience
pub use error::{ParseError, ParseErrorKind};
pub use external_scanner::{ExternalScanner, ScanResult};
pub use language::Language;
pub use node::{Node, Point};
pub use parser::Parser;
pub use tree::Tree;

/// Input edit information for incremental parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputEdit {
    /// Byte offset where the edit starts
    pub start_byte: usize,
    /// Byte offset where the edit ended in the old text
    pub old_end_byte: usize,
    /// Byte offset where the edit ends in the new text
    pub new_end_byte: usize,
    /// Point where the edit starts
    pub start_position: Point,
    /// Point where the edit ended in the old text
    pub old_end_position: Point,
    /// Point where the edit ends in the new text
    pub new_end_position: Point,
}

/// Query system types (stub for now)
#[cfg(feature = "queries")]
#[cfg_attr(docsrs, doc(cfg(feature = "queries")))]
pub mod query {
    /// A compiled query
    pub struct Query;
    /// A query cursor for executing queries
    pub struct QueryCursor;
    /// A query match
    pub struct QueryMatch;
}