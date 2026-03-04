// Comprehensive integration tests for runtime2 public API:
// Parser, Language, Point, Error — non-parsing paths only.
// (Parsing with stub_language panics because the empty parse table
// triggers a GLR driver assertion, so we test only the setup/config paths.)

use adze_runtime::error::{ErrorLocation, ParseError};
use adze_runtime::node::Point;
use adze_runtime::parser::Parser;
use adze_runtime::test_helpers::stub_language;
use std::time::Duration;

// ===== Parser construction =====

#[test]
fn parser_new_no_language() {
    let p = Parser::new();
    assert!(p.language().is_none());
}

#[test]
fn parser_default_no_language() {
    let p = Parser::default();
    assert!(p.language().is_none());
}

#[test]
fn parser_new_no_timeout() {
    let p = Parser::new();
    assert!(p.timeout().is_none());
}

// ===== Language setup =====

#[test]
fn parser_set_language() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    assert!(p.language().is_some());
}

#[test]
fn parser_set_language_twice() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    p.set_language(stub_language()).unwrap();
    assert!(p.language().is_some());
}

// ===== Timeout =====

#[test]
fn parser_set_timeout() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(5));
    assert_eq!(p.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn parser_set_timeout_zero() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(0));
    assert_eq!(p.timeout(), Some(Duration::from_secs(0)));
}

#[test]
fn parser_set_timeout_large() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(3600));
    assert_eq!(p.timeout(), Some(Duration::from_secs(3600)));
}

#[test]
fn parser_set_timeout_millis() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_millis(500));
    assert_eq!(p.timeout(), Some(Duration::from_millis(500)));
}

#[test]
fn parser_set_timeout_nanos() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_nanos(123_456));
    assert_eq!(p.timeout(), Some(Duration::from_nanos(123_456)));
}

#[test]
fn parser_timeout_before_language() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(30));
    p.set_language(stub_language()).unwrap();
    assert!(p.language().is_some());
    assert_eq!(p.timeout(), Some(Duration::from_secs(30)));
}

#[test]
fn parser_timeout_after_language() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    p.set_timeout(Duration::from_millis(500));
    assert_eq!(p.timeout(), Some(Duration::from_millis(500)));
}

// ===== Parse error paths =====

#[test]
fn parse_without_language_errors() {
    let mut p = Parser::new();
    assert!(p.parse("hello", None).is_err());
}

#[test]
fn parse_utf8_without_language_errors() {
    let mut p = Parser::new();
    assert!(p.parse_utf8("hello", None).is_err());
}

#[test]
fn parse_empty_without_language_errors() {
    let mut p = Parser::new();
    assert!(p.parse("", None).is_err());
}

// ===== Reset =====

#[test]
fn parser_reset() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    p.set_timeout(Duration::from_secs(5));
    p.reset();
}

#[test]
fn parser_reset_then_set_language() {
    let mut p = Parser::new();
    p.set_language(stub_language()).unwrap();
    p.reset();
    p.set_language(stub_language()).unwrap();
    assert!(p.language().is_some());
}

#[test]
fn parser_triple_reset() {
    let mut p = Parser::new();
    p.reset();
    p.reset();
    p.reset();
}

#[test]
fn parser_reset_after_failed_parse() {
    let mut p = Parser::new();
    let _ = p.parse("x", None); // fails (no language)
    p.reset();
    p.set_language(stub_language()).unwrap();
    assert!(p.language().is_some());
}

// ===== Point type =====

#[test]
fn point_zero() {
    let p = Point { row: 0, column: 0 };
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

#[test]
fn point_nonzero() {
    let p = Point {
        row: 10,
        column: 25,
    };
    assert_eq!(p.row, 10);
    assert_eq!(p.column, 25);
}

#[test]
fn point_clone() {
    let a = Point { row: 5, column: 3 };
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn point_copy() {
    let a = Point { row: 1, column: 2 };
    let b = a;
    assert_eq!(a.row, b.row);
    assert_eq!(a.column, b.column);
}

#[test]
fn point_eq() {
    assert_eq!(Point { row: 3, column: 7 }, Point { row: 3, column: 7 });
}

#[test]
fn point_ne_row() {
    assert_ne!(Point { row: 1, column: 5 }, Point { row: 2, column: 5 });
}

#[test]
fn point_ne_column() {
    assert_ne!(Point { row: 1, column: 5 }, Point { row: 1, column: 6 });
}

#[test]
fn point_ord_different_row() {
    assert!(Point { row: 1, column: 99 } < Point { row: 2, column: 0 });
}

#[test]
fn point_ord_same_row() {
    assert!(Point { row: 1, column: 3 } < Point { row: 1, column: 7 });
}

#[test]
fn point_debug() {
    let s = format!(
        "{:?}",
        Point {
            row: 10,
            column: 20
        }
    );
    assert!(!s.is_empty());
}

#[test]
fn point_large_values() {
    let p = Point {
        row: usize::MAX,
        column: usize::MAX,
    };
    assert_eq!(p.row, usize::MAX);
}

#[test]
fn point_sort_vec() {
    let mut points = vec![
        Point { row: 3, column: 5 },
        Point { row: 1, column: 10 },
        Point { row: 3, column: 2 },
        Point { row: 2, column: 0 },
    ];
    points.sort();
    assert_eq!(points[0], Point { row: 1, column: 10 });
    assert_eq!(points[1], Point { row: 2, column: 0 });
    assert_eq!(points[2], Point { row: 3, column: 2 });
    assert_eq!(points[3], Point { row: 3, column: 5 });
}

// ===== ParseError =====

#[test]
fn parse_error_no_language_display() {
    let e = ParseError::no_language();
    let s = format!("{}", e);
    assert!(!s.is_empty());
}

#[test]
fn parse_error_timeout_display() {
    let e = ParseError::timeout();
    let s = format!("{}", e);
    assert!(!s.is_empty());
}

#[test]
fn parse_error_with_msg() {
    let e = ParseError::with_msg("bad parse");
    let s = format!("{}", e);
    assert!(s.contains("bad parse"));
}

#[test]
fn parse_error_syntax_error() {
    let loc = ErrorLocation {
        line: 5,
        column: 12,
        byte_offset: 42,
    };
    let e = ParseError::syntax_error("unexpected }", loc);
    let s = format!("{}", e);
    assert!(!s.is_empty());
}

#[test]
fn parse_error_with_location() {
    let loc = ErrorLocation {
        line: 1,
        column: 1,
        byte_offset: 0,
    };
    let e = ParseError::with_msg("error").with_location(loc);
    let _ = format!("{}", e);
}

#[test]
fn parse_error_debug() {
    let e = ParseError::timeout();
    let s = format!("{:?}", e);
    assert!(!s.is_empty());
}

// ===== ErrorLocation =====

#[test]
fn error_location_display() {
    let loc = ErrorLocation {
        line: 42,
        column: 7,
        byte_offset: 100,
    };
    let s = format!("{}", loc);
    assert!(s.contains("42") || s.contains("7"));
}

#[test]
fn error_location_fields() {
    let loc = ErrorLocation {
        line: 1,
        column: 2,
        byte_offset: 3,
    };
    assert_eq!(loc.line, 1);
    assert_eq!(loc.column, 2);
    assert_eq!(loc.byte_offset, 3);
}

#[test]
fn error_location_zero() {
    let loc = ErrorLocation {
        line: 0,
        column: 0,
        byte_offset: 0,
    };
    assert_eq!(loc.line, 0);
}

#[test]
fn error_location_large() {
    let loc = ErrorLocation {
        line: 1_000_000,
        column: 500,
        byte_offset: 50_000_000,
    };
    assert_eq!(loc.byte_offset, 50_000_000);
}
