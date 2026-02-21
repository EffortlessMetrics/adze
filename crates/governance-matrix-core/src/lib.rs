//! Compatibility facade for the governance matrix core implementation.
//!
//! The actual implementation now lives in `adze-governance-matrix-core-impl` so
//! façade crates can keep this historical crate name while downstream users are
//! insulated from package reshuffling.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_governance_matrix_core_impl::*;
