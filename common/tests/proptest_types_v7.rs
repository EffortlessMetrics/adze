//! Property-based tests (v7) for type operations in adze-common.
//!
//! Covers `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! with 80+ proptest and unit tests across multiple categories.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_primitive_type() -> impl Strategy<Value = Type> {
    prop_oneof![
        Just(parse_quote!(i32)),
        Just(parse_quote!(u32)),
        Just(parse_quote!(i64)),
        Just(parse_quote!(u64)),
        Just(parse_quote!(f32)),
        Just(parse_quote!(f64)),
        Just(parse_quote!(bool)),
        Just(parse_quote!(String)),
        Just(parse_quote!(u8)),
        Just(parse_quote!(i8)),
    ]
}

fn arb_wrapper_name() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("Vec"), Just("Option"), Just("Box"),]
}

fn arb_skip_member() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("Box"), Just("Arc"), Just("Rc"),]
}

fn arb_leaf_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

fn parse_ty(s: &str) -> Type {
    parse_str(s).unwrap()
}

fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

// ===========================================================================
// Category 1: wrap then extract roundtrip (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Wrapping a leaf with Vec in skip produces Vec<WithLeaf<T>>; extracting
    /// Vec from that succeeds and yields WithLeaf<T>.
    #[test]
    fn wrap_then_extract_vec_roundtrip(leaf in arb_primitive_type()) {
        let vec_ty: Type = parse_quote!(Vec<#leaf>);
        let wrapped = wrap_leaf_type(&vec_ty, &skip(&["Vec"]));
        let (inner, extracted) = try_extract_inner_type(&wrapped, "Vec", &empty_skip());
        prop_assert!(extracted, "should extract Vec from wrapped type");
        let s = ty_str(&inner);
        prop_assert!(s.contains("WithLeaf"), "inner should contain WithLeaf, got: {s}");
    }

    /// Wrapping Option<T> with Option in skip, then extracting Option succeeds.
    #[test]
    fn wrap_then_extract_option_roundtrip(leaf in arb_primitive_type()) {
        let opt_ty: Type = parse_quote!(Option<#leaf>);
        let wrapped = wrap_leaf_type(&opt_ty, &skip(&["Option"]));
        let (inner, extracted) = try_extract_inner_type(&wrapped, "Option", &empty_skip());
        prop_assert!(extracted);
        let s = ty_str(&inner);
        prop_assert!(s.contains("WithLeaf"), "got: {s}");
    }

    /// Extract then wrap: extracting from Vec<T> yields T, wrapping T yields WithLeaf<T>.
    #[test]
    fn extract_then_wrap_roundtrip(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        prop_assert!(extracted);
        let wrapped = wrap_leaf_type(&inner, &empty_skip());
        let s = ty_str(&wrapped);
        prop_assert!(s.contains("adze :: WithLeaf"), "got: {s}");
    }

    /// Filter then wrap: filter Box<T> → T, wrap T → WithLeaf<T>.
    #[test]
    fn filter_then_wrap_roundtrip(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let filtered = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(ty_str(&filtered), leaf);
        let wrapped = wrap_leaf_type(&filtered, &empty_skip());
        let s = ty_str(&wrapped);
        prop_assert!(s.contains("adze :: WithLeaf"), "got: {s}");
    }
}

// ===========================================================================
// Category 2: extract non-matching returns original with false (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Extracting "Vec" from a plain leaf returns false and the original type.
    #[test]
    fn extract_nonmatching_plain_returns_false(leaf in arb_primitive_type()) {
        let (result, extracted) = try_extract_inner_type(&leaf, "Vec", &empty_skip());
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&result), ty_str(&leaf));
    }

    /// Extracting "Option" from Vec<T> returns false.
    #[test]
    fn extract_vec_as_option_returns_false(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &empty_skip());
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&result), format!("Vec < {leaf} >"));
    }

    /// Extracting "Box" from Option<T> returns false.
    #[test]
    fn extract_option_as_box_returns_false(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Option<{leaf}>"));
        let (result, extracted) = try_extract_inner_type(&ty, "Box", &empty_skip());
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&result), format!("Option < {leaf} >"));
    }

    /// Extracting any wrapper from a completely different wrapper returns false.
    #[test]
    fn extract_mismatched_wrapper_returns_false(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Arc<{leaf}>"));
        let (_, extracted) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        prop_assert!(!extracted);
    }
}

// ===========================================================================
// Category 3: wrap produces valid type (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Wrapping any primitive produces a parameterized type.
    #[test]
    fn wrap_produces_parameterized_type(leaf in arb_primitive_type()) {
        let wrapped = wrap_leaf_type(&leaf, &empty_skip());
        prop_assert!(is_parameterized(&wrapped), "wrapped should be parameterized");
    }

    /// Wrapping a container in skip set preserves parameterization.
    #[test]
    fn wrap_container_in_skip_stays_valid(
        wrapper in arb_wrapper_name(),
        leaf in arb_leaf_name(),
    ) {
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let arr = [wrapper];
        let skip_set = skip(&arr);
        let wrapped = wrap_leaf_type(&ty, &skip_set);
        prop_assert!(is_parameterized(&wrapped));
    }

    /// Wrapping never produces an empty string.
    #[test]
    fn wrap_result_is_nonempty(leaf in arb_primitive_type()) {
        let wrapped = wrap_leaf_type(&leaf, &empty_skip());
        prop_assert!(!ty_str(&wrapped).is_empty());
    }

    /// Wrapped type string always contains the leaf name.
    #[test]
    fn wrap_preserves_leaf_name_in_output(leaf in arb_leaf_name()) {
        let ty = parse_ty(leaf);
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        let s = ty_str(&wrapped);
        prop_assert!(s.contains(leaf), "expected {leaf} in {s}");
    }
}

// ===========================================================================
// Category 4: filter with empty skip → unchanged (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Filter on plain leaf with empty skip returns original.
    #[test]
    fn filter_empty_skip_preserves_leaf(leaf in arb_leaf_name()) {
        let ty = parse_ty(leaf);
        let result = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(ty_str(&result), leaf);
    }

    /// Filter on wrapped type with empty skip returns original wrapped type.
    #[test]
    fn filter_empty_skip_preserves_wrapped(
        wrapper in arb_wrapper_name(),
        leaf in arb_leaf_name(),
    ) {
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let result = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(ty_str(&result), format!("{wrapper} < {leaf} >"));
    }

    /// Filter on primitive with empty skip is identity.
    #[test]
    fn filter_empty_skip_identity_on_primitive(leaf in arb_primitive_type()) {
        let result = filter_inner_type(&leaf, &empty_skip());
        prop_assert_eq!(ty_str(&result), ty_str(&leaf));
    }
}

// ===========================================================================
// Category 5: wrap changes type string (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Wrapping a plain leaf always changes the type string.
    #[test]
    fn wrap_changes_type_string_for_leaf(leaf in arb_primitive_type()) {
        let original = ty_str(&leaf);
        let wrapped = ty_str(&wrap_leaf_type(&leaf, &empty_skip()));
        prop_assert_ne!(original, wrapped);
    }

    /// Wrapping a container not in skip changes the string.
    #[test]
    fn wrap_changes_string_when_not_in_skip(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let original = ty_str(&ty);
        let wrapped = ty_str(&wrap_leaf_type(&ty, &empty_skip()));
        prop_assert_ne!(original, wrapped);
    }

    /// Wrapping a container in skip also changes the string (inner gets wrapped).
    #[test]
    fn wrap_changes_string_when_in_skip(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let original = ty_str(&ty);
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"])));
        prop_assert_ne!(original, wrapped);
    }
}

// ===========================================================================
// Category 6: extract with correct wrapper → true (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Extracting with the matching wrapper always succeeds.
    #[test]
    fn extract_matching_wrapper_succeeds(
        wrapper in arb_wrapper_name(),
        leaf in arb_leaf_name(),
    ) {
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, wrapper, &empty_skip());
        prop_assert!(extracted, "{wrapper}<{leaf}> should extract");
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    /// Extracting through a skip layer succeeds.
    #[test]
    fn extract_through_skip_layer_succeeds(
        leaf in arb_leaf_name(),
        skipper in arb_skip_member(),
    ) {
        let ty = parse_ty(&format!("{skipper}<Vec<{leaf}>>"));
        let arr = [skipper];
        let skip_set = skip(&arr);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    /// Extracting Vec from Vec<T> always yields T.
    #[test]
    fn extract_vec_yields_inner(leaf in arb_primitive_type()) {
        let vec_ty: Type = parse_quote!(Vec<#leaf>);
        let (inner, extracted) = try_extract_inner_type(&vec_ty, "Vec", &empty_skip());
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&leaf));
    }

    /// Extracting Option from Option<T> always yields T.
    #[test]
    fn extract_option_yields_inner(leaf in arb_primitive_type()) {
        let opt_ty: Type = parse_quote!(Option<#leaf>);
        let (inner, extracted) = try_extract_inner_type(&opt_ty, "Option", &empty_skip());
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&leaf));
    }
}

// ===========================================================================
// Category 7: double wrap → correct nesting (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Double wrapping a leaf produces nested WithLeaf.
    #[test]
    fn double_wrap_leaf_nests(leaf in arb_primitive_type()) {
        let once = wrap_leaf_type(&leaf, &empty_skip());
        let twice = wrap_leaf_type(&once, &empty_skip());
        let s = ty_str(&twice);
        // Should contain two occurrences of WithLeaf
        let count = s.matches("WithLeaf").count();
        prop_assert!(count >= 2, "expected 2+ WithLeaf in: {s}");
    }

    /// Double wrapping still produces a parameterized type.
    #[test]
    fn double_wrap_still_parameterized(leaf in arb_primitive_type()) {
        let once = wrap_leaf_type(&leaf, &empty_skip());
        let twice = wrap_leaf_type(&once, &empty_skip());
        prop_assert!(is_parameterized(&twice));
    }

    /// Double wrapping a container in skip wraps inner twice.
    #[test]
    fn double_wrap_container_in_skip(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let skip_set = skip(&["Vec"]);
        let once = wrap_leaf_type(&ty, &skip_set);
        // First wrap: Vec<WithLeaf<T>>
        let s1 = ty_str(&once);
        prop_assert!(s1.starts_with("Vec <"), "got: {s1}");
        // We can't double-wrap the same way easily since inner is now WithLeaf
        // but wrapping the result with empty skip wraps the whole Vec.
        let twice = wrap_leaf_type(&once, &empty_skip());
        let s2 = ty_str(&twice);
        prop_assert!(s2.contains("WithLeaf"), "got: {s2}");
    }
}

// ===========================================================================
// Category 8: type equality is reflexive (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Type string of a type equals itself.
    #[test]
    fn type_string_reflexive(leaf in arb_primitive_type()) {
        prop_assert_eq!(ty_str(&leaf), ty_str(&leaf));
    }

    /// Cloned type has same string representation.
    #[test]
    fn cloned_type_equal_string(leaf in arb_primitive_type()) {
        let cloned = leaf.clone();
        prop_assert_eq!(ty_str(&leaf), ty_str(&cloned));
    }

    /// Wrapping the same type twice produces same string.
    #[test]
    fn wrap_deterministic(leaf in arb_primitive_type()) {
        let a = ty_str(&wrap_leaf_type(&leaf, &empty_skip()));
        let b = ty_str(&wrap_leaf_type(&leaf, &empty_skip()));
        prop_assert_eq!(a, b);
    }

    /// Filtering the same type twice produces same string.
    #[test]
    fn filter_deterministic(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let skip_set = skip(&["Box"]);
        let a = ty_str(&filter_inner_type(&ty, &skip_set));
        let b = ty_str(&filter_inner_type(&ty, &skip_set));
        prop_assert_eq!(a, b);
    }

    /// Extracting the same type twice produces same result.
    #[test]
    fn extract_deterministic(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let (a, ok_a) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        let (b, ok_b) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        prop_assert_eq!(ok_a, ok_b);
        prop_assert_eq!(ty_str(&a), ty_str(&b));
    }
}

// ===========================================================================
// Category 9: type string equality for same type (proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    /// Parsing a leaf name always produces the same string.
    #[test]
    fn parse_same_name_same_string(leaf in arb_leaf_name()) {
        let a = ty_str(&parse_ty(leaf));
        let b = ty_str(&parse_ty(leaf));
        prop_assert_eq!(a, b);
    }

    /// Wrapped type string starts with "adze" for leaves.
    #[test]
    fn wrapped_leaf_starts_with_adze(leaf in arb_leaf_name()) {
        let ty = parse_ty(leaf);
        let s = ty_str(&wrap_leaf_type(&ty, &empty_skip()));
        prop_assert!(s.starts_with("adze"), "expected adze prefix, got: {s}");
    }

    /// Extraction result string matches leaf when successful.
    #[test]
    fn extracted_string_matches_leaf(
        wrapper in arb_wrapper_name(),
        leaf in arb_leaf_name(),
    ) {
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, wrapper, &empty_skip());
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }
}

// ===========================================================================
// Category 10: Unit tests for specific edge cases
// ===========================================================================

#[test]
fn unit_extract_reference_type_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!extracted);
    assert_eq!(ty_str(&result), "& str");
}

#[test]
fn unit_extract_tuple_type_returns_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let (result, extracted) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!extracted);
    assert_eq!(ty_str(&result), "(i32 , u32)");
}

#[test]
fn unit_filter_reference_type_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "& str");
}

#[test]
fn unit_filter_tuple_type_returns_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "(i32 , u32)");
}

#[test]
fn unit_wrap_reference_type_wraps_entirely() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn unit_wrap_array_type_wraps_entirely() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn unit_extract_nested_vec_in_box_with_skip() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn unit_extract_nested_option_in_arc_with_skip() {
    let ty: Type = parse_quote!(Arc<Option<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn unit_filter_double_box() {
    let ty: Type = parse_quote!(Box<Box<String>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn unit_filter_box_arc_chain() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn unit_filter_arc_box_chain() {
    let ty: Type = parse_quote!(Arc<Box<String>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn unit_wrap_result_type_with_skip() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    let s = ty_str(&wrapped);
    assert!(s.contains("adze :: WithLeaf < String >"));
    assert!(s.contains("adze :: WithLeaf < i32 >"));
}

#[test]
fn unit_wrap_vec_with_vec_in_skip() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn unit_wrap_option_with_option_in_skip() {
    let ty: Type = parse_quote!(Option<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < bool > >");
}

#[test]
fn unit_extract_box_skip_no_inner_target() {
    // Box in skip, looking for Option, but inner is String (not Option).
    let ty: Type = parse_quote!(Box<String>);
    let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!extracted);
    assert_eq!(ty_str(&result), "Box < String >");
}

#[test]
fn unit_filter_empty_skip_preserves_box() {
    let ty: Type = parse_quote!(Box<String>);
    let result = filter_inner_type(&ty, &empty_skip());
    assert_eq!(ty_str(&result), "Box < String >");
}

#[test]
fn unit_wrap_i32_leaf() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn unit_wrap_string_leaf() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn unit_wrap_bool_leaf() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < bool >");
}

#[test]
fn unit_extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn unit_extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn unit_extract_box_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

// ===========================================================================
// Category 11+: Additional property combinations
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Filter with matching skip returns the inner type.
    #[test]
    fn filter_matching_skip_unwraps(
        wrapper in arb_wrapper_name(),
        leaf in arb_leaf_name(),
    ) {
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let arr = [wrapper];
        let skip_set = skip(&arr);
        let result = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(ty_str(&result), leaf);
    }

    /// Filter with non-matching skip preserves original.
    #[test]
    fn filter_nonmatching_skip_preserves(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let result = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(ty_str(&result), format!("Vec < {leaf} >"));
    }

    /// Extract and filter agree when using single wrapper.
    #[test]
    fn extract_and_filter_agree(
        wrapper in arb_wrapper_name(),
        leaf in arb_leaf_name(),
    ) {
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let (extracted_inner, ok) = try_extract_inner_type(&ty, wrapper, &empty_skip());
        let arr = [wrapper];
        let filtered = filter_inner_type(&ty, &skip(&arr));
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&extracted_inner), ty_str(&filtered));
    }

    /// Wrapping with skip containing the wrapper changes only inner.
    #[test]
    fn wrap_skip_wrapper_keeps_outer(
        wrapper in arb_wrapper_name(),
        leaf in arb_leaf_name(),
    ) {
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let arr = [wrapper];
        let wrapped = wrap_leaf_type(&ty, &skip(&arr));
        let s = ty_str(&wrapped);
        prop_assert!(s.starts_with(&format!("{wrapper} <")), "got: {s}");
        prop_assert!(s.contains("WithLeaf"), "inner should be wrapped: {s}");
    }

    /// Extract from nested skip layers reaches inner target.
    #[test]
    fn extract_through_nested_skip(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Box<Arc<Vec<{leaf}>>>"));
        let skip_set = skip(&["Box", "Arc"]);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    /// Wrapping a container NOT in skip wraps the entire thing.
    #[test]
    fn wrap_container_not_in_skip_wraps_entire(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        let s = ty_str(&wrapped);
        prop_assert!(s.starts_with("adze :: WithLeaf <"), "got: {s}");
    }

    /// Idempotent filter: filtering twice with same skip gives same result.
    #[test]
    fn filter_idempotent(leaf in arb_leaf_name()) {
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let skip_set = skip(&["Box"]);
        let once = filter_inner_type(&ty, &skip_set);
        let twice = filter_inner_type(&once, &skip_set);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    /// Extracting from a wrapped result: wrap leaf, then extract fails (wrapper is adze::WithLeaf).
    #[test]
    fn extract_from_wrapped_leaf_no_match(leaf in arb_primitive_type()) {
        let wrapped = wrap_leaf_type(&leaf, &empty_skip());
        let (_, extracted) = try_extract_inner_type(&wrapped, "Vec", &empty_skip());
        prop_assert!(!extracted);
    }
}

// ===========================================================================
// More unit tests for coverage
// ===========================================================================

#[test]
fn unit_extract_deeply_nested_through_multiple_skips() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<u64>>>>);
    let skip_set = skip(&["Box", "Arc", "Rc"]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_set);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn unit_filter_triple_nesting() {
    let ty: Type = parse_quote!(Box<Arc<Rc<String>>>);
    let skip_set = skip(&["Box", "Arc", "Rc"]);
    let result = filter_inner_type(&ty, &skip_set);
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn unit_wrap_nested_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    let s = ty_str(&wrapped);
    assert!(s.starts_with("Option <"));
    assert!(s.contains("Vec <"));
    assert!(s.contains("WithLeaf"));
}

#[test]
fn unit_extract_plain_i32_returns_false() {
    let ty: Type = parse_quote!(i32);
    let (result, extracted) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!extracted);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn unit_extract_plain_string_returns_false() {
    let ty: Type = parse_quote!(String);
    let (result, extracted) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!extracted);
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn unit_wrap_custom_type() {
    let ty: Type = parse_quote!(MyCustomType);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < MyCustomType >");
}

#[test]
fn unit_extract_custom_wrapper() {
    let ty: Type = parse_quote!(MyWrapper<i64>);
    let (inner, extracted) = try_extract_inner_type(&ty, "MyWrapper", &empty_skip());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i64");
}

#[test]
fn unit_filter_preserves_non_skip_wrapper() {
    let ty: Type = parse_quote!(Vec<String>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Vec < String >");
}

#[test]
fn unit_wrap_f64_leaf() {
    let ty: Type = parse_quote!(f64);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn unit_wrap_usize_leaf() {
    let ty: Type = parse_quote!(usize);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < usize >");
}

#[test]
fn unit_extract_vec_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn unit_extract_option_f32() {
    let ty: Type = parse_quote!(Option<f32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn unit_filter_box_preserves_vec_inner() {
    // filter with skip=["Box"] on Box<Vec<i32>> should yield Vec<i32>
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

#[test]
fn unit_wrap_box_not_in_skip() {
    let ty: Type = parse_quote!(Box<i32>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Box < i32 > >");
}

#[test]
fn unit_wrap_box_in_skip_wraps_inner() {
    let ty: Type = parse_quote!(Box<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&wrapped), "Box < adze :: WithLeaf < i32 > >");
}

#[test]
fn unit_extract_rc_through_arc_skip() {
    let ty: Type = parse_quote!(Arc<Rc<u32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Rc", &skip(&["Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn unit_filter_single_layer_returns_leaf() {
    let ty: Type = parse_quote!(Rc<char>);
    let result = filter_inner_type(&ty, &skip(&["Rc"]));
    assert_eq!(ty_str(&result), "char");
}
