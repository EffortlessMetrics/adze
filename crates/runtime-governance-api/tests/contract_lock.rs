//! Contract lock test - verifies that public API remains stable.
//!
//! This crate provides a runtime-facing governance API for parser selection and BDD snapshot reporting.

use adze_runtime_governance_api::{
    BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_FALLBACK,
    GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, bdd_governance_matrix_for_current_profile,
    bdd_governance_matrix_for_profile, bdd_governance_matrix_for_runtime,
    bdd_governance_matrix_for_runtime2, bdd_governance_matrix_for_runtime2_profile,
    bdd_governance_snapshot, bdd_progress, bdd_progress_report,
    bdd_progress_report_for_current_profile, bdd_progress_report_with_profile,
    bdd_progress_report_with_profile_runtime, bdd_progress_status_line,
    bdd_status_line_for_current_profile, current_backend_for, describe_backend_for_conflicts,
    parser_feature_profile_for_runtime, resolve_backend_for_profile, runtime_governance_snapshot,
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
    let _profile = parser_feature_profile_for_runtime();

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
        profile: parser_feature_profile_for_runtime(),
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
    let profile = parser_feature_profile_for_runtime();

    // Verify current_backend_for function exists
    let _backend = current_backend_for(false);

    // Verify parser_feature_profile_for_runtime function exists
    let _profile = parser_feature_profile_for_runtime();

    // Verify resolve_backend_for_profile function exists
    let _backend = resolve_backend_for_profile(profile, false);

    // Verify describe_backend_for_conflicts function exists
    let _desc = describe_backend_for_conflicts(profile);
    assert!(!_desc.is_empty());

    // Verify bdd_progress function exists
    let (_implemented, _total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);

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

    // Verify bdd_progress_report_with_profile_runtime function exists
    let _report = bdd_progress_report_with_profile_runtime(
        BddPhase::Core,
        GLR_CONFLICT_PRESERVATION_GRID,
        "Test",
        profile,
    );

    // Verify bdd_progress_status_line function exists
    let _status = bdd_progress_status_line(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);
}

/// Verify current profile helper functions.
#[test]
fn test_contract_lock_current_profile_functions() {
    // Verify bdd_progress_report_for_current_profile function exists
    let _report = bdd_progress_report_for_current_profile(BddPhase::Core, "Core");
    assert!(_report.contains("Core"));

    // Verify bdd_status_line_for_current_profile function exists
    let _status = bdd_status_line_for_current_profile(BddPhase::Runtime);
    assert!(_status.starts_with("runtime:"));
}

/// Verify matrix helper functions.
#[test]
fn test_contract_lock_matrix_functions() {
    let profile = parser_feature_profile_for_runtime();

    // Verify bdd_governance_matrix_for_current_profile function exists
    let _matrix = bdd_governance_matrix_for_current_profile(BddPhase::Core);

    // Verify bdd_governance_matrix_for_profile function exists
    let _matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);

    // Verify bdd_governance_matrix_for_runtime function exists
    let _matrix = bdd_governance_matrix_for_runtime();

    // Verify bdd_governance_matrix_for_runtime2 function exists
    let _matrix = bdd_governance_matrix_for_runtime2(BddPhase::Core, profile.glr);

    // Verify bdd_governance_matrix_for_runtime2_profile function exists
    let _matrix = bdd_governance_matrix_for_runtime2_profile(BddPhase::Core, profile.glr);
}

/// Verify snapshot helper functions.
#[test]
fn test_contract_lock_snapshot_functions() {
    let profile = parser_feature_profile_for_runtime();

    // Verify bdd_governance_snapshot function exists
    let _snapshot =
        bdd_governance_snapshot(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, profile);

    // Verify runtime_governance_snapshot function exists
    let snapshot = runtime_governance_snapshot(BddPhase::Core);
    assert_eq!(snapshot.phase, BddPhase::Core);
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

    // Verify select method exists
    let _backend = ParserBackend::select(false);
}

/// Verify ParserFeatureProfile methods.
#[test]
fn test_contract_lock_profile_methods() {
    let profile = parser_feature_profile_for_runtime();

    // Verify resolve_backend method exists
    let _backend = profile.resolve_backend(false);

    // Verify has_pure_rust method exists
    let _has = profile.has_pure_rust();

    // Verify has_glr method exists
    let _has = profile.has_glr();

    // Verify has_tree_sitter method exists
    let _has = profile.has_tree_sitter();
}

/// Verify BddGovernanceSnapshot methods.
#[test]
fn test_contract_lock_snapshot_methods() {
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 5,
        total: 5,
        profile: parser_feature_profile_for_runtime(),
    };

    // Verify is_fully_implemented method exists
    assert!(snapshot.is_fully_implemented());

    let partial = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 2,
        total: 5,
        profile: parser_feature_profile_for_runtime(),
    };
    assert!(!partial.is_fully_implemented());
}

/// Verify current_backend_matches_selection_logic behavior.
#[test]
fn test_contract_lock_backend_selection() {
    // Verify current_backend_for matches ParserBackend::select
    assert_eq!(current_backend_for(false), ParserBackend::select(false));
}
