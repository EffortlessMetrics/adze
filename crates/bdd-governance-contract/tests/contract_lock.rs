//! Contract lock tests - verify API stability
//! These tests ensure the public API remains stable.

#[cfg(test)]
mod contract_lock {
    use adze_bdd_governance_contract::*;

    #[test]
    fn contract_lock_types() {
        // Verify types exist and are accessible
        let _phase = BddPhase::Core;
        let _phase = BddPhase::Runtime;

        let _status = BddScenarioStatus::Implemented;
        let _status = BddScenarioStatus::Deferred { reason: "test" };

        // Verify governance types
        let _profile = ParserFeatureProfile::current();
        let _backend = ParserBackend::GLR;

        // Verify snapshot type
        let _snap = BddGovernanceSnapshot {
            phase: BddPhase::Core,
            implemented: 0,
            total: 0,
            profile: ParserFeatureProfile::current(),
        };

        // Verify matrix type
        let _matrix = BddGovernanceMatrix::standard(ParserFeatureProfile::current());
    }

    #[test]
    fn contract_lock_functions() {
        // Verify functions exist and are callable
        let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
        assert!(implemented <= total);

        let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Test");
        assert!(!report.is_empty());

        let profile = ParserFeatureProfile::current();
        let report_with_profile = bdd_progress_report_with_profile(
            BddPhase::Core,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Test",
            profile,
        );
        assert!(!report_with_profile.is_empty());

        let status_line =
            bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
        assert!(!status_line.is_empty());

        let _desc = describe_backend_for_conflicts(profile);

        let _snapshot =
            bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
    }

    #[test]
    fn contract_lock_constants() {
        // Verify constants exist and are accessible
        assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
        assert!(!GLR_CONFLICT_FALLBACK.is_empty());
    }
}
