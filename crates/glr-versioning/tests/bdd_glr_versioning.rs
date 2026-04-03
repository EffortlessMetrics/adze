//! BDD tests for glr-versioning crate.
//!
//! These tests verify the public API behavior using Given/When/Then style.

use adze_glr_versioning::{CompareResult, VersionInfo, compare_versions};

// =============================================================================
// VersionInfo Creation Tests
// =============================================================================

#[test]
fn given_new_version_info_when_checking_defaults_then_not_in_error() {
    // Given / When
    let version = VersionInfo::new();

    // Then
    assert!(!version.in_error);
    assert_eq!(version.cost, 0);
    assert_eq!(version.node_count, 0);
    assert_eq!(version.dynamic_prec, 0);
}

#[test]
fn given_default_version_info_when_checking_defaults_then_all_zero() {
    // Given / When
    let version = VersionInfo::default();

    // Then
    assert!(!version.in_error);
    assert_eq!(version.cost, 0);
    assert_eq!(version.node_count, 0);
    assert_eq!(version.dynamic_prec, 0);
}

// =============================================================================
// VersionInfo Error State Tests
// =============================================================================

#[test]
fn given_version_info_when_entering_error_then_in_error_is_true() {
    // Given
    let mut version = VersionInfo::new();

    // When
    version.enter_error();

    // Then
    assert!(version.in_error);
}

// =============================================================================
// VersionInfo Dynamic Precedence Tests
// =============================================================================

#[test]
fn given_version_info_when_adding_dynamic_prec_then_value_accumulates() {
    // Given
    let mut version = VersionInfo::new();

    // When
    version.add_dynamic_prec(5);

    // Then
    assert_eq!(version.dynamic_prec, 5);
}

#[test]
fn given_version_info_with_prec_when_adding_more_prec_then_sums() {
    // Given
    let mut version = VersionInfo::new();
    version.add_dynamic_prec(5);

    // When
    version.add_dynamic_prec(3);

    // Then
    assert_eq!(version.dynamic_prec, 8);
}

#[test]
fn given_version_info_when_adding_negative_prec_then_decreases() {
    // Given
    let mut version = VersionInfo::new();
    version.add_dynamic_prec(10);

    // When
    version.add_dynamic_prec(-3);

    // Then
    assert_eq!(version.dynamic_prec, 7);
}

// =============================================================================
// VersionInfo Error Cost Tests
// =============================================================================

#[test]
fn given_version_info_when_adding_error_cost_then_accumulates() {
    // Given
    let mut version = VersionInfo::new();

    // When
    version.add_error_cost(100, 5);

    // Then
    assert_eq!(version.cost, 100);
    assert_eq!(version.node_count, 5);
}

#[test]
fn given_version_info_with_cost_when_adding_more_cost_then_sums() {
    // Given
    let mut version = VersionInfo::new();
    version.add_error_cost(100, 5);

    // When
    version.add_error_cost(50, 3);

    // Then
    assert_eq!(version.cost, 150);
    assert_eq!(version.node_count, 8);
}

// =============================================================================
// compare_versions Error Preference Tests
// =============================================================================

#[test]
fn given_clean_vs_error_when_comparing_then_takes_clean() {
    // Given
    let clean = VersionInfo::new();
    let mut errored = VersionInfo::new();
    errored.enter_error();

    // When
    let result = compare_versions(&clean, &errored);

    // Then
    assert_eq!(result, CompareResult::TakeLeft);
}

#[test]
fn given_error_vs_clean_when_comparing_then_takes_clean() {
    // Given
    let mut errored = VersionInfo::new();
    errored.enter_error();
    let clean = VersionInfo::new();

    // When
    let result = compare_versions(&errored, &clean);

    // Then
    assert_eq!(result, CompareResult::TakeRight);
}

#[test]
fn given_both_in_error_when_comparing_then_continues_to_cost() {
    // Given
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.enter_error();
    b.enter_error();
    a.add_error_cost(100, 1);
    b.add_error_cost(200, 1);

    // When
    let result = compare_versions(&a, &b);

    // Then
    assert_eq!(result, CompareResult::PreferLeft);
}

// =============================================================================
// compare_versions Cost Comparison Tests
// =============================================================================

#[test]
fn given_large_cost_difference_when_comparing_then_takes_lower_cost() {
    // Given
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_error_cost(0, 1);
    b.add_error_cost(5000, 1);

    // When
    let result = compare_versions(&a, &b);

    // Then - threshold is 18 * 100 * 2 = 3600, so 5000 > 3600
    assert_eq!(result, CompareResult::TakeLeft);
}

#[test]
fn given_small_cost_difference_when_comparing_then_prefers_lower_cost() {
    // Given
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_error_cost(100, 1);
    b.add_error_cost(200, 1);

    // When
    let result = compare_versions(&a, &b);

    // Then - difference is small, so prefer but keep both
    assert_eq!(result, CompareResult::PreferLeft);
}

#[test]
fn given_equal_costs_when_comparing_then_continues_to_precedence() {
    // Given
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_error_cost(100, 1);
    b.add_error_cost(100, 1);
    a.add_dynamic_prec(5);
    b.add_dynamic_prec(3);

    // When
    let result = compare_versions(&a, &b);

    // Then
    assert_eq!(result, CompareResult::TakeLeft);
}

// =============================================================================
// compare_versions Dynamic Precedence Tests
// =============================================================================

#[test]
fn given_higher_prec_when_comparing_then_takes_higher() {
    // Given
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_dynamic_prec(5);
    b.add_dynamic_prec(3);

    // When
    let result = compare_versions(&a, &b);

    // Then
    assert_eq!(result, CompareResult::TakeLeft);
}

#[test]
fn given_lower_prec_when_comparing_then_takes_higher() {
    // Given
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_dynamic_prec(3);
    b.add_dynamic_prec(5);

    // When
    let result = compare_versions(&a, &b);

    // Then
    assert_eq!(result, CompareResult::TakeRight);
}

#[test]
fn given_cumulative_prec_when_comparing_then_sums_before_compare() {
    // Given
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_dynamic_prec(2);
    a.add_dynamic_prec(3); // total 5
    b.add_dynamic_prec(4); // total 4

    // When
    let result = compare_versions(&a, &b);

    // Then
    assert_eq!(result, CompareResult::TakeLeft);
}

// =============================================================================
// compare_versions Tie Tests
// =============================================================================

#[test]
fn given_identical_versions_when_comparing_then_returns_tie() {
    // Given
    let a = VersionInfo::new();
    let b = VersionInfo::new();

    // When
    let result = compare_versions(&a, &b);

    // Then
    assert_eq!(result, CompareResult::Tie);
}

#[test]
fn given_equal_prec_and_cost_when_comparing_then_returns_tie() {
    // Given
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();
    a.add_dynamic_prec(5);
    a.add_error_cost(100, 1);
    b.add_dynamic_prec(5);
    b.add_error_cost(100, 1);

    // When
    let result = compare_versions(&a, &b);

    // Then
    assert_eq!(result, CompareResult::Tie);
}

// =============================================================================
// CompareResult Tests
// =============================================================================

#[test]
fn given_compare_result_when_debug_formatting_then_shows_variant_name() {
    // Given
    let results = [
        CompareResult::TakeLeft,
        CompareResult::TakeRight,
        CompareResult::PreferLeft,
        CompareResult::PreferRight,
        CompareResult::Tie,
    ];

    // When / Then
    for result in results {
        let debug = format!("{:?}", result);
        assert!(!debug.is_empty());
    }
}

#[test]
fn given_compare_results_when_checking_equality_then_works() {
    // Given
    let a = CompareResult::TakeLeft;
    let b = CompareResult::TakeLeft;
    let c = CompareResult::TakeRight;

    // When / Then
    assert_eq!(a, b);
    assert_ne!(a, c);
}
