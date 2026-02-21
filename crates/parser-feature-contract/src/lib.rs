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
