//! Comprehensive tests for type wrapping, leaf type generation, and complex
//! type analysis in `adze-common`.
//!
//! Covers `wrap_leaf_type`, `try_extract_inner_type`, `filter_inner_type`, and
//! a local `is_parameterized` helper across 8 categories with 60+ tests.

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{self, Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
// 1. Wrapping primitive types (10 tests)
// ===========================================================================

#[test]
fn wrap_prim_u8() {
    let ty: Type = parse_quote!(u8);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < u8 >");
}

#[test]
fn wrap_prim_u16() {
    let ty: Type = parse_quote!(u16);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < u16 >");
}

#[test]
fn wrap_prim_u32() {
    let ty: Type = parse_quote!(u32);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < u32 >");
}

#[test]
fn wrap_prim_u64() {
    let ty: Type = parse_quote!(u64);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < u64 >");
}

#[test]
fn wrap_prim_i32() {
    let ty: Type = parse_quote!(i32);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_prim_f64() {
    let ty: Type = parse_quote!(f64);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < f64 >");
}

#[test]
fn wrap_prim_bool() {
    let ty: Type = parse_quote!(bool);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_prim_char() {
    let ty: Type = parse_quote!(char);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < char >");
}

#[test]
fn wrap_prim_string() {
    let ty: Type = parse_quote!(String);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_prim_usize() {
    let ty: Type = parse_quote!(usize);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < usize >");
}

// ===========================================================================
// 2. Wrapping container types (8 tests)
// ===========================================================================

#[test]
fn wrap_option_skipped_wraps_inner() {
    let ty: Type = parse_quote!(Option<i32>);
    let w = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&w), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_vec_skipped_wraps_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&w), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_box_skipped_wraps_inner() {
    let ty: Type = parse_quote!(Box<u64>);
    let w = wrap_leaf_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&w), "Box < adze :: WithLeaf < u64 > >");
}

#[test]
fn wrap_option_not_skipped_wraps_whole() {
    let ty: Type = parse_quote!(Option<i32>);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < Option < i32 > >");
}

#[test]
fn wrap_vec_not_skipped_wraps_whole() {
    let ty: Type = parse_quote!(Vec<u8>);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < Vec < u8 > >");
}

#[test]
fn wrap_box_not_skipped_wraps_whole() {
    let ty: Type = parse_quote!(Box<bool>);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < Box < bool > >");
}

#[test]
fn wrap_result_skipped_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let w = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&w),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_arc_skipped_wraps_inner() {
    let ty: Type = parse_quote!(Arc<f64>);
    let w = wrap_leaf_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&w), "Arc < adze :: WithLeaf < f64 > >");
}

// ===========================================================================
// 3. Wrapping nested containers (8 tests)
// ===========================================================================

#[test]
fn wrap_option_vec_both_skipped() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let w = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(ty_str(&w), "Option < Vec < adze :: WithLeaf < u8 > > >");
}

#[test]
fn wrap_vec_option_both_skipped() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&w), "Vec < Option < adze :: WithLeaf < String > > >");
}

#[test]
fn wrap_option_vec_only_option_skipped() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let w = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&w), "Option < adze :: WithLeaf < Vec < u8 > > >");
}

#[test]
fn wrap_vec_option_only_vec_skipped() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&w), "Vec < adze :: WithLeaf < Option < i32 > > >");
}

#[test]
fn wrap_box_vec_both_skipped() {
    let ty: Type = parse_quote!(Box<Vec<bool>>);
    let w = wrap_leaf_type(&ty, &skip(&["Box", "Vec"]));
    assert_eq!(ty_str(&w), "Box < Vec < adze :: WithLeaf < bool > > >");
}

#[test]
fn wrap_box_option_vec_all_skipped() {
    let ty: Type = parse_quote!(Box<Option<Vec<f64>>>);
    let w = wrap_leaf_type(&ty, &skip(&["Box", "Option", "Vec"]));
    assert_eq!(
        ty_str(&w),
        "Box < Option < Vec < adze :: WithLeaf < f64 > > > >"
    );
}

#[test]
fn wrap_vec_vec_both_skipped() {
    let ty: Type = parse_quote!(Vec<Vec<char>>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&w), "Vec < Vec < adze :: WithLeaf < char > > >");
}

#[test]
fn wrap_option_option_both_skipped() {
    let ty: Type = parse_quote!(Option<Option<u32>>);
    let w = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&w), "Option < Option < adze :: WithLeaf < u32 > > >");
}

// ===========================================================================
// 4. TokenStream output validity — parses back (8 tests)
// ===========================================================================

fn roundtrip_wrap(src: &str, skips: &[&str]) {
    let ty: Type = syn::parse_str(src).expect("parse source type");
    let wrapped = wrap_leaf_type(&ty, &skip(skips));
    let ts = wrapped.to_token_stream().to_string();
    syn::parse_str::<Type>(&ts).expect("wrapped output must parse back as Type");
}

#[test]
fn roundtrip_wrap_i32() {
    roundtrip_wrap("i32", &[]);
}

#[test]
fn roundtrip_wrap_string() {
    roundtrip_wrap("String", &[]);
}

#[test]
fn roundtrip_wrap_option_u8() {
    roundtrip_wrap("Option<u8>", &["Option"]);
}

#[test]
fn roundtrip_wrap_vec_string() {
    roundtrip_wrap("Vec<String>", &["Vec"]);
}

#[test]
fn roundtrip_wrap_nested_option_vec() {
    roundtrip_wrap("Option<Vec<bool>>", &["Option", "Vec"]);
}

#[test]
fn roundtrip_wrap_result() {
    roundtrip_wrap("Result<String, i32>", &["Result"]);
}

#[test]
fn roundtrip_wrap_array() {
    roundtrip_wrap("[u8; 4]", &[]);
}

#[test]
fn roundtrip_wrap_tuple() {
    roundtrip_wrap("(i32, u64)", &[]);
}

// ===========================================================================
// 5. Wrapping preserves type information (8 tests)
// ===========================================================================

#[test]
fn preserve_wrap_contains_original_simple() {
    let ty: Type = parse_quote!(u32);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    let s = ty_str(&w);
    assert!(
        s.contains("u32"),
        "wrapped output must contain original type"
    );
}

#[test]
fn preserve_wrap_contains_original_string() {
    let ty: Type = parse_quote!(String);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    let s = ty_str(&w);
    assert!(s.contains("String"));
}

#[test]
fn preserve_wrap_vec_keeps_vec_and_inner() {
    let ty: Type = parse_quote!(Vec<i64>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec"]));
    let s = ty_str(&w);
    assert!(s.contains("Vec"));
    assert!(s.contains("i64"));
}

#[test]
fn preserve_wrap_option_keeps_option_and_inner() {
    let ty: Type = parse_quote!(Option<bool>);
    let w = wrap_leaf_type(&ty, &skip(&["Option"]));
    let s = ty_str(&w);
    assert!(s.contains("Option"));
    assert!(s.contains("bool"));
}

#[test]
fn preserve_wrap_always_adds_with_leaf_for_leaf() {
    let ty: Type = parse_quote!(f32);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert!(ty_str(&w).contains("WithLeaf"));
}

#[test]
fn preserve_wrap_nested_keeps_all_layers() {
    let ty: Type = parse_quote!(Vec<Option<u8>>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    let s = ty_str(&w);
    assert!(s.contains("Vec"));
    assert!(s.contains("Option"));
    assert!(s.contains("u8"));
    assert!(s.contains("WithLeaf"));
}

#[test]
fn preserve_result_both_type_args() {
    let ty: Type = parse_quote!(Result<String, u32>);
    let w = wrap_leaf_type(&ty, &skip(&["Result"]));
    let s = ty_str(&w);
    assert!(s.contains("String"));
    assert!(s.contains("u32"));
}

#[test]
fn preserve_wrap_does_not_lose_path_prefix() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec"]));
    let s = ty_str(&w);
    assert!(s.contains("std"));
    assert!(s.contains("vec"));
}

// ===========================================================================
// 6. Interaction between wrap and extract (8 tests)
// ===========================================================================

#[test]
fn extract_then_wrap_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn extract_then_wrap_option_u32() {
    let ty: Type = parse_quote!(Option<u32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u32 >");
}

#[test]
fn filter_then_wrap_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_wrap_box_arc_i32() {
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn extract_skip_box_then_wrap() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u8 >");
}

#[test]
fn wrap_then_check_parameterized() {
    let ty: Type = parse_quote!(i32);
    assert!(!is_parameterized(&ty));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    // adze::WithLeaf<i32> — last segment has angle brackets
    assert!(is_parameterized(&wrapped));
}

#[test]
fn filter_noop_then_wrap() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn extract_miss_then_wrap_original() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (_inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

// ===========================================================================
// 7. Complex types: tuples, references, arrays, function pointers (8 tests)
// ===========================================================================

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, u64));
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < (i32 , u64) >");
}

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_mutable_reference() {
    let ty: Type = parse_quote!(&mut i32);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < & mut i32 >");
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 16]);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < [u8 ; 16] >");
}

#[test]
fn wrap_slice_reference() {
    let ty: Type = parse_quote!(&[u8]);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < & [u8] >");
}

#[test]
fn wrap_fn_pointer() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < fn (i32) -> bool >");
}

#[test]
fn wrap_triple_tuple() {
    let ty: Type = parse_quote!((bool, u8, f64));
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < (bool , u8 , f64) >");
}

#[test]
fn is_parameterized_fn_pointer_false() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 8. Edge cases: never type, unit type, empty path, qualified paths (10 tests)
// ===========================================================================

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < () >");
}

#[test]
fn wrap_never_type() {
    let ty: Type = parse_quote!(!);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < ! >");
}

#[test]
fn wrap_qualified_path_not_skipped() {
    let ty: Type = parse_quote!(std::string::String);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&w), "adze :: WithLeaf < std :: string :: String >");
}

#[test]
fn wrap_qualified_vec_skipped() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec"]));
    let s = ty_str(&w);
    assert!(s.contains("WithLeaf"));
    assert!(s.contains("u8"));
}

#[test]
fn extract_non_path_never_type() {
    let ty: Type = parse_quote!(!);
    let (_inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
}

#[test]
fn filter_non_path_unit_type() {
    let ty: Type = parse_quote!(());
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "()");
}

#[test]
fn is_parameterized_unit_false() {
    let ty: Type = parse_quote!(());
    assert!(!is_parameterized(&ty));
}

#[test]
fn is_parameterized_never_false() {
    let ty: Type = parse_quote!(!);
    assert!(!is_parameterized(&ty));
}

#[test]
fn is_parameterized_qualified_generic_true() {
    let ty: Type = parse_quote!(std::option::Option<u8>);
    assert!(is_parameterized(&ty));
}

#[test]
fn is_parameterized_crate_path_no_generics_false() {
    let ty: Type = parse_quote!(crate::module::MyType);
    assert!(!is_parameterized(&ty));
}
