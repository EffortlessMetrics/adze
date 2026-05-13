//! Core contracts for parser backend selection and feature profiles.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use core::fmt::{self, Display, Formatter};

/// Message emitted when conflict handling requires GLR support.
pub const CONFLICTS_REQUIRE_GLR_MESSAGE: &str = "Grammar has conflicts but GLR feature is not enabled. Enable the 'glr' feature in Cargo.toml or use the tree-sitter C runtime.";

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

/// Parser backend supported by the runtime feature matrix.
///
/// # Examples
///
/// ```
/// use adze_feature_policy_core::ParserBackend;
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
    /// Resolve the backend-selection contract for a conflict condition.
    ///
    /// This mirrors `select` but returns an explicit outcome rather than panicking.
    #[must_use]
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

    /// Select parser backend based on feature flags and grammar metadata.
    ///
    /// # Arguments
    ///
    /// * `has_conflicts` - Whether the grammar contains shift/reduce or reduce/reduce conflicts.
    ///
    /// # Panics
    ///
    /// Panics if `has_conflicts` is true and the `pure-rust` feature is enabled without the `glr` feature.
    #[must_use]
    pub const fn select(has_conflicts: bool) -> Self {
        match Self::select_contract(has_conflicts) {
            ParserBackendSelection::Backend(backend) => backend,
            ParserBackendSelection::ConflictsRequireGlr => {
                panic!("{}", CONFLICTS_REQUIRE_GLR_MESSAGE)
            }
        }
    }

    /// Whether this backend is the GLR parser.
    #[must_use]
    pub const fn is_glr(self) -> bool {
        matches!(self, Self::GLR)
    }

    /// Whether this backend is a pure-Rust parser (LR or GLR).
    #[must_use]
    pub const fn is_pure_rust(self) -> bool {
        matches!(self, Self::PureRust | Self::GLR)
    }

    /// Human-readable backend name.
    #[must_use]
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
    fn facade_reexports_profile_and_backend() {
        let profile = ParserFeatureProfile::current();
        let backend = profile.resolve_backend(false);
        assert!(matches!(
            backend,
            ParserBackend::TreeSitter | ParserBackend::PureRust | ParserBackend::GLR
        ));
    }

    #[test]
    fn display_values_are_stable() {
        assert_eq!(
            ParserBackend::TreeSitter.to_string(),
            "tree-sitter C runtime"
        );
        assert_eq!(ParserBackend::PureRust.to_string(), "pure-Rust LR parser");
        assert_eq!(ParserBackend::GLR.to_string(), "pure-Rust GLR parser");
    }

    #[test]
    fn backend_name_returns_human_readable_string() {
        assert_eq!(ParserBackend::TreeSitter.name(), "tree-sitter C runtime");
        assert_eq!(ParserBackend::PureRust.name(), "pure-Rust LR parser");
        assert_eq!(ParserBackend::GLR.name(), "pure-Rust GLR parser");
    }

    #[test]
    fn backend_is_glr_only_for_glr() {
        assert!(ParserBackend::GLR.is_glr());
        assert!(!ParserBackend::PureRust.is_glr());
        assert!(!ParserBackend::TreeSitter.is_glr());
    }

    #[test]
    fn backend_is_pure_rust_for_lr_and_glr() {
        assert!(ParserBackend::PureRust.is_pure_rust());
        assert!(ParserBackend::GLR.is_pure_rust());
        assert!(!ParserBackend::TreeSitter.is_pure_rust());
    }

    #[test]
    fn backend_select_matches_feature_contract() {
        #[cfg(feature = "glr")]
        {
            assert_eq!(ParserBackend::select(false), ParserBackend::GLR);
            assert_eq!(ParserBackend::select(true), ParserBackend::GLR);
        }

        #[cfg(all(feature = "pure-rust", not(feature = "glr")))]
        {
            assert_eq!(ParserBackend::select(false), ParserBackend::PureRust);
            assert!(std::panic::catch_unwind(|| ParserBackend::select(true)).is_err());
        }

        #[cfg(not(any(feature = "pure-rust", feature = "glr")))]
        {
            assert_eq!(ParserBackend::select(false), ParserBackend::TreeSitter);
            assert_eq!(ParserBackend::select(true), ParserBackend::TreeSitter);
        }
    }

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
    fn backend_selection_contract_reports_conflict_requirement() {
        let profile = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert_eq!(
            profile.backend_selection_contract(true),
            ParserBackendSelection::ConflictsRequireGlr
        );
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
