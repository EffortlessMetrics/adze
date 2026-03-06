#![allow(clippy::needless_range_loop)]

//! Property-based tests for Unicode handling in the adze runtime.
//!
//! Uses proptest to verify that UTF-8 text extraction, byte positions,
//! multi-byte character handling, error messages, node text, empty strings,
//! ASCII-only strings, and mixed ASCII + Unicode all behave correctly.

use std::mem::MaybeUninit;

use adze::errors::{ParseError, ParseErrorReason};
use adze::glr_lexer::GLRLexer;
use adze::pure_parser::{ParsedNode, Point};
use adze::{Extract, SpanError, SpanErrorReason, Spanned};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn pt(row: u32, col: u32) -> Point {
    Point { row, column: col }
}

/// Build a `ParsedNode` by writing each public field.
fn make_node(symbol: u16, children: Vec<ParsedNode>, start: usize, end: usize) -> ParsedNode {
    let mut uninit = MaybeUninit::<ParsedNode>::uninit();
    let ptr = uninit.as_mut_ptr();
    unsafe {
        std::ptr::write_bytes(ptr, 0, 1);
        std::ptr::addr_of_mut!((*ptr).symbol).write(symbol);
        std::ptr::addr_of_mut!((*ptr).children).write(children);
        std::ptr::addr_of_mut!((*ptr).start_byte).write(start);
        std::ptr::addr_of_mut!((*ptr).end_byte).write(end);
        std::ptr::addr_of_mut!((*ptr).start_point).write(pt(0, start as u32));
        std::ptr::addr_of_mut!((*ptr).end_point).write(pt(0, end as u32));
        std::ptr::addr_of_mut!((*ptr).is_extra).write(false);
        std::ptr::addr_of_mut!((*ptr).is_error).write(false);
        std::ptr::addr_of_mut!((*ptr).is_missing).write(false);
        std::ptr::addr_of_mut!((*ptr).is_named).write(true);
        std::ptr::addr_of_mut!((*ptr).field_id).write(None);
        uninit.assume_init()
    }
}

fn leaf(start: usize, end: usize) -> ParsedNode {
    make_node(1, vec![], start, end)
}

/// Grammar: expr → id  (where id matches any non-whitespace run).
fn unicode_id_grammar() -> Grammar {
    let mut g = Grammar::new("unicode".into());
    let id = SymbolId(1);
    let ws = SymbolId(2);
    let expr = SymbolId(10);

    g.tokens.insert(
        id,
        Token {
            name: "id".into(),
            pattern: TokenPattern::Regex(r"[^\s]+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        ws,
        Token {
            name: "ws".into(),
            pattern: TokenPattern::Regex(r"\s+".into()),
            fragile: false,
        },
    );

    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });
    g.rule_names.insert(expr, "expression".into());
    g
}

// ---------------------------------------------------------------------------
// Proptest strategies
// ---------------------------------------------------------------------------

/// Arbitrary valid UTF-8 string (1..128 chars).
fn any_utf8() -> impl Strategy<Value = String> {
    "\\PC{1,128}"
}

/// ASCII-only printable string (1..128 chars).
fn ascii_only() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_]{1,128}"
}

/// Strategy producing strings with multi-byte chars (CJK, emoji, accented).
fn multibyte_str() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "café".to_string(),
        "naïve".to_string(),
        "日本語".to_string(),
        "世界".to_string(),
        "🦀🎉🚀".to_string(),
        "مرحبا".to_string(),
        "שלום".to_string(),
        "Ελληνικά".to_string(),
        "한국어".to_string(),
        "𝐀𝐁𝐂".to_string(),
        "e\u{0301}".to_string(),
        "a\u{0303}\u{0301}".to_string(),
        "€100".to_string(),
        "\u{FFFD}".to_string(),
        "Ω≈ç".to_string(),
    ])
}

/// Mixed ASCII + Unicode tokens separated by spaces.
fn mixed_tokens() -> impl Strategy<Value = String> {
    prop::collection::vec(prop_oneof![ascii_only(), multibyte_str()], 1..=5)
        .prop_map(|v| v.join(" "))
}

// ===========================================================================
// 1. UTF-8 text extraction — String::extract preserves full text
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn extract_string_preserves_utf8(s in any_utf8()) {
        let source = s.as_bytes();
        let node = leaf(0, source.len());
        let result: String = String::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, s);
    }
}

// ===========================================================================
// 2. Extract from sub-range of Unicode source
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn extract_string_sub_range(s in any_utf8()) {
        let source = s.as_bytes();
        // Pick a sub-range on char boundaries
        let chars: Vec<(usize, char)> = s.char_indices().collect();
        if chars.len() >= 2 {
            let start_idx = chars[0].0;
            let end_idx = chars[chars.len() / 2].0;
            if start_idx < end_idx && end_idx <= source.len() {
                let node = leaf(start_idx, end_idx);
                let result: String = String::extract(Some(&node), source, 0, None);
                let expected = &s[start_idx..end_idx];
                prop_assert_eq!(result, expected);
            }
        }
    }
}

// ===========================================================================
// 3. Unicode byte positions — byte_len matches String::len
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn byte_length_equals_string_len(s in any_utf8()) {
        let bytes = s.as_bytes();
        prop_assert_eq!(bytes.len(), s.len());
    }
}

// ===========================================================================
// 4. Multi-byte character — each char's UTF-8 len is 1..=4
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn each_char_utf8_len_1_to_4(s in any_utf8()) {
        for ch in s.chars() {
            let len = ch.len_utf8();
            prop_assert!((1..=4).contains(&len), "char {:?} has len {}", ch, len);
        }
    }
}

// ===========================================================================
// 5. char_indices byte offsets are consistent
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn char_indices_monotonic_and_bounded(s in any_utf8()) {
        let indices: Vec<(usize, char)> = s.char_indices().collect();
        for i in 1..indices.len() {
            prop_assert!(indices[i].0 > indices[i - 1].0);
        }
        if let Some(&(last_off, last_ch)) = indices.last() {
            prop_assert_eq!(last_off + last_ch.len_utf8(), s.len());
        }
    }
}

// ===========================================================================
// 6. Unicode in error messages — ParseError with Unicode token text
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn parse_error_preserves_unicode_token(s in multibyte_str()) {
        let err = ParseError {
            reason: ParseErrorReason::UnexpectedToken(s.clone()),
            start: 0,
            end: s.len(),
        };
        if let ParseErrorReason::UnexpectedToken(ref tok) = err.reason {
            prop_assert_eq!(tok, &s);
        } else {
            prop_assert!(false, "wrong variant");
        }
    }
}

// ===========================================================================
// 7. Unicode in error messages — MissingToken
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn parse_error_missing_token_unicode(s in multibyte_str()) {
        let err = ParseError {
            reason: ParseErrorReason::MissingToken(s.clone()),
            start: 0,
            end: 0,
        };
        if let ParseErrorReason::MissingToken(ref tok) = err.reason {
            prop_assert_eq!(tok, &s);
        } else {
            prop_assert!(false, "wrong variant");
        }
    }
}

// ===========================================================================
// 8. Unicode in node text — extract from multibyte source
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn extract_multibyte_node_text(s in multibyte_str()) {
        let source = s.as_bytes();
        let node = leaf(0, source.len());
        let result: String = String::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, s);
    }
}

// ===========================================================================
// 9. Empty string handling — extract from empty source
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn extract_empty_string(_dummy in 0u8..1) {
        let source: &[u8] = b"";
        let node = leaf(0, 0);
        let result: String = String::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, "");
    }
}

// ===========================================================================
// 10. Extract None yields empty string
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn extract_none_yields_empty(s in any_utf8()) {
        let source = s.as_bytes();
        let result: String = String::extract(None, source, 0, None);
        prop_assert_eq!(result, "");
    }
}

// ===========================================================================
// 11. ASCII-only strings — round-trip through extract
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn extract_ascii_only(s in ascii_only()) {
        let source = s.as_bytes();
        let node = leaf(0, source.len());
        let result: String = String::extract(Some(&node), source, 0, None);
        // All bytes are single-byte chars
        prop_assert_eq!(s.chars().count(), s.len());
        prop_assert_eq!(result, s);
    }
}

// ===========================================================================
// 12. Mixed ASCII + Unicode — extract preserves mixed content
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn extract_mixed_ascii_unicode(s in mixed_tokens()) {
        let source = s.as_bytes();
        let node = leaf(0, source.len());
        let result: String = String::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, s);
    }
}

// ===========================================================================
// 13. Spanned indexing with Unicode source
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn spanned_index_unicode_source(s in any_utf8()) {
        let chars: Vec<(usize, char)> = s.char_indices().collect();
        if chars.len() >= 2 {
            let start = chars[0].0;
            let end = chars[chars.len() / 2].0;
            if start <= end && end <= s.len() {
                let span = Spanned { value: (), span: (start, end) };
                let sliced = &s.as_str()[span];
                prop_assert_eq!(sliced, &s[start..end]);
            }
        }
    }
}

// ===========================================================================
// 14. Spanned full-range on Unicode
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn spanned_full_range_unicode(s in any_utf8()) {
        let span = Spanned { value: (), span: (0, s.len()) };
        let sliced = &s.as_str()[span];
        prop_assert_eq!(sliced, s.as_str());
    }
}

// ===========================================================================
// 15. Spanned empty span at any char boundary
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn spanned_empty_span_at_char_boundary(s in any_utf8()) {
        let boundaries: Vec<usize> = s.char_indices().map(|(i, _)| i).chain(std::iter::once(s.len())).collect();
        for &b in &boundaries {
            let span = Spanned { value: (), span: (b, b) };
            let sliced = &s.as_str()[span];
            prop_assert_eq!(sliced, "");
        }
    }
}

// ===========================================================================
// 16. SpanError display with Unicode-length source
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn span_error_display_with_unicode_len(s in any_utf8()) {
        let src_len = s.len();
        if src_len > 0 {
            let error = SpanError {
                span: (0, src_len + 1),
                source_len: src_len,
                reason: SpanErrorReason::EndOutOfBounds,
            };
            let msg = error.to_string();
            let expected_fragment = format!("source length ({})", src_len);
            prop_assert!(msg.contains(&expected_fragment), "msg={}", msg);
        }
    }
}

// ===========================================================================
// 17. GLRLexer tokenizes arbitrary non-whitespace Unicode
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn glr_lexer_tokenizes_nonws_unicode(s in multibyte_str()) {
        let g = unicode_id_grammar();
        let trimmed = s.trim();
        if !trimmed.is_empty() && !trimmed.contains(char::is_whitespace) {
            let mut lexer = GLRLexer::new(&g, trimmed.to_string()).unwrap();
            let tokens = lexer.tokenize_all();
            prop_assert!(!tokens.is_empty(), "should produce at least one token for {:?}", trimmed);
            prop_assert_eq!(&tokens[0].text, trimmed);
            prop_assert_eq!(tokens[0].byte_length, trimmed.len());
        }
    }
}

// ===========================================================================
// 18. GLRLexer byte offsets sum to input length
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn glr_lexer_byte_offsets_consistent(s in mixed_tokens()) {
        let g = unicode_id_grammar();
        if !s.trim().is_empty() {
            let mut lexer = GLRLexer::new(&g, s.clone()).unwrap();
            let tokens = lexer.tokenize_all();
            // Every token's range must be within the input
            for tok in &tokens {
                prop_assert!(
                    tok.byte_offset + tok.byte_length <= s.len(),
                    "token {:?} at {}+{} exceeds input len {}",
                    tok.text, tok.byte_offset, tok.byte_length, s.len()
                );
            }
            // Tokens should be in non-decreasing offset order
            for i in 1..tokens.len() {
                prop_assert!(
                    tokens[i].byte_offset >= tokens[i - 1].byte_offset,
                    "tokens not in order: {} vs {}",
                    tokens[i - 1].byte_offset, tokens[i].byte_offset
                );
            }
        }
    }
}

// ===========================================================================
// 19. GLRLexer token text matches source slice
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn glr_lexer_token_text_matches_source(s in mixed_tokens()) {
        let g = unicode_id_grammar();
        if !s.trim().is_empty() {
            let mut lexer = GLRLexer::new(&g, s.clone()).unwrap();
            let tokens = lexer.tokenize_all();
            for tok in &tokens {
                let expected = &s[tok.byte_offset..tok.byte_offset + tok.byte_length];
                prop_assert_eq!(
                    &tok.text, expected,
                    "token text mismatch at offset {}",
                    tok.byte_offset
                );
            }
        }
    }
}

// ===========================================================================
// 20. Spanned extract preserves byte span with Unicode
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn spanned_extract_preserves_span(s in any_utf8()) {
        let source = s.as_bytes();
        let node = leaf(0, source.len());
        let result: Spanned<String> = <Spanned<String>>::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result.span, (0, source.len()));
        prop_assert_eq!(&*result, &s);
    }
}

// ===========================================================================
// 21. Multi-byte char boundary alignment
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn char_boundary_alignment(s in any_utf8()) {
        // Every byte offset returned by char_indices is a valid char boundary
        for (i, _) in s.char_indices() {
            prop_assert!(s.is_char_boundary(i), "offset {} is not a char boundary in {:?}", i, s);
        }
        prop_assert!(s.is_char_boundary(s.len()));
    }
}

// ===========================================================================
// 22. Extract from each individual char in a Unicode string
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn extract_each_char_individually(s in any_utf8()) {
        let source = s.as_bytes();
        for (i, ch) in s.char_indices() {
            let start = i;
            let end = i + ch.len_utf8();
            let node = leaf(start, end);
            let result: String = String::extract(Some(&node), source, 0, None);
            prop_assert_eq!(result, ch.to_string());
        }
    }
}

// ===========================================================================
// 23. Byte sum of all chars equals string byte length
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn byte_sum_of_chars(s in any_utf8()) {
        let sum: usize = s.chars().map(|c| c.len_utf8()).sum();
        prop_assert_eq!(sum, s.len());
    }
}

// ===========================================================================
// 24. ASCII strings: char count equals byte count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn ascii_char_count_eq_byte_count(s in ascii_only()) {
        prop_assert_eq!(s.chars().count(), s.len());
        for ch in s.chars() {
            prop_assert!(ch.is_ascii());
            prop_assert_eq!(ch.len_utf8(), 1);
        }
    }
}

// ===========================================================================
// 25. Mixed ASCII + Unicode: byte length >= char count
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn mixed_byte_length_ge_char_count(s in mixed_tokens()) {
        prop_assert!(s.len() >= s.chars().count());
    }
}

// ===========================================================================
// 26. GLRLexer does not panic on arbitrary UTF-8
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn glr_lexer_no_panic_on_arbitrary_utf8(s in any_utf8()) {
        let g = unicode_id_grammar();
        // Should never panic regardless of input
        let result = GLRLexer::new(&g, s);
        if let Ok(mut lexer) = result {
            let _ = lexer.tokenize_all();
        }
    }
}

// ===========================================================================
// 27. Spanned validate_span on Unicode source lengths
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn validate_span_unicode_lengths(s in any_utf8()) {
        let len = s.len();
        // Valid full span
        let span = Spanned { value: (), span: (0, len) };
        let sliced = &s.as_str()[span];
        prop_assert_eq!(sliced, s.as_str());

        // Valid empty span
        let span = Spanned { value: (), span: (len, len) };
        let sliced = &s.as_str()[span];
        prop_assert_eq!(sliced, "");
    }
}

// ===========================================================================
// 28. ParseError byte ranges valid within Unicode source
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn parse_error_byte_ranges_valid(s in any_utf8()) {
        let chars: Vec<(usize, char)> = s.char_indices().collect();
        if chars.len() >= 2 {
            let start = chars[0].0;
            let end = chars[chars.len() - 1].0 + chars[chars.len() - 1].1.len_utf8();
            let err = ParseError {
                reason: ParseErrorReason::UnexpectedToken(s[start..end].to_string()),
                start,
                end,
            };
            prop_assert!(err.start <= err.end);
            prop_assert!(err.end <= s.len());
            if let ParseErrorReason::UnexpectedToken(ref tok) = err.reason {
                prop_assert_eq!(tok, &s[start..end]);
            }
        }
    }
}

// ===========================================================================
// 29. Extract Option<String> with Unicode
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn extract_option_string_unicode(s in multibyte_str()) {
        let source = s.as_bytes();
        let node = leaf(0, source.len());
        let result: Option<String> = <Option<String>>::extract(Some(&node), source, 0, None);
        prop_assert_eq!(result, Some(s));
    }
}

// ===========================================================================
// 30. Extract Option<String> None with Unicode source
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn extract_option_none_unicode(s in multibyte_str()) {
        let source = s.as_bytes();
        let result: Option<String> = <Option<String>>::extract(None, source, 0, None);
        prop_assert_eq!(result, None);
    }
}

// ===========================================================================
// 31. SpanError reason variants cover Unicode edge cases
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn span_error_variants_unicode(s in any_utf8()) {
        let len = s.len();
        if len > 0 {
            // StartGreaterThanEnd
            let e1 = SpanError {
                span: (len, 0),
                source_len: len,
                reason: SpanErrorReason::StartGreaterThanEnd,
            };
            prop_assert!(e1.to_string().contains("start"));

            // EndOutOfBounds
            let e2 = SpanError {
                span: (0, len + 1),
                source_len: len,
                reason: SpanErrorReason::EndOutOfBounds,
            };
            prop_assert!(e2.to_string().contains("end"));

            // StartOutOfBounds
            let e3 = SpanError {
                span: (len + 1, len + 2),
                source_len: len,
                reason: SpanErrorReason::StartOutOfBounds,
            };
            prop_assert!(e3.to_string().contains("start"));
        }
    }
}

// ===========================================================================
// 32. from_utf8 round-trip for arbitrary Unicode
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn utf8_roundtrip(s in any_utf8()) {
        let bytes = s.as_bytes();
        let recovered = std::str::from_utf8(bytes).unwrap();
        prop_assert_eq!(recovered, s.as_str());
    }
}
