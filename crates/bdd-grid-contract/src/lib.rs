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
    BddGridIssue, BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID,
    bdd_grid_issues, bdd_progress, bdd_progress_report,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bdd_phase_display() {
        let core_str = format!("{}", BddPhase::Core);
        let runtime_str = format!("{}", BddPhase::Runtime);
        assert!(!core_str.is_empty());
        assert!(!runtime_str.is_empty());
        assert_ne!(core_str, runtime_str);
    }

    #[test]
    fn scenario_status_label_and_detail() {
        let implemented = BddScenarioStatus::Implemented;
        assert!(!implemented.label().is_empty());
        assert_eq!(implemented.detail(), "");

        let deferred = BddScenarioStatus::Deferred { reason: "wip" };
        assert!(!deferred.label().is_empty());
        assert_eq!(deferred.detail(), "wip");
    }

    #[test]
    fn scenario_status_for_phase() {
        for scenario in GLR_CONFLICT_PRESERVATION_GRID {
            let core_status = scenario.status(BddPhase::Core);
            let runtime_status = scenario.status(BddPhase::Runtime);
            // Both should return valid statuses
            let _ = core_status.icon();
            let _ = runtime_status.icon();
        }
    }

    #[test]
    fn bdd_progress_counts_match_grid() {
        let (core_impl, core_total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
        let (rt_impl, rt_total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);
        // Total should be the same for both phases (same grid)
        assert_eq!(core_total, rt_total);
        assert_eq!(core_total, GLR_CONFLICT_PRESERVATION_GRID.len());
        assert!(core_impl <= core_total);
        assert!(rt_impl <= rt_total);
    }

    #[test]
    fn scenario_display_contains_title() {
        for scenario in GLR_CONFLICT_PRESERVATION_GRID {
            let display = format!("{}", scenario);
            assert!(display.contains(scenario.title));
        }
    }

    #[test]
    fn scenario_debug_impl() {
        for scenario in GLR_CONFLICT_PRESERVATION_GRID {
            let debug = format!("{:?}", scenario);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn canonical_grid_integrity_is_stable() {
        assert!(bdd_grid_issues(GLR_CONFLICT_PRESERVATION_GRID).is_empty());
    }
}
