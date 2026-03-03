#![allow(clippy::needless_range_loop)]

//! Property-based tests for pattern extraction from leaf attributes in adze-common.
//!
//! Exercises extracting `text` and `pattern` (regex) values from
//! `NameValueExpr` / `FieldThenParams` parsed attribute parameters,
//! covering special characters, escaping, multiple patterns, empty
//! patterns, and determinism.

use adze_common::{FieldThenParams, NameValueExpr};
use proptest::prelude::*;
use quote::ToTokens;
use syn::Expr;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers (lowercase start, alphanumeric + underscore).
fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,10}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Simple leaf type names for unnamed fields.
fn type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&[
        "i32", "u32", "i64", "u64", "f32", "f64", "bool", "char", "String", "usize",
    ][..])
}

/// Printable ASCII characters safe inside a Rust string literal (no backslash, no quote).
fn safe_pattern_char() -> impl Strategy<Value = char> {
    prop::char::range(' ', '~').prop_filter("no backslash or quote", |c| {
        *c != '\\' && *c != '"'
    })
}

/// Non-empty pattern strings made of safe chars.
fn safe_pattern_string(max_len: usize) -> impl Strategy<Value = String> {
    prop::collection::vec(safe_pattern_char(), 1..=max_len)
        .prop_map(|v| v.into_iter().collect::<String>())
}

/// Simple regex-like patterns (safe subset).
fn simple_regex_pattern() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&[
        r"\d+",
        r"\w+",
        r"\s",
        r"[a-z]+",
        r"[A-Z][a-zA-Z0-9]*",
        r"[0-9]+",
        r"[-+*/]",
        r"\d+\.\d+",
        r"[_a-zA-Z][_a-zA-Z0-9]*",
        r"0[xX][0-9a-fA-F]+",
    ][..])
}

/// Special-character patterns that need careful handling.
fn special_char_pattern() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&[
        "+", "-", "*", "/", "(", ")", "{", "}", "[", "]",
        ".", ",", ";", ":", "!", "?", "@", "#", "$", "%",
        "^", "&", "|", "~", "<", ">", "=",
    ][..])
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the string value from an `Expr::Lit(LitStr)`.
fn extract_str_value(expr: &Expr) -> Option<String> {
    if let Expr::Lit(lit) = expr {
        if let syn::Lit::Str(s) = &lit.lit {
            return Some(s.value());
        }
    }
    None
}

/// Find a `NameValueExpr` by key in parsed `FieldThenParams` params.
fn find_param<'a>(
    ftp: &'a FieldThenParams,
    key: &str,
) -> Option<&'a NameValueExpr> {
    ftp.params.iter().find(|p| p.path == key)
}

// ---------------------------------------------------------------------------
// 1–8: Extract text pattern from leaf attribute
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// 1. A `text = "..."` param is extracted with the correct key.
    #[test]
    fn text_param_key_preserved(ty in type_name(), val in safe_pattern_string(12)) {
        let src = format!(r#"{ty}, text = "{val}""#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let param = find_param(&ftp, "text").unwrap();
        prop_assert_eq!(param.path.to_string(), "text");
    }

    /// 2. The text value round-trips through parsing.
    #[test]
    fn text_value_roundtrip(ty in type_name(), val in safe_pattern_string(12)) {
        let src = format!(r#"{ty}, text = "{val}""#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let param = find_param(&ftp, "text").unwrap();
        let extracted = extract_str_value(&param.expr).unwrap();
        prop_assert_eq!(extracted, val);
    }

    /// 3. Text pattern is a Lit expression.
    #[test]
    fn text_value_is_lit(ty in type_name(), val in safe_pattern_string(8)) {
        let src = format!(r#"{ty}, text = "{val}""#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let param = find_param(&ftp, "text").unwrap();
        prop_assert!(matches!(param.expr, Expr::Lit(_)));
    }

    /// 4. Text param token stream contains the literal.
    #[test]
    fn text_tokens_contain_value(ty in type_name(), val in safe_pattern_string(8)) {
        let src = format!(r#"{ty}, text = "{val}""#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let param = find_param(&ftp, "text").unwrap();
        let tokens = param.expr.to_token_stream().to_string();
        prop_assert!(tokens.contains(&val));
    }

    /// 5. text = "x" as standalone NameValueExpr.
    #[test]
    fn text_standalone_nve(val in safe_pattern_string(10)) {
        let src = format!(r#"text = "{val}""#);
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(nve.path.to_string(), "text");
        let extracted = extract_str_value(&nve.expr).unwrap();
        prop_assert_eq!(extracted, val);
    }

    /// 6. Clone preserves text pattern value.
    #[test]
    fn text_clone_preserves(val in safe_pattern_string(10)) {
        let src = format!(r#"text = "{val}""#);
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        let cloned = nve.clone();
        prop_assert_eq!(
            extract_str_value(&nve.expr),
            extract_str_value(&cloned.expr),
        );
    }

    /// 7. Text param length matches original.
    #[test]
    fn text_length_preserved(ty in type_name(), val in safe_pattern_string(15)) {
        let src = format!(r#"{ty}, text = "{val}""#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let extracted = extract_str_value(&find_param(&ftp, "text").unwrap().expr).unwrap();
        prop_assert_eq!(extracted.len(), val.len());
    }

    /// 8. Text param with single character.
    #[test]
    fn text_single_char(ty in type_name(), ch in safe_pattern_char()) {
        let src = format!(r#"{ty}, text = "{ch}""#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let extracted = extract_str_value(&find_param(&ftp, "text").unwrap().expr).unwrap();
        prop_assert_eq!(extracted, ch.to_string());
    }
}

// ---------------------------------------------------------------------------
// 9–15: Extract regex pattern from leaf attribute
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// 9. A `pattern = r"..."` param is extracted with correct key.
    #[test]
    fn pattern_param_key_preserved(ty in type_name(), pat in simple_regex_pattern()) {
        let src = format!(r##"{ty}, pattern = r"{pat}""##);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let param = find_param(&ftp, "pattern").unwrap();
        prop_assert_eq!(param.path.to_string(), "pattern");
    }

    /// 10. Regex value round-trips through parsing.
    #[test]
    fn pattern_value_roundtrip(ty in type_name(), pat in simple_regex_pattern()) {
        let src = format!(r##"{ty}, pattern = r"{pat}""##);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let param = find_param(&ftp, "pattern").unwrap();
        let extracted = extract_str_value(&param.expr).unwrap();
        prop_assert_eq!(extracted, pat);
    }

    /// 11. Regex pattern as standalone NameValueExpr.
    #[test]
    fn pattern_standalone_nve(pat in simple_regex_pattern()) {
        let src = format!(r##"pattern = r"{pat}""##);
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(nve.path.to_string(), "pattern");
        let extracted = extract_str_value(&nve.expr).unwrap();
        prop_assert_eq!(extracted, pat);
    }

    /// 12. Clone preserves regex pattern value.
    #[test]
    fn pattern_clone_preserves(pat in simple_regex_pattern()) {
        let src = format!(r##"pattern = r"{pat}""##);
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        let cloned = nve.clone();
        prop_assert_eq!(
            extract_str_value(&nve.expr),
            extract_str_value(&cloned.expr),
        );
    }

    /// 13. Regex pattern is a string literal expression.
    #[test]
    fn pattern_is_str_lit(ty in type_name(), pat in simple_regex_pattern()) {
        let src = format!(r##"{ty}, pattern = r"{pat}""##);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let param = find_param(&ftp, "pattern").unwrap();
        prop_assert!(extract_str_value(&param.expr).is_some());
    }

    /// 14. pattern and text are different params when both present.
    #[test]
    fn pattern_text_distinct_keys(ty in type_name(), pat in simple_regex_pattern(), txt in safe_pattern_string(6)) {
        let src = format!(r##"{ty}, pattern = r"{pat}", text = "{txt}""##);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert!(find_param(&ftp, "pattern").is_some());
        prop_assert!(find_param(&ftp, "text").is_some());
        let pat_val = extract_str_value(&find_param(&ftp, "pattern").unwrap().expr).unwrap();
        let txt_val = extract_str_value(&find_param(&ftp, "text").unwrap().expr).unwrap();
        prop_assert_eq!(pat_val, pat);
        prop_assert_eq!(txt_val, txt);
    }

    /// 15. Regex pattern length is preserved.
    #[test]
    fn pattern_length_preserved(pat in simple_regex_pattern()) {
        let src = format!(r##"pattern = r"{pat}""##);
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        let extracted = extract_str_value(&nve.expr).unwrap();
        prop_assert_eq!(extracted.len(), pat.len());
    }
}

// ---------------------------------------------------------------------------
// 16–19: Pattern with special characters
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// 16. Special-character text patterns round-trip.
    #[test]
    fn special_char_text_roundtrip(ty in type_name(), ch in special_char_pattern()) {
        let src = format!(r#"{ty}, text = "{ch}""#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let extracted = extract_str_value(&find_param(&ftp, "text").unwrap().expr).unwrap();
        prop_assert_eq!(extracted, ch);
    }

    /// 17. Special-character patterns parse as Lit.
    #[test]
    fn special_char_is_lit(ch in special_char_pattern()) {
        let src = format!(r#"text = "{ch}""#);
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert!(matches!(nve.expr, Expr::Lit(_)));
    }

    /// 18. Multiple special chars concatenated round-trip.
    #[test]
    fn special_chars_concat(
        ty in type_name(),
        chars in prop::collection::vec(special_char_pattern(), 1..=5),
    ) {
        let combined: String = chars.join("");
        let src = format!(r#"{ty}, text = "{combined}""#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let extracted = extract_str_value(&find_param(&ftp, "text").unwrap().expr).unwrap();
        prop_assert_eq!(extracted, combined);
    }

    /// 19. Special char clone preserves value.
    #[test]
    fn special_char_clone(ch in special_char_pattern()) {
        let src = format!(r#"text = "{ch}""#);
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        let cloned = nve.clone();
        prop_assert_eq!(extract_str_value(&nve.expr), extract_str_value(&cloned.expr));
    }
}

// ---------------------------------------------------------------------------
// 20–22: Pattern escaping
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// 20. Escaped sequences in regex patterns are preserved.
    #[test]
    fn escaped_regex_preserved(pat in simple_regex_pattern()) {
        // raw strings avoid Rust-level escaping; the value should match exactly
        let src = format!(r##"pattern = r"{pat}""##);
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        let extracted = extract_str_value(&nve.expr).unwrap();
        prop_assert_eq!(extracted, pat);
    }

    /// 21. Patterns with \\n / \\t via regular strings resolve escapes.
    #[test]
    fn escape_sequences_in_regular_strings(_i in 0..1u8) {
        // \n in a regular Rust string literal becomes a newline character
        let src = r#"text = "\n\t""#;
        let nve: NameValueExpr = syn::parse_str(src).unwrap();
        let extracted = extract_str_value(&nve.expr).unwrap();
        prop_assert_eq!(extracted, "\n\t");
    }

    /// 22. Unicode escape in pattern string.
    #[test]
    fn unicode_escape_pattern(_i in 0..1u8) {
        let src = r#"text = "\u{03B1}\u{03B2}""#;
        let nve: NameValueExpr = syn::parse_str(src).unwrap();
        let extracted = extract_str_value(&nve.expr).unwrap();
        prop_assert_eq!(extracted, "\u{03B1}\u{03B2}"); // αβ
    }
}

// ---------------------------------------------------------------------------
// 23: Pattern from string literal
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// 23. Arbitrary safe string literal values extract correctly.
    #[test]
    fn string_literal_extraction(val in safe_pattern_string(20)) {
        let src = format!(r#"text = "{val}""#);
        let nve: NameValueExpr = syn::parse_str(&src).unwrap();
        let extracted = extract_str_value(&nve.expr).unwrap();
        prop_assert_eq!(extracted, val);
    }
}

// ---------------------------------------------------------------------------
// 24–27: Multiple patterns
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// 24. Two params with different keys both extractable.
    #[test]
    fn two_params_both_extractable(
        ty in type_name(),
        k1 in ident_strategy(),
        k2 in ident_strategy(),
        v1 in safe_pattern_string(6),
        v2 in safe_pattern_string(6),
    ) {
        prop_assume!(k1 != k2);
        let src = format!(r#"{ty}, {k1} = "{v1}", {k2} = "{v2}""#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(ftp.params.len(), 2);
        let p1 = find_param(&ftp, &k1).unwrap();
        let p2 = find_param(&ftp, &k2).unwrap();
        prop_assert_eq!(extract_str_value(&p1.expr).unwrap(), v1);
        prop_assert_eq!(extract_str_value(&p2.expr).unwrap(), v2);
    }

    /// 25. Three params preserve all values.
    #[test]
    fn three_params_preserve_values(ty in type_name()) {
        let src = format!(r##"{ty}, pattern = r"\d+", text = "hello", transform = "id""##);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(ftp.params.len(), 3);
        prop_assert_eq!(extract_str_value(&find_param(&ftp, "pattern").unwrap().expr).unwrap(), r"\d+");
        prop_assert_eq!(extract_str_value(&find_param(&ftp, "text").unwrap().expr).unwrap(), "hello");
        prop_assert_eq!(extract_str_value(&find_param(&ftp, "transform").unwrap().expr).unwrap(), "id");
    }

    /// 26. Param ordering is preserved.
    #[test]
    fn param_ordering_preserved(
        ty in type_name(),
        k1 in ident_strategy(),
        k2 in ident_strategy(),
    ) {
        prop_assume!(k1 != k2);
        let src = format!(r#"{ty}, {k1} = "a", {k2} = "b""#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(ftp.params[0].path.to_string(), k1);
        prop_assert_eq!(ftp.params[1].path.to_string(), k2);
    }

    /// 27. Missing key returns None from find_param.
    #[test]
    fn missing_key_returns_none(ty in type_name(), val in safe_pattern_string(6)) {
        let src = format!(r#"{ty}, text = "{val}""#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert!(find_param(&ftp, "nonexistent").is_none());
    }
}

// ---------------------------------------------------------------------------
// 28–29: Empty pattern
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// 28. Empty text = "" parses successfully and extracts empty string.
    #[test]
    fn empty_text_pattern(ty in type_name()) {
        let src = format!(r#"{ty}, text = """#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let extracted = extract_str_value(&find_param(&ftp, "text").unwrap().expr).unwrap();
        prop_assert!(extracted.is_empty());
    }

    /// 29. Empty pattern = "" parses and extracts empty string.
    #[test]
    fn empty_regex_pattern(ty in type_name()) {
        let src = format!(r#"{ty}, pattern = """#);
        let ftp: FieldThenParams = syn::parse_str(&src).unwrap();
        let extracted = extract_str_value(&find_param(&ftp, "pattern").unwrap().expr).unwrap();
        prop_assert!(extracted.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 30–35: Pattern determinism
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// 30. Parsing text param twice yields equal NVE.
    #[test]
    fn text_deterministic(val in safe_pattern_string(10)) {
        let src = format!(r#"text = "{val}""#);
        let a: NameValueExpr = syn::parse_str(&src).unwrap();
        let b: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(a, b);
    }

    /// 31. Parsing regex param twice yields equal NVE.
    #[test]
    fn pattern_deterministic(pat in simple_regex_pattern()) {
        let src = format!(r##"pattern = r"{pat}""##);
        let a: NameValueExpr = syn::parse_str(&src).unwrap();
        let b: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(a, b);
    }

    /// 32. FTP with text param is deterministic.
    #[test]
    fn ftp_text_deterministic(ty in type_name(), val in safe_pattern_string(8)) {
        let src = format!(r#"{ty}, text = "{val}""#);
        let a: FieldThenParams = syn::parse_str(&src).unwrap();
        let b: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(a, b);
    }

    /// 33. FTP with pattern param is deterministic.
    #[test]
    fn ftp_pattern_deterministic(ty in type_name(), pat in simple_regex_pattern()) {
        let src = format!(r##"{ty}, pattern = r"{pat}""##);
        let a: FieldThenParams = syn::parse_str(&src).unwrap();
        let b: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(a, b);
    }

    /// 34. Extracted string value is deterministic across parses.
    #[test]
    fn extracted_value_deterministic(val in safe_pattern_string(10)) {
        let src = format!(r#"text = "{val}""#);
        let a: NameValueExpr = syn::parse_str(&src).unwrap();
        let b: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(extract_str_value(&a.expr), extract_str_value(&b.expr));
    }

    /// 35. Debug representation is deterministic.
    #[test]
    fn debug_deterministic(val in safe_pattern_string(10)) {
        let src = format!(r#"text = "{val}""#);
        let a: NameValueExpr = syn::parse_str(&src).unwrap();
        let b: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(format!("{a:?}"), format!("{b:?}"));
    }
}
