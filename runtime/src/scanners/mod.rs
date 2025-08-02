// Common external scanners for rust-sitter
// These are Rust implementations of common scanning patterns

pub mod heredoc;
pub mod indentation;

pub use heredoc::HeredocScanner;
pub use indentation::IndentationScanner;

// Re-export from parent module for convenience
pub use crate::external_scanner::{CommentScanner, StringScanner};
