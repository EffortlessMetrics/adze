//! Query engine components (AST, compiler, matcher, highlighter, etc.).
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

// Tree-sitter query language support for rust-sitter
// This module implements the S-expression based query language for pattern matching

pub mod ast;
pub mod compiler;
pub mod cursor;
pub mod highlights;
pub mod matcher;
pub mod matcher_v2;
pub mod parser;
pub mod pattern;
pub mod predicate_eval;

pub use ast::{Query, QueryError};
pub use compiler::compile_query;
pub use cursor::QueryCursor;
pub use highlights::{Color, Highlight, Highlighter, Theme};
pub use matcher::{QueryCapture, QueryMatch, QueryMatches};
pub use pattern::{Pattern, Predicate};
pub use predicate_eval::PredicateContext;
