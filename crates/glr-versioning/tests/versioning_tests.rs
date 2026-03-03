use adze_glr_versioning::{CompareResult, VersionInfo, compare_versions};

#[test]
fn default_version_info_has_zero_fields() {
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
    v.add_dynamic_prec(3);
    v.add_dynamic_prec(-1);
    assert_eq!(v.dynamic_prec, 2);
}

#[test]
fn add_error_cost_accumulates() {
    let mut v = VersionInfo::new();
    v.add_error_cost(100, 2);
    v.add_error_cost(50, 1);
    assert_eq!(v.cost, 150);
    assert_eq!(v.node_count, 3);
}

#[test]
fn non_error_beats_error_left() {
    let clean = VersionInfo::new();
    let mut errored = VersionInfo::new();
    errored.enter_error();
    assert_eq!(compare_versions(&clean, &errored), CompareResult::TakeLeft);
}

#[test]
fn non_error_beats_error_right() {
    let mut errored = VersionInfo::new();
    errored.enter_error();
    let clean = VersionInfo::new();
    assert_eq!(compare_versions(&errored, &clean), CompareResult::TakeRight);
}

#[test]
fn both_in_error_falls_through_to_cost() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.enter_error();
    b.enter_error();
    a.add_error_cost(100, 1);
    b.add_error_cost(200, 1);
    // Small cost diff -> PreferLeft
    assert_eq!(compare_versions(&a, &b), CompareResult::PreferLeft);
}

#[test]
fn large_cost_difference_takes_unconditionally() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_error_cost(0, 1);
    b.add_error_cost(10_000, 1);
    assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
}

#[test]
fn higher_dynamic_prec_wins() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_dynamic_prec(10);
    b.add_dynamic_prec(5);
    assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
    assert_eq!(compare_versions(&b, &a), CompareResult::TakeRight);
}

#[test]
fn identical_versions_tie() {
    let a = VersionInfo::new();
    let b = VersionInfo::new();
    assert_eq!(compare_versions(&a, &b), CompareResult::Tie);
}
