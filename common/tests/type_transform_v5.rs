//! Tests for type transformations, wrappers, and conversions in adze-common (v5).
//!
//! 64 tests across 8 categories (8 each):
//!   1. extract_vec_*       — extract inner type from Vec<T>
//!   2. extract_option_*    — extract inner type from Option<T>
//!   3. extract_box_*       — extract inner type from Box<T>
//!   4. extract_nested_*    — extraction from nested wrappers
//!   5. filter_type_*       — filter_inner_type operations
//!   6. wrap_leaf_*         — wrap_leaf_type operations
//!   7. parameterized_*     — is_parameterized checks (local helper)
//!   8. transform_edge_*    — edge cases (unit, references, tuples, arrays)

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{self, Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

/// Local parameterization check — mirrors the pipeline heuristic for detecting
/// whether a type has angle-bracketed generic parameters.
fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

// ===========================================================================
// 1. extract_vec — extract inner from Vec<T>  (8 tests)
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_vec_i64() {
    let ty: Type = parse_quote!(Vec<i64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i64");
}

#[test]
fn extract_vec_bool() {
    let ty: Type = parse_quote!(Vec<bool>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_vec_custom_struct() {
    let ty: Type = parse_quote!(Vec<MyStruct>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "MyStruct");
}

#[test]
fn extract_vec_tuple_inner() {
    let ty: Type = parse_quote!(Vec<(i32, u32)>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "(i32 , u32)");
}

#[test]
fn extract_vec_mismatch_returns_original() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

#[test]
fn extract_vec_with_option_inner() {
    let ty: Type = parse_quote!(Vec<Option<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < u32 >");
}

// ===========================================================================
// 2. extract_option — extract inner from Option<T>  (8 tests)
// ===========================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
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
fn extract_option_f64() {
    let ty: Type = parse_quote!(Option<f64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_option_vec_inner() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < u8 >");
}

#[test]
fn extract_option_box_inner() {
    let ty: Type = parse_quote!(Option<Box<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_option_usize() {
    let ty: Type = parse_quote!(Option<usize>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn extract_option_mismatch_on_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn extract_option_custom_type() {
    let ty: Type = parse_quote!(Option<TokenStream>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "TokenStream");
}

// ===========================================================================
// 3. extract_box — extract inner from Box<T>  (8 tests)
// ===========================================================================

#[test]
fn extract_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_box_i32() {
    let ty: Type = parse_quote!(Box<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_box_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_box_vec_inner() {
    let ty: Type = parse_quote!(Box<Vec<f32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < f32 >");
}

#[test]
fn extract_box_option_inner() {
    let ty: Type = parse_quote!(Box<Option<u64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < u64 >");
}

#[test]
fn extract_box_custom() {
    let ty: Type = parse_quote!(Box<MyParser>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "MyParser");
}

#[test]
fn extract_box_mismatch_on_arc() {
    let ty: Type = parse_quote!(Arc<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Arc < String >");
}

#[test]
fn extract_box_unit_inner() {
    let ty: Type = parse_quote!(Box<()>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "()");
}

// ===========================================================================
// 4. extract_nested — extraction from nested wrappers  (8 tests)
// ===========================================================================

#[test]
fn extract_nested_box_wrapping_vec() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_nested_arc_wrapping_option() {
    let ty: Type = parse_quote!(Arc<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_nested_box_arc_wrapping_vec() {
    let ty: Type = parse_quote!(Box<Arc<Vec<u8>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_nested_skip_no_target_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_nested_double_box_first_layer() {
    let ty: Type = parse_quote!(Box<Box<u16>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < u16 >");
}

#[test]
fn extract_nested_skip_box_find_option() {
    let ty: Type = parse_quote!(Box<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_nested_skip_irrelevant_no_match() {
    // Skip set contains "Rc" but type is Box<Vec<i32>>; Box is not skipped so
    // we look for Option at the top level only — mismatch.
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Rc"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < Vec < i32 > >");
}

#[test]
fn extract_nested_arc_wrapping_result() {
    let ty: Type = parse_quote!(Arc<Result<String, i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

// ===========================================================================
// 5. filter_type — filter_inner_type operations  (8 tests)
// ===========================================================================

#[test]
fn filter_type_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_type_arc_i32() {
    let ty: Type = parse_quote!(Arc<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_type_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_type_no_match_returns_original() {
    let ty: Type = parse_quote!(Vec<u8>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
}

#[test]
fn filter_type_empty_skip_set() {
    let ty: Type = parse_quote!(Box<bool>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), "Box < bool >");
}

#[test]
fn filter_type_plain_ident_unchanged() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_type_triple_nesting() {
    let ty: Type = parse_quote!(Box<Arc<Rc<f64>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ty_str(&filtered), "f64");
}

#[test]
fn filter_type_only_outer_matched() {
    // Only Box is in the skip set; Arc is not, so we stop at Arc.
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Arc < String >");
}

// ===========================================================================
// 6. wrap_leaf — wrap_leaf_type operations  (8 tests)
// ===========================================================================

#[test]
fn wrap_leaf_plain_string() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_leaf_plain_i32() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_leaf_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_leaf_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < bool > >");
}

#[test]
fn wrap_leaf_vec_option_nested() {
    let ty: Type = parse_quote!(Vec<Option<u8>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < u8 > > >"
    );
}

#[test]
fn wrap_leaf_not_in_skip_wraps_entire() {
    let ty: Type = parse_quote!(Vec<String>);
    // Vec is NOT in skip set, so the entire Vec<String> is wrapped.
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < String > >");
}

#[test]
fn wrap_leaf_result_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_leaf_custom_type() {
    let ty: Type = parse_quote!(MyNode);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < MyNode >");
}

// ===========================================================================
// 7. parameterized — is_parameterized checks  (8 tests)
// ===========================================================================

#[test]
fn parameterized_vec_string_is_true() {
    let ty: Type = parse_quote!(Vec<String>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_option_i32_is_true() {
    let ty: Type = parse_quote!(Option<i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_plain_string_is_false() {
    let ty: Type = parse_quote!(String);
    assert!(!is_parameterized(&ty));
}

#[test]
fn parameterized_plain_u8_is_false() {
    let ty: Type = parse_quote!(u8);
    assert!(!is_parameterized(&ty));
}

#[test]
fn parameterized_box_bool_is_true() {
    let ty: Type = parse_quote!(Box<bool>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_hashmap_is_true() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_reference_is_false() {
    let ty: Type = parse_quote!(&str);
    assert!(!is_parameterized(&ty));
}

#[test]
fn parameterized_tuple_is_false() {
    let ty: Type = parse_quote!((i32, u32));
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 8. transform_edge — edge cases  (8 tests)
// ===========================================================================

#[test]
fn transform_edge_unit_type_extract() {
    let ty: Type = parse_quote!(());
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "()");
}

#[test]
fn transform_edge_reference_extract() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn transform_edge_tuple_filter() {
    let ty: Type = parse_quote!((i32, u64));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(i32 , u64)");
}

#[test]
fn transform_edge_array_wrap() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn transform_edge_unit_wrap() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < () >");
}

#[test]
fn transform_edge_reference_filter() {
    let ty: Type = parse_quote!(&mut String);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "& mut String");
}

#[test]
fn transform_edge_slice_extract() {
    let ty: Type = parse_quote!([u8]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "[u8]");
}

#[test]
fn transform_edge_never_type_wrap() {
    let ty: Type = parse_quote!(!);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < ! >");
}
