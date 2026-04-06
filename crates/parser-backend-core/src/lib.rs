//! Core parser backend selection primitives.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use core::fmt::{self, Display, Formatter};

/// Contract outcome for backend selection under a given conflict condition.
///
/// This keeps behavior assertions in one place across parser and governance
/// contracts without forcing callers to duplicate panic-string checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserBackendSelection {
    /// A concrete backend can be selected for the current feature set.
    Backend(ParserBackend),
    /// Conflict grammars require the `glr` feature to be enabled.
    ConflictsRequireGlr,
}

/// Message emitted when conflict handling requires GLR support.
pub const CONFLICTS_REQUIRE_GLR_MESSAGE: &str = "Grammar has conflicts but GLR feature is not enabled. Enable the 'glr' feature in Cargo.toml or use the tree-sitter C runtime.";

impl ParserBackend {
    /// Resolve the backend-selection contract for a conflict condition.
    ///
    /// This mirrors `select` but returns an explicit outcome rather than panicking.
    pub const fn select_contract(has_conflicts: bool) -> ParserBackendSelection {
        match (cfg!(feature = "glr"), cfg!(feature = "pure-rust")) {
            (true, _) => ParserBackendSelection::Backend(Self::GLR),
            (false, true) => {
                if has_conflicts {
                    ParserBackendSelection::ConflictsRequireGlr
                } else {
                    ParserBackendSelection::Backend(Self::PureRust)
                }
            }
            _ => ParserBackendSelection::Backend(Self::TreeSitter),
        }
    }
}

/// Parser backend supported by the runtime feature matrix.
///
/// # Examples
///
/// ```
/// use adze_parser_backend_core::ParserBackend;
///
/// let backend = ParserBackend::GLR;
/// assert!(backend.is_glr());
/// assert!(backend.is_pure_rust());
/// assert_eq!(backend.name(), "pure-Rust GLR parser");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserBackend {
    /// Tree-sitter C runtime (default when pure-Rust is disabled).
    TreeSitter,
    /// Pure Rust LR parser (simple grammars without conflicts).
    PureRust,
    /// Pure Rust GLR parser (conflict-capable).
    GLR,
}

impl ParserBackend {
    /// Select parser backend based on feature flags and grammar metadata.
    ///
    /// # Arguments
    /// * `has_conflicts` - Whether the grammar contains shift/reduce or reduce/reduce conflicts.
    ///
    /// # Panics
    /// Panics if `has_conflicts` is true and the `pure-rust` feature is enabled without the `glr` feature.
    pub const fn select(_has_conflicts: bool) -> Self {
        match Self::select_contract(_has_conflicts) {
            ParserBackendSelection::Backend(backend) => backend,
            ParserBackendSelection::ConflictsRequireGlr => {
                panic!("{}", CONFLICTS_REQUIRE_GLR_MESSAGE)
            }
        }
    }

    /// Whether this backend is the GLR parser.
    pub const fn is_glr(self) -> bool {
        matches!(self, Self::GLR)
    }

    /// Whether this backend is a pure-Rust parser (LR or GLR).
    pub const fn is_pure_rust(self) -> bool {
        matches!(self, Self::PureRust | Self::GLR)
    }

    /// Human-readable backend name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::TreeSitter => "tree-sitter C runtime",
            Self::PureRust => "pure-Rust LR parser",
            Self::GLR => "pure-Rust GLR parser",
        }
    }
}

impl Display for ParserBackend {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_returns_human_readable_string() {
        assert_eq!(ParserBackend::TreeSitter.name(), "tree-sitter C runtime");
        assert_eq!(ParserBackend::PureRust.name(), "pure-Rust LR parser");
        assert_eq!(ParserBackend::GLR.name(), "pure-Rust GLR parser");
    }

    #[test]
    fn display_matches_name() {
        for backend in [
            ParserBackend::TreeSitter,
            ParserBackend::PureRust,
            ParserBackend::GLR,
        ] {
            assert_eq!(format!("{backend}"), backend.name());
        }
    }

    #[test]
    fn is_glr_only_for_glr() {
        assert!(ParserBackend::GLR.is_glr());
        assert!(!ParserBackend::PureRust.is_glr());
        assert!(!ParserBackend::TreeSitter.is_glr());
    }

    #[test]
    fn is_pure_rust_for_lr_and_glr() {
        assert!(ParserBackend::PureRust.is_pure_rust());
        assert!(ParserBackend::GLR.is_pure_rust());
        assert!(!ParserBackend::TreeSitter.is_pure_rust());
    }

    #[test]
    fn clone_and_eq() {
        let a = ParserBackend::GLR;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn debug_format() {
        let dbg = format!("{:?}", ParserBackend::TreeSitter);
        assert_eq!(dbg, "TreeSitter");
    }
}
