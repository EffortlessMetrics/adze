//! Contract lock tests - verify API stability
//! These tests ensure the public API remains stable.

#[cfg(test)]
mod contract_lock {
    use adze_bdd_contract::*;

    #[test]
    fn contract_lock_types() {
        // Verify types exist and are accessible
        let _phase = BddPhase::Core;
        let _phase = BddPhase::Runtime;

        let _status = BddScenarioStatus::Implemented;
        let _status = BddScenarioStatus::Deferred { reason: "test" };

        // Verify scenario type exists
        let scenario: &BddScenario = &GLR_CONFLICT_PRESERVATION_GRID[0];
        let _title = scenario.title;
        let _reference = scenario.reference;
    }

    #[test]
    fn contract_lock_functions() {
        // Verify functions exist and are callable
        let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
        assert!(implemented <= total);

        let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Test");
        assert!(!report.is_empty());
    }

    #[test]
    fn contract_lock_constants() {
        // Verify constants exist and are accessible
        assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
    }
}
