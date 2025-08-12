// Re-export the GLRError from lib.rs for consistent naming
pub type Result<T> = std::result::Result<T, crate::GLRError>;