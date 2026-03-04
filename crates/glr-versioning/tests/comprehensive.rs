// Comprehensive tests for GLR versioning
use adze_glr_versioning::*;

// ---------------------------------------------------------------------------
// VersionInfo construction
// ---------------------------------------------------------------------------

#[test]
fn new_version_defaults() {
    let v = VersionInfo::new();
    assert!(!v.in_error);
    assert_eq!(v.cost, 0);
    assert_eq!(v.node_count, 0);
    assert_eq!(v.dynamic_prec, 0);
}

#[test]
fn default_matches_new() {
    let v1 = VersionInfo::new();
    let v2 = VersionInfo::default();
    assert_eq!(v1.in_error, v2.in_error);
    assert_eq!(v1.cost, v2.cost);
    assert_eq!(v1.dynamic_prec, v2.dynamic_prec);
}

// ---------------------------------------------------------------------------
// Dynamic precedence
// ---------------------------------------------------------------------------

#[test]
fn add_dynamic_prec_positive() {
    let mut v = VersionInfo::new();
    v.add_dynamic_prec(5);
    assert_eq!(v.dynamic_prec, 5);
}

#[test]
fn add_dynamic_prec_negative() {
    let mut v = VersionInfo::new();
    v.add_dynamic_prec(-3);
    assert_eq!(v.dynamic_prec, -3);
}

#[test]
fn add_dynamic_prec_accumulates() {
    let mut v = VersionInfo::new();
    v.add_dynamic_prec(5);
    v.add_dynamic_prec(3);
    assert_eq!(v.dynamic_prec, 8);
}

#[test]
fn add_dynamic_prec_mixed() {
    let mut v = VersionInfo::new();
    v.add_dynamic_prec(10);
    v.add_dynamic_prec(-7);
    assert_eq!(v.dynamic_prec, 3);
}

// ---------------------------------------------------------------------------
// Error state
// ---------------------------------------------------------------------------

#[test]
fn enter_error() {
    let mut v = VersionInfo::new();
    assert!(!v.in_error);
    v.enter_error();
    assert!(v.in_error);
}

#[test]
fn add_error_cost() {
    let mut v = VersionInfo::new();
    v.add_error_cost(10, 3);
    assert_eq!(v.cost, 10);
    assert_eq!(v.node_count, 3);
}

#[test]
fn add_error_cost_accumulates() {
    let mut v = VersionInfo::new();
    v.add_error_cost(5, 2);
    v.add_error_cost(3, 1);
    assert_eq!(v.cost, 8);
    assert_eq!(v.node_count, 3);
}

// ---------------------------------------------------------------------------
// CompareResult
// ---------------------------------------------------------------------------

#[test]
fn compare_result_eq() {
    assert_eq!(CompareResult::TakeLeft, CompareResult::TakeLeft);
    assert_eq!(CompareResult::TakeRight, CompareResult::TakeRight);
    assert_eq!(CompareResult::PreferLeft, CompareResult::PreferLeft);
    assert_eq!(CompareResult::PreferRight, CompareResult::PreferRight);
    assert_eq!(CompareResult::Tie, CompareResult::Tie);
}

#[test]
fn compare_result_ne() {
    assert_ne!(CompareResult::TakeLeft, CompareResult::TakeRight);
    assert_ne!(CompareResult::PreferLeft, CompareResult::Tie);
}

#[test]
fn compare_result_debug() {
    let d = format!("{:?}", CompareResult::TakeLeft);
    assert!(d.contains("TakeLeft"));
}

// ---------------------------------------------------------------------------
// compare_versions
// ---------------------------------------------------------------------------

#[test]
fn identical_versions_tie() {
    let a = VersionInfo::new();
    let b = VersionInfo::new();
    assert_eq!(compare_versions(&a, &b), CompareResult::Tie);
}

#[test]
fn error_vs_clean_takes_clean() {
    let clean = VersionInfo::new();
    let mut errored = VersionInfo::new();
    errored.enter_error();
    assert_eq!(compare_versions(&clean, &errored), CompareResult::TakeLeft);
    assert_eq!(compare_versions(&errored, &clean), CompareResult::TakeRight);
}

#[test]
fn both_error_lower_cost_wins() {
    let mut a = VersionInfo::new();
    a.enter_error();
    a.add_error_cost(5, 1);
    let mut b = VersionInfo::new();
    b.enter_error();
    b.add_error_cost(10, 2);
    let result = compare_versions(&a, &b);
    // a has lower cost, should be preferred
    assert!(result == CompareResult::TakeLeft || result == CompareResult::PreferLeft);
}

#[test]
fn higher_dynamic_prec_preferred() {
    let mut a = VersionInfo::new();
    a.add_dynamic_prec(10);
    let mut b = VersionInfo::new();
    b.add_dynamic_prec(5);
    let result = compare_versions(&a, &b);
    assert!(result == CompareResult::TakeLeft || result == CompareResult::PreferLeft);
}

#[test]
fn lower_dynamic_prec_loses() {
    let mut a = VersionInfo::new();
    a.add_dynamic_prec(5);
    let mut b = VersionInfo::new();
    b.add_dynamic_prec(10);
    let result = compare_versions(&a, &b);
    assert!(result == CompareResult::TakeRight || result == CompareResult::PreferRight);
}

#[test]
fn same_prec_ties() {
    let mut a = VersionInfo::new();
    a.add_dynamic_prec(5);
    let mut b = VersionInfo::new();
    b.add_dynamic_prec(5);
    assert_eq!(compare_versions(&a, &b), CompareResult::Tie);
}

#[test]
fn error_with_high_prec_still_loses_to_clean() {
    let clean = VersionInfo::new();
    let mut errored = VersionInfo::new();
    errored.enter_error();
    errored.add_dynamic_prec(100);
    assert_eq!(compare_versions(&clean, &errored), CompareResult::TakeLeft);
}

#[test]
fn both_error_same_cost_prec_breaks_tie() {
    let mut a = VersionInfo::new();
    a.enter_error();
    a.add_error_cost(5, 1);
    a.add_dynamic_prec(10);
    let mut b = VersionInfo::new();
    b.enter_error();
    b.add_error_cost(5, 1);
    b.add_dynamic_prec(5);
    let result = compare_versions(&a, &b);
    // Same cost but a has higher prec
    assert!(result == CompareResult::TakeLeft || result == CompareResult::PreferLeft);
}

// ---------------------------------------------------------------------------
// Clone
// ---------------------------------------------------------------------------

#[test]
fn version_clone() {
    let mut v = VersionInfo::new();
    v.add_dynamic_prec(5);
    v.enter_error();
    v.add_error_cost(3, 1);
    let v2 = v.clone();
    assert_eq!(v2.in_error, true);
    assert_eq!(v2.dynamic_prec, 5);
    assert_eq!(v2.cost, 3);
    assert_eq!(v2.node_count, 1);
}

#[test]
fn version_debug() {
    let v = VersionInfo::new();
    let d = format!("{:?}", v);
    assert!(d.contains("VersionInfo"));
}
