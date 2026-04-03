//! Property-based tests for glr-versioning.

use proptest::prelude::*;

use adze_glr_versioning::{CompareResult, VersionInfo, compare_versions};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate arbitrary VersionInfo values.
fn arb_version_info() -> impl Strategy<Value = VersionInfo> {
    (
        any::<bool>(),
        0usize..10_000usize,
        0usize..10_000usize,
        -1_000_000i32..1_000_000i32,
    )
        .prop_map(|(in_error, cost, node_count, dynamic_prec)| VersionInfo {
            in_error,
            cost,
            node_count,
            dynamic_prec,
        })
}

/// Generate VersionInfo with error state.
fn arb_version_info_in_error() -> impl Strategy<Value = VersionInfo> {
    (
        0usize..10_000usize,
        0usize..10_000usize,
        -1_000_000i32..1_000_000i32,
    )
        .prop_map(|(cost, node_count, dynamic_prec)| {
            let mut v = VersionInfo::new();
            v.enter_error();
            v.add_error_cost(cost, node_count);
            v.add_dynamic_prec(dynamic_prec);
            v
        })
}

/// Generate VersionInfo without error state.
fn arb_version_info_no_error() -> impl Strategy<Value = VersionInfo> {
    (
        0usize..10_000usize,
        0usize..10_000usize,
        -1_000_000i32..1_000_000i32,
    )
        .prop_map(|(cost, node_count, dynamic_prec)| {
            let mut v = VersionInfo::new();
            v.add_error_cost(cost, node_count);
            v.add_dynamic_prec(dynamic_prec);
            v
        })
}

// ---------------------------------------------------------------------------
// 1 – VersionInfo tests
// ---------------------------------------------------------------------------

#[test]
fn version_info_default_not_in_error() {
    let v = VersionInfo::new();
    assert!(!v.in_error);
    assert_eq!(v.cost, 0);
    assert_eq!(v.node_count, 0);
    assert_eq!(v.dynamic_prec, 0);
}

proptest! {
    #[test]
    fn version_info_clone_equals_original(v in arb_version_info()) {
        let cloned = v.clone();
        prop_assert_eq!(v.in_error, cloned.in_error);
        prop_assert_eq!(v.cost, cloned.cost);
        prop_assert_eq!(v.node_count, cloned.node_count);
        prop_assert_eq!(v.dynamic_prec, cloned.dynamic_prec);
    }

    #[test]
    fn version_info_debug_non_empty(v in arb_version_info()) {
        let debug = format!("{:?}", v);
        prop_assert!(!debug.is_empty());
    }

    #[test]
    fn add_dynamic_prec_accumulates(
        mut v in arb_version_info(),
        prec in -1_000_000i32..1_000_000i32
    ) {
        let original = v.dynamic_prec;
        v.add_dynamic_prec(prec);
        prop_assert_eq!(v.dynamic_prec, original.wrapping_add(prec));
    }

    #[test]
    fn enter_error_sets_flag(mut v in arb_version_info()) {
        v.enter_error();
        prop_assert!(v.in_error);
    }

    #[test]
    fn add_error_cost_accumulates(mut v in arb_version_info(), cost in any::<usize>(), nodes in any::<usize>()) {
        let original_cost = v.cost;
        let original_nodes = v.node_count;
        v.add_error_cost(cost, nodes);
        prop_assert_eq!(v.cost, original_cost.saturating_add(cost));
        prop_assert_eq!(v.node_count, original_nodes.saturating_add(nodes));
    }
}

// ---------------------------------------------------------------------------
// 2 – CompareResult tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compare_result_debug_non_empty(result in arb_compare_result()) {
        let debug = format!("{:?}", result);
        prop_assert!(!debug.is_empty());
    }

    #[test]
    fn compare_result_eq_reflexive(result in arb_compare_result()) {
        prop_assert_eq!(&result, &result);
    }
}

/// Generate arbitrary CompareResult values.
fn arb_compare_result() -> impl Strategy<Value = CompareResult> {
    any::<u8>().prop_map(|choice| match choice % 5 {
        0 => CompareResult::TakeLeft,
        1 => CompareResult::TakeRight,
        2 => CompareResult::PreferLeft,
        3 => CompareResult::PreferRight,
        _ => CompareResult::Tie,
    })
}

// ---------------------------------------------------------------------------
// 3 – compare_versions tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compare_versions_non_error_beats_error(
        clean in arb_version_info_no_error(),
        errored in arb_version_info_in_error()
    ) {
        // Non-error always wins over error
        prop_assert_eq!(compare_versions(&clean, &errored), CompareResult::TakeLeft);
        prop_assert_eq!(compare_versions(&errored, &clean), CompareResult::TakeRight);
    }

    #[test]
    fn compare_versions_same_is_tie(v in arb_version_info()) {
        // Same version should be a tie
        let result = compare_versions(&v, &v);
        prop_assert_eq!(result, CompareResult::Tie);
    }

    #[test]
    fn compare_versions_higher_dynamic_prec_wins(
        prec_a in -1000i32..1000,
        prec_b in -1000i32..1000
    ) {
        let mut a = VersionInfo::new();
        let mut b = VersionInfo::new();
        a.add_dynamic_prec(prec_a);
        b.add_dynamic_prec(prec_b);

        let result = compare_versions(&a, &b);

        if prec_a > prec_b {
            prop_assert_eq!(result, CompareResult::TakeLeft);
        } else if prec_a < prec_b {
            prop_assert_eq!(result, CompareResult::TakeRight);
        } else {
            prop_assert_eq!(result, CompareResult::Tie);
        }
    }

    #[test]
    fn compare_versions_error_both_uses_cost(
        cost_a in 0usize..10000,
        cost_b in 0usize..10000
    ) {
        let mut a = VersionInfo::new();
        let mut b = VersionInfo::new();
        a.enter_error();
        b.enter_error();
        a.add_error_cost(cost_a, 1);
        b.add_error_cost(cost_b, 1);

        let result = compare_versions(&a, &b);

        // With node_count=1, threshold = 18 * 100 * 2 = 3600
        let cost_diff = cost_a.abs_diff(cost_b);
        let threshold = 3600;

        if cost_diff >= threshold {
            // Large cost difference: take unconditionally
            if cost_a < cost_b {
                prop_assert_eq!(result, CompareResult::TakeLeft);
            } else if cost_a > cost_b {
                prop_assert_eq!(result, CompareResult::TakeRight);
            }
        } else if cost_a != cost_b {
            // Small cost difference: prefer
            if cost_a < cost_b {
                prop_assert_eq!(result, CompareResult::PreferLeft);
            } else {
                prop_assert_eq!(result, CompareResult::PreferRight);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4 – Symmetry and transitivity tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn compare_versions_symmetric_tie(a in arb_version_info_no_error(), b in arb_version_info_no_error()) {
        // When both are equal (same values), result should be symmetric
        let result_ab = compare_versions(&a, &b);
        let result_ba = compare_versions(&b, &a);

        match (result_ab, result_ba) {
            (CompareResult::TakeLeft, CompareResult::TakeRight) => {}
            (CompareResult::TakeRight, CompareResult::TakeLeft) => {}
            (CompareResult::PreferLeft, CompareResult::PreferRight) => {}
            (CompareResult::PreferRight, CompareResult::PreferLeft) => {}
            (CompareResult::Tie, CompareResult::Tie) => {}
            _ => prop_assert!(false, "Results should be symmetric"),
        }
        prop_assert!(true);
    }
}
