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

use core::fmt::{self, Display, Formatter};

/// Parser backend supported by the runtime feature matrix.
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
    pub const fn select(_has_conflicts: bool) -> Self {
        // Priority 1: GLR feature explicitly enabled.
        #[cfg(feature = "glr")]
        {
            return Self::GLR;
        }

        // Priority 2: Pure-Rust mode.
        #[cfg(all(feature = "pure-rust", not(feature = "glr")))]
        {
            if _has_conflicts {
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
            return Self::PureRust;
        }

        // Priority 3: Tree-sitter C runtime.
        #[cfg(all(
            not(feature = "pure-rust"),
            not(feature = "glr"),
            any(feature = "tree-sitter-standard", feature = "tree-sitter-c2rust")
        ))]
        {
            return Self::TreeSitter;
        }

        #[allow(unreachable_code)]
        {
            Self::TreeSitter
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

/// Snapshot of parser-related feature flags for this build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub const fn current() -> Self {
        Self {
            pure_rust: cfg!(feature = "pure-rust"),
            tree_sitter_standard: cfg!(feature = "tree-sitter-standard"),
            tree_sitter_c2rust: cfg!(feature = "tree-sitter-c2rust"),
            glr: cfg!(feature = "glr"),
        }
    }

    /// Resolve the effective backend from this profile.
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
    pub const fn has_pure_rust(self) -> bool {
        self.pure_rust
    }

    /// Whether feature flags indicate GLR is compiled in.
    pub const fn has_glr(self) -> bool {
        self.glr
    }

    /// Whether feature flags indicate any tree-sitter backend is compiled in.
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
