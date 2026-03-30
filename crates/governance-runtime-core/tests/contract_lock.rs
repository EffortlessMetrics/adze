//! Contract lock test - verifies that public API remains stable.

use adze_governance_runtime_core::{
    BddGovernanceMatrix, BddPhase, ParserBackend, ParserFeatureProfile,
    bdd_governance_matrix_for_profile, bdd_governance_matrix_for_runtime,
    bdd_governance_matrix_for_runtime2, bdd_progress_report_for_profile,
    bdd_progress_status_line_for_profile, parser_feature_profile_for_runtime,
    parser_feature_profile_for_runtime2, resolve_backend_for_profile,
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

    // Verify ParserFeatureProfile struct exists with expected fields
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };
    assert!(profile.pure_rust);
    assert!(profile.glr);

    // Verify BddGovernanceMatrix struct exists
    let profile = ParserFeatureProfile::current();
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);
    assert_eq!(matrix.phase, BddPhase::Core);
    assert!(!matrix.scenarios.is_empty());
}

/// Verify all public functions exist with expected signatures.
#[test]
fn test_contract_lock_functions() {
    // Verify parser_feature_profile_for_runtime function exists
    let _fn_ptr: Option<fn() -> ParserFeatureProfile> = Some(parser_feature_profile_for_runtime);

    // Verify parser_feature_profile_for_runtime2 function exists
    let _fn_ptr: Option<fn(bool) -> ParserFeatureProfile> =
        Some(parser_feature_profile_for_runtime2);

    // Verify resolve_backend_for_profile function exists
    let _fn_ptr: Option<fn(ParserFeatureProfile, bool) -> ParserBackend> =
        Some(resolve_backend_for_profile);

    // Verify bdd_progress_report_for_profile function exists
    let _fn_ptr: Option<fn(BddPhase, &str, ParserFeatureProfile) -> String> =
        Some(bdd_progress_report_for_profile);

    // Verify bdd_progress_status_line_for_profile function exists
    let _fn_ptr: Option<fn(BddPhase, ParserFeatureProfile) -> String> =
        Some(bdd_progress_status_line_for_profile);

    // Verify bdd_governance_matrix_for_profile function exists
    let _fn_ptr: Option<fn(BddPhase, ParserFeatureProfile) -> BddGovernanceMatrix> =
        Some(bdd_governance_matrix_for_profile);

    // Verify bdd_governance_matrix_for_runtime function exists
    let _fn_ptr: Option<fn() -> BddGovernanceMatrix> = Some(bdd_governance_matrix_for_runtime);

    // Verify bdd_governance_matrix_for_runtime2 function exists
    let _fn_ptr: Option<fn(BddPhase, bool) -> BddGovernanceMatrix> =
        Some(bdd_governance_matrix_for_runtime2);
}

/// Verify profile functions return valid profiles.
#[test]
fn test_contract_lock_profile_functions() {
    // Verify parser_feature_profile_for_runtime returns valid profile
    let profile = parser_feature_profile_for_runtime();
    let _ = profile.resolve_backend(false);

    // Verify parser_feature_profile_for_runtime2 returns valid profiles
    let enabled = parser_feature_profile_for_runtime2(true);
    assert!(enabled.pure_rust);
    assert!(enabled.glr);

    let disabled = parser_feature_profile_for_runtime2(false);
    assert!(!disabled.pure_rust);
    assert!(!disabled.glr);
}

/// Verify matrix functions return valid matrices.
#[test]
fn test_contract_lock_matrix_functions() {
    // Verify bdd_governance_matrix_for_runtime returns valid matrix
    let matrix = bdd_governance_matrix_for_runtime();
    assert_eq!(matrix.phase, BddPhase::Runtime);
    assert!(!matrix.scenarios.is_empty());

    // Verify bdd_governance_matrix_for_runtime2 returns valid matrices
    let enabled = bdd_governance_matrix_for_runtime2(BddPhase::Core, true);
    assert_eq!(enabled.phase, BddPhase::Core);
    assert!(enabled.profile.glr);

    let disabled = bdd_governance_matrix_for_runtime2(BddPhase::Runtime, false);
    assert_eq!(disabled.phase, BddPhase::Runtime);
    assert!(!disabled.profile.glr);

    // Verify bdd_governance_matrix_for_profile returns valid matrix
    let profile = ParserFeatureProfile::current();
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);
    assert_eq!(matrix.phase, BddPhase::Core);
    assert_eq!(matrix.profile, profile);
}

/// Verify report functions return non-empty strings.
#[test]
fn test_contract_lock_report_functions() {
    let profile = parser_feature_profile_for_runtime2(true);

    // Verify bdd_progress_report_for_profile returns non-empty string
    let report = bdd_progress_report_for_profile(BddPhase::Core, "Test", profile);
    assert!(!report.is_empty());
    assert!(report.contains("Test"));

    // Verify bdd_progress_status_line_for_profile returns non-empty string
    let status = bdd_progress_status_line_for_profile(BddPhase::Core, profile);
    assert!(!status.is_empty());
    assert!(status.starts_with("core:"));
}
