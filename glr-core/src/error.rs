#[derive(Debug, thiserror::Error)]
pub enum GlrError {
    #[error("invalid grammar: {0}")]
    InvalidGrammar(String),

    #[error("unresolvable conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, GlrError>;