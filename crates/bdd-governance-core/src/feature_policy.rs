//! Parser backend selection and feature profile contracts for governance reports.

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
/// use adze_bdd_governance_core::ParserBackend;
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
