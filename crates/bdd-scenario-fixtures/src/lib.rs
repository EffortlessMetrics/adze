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
