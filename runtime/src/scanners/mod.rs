//! Common external scanners for adze.
//! These are Rust implementations of common scanning patterns.
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

// Common external scanners for adze
// These are Rust implementations of common scanning patterns

/// Heredoc scanner implementation.
pub mod heredoc;
/// Indentation-based scanner implementation.
pub mod indentation;

pub use heredoc::HeredocScanner;
pub use indentation::IndentationScanner;

// Re-export from parent module for convenience
pub use crate::external_scanner::{CommentScanner, StringScanner};
