//! Shared BDD contracts for scenario tracking and feature-matrix summaries.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_bdd_grid_contract::{
    BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress,
    bdd_progress_report,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bdd_phase_debug_impl() {
        assert_eq!(format!("{:?}", BddPhase::Core), "Core");
        assert_eq!(format!("{:?}", BddPhase::Runtime), "Runtime");
    }

    #[test]
    fn bdd_phase_clone_and_eq() {
        let phase = BddPhase::Core;
        let cloned = phase;
        assert_eq!(phase, cloned);
    }

    #[test]
    fn bdd_scenario_status_implemented() {
        let status = BddScenarioStatus::Implemented;
        assert!(status.implemented());
        assert_eq!(status.icon(), "✅");
    }

    #[test]
    fn bdd_scenario_status_deferred() {
        let status = BddScenarioStatus::Deferred {
            reason: "not yet ready",
        };
        assert!(!status.implemented());
        assert!(!status.icon().is_empty());
    }

    #[test]
    fn grid_constant_has_scenarios() {
        assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
        for scenario in GLR_CONFLICT_PRESERVATION_GRID {
            assert!(!scenario.title.is_empty());
            assert!(!scenario.reference.is_empty());
        }
    }

    #[test]
    fn bdd_progress_empty_scenarios() {
        let (implemented, total) = bdd_progress(BddPhase::Core, &[]);
        assert_eq!(implemented, 0);
        assert_eq!(total, 0);
    }

    #[test]
    fn bdd_progress_report_contains_title() {
        let report =
            bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Test Title");
        assert!(report.contains("Test Title"));
    }

    #[test]
    fn bdd_progress_report_empty_scenarios() {
        let report = bdd_progress_report(BddPhase::Core, &[], "Empty");
        assert!(report.contains("Empty"));
        assert!(report.contains("0/0"));
    }
}
