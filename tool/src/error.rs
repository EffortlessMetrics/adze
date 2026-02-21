/// Errors that can occur during grammar parsing and expansion
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// Multiple word rules were specified when only one is allowed
    #[error("multiple word rules specified - only one word rule is allowed per grammar")]
    MultipleWordRules,

    /// Multiple precedence attributes were specified when only one is allowed
    #[error("only one of prec, prec_left, and prec_right can be specified")]
    MultiplePrecedenceAttributes,

    /// Expected a string literal but found something else
    #[error("expected string literal for {context}: {actual}")]
    ExpectedStringLiteral { context: String, actual: String },

    /// Expected an integer literal but found something else
    #[error("expected integer literal for precedence: {actual}")]
    ExpectedIntegerLiteral { actual: String },

    /// Expected a path type but found something else
    #[error("expected a path or unit type: {actual}")]
    ExpectedPathType { actual: String },

    /// Expected a single segment path but found multiple segments
    #[error("expected a single segment path: {actual}")]
    ExpectedSingleSegmentPath { actual: String },

    /// Nested Option types are not supported
    #[error("Option<Option<_>> is not supported")]
    NestedOptionType,

    /// Struct has no non-skipped fields
    #[error("struct {name} has no non-skipped fields")]
    StructHasNoFields { name: String },

    /// Complex symbols should be normalized before processing
    #[error("complex symbols should be normalized before {operation}")]
    ComplexSymbolsNotNormalized { operation: String },

    /// Expected a specific symbol type but found something else
    #[error("expected {expected} symbol")]
    ExpectedSymbolType { expected: String },

    /// Expected a specific action type but found something else
    #[error("expected {expected} action")]
    ExpectedActionType { expected: String },

    /// Expected a specific error type but found something else
    #[error("expected {expected} error")]
    ExpectedErrorType { expected: String },

    /// String too long for extraction
    #[error("string too long for {operation}: length {length} exceeds maximum")]
    StringTooLong { operation: String, length: usize },

    /// Invalid production rule
    #[error("invalid production rule: {details}")]
    InvalidProduction { details: String },

    /// Grammar validation failed
    #[error("grammar validation failed: {reason}")]
    GrammarValidation { reason: String },

    /// Other tool error with custom message
    #[error("{0}")]
    Other(String),

    /// IO error occurred during file operations
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Error from the IR layer
    #[error(transparent)]
    Ir(#[from] adze_ir::IrError),

    /// Error from the GLR core
    #[error(transparent)]
    Glr(#[from] adze_glr_core::GLRError),

    /// Error from table generation
    #[error(transparent)]
    TableGen(#[from] adze_tablegen::TableGenError),

    /// Syn parsing error
    #[error(transparent)]
    SynError {
        #[from]
        syn_error: syn::Error,
    },
}

/// Convenience type alias for tool results
pub type Result<T> = std::result::Result<T, ToolError>;

impl From<String> for ToolError {
    fn from(s: String) -> Self {
        ToolError::Other(s)
    }
}

impl From<&str> for ToolError {
    fn from(s: &str) -> Self {
        ToolError::Other(s.to_string())
    }
}

impl ToolError {
    /// Create a string too long error
    pub fn string_too_long(operation: &str, length: usize) -> Self {
        ToolError::StringTooLong {
            operation: operation.to_string(),
            length,
        }
    }

    /// Create a complex symbols error
    pub fn complex_symbols_not_normalized(operation: &str) -> Self {
        ToolError::ComplexSymbolsNotNormalized {
            operation: operation.to_string(),
        }
    }

    /// Create an expected symbol type error
    pub fn expected_symbol_type(expected: &str) -> Self {
        ToolError::ExpectedSymbolType {
            expected: expected.to_string(),
        }
    }

    /// Create an expected action type error
    pub fn expected_action_type(expected: &str) -> Self {
        ToolError::ExpectedActionType {
            expected: expected.to_string(),
        }
    }

    /// Create an expected error type error
    pub fn expected_error_type(expected: &str) -> Self {
        ToolError::ExpectedErrorType {
            expected: expected.to_string(),
        }
    }

    /// Create a grammar validation error
    pub fn grammar_validation(reason: &str) -> Self {
        ToolError::GrammarValidation {
            reason: reason.to_string(),
        }
    }
}
