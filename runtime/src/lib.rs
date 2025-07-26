pub mod __private;
pub mod external_scanner;
pub mod external_scanner_ffi;
pub mod scanner_registry;
pub mod scanners;
pub mod incremental;
pub mod incremental_v2;
pub mod incremental_v3;
pub mod lexer;
// Use parser_v3 as the main parser implementation
pub mod parser {
    pub use super::parser_v3::*;
}
mod parser_v2;
mod parser_v3;
// mod parser_v4; // Disabled - incompatible with current ParseTable structure
pub mod glr;
pub mod glr_parser;
pub mod error_recovery;
pub mod visitor;
pub mod query;
pub mod subtree;
pub mod error_reporting;
pub mod glr_lexer;
pub mod glr_tree_bridge;
pub mod glr_incremental;
pub mod glr_validation;
pub mod glr_query;
#[cfg(feature = "serialization")]
pub mod serialization;
pub mod pure_parser;
pub mod pure_incremental;
pub mod pure_external_scanner;
pub mod unified_parser;
pub mod optimizations;
pub mod simd_lexer {
    pub use super::simd_lexer_v2::*;
}
mod simd_lexer_v2;
pub mod parallel_parser {
    pub use super::parallel_parser_v2::*;
}
mod parallel_parser_v2;

#[cfg(feature = "pure-rust")]
mod tree_sitter_compat;

use std::ops::Deref;

pub use rust_sitter_macro::*;

#[cfg(all(feature = "tree-sitter-standard", not(feature = "pure-rust")))]
pub use tree_sitter_runtime_standard as tree_sitter;

#[cfg(all(feature = "tree-sitter-c2rust", not(feature = "pure-rust")))]
pub use tree_sitter_runtime_c2rust as tree_sitter;

#[cfg(feature = "pure-rust")]
pub mod tree_sitter {
    // Re-export pure-Rust types with Tree-sitter compatible names
    pub use crate::pure_parser::{Parser, TSLanguage as Language};
    pub use crate::pure_parser::{ParsedNode as Node, ParseResult};
    pub use crate::pure_incremental::{Tree, Edit};
    pub use crate::pure_parser::Point;
    
    // Re-export constants
    pub const LANGUAGE_VERSION: u32 = 14;
    pub const MIN_COMPATIBLE_LANGUAGE_VERSION: u32 = 13;
}

/// Defines the logic used to convert a node in a Tree Sitter tree to
/// the corresponding Rust type.
pub trait Extract<Output> {
    type LeafFn: ?Sized;
    #[cfg(not(feature = "pure-rust"))]
    fn extract(
        node: Option<tree_sitter::Node>,
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Output;
    
    #[cfg(feature = "pure-rust")]
    fn extract(
        node: Option<&crate::pure_parser::ParsedNode>,
        source: &[u8],
        last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Output;
}

pub struct WithLeaf<L> {
    _phantom: std::marker::PhantomData<L>,
}

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
        mut last_idx: usize,
        leaf_fn: Option<&Self::LeafFn>,
    ) -> Vec<U> {
        node.map(|node| {
            let mut out = vec![];
            // For pure-rust, iterate through children directly
            for child in &node.children {
                // TODO: Check field names when available
                out.push(T::extract(Some(child), source, last_idx, leaf_fn));
                last_idx = child.end_byte;
            }
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

pub mod errors {
    #[cfg(all(feature = "tree-sitter-standard", not(feature = "pure-rust")))]
    use tree_sitter_runtime_standard as tree_sitter;

    #[cfg(all(feature = "tree-sitter-c2rust", not(feature = "pure-rust")))]
    use tree_sitter_runtime_c2rust as tree_sitter;
    
    #[cfg(feature = "pure-rust")]
    use crate::tree_sitter;

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
        if false { // TODO: Check if error node
            let contents = std::str::from_utf8(&source[node.start_byte..node.end_byte]).unwrap_or("");
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
