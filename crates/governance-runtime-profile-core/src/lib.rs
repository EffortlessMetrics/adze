//! Runtime parser-profile and backend-selection primitives.
//!
//! This crate isolates feature-profile construction and backend resolution so
//! higher-level governance crates can compose reporting without coupling to
//! runtime/runtime2 profile wiring details.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_governance_runtime_reporting::{ParserBackend, ParserFeatureProfile};

/// Return the compile-time parser feature profile for the runtime crate.
pub const fn parser_feature_profile_for_runtime() -> ParserFeatureProfile {
    ParserFeatureProfile::current()
}

/// Return a parser profile equivalent to the runtime2 `pure-rust-glr` toggle.
pub const fn parser_feature_profile_for_runtime2(pure_rust_glr: bool) -> ParserFeatureProfile {
    ParserFeatureProfile {
        pure_rust: pure_rust_glr,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: pure_rust_glr,
    }
}

/// Resolve a backend using an explicit profile.
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
        assert_eq!(
            parser_feature_profile_for_runtime().pure_rust,
            cfg!(feature = "pure-rust")
        );
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
    fn backend_resolution_matches_profile() {
        let profile = parser_feature_profile_for_runtime2(true);
        assert_eq!(
            resolve_backend_for_profile(profile, false),
            profile.resolve_backend(false)
        );
    }
}
