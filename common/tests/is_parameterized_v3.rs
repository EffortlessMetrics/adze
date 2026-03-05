//! Comprehensive tests for type detection utilities in adze-common.
//!
//! Tests cover `try_extract_inner_type`, `filter_inner_type`, `wrap_leaf_type`,
//! and a local `is_parameterized` helper that detects whether a `syn::Type`
//! carries generic parameters (i.e. is a `Type::Path` whose last segment has
//! angle-bracketed arguments).

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{self, Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `true` when the type is a path type whose last segment carries
/// angle-bracketed generic arguments — the same heuristic the adze pipeline
/// uses to decide whether a type is "parameterized".
fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. Simple types — NOT parameterized  (8 tests)
// ===========================================================================

#[test]
fn simple_i32_not_parameterized() {
    let ty: Type = parse_quote!(i32);
    assert!(!is_parameterized(&ty));
}

#[test]
fn simple_string_not_parameterized() {
    let ty: Type = parse_quote!(String);
    assert!(!is_parameterized(&ty));
}

#[test]
fn simple_bool_not_parameterized() {
    let ty: Type = parse_quote!(bool);
    assert!(!is_parameterized(&ty));
}

#[test]
fn simple_u8_not_parameterized() {
    let ty: Type = parse_quote!(u8);
    assert!(!is_parameterized(&ty));
}

#[test]
fn simple_f64_not_parameterized() {
    let ty: Type = parse_quote!(f64);
    assert!(!is_parameterized(&ty));
}

#[test]
fn simple_usize_not_parameterized() {
    let ty: Type = parse_quote!(usize);
    assert!(!is_parameterized(&ty));
}

#[test]
fn simple_char_not_parameterized() {
    let ty: Type = parse_quote!(char);
    assert!(!is_parameterized(&ty));
}

#[test]
fn simple_unit_tuple_not_parameterized() {
    // `()` parses as Type::Tuple, not Type::Path — cannot be parameterized.
    let ty: Type = parse_quote!(());
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 2. Generic types — ARE parameterized  (8 tests)
// ===========================================================================

#[test]
fn generic_option_is_parameterized() {
    let ty: Type = parse_quote!(Option<i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn generic_vec_is_parameterized() {
    let ty: Type = parse_quote!(Vec<String>);
    assert!(is_parameterized(&ty));
}

#[test]
fn generic_box_is_parameterized() {
    let ty: Type = parse_quote!(Box<u8>);
    assert!(is_parameterized(&ty));
}

#[test]
fn generic_hashmap_is_parameterized() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn generic_result_is_parameterized() {
    let ty: Type = parse_quote!(Result<(), String>);
    assert!(is_parameterized(&ty));
}

#[test]
fn generic_arc_is_parameterized() {
    let ty: Type = parse_quote!(Arc<String>);
    assert!(is_parameterized(&ty));
}

#[test]
fn generic_rc_is_parameterized() {
    let ty: Type = parse_quote!(Rc<bool>);
    assert!(is_parameterized(&ty));
}

#[test]
fn generic_btreemap_is_parameterized() {
    let ty: Type = parse_quote!(BTreeMap<u32, Vec<u8>>);
    assert!(is_parameterized(&ty));
}

// ===========================================================================
// 3. Nested generics — parameterized  (8 tests)
// ===========================================================================

#[test]
fn nested_vec_option_is_parameterized() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_option_vec_is_parameterized() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_box_vec_option_is_parameterized() {
    let ty: Type = parse_quote!(Box<Vec<Option<u8>>>);
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_result_option_is_parameterized() {
    let ty: Type = parse_quote!(Result<Option<i32>, String>);
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_option_option_is_parameterized() {
    let ty: Type = parse_quote!(Option<Option<bool>>);
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_vec_vec_is_parameterized() {
    let ty: Type = parse_quote!(Vec<Vec<f64>>);
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_hashmap_vec_is_parameterized() {
    let ty: Type = parse_quote!(HashMap<String, Vec<u32>>);
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_arc_box_is_parameterized() {
    let ty: Type = parse_quote!(Arc<Box<String>>);
    assert!(is_parameterized(&ty));
}

// ===========================================================================
// 4. Reference types — not parameterized  (7 tests)
// ===========================================================================

#[test]
fn ref_str_not_parameterized() {
    let ty: Type = parse_quote!(&str);
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_mut_i32_not_parameterized() {
    let ty: Type = parse_quote!(&mut i32);
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_slice_u8_not_parameterized() {
    let ty: Type = parse_quote!(&[u8]);
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_lifetime_str_not_parameterized() {
    let ty: Type = parse_quote!(&'a str);
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_lifetime_mut_not_parameterized() {
    let ty: Type = parse_quote!(&'a mut u32);
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_option_inner_parameterized_but_ref_is_not() {
    // The outermost type is a reference — `is_parameterized` returns false.
    let ty: Type = parse_quote!(&Option<i32>);
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_vec_inner_parameterized_but_ref_is_not() {
    let ty: Type = parse_quote!(&Vec<u8>);
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 5. Tuple types  (8 tests)
// ===========================================================================

#[test]
fn tuple_pair_not_parameterized() {
    let ty: Type = parse_quote!((i32, String));
    assert!(!is_parameterized(&ty));
}

#[test]
fn tuple_triple_not_parameterized() {
    let ty: Type = parse_quote!((bool, u8, f64));
    assert!(!is_parameterized(&ty));
}

#[test]
fn tuple_unit_not_parameterized() {
    let ty: Type = parse_quote!(());
    assert!(!is_parameterized(&ty));
}

#[test]
fn tuple_single_not_parameterized() {
    let ty: Type = parse_quote!((i32,));
    assert!(!is_parameterized(&ty));
}

#[test]
fn tuple_nested_option_not_parameterized_at_top() {
    let ty: Type = parse_quote!((Option<i32>, String));
    assert!(!is_parameterized(&ty));
}

#[test]
fn tuple_four_elements() {
    let ty: Type = parse_quote!((u8, u16, u32, u64));
    assert!(!is_parameterized(&ty));
}

#[test]
fn tuple_with_ref() {
    let ty: Type = parse_quote!((&str, i32));
    assert!(!is_parameterized(&ty));
}

#[test]
fn tuple_with_bool_pair() {
    let ty: Type = parse_quote!((bool, bool));
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 6. Array types  (8 tests)
// ===========================================================================

#[test]
fn array_u8_not_parameterized() {
    let ty: Type = parse_quote!([u8; 4]);
    assert!(!is_parameterized(&ty));
}

#[test]
fn array_string_not_parameterized() {
    let ty: Type = parse_quote!([String; 10]);
    assert!(!is_parameterized(&ty));
}

#[test]
fn array_bool_not_parameterized() {
    let ty: Type = parse_quote!([bool; 1]);
    assert!(!is_parameterized(&ty));
}

#[test]
fn array_f64_not_parameterized() {
    let ty: Type = parse_quote!([f64; 3]);
    assert!(!is_parameterized(&ty));
}

#[test]
fn array_i32_large_not_parameterized() {
    let ty: Type = parse_quote!([i32; 256]);
    assert!(!is_parameterized(&ty));
}

#[test]
fn array_usize_not_parameterized() {
    let ty: Type = parse_quote!([usize; 2]);
    assert!(!is_parameterized(&ty));
}

#[test]
fn array_char_not_parameterized() {
    let ty: Type = parse_quote!([char; 8]);
    assert!(!is_parameterized(&ty));
}

#[test]
fn array_u16_not_parameterized() {
    let ty: Type = parse_quote!([u16; 64]);
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 7. Path types  (8 tests)
// ===========================================================================

#[test]
fn qualified_vec_is_parameterized() {
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn qualified_option_is_parameterized() {
    let ty: Type = parse_quote!(std::option::Option<u8>);
    assert!(is_parameterized(&ty));
}

#[test]
fn crate_my_type_not_parameterized() {
    let ty: Type = parse_quote!(crate::MyType);
    assert!(!is_parameterized(&ty));
}

#[test]
fn self_type_not_parameterized() {
    let ty: Type = parse_quote!(Self);
    assert!(!is_parameterized(&ty));
}

#[test]
fn module_path_not_parameterized() {
    let ty: Type = parse_quote!(foo::bar::Baz);
    assert!(!is_parameterized(&ty));
}

#[test]
fn qualified_hashmap_is_parameterized() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn crate_generic_type_is_parameterized() {
    let ty: Type = parse_quote!(crate::Wrapper<u64>);
    assert!(is_parameterized(&ty));
}

#[test]
fn super_path_not_parameterized() {
    let ty: Type = parse_quote!(super::Item);
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 8. try_extract_inner_type  (5 tests)
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_skips_box() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_miss_returns_original() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_ref_type_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

// ===========================================================================
// 9. filter_inner_type  (5 tests)
// ===========================================================================

#[test]
fn filter_strips_box() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_strips_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_no_match_returns_original() {
    let ty: Type = parse_quote!(Vec<u8>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
}

#[test]
fn filter_empty_skip_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), "Box < String >");
}

#[test]
fn filter_non_path_returns_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(i32 , u32)");
}

// ===========================================================================
// 10. wrap_leaf_type  (5 tests)
// ===========================================================================

#[test]
fn wrap_simple_type() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_vec_wraps_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_wraps_inner() {
    let ty: Type = parse_quote!(Option<u32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < u32 > >");
}

#[test]
fn wrap_non_path_wraps_entirely() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_result_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}
