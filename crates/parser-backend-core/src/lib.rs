//! Core parser backend selection primitives.

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
        #[cfg(feature = "glr")]
        {
            return Self::GLR;
        }

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
