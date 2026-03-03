//! Compatibility facade for the extracted BDD governance core implementation.
//!
//! The concrete implementation now lives in `adze-bdd-governance-core`.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_bdd_governance_core::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_exports_bdd_phase() {
        let core = BddPhase::Core;
        let runtime = BddPhase::Runtime;
        assert_ne!(core, runtime);
    }

    #[test]
    fn facade_exports_governance_snapshot() {
        let snap = BddGovernanceSnapshot {
            phase: BddPhase::Core,
            implemented: 0,
            total: 0,
            profile: ParserFeatureProfile::current(),
        };
        assert!(snap.is_fully_implemented()); // 0/0 is considered fully implemented
    }

    #[test]
    fn facade_exports_governance_matrix() {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        let snap = matrix.snapshot();
        assert_eq!(snap.profile, profile);
    }

    #[test]
    fn facade_exports_glr_conflict_fallback() {
        assert!(!GLR_CONFLICT_FALLBACK.is_empty());
    }

    #[test]
    fn facade_exports_describe_backend() {
        let profile = ParserFeatureProfile::current();
        let desc = describe_backend_for_conflicts(profile);
        assert!(!desc.is_empty());
    }

    #[test]
    fn facade_exports_bdd_progress_report() {
        let report =
            bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Impl Test");
        assert!(report.contains("Impl Test"));
    }

    #[test]
    fn facade_exports_bdd_progress_report_with_profile() {
        let profile = ParserFeatureProfile::current();
        let report = bdd_progress_report_with_profile(
            BddPhase::Runtime,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Profile Report",
            profile,
        );
        assert!(report.contains("Profile Report"));
    }

    #[test]
    fn facade_exports_bdd_progress_status_line() {
        let profile = ParserFeatureProfile::current();
        let status =
            bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
        assert!(status.starts_with("core:"));
    }

    #[test]
    fn facade_snapshot_is_fully_implemented_variations() {
        let full = BddGovernanceSnapshot {
            phase: BddPhase::Core,
            implemented: 5,
            total: 5,
            profile: ParserFeatureProfile::current(),
        };
        assert!(full.is_fully_implemented());

        let partial = BddGovernanceSnapshot {
            phase: BddPhase::Runtime,
            implemented: 2,
            total: 5,
            profile: ParserFeatureProfile::current(),
        };
        assert!(!partial.is_fully_implemented());
    }

    #[test]
    fn facade_matrix_new_constructor() {
        let profile = ParserFeatureProfile::current();
        let matrix =
            BddGovernanceMatrix::new(BddPhase::Core, profile, GLR_CONFLICT_PRESERVATION_GRID);
        assert_eq!(matrix.phase, BddPhase::Core);
        assert_eq!(matrix.profile, profile);
    }

    #[test]
    fn facade_glr_conflict_fallback_non_empty() {
        assert!(!GLR_CONFLICT_FALLBACK.is_empty());
    }
}
