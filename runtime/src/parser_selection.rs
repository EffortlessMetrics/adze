//! Parser Backend Selection
//!
//! This module implements compile-time parser backend selection based on feature flags
//! and grammar metadata. It ensures that grammars with conflicts use the GLR parser
//! while simple grammars can use the more lightweight LR parser.
//!
//! ## Feature Flag Architecture
//!
//! - `default`: pure-rust (simple LR parser)
//! - `pure-rust`: Pure Rust LR parser (no conflicts allowed)
//! - `glr`: Pure Rust GLR parser (handles conflicts)
//! - `tree-sitter-standard`: Tree-sitter C runtime (stable)
//! - `tree-sitter-c2rust`: Tree-sitter C2Rust runtime (legacy)
//!
//! ## Grammar Metadata
//!
//! Each generated grammar includes `HAS_CONFLICTS: bool` metadata indicating
//! whether the grammar has any shift/reduce or reduce/reduce conflicts.
//!
//! ## Selection Logic
//!
//! 1. If `glr` feature enabled → Always use GLR parser
//! 2. If `pure-rust` (no glr) + has_conflicts → Panic with helpful error
//! 3. If `pure-rust` (no glr) + no conflicts → Use simple LR parser
//! 4. If tree-sitter features → Use Tree-sitter C runtime
//!
//! ## Related Documents
//!
//! - tests/features/glr_runtime_integration.feature - BDD scenarios
//! - docs/plans/GLR_RUNTIME_WIRING_PLAN.md - Implementation plan
//! - ARCHITECTURE_ISSUE_GLR_PARSER.md - Problem statement

/// Represents which parser backend to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserBackend {
    /// Tree-sitter C runtime (default when not using pure-rust)
    TreeSitter,

    /// Pure Rust LR parser (simple grammars without conflicts)
    PureRust,

    /// Pure Rust GLR parser (ambiguous grammars, handles conflicts)
    GLR,
}

impl ParserBackend {
    /// Select parser backend based on compile-time features and grammar metadata
    ///
    /// # Arguments
    /// * `has_conflicts` - Whether the grammar has shift/reduce or reduce/reduce conflicts
    ///
    /// # Returns
    /// The appropriate parser backend
    ///
    /// # Panics
    /// Panics if `has_conflicts=true` but GLR feature is not enabled
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Grammar without conflicts can use simple LR parser
    /// let backend = ParserBackend::select(false);
    /// assert_eq!(backend, ParserBackend::PureRust);
    ///
    /// // Grammar with conflicts requires GLR feature
    /// #[cfg(feature = "glr")]
    /// let backend = ParserBackend::select(true);
    /// assert_eq!(backend, ParserBackend::GLR);
    /// ```
    pub fn select(has_conflicts: bool) -> Self {
        // Priority 1: GLR feature explicitly enabled
        #[cfg(feature = "glr")]
        {
            return Self::GLR;
        }

        // Priority 2: Pure-rust mode (without GLR)
        #[cfg(all(feature = "pure-rust", not(feature = "glr")))]
        {
            if has_conflicts {
                panic!(
                    "Grammar has shift/reduce or reduce/reduce conflicts, but the GLR feature is not enabled.\n\
                     \n\
                     This grammar requires the GLR parser to handle ambiguities correctly.\n\
                     \n\
                     To fix this, enable the GLR feature in your Cargo.toml:\n\
                     \n\
                     [dependencies]\n\
                     adze = {{ version = \"0.7\", features = [\"glr\"] }}\n\
                     \n\
                     Or use the tree-sitter C runtime (default):\n\
                     \n\
                     [dependencies]\n\
                     adze = \"0.7\"\n\
                     \n\
                     See: ARCHITECTURE_ISSUE_GLR_PARSER.md for details"
                );
            }
            return Self::PureRust;
        }

        // Priority 3: Tree-sitter C runtime (default)
        #[cfg(all(not(feature = "pure-rust"), not(feature = "glr"),))]
        {
            return Self::TreeSitter;
        }

        // This should be unreachable due to feature flag configuration,
        // but provide a sensible default
        #[allow(unreachable_code)]
        {
            Self::TreeSitter
        }
    }

    /// Check if this backend is the GLR parser
    pub fn is_glr(&self) -> bool {
        matches!(self, Self::GLR)
    }

    /// Check if this backend is pure Rust (either LR or GLR)
    pub fn is_pure_rust(&self) -> bool {
        matches!(self, Self::PureRust | Self::GLR)
    }

    /// Get a human-readable name for this backend
    pub fn name(&self) -> &'static str {
        match self {
            Self::TreeSitter => "tree-sitter C runtime",
            Self::PureRust => "pure-Rust LR parser",
            Self::GLR => "pure-Rust GLR parser",
        }
    }
}

impl std::fmt::Display for ParserBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: GLR feature always selects GLR backend
    #[test]
    #[cfg(feature = "glr")]
    fn test_glr_feature_selects_glr_backend() {
        // With GLR feature, always use GLR parser regardless of conflicts
        assert_eq!(ParserBackend::select(false), ParserBackend::GLR);
        assert_eq!(ParserBackend::select(true), ParserBackend::GLR);
    }

    /// Test: Pure-rust without GLR works for conflict-free grammars
    #[test]
    #[cfg(all(feature = "pure-rust", not(feature = "glr")))]
    fn test_pure_rust_accepts_conflict_free_grammars() {
        // Should succeed for conflict-free grammars
        assert_eq!(ParserBackend::select(false), ParserBackend::PureRust);
    }

    /// Test: Pure-rust without GLR panics on conflicting grammars
    #[test]
    #[cfg(all(feature = "pure-rust", not(feature = "glr")))]
    #[should_panic(expected = "Grammar has shift/reduce")]
    fn test_pure_rust_rejects_conflicts() {
        // Should panic for conflicting grammars with helpful message
        ParserBackend::select(true);
    }

    /// Test: Default configuration uses tree-sitter
    #[test]
    #[cfg(not(any(feature = "pure-rust", feature = "glr")))]
    fn test_default_selects_tree_sitter() {
        // Without pure-rust or glr features, use tree-sitter C runtime
        assert_eq!(ParserBackend::select(false), ParserBackend::TreeSitter);
        assert_eq!(ParserBackend::select(true), ParserBackend::TreeSitter);
    }

    /// Test: Backend predicates work correctly
    #[test]
    fn test_backend_predicates() {
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

    /// Test: Display trait provides readable names
    #[test]
    fn test_display_trait() {
        assert_eq!(
            ParserBackend::TreeSitter.to_string(),
            "tree-sitter C runtime"
        );
        assert_eq!(ParserBackend::PureRust.to_string(), "pure-Rust LR parser");
        assert_eq!(ParserBackend::GLR.to_string(), "pure-Rust GLR parser");
    }
}
