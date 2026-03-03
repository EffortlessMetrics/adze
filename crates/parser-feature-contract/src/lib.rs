//! Shared contracts for parser backend selection.
//!
//! This crate intentionally has a narrow surface: it centralizes feature-based
//! backend policy decisions that should stay synchronized across runtime and tests.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_feature_policy_contract::{ParserBackend, ParserFeatureProfile};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_equality() {
        assert_eq!(ParserBackend::TreeSitter, ParserBackend::TreeSitter);
        assert_ne!(ParserBackend::TreeSitter, ParserBackend::GLR);
        assert_ne!(ParserBackend::PureRust, ParserBackend::GLR);
    }

    #[test]
    fn backend_properties() {
        assert!(ParserBackend::GLR.is_glr());
        assert!(ParserBackend::GLR.is_pure_rust());
        assert!(ParserBackend::PureRust.is_pure_rust());
        assert!(!ParserBackend::TreeSitter.is_pure_rust());
    }

    #[test]
    fn backend_select() {
        let backend = ParserBackend::select(false);
        let _ = backend.name();
    }

    #[test]
    fn profile_current_is_consistent() {
        let a = ParserFeatureProfile::current();
        let b = ParserFeatureProfile::current();
        assert_eq!(a, b);
    }

    #[test]
    fn profile_display_nonempty() {
        let profile = ParserFeatureProfile::current();
        let display = format!("{}", profile);
        assert!(!display.is_empty());
    }

    #[test]
    fn profile_resolve_backend() {
        let profile = ParserFeatureProfile::current();
        let b1 = profile.resolve_backend(false);
        let b2 = profile.resolve_backend(true);
        let _ = b1.name();
        let _ = b2.name();
    }
}
