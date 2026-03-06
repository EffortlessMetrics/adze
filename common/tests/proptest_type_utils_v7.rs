//! Property-based tests (v7) for adze-common type utilities.
//!
//! Covers: `try_extract_inner_type`, `filter_inner_type`, `wrap_leaf_type`
//! with 55 proptest+unit tests across 7 categories.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers that avoid all keywords (including 2024-edition `gen`).
fn ident_name() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{1,8}".prop_filter("must not be keyword-like", |s| {
        !matches!(
            s.as_str(),
            "Self" | "Box" | "Vec" | "Option" | "Arc" | "Rc" | "Result" | "Gen"
        )
    })
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

/// Common wrapper/container names.
fn wrapper_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec", "Box", "Arc", "Rc"][..])
}

/// Random skip-set members.
fn skip_set_member() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Arc", "Rc"][..])
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn to_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

/// Local helper: check if a Type::Path has angle-bracketed arguments.
fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

fn skip<'a>(items: &'a [&'a str]) -> HashSet<&'a str> {
    items.iter().copied().collect()
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

// ===========================================================================
// 1. Wrapper detection proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Extracting with matching wrapper name succeeds.
    #[test]
    fn detect_matching_wrapper(
        wrapper in wrapper_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<{leaf}>")).unwrap();
        let (_, extracted) = try_extract_inner_type(&ty, wrapper, &empty_skip());
        prop_assert!(extracted, "should detect {wrapper}<{leaf}>");
    }

    /// Extracting with a different wrapper name fails.
    #[test]
    fn detect_mismatched_wrapper(
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let (_, extracted) = try_extract_inner_type(&ty, "Option", &empty_skip());
        prop_assert!(!extracted);
    }

    /// Random identifier as wrapper name is not detected when type is plain.
    #[test]
    fn detect_random_wrapper_on_plain_type(
        name in ident_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(leaf).unwrap();
        let (_, extracted) = try_extract_inner_type(&ty, &name, &empty_skip());
        prop_assert!(!extracted);
    }

    /// Wrapper detection works when skip set contains different wrappers.
    #[test]
    fn detect_wrapper_with_skip_set(
        leaf in leaf_type(),
        skipper in skip_set_member(),
    ) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let arr = [skipper];
        let skip_set = skip(&arr);
        let (_, extracted) = try_extract_inner_type(&ty, "Option", &skip_set);
        prop_assert!(extracted);
    }

    /// Wrapper detection through a skip layer.
    #[test]
    fn detect_wrapper_through_skip(
        leaf in leaf_type(),
        skipper in skip_set_member(),
    ) {
        let ty: Type = parse_str(&format!("{skipper}<Vec<{leaf}>>")).unwrap();
        let arr = [skipper];
        let skip_set = skip(&arr);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }
}

// ===========================================================================
// 2. Extraction consistency (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// When container is in skip set, filter removes it; extract with that container
    /// as target also removes it. Both yield the same leaf.
    #[test]
    fn extract_and_filter_agree_on_single_wrapper(
        wrapper in wrapper_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<{leaf}>")).unwrap();
        let (extracted_inner, ok) = try_extract_inner_type(&ty, wrapper, &empty_skip());
        let filtered = filter_inner_type(&ty, &skip(&[wrapper]));
        prop_assert!(ok);
        prop_assert_eq!(to_str(&extracted_inner), to_str(&filtered));
    }

    /// Extraction from plain leaf with empty skip returns original.
    #[test]
    fn extract_plain_leaf_returns_original(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        prop_assert!(!ok);
        prop_assert_eq!(to_str(&result), leaf);
    }

    /// Filter on plain leaf with empty skip returns original.
    #[test]
    fn filter_plain_leaf_returns_original(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let filtered = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Double-wrapped: extract targets inner, skip covers outer.
    #[test]
    fn extract_inner_through_outer_skip(
        leaf in leaf_type(),
        outer in skip_set_member(),
    ) {
        let ty: Type = parse_str(&format!("{outer}<Option<{leaf}>>")).unwrap();
        let arr = [outer];
        let skip_set = skip(&arr);
        let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set);
        prop_assert!(ok);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Filter with multiple skip layers strips them all.
    #[test]
    fn filter_strips_all_skip_layers(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<{leaf}>>")).unwrap();
        let skip_set = skip(&["Box", "Arc"]);
        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(to_str(&filtered), leaf);
    }
}

// ===========================================================================
// 3. Parameterized round-trip (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Wrapping a leaf type produces a parameterized type.
    #[test]
    fn wrap_leaf_produces_parameterized(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        prop_assert!(is_parameterized(&wrapped), "wrapped should be parameterized");
    }

    /// Wrapping a container in skip set keeps it parameterized.
    #[test]
    fn wrap_container_in_skip_stays_parameterized(
        wrapper in wrapper_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<{leaf}>")).unwrap();
        let arr = [wrapper];
        let skip_set = skip(&arr);
        let wrapped = wrap_leaf_type(&ty, &skip_set);
        prop_assert!(is_parameterized(&wrapped));
    }

    /// Wrapping a container NOT in skip set adds WithLeaf around the whole thing.
    #[test]
    fn wrap_container_not_in_skip_wraps_entirely(
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        // Vec not in skip set => entire type gets wrapped
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        let s = to_str(&wrapped);
        prop_assert!(s.starts_with("adze :: WithLeaf <"), "got: {s}");
    }

    /// Double wrapping: wrap then check still parameterized.
    #[test]
    fn double_wrap_parameterized(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let once = wrap_leaf_type(&ty, &empty_skip());
        prop_assert!(is_parameterized(&once));
        // Wrap again with skip containing "adze" won't match (different path structure)
        // but the result is still parameterized.
        let twice = wrap_leaf_type(&once, &empty_skip());
        prop_assert!(is_parameterized(&twice));
    }

    /// Wrap Vec<Leaf> with Vec in skip: inner leaf gets WithLeaf but outer stays Vec.
    #[test]
    fn wrap_skip_container_wraps_inner_leaf(
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let skip_set = skip(&["Vec"]);
        let wrapped = wrap_leaf_type(&ty, &skip_set);
        let s = to_str(&wrapped);
        prop_assert!(s.starts_with("Vec <"), "expected Vec< wrapper, got: {s}");
        prop_assert!(s.contains("WithLeaf"), "inner should be wrapped: {s}");
    }
}

// ===========================================================================
// 4. Type name preservation (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Extracting from Container<Leaf> preserves the leaf name.
    #[test]
    fn extract_preserves_leaf_name(
        wrapper in wrapper_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<{leaf}>")).unwrap();
        let (inner, ok) = try_extract_inner_type(&ty, wrapper, &empty_skip());
        prop_assert!(ok);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Filtering Container<Leaf> with container in skip preserves leaf name.
    #[test]
    fn filter_preserves_leaf_name(
        wrapper in wrapper_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[wrapper]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Non-matching extraction preserves full type string.
    #[test]
    fn non_matching_extract_preserves_full_type(
        wrapper in wrapper_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<{leaf}>")).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Nonexistent", &empty_skip());
        prop_assert!(!ok);
        // The full type should be preserved
        let s = to_str(&result);
        prop_assert!(s.contains(wrapper), "should contain wrapper: {s}");
        prop_assert!(s.contains(leaf), "should contain leaf: {s}");
    }

    /// Filter with empty skip preserves the original type entirely.
    #[test]
    fn filter_empty_skip_preserves_type(
        wrapper in wrapper_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<{leaf}>")).unwrap();
        let original = to_str(&ty);
        let filtered = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(to_str(&filtered), original);
    }

    /// Random ident as wrapper: extraction preserves full path.
    #[test]
    fn random_ident_wrapper_preserves_full_path(
        name in ident_name(),
        leaf in leaf_type(),
    ) {
        let src = format!("{name}<{leaf}>");
        let ty: Type = parse_str(&src).unwrap();
        let (result, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        prop_assert!(!ok);
        let s = to_str(&result);
        prop_assert!(s.contains(&name), "should contain {name}: {s}");
    }
}

// ===========================================================================
// 5. Multi-level nesting (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Extract Vec from Option<Vec<Leaf>> with Option skipped.
    #[test]
    fn extract_vec_from_option_vec(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Vec<{leaf}>>")).unwrap();
        let skip_set = skip(&["Option"]);
        let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set);
        prop_assert!(ok);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Extract Option from Box<Arc<Option<Leaf>>> with Box+Arc skipped.
    #[test]
    fn extract_option_through_box_arc(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<Option<{leaf}>>>")).unwrap();
        let skip_set = skip(&["Box", "Arc"]);
        let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set);
        prop_assert!(ok);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Filter Box<Arc<Rc<Leaf>>> with all three in skip yields leaf.
    #[test]
    fn filter_triple_nesting(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<Rc<{leaf}>>>")).unwrap();
        let skip_set = skip(&["Box", "Arc", "Rc"]);
        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Wrap Option<Vec<Leaf>> with both in skip: leaf gets WithLeaf.
    #[test]
    fn wrap_nested_skip_wraps_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Vec<{leaf}>>")).unwrap();
        let skip_set = skip(&["Option", "Vec"]);
        let wrapped = wrap_leaf_type(&ty, &skip_set);
        let s = to_str(&wrapped);
        prop_assert!(s.starts_with("Option <"), "should start with Option: {s}");
        prop_assert!(s.contains("Vec <"), "should contain Vec: {s}");
        prop_assert!(s.contains("WithLeaf"), "leaf should be wrapped: {s}");
    }

    /// Partial skip on Box<Vec<Leaf>>: skip only Box => Vec<Leaf> remains unwrapped by filter.
    #[test]
    fn partial_skip_nesting(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Vec<{leaf}>>")).unwrap();
        let skip_set = skip(&["Box"]);
        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(to_str(&filtered), format!("Vec < {leaf} >"));
    }
}

// ===========================================================================
// 6. Regular type utils tests (15 tests)
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(to_str(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(to_str(&inner), "i32");
}

#[test]
fn extract_box_u8() {
    let ty: Type = parse_quote!(Box<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(ok);
    assert_eq!(to_str(&inner), "u8");
}

#[test]
fn extract_no_match_returns_original() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (result, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(to_str(&result), to_str(&ty));
}

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_str(&filtered), "String");
}

#[test]
fn filter_arc_box_string() {
    let ty: Type = parse_quote!(Arc<Box<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(to_str(&filtered), "String");
}

#[test]
fn filter_no_skip_returns_original() {
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &empty_skip());
    assert_eq!(to_str(&filtered), "Vec < i32 >");
}

#[test]
fn wrap_string_adds_with_leaf() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(to_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_vec_in_skip_wraps_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(to_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_vec_in_skip_wraps_deepest() {
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        to_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < u32 > > >"
    );
}

#[test]
fn wrap_not_in_skip_wraps_entirely() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(to_str(&wrapped), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn is_parameterized_plain_false() {
    let ty: Type = parse_quote!(String);
    assert!(!is_parameterized(&ty));
}

#[test]
fn is_parameterized_generic_true() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn extract_box_vec_through_skip() {
    let ty: Type = parse_quote!(Box<Vec<bool>>);
    let skip_set = skip(&["Box"]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set);
    assert!(ok);
    assert_eq!(to_str(&inner), "bool");
}

#[test]
fn wrap_result_in_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        to_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

// ===========================================================================
// 7. Edge cases (10 tests)
// ===========================================================================

#[test]
fn edge_reference_type_extract_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (result, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(to_str(&result), "& str");
}

#[test]
fn edge_reference_type_filter_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_str(&filtered), "& str");
}

#[test]
fn edge_reference_type_wrap_adds_with_leaf() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(to_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn edge_tuple_type_extract_returns_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let (result, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(to_str(&result), "(i32 , u32)");
}

#[test]
fn edge_tuple_type_filter_returns_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_str(&filtered), "(i32 , u32)");
}

#[test]
fn edge_tuple_type_wrap_adds_with_leaf() {
    let ty: Type = parse_quote!((i32, u32));
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(to_str(&wrapped), "adze :: WithLeaf < (i32 , u32) >");
}

#[test]
fn edge_array_type_extract_returns_unchanged() {
    let ty: Type = parse_quote!([u8; 4]);
    let (result, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(to_str(&result), "[u8 ; 4]");
}

#[test]
fn edge_array_type_wrap_adds_with_leaf() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(to_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn edge_unit_type_extract_returns_unchanged() {
    let ty: Type = parse_quote!(());
    let (result, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(to_str(&result), "()");
}

#[test]
fn edge_qualified_path_extract_returns_unchanged() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    let (result, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok);
    // HashMap is the last segment, not Vec
    let s = to_str(&result);
    assert!(s.contains("HashMap"));
}
