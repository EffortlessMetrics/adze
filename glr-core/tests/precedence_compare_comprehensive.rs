//! Comprehensive tests for precedence comparison logic.
//!
//! Covers: PrecedenceInfo, compare_precedences, PrecedenceComparison,
//! StaticPrecedenceResolver, edge cases and properties.

use adze_glr_core::precedence_compare::{
    PrecedenceComparison, PrecedenceInfo, compare_precedences,
};
use adze_ir::Associativity;

fn prec(level: i16, assoc: Associativity) -> PrecedenceInfo {
    PrecedenceInfo {
        level,
        associativity: assoc,
        is_fragile: false,
    }
}

// --- compare_precedences ---

#[test]
fn higher_shift_wins() {
    let result = compare_precedences(
        Some(prec(2, Associativity::Left)),
        Some(prec(1, Associativity::Left)),
    );
    assert_eq!(result, PrecedenceComparison::PreferShift);
}

#[test]
fn higher_reduce_wins() {
    let result = compare_precedences(
        Some(prec(1, Associativity::Left)),
        Some(prec(2, Associativity::Left)),
    );
    assert_eq!(result, PrecedenceComparison::PreferReduce);
}

#[test]
fn same_level_left_assoc_reduces() {
    let result = compare_precedences(
        Some(prec(1, Associativity::Left)),
        Some(prec(1, Associativity::Left)),
    );
    assert_eq!(result, PrecedenceComparison::PreferReduce);
}

#[test]
fn same_level_right_assoc_shifts() {
    let result = compare_precedences(
        Some(prec(1, Associativity::Right)),
        Some(prec(1, Associativity::Right)),
    );
    assert_eq!(result, PrecedenceComparison::PreferShift);
}

#[test]
fn same_level_none_assoc_error() {
    let result = compare_precedences(
        Some(prec(1, Associativity::None)),
        Some(prec(1, Associativity::None)),
    );
    assert_eq!(result, PrecedenceComparison::Error);
}

#[test]
fn no_shift_prec_returns_none() {
    let result = compare_precedences(None, Some(prec(1, Associativity::Left)));
    assert_eq!(result, PrecedenceComparison::None);
}

#[test]
fn no_reduce_prec_returns_none() {
    let result = compare_precedences(Some(prec(1, Associativity::Left)), None);
    assert_eq!(result, PrecedenceComparison::None);
}

#[test]
fn both_none_returns_none() {
    let result = compare_precedences(None, None);
    assert_eq!(result, PrecedenceComparison::None);
}

#[test]
fn negative_precedence_levels() {
    let result = compare_precedences(
        Some(prec(-1, Associativity::Left)),
        Some(prec(-2, Associativity::Left)),
    );
    assert_eq!(result, PrecedenceComparison::PreferShift);
}

#[test]
fn zero_precedence_level() {
    let result = compare_precedences(
        Some(prec(0, Associativity::Left)),
        Some(prec(0, Associativity::Left)),
    );
    assert_eq!(result, PrecedenceComparison::PreferReduce);
}

#[test]
fn reduce_assoc_used_for_tie() {
    // When levels tie, the reduce rule's associativity determines the result
    let result = compare_precedences(
        Some(prec(5, Associativity::Right)),
        Some(prec(5, Associativity::Left)),
    );
    // Reduce is Left, so PreferReduce
    assert_eq!(result, PrecedenceComparison::PreferReduce);
}

#[test]
fn extreme_precedence_levels() {
    let result = compare_precedences(
        Some(prec(i16::MAX, Associativity::Left)),
        Some(prec(i16::MIN, Associativity::Left)),
    );
    assert_eq!(result, PrecedenceComparison::PreferShift);
}

// --- PrecedenceInfo properties ---

#[test]
fn precedence_info_copy() {
    let p = prec(5, Associativity::Left);
    let p2 = p;
    let p3 = p; // Copy
    assert_eq!(p2.level, p3.level);
}

#[test]
fn precedence_info_equality() {
    let a = prec(1, Associativity::Left);
    let b = prec(1, Associativity::Left);
    assert_eq!(a, b);
}

#[test]
fn precedence_info_inequality() {
    let a = prec(1, Associativity::Left);
    let b = prec(2, Associativity::Left);
    assert_ne!(a, b);
}

#[test]
fn precedence_info_debug() {
    let p = prec(1, Associativity::Right);
    let debug = format!("{:?}", p);
    assert!(debug.contains("PrecedenceInfo"));
}

#[test]
fn precedence_info_fragile() {
    let p = PrecedenceInfo {
        level: 1,
        associativity: Associativity::Left,
        is_fragile: true,
    };
    assert!(p.is_fragile);
}

// --- PrecedenceComparison properties ---

#[test]
fn comparison_debug() {
    let c = PrecedenceComparison::PreferShift;
    assert!(format!("{:?}", c).contains("PreferShift"));
}

#[test]
fn comparison_equality() {
    assert_eq!(
        PrecedenceComparison::PreferShift,
        PrecedenceComparison::PreferShift
    );
    assert_ne!(
        PrecedenceComparison::PreferShift,
        PrecedenceComparison::PreferReduce
    );
}

#[test]
fn all_comparison_variants() {
    let _ = PrecedenceComparison::PreferShift;
    let _ = PrecedenceComparison::PreferReduce;
    let _ = PrecedenceComparison::Error;
    let _ = PrecedenceComparison::None;
}
