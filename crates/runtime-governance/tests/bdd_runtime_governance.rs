//! BDD tests for runtime-governance facade crate.
//!
//! These tests verify the public API behavior using Given/When/Then style.

use adze_runtime_governance::*;

// =============================================================================
// BddPhase Tests
// =============================================================================

#[test]
fn given_core_phase_when_checking_variants_then_core_exists() {
    // Given / When
    let phase = BddPhase::Core;

    // Then
    assert!(matches!(phase, BddPhase::Core));
}

#[test]
fn given_runtime_phase_when_checking_variants_then_runtime_exists() {
    // Given / When
    let phase = BddPhase::Runtime;

    // Then
    assert!(matches!(phase, BddPhase::Runtime));
}

// =============================================================================
// ParserFeatureProfile Tests
// =============================================================================

#[test]
fn given_current_profile_when_accessing_then_returns_valid_profile() {
    // Given / When
    let profile = parser_feature_profile_for_runtime();

    // Then
    let _ = format!("{:?}", profile);
    let _ = format!("{}", profile);
}

#[test]
fn given_current_profile_when_checking_features_then_returns_booleans() {
    // Given
    let profile = parser_feature_profile_for_runtime();

    // When / Then - These should return booleans without panicking
    let _ = profile.has_pure_rust();
    let _ = profile.has_glr();
    let _ = profile.has_tree_sitter();
}

#[test]
fn given_current_profile_when_checking_cfg_match_then_matches() {
    // Given
    let profile = parser_feature_profile_for_runtime();

    // When / Then
    assert_eq!(profile.pure_rust, cfg!(feature = "pure-rust"));
    assert_eq!(profile.glr, cfg!(feature = "glr"));
}

// =============================================================================
// Backend Resolution Tests
// =============================================================================

#[test]
fn given_no_conflicts_when_resolving_backend_then_returns_valid_backend() {
    // Given / When
    let backend = current_backend_for(false);

    // Then
    assert!(!backend.name().is_empty());
}

#[test]
fn given_current_profile_when_resolving_backend_then_matches_profile_resolution() {
    // Given
    let profile = parser_feature_profile_for_runtime();

    // When
    let backend = current_backend_for(false);
    let profile_backend = profile.resolve_backend(false);

    // Then
    assert_eq!(backend, profile_backend);
}

#[test]
fn given_profile_and_no_conflicts_when_resolving_then_returns_expected_backend() {
    // Given
    let profile = parser_feature_profile_for_runtime();

    // When
    let backend = resolve_backend_for_profile(profile, false);

    // Then
    assert_eq!(backend, profile.resolve_backend(false));
}

// =============================================================================
// BDD Progress Report Tests
// =============================================================================

#[test]
fn given_core_phase_and_title_when_generating_report_then_contains_title() {
    // Given
    let phase = BddPhase::Core;
    let title = "Core Tests";

    // When
    let report = bdd_progress_report_for_current_profile(phase, title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_runtime_phase_and_title_when_generating_report_then_contains_phase_info() {
    // Given
    let phase = BddPhase::Runtime;
    let title = "Runtime Tests";

    // When
    let report = bdd_progress_report_for_current_profile(phase, title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_core_phase_when_generating_status_line_then_starts_with_core() {
    // Given
    let phase = BddPhase::Core;

    // When
    let status = bdd_status_line_for_current_profile(phase);

    // Then
    assert!(status.starts_with("core:"));
}

#[test]
fn given_runtime_phase_when_generating_status_line_then_starts_with_runtime() {
    // Given
    let phase = BddPhase::Runtime;

    // When
    let status = bdd_status_line_for_current_profile(phase);

    // Then
    assert!(status.starts_with("runtime:"));
}

// =============================================================================
// GLR Conflict Grid Tests
// =============================================================================

#[test]
fn given_glr_conflict_grid_when_accessing_then_is_not_empty() {
    // Given / When
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // Then
    assert!(!grid.is_empty());
}

#[test]
fn given_glr_conflict_fallback_when_accessing_then_is_not_empty() {
    // Given / When
    let grid = GLR_CONFLICT_FALLBACK;

    // Then
    assert!(!grid.is_empty());
}

// =============================================================================
// BDD Progress Tests
// =============================================================================

#[test]
fn given_core_phase_and_grid_when_calculating_progress_then_returns_valid_counts() {
    // Given
    let phase = BddPhase::Core;
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // When
    let (implemented, total) = bdd_progress(phase, grid);

    // Then
    assert!(implemented <= total);
    assert!(total > 0);
}

#[test]
fn given_runtime_phase_and_grid_when_calculating_progress_then_returns_valid_counts() {
    // Given
    let phase = BddPhase::Runtime;
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // When
    let (implemented, total) = bdd_progress(phase, grid);

    // Then
    assert!(implemented <= total);
    assert!(total > 0);
}

// =============================================================================
// BDD Governance Matrix Tests
// =============================================================================

#[test]
fn given_core_phase_when_getting_matrix_then_returns_matrix_with_phase() {
    // Given
    let phase = BddPhase::Core;

    // When
    let matrix = bdd_governance_matrix_for_current_profile(phase);

    // Then
    assert_eq!(matrix.phase, phase);
}

#[test]
fn given_runtime_phase_when_getting_matrix_then_returns_matrix_with_phase() {
    // Given
    let phase = BddPhase::Runtime;

    // When
    let matrix = bdd_governance_matrix_for_current_profile(phase);

    // Then
    assert_eq!(matrix.phase, phase);
}

#[test]
fn given_profile_when_getting_matrix_then_matrix_has_same_profile() {
    // Given
    let profile = parser_feature_profile_for_runtime();

    // When
    let matrix = bdd_governance_matrix_for_profile(BddPhase::Core, profile);

    // Then
    assert_eq!(matrix.profile, profile);
}

#[test]
fn given_runtime_matrix_when_getting_then_returns_matrix() {
    // Given
    let profile = parser_feature_profile_for_runtime();

    // When
    let matrix = bdd_governance_matrix_for_runtime();

    // Then
    assert_eq!(matrix.profile, profile);
}

// =============================================================================
// BDD Governance Snapshot Tests
// =============================================================================

#[test]
fn given_core_phase_when_getting_snapshot_then_returns_snapshot_with_phase() {
    // Given
    let phase = BddPhase::Core;

    // When
    let snap = runtime_governance_snapshot(phase);

    // Then
    assert_eq!(snap.phase, phase);
}

#[test]
fn given_runtime_phase_when_getting_snapshot_then_returns_snapshot_with_phase() {
    // Given
    let phase = BddPhase::Runtime;

    // When
    let snap = runtime_governance_snapshot(phase);

    // Then
    assert_eq!(snap.phase, phase);
}

#[test]
fn given_snapshot_when_checking_profile_then_matches_current() {
    // Given
    let profile = parser_feature_profile_for_runtime();

    // When
    let snap = runtime_governance_snapshot(BddPhase::Core);

    // Then
    assert_eq!(snap.profile, profile);
}

// =============================================================================
// Runtime2 Specific Tests
// =============================================================================

#[test]
fn given_runtime2_with_glr_when_getting_matrix_then_returns_matrix() {
    // Given
    let profile = parser_feature_profile_for_runtime();

    // When
    let matrix = bdd_governance_matrix_for_runtime2(BddPhase::Core, profile.glr);

    // Then
    assert_eq!(matrix.phase, BddPhase::Core);
}

// =============================================================================
// Describe Backend Tests
// =============================================================================

#[test]
fn given_glr_profile_when_describing_backend_then_returns_description() {
    // Given
    let profile = parser_feature_profile_for_runtime();

    // When
    let desc = describe_backend_for_conflicts(profile);

    // Then
    assert!(!desc.is_empty());
}

#[test]
fn given_custom_profile_when_describing_backend_then_returns_description() {
    // Given
    let profile = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    // When
    let desc = describe_backend_for_conflicts(profile);

    // Then
    assert!(!desc.is_empty());
}
