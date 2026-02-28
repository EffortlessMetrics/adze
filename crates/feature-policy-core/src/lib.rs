//! Core contracts for parser backend selection and feature profiles.
//!
//! This crate intentionally owns only policy semantics so governance and fixture
//! crates can share behavior without inheriting extra dependencies.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Re-exported parser backend enum for feature-profile–based backend resolution.
pub use adze_parser_backend_core::ParserBackend;

use core::fmt::{self, Display, Formatter};

/// Snapshot of parser-related feature flags for this build.
///
/// # Examples
///
/// ```
/// use adze_feature_policy_core::ParserFeatureProfile;
///
/// let profile = ParserFeatureProfile {
///     pure_rust: true,
///     tree_sitter_standard: false,
///     tree_sitter_c2rust: false,
///     glr: false,
/// };
/// assert!(profile.has_pure_rust());
/// assert!(!profile.has_glr());
/// assert!(!profile.has_tree_sitter());
/// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use adze_feature_policy_core::{ParserFeatureProfile, ParserBackend};
    ///
    /// let profile = ParserFeatureProfile {
    ///     pure_rust: true, tree_sitter_standard: false,
    ///     tree_sitter_c2rust: false, glr: false,
    /// };
    /// assert_eq!(profile.resolve_backend(false), ParserBackend::PureRust);
    /// ```
    #[must_use]
    pub const fn resolve_backend(self, has_conflicts: bool) -> ParserBackend {
        if self.glr {
            ParserBackend::GLR
        } else if self.pure_rust {
            if has_conflicts {
                panic!(
                    "{}",
                    "Grammar has shift/reduce or reduce/reduce conflicts, but the GLR feature is not enabled.\n\n\
To fix this, enable the GLR feature in Cargo.toml:\n\n\
[dependencies]\n\
adze = { version = \"0.8\", features = [\"glr\"] }\n\n\
Or use the tree-sitter C runtime (default):\n\n\
[dependencies]\n\
adze = \"0.8\"\n"
                );
            }
            ParserBackend::PureRust
        } else {
            ParserBackend::TreeSitter
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
    fn backend_predicates_work() {
        #[cfg(feature = "glr")]
        {
            let backend = ParserBackend::GLR;
            assert!(backend.is_glr());
            assert!(backend.is_pure_rust());
        }

        #[cfg(all(feature = "pure-rust", not(feature = "glr")))]
        {
            let backend = ParserBackend::PureRust;
            assert!(!backend.is_glr());
            assert!(backend.is_pure_rust());
        }

        #[cfg(not(any(feature = "pure-rust", feature = "glr")))]
        {
            let backend = ParserBackend::TreeSitter;
            assert!(!backend.is_glr());
            assert!(!backend.is_pure_rust());
        }
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
    fn resolve_backend_from_profile_matches_contract() {
        let profile = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert_eq!(profile.resolve_backend(false), ParserBackend::PureRust);
    }
}
