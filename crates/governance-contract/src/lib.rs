//! Shared governance contracts for parser backend selection and BDD progress tracking.
//!
//! This crate is a compatibility facade used by the existing workspace,
//! while the concrete contracts live in `adze-parser-governance-contract`.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]
pub use adze_parser_governance_contract::{
    BddGovernanceMatrix, BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID,
    ParserBackend, ParserFeatureProfile, bdd_progress, bdd_progress_report,
    bdd_progress_report_with_profile, bdd_progress_status_line,
};
