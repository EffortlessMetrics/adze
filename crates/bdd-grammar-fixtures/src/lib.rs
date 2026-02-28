//! Compatibility façade for GLR BDD fixture helpers.
//!
//! This crate preserves the legacy API while delegating to SRP microcrates:
//! - `adze-bdd-grammar-core` for fixture grammar/token/symbol definitions.
//! - `adze-bdd-parse-fixtures-core` for parse-table building and conflict analysis.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_bdd_grammar_core::*;
pub use adze_bdd_parse_fixtures_core::*;
