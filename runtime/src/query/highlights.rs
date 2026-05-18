//! Syntax highlighting support using queries.
//!
//! This module is split by responsibility:
//! - capture name constants live in [`capture_names`]
//! - query-to-range extraction lives in [`highlighter`]
//! - theme and color handling live in [`theme`]
//! - bundled query snippets live in [`queries`]

pub mod capture_names;
pub mod highlighter;
pub mod queries;
pub mod theme;
pub mod types;

pub use highlighter::Highlighter;
pub use theme::{Color, Theme};
pub use types::Highlight;
