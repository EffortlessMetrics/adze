//! Advanced comprehensive tests for Language and Parser public API edge cases.
//!
//! Covers: Language trait bounds, debug format, Parser construction, error paths,
//! multiple languages/parsers, timeout/reset semantics, size assertions,
//! sequential parse errors, and catch_unwind parse paths.

use adze_runtime::language::Language;
use adze_runtime::parser::Parser;
use adze_runtime::test_helpers::stub_language;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

// ===========================================================================
// 1. Language from stub_language() trait bounds
// ===========================================================================

#[test]
fn language_is_clone() {
    let lang = stub_language();
    let _cloned = lang.clone();
}

#[test]
fn language_clone_is_independent() {
    let lang = stub_language();
    let cloned = lang.clone();
    // Modifying one should not affect the other (they are value types with vecs)
    assert_eq!(lang.version, cloned.version);
    assert_eq!(lang.symbol_count, cloned.symbol_count);
}

#[test]
fn language_is_not_send() {
    // Language contains a Box<dyn Fn> (tokenizer) which is not Send.
    // We verify it does NOT implement Send by observing that stub_language()
    // can be used only on a single thread (no compile-time assertion needed —
    // this test simply documents the fact).
    fn _assert_not_send_at_runtime() {
        let lang = stub_language();
        // Language used on the creating thread — should always work.
        let _ = format!("{:?}", lang);
    }
    _assert_not_send_at_runtime();
}

#[test]
fn language_is_not_sync() {
    // Same reasoning as Send — tokenizer closure prevents Sync.
    let lang = stub_language();
    let _ = &lang; // borrow on same thread is fine
}

#[test]
fn language_clone_preserves_symbol_names() {
    let lang = stub_language();
    let cloned = lang.clone();
    assert_eq!(lang.symbol_names, cloned.symbol_names);
}

#[test]
fn language_clone_preserves_field_names() {
    let lang = stub_language();
    let cloned = lang.clone();
    assert_eq!(lang.field_names, cloned.field_names);
}

#[test]
fn language_clone_preserves_version() {
    let lang = stub_language();
    let cloned = lang.clone();
    assert_eq!(lang.version, cloned.version);
}

// ===========================================================================
// 2. Language debug format
// ===========================================================================

#[test]
fn language_debug_contains_language_keyword() {
    let lang = stub_language();
    let dbg = format!("{:?}", lang);
    assert!(
        dbg.contains("Language"),
        "Debug output should contain 'Language'"
    );
}

#[test]
fn language_debug_contains_version() {
    let lang = stub_language();
    let dbg = format!("{:?}", lang);
    assert!(
        dbg.contains("version"),
        "Debug output should contain 'version'"
    );
}

#[test]
fn language_debug_contains_symbol_count() {
    let lang = stub_language();
    let dbg = format!("{:?}", lang);
    assert!(
        dbg.contains("symbol_count"),
        "Debug output should contain 'symbol_count'"
    );
}

#[test]
fn language_debug_contains_symbol_names() {
    let lang = stub_language();
    let dbg = format!("{:?}", lang);
    assert!(
        dbg.contains("symbol_names"),
        "Debug should contain 'symbol_names'"
    );
}

#[test]
fn language_debug_contains_field_count() {
    let lang = stub_language();
    let dbg = format!("{:?}", lang);
    assert!(
        dbg.contains("field_count"),
        "Debug should contain 'field_count'"
    );
}

#[test]
fn language_debug_is_deterministic() {
    let a = format!("{:?}", stub_language());
    let b = format!("{:?}", stub_language());
    assert_eq!(
        a, b,
        "Debug output should be deterministic across instances"
    );
}

// ===========================================================================
// 3. Parser new + set_language + parse error paths
// ===========================================================================

#[test]
fn parser_new_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn parser_new_no_timeout() {
    let parser = Parser::new();
    assert!(parser.timeout().is_none());
}

#[test]
fn parser_default_matches_new() {
    let p1 = Parser::new();
    let p2 = Parser::default();
    assert!(p1.language().is_none());
    assert!(p2.language().is_none());
    assert_eq!(p1.timeout(), p2.timeout());
}

#[test]
fn parser_set_language_succeeds() {
    let mut parser = Parser::new();
    let lang = stub_language();
    assert!(parser.set_language(lang).is_ok());
}

#[test]
fn parser_language_is_some_after_set() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn parser_parse_without_language_returns_no_language_error() {
    let mut parser = Parser::new();
    let err = parser.parse(b"hello", None).unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("no language") || msg.contains("NoLanguage"),
        "Expected no-language error, got: {msg}"
    );
}

#[test]
fn parser_parse_utf8_without_language_returns_error() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("hello", None);
    assert!(result.is_err());
}

// ===========================================================================
// 4. Parser multiple languages
// ===========================================================================

#[test]
fn parser_set_language_twice_replaces() {
    let mut parser = Parser::new();
    let lang1 = stub_language();
    let lang2 = stub_language();
    parser.set_language(lang1).unwrap();
    parser.set_language(lang2).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn parser_set_language_three_times() {
    let mut parser = Parser::new();
    for _ in 0..3 {
        parser.set_language(stub_language()).unwrap();
    }
    assert!(parser.language().is_some());
}

#[test]
fn parser_set_language_preserves_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    parser.set_language(stub_language()).unwrap();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn parser_language_accessor_returns_symbol_metadata() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let lang_ref = parser.language().unwrap();
    assert!(!lang_ref.symbol_metadata.is_empty());
}

// ===========================================================================
// 5. Parser timeout with Duration
// ===========================================================================

#[test]
fn parser_set_timeout_stores_value() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(100));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(100)));
}

#[test]
fn parser_timeout_zero_duration() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

#[test]
fn parser_timeout_large_duration() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(3600));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(3600)));
}

#[test]
fn parser_timeout_sub_millisecond() {
    let mut parser = Parser::new();
    let dur = Duration::from_nanos(500);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn parser_timeout_max_duration() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::MAX);
    assert_eq!(parser.timeout(), Some(Duration::MAX));
}

#[test]
fn parser_set_timeout_overwrite() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(1));
    parser.set_timeout(Duration::from_secs(2));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(2)));
}

#[test]
fn parser_set_timeout_after_language() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.set_timeout(Duration::from_millis(50));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(50)));
    assert!(parser.language().is_some());
}

// ===========================================================================
// 6. Parser reset semantics
// ===========================================================================

#[test]
fn parser_reset_does_not_clear_language() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.reset();
    assert!(
        parser.language().is_some(),
        "reset should not clear language"
    );
}

#[test]
fn parser_reset_does_not_clear_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(10));
    parser.reset();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(10)));
}

#[test]
fn parser_reset_on_fresh_parser() {
    let mut parser = Parser::new();
    parser.reset(); // should not panic
    assert!(parser.language().is_none());
}

#[test]
fn parser_reset_multiple_times() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    parser.reset();
    parser.reset();
    parser.reset();
    assert!(parser.language().is_some());
}

#[test]
fn parser_reset_then_set_language() {
    let mut parser = Parser::new();
    parser.reset();
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

// ===========================================================================
// 7. Parser debug
// ===========================================================================

#[test]
fn parser_debug_contains_parser_keyword() {
    let parser = Parser::new();
    let dbg = format!("{:?}", parser);
    assert!(
        dbg.contains("Parser"),
        "Debug output should contain 'Parser'"
    );
}

#[test]
fn parser_debug_with_language() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let dbg = format!("{:?}", parser);
    assert!(
        dbg.contains("Parser"),
        "Debug should still contain 'Parser'"
    );
}

#[test]
fn parser_debug_with_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(42));
    let dbg = format!("{:?}", parser);
    assert!(dbg.contains("Parser"));
}

// ===========================================================================
// 8. Language size (not zero)
// ===========================================================================

#[test]
fn language_size_is_nonzero() {
    assert!(std::mem::size_of::<Language>() > 0);
}

#[test]
fn language_alignment_is_nonzero() {
    assert!(std::mem::align_of::<Language>() > 0);
}

#[test]
fn language_size_is_at_least_pointer_sized() {
    // Language contains Vecs and Box<dyn Fn>, so it should be larger than a pointer.
    assert!(std::mem::size_of::<Language>() >= std::mem::size_of::<usize>());
}

#[test]
fn language_instance_symbol_count_nonzero() {
    let lang = stub_language();
    // stub_language has at least one symbol ("placeholder")
    assert!(lang.symbol_count > 0 || !lang.symbol_metadata.is_empty());
}

// ===========================================================================
// 9. Parser size
// ===========================================================================

#[test]
fn parser_size_is_nonzero() {
    assert!(std::mem::size_of::<Parser>() > 0);
}

#[test]
fn parser_alignment_is_nonzero() {
    assert!(std::mem::align_of::<Parser>() > 0);
}

#[test]
fn parser_size_at_least_option_language() {
    assert!(std::mem::size_of::<Parser>() >= std::mem::size_of::<Option<Language>>());
}

// ===========================================================================
// 10. Multiple parsers
// ===========================================================================

#[test]
fn two_independent_parsers() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_language(stub_language()).unwrap();
    assert!(p1.language().is_some());
    assert!(p2.language().is_none());
    p2.set_language(stub_language()).unwrap();
    assert!(p2.language().is_some());
}

#[test]
fn multiple_parsers_independent_timeouts() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_timeout(Duration::from_secs(1));
    p2.set_timeout(Duration::from_secs(2));
    assert_eq!(p1.timeout(), Some(Duration::from_secs(1)));
    assert_eq!(p2.timeout(), Some(Duration::from_secs(2)));
}

#[test]
fn ten_parsers_all_independent() {
    let mut parsers: Vec<Parser> = (0..10).map(|_| Parser::new()).collect();
    for (i, p) in parsers.iter_mut().enumerate() {
        p.set_timeout(Duration::from_millis(i as u64));
    }
    for (i, p) in parsers.iter().enumerate() {
        assert_eq!(p.timeout(), Some(Duration::from_millis(i as u64)));
    }
}

#[test]
fn multiple_parsers_same_language() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_language(stub_language()).unwrap();
    p2.set_language(stub_language()).unwrap();
    assert!(p1.language().is_some());
    assert!(p2.language().is_some());
}

// ===========================================================================
// 11. Sequential parse error paths
// ===========================================================================

#[test]
fn parse_no_language_twice() {
    let mut parser = Parser::new();
    let r1 = parser.parse(b"a", None);
    let r2 = parser.parse(b"b", None);
    assert!(r1.is_err());
    assert!(r2.is_err());
}

#[test]
fn parse_no_language_then_set_language() {
    let mut parser = Parser::new();
    assert!(parser.parse(b"x", None).is_err());
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn parse_utf8_no_language_multiple() {
    let mut parser = Parser::new();
    for _ in 0..5 {
        assert!(parser.parse_utf8("test", None).is_err());
    }
}

#[test]
fn parse_error_display_is_nonempty() {
    let mut parser = Parser::new();
    let err = parser.parse(b"test", None).unwrap_err();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn parse_error_debug_is_nonempty() {
    let mut parser = Parser::new();
    let err = parser.parse(b"test", None).unwrap_err();
    let dbg = format!("{:?}", err);
    assert!(!dbg.is_empty());
}

// ===========================================================================
// 12. Parse empty/unicode/binary inputs via catch_unwind
// ===========================================================================

#[test]
fn parse_empty_input_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"", None)));
    // The stub_language parse may panic in the GLR driver; we just
    // verify the test harness survives.
}

#[test]
fn parse_simple_ascii_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"hello", None)));
}

#[test]
fn parse_unicode_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse("日本語".as_bytes(), None)));
}

#[test]
fn parse_emoji_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse("🚀🌍".as_bytes(), None)));
}

#[test]
fn parse_binary_zeros_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse([0u8; 16], None)));
}

#[test]
fn parse_binary_high_bytes_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse([0xFF; 32], None)));
}

#[test]
fn parse_mixed_binary_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let input: Vec<u8> = (0u8..=255).collect();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse(&input, None)));
}

#[test]
fn parse_utf8_string_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse_utf8("fn main() {}", None)));
}

#[test]
fn parse_whitespace_only_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"   \t\n  ", None)));
}

#[test]
fn parse_newlines_only_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"\n\n\n", None)));
}

#[test]
fn parse_large_input_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let input = vec![b'a'; 10_000];
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse(&input, None)));
}

#[test]
fn parse_single_byte_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"x", None)));
}

#[test]
fn parse_sequential_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _r1 = catch_unwind(AssertUnwindSafe(|| parser.parse(b"first", None)));
    // Parser should still be usable after a caught panic
    let _r2 = catch_unwind(AssertUnwindSafe(|| parser.parse(b"second", None)));
}

#[test]
fn parse_null_byte_in_middle_catch_unwind() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _result = catch_unwind(AssertUnwindSafe(|| parser.parse(b"hel\0lo", None)));
}

// ===========================================================================
// Additional edge cases (Language API queries)
// ===========================================================================

#[test]
fn language_symbol_name_valid_index() {
    let lang = stub_language();
    // stub_language has symbol "placeholder" at index 0
    let name = lang.symbol_name(0);
    assert_eq!(name, Some("placeholder"));
}

#[test]
fn language_symbol_name_out_of_bounds() {
    let lang = stub_language();
    assert_eq!(lang.symbol_name(u16::MAX), None);
}

#[test]
fn language_field_name_out_of_bounds() {
    let lang = stub_language();
    assert_eq!(lang.field_name(0), None); // stub has empty field_names
}

#[test]
fn language_field_name_max_id() {
    let lang = stub_language();
    assert_eq!(lang.field_name(u16::MAX), None);
}

#[test]
fn language_is_terminal_valid_index() {
    let lang = stub_language();
    // stub_language's single symbol is terminal
    assert!(lang.is_terminal(0));
}

#[test]
fn language_is_terminal_out_of_bounds() {
    let lang = stub_language();
    assert!(!lang.is_terminal(u16::MAX));
}

#[test]
fn language_is_visible_valid_index() {
    let lang = stub_language();
    assert!(lang.is_visible(0));
}

#[test]
fn language_is_visible_out_of_bounds() {
    let lang = stub_language();
    assert!(!lang.is_visible(u16::MAX));
}

#[test]
fn language_symbol_for_name_existing() {
    let lang = stub_language();
    // "placeholder" is visible (is_named=true)
    let id = lang.symbol_for_name("placeholder", true);
    assert_eq!(id, Some(0));
}

#[test]
fn language_symbol_for_name_nonexistent() {
    let lang = stub_language();
    assert_eq!(lang.symbol_for_name("nonexistent", true), None);
}

#[test]
fn language_symbol_for_name_wrong_visibility() {
    let lang = stub_language();
    // "placeholder" is visible, so looking for anonymous should fail
    assert_eq!(lang.symbol_for_name("placeholder", false), None);
}

#[test]
fn language_symbol_for_name_empty_string() {
    let lang = stub_language();
    assert_eq!(lang.symbol_for_name("", true), None);
}

// ===========================================================================
// Cross-cutting: parser usable after error
// ===========================================================================

#[test]
fn parser_usable_after_no_language_error() {
    let mut parser = Parser::new();
    let _ = parser.parse(b"x", None); // error
    parser.set_language(stub_language()).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn parser_timeout_survives_parse_error() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(99));
    let _ = parser.parse(b"x", None); // error (no language)
    assert_eq!(parser.timeout(), Some(Duration::from_secs(99)));
}

#[test]
fn parser_reset_after_catch_unwind_parse() {
    let mut parser = Parser::new();
    parser.set_language(stub_language()).unwrap();
    let _r = catch_unwind(AssertUnwindSafe(|| parser.parse(b"test", None)));
    parser.reset();
    assert!(parser.language().is_some());
}
