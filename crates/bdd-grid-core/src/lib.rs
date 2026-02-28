//! Compatibility façade for BDD scenario-grid contracts and fixtures.
//!
//! This crate keeps the long-standing `adze_bdd_grid_core` import path while the
//! responsibilities are split into SRP microcrates:
//! - `adze-bdd-grid-contract` for scenario types and reporting logic
//! - `adze-bdd-grid-fixtures` for concrete scenario datasets

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_bdd_grid_contract::{
    BddPhase, BddScenario, BddScenarioStatus, bdd_progress, bdd_progress_report,
};
pub use adze_bdd_grid_fixtures::GLR_CONFLICT_PRESERVATION_GRID;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_summary_reports_counts() {
        let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
        assert_eq!(implemented, 6);
        assert_eq!(total, 8);
    }

    #[test]
    fn progress_report_text_includes_title() {
        let report =
            bdd_progress_report(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, "Runtime");
        assert!(report.contains("Runtime"));
        assert!(report.contains("Scenario 1"));
    }
}
