//! Property-based tests for governance-matrix-core-impl facade.

use proptest::prelude::*;

use adze_governance_matrix_core_impl::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, GLR_CONFLICT_FALLBACK,
    GLR_CONFLICT_PRESERVATION_GRID, ParserFeatureProfile, bdd_progress, bdd_progress_report,
    bdd_progress_report_with_profile, bdd_progress_status_line, describe_backend_for_conflicts,
};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate arbitrary ParserFeatureProfile values.
fn arb_profile() -> impl Strategy<Value = ParserFeatureProfile> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(pure_rust, tree_sitter_standard, tree_sitter_c2rust, glr)| ParserFeatureProfile {
            pure_rust,
            tree_sitter_standard,
            tree_sitter_c2rust,
            glr,
        },
    )
}

/// Generate arbitrary BddPhase values.
fn arb_phase() -> impl Strategy<Value = BddPhase> {
    prop_oneof![Just(BddPhase::Core), Just(BddPhase::Runtime),]
}

/// Generate arbitrary non-empty strings for titles.
fn arb_title() -> impl Strategy<Value = String> {
    ".{1,20}".prop_filter("non-empty", |s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// 1 – BddGovernanceSnapshot tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn snapshot_is_fully_implemented(total in 0usize..100usize, impl_count in 0usize..100usize) {
        let profile = ParserFeatureProfile::current();
        // Ensure impl_count <= total for valid snapshot
        let impl_count = impl_count.min(total);
        let snap = BddGovernanceSnapshot {
            phase: BddPhase::Core,
            implemented: impl_count,
            total,
            profile,
        };

        // 0/0 or impl_count == total means fully implemented
        if total == 0 || impl_count == total {
            prop_assert!(snap.is_fully_implemented());
        } else {
            prop_assert!(!snap.is_fully_implemented());
        }
    }

    #[test]
    fn snapshot_phase_preserved(phase in arb_phase(), profile in arb_profile()) {
        let snap = BddGovernanceSnapshot {
            phase,
            implemented: 5,
            total: 10,
            profile,
        };
        prop_assert_eq!(snap.phase, phase);
    }
}

// ---------------------------------------------------------------------------
// 2 – BddGovernanceMatrix tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn matrix_standard_never_panics(profile in arb_profile()) {
        let _matrix = BddGovernanceMatrix::standard(profile);
    }

    #[test]
    fn matrix_standard_has_scenarios(profile in arb_profile()) {
        let matrix = BddGovernanceMatrix::standard(profile);
        prop_assert!(!matrix.scenarios.is_empty());
    }

    #[test]
    fn matrix_snapshot_profile_matches(profile in arb_profile()) {
        let matrix = BddGovernanceMatrix::standard(profile);
        let snap = matrix.snapshot();
        prop_assert_eq!(snap.profile, profile);
    }

    #[test]
    fn matrix_new_preserves_fields(phase in arb_phase(), profile in arb_profile()) {
        let matrix = BddGovernanceMatrix::new(phase, profile, GLR_CONFLICT_PRESERVATION_GRID);
        prop_assert_eq!(matrix.phase, phase);
        prop_assert_eq!(matrix.profile, profile);
    }
}

// ---------------------------------------------------------------------------
// 3 – Report functions tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn bdd_progress_report_contains_title(title in arb_title()) {
        let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, &title);
        prop_assert!(report.contains(&title));
    }

    #[test]
    fn bdd_progress_report_with_profile_contains_title(title in arb_title(), profile in arb_profile()) {
        let report = bdd_progress_report_with_profile(
            BddPhase::Runtime,
            GLR_CONFLICT_PRESERVATION_GRID,
            &title,
            profile,
        );
        prop_assert!(report.contains(&title));
    }
}

// ---------------------------------------------------------------------------
// 4 – Status line tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn status_line_starts_with_phase(phase in arb_phase(), profile in arb_profile()) {
        let status = bdd_progress_status_line(phase, GLR_CONFLICT_PRESERVATION_GRID, profile);
        let prefix = match phase {
            BddPhase::Core => "core:",
            BddPhase::Runtime => "runtime:",
        };
        prop_assert!(status.starts_with(prefix));
    }
}

// ---------------------------------------------------------------------------
// 5 – describe_backend_for_conflicts tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn describe_backend_never_empty(profile in arb_profile()) {
        let desc = describe_backend_for_conflicts(profile);
        prop_assert!(!desc.is_empty());
    }

    #[test]
    fn describe_backend_is_deterministic(profile in arb_profile()) {
        let desc1 = describe_backend_for_conflicts(profile);
        let desc2 = describe_backend_for_conflicts(profile);
        prop_assert_eq!(desc1, desc2);
    }
}

// ---------------------------------------------------------------------------
// 6 – Static data tests
// ---------------------------------------------------------------------------

#[test]
fn glr_conflict_fallback_is_non_empty() {
    assert!(!GLR_CONFLICT_FALLBACK.is_empty());
}

#[test]
fn glr_conflict_preservation_grid_is_non_empty() {
    assert!(!GLR_CONFLICT_PRESERVATION_GRID.is_empty());
}

// ---------------------------------------------------------------------------
// 7 – bdd_progress function tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn bdd_progress_empty_slice_returns_zeros(phase in arb_phase()) {
        let (impl_count, total) = bdd_progress(phase, &[]);
        prop_assert_eq!(impl_count, 0);
        prop_assert_eq!(total, 0);
    }
}
