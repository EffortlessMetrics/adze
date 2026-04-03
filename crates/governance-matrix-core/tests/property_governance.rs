//! Property-based tests for governance-matrix-core facade.
//! Note: Uses manual test functions to avoid compiler ICE with proptest macro.

use adze_governance_matrix_core::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, GLR_CONFLICT_PRESERVATION_GRID,
    ParserFeatureProfile, bdd_progress, bdd_progress_status_line,
};

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn create_test_profiles() -> Vec<ParserFeatureProfile> {
    vec![
        ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        },
        ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: false,
        },
        ParserFeatureProfile {
            pure_rust: false,
            tree_sitter_standard: true,
            tree_sitter_c2rust: false,
            glr: false,
        },
        ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: false,
            glr: true,
        },
        ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: true,
            tree_sitter_c2rust: true,
            glr: true,
        },
    ]
}

// ---------------------------------------------------------------------------
// 1 – BddGovernanceSnapshot tests
// ---------------------------------------------------------------------------

#[test]
fn snapshot_is_fully_implemented_various_counts() {
    let profile = ParserFeatureProfile::current();

    // 0/0 is fully implemented
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Core,
        implemented: 0,
        total: 0,
        profile,
    };
    assert!(snap.is_fully_implemented());

    // Equal counts are fully implemented
    for count in [1, 5, 10, 100] {
        let snap = BddGovernanceSnapshot {
            phase: BddPhase::Core,
            implemented: count,
            total: count,
            profile,
        };
        assert!(snap.is_fully_implemented());
    }

    // Partial implementation is not fully implemented
    for (impl_count, total) in [(0, 1), (3, 5), (9, 10), (99, 100)] {
        let snap = BddGovernanceSnapshot {
            phase: BddPhase::Core,
            implemented: impl_count,
            total,
            profile,
        };
        assert!(!snap.is_fully_implemented());
    }
}

#[test]
fn snapshot_copy_preserves_fields() {
    let profile = ParserFeatureProfile::current();
    let snap = BddGovernanceSnapshot {
        phase: BddPhase::Runtime,
        implemented: 7,
        total: 10,
        profile,
    };
    let snap2 = snap;

    assert_eq!(snap.phase, snap2.phase);
    assert_eq!(snap.implemented, snap2.implemented);
    assert_eq!(snap.total, snap2.total);
    assert_eq!(snap.profile, snap2.profile);
}

// ---------------------------------------------------------------------------
// 2 – BddGovernanceMatrix tests
// ---------------------------------------------------------------------------

#[test]
fn matrix_standard_never_panics_for_all_profiles() {
    for profile in create_test_profiles() {
        let _matrix = BddGovernanceMatrix::standard(profile);
    }
}

#[test]
fn matrix_standard_has_scenarios_for_all_profiles() {
    for profile in create_test_profiles() {
        let matrix = BddGovernanceMatrix::standard(profile);
        assert!(!matrix.scenarios.is_empty());
    }
}

#[test]
fn matrix_report_never_panics_for_all_profiles() {
    for profile in create_test_profiles() {
        let matrix = BddGovernanceMatrix::standard(profile);
        let _report = matrix.report("Test Matrix");
    }
}

#[test]
fn matrix_status_line_never_panics_for_all_profiles() {
    for profile in create_test_profiles() {
        let matrix = BddGovernanceMatrix::standard(profile);
        let status = matrix.status_line();
        assert!(!status.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 3 – bdd_progress function tests
// ---------------------------------------------------------------------------

#[test]
fn bdd_progress_empty_slice_returns_zeros() {
    for phase in [BddPhase::Core, BddPhase::Runtime] {
        let (impl_count, total) = bdd_progress(phase, &[]);
        assert_eq!(impl_count, 0);
        assert_eq!(total, 0);
    }
}

// ---------------------------------------------------------------------------
// 4 – bdd_progress_status_line tests
// ---------------------------------------------------------------------------

#[test]
fn status_line_starts_with_correct_phase_prefix() {
    for profile in create_test_profiles() {
        for phase in [BddPhase::Core, BddPhase::Runtime] {
            let status = bdd_progress_status_line(phase, GLR_CONFLICT_PRESERVATION_GRID, profile);
            let prefix = match phase {
                BddPhase::Core => "core:",
                BddPhase::Runtime => "runtime:",
            };
            assert!(
                status.starts_with(prefix),
                "status '{}' should start with '{}'",
                status,
                prefix
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 5 – Phase equality tests
// ---------------------------------------------------------------------------

#[test]
fn phase_eq_reflexive() {
    assert_eq!(BddPhase::Core, BddPhase::Core);
    assert_eq!(BddPhase::Runtime, BddPhase::Runtime);
}

#[test]
fn phase_neq_core_runtime() {
    assert_ne!(BddPhase::Core, BddPhase::Runtime);
}

// ---------------------------------------------------------------------------
// 6 – Profile consistency with matrix
// ---------------------------------------------------------------------------

#[test]
fn matrix_snapshot_preserves_profile_for_all_profiles() {
    for profile in create_test_profiles() {
        let matrix = BddGovernanceMatrix::standard(profile);
        let snap = matrix.snapshot();
        assert_eq!(snap.profile, profile);
    }
}
