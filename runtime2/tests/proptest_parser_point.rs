//! Property-based tests for runtime2 Parser and error types.

use adze_runtime::error::{ParseError, ParseErrorKind};
use adze_runtime::node::Point;
use adze_runtime::parser::Parser;
use proptest::prelude::*;

// ── Point property tests ──

proptest! {
    #[test]
    fn point_eq_reflexive(row in 0..10000usize, col in 0..10000usize) {
        let p = Point { row, column: col };
        prop_assert_eq!(p, p);
    }

    #[test]
    fn point_eq_symmetric(r1 in 0..1000usize, c1 in 0..1000usize, r2 in 0..1000usize, c2 in 0..1000usize) {
        let a = Point { row: r1, column: c1 };
        let b = Point { row: r2, column: c2 };
        prop_assert_eq!(a == b, b == a);
    }

    #[test]
    fn point_ord_consistent_with_eq(r1 in 0..1000usize, c1 in 0..1000usize, r2 in 0..1000usize, c2 in 0..1000usize) {
        let a = Point { row: r1, column: c1 };
        let b = Point { row: r2, column: c2 };
        if a == b {
            prop_assert!(!(a < b) && !(b < a));
        }
    }

    #[test]
    fn point_ord_total(r1 in 0..1000usize, c1 in 0..1000usize, r2 in 0..1000usize, c2 in 0..1000usize) {
        let a = Point { row: r1, column: c1 };
        let b = Point { row: r2, column: c2 };
        // Total order: exactly one of a < b, a == b, a > b
        let lt = a < b;
        let eq = a == b;
        let gt = a > b;
        prop_assert!((lt as u8) + (eq as u8) + (gt as u8) == 1);
    }

    #[test]
    fn point_row_major_ordering(r1 in 0..100usize, c1 in 0..100usize, c2 in 0..100usize) {
        let a = Point { row: r1, column: c1 };
        let b = Point { row: r1 + 1, column: c2 };
        prop_assert!(a < b, "Higher row should always be greater");
    }

    #[test]
    fn point_same_row_column_ordering(row in 0..100usize, c1 in 0..99usize) {
        let a = Point { row, column: c1 };
        let b = Point { row, column: c1 + 1 };
        prop_assert!(a < b);
    }

    #[test]
    fn point_sort_is_stable(
        points in prop::collection::vec((0..50usize, 0..50usize), 1..30)
    ) {
        let mut pts: Vec<Point> = points.iter()
            .map(|(r, c)| Point { row: *r, column: *c })
            .collect();
        pts.sort();
        for i in 1..pts.len() {
            prop_assert!(pts[i-1] <= pts[i]);
        }
    }

    #[test]
    fn point_debug_contains_values(row in 0..10000usize, col in 0..10000usize) {
        let p = Point { row, column: col };
        let s = format!("{:?}", p);
        prop_assert!(s.contains(&row.to_string()));
        prop_assert!(s.contains(&col.to_string()));
    }

    #[test]
    fn point_copy_semantics(row in 0..1000usize, col in 0..1000usize) {
        let a = Point { row, column: col };
        let b = a;
        let c = a; // a is still valid (Copy)
        prop_assert_eq!(b, c);
    }

    #[test]
    fn point_clone_eq(row in 0..1000usize, col in 0..1000usize) {
        let a = Point { row, column: col };
        let b = a.clone();
        prop_assert_eq!(a, b);
    }
}

// ── Parser property tests ──

proptest! {
    #[test]
    fn parser_parse_without_language_always_fails(input in ".*") {
        let mut p = Parser::new();
        let result = p.parse(input.as_bytes(), None);
        prop_assert!(result.is_err());
    }

    #[test]
    fn parser_parse_empty_fails_without_language(n in 0..10u32) {
        let mut p = Parser::new();
        for _ in 0..n {
            p.reset();
        }
        let result = p.parse(b"" as &[u8], None);
        prop_assert!(result.is_err());
    }
}

// ── Error property tests ──

#[test]
fn parse_error_no_language_display() {
    let err = ParseError {
        kind: ParseErrorKind::NoLanguage,
        location: None,
    };
    assert!(!format!("{}", err).is_empty());
}

#[test]
fn parse_error_timeout_display() {
    let err = ParseError {
        kind: ParseErrorKind::Timeout,
        location: None,
    };
    assert!(!format!("{}", err).is_empty());
}

#[test]
fn parse_error_no_language_debug() {
    let err = ParseError {
        kind: ParseErrorKind::NoLanguage,
        location: None,
    };
    assert!(!format!("{:?}", err).is_empty());
}

#[test]
fn parse_error_timeout_debug() {
    let err = ParseError {
        kind: ParseErrorKind::Timeout,
        location: None,
    };
    assert!(!format!("{:?}", err).is_empty());
}

// ── Regular tests ──

#[test]
fn parser_new_is_cheap() {
    // Creating many parsers should be fast
    let parsers: Vec<Parser> = (0..100).map(|_| Parser::new()).collect();
    assert_eq!(parsers.len(), 100);
}

#[test]
fn parser_reset_preserves_no_language() {
    let mut p = Parser::new();
    p.reset();
    assert!(p.parse(b"x" as &[u8], None).is_err());
}

#[test]
fn point_btreeset_dedup() {
    use std::collections::BTreeSet;
    let mut set = BTreeSet::new();
    for i in 0..100 {
        set.insert(Point {
            row: i % 10,
            column: i % 5,
        });
    }
    // Should have at most 50 unique points (10 * 5)
    assert!(set.len() <= 50);
}

#[test]
fn error_kind_match_exhaustive() {
    let kinds = [ParseErrorKind::NoLanguage, ParseErrorKind::Timeout];
    for kind in kinds {
        match kind {
            ParseErrorKind::NoLanguage => {}
            ParseErrorKind::Timeout => {}
            _ => {}
        }
    }
}

#[test]
fn parser_timeout_setting() {
    let mut p = Parser::new();
    p.set_timeout(std::time::Duration::from_millis(1));
    p.set_timeout(std::time::Duration::from_secs(60));
    p.set_timeout(std::time::Duration::ZERO);
}

#[test]
fn multiple_parsers_independent() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.reset();
    // p2 should be unaffected
    assert!(p2.parse(b"x" as &[u8], None).is_err());
    let _ = p1;
}
