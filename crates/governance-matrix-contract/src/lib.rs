//! Compatibility façade for governance contracts combining BDD progress and parser
//! feature profiles.
//!
//! Concrete API lives in `adze-governance-matrix-core`; this crate preserves
//! existing crate paths while keeping surface behavior stable.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_governance_matrix_core::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus,
    GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile,
    bdd_governance_snapshot, bdd_progress, bdd_progress_report, bdd_progress_report_with_profile,
    bdd_progress_status_line, describe_backend_for_conflicts,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_debug_and_clone() {
        let profile = ParserFeatureProfile::current();
        let snap = bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
        let cloned = snap;
        assert_eq!(snap, cloned);
        let debug = format!("{:?}", snap);
        assert!(debug.contains("BddGovernanceSnapshot"));
    }

    #[test]
    fn snapshot_non_conflict_backend() {
        let snap = BddGovernanceSnapshot {
            phase: BddPhase::Core,
            implemented: 5,
            total: 5,
            profile: ParserFeatureProfile::current(),
        };
        let backend = snap.non_conflict_backend();
        let _ = backend.name();
    }

    #[test]
    fn glr_conflict_fallback_is_descriptive() {
        assert!(GLR_CONFLICT_FALLBACK.contains("GLR") || GLR_CONFLICT_FALLBACK.contains("glr"));
    }

    #[test]
    fn describe_backend_returns_string() {
        let desc = describe_backend_for_conflicts(ParserFeatureProfile::current());
        assert!(!desc.is_empty());
    }

    #[test]
    fn matrix_new_constructor() {
        let profile = ParserFeatureProfile::current();
        let matrix =
            BddGovernanceMatrix::new(BddPhase::Runtime, profile, GLR_CONFLICT_PRESERVATION_GRID);
        assert_eq!(matrix.phase, BddPhase::Runtime);
        assert_eq!(matrix.profile, profile);
    }

    #[test]
    fn matrix_snapshot_round_trip() {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        let snap = matrix.snapshot();
        assert_eq!(snap.phase, BddPhase::Core);
        assert_eq!(snap.total, GLR_CONFLICT_PRESERVATION_GRID.len());
    }

    #[test]
    fn matrix_status_line_nonempty() {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        assert!(!matrix.status_line().is_empty());
    }
}
