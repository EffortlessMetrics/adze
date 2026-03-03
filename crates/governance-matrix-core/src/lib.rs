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
}
