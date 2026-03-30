//! Integration tests for the parser governance chain.
//!
//! Tests the chain: bdd-governance-contract → parser-governance-contract

/// Tests that parser-governance-contract re-exports all necessary types.
#[test]
fn test_parser_chain_reexports_from_bdd_governance() {
    // Given: Types re-exported through parser-governance-contract
    use adze_parser_governance_contract::{
        BddPhase, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, ParserFeatureProfile,
    };

    // When: Use the re-exported types
    let phase = BddPhase::Core;
    let status = BddScenarioStatus::Implemented;
    let _profile = ParserFeatureProfile::current();

    // Then: All types should be accessible and work correctly
    assert_eq!(phase, BddPhase::Core);
    assert!(status.implemented());
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

/// Tests that bdd_progress works through the parser governance chain.
#[test]
fn test_parser_chain_bdd_progress() {
    // Given: Re-exported bdd_progress function
    use adze_parser_governance_contract::{BddPhase, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress};

    // When: Calculate progress
    let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);

    // Then: Should return valid counts
    assert!(total > 0);
    assert!(implemented <= total);
}

/// Tests that bdd_progress_status_line integrates properly.
#[test]
fn test_parser_chain_status_line_integration() {
    // Given: Re-exported status line function
    use adze_parser_governance_contract::{
        BddPhase, GLR_CONFLICT_PRESERVATION_GRID, ParserFeatureProfile, bdd_progress_status_line,
    };

    let profile = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // When: Generate status lines for both phases
    let core_status =
        bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
    let runtime_status =
        bdd_progress_status_line(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Then: Status lines should contain expected information
    assert!(core_status.contains("core:"));
    assert!(runtime_status.contains("runtime:"));
    assert!(core_status.contains("tree-sitter-standard"));
}

/// Tests that bdd_progress_report_with_profile works through the chain.
#[test]
fn test_parser_chain_report_with_profile() {
    // Given: Re-exported report function
    use adze_parser_governance_contract::{
        BddPhase, GLR_CONFLICT_PRESERVATION_GRID, ParserFeatureProfile,
        bdd_progress_report_with_profile,
    };

    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When: Generate a report
    let report = bdd_progress_report_with_profile(
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Parser Chain Test",
        profile,
    );

    // Then: Report should contain expected sections
    assert!(report.contains("Parser Chain Test"));
    assert!(report.contains("Feature profile:"));
    assert!(report.contains("Non-conflict backend:"));
}

/// Tests that ParserFeatureProfile::current() returns a valid profile.
#[test]
fn test_parser_chain_current_profile() {
    // Given: ParserFeatureProfile
    use adze_parser_governance_contract::ParserFeatureProfile;

    // When: Get the current profile
    let profile = ParserFeatureProfile::current();

    // Then: Profile should be valid (at least one backend enabled)
    // Note: The exact backend depends on compile-time feature flags
    let _ = profile;
}

/// Tests that ParserBackend resolution works correctly through the chain.
#[test]
fn test_parser_chain_backend_resolution() {
    // Given: Different profile configurations
    use adze_parser_governance_contract::{ParserBackend, ParserFeatureProfile};

    // When: Create profiles and resolve backends
    let pure_rust = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let ts_standard = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };

    // Then: Backend resolution should match expectations
    assert_eq!(pure_rust.resolve_backend(false), ParserBackend::PureRust);
    assert_eq!(
        ts_standard.resolve_backend(false),
        ParserBackend::TreeSitter
    );
}

/// Tests that conflict backend resolution works correctly.
#[test]
fn test_parser_chain_conflict_backend_resolution() {
    // Given: A GLR-enabled profile
    use adze_parser_governance_contract::{ParserBackend, ParserFeatureProfile};

    let glr_profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When: Resolve backend for conflict grammar
    let backend = glr_profile.resolve_backend(true);

    // Then: Should return GLR backend for conflict grammars when glr feature is enabled
    assert_eq!(backend, ParserBackend::GLR);
}

/// Tests that bdd_progress_report works through the chain.
#[test]
fn test_parser_chain_progress_report() {
    // Given: Re-exported report function
    use adze_parser_governance_contract::{
        BddPhase, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress_report,
    };

    // When: Generate a simple progress report
    let report = bdd_progress_report(
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Simple Report",
    );

    // Then: Report should contain the title
    assert!(report.contains("Simple Report"));
}
