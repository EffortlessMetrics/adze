//! Runtime-targeted parser feature-profile helpers.
//!
//! This microcrate owns runtime/runtime2-specific profile composition so
//! governance reporting crates can depend on one focused contract.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};

/// Return the compile-time parser feature profile for the runtime crate.
#[must_use]
pub const fn parser_feature_profile_for_runtime() -> ParserFeatureProfile {
    ParserFeatureProfile::current()
}

/// Return a parser profile equivalent to the runtime2 `pure-rust-glr` toggle.
#[must_use]
pub const fn parser_feature_profile_for_runtime2(pure_rust_glr: bool) -> ParserFeatureProfile {
    ParserFeatureProfile {
        pure_rust: pure_rust_glr,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: pure_rust_glr,
    }
}

/// Resolve a backend using an explicit profile.
#[must_use]
pub const fn resolve_backend_for_profile(
    profile: ParserFeatureProfile,
    has_conflicts: bool,
) -> ParserBackend {
    profile.resolve_backend(has_conflicts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_profile_matches_current_cfg() {
        let profile = parser_feature_profile_for_runtime();
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
    fn runtime2_profile_reflects_glr_toggle() {
        let enabled = parser_feature_profile_for_runtime2(true);
        assert!(enabled.pure_rust);
        assert!(enabled.glr);
        assert!(!enabled.tree_sitter_standard);
        assert!(!enabled.tree_sitter_c2rust);

        let disabled = parser_feature_profile_for_runtime2(false);
        assert!(!disabled.pure_rust);
        assert!(!disabled.glr);
    }

    #[test]
    fn resolve_backend_for_profile_delegates_correctly() {
        let profile = parser_feature_profile_for_runtime2(true);
        let backend = resolve_backend_for_profile(profile, false);
        assert_eq!(backend, profile.resolve_backend(false));

        let conflict_backend = resolve_backend_for_profile(profile, true);
        assert_eq!(conflict_backend, profile.resolve_backend(true));
    }
}
