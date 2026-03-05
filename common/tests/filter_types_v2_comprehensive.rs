//! Comprehensive tests for adze-common type filtering and extraction utilities.
//!
//! Covers `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! across Option, Vec, Box, nested types, and edge cases.

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{self, Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip_box() -> HashSet<&'static str> {
    HashSet::from(["Box"])
}

fn skip_box_arc() -> HashSet<&'static str> {
    HashSet::from(["Box", "Arc"])
}

fn skip_option() -> HashSet<&'static str> {
    HashSet::from(["Option"])
}

fn skip_vec_option() -> HashSet<&'static str> {
    HashSet::from(["Vec", "Option"])
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. Extract from Option  (8 tests)
// ===========================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
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
fn extract_option_vec_u8() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < u8 >");
}

#[test]
fn extract_option_box_t() {
    let ty: Type = parse_quote!(Option<Box<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_nested_option_via_skip() {
    // Box<Option<String>> — skip Box, extract Option
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_box());
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_none_when_vec() {
    // Looking for Option but type is Vec<String>
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn extract_option_none_when_plain() {
    let ty: Type = parse_quote!(String);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
}

#[test]
fn extract_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

// ===========================================================================
// 2. Extract from Vec  (8 tests)
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_vec_option_t() {
    let ty: Type = parse_quote!(Vec<Option<i64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < i64 >");
}

#[test]
fn extract_nested_vec_via_skip() {
    // Box<Vec<u32>> — skip Box, extract Vec
    let ty: Type = parse_quote!(Box<Vec<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_box());
    assert!(ok);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn extract_vec_of_vec() {
    let ty: Type = parse_quote!(Vec<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < u8 >");
}

#[test]
fn extract_vec_none_when_option() {
    let ty: Type = parse_quote!(Option<String>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok);
}

#[test]
fn extract_vec_none_when_plain() {
    let ty: Type = parse_quote!(usize);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok);
}

#[test]
fn extract_vec_f64() {
    let ty: Type = parse_quote!(Vec<f64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

// ===========================================================================
// 3. Extract from Box  (8 tests)
// ===========================================================================

#[test]
fn extract_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_box_i32() {
    let ty: Type = parse_quote!(Box<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_box_vec_t() {
    let ty: Type = parse_quote!(Box<Vec<u16>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < u16 >");
}

#[test]
fn extract_box_of_box() {
    let ty: Type = parse_quote!(Box<Box<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_box_via_arc_skip() {
    // Arc<Box<f32>> — skip Arc, extract Box
    let skip: HashSet<&str> = HashSet::from(["Arc"]);
    let ty: Type = parse_quote!(Arc<Box<f32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn extract_box_none_when_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (_, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(!ok);
}

#[test]
fn extract_box_none_when_plain() {
    let ty: Type = parse_quote!(char);
    let (_, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(!ok);
}

#[test]
fn extract_box_option_inner() {
    let ty: Type = parse_quote!(Box<Option<u64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < u64 >");
}

// ===========================================================================
// 4. filter_inner_type  (8 tests)
// ===========================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip_box());
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_box_arc_string() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip_box_arc());
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_arc_box_i32() {
    let ty: Type = parse_quote!(Arc<Box<i32>>);
    let filtered = filter_inner_type(&ty, &skip_box_arc());
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_plain_type_unchanged() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip_box());
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_option_not_in_skip() {
    // Option is not in skip_box set, so it stays
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &skip_box());
    assert_eq!(ty_str(&filtered), "Option < String >");
}

#[test]
fn filter_vec_not_in_skip() {
    let ty: Type = parse_quote!(Vec<u8>);
    let filtered = filter_inner_type(&ty, &skip_box());
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
}

#[test]
fn filter_empty_skip_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &empty_skip());
    assert_eq!(ty_str(&filtered), "Box < String >");
}

#[test]
fn filter_triple_nested() {
    let ty: Type = parse_quote!(Box<Arc<Box<u64>>>);
    let filtered = filter_inner_type(&ty, &skip_box_arc());
    assert_eq!(ty_str(&filtered), "u64");
}

// ===========================================================================
// 5. wrap_leaf_type  (8 tests)
// ===========================================================================

#[test]
fn wrap_string_leaf() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_i32_leaf() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_vec_skip_wraps_inner() {
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_skip_wraps_inner() {
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_box_not_skipped_wraps_whole() {
    let ty: Type = parse_quote!(Box<String>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Box < String > >");
}

#[test]
fn wrap_nested_skip_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let wrapped = wrap_leaf_type(&ty, &skip_vec_option());
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wrap_option_vec_nested() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let wrapped = wrap_leaf_type(&ty, &skip_vec_option());
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < u8 > > >"
    );
}

#[test]
fn wrap_result_with_both_args_skipped() {
    let skip: HashSet<&str> = HashSet::from(["Result"]);
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

// ===========================================================================
// 6. Parameterized / generic detection via try_extract  (8 tests)
// ===========================================================================
// The crate doesn't expose `is_parameterized` directly, but we can validate
// parameterized vs non-parameterized behavior through extraction results.

#[test]
fn parameterized_option_detected() {
    let ty: Type = parse_quote!(Option<String>);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok, "Option<String> should be detected as Option");
}

#[test]
fn parameterized_vec_detected() {
    let ty: Type = parse_quote!(Vec<u32>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok, "Vec<u32> should be detected as Vec");
}

#[test]
fn non_parameterized_i32() {
    let ty: Type = parse_quote!(i32);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok, "i32 is not Option");
}

#[test]
fn non_parameterized_string() {
    let ty: Type = parse_quote!(String);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok, "String is not Vec");
}

#[test]
fn parameterized_box_detected() {
    let ty: Type = parse_quote!(Box<char>);
    let (_, ok) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(ok, "Box<char> should be detected as Box");
}

#[test]
fn custom_generic_detected() {
    let ty: Type = parse_quote!(MyWrapper<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "MyWrapper", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn hashmap_not_detected_as_option() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
}

#[test]
fn custom_generic_in_skip_set() {
    let skip: HashSet<&str> = HashSet::from(["Wrapper"]);
    let ty: Type = parse_quote!(Wrapper<Option<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

// ===========================================================================
// 7. Edge cases  (8 tests)
// ===========================================================================

#[test]
fn edge_deeply_nested_extraction() {
    // Arc<Box<Option<Vec<u8>>>> — skip Arc and Box, extract Option
    let skip: HashSet<&str> = HashSet::from(["Arc", "Box"]);
    let ty: Type = parse_quote!(Arc<Box<Option<Vec<u8>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < u8 >");
}

#[test]
fn edge_reference_type_not_extracted() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn edge_tuple_type_not_extracted() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(i32 , u32)");
}

#[test]
fn edge_unit_type_not_extracted() {
    let ty: Type = parse_quote!(());
    let (_, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!ok);
}

#[test]
fn edge_filter_reference_unchanged() {
    let ty: Type = parse_quote!(&mut Vec<u8>);
    let filtered = filter_inner_type(&ty, &skip_box());
    assert_eq!(ty_str(&filtered), "& mut Vec < u8 >");
}

#[test]
fn edge_filter_tuple_unchanged() {
    let ty: Type = parse_quote!((String, i32));
    let filtered = filter_inner_type(&ty, &skip_box_arc());
    assert_eq!(ty_str(&filtered), "(String , i32)");
}

#[test]
fn edge_wrap_reference_wraps_whole() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn edge_wrap_tuple_wraps_whole() {
    let ty: Type = parse_quote!((i32, u64));
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (i32 , u64) >");
}

// ===========================================================================
// 8. Composability / round-trip  (7 bonus tests)
// ===========================================================================

#[test]
fn compose_extract_then_wrap() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn compose_filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    let filtered = filter_inner_type(&ty, &skip_box_arc());
    assert_eq!(ty_str(&filtered), "i32");
    let wrapped = wrap_leaf_type(&filtered, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn compose_extract_vec_then_filter_box() {
    let ty: Type = parse_quote!(Vec<Box<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(ok);
    let filtered = filter_inner_type(&inner, &skip_box());
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn extract_does_not_match_wrong_target() {
    // Ensure extracting "Vec" from Option<Vec<u8>> returns the whole thing (no skip set)
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Option < Vec < u8 > >");
}

#[test]
fn extract_with_skip_reaches_inner_vec() {
    // With Option in skip set, we can reach Vec inside Option<Vec<u8>>
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_option());
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn filter_idempotent_on_leaf() {
    let ty: Type = parse_quote!(u64);
    let filtered = filter_inner_type(&ty, &skip_box_arc());
    assert_eq!(ty_str(&filtered), "u64");
    let filtered2 = filter_inner_type(&filtered, &skip_box_arc());
    assert_eq!(ty_str(&filtered2), "u64");
}

#[test]
fn wrap_idempotent_check() {
    // Wrapping an already-wrapped type wraps again (no special detection)
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    let double = wrap_leaf_type(&wrapped, &empty_skip());
    assert_eq!(
        ty_str(&double),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}
