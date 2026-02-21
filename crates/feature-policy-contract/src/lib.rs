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
