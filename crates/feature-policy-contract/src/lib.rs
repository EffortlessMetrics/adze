//! Compatibility façade for parser backend selection and feature profiles.
//!
//! This crate is intentionally thin and forwards to `adze-feature-policy-core` so
//! downstream users can depend on the historical crate name while code is shared in
//! the extracted policy core crate.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_backend_debug() {
        let ts = ParserBackend::TreeSitter;
        let pr = ParserBackend::PureRust;
        let glr = ParserBackend::GLR;
        assert_eq!(format!("{:?}", ts), "TreeSitter");
        assert_eq!(format!("{:?}", pr), "PureRust");
        assert_eq!(format!("{:?}", glr), "GLR");
    }

    #[test]
    fn parser_backend_display() {
        assert_eq!(
            format!("{}", ParserBackend::TreeSitter),
            "tree-sitter C runtime"
        );
        assert_eq!(
            format!("{}", ParserBackend::PureRust),
            "pure-Rust LR parser"
        );
        assert_eq!(format!("{}", ParserBackend::GLR), "pure-Rust GLR parser");
    }

    #[test]
    fn parser_backend_is_glr() {
        assert!(ParserBackend::GLR.is_glr());
        assert!(!ParserBackend::TreeSitter.is_glr());
        assert!(!ParserBackend::PureRust.is_glr());
    }

    #[test]
    fn parser_backend_is_pure_rust() {
        assert!(ParserBackend::GLR.is_pure_rust());
        assert!(ParserBackend::PureRust.is_pure_rust());
        assert!(!ParserBackend::TreeSitter.is_pure_rust());
    }

    #[test]
    fn parser_backend_name() {
        assert!(!ParserBackend::TreeSitter.name().is_empty());
        assert!(!ParserBackend::PureRust.name().is_empty());
        assert!(!ParserBackend::GLR.name().is_empty());
    }

    #[test]
    fn parser_backend_clone_eq() {
        let a = ParserBackend::GLR;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn feature_profile_current() {
        let profile = ParserFeatureProfile::current();
        let _ = format!("{:?}", profile);
        let _ = format!("{}", profile);
    }

    #[test]
    fn feature_profile_resolve_backend() {
        let profile = ParserFeatureProfile::current();
        let backend_no_conflicts = profile.resolve_backend(false);
        // Only test with conflicts=true if GLR is enabled, otherwise it panics
        if profile.has_glr() {
            let backend_with_conflicts = profile.resolve_backend(true);
            let _ = backend_with_conflicts.name();
        }
        // Verify the no-conflicts backend is valid
        let _ = backend_no_conflicts.name();
    }

    #[test]
    fn feature_profile_has_methods() {
        let profile = ParserFeatureProfile::current();
        // These should return booleans without panicking
        let _ = profile.has_pure_rust();
        let _ = profile.has_glr();
        let _ = profile.has_tree_sitter();
    }

    #[test]
    fn feature_profile_clone_eq() {
        let a = ParserFeatureProfile::current();
        let b = a;
        assert_eq!(a, b);
    }
}
