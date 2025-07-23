// Common external scanners for rust-sitter
// These are Rust implementations of common scanning patterns

pub mod indentation;

pub use indentation::IndentationScanner;

// Re-export from parent module for convenience
pub use crate::external_scanner::{StringScanner, CommentScanner};