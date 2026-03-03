//! Compatibility facade for the governance matrix core implementation.
//!
//! The actual implementation now lives in `adze-governance-matrix-core-impl` so
//! façade crates can keep this historical crate name while downstream users are
//! insulated from package reshuffling.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_governance_matrix_core_impl::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_variants_accessible() {
        assert_ne!(BddPhase::Core, BddPhase::Runtime);
    }

    #[test]
    fn governance_matrix_standard_has_scenarios() {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        assert!(!matrix.scenarios.is_empty());
    }

    #[test]
    fn governance_matrix_is_fully_implemented_check() {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        // Just verify it runs without panic
        let _ = matrix.is_fully_implemented();
    }

    #[test]
    fn governance_matrix_report() {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        let report = matrix.report("Matrix Core Test");
        assert!(report.contains("Matrix Core Test"));
    }

    #[test]
    fn bdd_progress_on_empty_slice() {
        let (impl_count, total) = bdd_progress(BddPhase::Core, &[]);
        assert_eq!(impl_count, 0);
        assert_eq!(total, 0);
    }

    #[test]
    fn snapshot_debug_format() {
        let profile = ParserFeatureProfile::current();
        let snap =
            bdd_governance_snapshot(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);
        let debug = format!("{:?}", snap);
        assert!(debug.contains("phase"));
    }

    #[test]
    fn bdd_progress_report_contains_title() {
        let profile = ParserFeatureProfile::current();
        let report = bdd_progress_report_with_profile(
            BddPhase::Core,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Core Title",
            profile,
        );
        assert!(report.contains("Core Title"));
    }

    #[test]
    fn bdd_progress_status_line_format() {
        let profile = ParserFeatureProfile::current();
        let core_status =
            bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
        let runtime_status =
            bdd_progress_status_line(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);
        assert!(core_status.starts_with("core:"));
        assert!(runtime_status.starts_with("runtime:"));
    }

    #[test]
    fn describe_backend_for_conflicts_non_empty() {
        let desc = describe_backend_for_conflicts(ParserFeatureProfile::current());
        assert!(!desc.is_empty());
    }

    #[test]
    fn glr_conflict_fallback_is_accessible() {
        assert!(!GLR_CONFLICT_FALLBACK.is_empty());
    }

    #[test]
    fn governance_snapshot_is_fully_implemented_check() {
        let snap = BddGovernanceSnapshot {
            phase: BddPhase::Core,
            implemented: 5,
            total: 5,
            profile: ParserFeatureProfile::current(),
        };
        assert!(snap.is_fully_implemented());

        let partial = BddGovernanceSnapshot {
            phase: BddPhase::Core,
            implemented: 3,
            total: 5,
            profile: ParserFeatureProfile::current(),
        };
        assert!(!partial.is_fully_implemented());
    }

    #[test]
    fn governance_matrix_status_line_non_empty() {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        assert!(!matrix.status_line().is_empty());
    }
}
