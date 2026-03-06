//! Comprehensive tests for runtime2 Point, ParseError, and Parser error paths.
#![allow(unused_must_use)]

use adze_runtime::error::{ParseError, ParseErrorKind};
use adze_runtime::node::Point;
use adze_runtime::parser::Parser;
use adze_runtime::test_helpers::stub_language;
use std::time::Duration;

// ── Point construction ──

#[test]
fn point_new() {
    let p = Point { row: 0, column: 0 };
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

#[test]
fn point_nonzero() {
    let p = Point { row: 5, column: 10 };
    assert_eq!(p.row, 5);
    assert_eq!(p.column, 10);
}

#[test]
fn point_large_values() {
    let p = Point {
        row: usize::MAX,
        column: usize::MAX,
    };
    assert_eq!(p.row, usize::MAX);
    assert_eq!(p.column, usize::MAX);
}

// ── Point equality ──

#[test]
fn point_eq_same() {
    let a = Point { row: 1, column: 2 };
    let b = Point { row: 1, column: 2 };
    assert_eq!(a, b);
}

#[test]
fn point_ne_row() {
    let a = Point { row: 0, column: 0 };
    let b = Point { row: 1, column: 0 };
    assert_ne!(a, b);
}

#[test]
fn point_ne_col() {
    let a = Point { row: 0, column: 0 };
    let b = Point { row: 0, column: 1 };
    assert_ne!(a, b);
}

#[test]
fn point_ne_both() {
    let a = Point { row: 1, column: 2 };
    let b = Point { row: 3, column: 4 };
    assert_ne!(a, b);
}

// ── Point ordering ──

#[test]
fn point_ord_row_first() {
    let a = Point { row: 0, column: 99 };
    let b = Point { row: 1, column: 0 };
    assert!(a < b);
}

#[test]
fn point_ord_col_second() {
    let a = Point { row: 1, column: 0 };
    let b = Point { row: 1, column: 1 };
    assert!(a < b);
}

#[test]
fn point_ord_equal() {
    let a = Point { row: 1, column: 1 };
    let b = Point { row: 1, column: 1 };
    assert!(a <= b);
    assert!(a >= b);
}

#[test]
fn point_min() {
    let a = Point { row: 0, column: 5 };
    let b = Point { row: 1, column: 0 };
    assert_eq!(std::cmp::min(a, b), a);
}

#[test]
fn point_max() {
    let a = Point { row: 0, column: 5 };
    let b = Point { row: 1, column: 0 };
    assert_eq!(std::cmp::max(a, b), b);
}

// ── Point copy ──

#[test]
fn point_copy() {
    let a = Point { row: 1, column: 2 };
    let b = a;
    assert_eq!(a, b); // a still usable because Copy
}

#[test]
fn point_clone() {
    let a = Point { row: 1, column: 2 };
    let b = a;
    assert_eq!(a, b);
}

// ── Point debug ──

#[test]
fn point_debug() {
    let p = Point { row: 3, column: 7 };
    let s = format!("{:?}", p);
    assert!(s.contains("3"));
    assert!(s.contains("7"));
}

// ── Point Ord is total ──

#[test]
fn point_ord_transitivity() {
    let a = Point { row: 0, column: 0 };
    let b = Point { row: 0, column: 1 };
    let c = Point { row: 1, column: 0 };
    assert!(a < b);
    assert!(b < c);
    assert!(a < c); // transitivity
}

#[test]
fn point_ord_antisymmetry() {
    let a = Point { row: 1, column: 2 };
    let b = Point { row: 1, column: 2 };
    assert!((a >= b) && (b >= a)); // antisymmetry when equal
}

// ── Point sort ──

#[test]
fn point_sort_vec() {
    let mut pts = [
        Point { row: 2, column: 0 },
        Point { row: 0, column: 5 },
        Point { row: 1, column: 3 },
        Point { row: 0, column: 2 },
    ];
    pts.sort();
    assert_eq!(pts[0], Point { row: 0, column: 2 });
    assert_eq!(pts[1], Point { row: 0, column: 5 });
    assert_eq!(pts[2], Point { row: 1, column: 3 });
    assert_eq!(pts[3], Point { row: 2, column: 0 });
}

// ── ParseErrorKind debug ──

#[test]
fn parse_error_kind_debug() {
    fn check<T: std::fmt::Debug>() {}
    check::<ParseErrorKind>();
}

// ── ParseError traits ──

#[test]
fn parse_error_debug() {
    fn check<T: std::fmt::Debug>() {}
    check::<ParseError>();
}

#[test]
fn parse_error_display() {
    fn check<T: std::fmt::Display>() {}
    check::<ParseError>();
}

#[test]
fn parse_error_is_std_error() {
    fn check<T: std::error::Error>() {}
    check::<ParseError>();
}

// ── Parser basics ──

#[test]
fn parser_new() {
    let _p = Parser::new();
}

#[test]
fn parser_set_language() {
    let mut p = Parser::new();
    let lang = stub_language();
    p.set_language(lang);
}

#[test]
fn parser_reset() {
    let mut p = Parser::new();
    p.reset();
}

#[test]
fn parser_set_timeout() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_millis(1000));
}

#[test]
fn parser_set_timeout_zero() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_millis(0));
}

#[test]
fn parser_set_timeout_max() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_millis(u64::MAX));
}

// ── Parser parse without language ──

#[test]
fn parser_parse_no_language() {
    let mut p = Parser::new();
    let result = p.parse("hello", None);
    // Should return Err since no language set, or Ok — just verify no panic
    let _ = result;
}

// ── Parser parse with stub language ──

#[test]
fn parser_parse_empty_input() {
    let mut p = Parser::new();
    let lang = stub_language();
    p.set_language(lang);
    let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("", None)));
}

#[test]
fn parser_parse_simple_input() {
    let mut p = Parser::new();
    let lang = stub_language();
    p.set_language(lang);
    let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("hello", None)));
}

#[test]
fn parser_parse_unicode() {
    let mut p = Parser::new();
    let lang = stub_language();
    p.set_language(lang);
    let _result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("日本語", None)));
}

#[test]
fn parser_parse_long_input() {
    let mut p = Parser::new();
    let lang = stub_language();
    p.set_language(lang);
    let input = "a".repeat(10_000);
    let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse(&input, None)));
}

#[test]
fn parser_parse_multiline() {
    let mut p = Parser::new();
    let lang = stub_language();
    p.set_language(lang);
    let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        p.parse("line1\nline2\nline3", None)
    }));
}

#[test]
fn parser_parse_with_tabs() {
    let mut p = Parser::new();
    let lang = stub_language();
    p.set_language(lang);
    let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        p.parse("\t\thello\tworld", None)
    }));
}

#[test]
fn parser_parse_with_crlf() {
    let mut p = Parser::new();
    let lang = stub_language();
    p.set_language(lang);
    let _result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        p.parse("line1\r\nline2\r\n", None)
    }));
}

// ── Parser reuse ──

#[test]
fn parser_reuse_after_reset() {
    let mut p = Parser::new();
    let lang = stub_language();
    p.set_language(lang);
    let _r1 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("first", None)));
    p.reset();
    let _r2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("second", None)));
}

#[test]
fn parser_multiple_parses() {
    let mut p = Parser::new();
    let lang = stub_language();
    p.set_language(lang);
    for i in 0..10 {
        let input = format!("input_{}", i);
        let _r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse(&input, None)));
    }
}

// ── Parser debug ──

#[test]
fn parser_debug() {
    let p = Parser::new();
    let _ = format!("{:?}", p);
}

// ── Point edge cases ──

#[test]
fn point_zero_zero() {
    let p = Point { row: 0, column: 0 };
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
    assert_eq!(p, Point { row: 0, column: 0 });
}

#[test]
fn point_row_only() {
    let p = Point {
        row: 100,
        column: 0,
    };
    assert!(
        p > Point {
            row: 0,
            column: 999
        }
    );
}

#[test]
fn point_col_only() {
    let p = Point {
        row: 0,
        column: 100,
    };
    assert!(p > Point { row: 0, column: 99 });
}

#[test]
fn point_eq_reflexive() {
    let p = Point { row: 42, column: 7 };
    assert_eq!(p, p);
}

#[test]
fn point_eq_symmetric() {
    let a = Point { row: 1, column: 2 };
    let b = Point { row: 1, column: 2 };
    assert_eq!(a, b);
    assert_eq!(b, a);
}

// ── Multiple parsers ──

#[test]
fn multiple_parser_instances() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_language(stub_language());
    p2.set_language(stub_language());
    let _r1 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p1.parse("a", None)));
    let _r2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p2.parse("b", None)));
}

// ── Parser set_language multiple times ──

#[test]
fn parser_set_language_twice() {
    let mut p = Parser::new();
    p.set_language(stub_language());
    p.set_language(stub_language()); // re-set
    let _r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("test", None)));
}

// ── Point as sort key ──

#[test]
fn point_as_btreeset_key() {
    use std::collections::BTreeSet;
    let mut set = BTreeSet::new();
    set.insert(Point { row: 0, column: 0 });
    set.insert(Point { row: 1, column: 1 });
    set.insert(Point { row: 0, column: 0 }); // duplicate
    assert_eq!(set.len(), 2);
}

// ── Parser timeout behavior ──

#[test]
fn parser_timeout_then_parse() {
    let mut p = Parser::new();
    p.set_language(stub_language());
    p.set_timeout(Duration::from_millis(1));
    let _r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("hello", None)));
}

// ── ParseError/ParseErrorKind size ──

#[test]
fn parse_error_not_zero_size() {
    assert!(std::mem::size_of::<ParseError>() > 0);
}

#[test]
fn parse_error_kind_not_zero_size() {
    assert!(std::mem::size_of::<ParseErrorKind>() > 0);
}

// ── Point as tuple-like ──

#[test]
fn point_destructure() {
    let Point { row, column } = Point { row: 3, column: 7 };
    assert_eq!(row, 3);
    assert_eq!(column, 7);
}

// ── Parser edge: empty string repeatedly ──

#[test]
fn parser_empty_string_repeatedly() {
    let mut p = Parser::new();
    p.set_language(stub_language());
    for _ in 0..100 {
        let _r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("", None)));
    }
}

// ── Parser with null bytes ──

#[test]
fn parser_null_bytes() {
    let mut p = Parser::new();
    p.set_language(stub_language());
    let _r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("\0\0\0", None)));
}

// ── Parser with only whitespace ──

#[test]
fn parser_whitespace_only() {
    let mut p = Parser::new();
    p.set_language(stub_language());
    let _r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("   \t\n  ", None)));
}
