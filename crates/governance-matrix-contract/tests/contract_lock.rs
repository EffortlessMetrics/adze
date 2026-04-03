//! Contract lock test - verifies that public API remains stable.

use adze_governance_matrix_contract::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, BddScenario, BddScenarioStatus,
    GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID, ParserBackend, ParserFeatureProfile,
    bdd_governance_snapshot, bdd_progress, bdd_progress_report, bdd_progress_report_with_profile,
    bdd_progress_status_line, describe_backend_for_conflicts,
};

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify BddPhase enum exists with expected variants
    let _core = BddPhase::Core;
    let _runtime = BddPhase::Runtime;

    // Verify ParserBackend enum exists with expected variants
    let _tree_sitter = ParserBackend::TreeSitter;
    let _pure_rust = ParserBackend::PureRust;
    let _glr = ParserBackend::GLR;

    // Verify BddScenarioStatus enum exists with expected variants
    let _implemented = BddScenarioStatus::Implemented;
    let _deferred = BddScenarioStatus::Deferred { reason: "test" };

    // Verify ParserFeatureProfile struct exists with expected fields
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };
    assert!(profile.pure_rust);
    assert!(profile.glr);

    // Verify BddScenario struct exists with expected fields
    let scenario = BddScenario {
        id: 1,
        title: "test",
        reference: "T-1",
        core_status: BddScenarioStatus::Implemented,
        runtime_status: BddScenarioStatus::Deferred { reason: "wip" },
    };
    assert_eq!(scenario.id, 1);

    // Verify BddGovernanceMatrix struct exists with expected fields
    let profile = ParserFeatureProfile::current();
    let matrix = BddGovernanceMatrix::standard(profile);
    assert_eq!(matrix.phase, BddPhase::Core);
    assert!(!matrix.scenarios.is_empty());

    // Verify BddGovernanceSnapshot struct exists with expected fields
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 5,
        total: 10,
        profile: ParserFeatureProfile::current(),
    };
    assert_eq!(snapshot.implemented, 5);
    assert_eq!(snapshot.total, 10);
}

/// Verify all public constants exist with expected values.
#[test]
fn test_contract_lock_constants() {
    // Verify GLR_CONFLICT_PRESERVATION_GRID constant exists
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());

    // Verify GLR_CONFLICT_FALLBACK constant exists
    assert!(!GLR_CONFLICT_FALLBACK.is_empty());
}

/// Verify all public functions exist with expected signatures.
#[allow(clippy::type_complexity)]
#[test]
fn test_contract_lock_functions() {
    // Verify bdd_progress function exists
    let _fn_ptr: Option<fn(BddPhase, &[BddScenario]) -> (usize, usize)> = Some(bdd_progress);

    // Verify bdd_progress_report function exists
    let _fn_ptr: Option<fn(BddPhase, &[BddScenario], &str) -> String> = Some(bdd_progress_report);

    // Verify bdd_progress_report_with_profile function exists
    let _fn_ptr: Option<fn(BddPhase, &[BddScenario], &str, ParserFeatureProfile) -> String> =
        Some(bdd_progress_report_with_profile);

    // Verify bdd_progress_status_line function exists
    let _fn_ptr: Option<fn(BddPhase, &[BddScenario], ParserFeatureProfile) -> String> =
        Some(bdd_progress_status_line);

    // Verify bdd_governance_snapshot function exists
    let _fn_ptr: Option<
        fn(BddPhase, &[BddScenario], ParserFeatureProfile) -> BddGovernanceSnapshot,
    > = Some(bdd_governance_snapshot);

    // Verify describe_backend_for_conflicts function exists
    let _fn_ptr: Option<fn(ParserFeatureProfile) -> &'static str> =
        Some(describe_backend_for_conflicts);
}

/// Verify BddGovernanceMatrix methods exist.
#[test]
fn test_contract_lock_matrix_methods() {
    let profile = ParserFeatureProfile::current();

    // Verify standard constructor exists
    let _matrix = BddGovernanceMatrix::standard(profile);

    // Verify new constructor exists
    let _matrix =
        BddGovernanceMatrix::new(BddPhase::Runtime, profile, GLR_CONFLICT_PRESERVATION_GRID);

    // Verify snapshot method exists
    let matrix = BddGovernanceMatrix::standard(profile);
    let snapshot = matrix.snapshot();
    assert_eq!(snapshot.phase, BddPhase::Core);

    // Verify report method exists
    let matrix = BddGovernanceMatrix::standard(profile);
    let _report = matrix.report("Test");

    // Verify status_line method exists
    let matrix = BddGovernanceMatrix::standard(profile);
    let _status = matrix.status_line();
}

/// Verify BddGovernanceSnapshot methods exist.
#[test]
fn test_contract_lock_snapshot_methods() {
    let snapshot = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 5,
        total: 5,
        profile: ParserFeatureProfile::current(),
    };

    // Verify is_fully_implemented method exists
    let _result = snapshot.is_fully_implemented();

    // Verify non_conflict_backend method exists
    let _backend = snapshot.non_conflict_backend();
}
