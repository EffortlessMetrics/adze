//! Core contracts for parser backend selection and feature profiles.
//!
//! This compatibility facade now re-exports the dedicated single-responsibility
//! parser-feature-profile microcrate.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_parser_backend_core::ParserBackend;
pub use adze_parser_feature_profile_core::ParserFeatureProfile;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_reexports_profile_and_backend() {
        let profile = ParserFeatureProfile::current();
        let backend = profile.resolve_backend(false);
        assert!(matches!(
            backend,
            ParserBackend::TreeSitter | ParserBackend::PureRust | ParserBackend::GLR
        ));
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
}
