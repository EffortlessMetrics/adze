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
/// use adze_parser_feature_profile_core::ParserFeatureProfile;
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
    /// use adze_parser_feature_profile_core::{ParserFeatureProfile, ParserBackend};
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

    // --- Feature flag detection ---

    #[test]
    fn has_pure_rust_true_when_flag_set() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert!(p.has_pure_rust());
    }

    #[test]
    fn has_pure_rust_false_when_flag_unset() {
        let p = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: true,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert!(!p.has_pure_rust());
    }

    #[test]
    fn has_glr_true_when_flag_set() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: true,
        };
        assert!(p.has_glr());
    }

    #[test]
    fn has_glr_false_when_flag_unset() {
        let p = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert!(!p.has_glr());
    }

    #[test]
    fn has_tree_sitter_with_standard_only() {
        let p = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: true,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert!(p.has_tree_sitter());
    }

    #[test]
    fn has_tree_sitter_with_c2rust_only() {
        let p = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: false,
            tree_sitter_c2rust: true,
            glr: false,
        };
        assert!(p.has_tree_sitter());
    }

    #[test]
    fn has_tree_sitter_with_both_backends() {
        let p = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: true,
            tree_sitter_c2rust: true,
            glr: false,
        };
        assert!(p.has_tree_sitter());
    }

    #[test]
    fn has_tree_sitter_false_without_either() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert!(!p.has_tree_sitter());
    }

    // --- Policy evaluation (resolve_backend) ---

    #[test]
    fn resolve_backend_glr_takes_priority_over_all() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: true,
            tree_sitter_c2rust: true,
            glr: true,
        };
        assert_eq!(p.resolve_backend(false), ParserBackend::GLR);
        assert_eq!(p.resolve_backend(true), ParserBackend::GLR);
    }

    #[test]
    fn resolve_backend_pure_rust_without_conflicts() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert_eq!(p.resolve_backend(false), ParserBackend::PureRust);
    }

    #[test]
    #[should_panic(expected = "GLR feature is not enabled")]
    fn resolve_backend_pure_rust_with_conflicts_panics() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        let _ = p.resolve_backend(true);
    }

    #[test]
    fn resolve_backend_tree_sitter_fallback() {
        let p = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: true,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert_eq!(p.resolve_backend(false), ParserBackend::TreeSitter);
        assert_eq!(p.resolve_backend(true), ParserBackend::TreeSitter);
    }

    #[test]
    fn resolve_backend_glr_handles_conflicts() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: true,
        };
        assert_eq!(p.resolve_backend(true), ParserBackend::GLR);
    }

    // --- Feature compatibility matrix ---

    #[test]
    fn compatibility_matrix_all_sixteen_combinations() {
        for bits in 0u8..16 {
            let p = ParserFeatureProfile {
                pure_rust: bits & 0b1000 != 0,
                tree_sitter_standard: bits & 0b0100 != 0,
                tree_sitter_c2rust: bits & 0b0010 != 0,
                glr: bits & 0b0001 != 0,
            };

            // has_pure_rust tracks its own field
            assert_eq!(p.has_pure_rust(), p.pure_rust);
            // has_glr tracks its own field
            assert_eq!(p.has_glr(), p.glr);
            // has_tree_sitter is OR of the two TS backends
            assert_eq!(
                p.has_tree_sitter(),
                p.tree_sitter_standard || p.tree_sitter_c2rust
            );

            // resolve_backend priority: glr > pure_rust > tree_sitter
            if p.glr {
                assert_eq!(p.resolve_backend(false), ParserBackend::GLR);
                assert_eq!(p.resolve_backend(true), ParserBackend::GLR);
            } else if p.pure_rust {
                assert_eq!(p.resolve_backend(false), ParserBackend::PureRust);
            } else {
                assert_eq!(p.resolve_backend(false), ParserBackend::TreeSitter);
                assert_eq!(p.resolve_backend(true), ParserBackend::TreeSitter);
            }
        }
    }

    // --- Default feature set (no features enabled) ---

    #[test]
    fn no_features_profile() {
        let p = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert!(!p.has_pure_rust());
        assert!(!p.has_glr());
        assert!(!p.has_tree_sitter());
        assert_eq!(p.resolve_backend(false), ParserBackend::TreeSitter);
        assert_eq!(p.resolve_backend(true), ParserBackend::TreeSitter);
    }

    // --- Feature intersection/union operations ---

    /// Helper: compute the union of two profiles (any flag set in either).
    fn profile_union(a: &ParserFeatureProfile, b: &ParserFeatureProfile) -> ParserFeatureProfile {
        ParserFeatureProfile {
            pure_rust: a.pure_rust || b.pure_rust,
            tree_sitter_standard: a.tree_sitter_standard || b.tree_sitter_standard,
            tree_sitter_c2rust: a.tree_sitter_c2rust || b.tree_sitter_c2rust,
            glr: a.glr || b.glr,
        }
    }

    /// Helper: compute the intersection of two profiles (flag set in both).
    fn profile_intersect(
        a: &ParserFeatureProfile,
        b: &ParserFeatureProfile,
    ) -> ParserFeatureProfile {
        ParserFeatureProfile {
            pure_rust: a.pure_rust && b.pure_rust,
            tree_sitter_standard: a.tree_sitter_standard && b.tree_sitter_standard,
            tree_sitter_c2rust: a.tree_sitter_c2rust && b.tree_sitter_c2rust,
            glr: a.glr && b.glr,
        }
    }

    #[test]
    fn union_of_disjoint_profiles() {
        let ts = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: true,
            tree_sitter_c2rust: false,
            glr: false,
        };
        let pr = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        let merged = profile_union(&ts, &pr);
        assert!(merged.has_pure_rust());
        assert!(merged.has_tree_sitter());
        assert!(!merged.has_glr());
    }

    #[test]
    fn union_with_glr_enables_glr() {
        let base = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        let glr_only = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: true,
        };
        let merged = profile_union(&base, &glr_only);
        assert!(merged.has_glr());
        assert!(merged.has_pure_rust());
        assert_eq!(merged.resolve_backend(true), ParserBackend::GLR);
    }

    #[test]
    fn intersection_of_disjoint_profiles_is_empty() {
        let a = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        let b = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: true,
            tree_sitter_c2rust: false,
            glr: false,
        };
        let common = profile_intersect(&a, &b);
        assert!(!common.has_pure_rust());
        assert!(!common.has_tree_sitter());
        assert!(!common.has_glr());
    }

    #[test]
    fn intersection_of_overlapping_profiles() {
        let a = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: true,
            tree_sitter_c2rust: false,
            glr: true,
        };
        let b = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: true,
            glr: true,
        };
        let common = profile_intersect(&a, &b);
        assert!(common.has_pure_rust());
        assert!(common.has_glr());
        assert!(!common.has_tree_sitter());
    }

    #[test]
    fn union_with_self_is_identity() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: true,
            glr: false,
        };
        assert_eq!(profile_union(&p, &p), p);
    }

    #[test]
    fn intersection_with_self_is_identity() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: true,
            glr: false,
        };
        assert_eq!(profile_intersect(&p, &p), p);
    }

    // --- Edge cases ---

    #[test]
    fn all_features_profile() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: true,
            tree_sitter_c2rust: true,
            glr: true,
        };
        assert!(p.has_pure_rust());
        assert!(p.has_glr());
        assert!(p.has_tree_sitter());
        // GLR takes priority regardless of other flags
        assert_eq!(p.resolve_backend(false), ParserBackend::GLR);
        assert_eq!(p.resolve_backend(true), ParserBackend::GLR);
    }

    #[test]
    fn display_no_features() {
        let p = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert_eq!(p.to_string(), "none");
    }

    #[test]
    fn display_single_feature() {
        let cases = [
            (true, false, false, false, "pure-rust"),
            (false, true, false, false, "tree-sitter-standard"),
            (false, false, true, false, "tree-sitter-c2rust"),
            (false, false, false, true, "glr"),
        ];
        for (pr, tss, tsc, glr, expected) in cases {
            let p = ParserFeatureProfile {
                pure_rust: pr,
                tree_sitter_standard: tss,
                tree_sitter_c2rust: tsc,
                glr,
            };
            assert_eq!(
                p.to_string(),
                expected,
                "single-feature display for {expected}"
            );
        }
    }

    #[test]
    fn display_all_features() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: true,
            tree_sitter_c2rust: true,
            glr: true,
        };
        let s = p.to_string();
        assert!(s.contains("pure-rust"));
        assert!(s.contains("tree-sitter-standard"));
        assert!(s.contains("tree-sitter-c2rust"));
        assert!(s.contains("glr"));
    }

    #[test]
    fn display_comma_separation() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: true,
        };
        assert_eq!(p.to_string(), "pure-rust, glr");
    }

    #[test]
    fn profile_equality() {
        let a = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: true,
        };
        let b = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: true,
        };
        let c = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: true,
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn profile_clone_and_copy() {
        let a = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: true,
            glr: false,
        };
        let b = a;
        #[allow(clippy::clone_on_copy)]
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn profile_debug_contains_field_names() {
        let p = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        let dbg = format!("{p:?}");
        assert!(dbg.contains("pure_rust: true"));
        assert!(dbg.contains("glr: false"));
    }

    #[test]
    fn profile_hash_consistent() {
        use std::collections::HashSet;
        let p1 = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        let p2 = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        let p3 = ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: true,
        };
        let mut set = HashSet::new();
        set.insert(p1);
        set.insert(p2);
        set.insert(p3);
        assert_eq!(set.len(), 2, "equal profiles should hash to same bucket");
    }
}
