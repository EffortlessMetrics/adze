//! Comprehensive tests for runtime2 Parser public API.
//!
//! Tests Parser construction, configuration, error handling, and edge cases.

use adze_runtime::parser::Parser;
use adze_runtime::test_helpers::stub_language;

// ── Parser Construction ──

#[test]
fn parser_new_creates_instance() {
    let _parser = Parser::new();
}

#[test]
fn parser_default_no_language() {
    let parser = Parser::new();
    let dbg = format!("{:?}", parser);
    assert!(!dbg.is_empty());
}

#[test]
fn parser_set_language_stub() {
    let mut parser = Parser::new();
    let lang = stub_language();
    let result = parser.set_language(lang);
    // stub_language may or may not succeed
    let _ = result;
}

#[test]
fn parser_reset() {
    let mut parser = Parser::new();
    parser.reset();
    // Should not panic after reset
}

#[test]
fn parser_reset_multiple_times() {
    let mut parser = Parser::new();
    for _ in 0..10 {
        parser.reset();
    }
}

#[test]
fn parser_set_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(std::time::Duration::from_secs(5));
}

#[test]
fn parser_set_timeout_zero() {
    let mut parser = Parser::new();
    parser.set_timeout(std::time::Duration::ZERO);
}

#[test]
fn parser_set_timeout_large() {
    let mut parser = Parser::new();
    parser.set_timeout(std::time::Duration::from_secs(3600));
}

// ── Parser Error Types ──

use adze_runtime::error::{ParseError, ParseErrorKind};

#[test]
fn parse_error_display() {
    let err = ParseError {
        kind: ParseErrorKind::NoLanguage,
        location: None,
    };
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn parse_error_debug() {
    let err = ParseError {
        kind: ParseErrorKind::NoLanguage,
        location: None,
    };
    let dbg = format!("{:?}", err);
    assert!(!dbg.is_empty());
}

#[test]
fn parse_error_kind_no_language() {
    let err = ParseError {
        kind: ParseErrorKind::NoLanguage,
        location: None,
    };
    assert!(matches!(&err.kind, ParseErrorKind::NoLanguage));
}

#[test]
fn parse_error_kind_timeout() {
    let err = ParseError {
        kind: ParseErrorKind::Timeout,
        location: None,
    };
    assert!(matches!(&err.kind, ParseErrorKind::Timeout));
}

// ── Node/Point Types ──

use adze_runtime::node::Point;

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
fn point_clone() {
    let p = Point { row: 1, column: 2 };
    let p2 = p;
    assert_eq!(p, p2);
}

#[test]
fn point_debug() {
    let p = Point { row: 3, column: 7 };
    let dbg = format!("{:?}", p);
    assert!(dbg.contains("3"));
    assert!(dbg.contains("7"));
}

#[test]
fn point_eq() {
    let a = Point { row: 1, column: 1 };
    let b = Point { row: 1, column: 1 };
    assert_eq!(a, b);
}

#[test]
fn point_ne() {
    let a = Point { row: 1, column: 1 };
    let b = Point { row: 2, column: 1 };
    assert_ne!(a, b);
}

#[test]
fn point_ord() {
    let a = Point { row: 0, column: 0 };
    let b = Point { row: 0, column: 1 };
    let c = Point { row: 1, column: 0 };
    assert!(a < b);
    assert!(b < c);
    assert!(a < c);
}

#[test]
fn point_copy() {
    let p = Point { row: 5, column: 5 };
    let p2 = p;
    // Original still usable (Copy)
    assert_eq!(p.row, 5);
    assert_eq!(p2.row, 5);
}

// ── Tree Types ──

use adze_runtime::tree::Tree;

#[test]
fn tree_debug() {
    // Tree construction from parse results
    // Since we can't parse without a real language, test what we can
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<Tree>();
}

// ── Language Type ──

use adze_runtime::language::Language;

#[test]
fn language_debug() {
    fn assert_debug<T: std::fmt::Debug>() {}
    assert_debug::<Language>();
}

#[test]
fn stub_language_constructs() {
    let lang = stub_language();
    let dbg = format!("{:?}", lang);
    assert!(!dbg.is_empty());
}

// ── Parse without language ──

#[test]
fn parse_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse("hello", None);
    assert!(result.is_err());
}

#[test]
fn parse_utf8_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse(b"hello", None);
    assert!(result.is_err());
}

#[test]
fn parse_empty_string_without_language() {
    let mut parser = Parser::new();
    let result = parser.parse("", None);
    assert!(result.is_err());
}

// ── Error edge cases ──

#[test]
fn parse_error_is_error_trait() {
    fn assert_error<T: std::error::Error>() {}
    assert_error::<ParseError>();
}

#[test]
fn parse_error_send() {
    fn assert_send<T: Send>() {}
    assert_send::<ParseError>();
}

#[test]
fn parse_error_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<ParseError>();
}

// ── Point arithmetic ──

#[test]
fn point_sort_vec() {
    let mut points = vec![
        Point { row: 2, column: 3 },
        Point { row: 0, column: 5 },
        Point { row: 1, column: 0 },
        Point { row: 0, column: 0 },
    ];
    points.sort();
    assert_eq!(points[0], Point { row: 0, column: 0 });
    assert_eq!(points[1], Point { row: 0, column: 5 });
    assert_eq!(points[2], Point { row: 1, column: 0 });
    assert_eq!(points[3], Point { row: 2, column: 3 });
}

#[test]
fn point_max_values() {
    let p = Point {
        row: usize::MAX,
        column: usize::MAX,
    };
    assert_eq!(p.row, usize::MAX);
    assert_eq!(p.column, usize::MAX);
}

// ── Parser after reset ──

#[test]
fn parser_parse_after_reset() {
    let mut parser = Parser::new();
    parser.reset();
    let result = parser.parse("test", None);
    assert!(result.is_err()); // No language set
}

#[test]
fn parser_set_timeout_then_reset() {
    let mut parser = Parser::new();
    parser.set_timeout(std::time::Duration::from_millis(100));
    parser.reset();
    // Timeout should be cleared or preserved; either way no panic
}

// ── Multiple parsers ──

#[test]
fn multiple_parsers_coexist() {
    let _p1 = Parser::new();
    let _p2 = Parser::new();
    let _p3 = Parser::new();
}

#[test]
fn parser_drop_clean() {
    {
        let _p = Parser::new();
    }
    // Parser dropped cleanly
}

// ── ParseErrorKind variants ──

#[test]
fn error_kind_debug() {
    let kinds = [ParseErrorKind::NoLanguage, ParseErrorKind::Timeout];
    for kind in &kinds {
        let dbg = format!("{:?}", kind);
        assert!(!dbg.is_empty());
    }
}

// ── Stub language properties ──

#[test]
fn stub_language_multiple_instances() {
    let a = stub_language();
    let b = stub_language();
    let da = format!("{:?}", a);
    let db = format!("{:?}", b);
    // Both should have same structure
    assert_eq!(da, db);
}
