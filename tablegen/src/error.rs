#[derive(Debug, thiserror::Error)]
pub enum TableGenError {
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),

    #[error("automaton build failed: {0}")]
    Automaton(String),

    #[error("compression failed: {0}")]
    Compression(String),

    #[error("table generation failed: {0}")]
    TableGeneration(String),
    
    #[error("invalid table structure: {0}")]
    InvalidTable(String),

    #[error("symbol index out of bounds: {0}")]
    InvalidSymbolIndex(usize),

    #[error("state index out of bounds: {0}")]
    InvalidStateIndex(usize),

    #[error("empty grammar")]
    EmptyGrammar,

    #[error("grammar validation failed: {0}")]
    ValidationError(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

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

impl From<rust_sitter_glr_core::GLRError> for TableGenError {
    fn from(e: rust_sitter_glr_core::GLRError) -> Self {
        // Treat upstream generator/analysis failures as table generation errors.
        TableGenError::TableGeneration(e.to_string())
    }
}

impl From<rust_sitter_ir::IrError> for TableGenError {
    fn from(e: rust_sitter_ir::IrError) -> Self {
        // Same rationale: tablegen orchestrates IR → automaton → compression.
        TableGenError::TableGeneration(e.to_string())
    }
}