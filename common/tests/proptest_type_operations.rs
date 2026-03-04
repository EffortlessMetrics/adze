//! Property-based tests for type-operation functions in adze-common.
//!
//! Covers: try_extract_inner_type, filter_inner_type, wrap_leaf_type,
//! NameValueExpr, and FieldThenParams.

use adze_common::{FieldThenParams, NameValueExpr};
use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Random lowercase identifier: [a-z]{1,10}
fn ident_name() -> impl Strategy<Value = String> {
    "[a-z]{1,10}".prop_filter("must not be a Rust keyword", |s| {
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

/// Known leaf types
fn leaf_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Common container wrappers
fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec", "Box", "Arc", "Rc"][..])
}

/// Wrapper<Leaf> string
fn wrapped_type_string() -> impl Strategy<Value = (String, String)> {
    (container_name(), leaf_type()).prop_map(|(c, l)| (c.to_string(), format!("{c}<{l}>")))
}

/// Nested type string up to depth 3
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

// ---------------------------------------------------------------------------
// proptest! blocks
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 1. Simple type name → try_extract_inner_type("Option") returns None
    #[test]
    fn simple_type_never_extracts_option(name in ident_name()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted, "plain ident should never match Option<T>");
    }

    // 2. Option<X> → try_extract_inner_type("Option") returns Some
    #[test]
    fn option_wrapped_always_extracts(inner_name in leaf_type()) {
        let type_str = format!("Option<{inner_name}>");
        let ty: Type = parse_str(&type_str).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(inner.to_token_stream().to_string(), inner_name);
    }

    // 3. wrap_leaf_type is deterministic
    #[test]
    fn wrap_leaf_is_deterministic(ts in nested_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
        let a = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        let b = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        prop_assert_eq!(a, b);
    }

    // 4. Non-matching wrapper always returns (original, false)
    #[test]
    fn non_matching_wrapper_returns_none(name in ident_name()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_inner, extracted) = try_extract_inner_type(&ty, "NoSuchWrapper", &skip);
        prop_assert!(!extracted);
    }

    // 5. filter_inner_type on non-Vec types returns unchanged
    #[test]
    fn filter_non_vec_unchanged(name in leaf_type()) {
        let ty: Type = parse_str(name).unwrap();
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(
            filtered.to_token_stream().to_string(),
            ty.to_token_stream().to_string()
        );
    }

    // 6. NameValueExpr round-trip: parse → to_string → parse
    #[test]
    fn name_value_expr_roundtrip(key in ident_name(), val in 1i64..1000) {
        let src = format!("{key} = {val}");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        let reparsed_src = format!(
            "{} = {}",
            parsed.path.to_string(),
            parsed.expr.to_token_stream().to_string()
        );
        let reparsed: NameValueExpr = syn::parse_str(&reparsed_src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), reparsed.path.to_string());
        prop_assert_eq!(
            parsed.expr.to_token_stream().to_string(),
            reparsed.expr.to_token_stream().to_string()
        );
    }

    // 7. Random type names don't crash try_extract_inner_type
    #[test]
    fn random_ident_no_crash(name in ident_name()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let _ = try_extract_inner_type(&ty, "Option", &skip);
    }

    // 8. Nested Option<Option<T>> can extract both layers
    #[test]
    fn nested_option_double_extract(inner_name in leaf_type()) {
        let ts = format!("Option<Option<{inner_name}>>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (mid, ex1) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(ex1);
        let (leaf, ex2) = try_extract_inner_type(&mid, "Option", &skip);
        prop_assert!(ex2);
        prop_assert_eq!(leaf.to_token_stream().to_string(), inner_name);
    }

    // 9. Vec<T> extracts with "Vec" wrapper
    #[test]
    fn vec_always_extracts(inner_name in leaf_type()) {
        let ts = format!("Vec<{inner_name}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, ex) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(ex);
        prop_assert_eq!(inner.to_token_stream().to_string(), inner_name);
    }

    // 10. filter_inner_type with empty skip set is identity
    #[test]
    fn filter_empty_skip_is_identity(ts in nested_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(
            filtered.to_token_stream().to_string(),
            ty.to_token_stream().to_string()
        );
    }

    // 11. wrap_leaf_type with empty skip always wraps
    #[test]
    fn wrap_empty_skip_always_wraps(name in leaf_type()) {
        let ty: Type = parse_str(name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.contains("adze :: WithLeaf"));
    }

    // 12. try_extract then wrap_leaf: extraction and wrapping compose
    #[test]
    fn extract_then_wrap_compose(inner_name in leaf_type()) {
        let ts = format!("Option<{inner_name}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, _) = try_extract_inner_type(&ty, "Option", &skip);
        let wrapped = wrap_leaf_type(&inner, &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.contains("adze :: WithLeaf"));
        prop_assert!(s.contains(inner_name));
    }

    // 13. Box in skip_over lets us see through Box<Vec<T>>
    #[test]
    fn skip_over_box_extracts_vec(inner_name in leaf_type()) {
        let ts = format!("Box<Vec<{inner_name}>>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (inner, ex) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(ex);
        prop_assert_eq!(inner.to_token_stream().to_string(), inner_name);
    }

    // 14. filter_inner_type strips Box from Box<T>
    #[test]
    fn filter_strips_box(inner_name in leaf_type()) {
        let ts = format!("Box<{inner_name}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(filtered.to_token_stream().to_string(), inner_name);
    }

    // 15. filter_inner_type strips nested Box<Arc<T>>
    #[test]
    fn filter_strips_nested_containers(inner_name in leaf_type()) {
        let ts = format!("Box<Arc<{inner_name}>>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(filtered.to_token_stream().to_string(), inner_name);
    }

    // 16. wrap_leaf_type on Vec<T> wraps inner T but preserves Vec
    #[test]
    fn wrap_preserves_vec_wraps_inner(inner_name in leaf_type()) {
        let ts = format!("Vec<{inner_name}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.starts_with("Vec <"));
        prop_assert!(s.contains("adze :: WithLeaf"));
    }

    // 17. wrap_leaf_type on Option<Vec<T>> wraps only T
    #[test]
    fn wrap_nested_skip_wraps_leaf_only(inner_name in leaf_type()) {
        let ts = format!("Option<Vec<{inner_name}>>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = wrapped.to_token_stream().to_string();
        prop_assert!(s.starts_with("Option <"));
        prop_assert!(s.contains("Vec <"));
        prop_assert!(s.contains("adze :: WithLeaf"));
    }

    // 18. try_extract_inner_type is idempotent on non-matching
    #[test]
    fn extract_idempotent_on_non_match(name in leaf_type()) {
        let ty: Type = parse_str(name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (out1, ex1) = try_extract_inner_type(&ty, "Option", &skip);
        let (out2, ex2) = try_extract_inner_type(&out1, "Option", &skip);
        prop_assert!(!ex1);
        prop_assert!(!ex2);
        prop_assert_eq!(
            out1.to_token_stream().to_string(),
            out2.to_token_stream().to_string()
        );
    }

    // 19. NameValueExpr with string literal value
    #[test]
    fn name_value_expr_string_literal(key in ident_name()) {
        let src = format!("{key} = \"hello\"");
        let parsed: NameValueExpr = syn::parse_str(&src).unwrap();
        prop_assert_eq!(parsed.path.to_string(), key);
    }

    // 20. FieldThenParams with no params
    #[test]
    fn field_then_params_no_params(name in leaf_type()) {
        let parsed: FieldThenParams = syn::parse_str(name).unwrap();
        prop_assert!(parsed.comma.is_none());
        prop_assert!(parsed.params.is_empty());
    }

    // 21. wrap_leaf_type deterministic on nested types
    #[test]
    fn wrap_deterministic_nested(
        c in container_name(),
        l in leaf_type()
    ) {
        let ts = format!("{c}<{l}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = [c].into_iter().collect();
        let a = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        let b = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        prop_assert_eq!(a, b);
    }

    // 22. filter_inner_type is idempotent
    #[test]
    fn filter_is_idempotent(inner_name in leaf_type()) {
        let ts = format!("Box<{inner_name}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let f1 = filter_inner_type(&ty, &skip);
        let f2 = filter_inner_type(&f1, &skip);
        prop_assert_eq!(
            f1.to_token_stream().to_string(),
            f2.to_token_stream().to_string()
        );
    }

    // 23. Random ident never matches try_extract with wrapper "Vec"
    #[test]
    fn random_ident_never_vec(name in ident_name()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_, ex) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(!ex);
    }

    // 24. Option<T> where T is generated ident — extraction gives back T
    #[test]
    fn option_of_generated_ident(name in ident_name()) {
        let ts = format!("Option<{name}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, ex) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(ex);
        prop_assert_eq!(inner.to_token_stream().to_string(), name);
    }

    // 25. wrap_leaf_type output always contains original leaf name
    #[test]
    fn wrap_output_contains_leaf(name in leaf_type()) {
        let ty: Type = parse_str(name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip).to_token_stream().to_string();
        prop_assert!(wrapped.contains(name));
    }
}

// ---------------------------------------------------------------------------
// Regular #[test] functions
// ---------------------------------------------------------------------------

// 26. try_extract on reference type returns unchanged
#[test]
fn extract_reference_type_unchanged() {
    let ty: Type = parse_str("&str").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let (inner, ex) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!ex);
    assert_eq!(inner.to_token_stream().to_string(), "& str");
}

// 27. filter on tuple type returns unchanged
#[test]
fn filter_tuple_type_unchanged() {
    let ty: Type = parse_str("(i32, u32)").unwrap();
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "(i32 , u32)");
}

// 28. wrap on array type wraps entirely
#[test]
fn wrap_array_type() {
    let ty: Type = parse_str("[u8; 4]").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert!(
        wrapped
            .to_token_stream()
            .to_string()
            .contains("adze :: WithLeaf")
    );
}

// 29. NameValueExpr basic parse
#[test]
fn name_value_basic_parse() {
    let nv: NameValueExpr = parse_quote!(key = "value");
    assert_eq!(nv.path.to_string(), "key");
}

// 30. NameValueExpr integer value
#[test]
fn name_value_integer_parse() {
    let nv: NameValueExpr = parse_quote!(precedence = 5);
    assert_eq!(nv.path.to_string(), "precedence");
    assert_eq!(nv.expr.to_token_stream().to_string(), "5");
}

// 31. FieldThenParams with no trailing params
#[test]
fn field_then_params_bare() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

// 32. FieldThenParams with two params
#[test]
fn field_then_params_with_two_params() {
    let ftp: FieldThenParams = parse_quote!(String, name = "test", value = 42);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "name");
    assert_eq!(ftp.params[1].path.to_string(), "value");
}

// 33. FieldThenParams single param
#[test]
fn field_then_params_single_param() {
    let ftp: FieldThenParams = parse_quote!(u32, limit = 100);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
}

// 34. Double Box extraction via skip_over
#[test]
fn double_box_skip_extracts_option() {
    let ty: Type = parse_str("Box<Box<Option<i32>>>").unwrap();
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let (inner, ex) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ex);
    assert_eq!(inner.to_token_stream().to_string(), "i32");
}

// 35. filter strips triple nested containers
#[test]
fn filter_triple_nested() {
    let ty: Type = parse_str("Box<Arc<Rc<String>>>").unwrap();
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

// 36. wrap_leaf_type wraps Result generic args
#[test]
fn wrap_result_generic_args() {
    let ty: Type = parse_str("Result<String, i32>").unwrap();
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let wrapped = wrap_leaf_type(&ty, &skip);
    let s = wrapped.to_token_stream().to_string();
    assert!(s.contains("adze :: WithLeaf < String >"));
    assert!(s.contains("adze :: WithLeaf < i32 >"));
}

// 37. try_extract with Arc in skip finds Option inside Arc<Option<T>>
#[test]
fn skip_arc_extract_option() {
    let ty: Type = parse_str("Arc<Option<bool>>").unwrap();
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let (inner, ex) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ex);
    assert_eq!(inner.to_token_stream().to_string(), "bool");
}

// 38. NameValueExpr with boolean expression
#[test]
fn name_value_bool_expr() {
    let nv: NameValueExpr = parse_quote!(flag = true);
    assert_eq!(nv.path.to_string(), "flag");
}

// 39. wrap_leaf_type on bare String
#[test]
fn wrap_bare_string() {
    let ty: Type = parse_str("String").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < String >"
    );
}

// 40. extract from Option<Vec<u8>> gets Vec<u8>
#[test]
fn extract_option_of_vec() {
    let ty: Type = parse_str("Option<Vec<u8>>").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let (inner, ex) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ex);
    assert_eq!(inner.to_token_stream().to_string(), "Vec < u8 >");
}

// 41. filter_inner_type on plain type with matching skip does strip it
#[test]
fn filter_strips_single_layer() {
    let ty: Type = parse_str("Arc<f64>").unwrap();
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "f64");
}

// 42. wrap then token-round-trip
#[test]
fn wrap_token_roundtrip() {
    let ty: Type = parse_str("i32").unwrap();
    let skip: HashSet<&str> = HashSet::new();
    let wrapped = wrap_leaf_type(&ty, &skip);
    let tokens = wrapped.to_token_stream().to_string();
    let reparsed: Type = parse_str(&tokens).unwrap();
    assert_eq!(
        reparsed.to_token_stream().to_string(),
        wrapped.to_token_stream().to_string()
    );
}

// 43. try_extract with skip that doesn't contain the outer returns false
#[test]
fn skip_not_containing_outer_returns_false() {
    let ty: Type = parse_str("Rc<Option<u8>>").unwrap();
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let (_inner, ex) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!ex);
}

// 44. NameValueExpr path expression value
#[test]
fn name_value_path_expr() {
    let nv: NameValueExpr = parse_quote!(module = foo);
    assert_eq!(nv.path.to_string(), "module");
    assert_eq!(nv.expr.to_token_stream().to_string(), "foo");
}

// 45. FieldThenParams complex field type
#[test]
fn field_then_params_complex_field() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>, sep = ",");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "sep");
}
