// Runtime crate needs unsafe for FFI bindings and performance-critical operations
#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(private_interfaces)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]
#![allow(clippy::missing_safety_doc)] // Many FFI functions - safety documented at module level
#![allow(clippy::needless_range_loop)] // Sometimes clearer than iterators
#![allow(clippy::only_used_in_recursion)] // Recursive algorithms in parsers

//! rust-sitter runtime library for Tree-sitter parsing

/// Private implementation details exposed for macro use only.
pub mod __private;
/// Concurrency caps for thread pools and parallel operations
pub mod concurrency_caps;
/// External scanner interface for custom tokenization.
pub mod external_scanner;
/// FFI bindings for external scanners.
pub mod external_scanner_ffi;
/// FFI bindings and types for Tree-sitter compatibility.
pub mod ffi;
/// Field mapping support for parse trees.
pub mod field_tree;
/// Lexer abstraction for TokenSource trait
pub mod lex;
/// Line and column position tracking utilities.
pub mod linecol;
/// Memory pool for efficient allocation and reuse
pub mod pool;
/// Tree-sitter format constants and helpers
pub mod ts_format;

// Stable, documented entry points for public API
// These re-exports are guaranteed stable across minor versions
pub use ffi::TSSymbol;
/// Type alias for symbol identifiers.
pub type SymbolId = TSSymbol;

// Stable re-exports for core functionality
// Note: ts_compat is already declared below as a module, not a re-export

#[cfg(feature = "pure-rust")]
pub use glr_incremental::{Edit, GLRToken, IncrementalGLRParser};

// Additional stable re-exports can be added here as needed
// DO NOT move or remove existing re-exports

// Legacy incremental modules - depend on deprecated parsers
/// Incremental parsing façade used by older callers.
#[cfg(feature = "legacy-parsers")]
pub mod incremental;
/// Incremental parser v2 (position- and range-centric editing).
#[cfg(feature = "legacy-parsers")]
pub mod incremental_v2;
/// Incremental parser v3 (next-gen prototype).
#[cfg(feature = "legacy-parsers")]
pub mod incremental_v3;
/// Lexer implementation and token types.
pub mod lexer;
/// Registry for managing external scanners.
pub mod scanner_registry;
/// Built-in scanner implementations.
pub mod scanners;
// Use parser_v4 (GLR) as the main parser implementation
/// Main parser module.
#[cfg(feature = "pure-rust")]
pub mod parser {
    pub use super::parser_v4::*;
}
/// Error recovery strategies for parsing.
pub mod error_recovery;
/// Error reporting utilities.
pub mod error_reporting;
/// Legacy GLR module used by `parser_v3`.
#[cfg(feature = "legacy-parsers")]
pub mod glr; // Legacy GLR module that depends on parser_v3
/// GLR parse forest representation.
#[cfg(feature = "pure-rust")]
pub mod glr_forest;
/// Incremental parsing support for GLR.
#[cfg(feature = "pure-rust")]
pub mod glr_incremental;
// pub mod glr_incremental_opt; // Temporarily disabled during chunk-based refactor
/// Lexer specialized for GLR parsing.
pub mod glr_lexer;
/// GLR parser implementation.
pub mod glr_parser;
/// Query support for GLR parse forests.
pub mod glr_query;
/// Bridge between GLR parser and Tree-sitter trees.
pub mod glr_tree_bridge;
/// Validation utilities for GLR parsing.
pub mod glr_validation;
/// Bridge for converting between parse representations.
#[cfg(feature = "pure-rust")]
pub mod tree_bridge;
// pub mod glr_visualization; // TODO: Update for new GLRStack structure
/// Decoder for compressed parse tables.
#[cfg(feature = "pure-rust")]
pub mod decoder;
/// JSON grammar format support.
#[cfg(feature = "pure-rust")]
pub mod grammar_json;
/// Performance optimizations for parsing.
pub mod optimizations;

// Legacy parser versions - deprecated
#[cfg(feature = "legacy-parsers")]
mod parser_v2;
#[cfg(feature = "legacy-parsers")]
mod parser_v3;

// Current parser version
/// Arena allocator for parse tree nodes.
pub mod arena_allocator;
/// Version 4 parser implementation (GLR).
#[cfg(feature = "pure-rust")]
pub mod parser_v4;
/// Pure Rust external scanner support.
pub mod pure_external_scanner;
/// Pure Rust incremental parsing support.
pub mod pure_incremental;
/// Pure Rust parser implementation.
pub mod pure_parser;
/// Query language support for pattern matching.
#[cfg(feature = "pure-rust")]
pub mod query;
/// Stack pooling for efficient parsing.
pub mod stack_pool;
// #[cfg(feature = "serialization")]
/// Tree serialization utilities.
#[cfg(feature = "serialization")]
pub mod serialization;
/// Subtree representation and utilities.
pub mod subtree;
/// Unified parser interface.
#[cfg(feature = "pure-rust")]
pub mod unified_parser;
/// Tree visitor pattern implementations.
pub mod visitor;
/// SIMD-accelerated lexer module.
pub mod simd_lexer {
    pub use super::simd_lexer_v2::*;
}
mod simd_lexer_v2;

// Tree-sitter compatibility API
#[cfg(feature = "ts-compat")]
pub mod ts_compat;

// Re-export IR and GLR core for ts-compat language construction
#[cfg(feature = "ts-compat")]
pub use rust_sitter_glr_core;
#[cfg(feature = "ts-compat")]
pub use rust_sitter_ir;
// TODO: Update parallel_parser for new Parser API
// pub mod parallel_parser {
//     pub use super::parallel_parser_v2::*;
// }
// mod parallel_parser_v2;

#[cfg(feature = "pure-rust")]
mod tree_sitter_compat;

use std::ops::Deref;

pub use rust_sitter_macro::*;

#[cfg(all(
    feature = "tree-sitter-standard",
    not(feature = "tree-sitter-c2rust"),
    not(feature = "pure-rust")
))]
pub use tree_sitter;

#[cfg(all(feature = "tree-sitter-c2rust", not(feature = "pure-rust")))]
pub use tree_sitter_c2rust as tree_sitter;

/// Tree-sitter compatibility module for pure-Rust implementation.
#[cfg(feature = "pure-rust")]
pub mod tree_sitter {
    // Re-export pure-Rust types with Tree-sitter compatible names
    pub use crate::pure_incremental::{Edit, Tree};
    pub use crate::pure_parser::Point;
    pub use crate::pure_parser::{ParseResult, ParsedNode as Node};
    pub use crate::pure_parser::{Parser, TSLanguage as Language};

    // Re-export constants
    /// Language ABI version this runtime targets (Tree-sitter compatible).
    pub const LANGUAGE_VERSION: u32 = 15;
    /// Minimum compatible language ABI version.
    pub const MIN_COMPATIBLE_LANGUAGE_VERSION: u32 = 13;
}

/// Private module for sealing traits to preserve future extensibility.
pub mod sealed {
    /// Marker trait for types that can implement Extract.
    /// This trait is automatically implemented by the rust_sitter macros.
    pub trait Sealed {}

    // Auto-implement for all types by default to support macro-generated code
    // This is safe because Extract still requires explicit implementation
    impl<T> Sealed for T {}
}

/// Defines the logic used to convert a node in a Tree Sitter tree to
/// the corresponding Rust type.
///
/// This trait is sealed and cannot be implemented outside this crate,
/// allowing us to add new methods in the future without breaking changes.
pub trait Extract<Output>: sealed::Sealed {
    /// Associated function type for leaf node extraction.
    type LeafFn: ?Sized;
    /// Extracts a Rust value from a Tree-sitter node.
    #[cfg(not(feature = "pure-rust"))]
    fn extract(
        node: Option<tree_sitter::Node>,
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Output;

    /// Extracts a Rust value from a pure-Rust parse node.
    #[cfg(feature = "pure-rust")]
    fn extract(
        node: Option<&crate::pure_parser::ParsedNode>,
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Output;
}

/// Helper struct for specifying leaf extraction logic.
pub struct WithLeaf<L> {
    _phantom: std::marker::PhantomData<L>,
}

// The sealed trait is now auto-implemented for all types via blanket impl

impl<L> Extract<L> for WithLeaf<L> {
    type LeafFn = dyn Fn(&str) -> L;

    #[cfg(not(feature = "pure-rust"))]
    fn extract(
        node: Option<tree_sitter::Node>,
        source: &[u8],
        _last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> L {
        node.and_then(|n| n.utf8_text(source).ok())
            .map(|s| leaf_fn.unwrap()(s))
            .unwrap()
    }

    #[cfg(feature = "pure-rust")]
    fn extract(
        node: Option<&crate::pure_parser::ParsedNode>,
        source: &[u8],
        _last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> L {
        node.and_then(|n| {
            // Extract text from node's byte range
            let text = &source[n.start_byte..n.end_byte];
            std::str::from_utf8(text).ok()
        })
        .map(|s| leaf_fn.unwrap()(s))
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_valid_span() {
        let source = "hello world";
        let span = Spanned {
            value: (),
            span: (0, 5),
        };
        assert_eq!(&source[span], "hello");
    }

    #[test]
    #[should_panic(expected = "Invalid span")]
    fn index_invalid_span_panics() {
        let source = "hello";
        let span = Spanned {
            value: (),
            span: (0, 10),
        };
        let _ = &source[span];
    }

    #[test]
    fn index_mut_valid_span() {
        let mut source = String::from("hello world");
        let span = Spanned {
            value: (),
            span: (6, 11),
        };
        source.as_mut_str()[span].make_ascii_uppercase();
        assert_eq!(source, "hello WORLD");
    }

    #[test]
    #[should_panic(expected = "Invalid span")]
    fn index_mut_invalid_span_panics() {
        let mut source = String::from("hello");
        let span = Spanned {
            value: (),
            span: (6, 7),
        };
        let s = source.as_mut_str();
        let _ = &mut s[span];
    }
}

impl Extract<()> for () {
    type LeafFn = ();

    #[cfg(not(feature = "pure-rust"))]
    fn extract(
        _node: Option<tree_sitter::Node>,
        _source: &[u8],
        _last_idx: usize,
        _leaf_fn: Option<&Self::LeafFn>,
    ) {
    }

    #[cfg(feature = "pure-rust")]
    fn extract(
        _node: Option<&crate::pure_parser::ParsedNode>,
        _source: &[u8],
        _last_idx: usize,
        _leaf_fn: Option<&Self::LeafFn>,
    ) {
    }
}

impl<T: Extract<U>, U> Extract<Option<U>> for Option<T> {
    type LeafFn = T::LeafFn;

    #[cfg(not(feature = "pure-rust"))]
    fn extract(
        node: Option<tree_sitter::Node>,
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Option<U> {
        node.map(|n| T::extract(Some(n), source, last_idx, leaf_fn))
    }

    #[cfg(feature = "pure-rust")]
    fn extract(
        node: Option<&crate::pure_parser::ParsedNode>,
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Option<U> {
        node.map(|n| T::extract(Some(n), source, last_idx, leaf_fn))
    }
}

impl<T: Extract<U>, U> Extract<Box<U>> for Box<T> {
    type LeafFn = T::LeafFn;

    #[cfg(not(feature = "pure-rust"))]
    fn extract(
        node: Option<tree_sitter::Node>,
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Box<U> {
        Box::new(T::extract(node, source, last_idx, leaf_fn))
    }

    #[cfg(feature = "pure-rust")]
    fn extract(
        node: Option<&crate::pure_parser::ParsedNode>,
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Box<U> {
        Box::new(T::extract(node, source, last_idx, leaf_fn))
    }
}

impl<T: Extract<U>, U> Extract<Vec<U>> for Vec<T> {
    type LeafFn = T::LeafFn;

    #[cfg(not(feature = "pure-rust"))]
    fn extract(
        node: Option<tree_sitter::Node>,
        source: &[u8],
        mut last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Vec<U> {
        node.map(|node| {
            let mut cursor = node.walk();
            let mut out = vec![];
            if cursor.goto_first_child() {
                loop {
                    let n = cursor.node();
                    if cursor.field_name().is_some() {
                        out.push(T::extract(Some(n), source, last_idx, leaf_fn));
                    }

                    last_idx = n.end_byte();

                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            out
        })
        .unwrap_or_default()
    }

    #[cfg(feature = "pure-rust")]
    fn extract(
        node: Option<&crate::pure_parser::ParsedNode>,
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Vec<U> {
        node.map(|node| {
            let mut out = vec![];

            // Debug output commented out
            // eprintln!("DEBUG Vec extract: node.symbol={}, children.len()={}", node.symbol, node.children.len());
            // for (i, child) in node.children.iter().enumerate() {
            //     eprintln!("  child[{}]: symbol={}, field_name={:?}", i, child.symbol, child.field_name);
            // }

            // For pure-rust parser, REPEAT1 creates a right-recursive structure:
            // For "12 23", the structure is:
            // - Vec_contents has [Vec_contents("12"), TestStatement("23")]
            // We need to flatten this recursively
            fn flatten_repeat1<T: Extract<U>, U>(
                node: &crate::pure_parser::ParsedNode,
                source: &[u8],
                mut last_idx: usize,
                leaf_fn: Option<&T::LeafFn>,
                out: &mut Vec<U>,
            ) {
                // eprintln!("  flatten_repeat1: node.symbol={}, children={}", node.symbol, node.children.len());

                // Check if this node has exactly 2 children and the first is the same symbol
                // This indicates a REPEAT1 recursive structure
                if node.children.len() == 2
                    && !node.children.is_empty()
                    && node.children[0].symbol == node.symbol
                {
                    // Recursively process the first child (which contains earlier elements)
                    flatten_repeat1::<T, U>(&node.children[0], source, last_idx, leaf_fn, out);
                    // Then extract the second child (the last element)
                    // eprintln!("  Extracting element from symbol={}", node.children[1].symbol);
                    out.push(T::extract(
                        Some(&node.children[1]),
                        source,
                        node.children[0].end_byte,
                        leaf_fn,
                    ));
                } else if node.children.len() == 1 {
                    // Base case: single element
                    // eprintln!("  Base case: extracting single element from symbol={}", node.children[0].symbol);
                    out.push(T::extract(
                        Some(&node.children[0]),
                        source,
                        last_idx,
                        leaf_fn,
                    ));
                } else {
                    // Fallback: extract all children
                    for child in &node.children {
                        // eprintln!("  Fallback: extracting child symbol={}", child.symbol);
                        out.push(T::extract(Some(child), source, last_idx, leaf_fn));
                        last_idx = child.end_byte;
                    }
                }
            }

            flatten_repeat1::<T, U>(node, source, last_idx, leaf_fn, &mut out);
            // eprintln!("  Vec extract returning {} items", out.len());
            out
        })
        .unwrap_or_default()
    }
}

#[derive(Clone, Debug)]
/// A wrapper around a value that also contains the span of the value in the source.
pub struct Spanned<T> {
    /// The underlying parsed node.
    pub value: T,
    /// The span of the node in the source. The first value is the inclusive start
    /// of the span, and the second value is the exclusive end of the span.
    pub span: (usize, usize),
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: Extract<U>, U> Extract<Spanned<U>> for Spanned<T> {
    type LeafFn = T::LeafFn;

    #[cfg(not(feature = "pure-rust"))]
    fn extract(
        node: Option<tree_sitter::Node>,
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Spanned<U> {
        Spanned {
            value: T::extract(node, source, last_idx, leaf_fn),
            span: node
                .map(|n| (n.start_byte(), n.end_byte()))
                .unwrap_or((last_idx, last_idx)),
        }
    }

    #[cfg(feature = "pure-rust")]
    fn extract(
        node: Option<&crate::pure_parser::ParsedNode>,
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Spanned<U> {
        Spanned {
            value: T::extract(node, source, last_idx, leaf_fn),
            span: node
                .map(|n| (n.start_byte, n.end_byte))
                .unwrap_or((last_idx, last_idx)),
        }
    }
}

impl<T> std::ops::Index<Spanned<T>> for str {
    type Output = str;

    fn index(&self, span: Spanned<T>) -> &Self::Output {
        let (start, end) = span.span;
        self.get(start..end).unwrap_or_else(|| {
            panic!(
                "Invalid span {start}..{end} for string of length {}",
                self.len()
            )
        })
    }
}

impl<T> std::ops::IndexMut<Spanned<T>> for str {
    fn index_mut(&mut self, span: Spanned<T>) -> &mut Self::Output {
        let (start, end) = span.span;
        let len = self.len();
        self.get_mut(start..end)
            .unwrap_or_else(|| panic!("Invalid span {start}..{end} for string of length {len}",))
    }
}

impl Extract<String> for String {
    type LeafFn = ();

    #[cfg(not(feature = "pure-rust"))]
    fn extract(
        node: Option<tree_sitter::Node>,
        source: &[u8],
        _last_idx: usize,
        _leaf_fn: Option<&Self::LeafFn>,
    ) -> String {
        node.and_then(|n| n.utf8_text(source).ok())
            .unwrap_or_default()
            .to_string()
    }

    #[cfg(feature = "pure-rust")]
    fn extract(
        node: Option<&crate::pure_parser::ParsedNode>,
        source: &[u8],
        _last_idx: usize,
        _leaf_fn: Option<&Self::LeafFn>,
    ) -> String {
        node.and_then(|n| {
            // Extract text from node's byte range
            let text = &source[n.start_byte..n.end_byte];
            std::str::from_utf8(text).ok()
        })
        .unwrap_or_default()
        .to_string()
    }
}

/// Error types for parsing operations.
pub mod errors {
    #[cfg(all(
        feature = "tree-sitter-standard",
        not(feature = "tree-sitter-c2rust"),
        not(feature = "pure-rust")
    ))]
    use tree_sitter;

    #[cfg(all(feature = "tree-sitter-c2rust", not(feature = "pure-rust")))]
    use tree_sitter_c2rust as tree_sitter;

    #[derive(Debug)]
    /// An explanation for an error that occurred during parsing.
    pub enum ParseErrorReason {
        /// The parser did not expect to see some token.
        UnexpectedToken(String),
        /// Tree Sitter failed to parse a specific intermediate node.
        /// The underlying failures are in the vector.
        FailedNode(Vec<ParseError>),
        /// The parser expected a specific token, but it was not found.
        MissingToken(String),
    }

    #[derive(Debug)]
    /// An error that occurred during parsing.
    pub struct ParseError {
        /// The reason for the parse error.
        pub reason: ParseErrorReason,
        /// Inclusive start of the error.
        pub start: usize,
        /// Exclusive end of the error.
        pub end: usize,
    }

    /// Given the root node of a Tree Sitter parsing result, accumulates all
    /// errors that were emitted.
    #[cfg(not(feature = "pure-rust"))]
    pub fn collect_parsing_errors(
        node: &tree_sitter::Node,
        source: &[u8],
        errors: &mut Vec<ParseError>,
    ) {
        if node.is_error() {
            if node.child(0).is_some() {
                // we managed to parse some children, so collect underlying errors for this node
                let mut inner_errors = vec![];
                let mut cursor = node.walk();
                node.children(&mut cursor)
                    .for_each(|c| collect_parsing_errors(&c, source, &mut inner_errors));

                errors.push(ParseError {
                    reason: ParseErrorReason::FailedNode(inner_errors),
                    start: node.start_byte(),
                    end: node.end_byte(),
                })
            } else {
                let contents = node.utf8_text(source).unwrap();
                if !contents.is_empty() {
                    errors.push(ParseError {
                        reason: ParseErrorReason::UnexpectedToken(contents.to_string()),
                        start: node.start_byte(),
                        end: node.end_byte(),
                    })
                } else {
                    errors.push(ParseError {
                        reason: ParseErrorReason::FailedNode(vec![]),
                        start: node.start_byte(),
                        end: node.end_byte(),
                    })
                }
            }
        } else if node.is_missing() {
            errors.push(ParseError {
                reason: ParseErrorReason::MissingToken(node.kind().to_string()),
                start: node.start_byte(),
                end: node.end_byte(),
            })
        } else if node.has_error() {
            let mut cursor = node.walk();
            node.children(&mut cursor)
                .for_each(|c| collect_parsing_errors(&c, source, errors));
        }
    }

    /// Given the root node of a Tree Sitter parsing result, accumulates all
    /// errors that were emitted.
    #[cfg(feature = "pure-rust")]
    pub fn collect_parsing_errors(
        node: &crate::pure_parser::ParsedNode,
        source: &[u8],
        errors: &mut Vec<ParseError>,
    ) {
        // TODO: Implement error collection for pure-rust parser
        // For now, just check if this is an error node
        if false {
            // TODO: Check if error node
            let contents =
                std::str::from_utf8(&source[node.start_byte..node.end_byte]).unwrap_or("");
            if !contents.is_empty() {
                errors.push(ParseError {
                    reason: ParseErrorReason::UnexpectedToken(contents.to_string()),
                    start: node.start_byte,
                    end: node.end_byte,
                })
            }
        }

        // Recursively check children
        for child in &node.children {
            collect_parsing_errors(child, source, errors);
        }
    }
}
