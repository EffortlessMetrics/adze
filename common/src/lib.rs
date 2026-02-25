// Common crate is pure-Rust - no unsafe needed
#![forbid(unsafe_code)]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

//! Shared utility entrypoint for macro and tool parsing behavior.

pub use adze_common_syntax_core::*;
