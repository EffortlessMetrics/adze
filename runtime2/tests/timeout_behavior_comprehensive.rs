#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for parser timeout behavior.
//!
//! Covers: set_timeout, timeout retrieval, timeout override, timeout after reset,
//! zero timeout, large timeout values, timeout with parse operations,
//! and multiple timeout changes.

use adze_runtime::language::{Language, SymbolMetadata};
use adze_runtime::parser::Parser;
use std::time::Duration;

fn minimal_language() -> Language {
    let table = Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    Language::builder()
        .version(14)
        .parse_table(table)
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        }])
        .tokenizer(|_input: &[u8]| {
            Box::new(std::iter::empty()) as Box<dyn Iterator<Item = adze_runtime::Token>>
        })
        .build()
        .unwrap()
}

// --- Initial state ---

#[test]
fn new_parser_has_no_timeout() {
    let parser = Parser::new();
    assert!(parser.timeout().is_none());
}

#[test]
fn default_parser_has_no_timeout() {
    let parser = Parser::default();
    assert!(parser.timeout().is_none());
}

// --- Setting timeout ---

#[test]
fn set_timeout_returns_value() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn set_timeout_millis() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(250));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(250)));
}

#[test]
fn set_timeout_micros() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_micros(500));
    assert_eq!(parser.timeout(), Some(Duration::from_micros(500)));
}

#[test]
fn set_timeout_nanos() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_nanos(12345));
    assert_eq!(parser.timeout(), Some(Duration::from_nanos(12345)));
}

#[test]
fn set_timeout_fractional_secs() {
    let mut parser = Parser::new();
    let dur = Duration::new(2, 500_000_000); // 2.5 seconds
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

// --- Timeout override ---

#[test]
fn timeout_override_replaces_previous() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(10));
    parser.set_timeout(Duration::from_secs(3));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(3)));
}

#[test]
fn timeout_override_larger_value() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(100));
    parser.set_timeout(Duration::from_secs(60));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(60)));
}

#[test]
fn timeout_override_same_value() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    parser.set_timeout(Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

// --- Timeout after reset ---

#[test]
fn timeout_persists_after_reset() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(7));
    parser.reset();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(7)));
}

#[test]
fn timeout_set_after_reset_works() {
    let mut parser = Parser::new();
    parser.reset();
    parser.set_timeout(Duration::from_secs(4));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(4)));
}

#[test]
fn timeout_override_after_reset() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(1));
    parser.reset();
    parser.set_timeout(Duration::from_secs(2));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(2)));
}

#[test]
fn multiple_resets_preserve_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(999));
    for _ in 0..5 {
        parser.reset();
    }
    assert_eq!(parser.timeout(), Some(Duration::from_millis(999)));
}

// --- Zero timeout ---

#[test]
fn zero_timeout_is_accepted() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

#[test]
fn zero_timeout_from_secs() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(0));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(0)));
}

#[test]
fn zero_timeout_after_nonzero() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(10));
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

// --- Large timeout values ---

#[test]
fn large_timeout_secs() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs(86400); // 24 hours
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn very_large_timeout() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs(u64::MAX / 2);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn max_duration_timeout() {
    let mut parser = Parser::new();
    let dur = Duration::MAX;
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

// --- Timeout with parse operations ---

#[test]
fn timeout_set_before_language() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    let lang = minimal_language();
    parser.set_language(lang).unwrap();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn timeout_set_after_language() {
    let mut parser = Parser::new();
    let lang = minimal_language();
    parser.set_language(lang).unwrap();
    parser.set_timeout(Duration::from_millis(200));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(200)));
}

#[test]
fn timeout_persists_after_parse_attempt() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(10));
    // No language set — parse will return Err, timeout should persist
    let _ = parser.parse(b"hello", None);
    assert_eq!(parser.timeout(), Some(Duration::from_secs(10)));
}

#[test]
fn timeout_persists_after_failed_parse() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(3));
    // No language set — parse will fail
    let result = parser.parse(b"test", None);
    assert!(result.is_err());
    assert_eq!(parser.timeout(), Some(Duration::from_secs(3)));
}

#[test]
fn timeout_persists_after_parse_utf8_attempt() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(500));
    // No language set — parse_utf8 will return Err, timeout should persist
    let _ = parser.parse_utf8("hello", None);
    assert_eq!(parser.timeout(), Some(Duration::from_millis(500)));
}

// --- Multiple timeout changes ---

#[test]
fn sequential_timeout_changes() {
    let mut parser = Parser::new();
    let durations = [
        Duration::from_millis(100),
        Duration::from_secs(1),
        Duration::from_micros(50),
        Duration::from_secs(30),
        Duration::from_nanos(999),
    ];
    for i in 0..durations.len() {
        parser.set_timeout(durations[i]);
        assert_eq!(parser.timeout(), Some(durations[i]));
    }
}

#[test]
fn timeout_changes_with_interleaved_resets() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(1));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(1)));
    parser.reset();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(1)));
    parser.set_timeout(Duration::from_secs(2));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(2)));
    parser.reset();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(2)));
}

#[test]
fn rapid_timeout_changes() {
    let mut parser = Parser::new();
    for i in 0..100u64 {
        parser.set_timeout(Duration::from_millis(i));
    }
    assert_eq!(parser.timeout(), Some(Duration::from_millis(99)));
}

#[test]
fn timeout_alternating_zero_and_nonzero() {
    let mut parser = Parser::new();
    for i in 0..10u64 {
        if i % 2 == 0 {
            parser.set_timeout(Duration::ZERO);
            assert_eq!(parser.timeout(), Some(Duration::ZERO));
        } else {
            parser.set_timeout(Duration::from_secs(i));
            assert_eq!(parser.timeout(), Some(Duration::from_secs(i)));
        }
    }
}

#[test]
fn timeout_increasing_sequence() {
    let mut parser = Parser::new();
    for exp in 0..20u32 {
        let dur = Duration::from_millis(2u64.pow(exp));
        parser.set_timeout(dur);
        assert_eq!(parser.timeout(), Some(dur));
    }
}

#[test]
fn timeout_decreasing_sequence() {
    let mut parser = Parser::new();
    for exp in (0..20u32).rev() {
        let dur = Duration::from_millis(2u64.pow(exp));
        parser.set_timeout(dur);
        assert_eq!(parser.timeout(), Some(dur));
    }
}

#[test]
fn timeout_independent_of_language_changes() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(42));

    let lang1 = minimal_language();
    parser.set_language(lang1).unwrap();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(42)));

    let lang2 = minimal_language();
    parser.set_language(lang2).unwrap();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(42)));
}

#[test]
fn timeout_with_reset_and_parse_cycle() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));

    // Cycle: failed parse attempt -> reset -> verify timeout persists
    for _ in 0..3 {
        let _ = parser.parse(b"x", None);
        assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
        parser.reset();
        assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
    }
}
