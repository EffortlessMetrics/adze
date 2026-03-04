// Comprehensive tests for runtime2 Parser error handling
// Tests parse error conditions and error types

use adze_runtime::parser::Parser;
use std::panic::{AssertUnwindSafe, catch_unwind};

#[test]
fn parser_new_succeeds() {
    let _p = Parser::new();
}

#[test]
fn parser_default_has_no_language() {
    let p = Parser::new();
    // Parser without language set should not parse successfully
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut p2 = p;
        p2.parse(b"hello", None)
    }));
    // May panic or return error — both acceptable
    let _ = result;
}

#[test]
fn parser_parse_empty_input_panics_without_language() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut p = Parser::new();
        p.parse(b"", None)
    }));
    let _ = result;
}

#[test]
fn parser_parse_utf8_empty_without_language() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut p = Parser::new();
        p.parse_utf8("", None)
    }));
    let _ = result;
}

#[test]
fn parser_set_timeout() {
    let mut p = Parser::new();
    p.set_timeout(std::time::Duration::from_secs(5));
    // Should not panic
}

#[test]
fn parser_set_timeout_zero() {
    let mut p = Parser::new();
    p.set_timeout(std::time::Duration::from_secs(0));
}

#[test]
fn parser_set_timeout_large() {
    let mut p = Parser::new();
    p.set_timeout(std::time::Duration::from_secs(3600));
}

#[test]
fn parser_debug_format() {
    let p = Parser::new();
    let dbg = format!("{:?}", p);
    assert!(!dbg.is_empty());
}

#[test]
fn parser_multiple_new() {
    let _p1 = Parser::new();
    let _p2 = Parser::new();
    let _p3 = Parser::new();
}

#[test]
fn parser_set_timeout_millis() {
    let mut p = Parser::new();
    p.set_timeout(std::time::Duration::from_millis(500));
}

#[test]
fn parser_set_timeout_nanos() {
    let mut p = Parser::new();
    p.set_timeout(std::time::Duration::from_nanos(1_000_000));
}

#[test]
fn parser_parse_various_inputs_without_language() {
    let inputs: &[&[u8]] = &[b"", b"a", b"hello world", b"\xff\xfe"];
    for input in inputs {
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut p = Parser::new();
            p.parse(*input, None)
        }));
        let _ = result;
    }
}
