//! Error types for parsing operations

use std::fmt;
use thiserror::Error;

/// Parse error with details about what went wrong
#[derive(Debug, Error)]
#[error("{kind}")]
pub struct ParseError {
    /// The kind of error
    pub kind: ParseErrorKind,
    /// Optional location where the error occurred
    pub location: Option<ErrorLocation>,
}

/// Kinds of parse errors
#[derive(Debug, Error)]
pub enum ParseErrorKind {
    /// No language was set on the parser
    #[error("no language set")]
    NoLanguage,

    /// Parse timeout exceeded
    #[error("parse timeout exceeded")]
    Timeout,

    /// Invalid input encoding
    #[error("invalid input encoding")]
    InvalidEncoding,

    /// Parse was cancelled
    #[error("parse cancelled")]
    Cancelled,

    /// Language version mismatch
    #[error("language version mismatch: expected {expected}, got {actual}")]
    VersionMismatch {
        /// Expected version
        expected: u32,
        /// Actual version found
        actual: u32,
    },

    /// Syntax error in input
    #[error("syntax error at {0}")]
    SyntaxError(String),

    /// External scanner error
    #[cfg(feature = "external-scanners")]
    #[error("external scanner error: {0}")]
    ExternalScannerError(String),

    /// Memory allocation failure
    #[error("memory allocation failed")]
    AllocationError,

    /// Other error with custom message
    #[error("{0}")]
    Other(String),
}

/// Location information for an error
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorLocation {
    /// Byte offset in the input
    pub byte_offset: usize,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

impl ParseError {
    /// Create a "no language" error
    pub fn no_language() -> Self {
        Self {
            kind: ParseErrorKind::NoLanguage,
            location: None,
        }
    }

    /// Create a timeout error
    pub fn timeout() -> Self {
        Self {
            kind: ParseErrorKind::Timeout,
            location: None,
        }
    }

    /// Create a syntax error with location
    pub fn syntax_error(message: impl Into<String>, location: ErrorLocation) -> Self {
        Self {
            kind: ParseErrorKind::SyntaxError(message.into()),
            location: Some(location),
        }
    }

    /// Add location information to this error
    pub fn with_location(mut self, location: ErrorLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Create an error with a custom message
    pub fn with_msg(msg: &str) -> Self {
        Self {
            kind: ParseErrorKind::Other(msg.to_string()),
            location: None,
        }
    }
}

impl fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[cfg(feature = "glr-core")]
impl From<rust_sitter_glr_core::driver::GlrError> for ParseError {
    fn from(e: rust_sitter_glr_core::driver::GlrError) -> Self {
        ParseError::with_msg(&e.to_string())
    }
}
