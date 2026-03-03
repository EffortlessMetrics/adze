//! Compatibility façade for parser governance reporting and feature profiles.
//!
//! The concrete implementation now lives in `adze-bdd-governance-core`.
//! This crate intentionally keeps existing public API paths stable for downstream users.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_bdd_governance_core::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus,
    GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile,
    bdd_governance_snapshot, bdd_progress, bdd_progress_report, bdd_progress_report_with_profile,
    bdd_progress_status_line, describe_backend_for_conflicts,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glr_conflict_fallback_is_nonempty() {
        assert!(!GLR_CONFLICT_FALLBACK.is_empty());
    }

    #[test]
    fn describe_backend_for_conflicts_returns_nonempty() {
        let profile = ParserFeatureProfile::current();
        let desc = describe_backend_for_conflicts(profile);
        assert!(!desc.is_empty());
    }

    #[test]
    fn governance_snapshot_from_grid() {
        let profile = ParserFeatureProfile::current();
        let snap = bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
        assert_eq!(snap.phase, BddPhase::Core);
        assert!(snap.total > 0);
        assert!(snap.implemented <= snap.total);
        assert_eq!(snap.profile, profile);
    }

    #[test]
    fn governance_snapshot_fully_implemented_check() {
        let snap = BddGovernanceSnapshot {
            phase: BddPhase::Core,
            implemented: 5,
            total: 5,
            profile: ParserFeatureProfile::current(),
        };
        assert!(snap.is_fully_implemented());
    }

    #[test]
    fn governance_snapshot_not_fully_implemented() {
        let snap = BddGovernanceSnapshot {
            phase: BddPhase::Runtime,
            implemented: 3,
            total: 5,
            profile: ParserFeatureProfile::current(),
        };
        assert!(!snap.is_fully_implemented());
    }

    #[test]
    fn governance_matrix_standard_constructor() {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        assert_eq!(matrix.profile, profile);
        assert!(!matrix.scenarios.is_empty());
    }

    #[test]
    fn governance_matrix_report_contains_title() {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        let report = matrix.report("Governance Test");
        assert!(report.contains("Governance Test"));
    }

    #[test]
    fn bdd_progress_status_line_nonempty() {
        let profile = ParserFeatureProfile::current();
        let line =
            bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
        assert!(!line.is_empty());
    }
}
