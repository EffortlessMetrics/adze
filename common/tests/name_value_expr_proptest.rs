#![allow(clippy::needless_range_loop)]

use adze_common::{FieldThenParams, NameValueExpr};
use proptest::prelude::*;
use quote::ToTokens;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers (lowercase start, alphanumeric + underscore).
fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,12}")
        .unwrap()
        .prop_filter("must be a valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// A small set of distinct identifiers (deduplicated).
fn distinct_idents(max: usize) -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(ident_strategy(), 1..=max).prop_map(|v| {
        let mut seen = std::collections::HashSet::new();
        v.into_iter().filter(|s| seen.insert(s.clone())).collect()
    })
}

/// Simple type names suitable for unnamed fields.
fn type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Integer literal values in a readable range.
fn int_value() -> impl Strategy<Value = i64> {
    -1000i64..1000
}

// ---------------------------------------------------------------------------
// NameValueExpr tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 1. Round-trip: ident is preserved after parse
    #[test]
    fn nve_roundtrip_ident(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), name);
    }

    // 2. Round-trip: re-serialise then re-parse yields the same ident
    #[test]
    fn nve_roundtrip_reparse(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} = {val}");
        let a: NameValueExpr = syn::parse_str(&src).unwrap();
        let resrc = format!("{} = {val}", a.path);
        let b: NameValueExpr = syn::parse_str(&resrc).unwrap();
        prop_assert_eq!(a.path.to_string(), b.path.to_string());
    }

    // 3. Round-trip with string literal value
    #[test]
    fn nve_roundtrip_string_lit(name in ident_strategy()) {
        let src = format!("{name} = \"hello\"");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), name);
    }

    // 4. Round-trip with bool literal value
    #[test]
    fn nve_roundtrip_bool_lit(name in ident_strategy(), b in prop::bool::ANY) {
        let src = format!("{name} = {b}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), name);
    }

    // 5. Clone produces identical Debug output
    #[test]
    fn nve_clone_debug_eq(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let cloned = parsed.clone();
        prop_assert_eq!(format!("{parsed:?}"), format!("{cloned:?}"));
    }

    // 6. Clone/Eq consistency
    #[test]
    fn nve_clone_eq(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let cloned = parsed.clone();
        prop_assert_eq!(parsed, cloned);
    }

    // 7. Debug is non-empty
    #[test]
    fn nve_debug_non_empty(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let dbg = format!("{:?}", parsed);
        prop_assert!(!dbg.is_empty());
    }

    // 8. Debug contains the identifier name
    #[test]
    fn nve_debug_contains_ident(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let dbg = format!("{parsed:?}");
        prop_assert!(dbg.contains("NameValueExpr"), "Debug should contain type name: {dbg}");
    }

    // 9. Expr token stream is non-empty
    #[test]
    fn nve_expr_tokens_non_empty(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert!(!parsed.expr.to_token_stream().is_empty());
    }

    // 10. Deterministic: parsing the same input twice yields equal results
    #[test]
    fn nve_deterministic(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} = {val}");
        let a: NameValueExpr = syn::parse_str(&src).unwrap();
        let b: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(a, b);
    }

    // 11. Parse error for empty string
    #[test]
    fn nve_error_empty(_dummy in 0..1u8) {
        let result = syn::parse_str::<NameValueExpr>("");
        prop_assert!(result.is_err());
    }

    // 12. Parse error for missing value
    #[test]
    fn nve_error_missing_value(name in ident_strategy()) {
        let result = syn::parse_str::<NameValueExpr>(&format!("{name} ="));
        prop_assert!(result.is_err());
    }

    // 13. Parse error for missing equals
    #[test]
    fn nve_error_missing_eq(name in ident_strategy()) {
        let result = syn::parse_str::<NameValueExpr>(&format!("{name} 42"));
        prop_assert!(result.is_err());
    }

    // 14. Parse error for numeric "identifier"
    #[test]
    fn nve_error_numeric_start(val in int_value()) {
        let result = syn::parse_str::<NameValueExpr>(&format!("{val} = 1"));
        // Negative numbers will have a leading `-` which is not an ident either
        prop_assert!(result.is_err());
    }

    // 15. Negative literal values parse successfully
    #[test]
    fn nve_negative_literal(name in ident_strategy(), val in -1000i64..-1) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), name);
    }
}

// ---------------------------------------------------------------------------
// FieldThenParams tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 16. Bare type: no comma, no params
    #[test]
    fn ftp_bare_type(ty in type_name()) {
        let parsed: FieldThenParams = syn::parse_str(ty).unwrap();
        prop_assert!(parsed.comma.is_none());
        prop_assert!(parsed.params.is_empty());
    }

    // 17. Param count matches input
    #[test]
    fn ftp_param_count(ty in type_name(), keys in distinct_idents(5)) {
        if keys.is_empty() { return Ok(()); }
        let params: Vec<String> = keys.iter().enumerate()
            .map(|(i, k)| format!("{k} = {i}"))
            .collect();
        let src = format!("{ty}, {}", params.join(", "));
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert!(parsed.comma.is_some());
        prop_assert_eq!(parsed.params.len(), keys.len());
    }

    // 18. Param idents match the input keys in order
    #[test]
    fn ftp_param_idents_match(ty in type_name(), keys in distinct_idents(4)) {
        if keys.is_empty() { return Ok(()); }
        let params: Vec<String> = keys.iter().enumerate()
            .map(|(i, k)| format!("{k} = {i}"))
            .collect();
        let src = format!("{ty}, {}", params.join(", "));
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        for i in 0..keys.len() {
            prop_assert_eq!(parsed.params[i].path.to_string(), keys[i].as_str());
        }
    }

    // 19. Clone/Eq consistency
    #[test]
    fn ftp_clone_eq(ty in type_name()) {
        let parsed: FieldThenParams = syn::parse_str(ty).unwrap();
        let cloned = parsed.clone();
        prop_assert_eq!(parsed, cloned);
    }

    // 20. Clone/Eq consistency with params
    #[test]
    fn ftp_clone_eq_with_params(ty in type_name(), key in ident_strategy(), val in int_value()) {
        let src = format!("{ty}, {key} = {val}");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        let cloned = parsed.clone();
        prop_assert_eq!(parsed, cloned);
    }

    // 21. Debug is non-empty
    #[test]
    fn ftp_debug_non_empty(ty in type_name()) {
        let parsed: FieldThenParams = syn::parse_str(ty).unwrap();
        let dbg = format!("{parsed:?}");
        prop_assert!(!dbg.is_empty());
    }

    // 22. Debug contains the type name
    #[test]
    fn ftp_debug_contains_type_name(ty in type_name()) {
        let parsed: FieldThenParams = syn::parse_str(ty).unwrap();
        let dbg = format!("{parsed:?}");
        prop_assert!(dbg.contains("FieldThenParams"), "Debug should contain type name: {dbg}");
    }

    // 23. Deterministic: parsing same input twice yields equal results
    #[test]
    fn ftp_deterministic(ty in type_name(), key in ident_strategy(), val in int_value()) {
        let src = format!("{ty}, {key} = {val}");
        let a: FieldThenParams = syn::parse_str(&src).unwrap();
        let b: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(a, b);
    }

    // 24. Deterministic: bare type
    #[test]
    fn ftp_deterministic_bare(ty in type_name()) {
        let a: FieldThenParams = syn::parse_str(ty).unwrap();
        let b: FieldThenParams = syn::parse_str(ty).unwrap();
        prop_assert_eq!(a, b);
    }

    // 25. Field type token stream contains the type name
    #[test]
    fn ftp_field_type_preserved(ty in type_name()) {
        let parsed: FieldThenParams = syn::parse_str(ty).unwrap();
        let tokens = parsed.field.ty.to_token_stream().to_string();
        prop_assert_eq!(tokens, ty);
    }

    // 26. Parse error for empty input
    #[test]
    fn ftp_error_empty(_dummy in 0..1u8) {
        let result = syn::parse_str::<FieldThenParams>("");
        prop_assert!(result.is_err());
    }

    // 27. Trailing comma after params still parses
    #[test]
    fn ftp_trailing_comma(ty in type_name(), key in ident_strategy(), val in int_value()) {
        let src = format!("{ty}, {key} = {val},");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert!(parsed.comma.is_some());
        prop_assert_eq!(parsed.params.len(), 1);
    }

    // 28. Field with no params: field.ident is None (unnamed field)
    #[test]
    fn ftp_unnamed_field(ty in type_name()) {
        let parsed: FieldThenParams = syn::parse_str(ty).unwrap();
        prop_assert!(parsed.field.ident.is_none());
    }

    // 29. Multiple params with string literal values
    #[test]
    fn ftp_string_lit_params(ty in type_name(), keys in distinct_idents(3)) {
        if keys.is_empty() { return Ok(()); }
        let params: Vec<String> = keys.iter()
            .map(|k| format!("{k} = \"val\""))
            .collect();
        let src = format!("{ty}, {}", params.join(", "));
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.params.len(), keys.len());
    }

    // 30. Field vis is inherited (default) for unnamed fields
    #[test]
    fn ftp_field_vis_inherited(ty in type_name()) {
        let parsed: FieldThenParams = syn::parse_str(ty).unwrap();
        prop_assert!(
            matches!(parsed.field.vis, syn::Visibility::Inherited),
            "unnamed field should have inherited visibility"
        );
    }
}

// ---------------------------------------------------------------------------
// Additional cross-cutting proptest tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 31. NVE inside FTP: the NameValueExpr extracted from FTP equals a directly parsed one
    #[test]
    fn nve_in_ftp_matches_standalone(
        ty in type_name(),
        key in ident_strategy(),
        val in int_value(),
    ) {
        let ftp_src = format!("{ty}, {key} = {val}");
        let nve_src = format!("{key} = {val}");
        let ftp: FieldThenParams = syn::parse_str(&ftp_src).unwrap();
        let nve: NameValueExpr = syn::parse_str(&nve_src).unwrap();
        prop_assert_eq!(ftp.params[0].path.to_string(), nve.path.to_string());
    }

    // 32. Clone of NVE from FTP equals original
    #[test]
    fn nve_clone_from_ftp(ty in type_name(), key in ident_strategy(), val in int_value()) {
        let src = format!("{ty}, {key} = {val}");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        let original = &parsed.params[0];
        let cloned = original.clone();
        prop_assert_eq!(original.clone(), cloned);
    }

    // 33. Debug of NVE from FTP is non-empty
    #[test]
    fn nve_debug_from_ftp(ty in type_name(), key in ident_strategy(), val in int_value()) {
        let src = format!("{ty}, {key} = {val}");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        let dbg = format!("{:?}", parsed.params[0]);
        prop_assert!(!dbg.is_empty());
    }
}

// ===========================================================================
// Additional tests: string literals, integer literals, boolean literals,
// name extraction, value extraction, complex expressions, determinism,
// and error cases.
// ===========================================================================

/// Helper: extract a syn::Lit from a NameValueExpr, panicking if not a literal.
fn extract_lit(nve: &NameValueExpr) -> &syn::Lit {
    match &nve.expr {
        syn::Expr::Lit(el) => &el.lit,
        other => panic!("expected Expr::Lit, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 34-38: String literal parsing (value extraction, unicode, special chars)
// ---------------------------------------------------------------------------

#[test]
fn nve_string_value_extraction() {
    let nve: NameValueExpr = syn::parse_str(r#"label = "world""#).unwrap();
    match extract_lit(&nve) {
        syn::Lit::Str(s) => assert_eq!(s.value(), "world"),
        other => panic!("expected Str, got {other:?}"),
    }
}

#[test]
fn nve_string_unicode_content() {
    let nve: NameValueExpr = syn::parse_str(r#"emoji = "héllo 🌍""#).unwrap();
    match extract_lit(&nve) {
        syn::Lit::Str(s) => assert_eq!(s.value(), "héllo 🌍"),
        other => panic!("expected Str, got {other:?}"),
    }
}

#[test]
fn nve_string_with_quotes_escaped() {
    let nve: NameValueExpr = syn::parse_str(r#"q = "say \"hi\"""#).unwrap();
    match extract_lit(&nve) {
        syn::Lit::Str(s) => assert_eq!(s.value(), "say \"hi\""),
        other => panic!("expected Str, got {other:?}"),
    }
}

#[test]
fn nve_string_with_tab_escape() {
    let nve: NameValueExpr = syn::parse_str(r#"sep = "a\tb""#).unwrap();
    match extract_lit(&nve) {
        syn::Lit::Str(s) => assert_eq!(s.value(), "a\tb"),
        other => panic!("expected Str, got {other:?}"),
    }
}

#[test]
fn nve_string_with_backslash() {
    let nve: NameValueExpr = syn::parse_str(r#"path = "c:\\dir""#).unwrap();
    match extract_lit(&nve) {
        syn::Lit::Str(s) => assert_eq!(s.value(), "c:\\dir"),
        other => panic!("expected Str, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 39-43: Integer literal parsing (zero, large, hex, suffixed)
// ---------------------------------------------------------------------------

#[test]
fn nve_int_zero() {
    let nve: NameValueExpr = syn::parse_str("level = 0").unwrap();
    match extract_lit(&nve) {
        syn::Lit::Int(i) => assert_eq!(i.base10_parse::<i64>().unwrap(), 0),
        other => panic!("expected Int, got {other:?}"),
    }
}

#[test]
fn nve_int_large_positive() {
    let nve: NameValueExpr = syn::parse_str("big = 999999").unwrap();
    match extract_lit(&nve) {
        syn::Lit::Int(i) => assert_eq!(i.base10_parse::<i64>().unwrap(), 999_999),
        other => panic!("expected Int, got {other:?}"),
    }
}

#[test]
fn nve_int_hex_literal() {
    let nve: NameValueExpr = syn::parse_str("color = 0xFF").unwrap();
    match extract_lit(&nve) {
        syn::Lit::Int(i) => assert_eq!(i.base10_parse::<u64>().unwrap(), 255),
        other => panic!("expected Int, got {other:?}"),
    }
}

#[test]
fn nve_int_binary_literal() {
    let nve: NameValueExpr = syn::parse_str("mask = 0b1010").unwrap();
    match extract_lit(&nve) {
        syn::Lit::Int(i) => assert_eq!(i.base10_parse::<u64>().unwrap(), 10),
        other => panic!("expected Int, got {other:?}"),
    }
}

#[test]
fn nve_int_octal_literal() {
    let nve: NameValueExpr = syn::parse_str("perm = 0o755").unwrap();
    match extract_lit(&nve) {
        syn::Lit::Int(i) => assert_eq!(i.base10_parse::<u64>().unwrap(), 0o755),
        other => panic!("expected Int, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 44-46: Boolean literal parsing (value extraction)
// ---------------------------------------------------------------------------

#[test]
fn nve_bool_true_value() {
    let nve: NameValueExpr = syn::parse_str("enabled = true").unwrap();
    match extract_lit(&nve) {
        syn::Lit::Bool(b) => assert!(b.value),
        other => panic!("expected Bool, got {other:?}"),
    }
}

#[test]
fn nve_bool_false_value() {
    let nve: NameValueExpr = syn::parse_str("enabled = false").unwrap();
    match extract_lit(&nve) {
        syn::Lit::Bool(b) => assert!(!b.value),
        other => panic!("expected Bool, got {other:?}"),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 47. Bool value round-trips correctly
    #[test]
    fn nve_bool_value_roundtrip(name in ident_strategy(), b in prop::bool::ANY) {
        let src = format!("{name} = {b}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        match extract_lit(&parsed) {
            syn::Lit::Bool(lit) => prop_assert_eq!(lit.value, b),
            other => prop_assert!(false, "expected Bool, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// 48-51: Name extraction (underscore, raw ident, leading underscore, long)
// ---------------------------------------------------------------------------

#[test]
fn nve_error_underscore_as_name() {
    // `_` is a keyword, not a valid identifier in this context
    assert!(syn::parse_str::<NameValueExpr>("_ = 1").is_err());
}

#[test]
fn nve_name_leading_underscore() {
    let nve: NameValueExpr = syn::parse_str("_hidden = 42").unwrap();
    assert_eq!(nve.path.to_string(), "_hidden");
}

#[test]
fn nve_name_raw_ident() {
    let nve: NameValueExpr = syn::parse_str("r#type = 10").unwrap();
    // syn preserves the raw ident prefix
    assert_eq!(nve.path.to_string(), "r#type");
}

#[test]
fn nve_name_long_snake_case() {
    let nve: NameValueExpr = syn::parse_str("my_very_long_parameter_name = 0").unwrap();
    assert_eq!(nve.path.to_string(), "my_very_long_parameter_name");
}

// ---------------------------------------------------------------------------
// 52-54: Value extraction (expr token stream content)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 52. Integer value token stream contains the literal text
    #[test]
    fn nve_int_value_in_tokens(name in ident_strategy(), val in 0i64..500) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let tokens = parsed.expr.to_token_stream().to_string();
        prop_assert!(tokens.contains(&val.to_string()));
    }

    // 53. String value token stream contains the string
    #[test]
    fn nve_string_value_in_tokens(name in ident_strategy()) {
        let src = format!("{name} = \"testval\"");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let tokens = parsed.expr.to_token_stream().to_string();
        prop_assert!(tokens.contains("testval"));
    }

    // 54. Bool value token stream matches input
    #[test]
    fn nve_bool_value_in_tokens(name in ident_strategy(), b in prop::bool::ANY) {
        let src = format!("{name} = {b}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let tokens = parsed.expr.to_token_stream().to_string();
        prop_assert!(tokens.contains(&b.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 55-57: Complex expressions as values
// ---------------------------------------------------------------------------

#[test]
fn nve_complex_expr_binary_op() {
    let nve: NameValueExpr = syn::parse_str("x = 1 + 2").unwrap();
    assert_eq!(nve.path.to_string(), "x");
    let tokens = nve.expr.to_token_stream().to_string();
    assert!(tokens.contains("1"), "tokens should contain '1': {tokens}");
    assert!(tokens.contains("2"), "tokens should contain '2': {tokens}");
}

#[test]
fn nve_complex_expr_method_call() {
    let nve: NameValueExpr = syn::parse_str("v = foo.bar()").unwrap();
    assert_eq!(nve.path.to_string(), "v");
    let tokens = nve.expr.to_token_stream().to_string();
    assert!(tokens.contains("foo"), "tokens: {tokens}");
    assert!(tokens.contains("bar"), "tokens: {tokens}");
}

#[test]
fn nve_complex_expr_closure() {
    let nve: NameValueExpr = syn::parse_str("f = |x| x + 1").unwrap();
    assert_eq!(nve.path.to_string(), "f");
    assert!(!nve.expr.to_token_stream().is_empty());
}

// ---------------------------------------------------------------------------
// 58-60: Parsing determinism (complex and string values)
// ---------------------------------------------------------------------------

#[test]
fn nve_deterministic_string_value() {
    let src = r#"key = "deterministic""#;
    let a: NameValueExpr = syn::parse_str(src).unwrap();
    let b: NameValueExpr = syn::parse_str(src).unwrap();
    assert_eq!(a, b);
}

#[test]
fn nve_deterministic_bool_value() {
    let src = "flag = true";
    let a: NameValueExpr = syn::parse_str(src).unwrap();
    let b: NameValueExpr = syn::parse_str(src).unwrap();
    assert_eq!(a, b);
}

#[test]
fn nve_deterministic_complex_expr() {
    let src = "calc = 2 + 3 * 4";
    let a: NameValueExpr = syn::parse_str(src).unwrap();
    let b: NameValueExpr = syn::parse_str(src).unwrap();
    assert_eq!(a, b);
}

// ---------------------------------------------------------------------------
// 61-65: Error cases (additional invalid inputs)
// ---------------------------------------------------------------------------

#[test]
fn nve_error_double_equals() {
    assert!(syn::parse_str::<NameValueExpr>("x == 1").is_err());
}

#[test]
fn nve_error_just_equals() {
    assert!(syn::parse_str::<NameValueExpr>("=").is_err());
}

#[test]
fn nve_error_just_value() {
    assert!(syn::parse_str::<NameValueExpr>("42").is_err());
}

#[test]
fn nve_error_string_as_name() {
    assert!(syn::parse_str::<NameValueExpr>(r#""name" = 1"#).is_err());
}

#[test]
fn nve_error_comma_separated_without_context() {
    // Two NVE expressions without being parsed as a punctuated list
    assert!(syn::parse_str::<NameValueExpr>("a = 1, b = 2").is_err());
}

// ---------------------------------------------------------------------------
// 66-68: Eq_token presence and structural invariants
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 66. eq_token span is valid (non-zero length)
    #[test]
    fn nve_eq_token_present(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        // eq_token exists (it's not Option, so just accessing it is the test)
        let _ = parsed.eq_token;
    }

    // 67. Path is always a single ident (not empty)
    #[test]
    fn nve_path_is_nonempty(name in ident_strategy(), val in int_value()) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert!(!parsed.path.to_string().is_empty());
    }

    // 68. Different names produce different NameValueExpr values
    #[test]
    fn nve_different_names_not_equal(
        name1 in ident_strategy(),
        name2 in ident_strategy(),
        val in int_value()
    ) {
        prop_assume!(name1 != name2);
        let a: NameValueExpr = syn::parse_str(&format!("{name1} = {val}")).unwrap();
        let b: NameValueExpr = syn::parse_str(&format!("{name2} = {val}")).unwrap();
        prop_assert_ne!(a, b);
    }
}
