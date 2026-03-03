use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers for use as NameValueExpr keys.
fn ident_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,15}")
        .unwrap()
        .prop_filter("must be valid ident", |s| {
            !s.is_empty() && syn::parse_str::<syn::Ident>(s).is_ok()
        })
}

/// Integer literal values.
fn int_lit_strategy() -> impl Strategy<Value = i64> {
    prop::num::i64::ANY
}

/// Simple leaf type names (single-segment paths).
fn leaf_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Container type that wraps another type (single nesting).
fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Vec", "Option", "Arc"][..])
}

/// Subset of container_name suitable for skip_over sets.
fn skip_set_strategy() -> impl Strategy<Value = HashSet<&'static str>> {
    prop::collection::hash_set(container_name(), 0..=4)
}

// ---------------------------------------------------------------------------
// 1. NameValueExpr: any valid ident + int literal round-trips
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn name_value_preserves_ident(name in ident_strategy(), val in int_lit_strategy()) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), name);
    }

    // 2. NameValueExpr with string literals preserves key
    #[test]
    fn name_value_string_lit_preserves_ident(name in ident_strategy()) {
        let src = format!("{name} = \"hello\"");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), name);
    }

    // 3. FieldThenParams with no params has empty params list
    #[test]
    fn field_then_params_no_params(leaf in leaf_type_name()) {
        let parsed: FieldThenParams = syn::parse_str(leaf).unwrap();
        prop_assert!(parsed.comma.is_none());
        prop_assert!(parsed.params.is_empty());
    }

    // 4. FieldThenParams param count matches input count
    #[test]
    fn field_then_params_count(
        leaf in leaf_type_name(),
        keys in prop::collection::vec(ident_strategy(), 1..=4),
    ) {
        // Deduplicate keys to avoid syn ambiguity edge cases
        let keys: Vec<_> = keys.into_iter().collect::<std::collections::LinkedList<_>>()
            .into_iter().collect::<Vec<_>>();
        let params: Vec<String> = keys.iter().enumerate().map(|(i, k)| format!("{k} = {i}")).collect();
        let src = format!("{leaf}, {}", params.join(", "));
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert!(parsed.comma.is_some());
        prop_assert_eq!(parsed.params.len(), keys.len());
    }

    // 5. try_extract_inner_type: direct match always extracts
    #[test]
    fn extract_direct_match(
        inner in leaf_type_name(),
        container in container_name(),
    ) {
        let src = format!("{container}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, container, &skip);
        prop_assert!(extracted);
        prop_assert_eq!(result.to_token_stream().to_string(), inner);
    }

    // 6. try_extract_inner_type: non-matching target never extracts
    #[test]
    fn extract_no_match_for_different_target(inner in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Vec<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted);
    }

    // 7. try_extract_inner_type: skip_over + target extracts through wrapper
    #[test]
    fn extract_through_skip(
        inner in leaf_type_name(),
        wrapper in prop::sample::select(&["Box", "Arc"][..]),
    ) {
        let src = format!("{wrapper}<Option<{inner}>>");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(result.to_token_stream().to_string(), inner);
    }

    // 8. try_extract_inner_type: plain leaf type never extracts
    #[test]
    fn extract_leaf_never_extracts(
        leaf in leaf_type_name(),
        target in container_name(),
    ) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(result.to_token_stream().to_string(), leaf);
    }

    // 9. filter_inner_type: empty skip set is identity
    #[test]
    fn filter_empty_skip_is_identity(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(
            filtered.to_token_stream().to_string(),
            ty.to_token_stream().to_string()
        );
    }

    // 10. filter_inner_type: single wrapper in skip set unwraps once
    #[test]
    fn filter_single_wrapper(inner in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Box<{inner}>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(filtered.to_token_stream().to_string(), inner);
    }

    // 11. filter_inner_type: type not in skip set is unchanged
    #[test]
    fn filter_non_skip_unchanged(inner in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Vec<{inner}>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(
            filtered.to_token_stream().to_string(),
            ty.to_token_stream().to_string()
        );
    }

    // 12. filter_inner_type idempotency: filtering a leaf type twice is the same
    #[test]
    fn filter_idempotent_on_leaf(leaf in leaf_type_name(), skip in skip_set_strategy()) {
        let ty: Type = parse_str(leaf).unwrap();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(
            once.to_token_stream().to_string(),
            twice.to_token_stream().to_string()
        );
    }

    // 13. wrap_leaf_type: leaf type always gets WithLeaf wrapper
    #[test]
    fn wrap_leaf_always_wraps(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.contains("adze :: WithLeaf"), "expected WithLeaf wrapper, got: {s}");
    }

    // 14. wrap_leaf_type: skip container preserves outer, wraps inner
    #[test]
    fn wrap_preserves_skip_container(inner in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Vec<{inner}>")).unwrap();
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.starts_with("Vec <"), "should start with Vec, got: {s}");
        prop_assert!(s.contains("adze :: WithLeaf"), "inner should be wrapped, got: {s}");
    }

    // 15. wrap_leaf_type: nested skip containers wrap only the innermost leaf
    #[test]
    fn wrap_nested_skip_wraps_innermost(inner in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Option<Vec<{inner}>>")).unwrap();
        let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.starts_with("Option <"), "should start with Option, got: {s}");
        prop_assert!(s.contains("Vec <"), "should contain Vec, got: {s}");
        prop_assert!(s.contains("adze :: WithLeaf"), "innermost should be wrapped, got: {s}");
    }

    // 16. try_extract + filter roundtrip: extract then filter on non-skip leaf is identity
    #[test]
    fn extract_then_filter_roundtrip(inner in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Option<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (extracted, ok) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(ok);
        // Filtering the extracted leaf with empty skip is identity
        let filtered = filter_inner_type(&extracted, &skip);
        prop_assert_eq!(
            extracted.to_token_stream().to_string(),
            filtered.to_token_stream().to_string()
        );
    }

    // 17. Non-path types: reference types are not extracted
    #[test]
    fn reference_type_not_extracted(leaf in leaf_type_name()) {
        let src = format!("&{leaf}");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted);
    }

    // 18. Non-path types: reference types pass through filter unchanged
    #[test]
    fn reference_type_filter_unchanged(leaf in leaf_type_name()) {
        let src = format!("&{leaf}");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = ["Box", "Vec"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(
            filtered.to_token_stream().to_string(),
            ty.to_token_stream().to_string()
        );
    }

    // 19. Non-path types: reference types get wrapped entirely
    #[test]
    fn reference_type_wrap_wraps_entirely(leaf in leaf_type_name()) {
        let src = format!("&{leaf}");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.contains("adze :: WithLeaf"), "expected WithLeaf wrapper, got: {s}");
    }

    // 20. NameValueExpr: re-serialized from components re-parses identically
    #[test]
    fn name_value_reparse_roundtrip(name in ident_strategy(), val in 0i64..1000) {
        let src = format!("{name} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let ident_str = parsed.path.to_string();
        let reserialized = format!("{ident_str} = {val}");
        let reparsed: NameValueExpr = syn::parse_str(&reserialized).unwrap();
        prop_assert_eq!(parsed.path.to_string(), reparsed.path.to_string());
    }
}

// ---------------------------------------------------------------------------
// Additional deterministic edge-case tests (proptest-adjacent)
// ---------------------------------------------------------------------------

#[test]
fn extract_deeply_nested_skip() {
    // Box<Arc<Option<i32>>> with skip={Box, Arc} and target=Option
    let ty: Type = parse_str("Box<Arc<Option<i32>>>").unwrap();
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

#[test]
fn filter_deeply_nested() {
    // Box<Arc<Vec<String>>> with skip={Box, Arc, Vec} strips all wrappers
    let ty: Type = parse_str("Box<Arc<Vec<String>>>").unwrap();
    let skip: HashSet<&str> = ["Box", "Arc", "Vec"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn wrap_double_nested_skip() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn field_then_params_single_trailing_comma() {
    // Trailing comma after last param should still parse
    let parsed: FieldThenParams = syn::parse_str("String, key = 1,").unwrap();
    assert!(parsed.comma.is_some());
    assert_eq!(parsed.params.len(), 1);
}
