//! Property-based tests for adze-common expansion functions (v3).
//!
//! 50+ tests covering try_extract_inner_type, filter_inner_type, wrap_leaf_type,
//! and is_parameterized (local helper) across 8 categories:
//!   1. Extract roundtrip proptest
//!   2. Filter consistency proptest
//!   3. Parameterized detection proptest
//!   4. Wrap preserves structure proptest
//!   5. Multiple wrapper proptest
//!   6. Regular extraction tests
//!   7. Regular wrap tests
//!   8. Edge cases

use adze_common::*;
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{PathArguments, Type, parse_quote, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn type_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("String".to_string()),
        Just("i32".to_string()),
        Just("u64".to_string()),
        Just("bool".to_string()),
        Just("f64".to_string()),
    ]
}

fn wrapper_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("Option".to_string()),
        Just("Vec".to_string()),
        Just("Box".to_string()),
        Just("Arc".to_string()),
        Just("Rc".to_string()),
    ]
}

/// Helper: stringify a type via token stream.
fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

/// Local helper: check whether a type has angle-bracket generic parameters.
fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty {
        p.path
            .segments
            .last()
            .is_some_and(|seg| matches!(seg.arguments, PathArguments::AngleBracketed(_)))
    } else {
        false
    }
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

fn box_skip() -> HashSet<&'static str> {
    ["Box"].into_iter().collect()
}

fn all_skip() -> HashSet<&'static str> {
    ["Option", "Vec", "Box", "Arc", "Rc"].into_iter().collect()
}

// ===========================================================================
// 1. Extract roundtrip proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn extract_roundtrip_direct_match(
        wrapper in wrapper_name_strategy(),
        inner in type_name_strategy(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let (extracted_ty, ok) = try_extract_inner_type(&ty, &wrapper, &empty_skip());
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&extracted_ty), inner);
    }

    #[test]
    fn extract_roundtrip_through_box(inner in type_name_strategy()) {
        let src = format!("Box<Vec<{inner}>>");
        let ty: Type = parse_str(&src).unwrap();
        let (extracted_ty, ok) = try_extract_inner_type(&ty, "Vec", &box_skip());
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&extracted_ty), inner);
    }

    #[test]
    fn extract_roundtrip_mismatch_returns_original(
        inner in type_name_strategy(),
    ) {
        let src = format!("Vec<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let (returned, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
        prop_assert!(!ok);
        prop_assert_eq!(ty_str(&returned), ty_str(&ty));
    }

    #[test]
    fn extract_roundtrip_plain_type_never_extracts(name in type_name_strategy()) {
        let ty: Type = parse_str(&name).unwrap();
        let (returned, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        prop_assert!(!ok);
        prop_assert_eq!(ty_str(&returned), name);
    }

    #[test]
    fn extract_roundtrip_deterministic(
        wrapper in wrapper_name_strategy(),
        inner in type_name_strategy(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let (a, a_ok) = try_extract_inner_type(&ty, &wrapper, &empty_skip());
        let (b, b_ok) = try_extract_inner_type(&ty, &wrapper, &empty_skip());
        prop_assert_eq!(a_ok, b_ok);
        prop_assert_eq!(ty_str(&a), ty_str(&b));
    }
}

// ===========================================================================
// 2. Filter consistency proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn filter_extract_agree_on_box(inner in type_name_strategy()) {
        let src = format!("Box<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let skip = box_skip();
        let filtered = filter_inner_type(&ty, &skip);
        // filter strips Box, so filtered should be inner
        prop_assert_eq!(ty_str(&filtered), inner.as_str());
        // extract with inner_of="Box" and empty skip finds Box and returns inner
        let (extracted, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&extracted), inner.as_str());
    }

    #[test]
    fn filter_idempotent(
        wrapper in wrapper_name_strategy(),
        inner in type_name_strategy(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let skip = all_skip();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    #[test]
    fn filter_empty_skip_is_identity(
        wrapper in wrapper_name_strategy(),
        inner in type_name_strategy(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let filtered = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    #[test]
    fn filter_plain_type_untouched(name in type_name_strategy()) {
        let ty: Type = parse_str(&name).unwrap();
        let filtered = filter_inner_type(&ty, &all_skip());
        prop_assert_eq!(ty_str(&filtered), name);
    }

    #[test]
    fn filter_double_strip(inner in type_name_strategy()) {
        let src = format!("Box<Arc<{inner}>>");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), inner);
    }
}

// ===========================================================================
// 3. Parameterized detection proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn parameterized_wrapped_type(
        wrapper in wrapper_name_strategy(),
        inner in type_name_strategy(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        prop_assert!(is_parameterized(&ty));
    }

    #[test]
    fn not_parameterized_plain_type(name in type_name_strategy()) {
        let ty: Type = parse_str(&name).unwrap();
        prop_assert!(!is_parameterized(&ty));
    }

    #[test]
    fn parameterized_double_wrapped(
        w1 in wrapper_name_strategy(),
        w2 in wrapper_name_strategy(),
        inner in type_name_strategy(),
    ) {
        let src = format!("{w1}<{w2}<{inner}>>");
        let ty: Type = parse_str(&src).unwrap();
        prop_assert!(is_parameterized(&ty));
    }

    #[test]
    fn parameterized_after_wrap_leaf(name in type_name_strategy()) {
        let ty: Type = parse_str(&name).unwrap();
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        // wrap_leaf_type produces adze::WithLeaf<T> which is parameterized
        prop_assert!(is_parameterized(&wrapped));
    }

    #[test]
    fn not_parameterized_after_full_extract(
        wrapper in wrapper_name_strategy(),
        inner in type_name_strategy(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let (extracted, ok) = try_extract_inner_type(&ty, &wrapper, &empty_skip());
        prop_assert!(ok);
        // extracted is the plain inner type
        prop_assert!(!is_parameterized(&extracted));
    }
}

// ===========================================================================
// 4. Wrap preserves structure proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn wrap_then_extract_recovers_name(name in type_name_strategy()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        // wrapped = adze::WithLeaf<T>, extract "WithLeaf" should recover T
        let ws = ty_str(&wrapped);
        prop_assert!(ws.contains(&name), "wrapped should contain original: {ws}");
    }

    #[test]
    fn wrap_skip_container_preserves_outer(
        wrapper in wrapper_name_strategy(),
        inner in type_name_strategy(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = [wrapper.as_str()].into_iter().collect();
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains(&wrapper), "outer preserved: {wrapped}");
        prop_assert!(wrapped.contains("WithLeaf"), "inner wrapped: {wrapped}");
    }

    #[test]
    fn wrap_produces_parseable_type(
        wrapper in wrapper_name_strategy(),
        inner in type_name_strategy(),
    ) {
        let src = format!("{wrapper}<{inner}>");
        let ty: Type = parse_str(&src).unwrap();
        let wrapped = wrap_leaf_type(&ty, &all_skip());
        let s = ty_str(&wrapped);
        let _reparsed: Type = parse_str(&s).unwrap();
    }

    #[test]
    fn wrap_deterministic(name in type_name_strategy()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let a = ty_str(&wrap_leaf_type(&ty, &skip));
        let b = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(a, b);
    }

    #[test]
    fn wrap_double_nests_with_leaf(name in type_name_strategy()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let once = wrap_leaf_type(&ty, &skip);
        let twice = wrap_leaf_type(&once, &skip);
        let s = ty_str(&twice);
        let count = s.matches("WithLeaf").count();
        prop_assert!(count >= 2, "double wrap needs >=2 WithLeaf: {s}");
    }
}

// ===========================================================================
// 5. Multiple wrapper proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn nested_option_vec_extract_option(inner in type_name_strategy()) {
        let src = format!("Option<Vec<{inner}>>");
        let ty: Type = parse_str(&src).unwrap();
        let (extracted, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
        prop_assert!(ok);
        let expected = format!("Vec < {inner} >");
        prop_assert_eq!(ty_str(&extracted), expected);
    }

    #[test]
    fn nested_option_vec_extract_vec_through_option(inner in type_name_strategy()) {
        let src = format!("Option<Vec<{inner}>>");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = ["Option"].into_iter().collect();
        let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&extracted), inner);
    }

    #[test]
    fn nested_option_vec_box_extract_deepest(inner in type_name_strategy()) {
        let src = format!("Option<Vec<Box<{inner}>>>");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
        let (extracted, ok) = try_extract_inner_type(&ty, "Box", &skip);
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&extracted), inner);
    }

    #[test]
    fn nested_filter_strips_all_containers(inner in type_name_strategy()) {
        let src = format!("Box<Arc<Rc<{inner}>>>");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), inner);
    }

    #[test]
    fn nested_wrap_wraps_innermost_only(inner in type_name_strategy()) {
        let src = format!("Vec<Option<{inner}>>");
        let ty: Type = parse_str(&src).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains("Vec"), "outer Vec present: {wrapped}");
        prop_assert!(wrapped.contains("Option"), "middle Option present: {wrapped}");
        prop_assert!(wrapped.contains("WithLeaf"), "innermost wrapped: {wrapped}");
    }
}

// ===========================================================================
// 6. Regular extraction tests (10 tests)
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_box_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_wrong_wrapper_fails() {
    let ty: Type = parse_quote!(Vec<String>);
    let (returned, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), ty_str(&ty));
}

#[test]
fn extract_plain_type_fails() {
    let ty: Type = parse_quote!(String);
    let (returned, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), "String");
}

#[test]
fn extract_through_box_to_vec() {
    let ty: Type = parse_quote!(Box<Vec<u64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &box_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn extract_through_arc_box_to_option() {
    let ty: Type = parse_quote!(Arc<Box<Option<f64>>>);
    let skip: HashSet<&str> = ["Arc", "Box"].into_iter().collect();
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_skip_no_target_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let (returned, ok) = try_extract_inner_type(&ty, "Vec", &box_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), ty_str(&ty));
}

#[test]
fn extract_reference_type_fails() {
    let ty: Type = parse_quote!(&str);
    let (returned, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), "& str");
}

#[test]
fn extract_tuple_type_fails() {
    let ty: Type = parse_quote!((i32, u32));
    let (returned, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), "(i32 , u32)");
}

// ===========================================================================
// 7. Regular wrap tests (8 tests)
// ===========================================================================

#[test]
fn wrap_plain_string() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_plain_i32() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_vec_with_skip() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_with_skip() {
    let ty: Type = parse_quote!(Option<bool>);
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < bool > >");
}

#[test]
fn wrap_nested_vec_option_with_skip() {
    let ty: Type = parse_quote!(Vec<Option<u64>>);
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < u64 > > >"
    );
}

#[test]
fn wrap_without_skip_wraps_container_directly() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < String > >");
}

#[test]
fn wrap_result_with_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

// ===========================================================================
// 8. Edge cases (7 tests)
// ===========================================================================

#[test]
fn edge_filter_non_path_type_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &all_skip());
    assert_eq!(ty_str(&filtered), "(i32 , u32)");
}

#[test]
fn edge_filter_empty_skip_is_identity() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &empty_skip());
    assert_eq!(ty_str(&filtered), "Box < String >");
}

#[test]
fn edge_qualified_path_not_extracted() {
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    // Only the last segment is checked, so "Vec" matches
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn edge_filter_deeply_nested() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<String>>>>);
    let skip: HashSet<&str> = ["Box", "Arc", "Rc", "Option"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn edge_wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn edge_is_parameterized_plain_false() {
    let ty: Type = parse_quote!(MyCustomType);
    assert!(!is_parameterized(&ty));
}

#[test]
fn edge_is_parameterized_generic_true() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert!(is_parameterized(&ty));
}
