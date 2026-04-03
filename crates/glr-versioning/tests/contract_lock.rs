//! Contract lock test - verifies that public API remains stable.

use adze_glr_versioning::{CompareResult, VersionInfo, compare_versions};

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify VersionInfo struct exists with expected fields
    let version = VersionInfo {
        in_error: false,
        cost: 0,
        node_count: 0,
        dynamic_prec: 0,
    };

    // Verify Debug trait is implemented
    let _debug = format!("{version:?}");

    // Verify Clone trait is implemented
    let _cloned = version.clone();

    // Verify Default trait is implemented
    let _default = VersionInfo::default();

    // Verify CompareResult enum exists with all variants
    let _take_left = CompareResult::TakeLeft;
    let _take_right = CompareResult::TakeRight;
    let _prefer_left = CompareResult::PreferLeft;
    let _prefer_right = CompareResult::PreferRight;
    let _tie = CompareResult::Tie;

    // Verify Debug trait is implemented for CompareResult
    let _debug = format!("{_take_left:?}");

    // Verify PartialEq trait is implemented for CompareResult
    assert_eq!(_take_left, CompareResult::TakeLeft);
    assert_ne!(_take_left, CompareResult::TakeRight);
}

/// Verify all public methods exist with expected signatures.
#[test]
fn test_contract_lock_methods() {
    // Verify VersionInfo::new method exists
    let _version = VersionInfo::new();

    // Verify add_dynamic_prec method exists
    let mut version = VersionInfo::new();
    version.add_dynamic_prec(5);

    // Verify enter_error method exists
    let mut error_version = VersionInfo::new();
    error_version.enter_error();
    assert!(error_version.in_error);

    // Verify add_error_cost method exists
    let mut cost_version = VersionInfo::new();
    cost_version.add_error_cost(100, 5);
    assert_eq!(cost_version.cost, 100);
    assert_eq!(cost_version.node_count, 5);
}

/// Verify compare_versions function exists with expected signature.
#[test]
fn test_contract_lock_functions() {
    // Verify compare_versions function exists
    let a = VersionInfo::new();
    let b = VersionInfo::new();
    let _result = compare_versions(&a, &b);

    // Verify error vs non-error comparison
    let clean = VersionInfo::new();
    let mut errored = VersionInfo::new();
    errored.enter_error();

    assert_eq!(compare_versions(&clean, &errored), CompareResult::TakeLeft);
    assert_eq!(compare_versions(&errored, &clean), CompareResult::TakeRight);

    // Verify tie when both are equal
    let v1 = VersionInfo::new();
    let v2 = VersionInfo::new();
    assert_eq!(compare_versions(&v1, &v2), CompareResult::Tie);
}

/// Verify dynamic precedence comparison works correctly.
#[test]
fn test_contract_lock_dynamic_precedence() {
    let mut a = VersionInfo::new();
    let mut b = VersionInfo::new();

    a.add_dynamic_prec(5);
    b.add_dynamic_prec(3);

    // Higher dynamic precedence wins
    assert_eq!(compare_versions(&a, &b), CompareResult::TakeLeft);
    assert_eq!(compare_versions(&b, &a), CompareResult::TakeRight);
}
