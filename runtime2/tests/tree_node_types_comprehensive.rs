//! Comprehensive tests for runtime2 Tree and Node types.

use adze_runtime::node::Point;
use adze_runtime::tree::Tree;

// ── Tree type properties ──

#[test]
fn tree_is_debug() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<Tree>();
}

#[test]
fn tree_is_clone() {
    fn assert_clone<T: Clone>() {}
    assert_clone::<Tree>();
}

// ── Point construction and properties ──

#[test]
fn point_zero() {
    let p = Point { row: 0, column: 0 };
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

#[test]
fn point_large_values() {
    let p = Point {
        row: 100000,
        column: 500,
    };
    assert_eq!(p.row, 100000);
    assert_eq!(p.column, 500);
}

#[test]
fn point_max() {
    let p = Point {
        row: usize::MAX,
        column: usize::MAX,
    };
    assert_eq!(p.row, usize::MAX);
}

#[test]
fn point_equality() {
    assert_eq!(Point { row: 1, column: 2 }, Point { row: 1, column: 2 });
    assert_ne!(Point { row: 1, column: 2 }, Point { row: 1, column: 3 });
    assert_ne!(Point { row: 1, column: 2 }, Point { row: 2, column: 2 });
}

#[test]
fn point_ordering() {
    let a = Point { row: 0, column: 0 };
    let b = Point { row: 0, column: 1 };
    let c = Point { row: 1, column: 0 };
    assert!(a < b);
    assert!(b < c);
}

#[test]
fn point_ordering_row_priority() {
    // Row takes priority over column in ordering
    let a = Point {
        row: 0,
        column: 100,
    };
    let b = Point { row: 1, column: 0 };
    assert!(a < b);
}

#[test]
fn point_sort() {
    let mut pts = [
        Point { row: 3, column: 0 },
        Point { row: 0, column: 5 },
        Point { row: 1, column: 2 },
        Point { row: 0, column: 0 },
        Point { row: 1, column: 0 },
    ];
    pts.sort();
    assert_eq!(pts[0], Point { row: 0, column: 0 });
    assert_eq!(pts[1], Point { row: 0, column: 5 });
    assert_eq!(pts[2], Point { row: 1, column: 0 });
    assert_eq!(pts[3], Point { row: 1, column: 2 });
    assert_eq!(pts[4], Point { row: 3, column: 0 });
}

#[test]
fn point_copy() {
    let a = Point { row: 5, column: 10 };
    let b = a; // Copy
    let c = a; // Still valid
    assert_eq!(b, c);
}

#[test]
fn point_clone() {
    let a = Point { row: 7, column: 3 };
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn point_debug_output() {
    let p = Point {
        row: 42,
        column: 17,
    };
    let s = format!("{:?}", p);
    assert!(s.contains("42"));
    assert!(s.contains("17"));
}

// ── Point edge cases ──

#[test]
fn point_same_row_different_column() {
    let a = Point { row: 5, column: 0 };
    let b = Point { row: 5, column: 10 };
    assert!(a < b);
    assert!(b > a);
}

#[test]
fn point_eq_reflexive() {
    let p = Point { row: 3, column: 3 };
    assert_eq!(p, p);
}

#[test]
fn point_eq_symmetric() {
    let a = Point { row: 1, column: 2 };
    let b = Point { row: 1, column: 2 };
    assert_eq!(a, b);
    assert_eq!(b, a);
}

#[test]
fn point_ord_transitive() {
    let a = Point { row: 0, column: 0 };
    let b = Point { row: 1, column: 0 };
    let c = Point { row: 2, column: 0 };
    assert!(a < b && b < c && a < c);
}

#[test]
fn point_min_max() {
    use std::cmp::{max, min};
    let a = Point { row: 1, column: 5 };
    let b = Point { row: 2, column: 0 };
    assert_eq!(min(a, b), a);
    assert_eq!(max(a, b), b);
}

// ── Point in collections ──

#[test]
fn point_in_btreeset() {
    use std::collections::BTreeSet;
    let mut set = BTreeSet::new();
    set.insert(Point { row: 1, column: 0 });
    set.insert(Point { row: 0, column: 5 });
    set.insert(Point { row: 1, column: 0 }); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn point_in_btreemap() {
    use std::collections::BTreeMap;
    let mut map = BTreeMap::new();
    map.insert(Point { row: 0, column: 0 }, "start");
    map.insert(Point { row: 10, column: 5 }, "end");
    assert_eq!(map[&Point { row: 0, column: 0 }], "start");
}

#[test]
fn point_vec_dedup() {
    let mut pts = vec![
        Point { row: 1, column: 0 },
        Point { row: 1, column: 0 },
        Point { row: 2, column: 0 },
        Point { row: 2, column: 0 },
    ];
    pts.dedup();
    assert_eq!(pts.len(), 2);
}

// ── Error types ──

use adze_runtime::error::{ParseError, ParseErrorKind};

#[test]
fn error_no_language_display() {
    let e = ParseError {
        kind: ParseErrorKind::NoLanguage,
        location: None,
    };
    let s = format!("{}", e);
    assert!(!s.is_empty());
}

#[test]
fn error_timeout_display() {
    let e = ParseError {
        kind: ParseErrorKind::Timeout,
        location: None,
    };
    let s = format!("{}", e);
    assert!(!s.is_empty());
}

#[test]
fn error_debug() {
    let e = ParseError {
        kind: ParseErrorKind::NoLanguage,
        location: None,
    };
    let s = format!("{:?}", e);
    assert!(!s.is_empty());
}

#[test]
fn error_is_std_error() {
    fn check<T: std::error::Error>() {}
    check::<ParseError>();
}

// ── ErrorLocation ──

use adze_runtime::error::ErrorLocation;

#[test]
fn error_location_debug() {
    fn check<T: std::fmt::Debug>() {}
    check::<ErrorLocation>();
}

// ── Parser interaction ──

use adze_runtime::parser::Parser;

#[test]
fn parser_no_language_parse_error() {
    let mut p = Parser::new();
    let result = p.parse("hello", None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parser_no_language_empty_input() {
    let mut p = Parser::new();
    let result = p.parse("", None);
    assert!(result.is_err());
}

#[test]
fn parser_no_language_bytes() {
    let mut p = Parser::new();
    let result = p.parse(b"test" as &[u8], None);
    assert!(result.is_err());
}

#[test]
fn parser_reset_then_parse() {
    let mut p = Parser::new();
    p.reset();
    let result = p.parse("x", None);
    assert!(result.is_err());
}

#[test]
fn parser_multiple_parses_all_fail_without_language() {
    let mut p = Parser::new();
    for _ in 0..5 {
        assert!(p.parse("input", None).is_err());
    }
}

// ── Language type ──

use adze_runtime::language::Language;
use adze_runtime::test_helpers::stub_language;

#[test]
fn language_is_debug() {
    fn check<T: std::fmt::Debug>() {}
    check::<Language>();
}

#[test]
fn stub_language_debug() {
    let lang = stub_language();
    let s = format!("{:?}", lang);
    assert!(!s.is_empty());
}

#[test]
fn stub_language_deterministic() {
    let a = format!("{:?}", stub_language());
    let b = format!("{:?}", stub_language());
    assert_eq!(a, b);
}

// ── External scanner module ──

#[test]
fn external_scanner_module_accessible() {
    // Just verify the module exists and is public
    use adze_runtime::external_scanner::ScanResult;
    let r = ScanResult {
        token_type: 0,
        bytes_consumed: 0,
    };
    assert_eq!(r.bytes_consumed, 0);
}

// ── Test helpers ──

#[test]
fn test_helpers_module_accessible() {
    use adze_runtime::test_helpers;
    let _ = test_helpers::stub_language();
}

// ── Tree type bounds check ──

#[test]
fn tree_send_check() {
    // Tree may or may not be Send depending on internal pointers
    // Just verify compilation
    let _ = std::any::type_name::<Tree>();
}

// ── Comprehensive Point stress ──

#[test]
fn point_thousand_sort() {
    let mut pts: Vec<Point> = (0..1000)
        .map(|i| Point {
            row: i / 10,
            column: i % 10,
        })
        .collect();
    pts.reverse();
    pts.sort();
    for i in 1..pts.len() {
        assert!(pts[i - 1] <= pts[i]);
    }
}

#[test]
fn point_boundary_ordering() {
    let pts = [
        Point {
            row: 0,
            column: usize::MAX,
        },
        Point { row: 1, column: 0 },
    ];
    assert!(pts[0] < pts[1]);
}
