//! Contract lock test - verifies that public API remains stable.
//!
//! This crate is a compatibility facade that re-exports from adze_bdd_governance_core.

use adze_governance_matrix_core_impl::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus,
    GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile,
    bdd_progress_report, bdd_progress_report_with_profile, bdd_progress_status_line,
    describe_backend_for_conflicts,
};

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify BddPhase enum exists with expected variants
    let _core = BddPhase::Core;
    let _runtime = BddPhase::Runtime;

    // Verify ParserBackend enum exists with all variants
    let _tree_sitter = ParserBackend::TreeSitter;
    let _pure_rust = ParserBackend::PureRust;
    let _glr = ParserBackend::GLR;

    // Verify ParserFeatureProfile struct exists
    let _profile = ParserFeatureProfile::current();

    // Verify BddScenario struct exists
    let _scenario = BddScenario {
        id: 0,
        title: "test scenario",
        reference: "test reference",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Implemented,
    };

    // Verify BddGovernanceSnapshot struct exists
    let _snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 0,
        total: 0,
        profile: ParserFeatureProfile::current(),
    };
}

/// Verify all public constants exist.
#[test]
fn test_contract_lock_constants() {
    // Verify GLR_CONFLICT_PRESERVATION_GRID constant exists
    let grid = GLR_CONFLICT_PRESERVATION_GRID;
    assert!(!grid.is_empty());

    // Verify GLR_CONFLICT_FALLBACK constant exists
    let fallback = GLR_CONFLICT_FALLBACK;
    assert!(!fallback.is_empty());
}

/// Verify all public functions exist with expected signatures.
#[test]
fn test_contract_lock_functions() {
    let profile = ParserFeatureProfile::current();

    // Verify describe_backend_for_conflicts function exists
    let _desc = describe_backend_for_conflicts(profile);
    assert!(!_desc.is_empty());

    // Verify bdd_progress_report function exists
    let _report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Test");
    assert!(_report.contains("Test"));

    // Verify bdd_progress_report_with_profile function exists
    let _report = bdd_progress_report_with_profile(
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Test",
        profile,
    );
    assert!(_report.contains("Test"));

    // Verify bdd_progress_status_line function exists
    let _status = bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
    assert!(_status.starts_with("core:"));
}

/// Verify BddGovernanceMatrix methods.
#[test]
fn test_contract_lock_matrix_methods() {
    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);

    // Verify snapshot method exists
    let snapshot = matrix.snapshot();
    assert_eq!(snapshot.profile, profile);

    // Verify phase field exists
    assert_eq!(matrix.phase, BddPhase::Core);

    // Verify profile field exists
    assert_eq!(matrix.profile, profile);
}

/// Verify BddGovernanceSnapshot methods.
#[test]
fn test_contract_lock_snapshot_methods() {
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 5,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };

    // Verify is_fully_implemented method exists
    assert!(snapshot.is_fully_implemented());

    let partial = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 2,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };
    assert!(!partial.is_fully_implemented());
}

/// Verify ParserBackend methods.
#[test]
fn test_contract_lock_backend_methods() {
    // Verify name method exists
    let name: &'static str = ParserBackend::TreeSitter.name();
    assert!(!name.is_empty());

    // Verify is_glr method exists
    assert!(ParserBackend::GLR.is_glr());
    assert!(!ParserBackend::TreeSitter.is_glr());

    // Verify is_pure_rust method exists
    assert!(ParserBackend::PureRust.is_pure_rust());
    assert!(!ParserBackend::TreeSitter.is_pure_rust());
}

/// Verify ParserFeatureProfile methods.
#[test]
fn test_contract_lock_profile_methods() {
    // Verify current method exists
    let _profile = ParserFeatureProfile::current();

    // Verify resolve_backend method exists
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };
    let _backend = profile.resolve_backend(false);

    // Verify has_pure_rust method exists
    let _has = profile.has_pure_rust();

    // Verify has_glr method exists
    let _has = profile.has_glr();

    // Verify has_tree_sitter method exists
    let _has = profile.has_tree_sitter();
}
