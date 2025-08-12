#[derive(Debug, thiserror::Error)]
pub enum IrError {
    #[error("invalid symbol: {0}")]
    InvalidSymbol(String),

    #[error("duplicate rule: {0}")]
    DuplicateRule(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, IrError>;