//! Error types for grammar IR operations.

/// Errors that can occur while building and validating the IR.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum IrError {
    /// The referenced symbol was malformed or not present in the grammar.
    #[error("invalid symbol: {0}")]
    InvalidSymbol(String),

    /// Attempted to insert a rule that already exists.
    #[error("duplicate rule: {0}")]
    DuplicateRule(String),

    /// An unexpected internal IR failure.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Convenience type alias for IR results.
pub type Result<T> = std::result::Result<T, IrError>;
