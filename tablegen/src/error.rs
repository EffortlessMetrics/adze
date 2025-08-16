/// Errors produced by table generation and compression.
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum TableGenError {
    /// Invalid input was provided to a table generation function.
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),

    /// Automaton construction failed during table generation.
    #[error("automaton build failed: {0}")]
    Automaton(String),

    /// Table compression algorithm encountered an error.
    #[error("compression failed: {0}")]
    Compression(String),

    /// General table generation failure, often from upstream errors.
    #[error("table generation failed: {0}")]
    TableGeneration(String),

    /// The table structure is invalid or corrupted.
    #[error("invalid table structure: {0}")]
    InvalidTable(String),

    /// Symbol index is out of bounds for the grammar.
    #[error("symbol index out of bounds: {0}")]
    InvalidSymbolIndex(usize),

    /// State index is out of bounds for the parse table.
    #[error("state index out of bounds: {0}")]
    InvalidStateIndex(usize),

    /// The grammar is empty and cannot be processed.
    #[error("empty grammar")]
    EmptyGrammar,

    /// Grammar validation failed before table generation.
    #[error("grammar validation failed: {0}")]
    ValidationError(String),

    /// I/O error occurred during file operations.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Error bubbled from the GLR automaton builder.
    #[error(transparent)]
    Glr(#[from] rust_sitter_glr_core::GLRError),

    /// Error bubbled from the IR layer.
    #[error(transparent)]
    Ir(#[from] rust_sitter_ir::IrError),
}

/// Convenience type alias for TableGen results.
pub type Result<T> = std::result::Result<T, TableGenError>;

impl From<String> for TableGenError {
    fn from(s: String) -> Self {
        TableGenError::TableGeneration(s)
    }
}

impl From<&str> for TableGenError {
    fn from(s: &str) -> Self {
        TableGenError::TableGeneration(s.to_string())
    }
}
