//! Shared contracts for parser backend selection and BDD scenario tracking.
//!
//! This crate remains the compatibility facade used by the existing workspace,
//! while the concrete contracts live in `adze-governance-contract`.

#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
pub use adze_governance_contract::{
    BddGovernanceMatrix, BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID,
    ParserBackend, ParserFeatureProfile, bdd_progress, bdd_progress_report,
};
