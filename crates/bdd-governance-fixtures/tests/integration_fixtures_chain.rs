//! Integration tests for the BDD fixtures chain.
//!
//! Tests the chain: bdd-governance-contract → bdd-governance-fixtures

/// Tests that fixtures properly re-export types from governance contract.
#[test]
fn test_fixtures_chain_reexports_from_contract() {
    // Given: Types re-exported through the fixtures chain
    use adze_bdd_governance_fixtures::{
        BddPhase, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, ParserFeatureProfile,
    };

    // When: Use the re-exported types
    let phase = BddPhase::Core;
    let status = BddScenarioStatus::Implemented;
    let _profile = ParserFeatureProfile::current();

    // Then: Types should work correctly
    assert_eq!(phase, BddPhase::Core);
    assert!(status.implemented());
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

/// Tests that bdd_progress_report_for_current_profile works through the chain.
#[test]
fn test_fixtures_chain_current_profile_report() {
    // Given: The fixtures helper for current profile reporting
    use adze_bdd_governance_fixtures::{BddPhase, bdd_progress_report_for_current_profile};

    // When: Generate a report for the Core phase
    let report = bdd_progress_report_for_current_profile(BddPhase::Core, "Test Phase");

    // Then: Report should contain the phase title
    assert!(report.contains("Test Phase"));
    assert!(!report.is_empty());
}

/// Tests that bdd_progress_status_line_for_current_profile works through the chain.
#[test]
fn test_fixtures_chain_current_profile_status_line() {
    // Given: The fixtures helper for status line
    use adze_bdd_governance_fixtures::{BddPhase, bdd_progress_status_line_for_current_profile};

    // When: Generate status lines for both phases
    let core_status = bdd_progress_status_line_for_current_profile(BddPhase::Core);
    let runtime_status = bdd_progress_status_line_for_current_profile(BddPhase::Runtime);

    // Then: Both should be non-empty and start with phase prefix
    assert!(!core_status.is_empty());
    assert!(!runtime_status.is_empty());
    assert!(core_status.starts_with("core:") || core_status.contains("core"));
}

/// Tests that bdd_progress function works through the fixtures chain.
#[test]
fn test_fixtures_chain_bdd_progress() {
    // Given: Re-exported bdd_progress function
    use adze_bdd_governance_fixtures::{BddPhase, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress};

    // When: Calculate progress for both phases
    let (core_impl, core_total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);
    let (rt_impl, rt_total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);

    // Then: Should return valid counts
    assert!(core_impl <= core_total);
    assert!(rt_impl <= rt_total);
}

/// Tests that bdd_progress_report_with_profile integrates properly.
#[test]
fn test_fixtures_chain_progress_report_with_profile() {
    // Given: Re-exported report function with profile
    use adze_bdd_governance_fixtures::{
        BddPhase, GLR_CONFLICT_PRESERVATION_GRID, ParserFeatureProfile,
        bdd_progress_report_with_profile,
    };

    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When: Generate a report with explicit profile
    let report = bdd_progress_report_with_profile(
        BddPhase::Runtime,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Runtime Check",
        profile,
    );

    // Then: Report should contain profile info and phase title
    assert!(report.contains("Runtime Check"));
    assert!(report.contains("Feature profile:"));
}

/// Tests that ParserBackend is accessible and works correctly.
#[test]
fn test_fixtures_chain_parser_backend() {
    // Given: Re-exported ParserBackend
    use adze_bdd_governance_fixtures::{ParserBackend, ParserFeatureProfile};

    // When: Create profiles with different backends
    let pure_rust_profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let ts_profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // Then: Backend resolution should work correctly
    assert_eq!(
        pure_rust_profile.resolve_backend(false),
        ParserBackend::PureRust
    );
    assert_eq!(ts_profile.resolve_backend(false), ParserBackend::TreeSitter);
}

/// Tests that all BddPhase variants are accessible.
#[test]
fn test_fixtures_chain_bdd_phase_variants() {
    // Given: Re-exported BddPhase
    use adze_bdd_governance_fixtures::BddPhase;

    // When: Use both variants
    let core = BddPhase::Core;
    let runtime = BddPhase::Runtime;

    // Then: They should be distinct
    assert_ne!(core, runtime);
    assert_eq!(core, BddPhase::Core);
    assert_eq!(runtime, BddPhase::Runtime);
}

/// Tests that BddScenarioStatus variants work correctly.
#[test]
fn test_fixtures_chain_scenario_status() {
    // Given: Re-exported BddScenarioStatus
    use adze_bdd_governance_fixtures::BddScenarioStatus;

    // When: Create both status variants
    let implemented = BddScenarioStatus::Implemented;
    let deferred = BddScenarioStatus::Deferred {
        reason: "pending work",
    };

    // Then: implemented() should return correct values
    assert!(implemented.implemented());
    assert!(!deferred.implemented());
    assert_eq!(implemented.icon(), "✅");
    assert_eq!(deferred.icon(), "⏳");
}
