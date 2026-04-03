//! Contract lock test - verifies that public API remains stable.

use adze_parser_contract::{
    BddGovernanceMatrix, BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID,
    ParserBackend, ParserFeatureProfile, bdd_progress, bdd_progress_report,
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
    let profile = ParserFeatureProfile::current();
    let _: bool = profile.pure_rust; // Field exists

    // Verify BddGovernanceMatrix struct exists with expected fields
    let matrix = BddGovernanceMatrix::standard(profile);
    assert_eq!(matrix.phase, BddPhase::Core);
    assert!(!matrix.scenarios.is_empty());
}

/// Verify all public constants exist with expected values.
#[test]
fn test_contract_lock_constants() {
    // Verify GLR_CONFLICT_PRESERVATION_GRID constant exists
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

/// Verify all public functions exist with expected signatures.
#[allow(clippy::type_complexity)]
#[test]
fn test_contract_lock_functions() {
    // Verify bdd_progress function exists
    let _fn_ptr: Option<fn(BddPhase, &[BddScenario]) -> (usize, usize)> = Some(bdd_progress);

    // Verify bdd_progress_report function exists
    let _fn_ptr: Option<fn(BddPhase, &[BddScenario], &str) -> String> = Some(bdd_progress_report);
}

/// Verify ParserBackend methods exist.
#[test]
fn test_contract_lock_backend_methods() {
    // Verify name method exists
    let name = ParserBackend::TreeSitter.name();
    assert!(!name.is_empty());

    // Verify is_glr method exists
    let _ = ParserBackend::GLR.is_glr();

    // Verify is_pure_rust method exists
    let _ = ParserBackend::PureRust.is_pure_rust();

    // Verify select method exists
    let _ = ParserBackend::select(false);
}

/// Verify ParserFeatureProfile methods exist.
#[test]
fn test_contract_lock_profile_methods() {
    // Verify current method exists
    let _ = ParserFeatureProfile::current();

    // Verify resolve_backend method exists
    let profile = ParserFeatureProfile::current();
    let _ = profile.resolve_backend(false);

    // Verify has_glr method exists
    let _ = profile.has_glr();
}

/// Verify BddGovernanceMatrix methods exist.
#[test]
fn test_contract_lock_matrix_methods() {
    let profile = ParserFeatureProfile::current();

    // Verify standard constructor exists
    let _matrix = BddGovernanceMatrix::standard(profile);

    // Verify status_line method exists
    let matrix = BddGovernanceMatrix::standard(profile);
    let status = matrix.status_line();
    assert!(!status.is_empty());
}

/// Verify BddScenarioStatus methods exist.
#[test]
fn test_contract_lock_status_methods() {
    // Verify implemented method exists
    let _ = BddScenarioStatus::Implemented.implemented();

    // Verify icon method exists
    let _ = BddScenarioStatus::Implemented.icon();
}
