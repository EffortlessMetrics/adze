//! Core parser feature-policy contracts shared by governance layers.
//!
//! This crate is now a facade over focused SRP microcrates:
//! - [`adze_parser_backend_core`] for backend enum/selection semantics.
//! - [`adze_parser_feature_profile_core`] for feature-profile snapshots and resolution.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_parser_backend_core::ParserBackend;
pub use adze_parser_feature_profile_core::ParserFeatureProfile;
