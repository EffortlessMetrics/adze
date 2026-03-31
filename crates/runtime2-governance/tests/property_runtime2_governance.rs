//! Property-based tests for runtime2-governance.

use proptest::prelude::*;

use adze_runtime2_governance::{
    BddGovernanceSnapshot, BddPhase, GLR_CONFLICT_FALLBACK, GLR_CONFLICT_PRESERVATION_GRID,
    ParserBackend, ParserFeatureProfile, bdd_governance_matrix_for_profile,
    bdd_governance_matrix_for_runtime2, bdd_governance_matrix_for_runtime2_profile,
    bdd_governance_snapshot, bdd_progress_report_for_profile,
    bdd_progress_report_for_runtime2_profile, bdd_progress_status_line_for_profile,
    bdd_progress_status_line_for_runtime2_profile, describe_backend_for_conflicts,
    parser_feature_profile_for_runtime2, resolve_backend_for_profile,
    resolve_backend_for_runtime2_profile, resolve_runtime2_backend, runtime2_governance_snapshot,
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

/// Generate arbitrary boolean flags for pure_rust toggle.
fn arb_pure_rust_flag() -> impl Strategy<Value = bool> {
    any::<bool>()
}

/// Generate arbitrary boolean flags for conflict toggle.
fn arb_conflict_flag() -> impl Strategy<Value = bool> {
    any::<bool>()
}

// ---------------------------------------------------------------------------
// 1 – parser_feature_profile_for_runtime2 Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn profile_for_runtime2_pure_rust_matches_flag(enabled in arb_pure_rust_flag()) {
        let profile = parser_feature_profile_for_runtime2(enabled);
        prop_assert_eq!(profile.pure_rust, enabled);
    }

    #[test]
    fn profile_for_runtime2_glr_matches_pure_rust(enabled in arb_pure_rust_flag()) {
        let profile = parser_feature_profile_for_runtime2(enabled);
        prop_assert_eq!(profile.glr, enabled);
    }

    #[test]
    fn profile_for_runtime2_tree_sitter_disabled_when_pure_rust(enabled in arb_pure_rust_flag()) {
        let profile = parser_feature_profile_for_runtime2(enabled);
        if enabled {
            prop_assert!(!profile.tree_sitter_standard);
            prop_assert!(!profile.tree_sitter_c2rust);
        }
    }

    #[test]
    fn profile_for_runtime2_is_deterministic(enabled in arb_pure_rust_flag()) {
        let profile1 = parser_feature_profile_for_runtime2(enabled);
        let profile2 = parser_feature_profile_for_runtime2(enabled);
        prop_assert_eq!(profile1, profile2);
    }
}

// ---------------------------------------------------------------------------
// 2 – resolve_backend_for_runtime2_profile Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn resolve_backend_glr_profile_returns_glr_for_conflict(profile in arb_profile(), has_conflict in arb_conflict_flag()) {
        if profile.glr && has_conflict {
            let backend = resolve_backend_for_runtime2_profile(profile, has_conflict);
            prop_assert_eq!(backend, ParserBackend::GLR);
        }
    }

    #[test]
    fn resolve_backend_glr_profile_returns_glr_for_no_conflict(profile in arb_profile(), has_conflict in arb_conflict_flag()) {
        if profile.glr && !has_conflict {
            let backend = resolve_backend_for_runtime2_profile(profile, has_conflict);
            prop_assert_eq!(backend, ParserBackend::GLR);
        }
    }

    #[test]
    fn resolve_backend_is_deterministic(profile in arb_profile(), has_conflict in arb_conflict_flag()) {
        // Skip invalid configurations: pure_rust with conflict but glr disabled
        prop_assume!(!(profile.pure_rust && has_conflict && !profile.glr));
        let backend1 = resolve_backend_for_runtime2_profile(profile, has_conflict);
        let backend2 = resolve_backend_for_runtime2_profile(profile, has_conflict);
        prop_assert_eq!(backend1, backend2);
    }
}

// ---------------------------------------------------------------------------
// 3 – resolve_runtime2_backend Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn resolve_runtime2_backend_glr_enabled_with_conflict(enabled in arb_pure_rust_flag(), has_conflict in arb_conflict_flag()) {
        let backend = resolve_runtime2_backend(enabled, has_conflict);
        if enabled {
            prop_assert_eq!(backend, ParserBackend::GLR);
        } else if has_conflict {
            prop_assert_eq!(backend, ParserBackend::TreeSitter);
        }
    }

    #[test]
    fn resolve_runtime2_backend_is_deterministic(enabled in arb_pure_rust_flag(), has_conflict in arb_conflict_flag()) {
        let backend1 = resolve_runtime2_backend(enabled, has_conflict);
        let backend2 = resolve_runtime2_backend(enabled, has_conflict);
        prop_assert_eq!(backend1, backend2);
    }
}

// ---------------------------------------------------------------------------
// 4 – bdd_progress_report_for_runtime2_profile Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn report_for_runtime2_contains_title(title in arb_title(), profile in arb_profile()) {
        let report = bdd_progress_report_for_runtime2_profile(BddPhase::Runtime, &title, profile);
        prop_assert!(report.contains(&title));
    }

    #[test]
    fn report_for_runtime2_contains_governance_status(phase in arb_phase(), profile in arb_profile()) {
        let report = bdd_progress_report_for_runtime2_profile(phase, "Test", profile);
        prop_assert!(report.contains("Governance status"));
    }

    #[test]
    fn report_for_runtime2_contains_feature_profile(phase in arb_phase(), profile in arb_profile()) {
        let report = bdd_progress_report_for_runtime2_profile(phase, "Test", profile);
        prop_assert!(report.contains("Feature profile:"));
    }

    #[test]
    fn report_for_runtime2_is_deterministic(phase in arb_phase(), title in arb_title(), profile in arb_profile()) {
        let report1 = bdd_progress_report_for_runtime2_profile(phase, &title, profile);
        let report2 = bdd_progress_report_for_runtime2_profile(phase, &title, profile);
        prop_assert_eq!(report1, report2);
    }
}

// ---------------------------------------------------------------------------
// 5 – bdd_progress_status_line_for_runtime2_profile Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn status_line_for_runtime2_starts_with_phase(phase in arb_phase(), profile in arb_profile()) {
        let status = bdd_progress_status_line_for_runtime2_profile(phase, profile);
        let prefix = match phase {
            BddPhase::Core => "core:",
            BddPhase::Runtime => "runtime:",
        };
        prop_assert!(status.starts_with(prefix));
    }

    #[test]
    fn status_line_for_runtime2_contains_profile(phase in arb_phase(), profile in arb_profile()) {
        let status = bdd_progress_status_line_for_runtime2_profile(phase, profile);
        let profile_str = format!("{}", profile);
        prop_assert!(status.contains(&profile_str));
    }

    #[test]
    fn status_line_for_runtime2_is_deterministic(phase in arb_phase(), profile in arb_profile()) {
        let status1 = bdd_progress_status_line_for_runtime2_profile(phase, profile);
        let status2 = bdd_progress_status_line_for_runtime2_profile(phase, profile);
        prop_assert_eq!(status1, status2);
    }
}

// ---------------------------------------------------------------------------
// 6 – runtime2_governance_snapshot Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn runtime2_snapshot_phase_preserved(phase in arb_phase(), profile in arb_profile()) {
        let snap = runtime2_governance_snapshot(phase, profile);
        prop_assert_eq!(snap.phase, phase);
    }

    #[test]
    fn runtime2_snapshot_profile_preserved(phase in arb_phase(), profile in arb_profile()) {
        let snap = runtime2_governance_snapshot(phase, profile);
        prop_assert_eq!(snap.profile, profile);
    }

    #[test]
    fn runtime2_snapshot_implemented_lte_total(phase in arb_phase(), profile in arb_profile()) {
        let snap = runtime2_governance_snapshot(phase, profile);
        prop_assert!(snap.implemented <= snap.total);
    }

    #[test]
    fn runtime2_snapshot_is_deterministic(phase in arb_phase(), profile in arb_profile()) {
        let snap1 = runtime2_governance_snapshot(phase, profile);
        let snap2 = runtime2_governance_snapshot(phase, profile);
        prop_assert_eq!(snap1.phase, snap2.phase);
        prop_assert_eq!(snap1.profile, snap2.profile);
        prop_assert_eq!(snap1.implemented, snap2.implemented);
        prop_assert_eq!(snap1.total, snap2.total);
    }
}

// ---------------------------------------------------------------------------
// 7 – bdd_governance_matrix_for_profile Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn matrix_for_profile_has_scenarios(phase in arb_phase(), profile in arb_profile()) {
        let matrix = bdd_governance_matrix_for_profile(phase, profile);
        prop_assert!(!matrix.scenarios.is_empty());
    }

    #[test]
    fn matrix_for_profile_phase_preserved(phase in arb_phase(), profile in arb_profile()) {
        let matrix = bdd_governance_matrix_for_profile(phase, profile);
        prop_assert_eq!(matrix.phase, phase);
    }

    #[test]
    fn matrix_for_profile_profile_preserved(phase in arb_phase(), profile in arb_profile()) {
        let matrix = bdd_governance_matrix_for_profile(phase, profile);
        prop_assert_eq!(matrix.profile, profile);
    }
}

// ---------------------------------------------------------------------------
// 8 – bdd_governance_matrix_for_runtime2 Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn matrix_for_runtime2_has_scenarios(phase in arb_phase(), enabled in arb_pure_rust_flag()) {
        let matrix = bdd_governance_matrix_for_runtime2(phase, enabled);
        prop_assert!(!matrix.scenarios.is_empty());
    }

    #[test]
    fn matrix_for_runtime2_phase_preserved(phase in arb_phase(), enabled in arb_pure_rust_flag()) {
        let matrix = bdd_governance_matrix_for_runtime2(phase, enabled);
        prop_assert_eq!(matrix.phase, phase);
    }
}

// ---------------------------------------------------------------------------
// 9 – bdd_governance_matrix_for_runtime2_profile Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn matrix_for_runtime2_profile_has_scenarios(phase in arb_phase(), enabled in arb_pure_rust_flag()) {
        let matrix = bdd_governance_matrix_for_runtime2_profile(phase, enabled);
        prop_assert!(!matrix.scenarios.is_empty());
    }

    #[test]
    fn matrix_for_runtime2_profile_phase_preserved(phase in arb_phase(), enabled in arb_pure_rust_flag()) {
        let matrix = bdd_governance_matrix_for_runtime2_profile(phase, enabled);
        prop_assert_eq!(matrix.phase, phase);
    }
}

// ---------------------------------------------------------------------------
// 10 – describe_backend_for_conflicts Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn describe_backend_for_runtime2_never_empty(profile in arb_profile()) {
        let desc = describe_backend_for_conflicts(profile);
        prop_assert!(!desc.is_empty());
    }

    #[test]
    fn describe_backend_for_runtime2_is_deterministic(profile in arb_profile()) {
        let desc1 = describe_backend_for_conflicts(profile);
        let desc2 = describe_backend_for_conflicts(profile);
        prop_assert_eq!(desc1, desc2);
    }
}

// ---------------------------------------------------------------------------
// 11 – resolve_backend_for_profile Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn resolve_backend_for_profile_is_deterministic(profile in arb_profile(), has_conflict in arb_conflict_flag()) {
        // Skip invalid configurations: pure_rust with conflict but glr disabled
        prop_assume!(!(profile.pure_rust && has_conflict && !profile.glr));
        let backend1 = resolve_backend_for_profile(profile, has_conflict);
        let backend2 = resolve_backend_for_profile(profile, has_conflict);
        prop_assert_eq!(backend1, backend2);
    }
}

// ---------------------------------------------------------------------------
// 12 – bdd_progress_status_line_for_profile Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn status_line_for_profile_starts_with_phase(phase in arb_phase(), profile in arb_profile()) {
        let status = bdd_progress_status_line_for_profile(phase, profile);
        let prefix = match phase {
            BddPhase::Core => "core:",
            BddPhase::Runtime => "runtime:",
        };
        prop_assert!(status.starts_with(prefix));
    }

    #[test]
    fn status_line_for_profile_is_deterministic(phase in arb_phase(), profile in arb_profile()) {
        let status1 = bdd_progress_status_line_for_profile(phase, profile);
        let status2 = bdd_progress_status_line_for_profile(phase, profile);
        prop_assert_eq!(status1, status2);
    }
}

// ---------------------------------------------------------------------------
// 13 – bdd_progress_report_for_profile Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn report_for_profile_contains_title(phase in arb_phase(), title in arb_title(), profile in arb_profile()) {
        let report = bdd_progress_report_for_profile(phase, &title, profile);
        prop_assert!(report.contains(&title));
    }

    #[test]
    fn report_for_profile_is_deterministic(phase in arb_phase(), title in arb_title(), profile in arb_profile()) {
        let report1 = bdd_progress_report_for_profile(phase, &title, profile);
        let report2 = bdd_progress_report_for_profile(phase, &title, profile);
        prop_assert_eq!(report1, report2);
    }
}

// ---------------------------------------------------------------------------
// 14 – bdd_governance_snapshot Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn bdd_snapshot_phase_preserved(phase in arb_phase(), profile in arb_profile()) {
        let snap = bdd_governance_snapshot(phase, GLR_CONFLICT_PRESERVATION_GRID, profile);
        prop_assert_eq!(snap.phase, phase);
    }

    #[test]
    fn bdd_snapshot_profile_preserved(phase in arb_phase(), profile in arb_profile()) {
        let snap = bdd_governance_snapshot(phase, GLR_CONFLICT_PRESERVATION_GRID, profile);
        prop_assert_eq!(snap.profile, profile);
    }
}

// ---------------------------------------------------------------------------
// 15 – Static Data Tests
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
// 16 – BddGovernanceSnapshot Direct Tests
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

// ---------------------------------------------------------------------------
// 17 – ParserBackend Tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parser_backend_display_is_non_empty(backend in prop_oneof![Just(ParserBackend::GLR), Just(ParserBackend::TreeSitter)]) {
        let display = format!("{}", backend);
        prop_assert!(!display.is_empty());
    }
}
