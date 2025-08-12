// Re-export the GLRError from lib.rs for consistent naming
/// Convenience type alias for GLR results.
pub type Result<T> = std::result::Result<T, crate::GLRError>;