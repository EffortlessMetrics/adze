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
