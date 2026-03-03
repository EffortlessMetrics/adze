#![allow(clippy::needless_range_loop)]

//! Property-based tests for `Point` (row, column) in the adze runtime.
//!
//! Covers creation, equality, cloning, copying, debug formatting,
//! default value, ordering (manual), large values, and usage in
//! `Spanned` context.

use adze::pure_parser::Point;
use adze::Spanned;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_point() -> impl Strategy<Value = Point> {
    (any::<u32>(), any::<u32>()).prop_map(|(r, c)| Point { row: r, column: c })
}

fn arb_small_point() -> impl Strategy<Value = Point> {
    (0u32..1000, 0u32..1000).prop_map(|(r, c)| Point { row: r, column: c })
}

// ---------------------------------------------------------------------------
// 1. Creation
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_creation_preserves_fields(row in any::<u32>(), col in any::<u32>()) {
        let p = Point { row, column: col };
        prop_assert_eq!(p.row, row);
        prop_assert_eq!(p.column, col);
    }

    #[test]
    fn test_creation_zero(row in Just(0u32), col in Just(0u32)) {
        let p = Point { row, column: col };
        prop_assert_eq!(p.row, 0);
        prop_assert_eq!(p.column, 0);
    }
}

// ---------------------------------------------------------------------------
// 2. Equality / PartialEq / Eq
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_eq_reflexive(p in arb_point()) {
        prop_assert_eq!(p, p);
    }

    #[test]
    fn test_eq_symmetric(p1 in arb_point(), p2 in arb_point()) {
        prop_assert_eq!(p1 == p2, p2 == p1);
    }

    #[test]
    fn test_eq_same_values(row in any::<u32>(), col in any::<u32>()) {
        let a = Point { row, column: col };
        let b = Point { row, column: col };
        prop_assert_eq!(a, b);
    }

    #[test]
    fn test_ne_different_row(row in 0u32..u32::MAX, col in any::<u32>()) {
        let a = Point { row, column: col };
        let b = Point { row: row + 1, column: col };
        prop_assert_ne!(a, b);
    }

    #[test]
    fn test_ne_different_column(row in any::<u32>(), col in 0u32..u32::MAX) {
        let a = Point { row, column: col };
        let b = Point { row, column: col + 1 };
        prop_assert_ne!(a, b);
    }
}

// ---------------------------------------------------------------------------
// 3. Ordering (manual — Point does not derive Ord)
// ---------------------------------------------------------------------------

/// Lexicographic ordering helper: compare by (row, column).
fn point_cmp(a: &Point, b: &Point) -> std::cmp::Ordering {
    (a.row, a.column).cmp(&(b.row, b.column))
}

proptest! {
    #[test]
    fn test_manual_ord_reflexive(p in arb_point()) {
        prop_assert_eq!(point_cmp(&p, &p), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_manual_ord_antisymmetric(p1 in arb_point(), p2 in arb_point()) {
        let cmp1 = point_cmp(&p1, &p2);
        let cmp2 = point_cmp(&p2, &p1);
        prop_assert_eq!(cmp1, cmp2.reverse());
    }

    #[test]
    fn test_manual_ord_transitive(
        p1 in arb_small_point(),
        p2 in arb_small_point(),
        p3 in arb_small_point(),
    ) {
        use std::cmp::Ordering::*;
        let c12 = point_cmp(&p1, &p2);
        let c23 = point_cmp(&p2, &p3);
        let c13 = point_cmp(&p1, &p3);
        if c12 == Less && c23 == Less {
            prop_assert_eq!(c13, Less);
        }
        if c12 == Greater && c23 == Greater {
            prop_assert_eq!(c13, Greater);
        }
    }

    #[test]
    fn test_manual_ord_row_dominates(
        row_a in 0u32..u32::MAX,
        col_a in any::<u32>(),
        col_b in any::<u32>(),
    ) {
        let a = Point { row: row_a, column: col_a };
        let b = Point { row: row_a + 1, column: col_b };
        prop_assert_eq!(point_cmp(&a, &b), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_manual_ord_same_row_column_decides(
        row in any::<u32>(),
        col_a in 0u32..u32::MAX,
    ) {
        let a = Point { row, column: col_a };
        let b = Point { row, column: col_a + 1 };
        prop_assert_eq!(point_cmp(&a, &b), std::cmp::Ordering::Less);
    }
}

// ---------------------------------------------------------------------------
// 4. Clone / Copy
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_clone_equals_original(p in arb_point()) {
        let cloned = p.clone();
        prop_assert_eq!(p, cloned);
    }

    #[test]
    fn test_copy_semantics(p in arb_point()) {
        let copied = p; // Copy
        prop_assert_eq!(p, copied);
        // Both remain usable after copy.
        prop_assert_eq!(p.row, copied.row);
        prop_assert_eq!(p.column, copied.column);
    }

    #[test]
    fn test_clone_independence(row in any::<u32>(), col in any::<u32>()) {
        let p = Point { row, column: col };
        let mut cloned = p;
        cloned.row = cloned.row.wrapping_add(1);
        // Original is unaffected (Copy type — independent).
        prop_assert_eq!(p.row, row);
    }
}

// ---------------------------------------------------------------------------
// 5. Debug formatting
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_debug_contains_row_and_column(p in arb_point()) {
        let dbg = format!("{:?}", p);
        prop_assert!(dbg.contains("Point"));
        prop_assert!(dbg.contains(&p.row.to_string()));
        prop_assert!(dbg.contains(&p.column.to_string()));
    }

    #[test]
    fn test_debug_not_empty(p in arb_point()) {
        let dbg = format!("{:?}", p);
        prop_assert!(!dbg.is_empty());
    }

    #[test]
    fn test_debug_deterministic(p in arb_point()) {
        let d1 = format!("{:?}", p);
        let d2 = format!("{:?}", p);
        prop_assert_eq!(d1, d2);
    }
}

// ---------------------------------------------------------------------------
// 6. Default (should be 0, 0)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_default_is_zero_zero(_seed in any::<u8>()) {
        let p = Point::default();
        prop_assert_eq!(p.row, 0);
        prop_assert_eq!(p.column, 0);
    }

    #[test]
    fn test_default_equals_explicit_zero(_seed in any::<u8>()) {
        let def = Point::default();
        let explicit = Point { row: 0, column: 0 };
        prop_assert_eq!(def, explicit);
    }
}

// ---------------------------------------------------------------------------
// 7. Large values
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_max_row(col in any::<u32>()) {
        let p = Point { row: u32::MAX, column: col };
        prop_assert_eq!(p.row, u32::MAX);
        prop_assert_eq!(p.column, col);
    }

    #[test]
    fn test_max_column(row in any::<u32>()) {
        let p = Point { row, column: u32::MAX };
        prop_assert_eq!(p.row, row);
        prop_assert_eq!(p.column, u32::MAX);
    }

    #[test]
    fn test_max_both(_seed in any::<u8>()) {
        let p = Point { row: u32::MAX, column: u32::MAX };
        prop_assert_eq!(p.row, u32::MAX);
        prop_assert_eq!(p.column, u32::MAX);
    }

    #[test]
    fn test_large_round_trip(row in any::<u32>(), col in any::<u32>()) {
        let p = Point { row, column: col };
        let cloned = p;
        prop_assert_eq!(p.row, cloned.row);
        prop_assert_eq!(p.column, cloned.column);
    }
}

// ---------------------------------------------------------------------------
// 8. Point in Spanned context
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_spanned_with_point_value(p in arb_point(), start in 0usize..1000, len in 0usize..1000) {
        let end = start.saturating_add(len);
        let spanned = Spanned { value: p, span: (start, end) };
        prop_assert_eq!(*spanned, p);
        prop_assert_eq!(spanned.span.0, start);
        prop_assert_eq!(spanned.span.1, end);
    }

    #[test]
    fn test_spanned_clone_preserves_point(p in arb_point(), start in 0usize..1000, len in 0usize..1000) {
        let end = start.saturating_add(len);
        let spanned = Spanned { value: p, span: (start, end) };
        let cloned = spanned.clone();
        prop_assert_eq!(*cloned, p);
        prop_assert_eq!(cloned.span, spanned.span);
    }

    #[test]
    fn test_spanned_deref_gives_point(p in arb_point()) {
        let spanned = Spanned { value: p, span: (0, 0) };
        let inner: &Point = &spanned;
        prop_assert_eq!(inner.row, p.row);
        prop_assert_eq!(inner.column, p.column);
    }

    #[test]
    fn test_spanned_debug_contains_point(p in arb_point()) {
        let spanned = Spanned { value: p, span: (0, 1) };
        let dbg = format!("{:?}", spanned);
        prop_assert!(dbg.contains("Point"));
    }
}

// ---------------------------------------------------------------------------
// 9. Structural / repr(C) size
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_size_is_8_bytes(_seed in any::<u8>()) {
        // Two u32 fields in repr(C) → 8 bytes total.
        prop_assert_eq!(std::mem::size_of::<Point>(), 8);
    }

    #[test]
    fn test_alignment_is_4(_seed in any::<u8>()) {
        prop_assert_eq!(std::mem::align_of::<Point>(), 4);
    }
}
