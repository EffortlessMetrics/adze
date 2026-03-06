//! Property-based and unit tests for wrap / extract / filter composition
//! in adze-common.
//!
//! Covers roundtrip properties, non-matching extraction, double wrapping,
//! filter identity, composition chains, and type preservation.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::quote;
use std::collections::HashSet;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_base_type() -> impl Strategy<Value = Type> {
    prop_oneof![
        Just(parse_quote!(i32)),
        Just(parse_quote!(u32)),
        Just(parse_quote!(String)),
        Just(parse_quote!(bool)),
        Just(parse_quote!(f64)),
        Just(parse_quote!(u8)),
        Just(parse_quote!(i64)),
    ]
}

fn arb_wrapper() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("Vec"), Just("Option"), Just("Box")]
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ty_str(ty: &Type) -> String {
    quote!(#ty).to_string()
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

fn skip<'a>(names: &[&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

/// Build a single-generic container type: `Wrapper<Inner>`.
fn containerize(wrapper: &str, inner: &Type) -> Type {
    let ident: syn::Ident = syn::parse_str(wrapper).unwrap();
    parse_quote!(#ident<#inner>)
}

// ===========================================================================
// Property 1: wrap(ty) then extract("WithLeaf") → (ty, true) roundtrip
// ===========================================================================

proptest! {
    #[test]
    fn prop_roundtrip_wrap_extract_bare(ty in arb_base_type()) {
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        let (extracted, found) = try_extract_inner_type(&wrapped, "WithLeaf", &empty_skip());
        prop_assert!(found, "expected extraction to succeed");
        prop_assert_eq!(ty_str(&extracted), ty_str(&ty));
    }

    #[test]
    fn prop_roundtrip_container_two_step(
        ty in arb_base_type(),
        w in arb_wrapper(),
    ) {
        // wrap(W<ty>, skip={W}) → W<adze::WithLeaf<ty>>
        // extract(W) → (adze::WithLeaf<ty>, true)
        // extract("WithLeaf") → (ty, true)
        let container = containerize(w, &ty);
        let wrapped = wrap_leaf_type(&container, &skip(&[w]));
        let (mid, found1) = try_extract_inner_type(&wrapped, w, &empty_skip());
        prop_assert!(found1);
        let (final_ty, found2) = try_extract_inner_type(&mid, "WithLeaf", &empty_skip());
        prop_assert!(found2);
        prop_assert_eq!(ty_str(&final_ty), ty_str(&ty));
    }
}

#[test]
fn roundtrip_i32() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    let (extracted, found) = try_extract_inner_type(&wrapped, "WithLeaf", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&extracted), "i32");
}

#[test]
fn roundtrip_string() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    let (extracted, found) = try_extract_inner_type(&wrapped, "WithLeaf", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&extracted), "String");
}

#[test]
fn roundtrip_bool() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    let (extracted, found) = try_extract_inner_type(&wrapped, "WithLeaf", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&extracted), "bool");
}

#[test]
fn roundtrip_f64() {
    let ty: Type = parse_quote!(f64);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    let (extracted, found) = try_extract_inner_type(&wrapped, "WithLeaf", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&extracted), "f64");
}

#[test]
fn roundtrip_vec_i32_two_step() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    let (mid, found1) = try_extract_inner_type(&wrapped, "Vec", &empty_skip());
    assert!(found1);
    let (inner, found2) = try_extract_inner_type(&mid, "WithLeaf", &empty_skip());
    assert!(found2);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn roundtrip_option_string_two_step() {
    let ty: Type = parse_quote!(Option<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    let (mid, found1) = try_extract_inner_type(&wrapped, "Option", &empty_skip());
    assert!(found1);
    let (inner, found2) = try_extract_inner_type(&mid, "WithLeaf", &empty_skip());
    assert!(found2);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn roundtrip_box_f64_two_step() {
    let ty: Type = parse_quote!(Box<f64>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    let (mid, found1) = try_extract_inner_type(&wrapped, "Box", &empty_skip());
    assert!(found1);
    let (inner, found2) = try_extract_inner_type(&mid, "WithLeaf", &empty_skip());
    assert!(found2);
    assert_eq!(ty_str(&inner), "f64");
}

// ===========================================================================
// Property 2: wrap produces different string than original
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_changes_bare_type(ty in arb_base_type()) {
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        prop_assert_ne!(ty_str(&wrapped), ty_str(&ty));
    }

    #[test]
    fn prop_wrap_changes_container_type(
        ty in arb_base_type(),
        w in arb_wrapper(),
    ) {
        let container = containerize(w, &ty);
        let wrapped = wrap_leaf_type(&container, &skip(&[w]));
        prop_assert_ne!(ty_str(&wrapped), ty_str(&container));
    }
}

#[test]
fn wrap_i32_differs_from_i32() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_ne!(ty_str(&wrapped), ty_str(&ty));
}

#[test]
fn wrap_vec_u32_differs_with_skip() {
    let ty: Type = parse_quote!(Vec<u32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_ne!(ty_str(&wrapped), ty_str(&ty));
}

#[test]
fn wrap_option_bool_differs_no_skip() {
    let ty: Type = parse_quote!(Option<bool>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_ne!(ty_str(&wrapped), ty_str(&ty));
}

// ===========================================================================
// Property 3: extract non-matching wrapper → (original, false)
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_nonmatching_returns_false(ty in arb_base_type()) {
        let (result, found) = try_extract_inner_type(&ty, "NonExistent", &empty_skip());
        prop_assert!(!found);
        prop_assert_eq!(ty_str(&result), ty_str(&ty));
    }

    #[test]
    fn prop_extract_wrong_container(
        ty in arb_base_type(),
        w in arb_wrapper(),
    ) {
        let container = containerize(w, &ty);
        let wrong_target = if w == "Vec" { "Option" } else { "Vec" };
        let (result, found) = try_extract_inner_type(&container, wrong_target, &empty_skip());
        prop_assert!(!found);
        prop_assert_eq!(ty_str(&result), ty_str(&container));
    }
}

#[test]
fn extract_foo_from_i32_returns_false() {
    let ty: Type = parse_quote!(i32);
    let (result, found) = try_extract_inner_type(&ty, "Foo", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn extract_vec_from_option_string_returns_false() {
    let ty: Type = parse_quote!(Option<String>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&result), "Option < String >");
}

#[test]
fn extract_option_from_vec_i32_returns_false() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

#[test]
fn extract_box_from_string_returns_false() {
    let ty: Type = parse_quote!(String);
    let (result, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&result), "String");
}

// ===========================================================================
// Property 4: wrap then wrap → doubly wrapped
// ===========================================================================

proptest! {
    #[test]
    fn prop_double_wrap_bare(ty in arb_base_type()) {
        let once = wrap_leaf_type(&ty, &empty_skip());
        let twice = wrap_leaf_type(&once, &empty_skip());
        let expected = format!("adze :: WithLeaf < adze :: WithLeaf < {} > >", ty_str(&ty));
        prop_assert_eq!(ty_str(&twice), expected);
    }
}

#[test]
fn double_wrap_i32() {
    let ty: Type = parse_quote!(i32);
    let once = wrap_leaf_type(&ty, &empty_skip());
    let twice = wrap_leaf_type(&once, &empty_skip());
    assert_eq!(
        ty_str(&twice),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn double_wrap_string() {
    let ty: Type = parse_quote!(String);
    let once = wrap_leaf_type(&ty, &empty_skip());
    let twice = wrap_leaf_type(&once, &empty_skip());
    assert_eq!(
        ty_str(&twice),
        "adze :: WithLeaf < adze :: WithLeaf < String > >"
    );
}

#[test]
fn double_wrap_vec_then_whole() {
    let ty: Type = parse_quote!(Vec<i32>);
    let first = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&first), "Vec < adze :: WithLeaf < i32 > >");
    let second = wrap_leaf_type(&first, &empty_skip());
    assert!(ty_str(&second).starts_with("adze :: WithLeaf <"));
    assert!(ty_str(&second).contains("Vec"));
}

// ===========================================================================
// Property 5: double extract → peels one layer each
// ===========================================================================

proptest! {
    #[test]
    fn prop_double_extract_peels_layers(ty in arb_base_type()) {
        let once = wrap_leaf_type(&ty, &empty_skip());
        let twice = wrap_leaf_type(&once, &empty_skip());
        let (mid, found1) = try_extract_inner_type(&twice, "WithLeaf", &empty_skip());
        prop_assert!(found1);
        let (inner, found2) = try_extract_inner_type(&mid, "WithLeaf", &empty_skip());
        prop_assert!(found2);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }
}

#[test]
fn double_extract_i32() {
    let ty: Type = parse_quote!(i32);
    let once = wrap_leaf_type(&ty, &empty_skip());
    let twice = wrap_leaf_type(&once, &empty_skip());
    let (mid, found1) = try_extract_inner_type(&twice, "WithLeaf", &empty_skip());
    assert!(found1);
    let (inner, found2) = try_extract_inner_type(&mid, "WithLeaf", &empty_skip());
    assert!(found2);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn double_extract_u32() {
    let ty: Type = parse_quote!(u32);
    let once = wrap_leaf_type(&ty, &empty_skip());
    let twice = wrap_leaf_type(&once, &empty_skip());
    let (mid, _) = try_extract_inner_type(&twice, "WithLeaf", &empty_skip());
    let (inner, found) = try_extract_inner_type(&mid, "WithLeaf", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "u32");
}

// ===========================================================================
// Property 6: filter with empty skip → unchanged
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_empty_skip_bare_unchanged(ty in arb_base_type()) {
        let filtered = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    #[test]
    fn prop_filter_empty_skip_container_unchanged(
        ty in arb_base_type(),
        w in arb_wrapper(),
    ) {
        let container = containerize(w, &ty);
        let filtered = filter_inner_type(&container, &empty_skip());
        prop_assert_eq!(ty_str(&filtered), ty_str(&container));
    }
}

#[test]
fn filter_empty_skip_i32() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &empty_skip())), "i32");
}

#[test]
fn filter_empty_skip_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &empty_skip())),
        "Vec < String >"
    );
}

#[test]
fn filter_empty_skip_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &empty_skip())),
        "Option < bool >"
    );
}

#[test]
fn filter_empty_skip_nested_box_vec() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &empty_skip())),
        "Box < Vec < i32 > >"
    );
}

// ===========================================================================
// Property 7: wrap preserves inner type name
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_preserves_leaf_name(ty in arb_base_type()) {
        let original = ty_str(&ty);
        let wrapped = ty_str(&wrap_leaf_type(&ty, &empty_skip()));
        prop_assert!(wrapped.contains(&original), "leaf name missing from: {wrapped}");
    }

    #[test]
    fn prop_wrap_container_preserves_both_names(
        ty in arb_base_type(),
        w in arb_wrapper(),
    ) {
        let container = containerize(w, &ty);
        let wrapped = ty_str(&wrap_leaf_type(&container, &skip(&[w])));
        prop_assert!(wrapped.contains(w), "wrapper name missing from: {wrapped}");
        prop_assert!(wrapped.contains(&ty_str(&ty)), "leaf name missing from: {wrapped}");
    }
}

#[test]
fn wrap_preserves_string_name() {
    let ty: Type = parse_quote!(String);
    let wrapped = ty_str(&wrap_leaf_type(&ty, &empty_skip()));
    assert!(wrapped.contains("String"));
}

#[test]
fn wrap_vec_preserves_vec_and_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"])));
    assert!(wrapped.contains("Vec"));
    assert!(wrapped.contains("i32"));
}

// ===========================================================================
// Property 8: extract is idempotent on non-matching
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_idempotent_nonmatching(ty in arb_base_type()) {
        let (first, found1) = try_extract_inner_type(&ty, "Nonexistent", &empty_skip());
        prop_assert!(!found1);
        let (second, found2) = try_extract_inner_type(&first, "Nonexistent", &empty_skip());
        prop_assert!(!found2);
        prop_assert_eq!(ty_str(&first), ty_str(&second));
    }
}

#[test]
fn extract_idempotent_i32_hashmap() {
    let ty: Type = parse_quote!(i32);
    let (first, _) = try_extract_inner_type(&ty, "HashMap", &empty_skip());
    let (second, found) = try_extract_inner_type(&first, "HashMap", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&first), ty_str(&second));
}

#[test]
fn extract_idempotent_vec_u8_box() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (first, _) = try_extract_inner_type(&ty, "Box", &empty_skip());
    let (second, found) = try_extract_inner_type(&first, "Box", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&first), ty_str(&second));
}

// ===========================================================================
// Property 9: wrap(Option<Vec<ty>>, skip) then extract "Option" → Vec<…>
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_nested_extract_outer(ty in arb_base_type()) {
        let inner_vec = containerize("Vec", &ty);
        let outer_opt = containerize("Option", &inner_vec);
        let wrapped = wrap_leaf_type(&outer_opt, &skip(&["Option", "Vec"]));
        let (after_extract, found) = try_extract_inner_type(&wrapped, "Option", &empty_skip());
        prop_assert!(found);
        let s = ty_str(&after_extract);
        prop_assert!(s.starts_with("Vec"), "expected Vec prefix, got: {s}");
    }
}

#[test]
fn wrap_option_vec_i32_extract_option() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    let (inner, found) = try_extract_inner_type(&wrapped, "Option", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "Vec < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_vec_box_string_extract_vec() {
    let ty: Type = parse_quote!(Vec<Box<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Box"]));
    let (inner, found) = try_extract_inner_type(&wrapped, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "Box < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_box_option_u8_extract_box() {
    let ty: Type = parse_quote!(Box<Option<u8>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box", "Option"]));
    let (inner, found) = try_extract_inner_type(&wrapped, "Box", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "Option < adze :: WithLeaf < u8 > >");
}

// ===========================================================================
// Property 10: composition — various wrap/extract/filter chains
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_then_filter_same_skip(
        ty in arb_base_type(),
        w in arb_wrapper(),
    ) {
        // wrap(W<ty>, skip={W}) → W<adze::WithLeaf<ty>>
        // filter(result, skip={W}) → adze::WithLeaf<ty>
        let container = containerize(w, &ty);
        let wrapped = wrap_leaf_type(&container, &skip(&[w]));
        let filtered = filter_inner_type(&wrapped, &skip(&[w]));
        let expected = format!("adze :: WithLeaf < {} >", ty_str(&ty));
        prop_assert_eq!(ty_str(&filtered), expected);
    }

    #[test]
    fn prop_filter_then_wrap_then_extract(
        ty in arb_base_type(),
        w in arb_wrapper(),
    ) {
        // filter(W<ty>, skip={W}) → ty
        // wrap(ty) → adze::WithLeaf<ty>
        // extract("WithLeaf") → (ty, true)
        let container = containerize(w, &ty);
        let filtered = filter_inner_type(&container, &skip(&[w]));
        let wrapped = wrap_leaf_type(&filtered, &empty_skip());
        let (extracted, found) = try_extract_inner_type(&wrapped, "WithLeaf", &empty_skip());
        prop_assert!(found);
        prop_assert_eq!(ty_str(&extracted), ty_str(&ty));
    }

    #[test]
    fn prop_wrap_output_always_contains_with_leaf(
        ty in arb_base_type(),
        w in arb_wrapper(),
    ) {
        let container = containerize(w, &ty);
        let wrapped = ty_str(&wrap_leaf_type(&container, &skip(&[w])));
        prop_assert!(wrapped.contains("WithLeaf"), "missing WithLeaf in: {wrapped}");
    }

    #[test]
    fn prop_wrap_output_parseable(ty in arb_base_type()) {
        let wrapped = ty_str(&wrap_leaf_type(&ty, &empty_skip()));
        prop_assert!(syn::parse_str::<Type>(&wrapped).is_ok(), "unparseable: {wrapped}");
    }
}

#[test]
fn chain_wrap_filter_extract_vec_u32() {
    let ty: Type = parse_quote!(Vec<u32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    let filtered = filter_inner_type(&wrapped, &skip(&["Vec"]));
    let (inner, found) = try_extract_inner_type(&filtered, "WithLeaf", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn chain_wrap_filter_extract_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    let filtered = filter_inner_type(&wrapped, &skip(&["Option"]));
    let (inner, found) = try_extract_inner_type(&filtered, "WithLeaf", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn chain_wrap_filter_extract_box_i64() {
    let ty: Type = parse_quote!(Box<i64>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    let filtered = filter_inner_type(&wrapped, &skip(&["Box"]));
    let (inner, found) = try_extract_inner_type(&filtered, "WithLeaf", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "i64");
}

#[test]
fn chain_filter_then_wrap_then_extract() {
    let ty: Type = parse_quote!(Box<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let wrapped = wrap_leaf_type(&filtered, &empty_skip());
    let (inner, found) = try_extract_inner_type(&wrapped, "WithLeaf", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn chain_extract_then_wrap_roundtrip() {
    let ty: Type = parse_quote!(Vec<String>);
    let (extracted, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    let wrapped = wrap_leaf_type(&extracted, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn chain_triple_wrap_extract_all_layers() {
    let ty: Type = parse_quote!(i32);
    let w1 = wrap_leaf_type(&ty, &empty_skip());
    let w2 = wrap_leaf_type(&w1, &empty_skip());
    let w3 = wrap_leaf_type(&w2, &empty_skip());
    let (l1, f1) = try_extract_inner_type(&w3, "WithLeaf", &empty_skip());
    assert!(f1);
    let (l2, f2) = try_extract_inner_type(&l1, "WithLeaf", &empty_skip());
    assert!(f2);
    let (l3, f3) = try_extract_inner_type(&l2, "WithLeaf", &empty_skip());
    assert!(f3);
    assert_eq!(ty_str(&l3), "i32");
}

#[test]
fn chain_full_pipeline_vec_option_i32() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    let (after_opt, f1) = try_extract_inner_type(&wrapped, "Option", &empty_skip());
    assert!(f1);
    let (after_vec, f2) = try_extract_inner_type(&after_opt, "Vec", &empty_skip());
    assert!(f2);
    let (inner, f3) = try_extract_inner_type(&after_vec, "WithLeaf", &empty_skip());
    assert!(f3);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn chain_full_pipeline_box_option_string() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box", "Option"]));
    let (after_box, f1) = try_extract_inner_type(&wrapped, "Box", &empty_skip());
    assert!(f1);
    let (after_opt, f2) = try_extract_inner_type(&after_box, "Option", &empty_skip());
    assert!(f2);
    let (inner, f3) = try_extract_inner_type(&after_opt, "WithLeaf", &empty_skip());
    assert!(f3);
    assert_eq!(ty_str(&inner), "String");
}

// ===========================================================================
// Additional unit tests — edge cases and broader coverage
// ===========================================================================

#[test]
fn extract_with_skip_passes_through_wrapper() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_with_skip_not_found() {
    let ty: Type = parse_quote!(Box<String>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!found);
    assert_eq!(ty_str(&result), "Box < String >");
}

#[test]
fn filter_strips_single_layer() {
    let ty: Type = parse_quote!(Box<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_strips_nested_layers() {
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_stops_at_non_skip_layer() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
}

#[test]
fn wrap_with_no_skip_wraps_entire_container() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn wrap_nested_with_partial_skip() {
    let ty: Type = parse_quote!(Option<Box<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < adze :: WithLeaf < Box < i32 > > >"
    );
}

#[test]
fn wrap_nested_with_full_skip() {
    let ty: Type = parse_quote!(Option<Box<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Box"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Box < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_always_contains_with_leaf_bare() {
    let ty: Type = parse_quote!(u8);
    let wrapped = ty_str(&wrap_leaf_type(&ty, &empty_skip()));
    assert!(wrapped.contains("WithLeaf"));
}

#[test]
fn wrap_always_contains_with_leaf_container() {
    let ty: Type = parse_quote!(Vec<u8>);
    let wrapped = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"])));
    assert!(wrapped.contains("WithLeaf"));
}

#[test]
fn filter_then_wrap_composition() {
    let ty: Type = parse_quote!(Arc<Box<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(ty_str(&filtered), "String");
    let wrapped = wrap_leaf_type(&filtered, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn extract_then_filter_composition() {
    let ty: Type = parse_quote!(Vec<Box<i32>>);
    let (extracted, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    let filtered = filter_inner_type(&extracted, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn wrap_deterministic_bare() {
    let ty: Type = parse_quote!(f64);
    let a = ty_str(&wrap_leaf_type(&ty, &empty_skip()));
    let b = ty_str(&wrap_leaf_type(&ty, &empty_skip()));
    assert_eq!(a, b);
}

#[test]
fn wrap_deterministic_container() {
    let ty: Type = parse_quote!(Vec<String>);
    let s = skip(&["Vec"]);
    let a = ty_str(&wrap_leaf_type(&ty, &s));
    let b = ty_str(&wrap_leaf_type(&ty, &s));
    assert_eq!(a, b);
}

#[test]
fn extract_deterministic_nonmatching() {
    let ty: Type = parse_quote!(String);
    let (a, _) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    let (b, _) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert_eq!(ty_str(&a), ty_str(&b));
}

#[test]
fn filter_deterministic() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let s = skip(&["Box"]);
    let a = ty_str(&filter_inner_type(&ty, &s));
    let b = ty_str(&filter_inner_type(&ty, &s));
    assert_eq!(a, b);
}

#[test]
fn wrap_output_is_valid_type_bare() {
    let ty: Type = parse_quote!(i64);
    let wrapped = ty_str(&wrap_leaf_type(&ty, &empty_skip()));
    assert!(syn::parse_str::<Type>(&wrapped).is_ok());
}

#[test]
fn wrap_output_is_valid_type_container() {
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    let wrapped = ty_str(&wrap_leaf_type(&ty, &skip(&["Option", "Vec"])));
    assert!(syn::parse_str::<Type>(&wrapped).is_ok());
}

#[test]
fn extract_output_is_valid_type() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    let (extracted, _) = try_extract_inner_type(&wrapped, "WithLeaf", &empty_skip());
    assert!(syn::parse_str::<Type>(&ty_str(&extracted)).is_ok());
}

#[test]
fn filter_nonskip_type_returns_self() {
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
}

#[test]
fn wrap_and_filter_cancel_for_single_wrapper() {
    let ty: Type = parse_quote!(Vec<i32>);
    let s = skip(&["Vec"]);
    let wrapped = wrap_leaf_type(&ty, &s);
    let filtered = filter_inner_type(&wrapped, &s);
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < i32 >");
}

#[test]
fn extract_via_skip_over_deep_nesting() {
    let ty: Type = parse_quote!(Arc<Box<Vec<i32>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc", "Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn wrap_u8_exact_output() {
    let ty: Type = parse_quote!(u8);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u8 >");
}

#[test]
fn wrap_u32_exact_output() {
    let ty: Type = parse_quote!(u32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u32 >");
}

#[test]
fn wrap_i64_exact_output() {
    let ty: Type = parse_quote!(i64);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i64 >");
}

#[test]
fn extract_vec_from_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_option_from_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_box_from_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn filter_vec_strips_vec() {
    let ty: Type = parse_quote!(Vec<u32>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&filtered), "u32");
}

#[test]
fn filter_option_strips_option() {
    let ty: Type = parse_quote!(Option<f64>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "f64");
}

#[test]
fn chain_wrap_extract_wrap_extract() {
    // wrap(i32) → adze::WithLeaf<i32>
    // extract → i32
    // wrap again → adze::WithLeaf<i32>
    // extract again → i32
    let ty: Type = parse_quote!(i32);
    let w1 = wrap_leaf_type(&ty, &empty_skip());
    let (e1, f1) = try_extract_inner_type(&w1, "WithLeaf", &empty_skip());
    assert!(f1);
    assert_eq!(ty_str(&e1), "i32");
    let w2 = wrap_leaf_type(&e1, &empty_skip());
    let (e2, f2) = try_extract_inner_type(&w2, "WithLeaf", &empty_skip());
    assert!(f2);
    assert_eq!(ty_str(&e2), "i32");
}

#[test]
fn double_wrap_container_skip() {
    // wrap(Vec<i32>, skip={Vec}) → Vec<adze::WithLeaf<i32>>
    // wrap(Vec<adze::WithLeaf<i32>>, skip={Vec}) → Vec<adze::WithLeaf<adze::WithLeaf<i32>>>
    let ty: Type = parse_quote!(Vec<i32>);
    let s = skip(&["Vec"]);
    let once = wrap_leaf_type(&ty, &s);
    let twice = wrap_leaf_type(&once, &s);
    let result = ty_str(&twice);
    assert!(result.starts_with("Vec <"));
    // Two layers of WithLeaf
    assert_eq!(
        result.matches("WithLeaf").count(),
        2,
        "expected 2 WithLeaf layers, got: {result}"
    );
}
