//! Compatibility façade for BDD scenario fixtures.
//!
//! This crate preserves the existing public API while splitting fixture
//! responsibilities into two focused microcrates:
//! - `adze-bdd-grammar-fixtures`: grammar tables, conflict analysis, and token metadata
//! - `adze-bdd-governance-fixtures`: BDD reporting and profile helpers

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_bdd_governance_fixtures::*;
pub use adze_bdd_grammar_fixtures::*;
