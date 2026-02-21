//! Compatibility façade for BDD scenario grid contracts.
//!
//! This crate forwards to `adze-bdd-grid-core` so existing import paths remain
//! stable while the scenario grid contract implementation lives in a focused
//! microcrate.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_bdd_grid_core::{
    BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress,
    bdd_progress_report,
};
