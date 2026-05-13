//! Compatibility facade for BDD scenario fixtures.
//!
//! This crate preserves the existing public API while splitting fixture
//! responsibilities into two focused microcrates:
//! - `adze-bdd-grammar-fixtures`: grammar tables, conflict analysis, and token metadata
//! - `adze-bdd-governance-core`: BDD reporting and profile helpers

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_governance_runtime_reporting::bdd_progress_report_with_profile_runtime;

pub use adze_bdd_governance_core::{
    BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend,
    ParserFeatureProfile, bdd_progress, bdd_progress_report, bdd_progress_report_with_profile,
    bdd_progress_status_line,
};
pub use adze_bdd_grammar_fixtures::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexports_bdd_phase_variants() {
        let _core = BddPhase::Core;
        let _runtime = BddPhase::Runtime;
    }

    #[test]
    fn reexports_scenario_status() {
        let status = BddScenarioStatus::Implemented;
        assert!(status.implemented());
    }

    #[test]
    fn reexports_grid_constant() {
        assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
    }

    #[test]
    fn reexports_bdd_progress_fn() {
        let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
        assert!(implemented <= total);
    }

    #[test]
    fn reexports_profile_functions() {
        let report = bdd_progress_report_for_current_profile(BddPhase::Core, "Fixture Test");
        assert!(report.contains("Fixture Test"));
    }

    #[test]
    fn reexports_parser_feature_profile() {
        let profile = ParserFeatureProfile::current();
        let _ = format!("{:?}", profile);
    }
}
