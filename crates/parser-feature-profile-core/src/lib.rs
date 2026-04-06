//! Core parser feature-profile representation and backend resolution policy.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use core::fmt::{self, Display, Formatter};

/// Re-exported parser backend enum for feature-profile–based backend resolution.
pub use adze_parser_backend_core::ParserBackend;
pub use adze_parser_backend_core::ParserBackendSelection;

/// Snapshot of parser-related feature flags for this build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParserFeatureProfile {
    /// `pure-rust` feature is enabled.
    pub pure_rust: bool,
    /// `tree-sitter-standard` feature is enabled.
    pub tree_sitter_standard: bool,
    /// `tree-sitter-c2rust` feature is enabled.
    pub tree_sitter_c2rust: bool,
    /// `glr` feature is enabled.
    pub glr: bool,
}

impl ParserFeatureProfile {
    /// Snapshot of active feature flags for the current crate compilation.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            pure_rust: cfg!(feature = "pure-rust"),
            tree_sitter_standard: cfg!(feature = "tree-sitter-standard"),
            tree_sitter_c2rust: cfg!(feature = "tree-sitter-c2rust"),
            glr: cfg!(feature = "glr"),
        }
    }

    /// Resolve the effective backend from this profile.
    #[must_use]
    pub const fn resolve_backend(self, has_conflicts: bool) -> ParserBackend {
        match Self::backend_selection_contract(self, has_conflicts) {
            ParserBackendSelection::Backend(backend) => backend,
            ParserBackendSelection::ConflictsRequireGlr => panic!(
                "Grammar has conflicts but GLR feature is not enabled. Enable the 'glr' feature in Cargo.toml or use the tree-sitter C runtime."
            ),
        }
    }

    /// Resolution contract for this profile and conflict condition.
    ///
    /// Exposed for test-surface and migration checks that need to compare panic-vs.
    /// backend-returning behavior without matching panic strings.
    #[must_use]
    pub const fn backend_selection_contract(self, has_conflicts: bool) -> ParserBackendSelection {
        if self.glr {
            ParserBackendSelection::Backend(ParserBackend::GLR)
        } else if self.pure_rust {
            if has_conflicts {
                ParserBackendSelection::ConflictsRequireGlr
            } else {
                ParserBackendSelection::Backend(ParserBackend::PureRust)
            }
        } else {
            ParserBackendSelection::Backend(ParserBackend::TreeSitter)
        }
    }

    /// Whether feature flags indicate the pure-Rust runtime is compiled in.
    #[must_use]
    pub const fn has_pure_rust(self) -> bool {
        self.pure_rust
    }

    /// Whether feature flags indicate GLR is compiled in.
    #[must_use]
    pub const fn has_glr(self) -> bool {
        self.glr
    }

    /// Whether feature flags indicate any tree-sitter backend is compiled in.
    #[must_use]
    pub const fn has_tree_sitter(self) -> bool {
        self.tree_sitter_standard || self.tree_sitter_c2rust
    }
}

impl Display for ParserFeatureProfile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut active = 0usize;

        if self.pure_rust {
            write!(f, "pure-rust")?;
            active += 1;
        }
        if self.tree_sitter_standard {
            if active > 0 {
                write!(f, ", ")?;
            }
            write!(f, "tree-sitter-standard")?;
            active += 1;
        }
        if self.tree_sitter_c2rust {
            if active > 0 {
                write!(f, ", ")?;
            }
            write!(f, "tree-sitter-c2rust")?;
            active += 1;
        }
        if self.glr {
            if active > 0 {
                write!(f, ", ")?;
            }
            write!(f, "glr")?;
            active += 1;
        }

        if active == 0 {
            write!(f, "none")
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_matches_cfg() {
        let profile = ParserFeatureProfile::current();
        assert_eq!(profile.pure_rust, cfg!(feature = "pure-rust"));
        assert_eq!(
            profile.tree_sitter_standard,
            cfg!(feature = "tree-sitter-standard")
        );
        assert_eq!(
            profile.tree_sitter_c2rust,
            cfg!(feature = "tree-sitter-c2rust")
        );
        assert_eq!(profile.glr, cfg!(feature = "glr"));
    }

    #[test]
    fn resolve_backend_glr_takes_priority() {
        let profile = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: true,
            tree_sitter_c2rust: true,
            glr: true,
        };
        assert_eq!(profile.resolve_backend(false), ParserBackend::GLR);
        assert_eq!(profile.resolve_backend(true), ParserBackend::GLR);
    }

    #[test]
    fn resolve_backend_pure_rust_without_conflicts() {
        let profile = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert_eq!(profile.resolve_backend(false), ParserBackend::PureRust);
    }

    #[test]
    #[should_panic(expected = "GLR feature is not enabled")]
    fn resolve_backend_pure_rust_with_conflicts_panics() {
        let profile = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        let _ = profile.resolve_backend(true);
    }

    #[test]
    fn display_none_when_empty() {
        let profile = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert_eq!(profile.to_string(), "none");
    }
}
