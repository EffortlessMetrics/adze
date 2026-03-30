//! Contract lock tests - verify API stability
//! These tests ensure the public API remains stable.

#[cfg(test)]
mod contract_lock {
    use adze_bdd_governance_fixtures::*;

    #[test]
    fn contract_lock_types() {
        // Verify types exist and are accessible
        let _phase = BddPhase::Core;
        let _phase = BddPhase::Runtime;

        let _status = BddScenarioStatus::Implemented;
        let _status = BddScenarioStatus::Deferred { reason: "test" };

        // Verify profile types
        let _profile = ParserFeatureProfile::current();
        let _backend = ParserBackend::GLR;
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

        // Crate-specific functions
        let current_report = bdd_progress_report_for_current_profile(BddPhase::Core, "Test");
        assert!(!current_report.is_empty());

        let current_status = bdd_progress_status_line_for_current_profile(BddPhase::Core);
        assert!(!current_status.is_empty());
    }

    #[test]
    fn contract_lock_constants() {
        // Verify constants exist and are accessible
        assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
    }
}
