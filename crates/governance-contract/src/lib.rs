//! Shared governance contracts for parser backend selection and BDD progress tracking.
//!
//! This crate owns the governance-facing parser backend and BDD progress
//! contracts used by the existing workspace.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]
pub use adze_bdd_governance_contract::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexported_types_are_usable() {
        let phase = BddPhase::Core;
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        assert_eq!(matrix.phase, phase);
    }

    #[test]
    fn bdd_progress_with_grid() {
        let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
        assert_eq!(total, GLR_CONFLICT_PRESERVATION_GRID.len());
        assert!(implemented <= total);
    }

    #[test]
    fn bdd_progress_report_with_profile_nonempty() {
        let profile = ParserFeatureProfile::current();
        let report = bdd_progress_report_with_profile(
            BddPhase::Runtime,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Gov Test",
            profile,
        );
        assert!(!report.is_empty());
        assert!(report.contains("Gov Test"));
    }

    #[test]
    fn status_line_nonempty() {
        let profile = ParserFeatureProfile::current();
        let line =
            bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
        assert!(!line.is_empty());
    }

    #[test]
    fn scenario_status_variants() {
        let impl_status = BddScenarioStatus::Implemented;
        let deferred = BddScenarioStatus::Deferred { reason: "later" };
        assert!(impl_status.implemented());
        assert!(!deferred.implemented());
    }

    #[test]
    fn parser_backend_variants_accessible() {
        let _ = ParserBackend::TreeSitter;
        let _ = ParserBackend::PureRust;
        let _ = ParserBackend::GLR;
    }
}
