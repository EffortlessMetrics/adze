#![allow(clippy::needless_range_loop)]

//! Property-based and deterministic tests for container type handling in adze-common.
//!
//! Covers recognition, unwrapping, construction, nesting, and determinism of
//! `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote, parse_str};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip_box() -> HashSet<&'static str> {
    HashSet::from(["Box"])
}

fn skip_box_arc() -> HashSet<&'static str> {
    HashSet::from(["Box", "Arc"])
}

fn skip_option_vec() -> HashSet<&'static str> {
    HashSet::from(["Option", "Vec"])
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn leaf_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec", "Box"][..])
}

// ===========================================================================
// 1. Option<T> is recognized as container
// ===========================================================================

#[test]
fn option_extract_recognizes_option() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn option_extract_with_numeric_inner() {
    let ty: Type = parse_quote!(Option<u32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn option_filter_unwraps_when_in_skip_set() {
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let ty: Type = parse_quote!(Option<i64>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&filtered), "i64");
}

#[test]
fn option_wrap_preserves_container() {
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let ty: Type = parse_quote!(Option<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < bool > >");
}

// ===========================================================================
// 2. Vec<T> is recognized as container
// ===========================================================================

#[test]
fn vec_extract_recognizes_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn vec_extract_with_bool_inner() {
    let ty: Type = parse_quote!(Vec<bool>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn vec_filter_unwraps_when_in_skip_set() {
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let ty: Type = parse_quote!(Vec<char>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&filtered), "char");
}

#[test]
fn vec_wrap_preserves_container() {
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let ty: Type = parse_quote!(Vec<f64>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < f64 > >");
}

// ===========================================================================
// 3. Box<T> is recognized as container
// ===========================================================================

#[test]
fn box_extract_recognizes_box_via_skip() {
    // Box is in skip_over, so extract should skip through it to find Vec inside.
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_box());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn box_filter_unwraps_single_layer() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip_box());
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn box_wrap_leaf_inside_box() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Box<usize>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "Box < adze :: WithLeaf < usize > >");
}

// ===========================================================================
// 4. Non-container types correctly identified
// ===========================================================================

#[test]
fn plain_type_not_extracted() {
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn reference_type_not_extracted() {
    let ty: Type = parse_quote!(&str);
    let (_, extracted) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(!extracted);
}

#[test]
fn tuple_type_not_extracted() {
    let ty: Type = parse_quote!((i32, u32));
    let (_, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(!extracted);
}

#[test]
fn non_container_filter_returns_unchanged() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let filtered = filter_inner_type(&ty, &skip_box());
    assert_eq!(ty_str(&filtered), "HashMap < String , i32 >");
}

#[test]
fn non_container_wrap_wraps_entirely() {
    let ty: Type = parse_quote!(MyCustomType);
    let wrapped = wrap_leaf_type(&ty, &skip_option_vec());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < MyCustomType >");
}

proptest! {
    #[test]
    fn proptest_leaf_types_never_extracted(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let (_, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
        prop_assert!(!extracted);
        let (_, extracted) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
        prop_assert!(!extracted);
    }
}

// ===========================================================================
// 5. Nested container detection
// ===========================================================================

#[test]
fn nested_option_vec_extracts_vec() {
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn nested_box_arc_vec_extracts_through_two_skips() {
    let ty: Type = parse_quote!(Box<Arc<Vec<i32>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_box_arc());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn nested_box_box_unwraps_all_layers() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Box<Box<f32>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&filtered), "f32");
}

#[test]
fn nested_skip_no_match_returns_original() {
    // Box<String> — skip through Box, but String is not Option, so not extracted.
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_box());
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < String >");
}

proptest! {
    #[test]
    fn proptest_nested_extract_finds_target(
        wrapper in prop::sample::select(&["Option", "Vec"][..]),
        leaf in leaf_type(),
    ) {
        // Outer is Box (in skip set), inner is Option or Vec (the target).
        let type_str = format!("Box<{wrapper}<{leaf}>>");
        let ty: Type = parse_str(&type_str).unwrap();
        let (inner, extracted) = try_extract_inner_type(&ty, wrapper, &skip_box());
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }
}

// ===========================================================================
// 6. Container unwrapping (filter_inner_type)
// ===========================================================================

#[test]
fn filter_single_box_unwrap() {
    let ty: Type = parse_quote!(Box<u64>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip_box())), "u64");
}

#[test]
fn filter_triple_nesting() {
    let skip: HashSet<&str> = HashSet::from(["Box", "Arc", "Rc"]);
    let ty: Type = parse_quote!(Box<Arc<Rc<bool>>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "bool");
}

#[test]
fn filter_stops_at_non_skip_container() {
    // Vec is NOT in skip set, so filtering stops at Vec.
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip_box());
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
}

proptest! {
    #[test]
    fn proptest_filter_removes_single_skip_layer(
        wrapper in prop::sample::select(&["Box", "Arc"][..]),
        leaf in leaf_type(),
    ) {
        let type_str = format!("{wrapper}<{leaf}>");
        let ty: Type = parse_str(&type_str).unwrap();
        let filtered = filter_inner_type(&ty, &skip_box_arc());
        prop_assert_eq!(ty_str(&filtered), leaf);
    }
}

// ===========================================================================
// 7. Container type construction (wrap_leaf_type)
// ===========================================================================

#[test]
fn wrap_plain_leaf_gets_with_leaf() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip_option_vec());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_nested_containers_wraps_inner_leaf() {
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_non_skip_container_wraps_whole() {
    // Box is NOT in skip set, so the entire Box<T> is wrapped.
    let ty: Type = parse_quote!(Box<u8>);
    let wrapped = wrap_leaf_type(&ty, &skip_option_vec());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Box < u8 > >");
}

proptest! {
    #[test]
    fn proptest_wrap_always_produces_with_leaf_for_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip_option_vec());
        let s = ty_str(&wrapped);
        prop_assert!(s.contains("WithLeaf"), "Expected WithLeaf wrapper, got: {}", s);
    }

    #[test]
    fn proptest_wrap_skip_container_keeps_outer(
        container in prop::sample::select(&["Option", "Vec"][..]),
        leaf in leaf_type(),
    ) {
        let type_str = format!("{container}<{leaf}>");
        let ty: Type = parse_str(&type_str).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip_option_vec());
        let s = ty_str(&wrapped);
        prop_assert!(s.starts_with(container), "Expected outer container {}, got: {}", container, s);
        prop_assert!(s.contains("WithLeaf"), "Expected inner WithLeaf, got: {}", s);
    }
}

// ===========================================================================
// 8. Container handling determinism
// ===========================================================================

#[test]
fn determinism_extract_same_result_on_repeated_calls() {
    let ty: Type = parse_quote!(Box<Option<Vec<String>>>);
    let skip: HashSet<&str> = HashSet::from(["Box", "Option"]);
    let results: Vec<_> = (0..10)
        .map(|_| {
            let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
            (ty_str(&inner), extracted)
        })
        .collect();
    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "Non-determinism detected at iteration {i}"
        );
    }
}

#[test]
fn determinism_filter_same_result_on_repeated_calls() {
    let ty: Type = parse_quote!(Box<Arc<Rc<u16>>>);
    let skip: HashSet<&str> = HashSet::from(["Box", "Arc", "Rc"]);
    let results: Vec<_> = (0..10)
        .map(|_| ty_str(&filter_inner_type(&ty, &skip)))
        .collect();
    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "Non-determinism detected at iteration {i}"
        );
    }
}

#[test]
fn determinism_wrap_same_result_on_repeated_calls() {
    let ty: Type = parse_quote!(Vec<Option<char>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    let results: Vec<_> = (0..10)
        .map(|_| ty_str(&wrap_leaf_type(&ty, &skip)))
        .collect();
    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "Non-determinism detected at iteration {i}"
        );
    }
}

proptest! {
    #[test]
    fn proptest_determinism_extract_is_stable(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let type_str = format!("{container}<{leaf}>");
        let ty: Type = parse_str(&type_str).unwrap();
        let (r1, e1) = try_extract_inner_type(&ty, container, &HashSet::new());
        let (r2, e2) = try_extract_inner_type(&ty, container, &HashSet::new());
        prop_assert_eq!(e1, e2);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }

    #[test]
    fn proptest_determinism_filter_is_stable(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let type_str = format!("{container}<{leaf}>");
        let ty: Type = parse_str(&type_str).unwrap();
        let skip: HashSet<&str> = HashSet::from([container]);
        let r1 = ty_str(&filter_inner_type(&ty, &skip));
        let r2 = ty_str(&filter_inner_type(&ty, &skip));
        prop_assert_eq!(r1, r2);
    }
}
