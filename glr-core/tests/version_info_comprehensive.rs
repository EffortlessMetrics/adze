//! Comprehensive tests for VersionInfo comparison and GLR version resolution.
//!
//! Covers: VersionInfo creation, mutation, compare_versions in all branches,
//! edge cases with extreme values, symmetry properties, and transitivity.

use adze_glr_core::version_info::{CompareResult, VersionInfo, compare_versions};

#[test]
fn version_info_default() {
    let v = VersionInfo::new();
    assert!(!v.in_error);
    assert_eq!(v.cost, 0);
    assert_eq!(v.node_count, 0);
    assert_eq!(v.dynamic_prec, 0);
}

#[test]
fn enter_error_sets_flag() {
    let mut v = VersionInfo::new();
    v.enter_error();
    assert!(v.in_error);
}

#[test]
fn add_dynamic_prec_accumulates() {
    let mut v = VersionInfo::new();
    v.add_dynamic_prec(5);
    v.add_dynamic_prec(3);
    assert_eq!(v.dynamic_prec, 8);
}

#[test]
fn add_dynamic_prec_negative() {
    let mut v = VersionInfo::new();
    v.add_dynamic_prec(-10);
    assert_eq!(v.dynamic_prec, -10);
}

#[test]
fn add_error_cost_accumulates() {
    let mut v = VersionInfo::new();
    v.add_error_cost(100, 2);
    v.add_error_cost(50, 1);
    assert_eq!(v.cost, 150);
    assert_eq!(v.node_count, 3);
}

// --- compare_versions: error vs non-error ---

#[test]
fn error_vs_nonerror_take_left() {
    let a = VersionInfo::new(); // not in error
    let mut b = VersionInfo::new();
    b.enter_error();
    assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
}

#[test]
fn error_vs_nonerror_take_right() {
    let mut a = VersionInfo::new();
    a.enter_error();
    let b = VersionInfo::new();
    assert_eq!(compare_versions(&a, &b), CompareResult::TakeRight);
}

#[test]
fn both_in_error_goes_to_cost() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.enter_error();
    b.enter_error();
    // Same cost, same prec → tie
    assert_eq!(compare_versions(&a, &b), CompareResult::Tie);
}

// --- compare_versions: cost branch ---

#[test]
fn large_cost_diff_takes() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_error_cost(0, 1);
    b.add_error_cost(10000, 1); // way over threshold
    assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
}

#[test]
fn small_cost_diff_prefers() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_error_cost(10, 1);
    b.add_error_cost(20, 1);
    assert_eq!(compare_versions(&a, &b), CompareResult::PreferLeft);
}

#[test]
fn cost_symmetry() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_error_cost(50, 1);
    b.add_error_cost(100, 1);

    let ab = compare_versions(&a, &b);
    let ba = compare_versions(&b, &a);

    // Should be symmetric/opposite
    match ab {
        CompareResult::PreferLeft => assert_eq!(ba, CompareResult::PreferRight),
        CompareResult::TakeLeft => assert_eq!(ba, CompareResult::TakeRight),
        _ => {}
    }
}

// --- compare_versions: dynamic precedence ---

#[test]
fn higher_dynamic_prec_wins() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_dynamic_prec(10);
    b.add_dynamic_prec(5);
    assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
}

#[test]
fn lower_dynamic_prec_loses() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_dynamic_prec(3);
    b.add_dynamic_prec(7);
    assert_eq!(compare_versions(&a, &b), CompareResult::TakeRight);
}

#[test]
fn equal_dynamic_prec_tie() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_dynamic_prec(5);
    b.add_dynamic_prec(5);
    assert_eq!(compare_versions(&a, &b), CompareResult::Tie);
}

// --- compare_versions: tie ---

#[test]
fn identical_versions_tie() {
    let a = VersionInfo::new();
    let b = VersionInfo::new();
    assert_eq!(compare_versions(&a, &b), CompareResult::Tie);
}

// --- CompareResult properties ---

#[test]
fn compare_result_debug() {
    let r = CompareResult::TakeLeft;
    let debug = format!("{:?}", r);
    assert!(debug.contains("TakeLeft"));
}

#[test]
fn compare_result_equality() {
    assert_eq!(CompareResult::TakeLeft, CompareResult::TakeLeft);
    assert_ne!(CompareResult::TakeLeft, CompareResult::TakeRight);
    assert_ne!(CompareResult::PreferLeft, CompareResult::PreferRight);
    assert_eq!(CompareResult::Tie, CompareResult::Tie);
}

// --- Edge cases ---

#[test]
fn zero_node_count_large_cost() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    // Zero nodes but with cost — threshold uses max(1)
    a.add_error_cost(0, 0);
    b.add_error_cost(5000, 0);
    // threshold = 18 * 100 * max(0, 1) = 1800
    // diff = 5000 >= 1800 → TakeLeft
    assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
}

#[test]
fn negative_dynamic_prec() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_dynamic_prec(-5);
    b.add_dynamic_prec(-10);
    assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
}

#[test]
fn version_info_clone() {
    let mut v = VersionInfo::new();
    v.add_dynamic_prec(42);
    v.enter_error();
    let v2 = v.clone();
    assert_eq!(v2.dynamic_prec, 42);
    assert!(v2.in_error);
}

#[test]
fn multiple_error_entries() {
    let mut v = VersionInfo::new();
    v.enter_error();
    v.enter_error(); // idempotent
    assert!(v.in_error);
}

#[test]
fn cost_priority_over_dynamic_prec() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    // a has lower cost but lower prec
    a.add_error_cost(10, 1);
    a.add_dynamic_prec(1);
    // b has higher cost but higher prec
    b.add_error_cost(20, 1);
    b.add_dynamic_prec(100);
    // Cost difference should be checked first
    let result = compare_versions(&a, &b);
    assert!(
        matches!(result, CompareResult::PreferLeft | CompareResult::TakeLeft),
        "Expected cost to take priority, got {:?}",
        result
    );
}
