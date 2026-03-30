//! Property-based tests for governance-runtime-core.

use proptest::prelude::*;

use adze_governance_runtime_core::{
    BddGovernanceMatrix, BddPhase, ParserBackend, ParserFeatureProfile,
    bdd_governance_matrix_for_profile, bdd_governance_matrix_for_runtime2,
    bdd_progress_report_for_profile, bdd_progress_status_line_for_profile,
    parser_feature_profile_for_runtime2, resolve_backend_for_profile,
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

/// Generate arbitrary pure_rust_glr toggle.
fn arb_glr_toggle() -> impl Strategy<Value = bool> {
    any::<bool>()
}

// ---------------------------------------------------------------------------
// 1 – parser_feature_profile_for_runtime2 tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn runtime2_profile_pure_rust_equals_glr(pure_rust_glr in arb_glr_toggle()) {
        let profile = parser_feature_profile_for_runtime2(pure_rust_glr);
        prop_assert_eq!(profile.pure_rust, pure_rust_glr);
        prop_assert_eq!(profile.glr, pure_rust_glr);
    }

    #[test]
    fn runtime2_profile_tree_sitter_flags_off(pure_rust_glr in arb_glr_toggle()) {
        let profile = parser_feature_profile_for_runtime2(pure_rust_glr);
        prop_assert!(!profile.tree_sitter_standard);
        prop_assert!(!profile.tree_sitter_c2rust);
    }

    #[test]
    fn runtime2_profile_copy_preserves_fields(pure_rust_glr in arb_glr_toggle()) {
        let profile = parser_feature_profile_for_runtime2(pure_rust_glr);
        let profile2 = profile;
        prop_assert_eq!(profile, profile2);
    }
}

// ---------------------------------------------------------------------------
// 2 – resolve_backend_for_profile tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn resolve_backend_glr_takes_priority(profile in arb_profile()) {
        if profile.glr {
            let backend = resolve_backend_for_profile(profile, false);
            prop_assert_eq!(backend, ParserBackend::GLR);
            let backend_conflict = resolve_backend_for_profile(profile, true);
            prop_assert_eq!(backend_conflict, ParserBackend::GLR);
        }
    }

    #[test]
    fn resolve_backend_matches_profile_method(profile in arb_profile(), has_conflicts in any::<bool>()) {
        // Skip profiles that would panic (pure_rust with conflicts but no glr)
        if profile.pure_rust && has_conflicts && !profile.glr {
            return Ok(());
        }
        let backend = resolve_backend_for_profile(profile, has_conflicts);
        let expected = profile.resolve_backend(has_conflicts);
        prop_assert_eq!(backend, expected);
    }
}

// ---------------------------------------------------------------------------
// 3 – bdd_governance_matrix_for_profile tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn matrix_for_profile_preserves_phase(phase in arb_phase(), profile in arb_profile()) {
        let matrix = bdd_governance_matrix_for_profile(phase, profile);
        prop_assert_eq!(matrix.phase, phase);
    }

    #[test]
    fn matrix_for_profile_preserves_profile(phase in arb_phase(), profile in arb_profile()) {
        let matrix = bdd_governance_matrix_for_profile(phase, profile);
        prop_assert_eq!(matrix.profile, profile);
    }

    #[test]
    fn matrix_for_profile_has_scenarios(phase in arb_phase(), profile in arb_profile()) {
        let matrix = bdd_governance_matrix_for_profile(phase, profile);
        prop_assert!(!matrix.scenarios.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 4 – bdd_governance_matrix_for_runtime2 tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn matrix_for_runtime2_preserves_phase(phase in arb_phase(), pure_rust_glr in arb_glr_toggle()) {
        let matrix = bdd_governance_matrix_for_runtime2(phase, pure_rust_glr);
        prop_assert_eq!(matrix.phase, phase);
    }

    #[test]
    fn matrix_for_runtime2_profile_matches(phase in arb_phase(), pure_rust_glr in arb_glr_toggle()) {
        let matrix = bdd_governance_matrix_for_runtime2(phase, pure_rust_glr);
        let expected_profile = parser_feature_profile_for_runtime2(pure_rust_glr);
        prop_assert_eq!(matrix.profile, expected_profile);
    }
}

// ---------------------------------------------------------------------------
// 5 – bdd_progress_report_for_profile tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn report_for_profile_contains_title(phase in arb_phase(), profile in arb_profile()) {
        let title = "Test Report";
        let report = bdd_progress_report_for_profile(phase, title, profile);
        prop_assert!(report.contains(title));
    }
}

// ---------------------------------------------------------------------------
// 6 – bdd_progress_status_line_for_profile tests
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
    fn status_line_for_profile_non_empty(phase in arb_phase(), profile in arb_profile()) {
        let status = bdd_progress_status_line_for_profile(phase, profile);
        prop_assert!(!status.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 7 – BddGovernanceMatrix consistency tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn matrix_snapshot_matches_constructor(phase in arb_phase(), profile in arb_profile()) {
        let matrix = bdd_governance_matrix_for_profile(phase, profile);
        let snap = matrix.snapshot();
        prop_assert_eq!(snap.phase, phase);
        prop_assert_eq!(snap.profile, profile);
    }
}
