//! Property-based and unit tests for type extraction functions (v5).
//!
//! Covers `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! with proptest strategies for random type names, container extraction,
//! wrap/unwrap roundtrips, predicate consistency, nesting, and edge cases.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote, parse_str};

// ===========================================================================
// Strategies
// ===========================================================================

/// Safe type names that avoid all Rust 2024 reserved keywords.
fn safe_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "Foo", "Bar", "Baz", "Qux", "Quux", "Corge", "Grault", "Garply", "Waldo", "Fred",
            "Plugh", "Xyzzy", "Thud", "Alpha", "Beta", "Gamma", "Delta", "Epsilon",
        ][..],
    )
}

fn primitive_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

fn leaf_name() -> impl Strategy<Value = &'static str> {
    prop_oneof![safe_type_name(), primitive_name(),]
}

fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Vec", "Option", "Arc", "Rc"][..])
}

fn skip_set_strategy() -> impl Strategy<Value = HashSet<&'static str>> {
    prop::collection::hash_set(container_name(), 0..=5)
}

/// Generate type strings at varying nesting depths.
fn type_string_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Leaf
        leaf_name().prop_map(|s| s.to_string()),
        // Single container
        (container_name(), leaf_name()).prop_map(|(c, l)| format!("{c}<{l}>")),
        // Double nesting
        (container_name(), container_name(), leaf_name())
            .prop_map(|(c1, c2, l)| format!("{c1}<{c2}<{l}>>")),
        // Triple nesting
        (
            container_name(),
            container_name(),
            container_name(),
            leaf_name()
        )
            .prop_map(|(c1, c2, c3, l)| format!("{c1}<{c2}<{c3}<{l}>>>")),
    ]
}

fn parse_ty(s: &str) -> Type {
    parse_str::<Type>(s).expect("should parse as Type")
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. Type name generation proptests (5 tests)
// ===========================================================================

proptest! {
    #[test]
    fn gen_leaf_names_are_parseable(name in leaf_name()) {
        let ty = parse_ty(name);
        prop_assert!(!ty_str(&ty).is_empty());
    }

    #[test]
    fn gen_container_types_are_parseable(
        container in container_name(),
        leaf in leaf_name(),
    ) {
        let s = format!("{container}<{leaf}>");
        let ty = parse_ty(&s);
        let rendered = ty_str(&ty);
        prop_assert!(rendered.contains(container));
        prop_assert!(rendered.contains(leaf));
    }

    #[test]
    fn gen_nested_types_are_parseable(s in type_string_strategy()) {
        let ty = parse_ty(&s);
        prop_assert!(!ty_str(&ty).is_empty());
    }

    #[test]
    fn gen_type_names_no_reserved_keywords(name in safe_type_name()) {
        // None of our generated names should be Rust reserved keywords
        let reserved = [
            "gen", "do", "abstract", "become", "final", "override", "priv",
            "typeof", "unsized", "virtual",
        ];
        prop_assert!(!reserved.contains(&name));
    }

    #[test]
    fn gen_all_type_strings_roundtrip_through_syn(s in type_string_strategy()) {
        let ty = parse_ty(&s);
        let rendered = ty_str(&ty);
        // Re-parse the rendered string to verify roundtrip
        let ty2 = parse_ty(&rendered);
        prop_assert_eq!(ty_str(&ty), ty_str(&ty2));
    }
}

// ===========================================================================
// 2. Container extraction proptests (8 tests)
// ===========================================================================

proptest! {
    #[test]
    fn extract_vec_inner_returns_leaf(leaf in leaf_name()) {
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    #[test]
    fn extract_option_inner_returns_leaf(leaf in leaf_name()) {
        let ty = parse_ty(&format!("Option<{leaf}>"));
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    #[test]
    fn extract_box_inner_returns_leaf(leaf in leaf_name()) {
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    #[test]
    fn extract_skips_through_box_to_find_vec(leaf in leaf_name()) {
        let ty = parse_ty(&format!("Box<Vec<{leaf}>>"));
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    #[test]
    fn extract_skips_through_arc_to_find_option(leaf in leaf_name()) {
        let ty = parse_ty(&format!("Arc<Option<{leaf}>>"));
        let skip: HashSet<&str> = ["Arc"].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    #[test]
    fn extract_wrong_container_returns_original(leaf in leaf_name()) {
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }

    #[test]
    fn extract_leaf_type_not_extracted(leaf in leaf_name()) {
        let ty = parse_ty(leaf);
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    #[test]
    fn extract_with_random_skip_set(
        container in container_name(),
        leaf in leaf_name(),
        skip in skip_set_strategy(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, container, &skip);
        // When the container IS the target, extraction should always succeed
        // (unless container is also in skip_over, which causes skip-through behavior)
        if !skip.contains(container) {
            prop_assert!(extracted);
            prop_assert_eq!(ty_str(&inner), leaf);
        }
    }
}

// ===========================================================================
// 3. Wrap/unwrap roundtrip proptests (5 tests)
// ===========================================================================

proptest! {
    #[test]
    fn wrap_leaf_produces_with_leaf_wrapper(leaf in leaf_name()) {
        let ty = parse_ty(leaf);
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let rendered = ty_str(&wrapped);
        prop_assert!(rendered.contains("adze :: WithLeaf"));
        prop_assert!(rendered.contains(leaf));
    }

    #[test]
    fn wrap_container_in_skip_set_wraps_inner_only(
        container in container_name(),
        leaf in leaf_name(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let skip: HashSet<&str> = [container].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let rendered = ty_str(&wrapped);
        // Container should remain, inner leaf should be wrapped
        prop_assert!(rendered.starts_with(container));
        prop_assert!(rendered.contains("adze :: WithLeaf"));
    }

    #[test]
    fn wrap_container_not_in_skip_wraps_entire_type(
        container in container_name(),
        leaf in leaf_name(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let rendered = ty_str(&wrapped);
        // Entire type should be wrapped
        prop_assert!(rendered.starts_with("adze :: WithLeaf"));
    }

    #[test]
    fn wrap_is_idempotent_on_structure_when_skip_empty(leaf in leaf_name()) {
        let ty = parse_ty(leaf);
        let skip: HashSet<&str> = HashSet::new();
        let wrapped1 = wrap_leaf_type(&ty, &skip);
        let wrapped2 = wrap_leaf_type(&wrapped1, &skip);
        // Double-wrapping should produce nested WithLeaf
        let r2 = ty_str(&wrapped2);
        prop_assert!(r2.contains("adze :: WithLeaf < adze :: WithLeaf"));
    }

    #[test]
    fn wrap_preserves_container_structure(
        c1 in container_name(),
        c2 in container_name(),
        leaf in leaf_name(),
    ) {
        let ty = parse_ty(&format!("{c1}<{c2}<{leaf}>>"));
        let skip: HashSet<&str> = [c1, c2].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let rendered = ty_str(&wrapped);
        // Both containers should be preserved, leaf wrapped
        prop_assert!(rendered.contains(c1));
        prop_assert!(rendered.contains("adze :: WithLeaf"));
    }
}

// ===========================================================================
// 4. Predicate consistency proptests (5 tests)
// ===========================================================================

proptest! {
    #[test]
    fn filter_leaf_is_identity(leaf in leaf_name()) {
        let ty = parse_ty(leaf);
        let skip: HashSet<&str> = ["Box", "Vec", "Option", "Arc", "Rc"]
            .into_iter()
            .collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    #[test]
    fn filter_removes_single_skip_container(
        container in container_name(),
        leaf in leaf_name(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let skip: HashSet<&str> = [container].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    #[test]
    fn filter_with_empty_skip_is_identity(s in type_string_strategy()) {
        let ty = parse_ty(&s);
        let skip: HashSet<&str> = HashSet::new();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    #[test]
    fn extract_and_filter_agree_on_single_container(
        container in container_name(),
        leaf in leaf_name(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let skip: HashSet<&str> = HashSet::new();
        let (extracted, did_extract) = try_extract_inner_type(&ty, container, &skip);
        let skip_with: HashSet<&str> = [container].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip_with);
        // Both should yield the leaf
        prop_assert!(did_extract);
        prop_assert_eq!(ty_str(&extracted), ty_str(&filtered));
    }

    #[test]
    fn filter_is_idempotent(
        container in container_name(),
        leaf in leaf_name(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let skip: HashSet<&str> = [container].into_iter().collect();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }
}

// ===========================================================================
// 5. Regular extraction tests (8 tests)
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_box_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let skip: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_arc_usize() {
    let ty: Type = parse_quote!(Arc<usize>);
    let skip: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(&ty, "Arc", &skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn extract_mismatched_container_not_found() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip: HashSet<&str> = HashSet::new();
    let (_inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
}

#[test]
fn extract_skip_box_find_option() {
    let ty: Type = parse_quote!(Box<Option<f64>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_skip_arc_find_vec() {
    let ty: Type = parse_quote!(Arc<Vec<char>>);
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn extract_plain_type_returns_unchanged() {
    let ty: Type = parse_quote!(String);
    let skip: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "String");
}

// ===========================================================================
// 6. Regular wrapping tests (5 tests)
// ===========================================================================

#[test]
fn wrap_plain_type() {
    let ty: Type = parse_quote!(String);
    let skip: HashSet<&str> = HashSet::new();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_vec_with_skip() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_option_with_skip() {
    let ty: Type = parse_quote!(Option<bool>);
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < bool > >");
}

#[test]
fn wrap_box_without_skip_wraps_entirely() {
    let ty: Type = parse_quote!(Box<u8>);
    let skip: HashSet<&str> = HashSet::new();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Box < u8 > >");
}

#[test]
fn wrap_result_in_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

// ===========================================================================
// 7. Nested type tests (8 tests)
// ===========================================================================

#[test]
fn nested_box_vec_extraction() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn nested_arc_box_vec_extraction() {
    let ty: Type = parse_quote!(Arc<Box<Vec<u32>>>);
    let skip: HashSet<&str> = ["Arc", "Box"].into_iter().collect();
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn nested_filter_strips_all_layers() {
    let ty: Type = parse_quote!(Box<Arc<Rc<i64>>>);
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&filtered), "i64");
}

#[test]
fn nested_wrap_through_two_skip_containers() {
    let ty: Type = parse_quote!(Vec<Option<f32>>);
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < f32 > > >"
    );
}

#[test]
fn nested_wrap_through_three_skip_containers() {
    let ty: Type = parse_quote!(Box<Vec<Option<char>>>);
    let skip: HashSet<&str> = ["Box", "Vec", "Option"].into_iter().collect();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&wrapped),
        "Box < Vec < Option < adze :: WithLeaf < char > > > >"
    );
}

#[test]
fn nested_extract_stops_at_first_match() {
    // Box<Vec<Option<u8>>> with skip={Box} looking for Vec
    // Should extract Option<u8> (the inner of Vec), not u8
    let ty: Type = parse_quote!(Box<Vec<Option<u8>>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < u8 >");
}

#[test]
fn nested_filter_partial_skip_set() {
    // Box<Arc<String>> with skip={Box} — only Box is stripped
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&filtered), "Arc < String >");
}

#[test]
fn nested_extract_not_found_through_skip() {
    // Box<Arc<String>> looking for Vec through skip={Box, Arc}
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < Arc < String > >");
}

// ===========================================================================
// 8. Edge cases (6 tests)
// ===========================================================================

#[test]
fn edge_reference_type_not_extracted() {
    let ty: Type = parse_quote!(&str);
    let skip: HashSet<&str> = HashSet::new();
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn edge_tuple_type_not_extracted() {
    let ty: Type = parse_quote!((i32, u32));
    let skip: HashSet<&str> = HashSet::new();
    let (_inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
}

#[test]
fn edge_filter_reference_type_unchanged() {
    let ty: Type = parse_quote!(&str);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&filtered), "& str");
}

#[test]
fn edge_wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let skip: HashSet<&str> = HashSet::new();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn edge_wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, bool));
    let skip: HashSet<&str> = HashSet::new();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (i32 , bool) >");
}

#[test]
fn edge_extract_target_checked_before_skip() {
    // When the target container is also in skip_over, the `inner_of` check
    // takes precedence over skip_over — extraction succeeds.
    let ty: Type = parse_quote!(Vec<String>);
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}
