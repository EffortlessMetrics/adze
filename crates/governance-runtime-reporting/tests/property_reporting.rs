//! Property-based tests for governance-runtime-reporting.

use proptest::prelude::*;

use adze_governance_runtime_reporting::{
    BddGovernanceMatrix, BddGovernanceSnapshot, BddPhase, GLR_CONFLICT_FALLBACK,
    GLR_CONFLICT_PRESERVATION_GRID, ParserFeatureProfile, bdd_governance_snapshot, bdd_progress,
    bdd_progress_report, bdd_progress_report_with_profile,
    bdd_progress_report_with_profile_runtime, bdd_progress_status_line,
    describe_backend_for_conflicts,
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
// 1 – bdd_progress_report_with_profile_runtime Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn runtime_report_contains_title(title in arb_title(), profile in arb_profile()) {
        let report = bdd_progress_report_with_profile_runtime(
            BddPhase::Runtime,
            GLR_CONFLICT_PRESERVATION_GRID,
            &title,
            profile,
        );
        prop_assert!(report.contains(&title));
    }

    #[test]
    fn runtime_report_contains_governance_status(phase in arb_phase(), profile in arb_profile()) {
        let report = bdd_progress_report_with_profile_runtime(
            phase,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Test",
            profile,
        );
        prop_assert!(report.contains("Governance status:"));
    }

    #[test]
    fn runtime_report_contains_feature_profile(phase in arb_phase(), profile in arb_profile()) {
        let report = bdd_progress_report_with_profile_runtime(
            phase,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Test",
            profile,
        );
        prop_assert!(report.contains("Feature profile:"));
    }

    #[test]
    fn runtime_report_contains_non_conflict_backend(phase in arb_phase(), profile in arb_profile()) {
        let report = bdd_progress_report_with_profile_runtime(
            phase,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Test",
            profile,
        );
        prop_assert!(report.contains("Non-conflict backend:"));
    }

    #[test]
    fn runtime_report_contains_conflict_profiles(phase in arb_phase(), profile in arb_profile()) {
        let report = bdd_progress_report_with_profile_runtime(
            phase,
            GLR_CONFLICT_PRESERVATION_GRID,
            "Test",
            profile,
        );
        prop_assert!(report.contains("Conflict profiles:"));
    }

    #[test]
    fn runtime_report_is_deterministic(phase in arb_phase(), profile in arb_profile(), title in arb_title()) {
        let report1 = bdd_progress_report_with_profile_runtime(
            phase,
            GLR_CONFLICT_PRESERVATION_GRID,
            &title,
            profile,
        );
        let report2 = bdd_progress_report_with_profile_runtime(
            phase,
            GLR_CONFLICT_PRESERVATION_GRID,
            &title,
            profile,
        );
        prop_assert_eq!(report1, report2);
    }
}

// ---------------------------------------------------------------------------
// 2 – bdd_progress Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn bdd_progress_returns_valid_counts(phase in arb_phase()) {
        let (implemented, total) = bdd_progress(phase, GLR_CONFLICT_PRESERVATION_GRID);
        prop_assert!(implemented <= total);
    }

    #[test]
    fn bdd_progress_empty_scenarios(phase in arb_phase()) {
        let (implemented, total) = bdd_progress(phase, &[]);
        prop_assert_eq!(implemented, 0);
        prop_assert_eq!(total, 0);
    }

    #[test]
    fn bdd_progress_is_deterministic(phase in arb_phase()) {
        let (impl1, total1) = bdd_progress(phase, GLR_CONFLICT_PRESERVATION_GRID);
        let (impl2, total2) = bdd_progress(phase, GLR_CONFLICT_PRESERVATION_GRID);
        prop_assert_eq!(impl1, impl2);
        prop_assert_eq!(total1, total2);
    }
}

// ---------------------------------------------------------------------------
// 3 – bdd_progress_status_line Tests
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

    #[test]
    fn status_line_contains_profile(phase in arb_phase(), profile in arb_profile()) {
        let status = bdd_progress_status_line(phase, GLR_CONFLICT_PRESERVATION_GRID, profile);
        let profile_str = format!("{}", profile);
        prop_assert!(status.contains(&profile_str));
    }

    #[test]
    fn status_line_is_deterministic(phase in arb_phase(), profile in arb_profile()) {
        let status1 = bdd_progress_status_line(phase, GLR_CONFLICT_PRESERVATION_GRID, profile);
        let status2 = bdd_progress_status_line(phase, GLR_CONFLICT_PRESERVATION_GRID, profile);
        prop_assert_eq!(status1, status2);
    }
}

// ---------------------------------------------------------------------------
// 4 – describe_backend_for_conflicts Tests
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
// 5 – bdd_governance_snapshot Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn snapshot_phase_preserved(phase in arb_phase(), profile in arb_profile()) {
        let snap = bdd_governance_snapshot(phase, GLR_CONFLICT_PRESERVATION_GRID, profile);
        prop_assert_eq!(snap.phase, phase);
    }

    #[test]
    fn snapshot_profile_preserved(phase in arb_phase(), profile in arb_profile()) {
        let snap = bdd_governance_snapshot(phase, GLR_CONFLICT_PRESERVATION_GRID, profile);
        prop_assert_eq!(snap.profile, profile);
    }

    #[test]
    fn snapshot_implemented_lte_total(phase in arb_phase(), profile in arb_profile()) {
        let snap = bdd_governance_snapshot(phase, GLR_CONFLICT_PRESERVATION_GRID, profile);
        prop_assert!(snap.implemented <= snap.total);
    }
}

// ---------------------------------------------------------------------------
// 6 – BddGovernanceMatrix Tests
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
}

// ---------------------------------------------------------------------------
// 7 – bdd_progress_report Tests
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
// 8 – Static Data Tests
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
// 9 – BddGovernanceSnapshot Direct Tests
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
}
