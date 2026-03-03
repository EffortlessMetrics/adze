//! Shared contracts for parser backend selection and BDD scenario tracking.
//!
//! This crate remains the compatibility facade used by the existing workspace,
//! while the concrete contracts live in `adze-governance-contract`.

#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
pub use adze_governance_contract::{
    BddGovernanceMatrix, BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID,
    ParserBackend, ParserFeatureProfile, bdd_progress, bdd_progress_report,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_backend_display_names() {
        assert!(!format!("{}", ParserBackend::TreeSitter).is_empty());
        assert!(!format!("{}", ParserBackend::PureRust).is_empty());
        assert!(!format!("{}", ParserBackend::GLR).is_empty());
    }

    #[test]
    fn bdd_governance_matrix_standard() {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        assert!(!matrix.scenarios.is_empty());
    }

    #[test]
    fn bdd_progress_from_grid() {
        let (impl_count, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
        assert_eq!(total, GLR_CONFLICT_PRESERVATION_GRID.len());
        assert!(impl_count <= total);
    }

    #[test]
    fn bdd_progress_report_output() {
        let report = bdd_progress_report(
            BddPhase::Runtime,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Parser Contract",
        );
        assert!(report.contains("Parser Contract"));
    }

    #[test]
    fn scenario_status_icons() {
        let implemented = BddScenarioStatus::Implemented;
        let deferred = BddScenarioStatus::Deferred { reason: "pending" };
        assert_ne!(implemented.icon(), deferred.icon());
    }

    #[test]
    fn feature_profile_consistency() {
        let a = ParserFeatureProfile::current();
        let b = ParserFeatureProfile::current();
        assert_eq!(a, b);
    }
}
