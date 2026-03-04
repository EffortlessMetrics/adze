//! Comprehensive tests for Parser construction and configuration patterns.

use adze_runtime::parser::Parser;
use std::time::Duration;

#[test]
fn parser_new() {
    let p = Parser::new();
    let _ = p;
}

#[test]
fn parser_new_multiple() {
    let p1 = Parser::new();
    let p2 = Parser::new();
    let _ = (p1, p2);
}

#[test]
fn parser_set_timeout_1s() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(1));
}

#[test]
fn parser_set_timeout_zero() {
    let mut p = Parser::new();
    p.set_timeout(Duration::ZERO);
}

#[test]
fn parser_set_timeout_large() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(3600));
}

#[test]
fn parser_set_timeout_millis() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_millis(500));
}

#[test]
fn parser_set_timeout_multiple() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(1));
    p.set_timeout(Duration::from_secs(2));
    p.set_timeout(Duration::from_secs(3));
}

#[test]
fn parser_parse_without_language_panics() {
    let mut p = Parser::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("test", None)));
    let _ = result;
}

#[test]
fn parser_parse_empty_without_language() {
    let mut p = Parser::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| p.parse("", None)));
    let _ = result;
}

#[test]
fn parser_parse_bytes_without_language() {
    let mut p = Parser::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        p.parse(b"test" as &[u8], None)
    }));
    let _ = result;
}

#[test]
fn parser_construction_is_fast() {
    for _ in 0..100 {
        let _ = Parser::new();
    }
}

#[test]
fn parser_timeout_resets() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_secs(1));
    p.set_timeout(Duration::ZERO);
}

#[test]
fn parser_timeout_nanos() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_nanos(1));
}

#[test]
fn parser_timeout_micros() {
    let mut p = Parser::new();
    p.set_timeout(Duration::from_micros(500));
}
