//! Property-based tests (v2) for adze-common type utilities.
//!
//! Covers: try_extract_inner_type, filter_inner_type, wrap_leaf_type,
//! NameValueExpr, and FieldThenParams with 50+ proptest cases.

use adze_common::{FieldThenParams, NameValueExpr};
use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers that avoid all keywords (including 2024 edition `gen`).
fn ident_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,9}".prop_filter("must not be a Rust keyword", |s| {
        !matches!(
            s.as_str(),
            "as" | "break"
                | "const"
                | "continue"
                | "crate"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "gen"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
                | "async"
                | "await"
                | "dyn"
                | "abstract"
                | "become"
                | "box"
                | "do"
                | "final"
                | "macro"
                | "override"
                | "priv"
                | "typeof"
                | "unsized"
                | "virtual"
                | "yield"
                | "try"
        )
    })
}

/// CamelCase identifiers for use as type names.
fn camel_ident() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{1,8}".prop_filter("must not be Self", |s| s != "Self")
}

/// Known primitive leaf types.
fn leaf_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Common generic containers.
fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec", "Box", "Arc", "Rc"][..])
}

/// Container<Leaf> string plus the container and leaf names.
fn wrapped_type_string() -> impl Strategy<Value = (String, String, String)> {
    (container_name(), leaf_type())
        .prop_map(|(c, l)| (c.to_string(), l.to_string(), format!("{c}<{l}>")))
}

/// Nested type strings with depth 0–3.
fn nested_type_string() -> impl Strategy<Value = String> {
    let d0 = leaf_type().prop_map(|s| s.to_string());
    let d1 = (container_name(), leaf_type()).prop_map(|(c, l)| format!("{c}<{l}>"));
    let d2 = (container_name(), container_name(), leaf_type())
        .prop_map(|(c1, c2, l)| format!("{c1}<{c2}<{l}>>"));
    let d3 = (
        container_name(),
        container_name(),
        container_name(),
        leaf_type(),
    )
        .prop_map(|(c1, c2, c3, l)| format!("{c1}<{c2}<{c3}<{l}>>>"));
    prop_oneof![d0, d1, d2, d3]
}

/// Subset of containers chosen from a fixed set.
fn skip_set_strategy() -> impl Strategy<Value = Vec<&'static str>> {
    proptest::sample::subsequence(&["Option", "Vec", "Box", "Arc", "Rc"][..], 0..=5)
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn to_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // ===== try_extract_inner_type =====

    // 1. Plain ident never matches Option extraction.
    #[test]
    fn plain_ident_no_option_extract(name in ident_name()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted);
    }

    // 2. Option<Leaf> always extracts for "Option".
    #[test]
    fn option_leaf_always_extracts(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    // 3. Vec<Leaf> always extracts for "Vec".
    #[test]
    fn vec_leaf_always_extracts(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    // 4. Extraction with wrong target returns false.
    #[test]
    fn extract_wrong_target_returns_false(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_inner, extracted) = try_extract_inner_type(&ty, "NoMatch", &skip);
        prop_assert!(!extracted);
    }

    // 5. Skip-over lets us see through Box to find Vec.
    #[test]
    fn skip_box_extract_vec(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Vec<{leaf}>>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    // 6. Skip-over with Arc wrapping Option.
    #[test]
    fn skip_arc_extract_option(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Arc<Option<{leaf}>>")).unwrap();
        let skip: HashSet<&str> = ["Arc"].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    // 7. Double skip: Box<Arc<Vec<T>>>.
    #[test]
    fn double_skip_extract(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<Vec<{leaf}>>>")).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    // 8. Skip-over type present but target absent → false.
    #[test]
    fn skip_present_no_target(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<{leaf}>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (_inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted);
    }

    // 9. Non-path type (reference) returns unchanged.
    #[test]
    fn ref_type_extract_unchanged(leaf in leaf_type()) {
        let ts = format!("&{leaf}");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(to_str(&inner), to_str(&ty));
    }

    // 10. Tuple type returns unchanged.
    #[test]
    fn tuple_type_extract_unchanged(a in leaf_type(), b in leaf_type()) {
        let ts = format!("({a}, {b})");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(to_str(&inner), to_str(&ty));
    }

    // 11. Extraction is idempotent on non-matching types.
    #[test]
    fn extract_idempotent_non_match(ts in nested_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (out1, ext1) = try_extract_inner_type(&ty, "NoMatch", &skip);
        let (out2, ext2) = try_extract_inner_type(&out1, "NoMatch", &skip);
        prop_assert!(!ext1);
        prop_assert!(!ext2);
        prop_assert_eq!(to_str(&out1), to_str(&out2));
    }

    // 12. Matching the container itself extracts inner.
    #[test]
    fn extract_matches_own_container(
        (container, leaf, ts) in wrapped_type_string(),
    ) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, &container, &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    // 13. CamelCase custom type never matches standard extraction.
    #[test]
    fn custom_type_no_extract(name in camel_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (_inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(!extracted);
    }

    // ===== filter_inner_type =====

    // 14. filter_inner_type on a leaf is identity.
    #[test]
    fn filter_leaf_identity(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    // 15. filter_inner_type removes Box.
    #[test]
    fn filter_removes_box(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<{leaf}>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    // 16. filter_inner_type removes nested Box<Arc<T>>.
    #[test]
    fn filter_removes_nested_box_arc(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<{leaf}>>")).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    // 17. filter_inner_type with empty skip set is identity.
    #[test]
    fn filter_empty_skip_identity(ts in nested_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), to_str(&ty));
    }

    // 18. filter_inner_type does not remove non-skip containers.
    #[test]
    fn filter_preserves_non_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), to_str(&ty));
    }

    // 19. filter on non-path (tuple) returns unchanged.
    #[test]
    fn filter_tuple_unchanged(a in leaf_type(), b in leaf_type()) {
        let ts = format!("({a}, {b})");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), to_str(&ty));
    }

    // 20. filter on non-path (reference) returns unchanged.
    #[test]
    fn filter_ref_unchanged(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("&{leaf}")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), to_str(&ty));
    }

    // 21. filter is idempotent.
    #[test]
    fn filter_idempotent(ts in nested_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let f1 = filter_inner_type(&ty, &skip);
        let f2 = filter_inner_type(&f1, &skip);
        prop_assert_eq!(to_str(&f1), to_str(&f2));
    }

    // 22. filter removes Arc.
    #[test]
    fn filter_removes_arc(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Arc<{leaf}>")).unwrap();
        let skip: HashSet<&str> = ["Arc"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    // 23. filter removes Rc.
    #[test]
    fn filter_removes_rc(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Rc<{leaf}>")).unwrap();
        let skip: HashSet<&str> = ["Rc"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    // 24. filter with random skip subsets is deterministic.
    #[test]
    fn filter_deterministic(ts in nested_type_string(), skips in skip_set_strategy()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = skips.into_iter().collect();
        let a = to_str(&filter_inner_type(&ty, &skip));
        let b = to_str(&filter_inner_type(&ty, &skip));
        prop_assert_eq!(a, b);
    }

    // ===== wrap_leaf_type =====

    // 25. Leaf type gets wrapped in adze::WithLeaf.
    #[test]
    fn wrap_leaf_wraps(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let expected = format!("adze :: WithLeaf < {leaf} >");
        prop_assert_eq!(to_str(&wrapped), expected);
    }

    // 26. CamelCase custom type gets wrapped.
    #[test]
    fn wrap_custom_type_wraps(name in camel_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        prop_assert!(to_str(&wrapped).contains("adze :: WithLeaf"));
    }

    // 27. Vec<T> with Vec in skip set → Vec<adze::WithLeaf<T>>.
    #[test]
    fn wrap_vec_skip_wraps_inner(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let expected = format!("Vec < adze :: WithLeaf < {leaf} > >");
        prop_assert_eq!(to_str(&wrapped), expected);
    }

    // 28. Option<T> with Option in skip set → Option<adze::WithLeaf<T>>.
    #[test]
    fn wrap_option_skip_wraps_inner(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let skip: HashSet<&str> = ["Option"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let expected = format!("Option < adze :: WithLeaf < {leaf} > >");
        prop_assert_eq!(to_str(&wrapped), expected);
    }

    // 29. Container not in skip set gets wrapped entirely.
    #[test]
    fn wrap_non_skip_container_wraps_whole(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        prop_assert!(to_str(&wrapped).starts_with("adze :: WithLeaf <"));
    }

    // 30. wrap_leaf_type is deterministic.
    #[test]
    fn wrap_deterministic(ts in nested_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
        let a = to_str(&wrap_leaf_type(&ty, &skip));
        let b = to_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(a, b);
    }

    // 31. Non-path type (reference) gets wrapped entirely.
    #[test]
    fn wrap_ref_type_wraps(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("&{leaf}")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        prop_assert!(to_str(&wrapped).contains("adze :: WithLeaf"));
    }

    // 32. Nested skip: Vec<Option<T>> with both in skip.
    #[test]
    fn wrap_nested_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<Option<{leaf}>>")).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let expected = format!("Vec < Option < adze :: WithLeaf < {leaf} > > >");
        prop_assert_eq!(to_str(&wrapped), expected);
    }

    // 33. Triple nesting: Box<Vec<Option<T>>> with all in skip.
    #[test]
    fn wrap_triple_nesting(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Vec<Option<{leaf}>>>")).unwrap();
        let skip: HashSet<&str> = ["Box", "Vec", "Option"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let expected = format!("Box < Vec < Option < adze :: WithLeaf < {leaf} > > > >");
        prop_assert_eq!(to_str(&wrapped), expected);
    }

    // 34. wrap with random skip subsets never panics on simple types.
    #[test]
    fn wrap_random_skip_no_panic(leaf in leaf_type(), skips in skip_set_strategy()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = skips.into_iter().collect();
        let _ = wrap_leaf_type(&ty, &skip);
    }

    // 35. wrap output always contains the leaf type name somewhere.
    #[test]
    fn wrap_output_contains_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = to_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains(leaf));
    }

    // ===== NameValueExpr =====

    // 36. NameValueExpr parse roundtrip with integer literal.
    #[test]
    fn nve_int_roundtrip(key in ident_name(), val in 0i64..10_000) {
        let src = format!("{key} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), key);
    }

    // 37. NameValueExpr parse roundtrip with string literal.
    #[test]
    fn nve_string_roundtrip(key in ident_name()) {
        let src = format!("{key} = \"hello\"");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), key);
    }

    // 38. NameValueExpr key preserved exactly.
    #[test]
    fn nve_key_preserved(key in ident_name()) {
        let src = format!("{key} = 42");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), key);
    }

    // 39. NameValueExpr equality: same source parses equal.
    #[test]
    fn nve_same_source_equal(key in ident_name(), val in 0i64..1000) {
        let src = format!("{key} = {val}");
        let a: NameValueExpr = syn::parse_str(&src).unwrap();
        let b: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(a.path.to_string(), b.path.to_string());
        prop_assert_eq!(
            a.expr.to_token_stream().to_string(),
            b.expr.to_token_stream().to_string()
        );
    }

    // 40. NameValueExpr with bool literal.
    #[test]
    fn nve_bool_value(key in ident_name(), val in prop::bool::ANY) {
        let src = format!("{key} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), key);
    }

    // 41. NameValueExpr with negative int.
    #[test]
    fn nve_negative_int(key in ident_name(), val in -10_000i64..-1) {
        let src = format!("{key} = ({val})");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), key);
    }

    // ===== FieldThenParams =====

    // 42. FieldThenParams with just a type, no comma/params.
    #[test]
    fn ftp_type_only(leaf in leaf_type()) {
        let parsed: FieldThenParams = syn::parse_str(leaf).unwrap();
        prop_assert!(parsed.comma.is_none());
        prop_assert!(parsed.params.is_empty());
    }

    // 43. FieldThenParams with one param.
    #[test]
    fn ftp_one_param(leaf in leaf_type(), key in ident_name(), val in 0i64..100) {
        let src = format!("{leaf}, {key} = {val}");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert!(parsed.comma.is_some());
        prop_assert_eq!(parsed.params.len(), 1);
        prop_assert_eq!(parsed.params[0].path.to_string(), key);
    }

    // 44. FieldThenParams with two params.
    #[test]
    fn ftp_two_params(
        leaf in leaf_type(),
        k1 in ident_name(),
        k2 in ident_name(),
        v1 in 0i64..100,
        v2 in 0i64..100,
    ) {
        let src = format!("{leaf}, {k1} = {v1}, {k2} = {v2}");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert!(parsed.comma.is_some());
        prop_assert_eq!(parsed.params.len(), 2);
        prop_assert_eq!(parsed.params[0].path.to_string(), k1);
        prop_assert_eq!(parsed.params[1].path.to_string(), k2);
    }

    // 45. FieldThenParams field type is preserved in token stream.
    #[test]
    fn ftp_field_type_preserved(leaf in leaf_type()) {
        let parsed: FieldThenParams = syn::parse_str(leaf).unwrap();
        let field_ty = to_str(&parsed.field.ty);
        prop_assert_eq!(field_ty, leaf);
    }

    // 46. FieldThenParams with generic field type.
    #[test]
    fn ftp_generic_field(container in container_name(), leaf in leaf_type()) {
        let src = format!("{container}<{leaf}>");
        let parsed: FieldThenParams = syn::parse_str(&src).unwrap();
        prop_assert!(parsed.params.is_empty());
    }

    // ===== Cross-function properties =====

    // 47. extract then filter: filtering the extraction result is still identity.
    #[test]
    fn extract_then_filter_identity(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, _) = try_extract_inner_type(&ty, "Option", &skip);
        let filtered = filter_inner_type(&inner, &skip);
        prop_assert_eq!(to_str(&inner), to_str(&filtered));
    }

    // 48. filter then wrap: wrapping a filtered leaf still wraps.
    #[test]
    fn filter_then_wrap(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<{leaf}>")).unwrap();
        let skip_filter: HashSet<&str> = ["Box"].into_iter().collect();
        let skip_wrap: HashSet<&str> = HashSet::new();
        let filtered = filter_inner_type(&ty, &skip_filter);
        let wrapped = wrap_leaf_type(&filtered, &skip_wrap);
        prop_assert_eq!(to_str(&wrapped), format!("adze :: WithLeaf < {leaf} >"));
    }

    // 49. extract from nested skip preserves extraction flag.
    #[test]
    fn nested_skip_preserves_flag(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Arc<Box<Vec<{leaf}>>>")).unwrap();
        let skip: HashSet<&str> = ["Arc", "Box"].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    // 50. wrap then token-stream parse roundtrip.
    #[test]
    fn wrap_roundtrip_parse(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let wrapped_str = to_str(&wrapped);
        let reparsed: Type = parse_str(&wrapped_str).unwrap();
        prop_assert_eq!(to_str(&reparsed), wrapped_str);
    }

    // 51. filter + extract composition: filter Box, then extract Vec.
    #[test]
    fn filter_extract_composition(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Vec<{leaf}>>")).unwrap();
        let skip_filter: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip_filter);
        let skip_extract: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&filtered, "Vec", &skip_extract);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    // 52. wrap is always parseable back.
    #[test]
    fn wrap_always_parseable(ts in nested_type_string(), skips in skip_set_strategy()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = skips.into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = to_str(&wrapped);
        let reparsed = parse_str::<Type>(&s);
        prop_assert!(reparsed.is_ok(), "Failed to reparse: {}", s);
    }

    // 53. Lowercase ident never crashes any function.
    #[test]
    fn ident_no_crash_all_fns(name in ident_name()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = ["Box", "Vec", "Option"].into_iter().collect();
        let _ = try_extract_inner_type(&ty, "Vec", &skip);
        let _ = filter_inner_type(&ty, &skip);
        let _ = wrap_leaf_type(&ty, &skip);
    }

    // 54. Extraction from same-name container always succeeds.
    #[test]
    fn extract_same_name_succeeds(container in container_name(), leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, container, &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    // 55. filter_inner_type on a single skip wrapper yields the leaf.
    #[test]
    fn filter_single_skip_yields_leaf(container in container_name(), leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip: HashSet<&str> = [container].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), leaf);
    }
}
