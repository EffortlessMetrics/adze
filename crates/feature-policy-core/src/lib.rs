//! Core contracts for parser backend selection and feature profiles.
//!
//! This crate now acts as a compatibility facade for focused microcrates.

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
    fn facade_profile_is_usable() {
        let profile = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        };
        assert_eq!(profile.resolve_backend(false), ParserBackend::PureRust);
    }

    #[test]
    fn facade_still_exposes_display_contract() {
        assert_eq!(
            ParserBackend::TreeSitter.to_string(),
            "tree-sitter C runtime"
        );
    }
}
