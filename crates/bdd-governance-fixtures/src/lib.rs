//! BDD reporting helpers composed with parser feature profile contracts.
//!
//! This crate keeps feature-flagged reporting behavior available for tests and
//! fixtures while leaving governance rules in the underlying contract crate.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_governance_runtime_reporting::bdd_progress_report_with_profile_runtime;

/// Re-exported progress constants and helpers from the shared BDD grid contracts.
pub use adze_bdd_governance_contract::{
    BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend,
    ParserFeatureProfile, bdd_progress, bdd_progress_report, bdd_progress_report_with_profile,
    bdd_progress_status_line,
};

/// BDD progress report using the current compile-time parser profile.
pub fn bdd_progress_report_for_current_profile(phase: BddPhase, phase_title: &str) -> String {
    bdd_progress_report_with_profile_runtime(
        phase,
        GLR_CONFLICT_PRESERVATION_GRID,
        phase_title,
        ParserFeatureProfile::current(),
    )
}

/// BDD status line using the current compile-time parser profile.
pub fn bdd_progress_status_line_for_current_profile(phase: BddPhase) -> String {
    let profile = ParserFeatureProfile::current();
    bdd_progress_status_line(phase, GLR_CONFLICT_PRESERVATION_GRID, profile)
}
