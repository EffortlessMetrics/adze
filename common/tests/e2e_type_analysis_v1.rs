//! End-to-end integration tests for the full type analysis pipeline in
//! `adze-common`.
//!
//! Pipeline: parse type string → extract inner → filter → wrap → verify
//! consistency.
//!
//! 8 categories × 8 tests = 64 tests total.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use std::collections::HashSet;
use syn::{self, Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `true` when the type is a path whose last segment has angle-bracketed
/// generic arguments.
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

fn parse_ty(s: &str) -> Type {
    syn::parse_str(s).unwrap_or_else(|e| panic!("failed to parse `{s}`: {e}"))
}

// ===========================================================================
// 1. Full pipeline: parse → extract → verify  (8 tests)
// ===========================================================================

#[test]
fn pipeline_extract_vec_string() {
    let ty = parse_ty("Vec<String>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn pipeline_extract_option_i32() {
    let ty = parse_ty("Option<i32>");
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn pipeline_extract_box_u8_skip_box() {
    let ty = parse_ty("Box<Vec<u8>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn pipeline_extract_arc_option_bool_skip_arc() {
    let ty = parse_ty("Arc<Option<bool>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn pipeline_extract_miss_returns_original() {
    let ty = parse_ty("String");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn pipeline_extract_nested_skip_box_arc() {
    let ty = parse_ty("Box<Arc<Vec<f64>>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn pipeline_extract_rc_vec_skip_rc() {
    let ty = parse_ty("Rc<Vec<usize>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Rc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn pipeline_extract_option_option_inner() {
    let ty = parse_ty("Option<Option<char>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < char >");
}

// ===========================================================================
// 2. Full pipeline: parse → filter → verify  (8 tests)
// ===========================================================================

#[test]
fn pipeline_filter_box_string() {
    let ty = parse_ty("Box<String>");
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn pipeline_filter_arc_i32() {
    let ty = parse_ty("Arc<i32>");
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn pipeline_filter_box_arc_u64() {
    let ty = parse_ty("Box<Arc<u64>>");
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "u64");
}

#[test]
fn pipeline_filter_no_match_preserves_type() {
    let ty = parse_ty("Vec<u8>");
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
}

#[test]
fn pipeline_filter_empty_skip_preserves() {
    let ty = parse_ty("Box<String>");
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), "Box < String >");
}

#[test]
fn pipeline_filter_triple_nesting() {
    let ty = parse_ty("Box<Arc<Rc<bool>>>");
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ty_str(&filtered), "bool");
}

#[test]
fn pipeline_filter_ref_type_unchanged() {
    let ty: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "& str");
}

#[test]
fn pipeline_filter_tuple_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(i32 , u32)");
}

// ===========================================================================
// 3. Full pipeline: parse → wrap → verify output  (8 tests)
// ===========================================================================

#[test]
fn pipeline_wrap_plain_string() {
    let ty = parse_ty("String");
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn pipeline_wrap_vec_inner() {
    let ty = parse_ty("Vec<String>");
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn pipeline_wrap_option_inner() {
    let ty = parse_ty("Option<u32>");
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < u32 > >");
}

#[test]
fn pipeline_wrap_vec_not_in_skip_wraps_whole() {
    let ty = parse_ty("Vec<i32>");
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn pipeline_wrap_result_both_args() {
    let ty = parse_ty("Result<String, i32>");
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn pipeline_wrap_nested_option_vec() {
    let ty = parse_ty("Option<Vec<u8>>");
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < u8 > > >"
    );
}

#[test]
fn pipeline_wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn pipeline_wrap_bool() {
    let ty = parse_ty("bool");
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < bool >");
}

// ===========================================================================
// 4. Consistency: extract and filter agree  (8 tests)
// ===========================================================================

#[test]
fn consistency_box_string_extract_vs_filter() {
    let ty = parse_ty("Box<String>");
    let skip_set = skip(&["Box"]);
    let (extracted, ok) = try_extract_inner_type(&ty, "Box", &skip_set);
    let filtered = filter_inner_type(&ty, &skip_set);
    assert!(ok);
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn consistency_arc_i32_extract_vs_filter() {
    let ty = parse_ty("Arc<i32>");
    let skip_set = skip(&["Arc"]);
    let (extracted, ok) = try_extract_inner_type(&ty, "Arc", &skip_set);
    let filtered = filter_inner_type(&ty, &skip_set);
    assert!(ok);
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn consistency_non_container_both_return_same() {
    let ty = parse_ty("String");
    let skip_set = skip(&["Box"]);
    let (extracted, ok) = try_extract_inner_type(&ty, "Box", &skip_set);
    let filtered = filter_inner_type(&ty, &skip_set);
    assert!(!ok);
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn consistency_ref_type_both_return_same() {
    let ty: Type = parse_quote!(&str);
    let skip_set = skip(&["Box"]);
    let (extracted, _) = try_extract_inner_type(&ty, "Box", &skip_set);
    let filtered = filter_inner_type(&ty, &skip_set);
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn consistency_tuple_both_return_same() {
    let ty: Type = parse_quote!((i32, bool));
    let skip_set = skip(&["Box"]);
    let (extracted, ok) = try_extract_inner_type(&ty, "Box", &skip_set);
    let filtered = filter_inner_type(&ty, &skip_set);
    assert!(!ok);
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn consistency_single_skip_layer() {
    let ty = parse_ty("Rc<u8>");
    let skip_set = skip(&["Rc"]);
    let (extracted, ok) = try_extract_inner_type(&ty, "Rc", &skip_set);
    let filtered = filter_inner_type(&ty, &skip_set);
    assert!(ok);
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn consistency_nested_box_arc_extract_target_box() {
    // extract(target=Box, skip={}) → inner = Arc<u32>, extracted = true
    // filter(skip={Box})           → inner = Arc<u32>
    let ty = parse_ty("Box<Arc<u32>>");
    let (extracted, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn consistency_plain_u64_neither_extracts() {
    let ty = parse_ty("u64");
    let skip_set = skip(&["Box", "Arc"]);
    let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &skip_set);
    let filtered = filter_inner_type(&ty, &skip_set);
    assert!(!ok);
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

// ===========================================================================
// 5. Consistency: is_parameterized matches extract success  (8 tests)
// ===========================================================================

#[test]
fn param_vec_string_matches_extract() {
    let ty = parse_ty("Vec<String>");
    assert!(is_parameterized(&ty));
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
}

#[test]
fn param_option_bool_matches_extract() {
    let ty = parse_ty("Option<bool>");
    assert!(is_parameterized(&ty));
    let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
}

#[test]
fn param_plain_string_no_extract() {
    let ty = parse_ty("String");
    assert!(!is_parameterized(&ty));
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
}

#[test]
fn param_plain_i32_no_extract() {
    let ty = parse_ty("i32");
    assert!(!is_parameterized(&ty));
    let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
}

#[test]
fn param_ref_not_parameterized_no_extract() {
    let ty: Type = parse_quote!(&str);
    assert!(!is_parameterized(&ty));
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
}

#[test]
fn param_box_u8_parameterized_extracts() {
    let ty = parse_ty("Box<u8>");
    assert!(is_parameterized(&ty));
    let (_, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
}

#[test]
fn param_hashmap_parameterized_extracts() {
    let ty = parse_ty("HashMap<String, i32>");
    assert!(is_parameterized(&ty));
    let (_, ok) = try_extract_inner_type(&ty, "HashMap", &skip(&[]));
    assert!(ok);
}

#[test]
fn param_tuple_not_parameterized_no_extract() {
    let ty: Type = parse_quote!((u8, u16));
    assert!(!is_parameterized(&ty));
    let (_, ok) = try_extract_inner_type(&ty, "Tuple", &skip(&[]));
    assert!(!ok);
}

// ===========================================================================
// 6. Complex type pipelines (nested generics)  (8 tests)
// ===========================================================================

#[test]
fn complex_vec_option_string_extract_vec() {
    let ty = parse_ty("Vec<Option<String>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < String >");
}

#[test]
fn complex_option_vec_u8_extract_option() {
    let ty = parse_ty("Option<Vec<u8>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < u8 >");
}

#[test]
fn complex_box_arc_vec_extract_through_two_skips() {
    let ty = parse_ty("Box<Arc<Vec<i32>>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn complex_filter_three_layers() {
    let ty = parse_ty("Box<Rc<Arc<String>>>");
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Rc", "Arc"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn complex_wrap_nested_vec_option() {
    let ty = parse_ty("Vec<Option<bool>>");
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn complex_extract_then_wrap() {
    let ty = parse_ty("Vec<String>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn complex_filter_then_wrap() {
    let ty = parse_ty("Box<Arc<u32>>");
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "u32");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u32 >");
}

#[test]
fn complex_extract_then_filter_then_wrap() {
    let ty = parse_ty("Vec<Box<Arc<f64>>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < Arc < f64 > >");
    let filtered = filter_inner_type(&inner, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "f64");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f64 >");
}

// ===========================================================================
// 7. Pipeline with all standard container types  (8 tests)
// ===========================================================================

#[test]
fn container_vec_extract() {
    let ty = parse_ty("Vec<u16>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn container_option_filter() {
    let ty = parse_ty("Option<String>");
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn container_box_wrap() {
    let ty = parse_ty("Box<i64>");
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&wrapped), "Box < adze :: WithLeaf < i64 > >");
}

#[test]
fn container_arc_extract() {
    let ty = parse_ty("Arc<bool>");
    let (inner, ok) = try_extract_inner_type(&ty, "Arc", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn container_rc_filter() {
    let ty = parse_ty("Rc<f32>");
    let filtered = filter_inner_type(&ty, &skip(&["Rc"]));
    assert_eq!(ty_str(&filtered), "f32");
}

#[test]
fn container_result_wrap_both() {
    let ty = parse_ty("Result<u8, String>");
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < u8 > , adze :: WithLeaf < String > >"
    );
}

#[test]
fn container_hashmap_extract() {
    let ty = parse_ty("HashMap<String, Vec<u8>>");
    let (inner, ok) = try_extract_inner_type(&ty, "HashMap", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn container_btreemap_wrap() {
    let ty = parse_ty("BTreeMap<String, i32>");
    let wrapped = wrap_leaf_type(&ty, &skip(&["BTreeMap"]));
    assert_eq!(
        ty_str(&wrapped),
        "BTreeMap < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

// ===========================================================================
// 8. Edge cases: non-generic, qualified paths, special types  (8 tests)
// ===========================================================================

#[test]
fn edge_plain_primitive_not_extracted() {
    let ty = parse_ty("usize");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn edge_qualified_path_extract() {
    let ty = parse_ty("std::vec::Vec<i32>");
    // Last segment is Vec<i32>, so extraction by "Vec" should succeed.
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn edge_crate_path_not_extracted() {
    let ty = parse_ty("crate::MyType");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "crate :: MyType");
}

#[test]
fn edge_self_type_not_extracted() {
    let ty: Type = parse_quote!(Self);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Self");
}

#[test]
fn edge_unit_tuple_not_extracted() {
    let ty: Type = parse_quote!(());
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
}

#[test]
fn edge_ref_type_filter_passthrough() {
    let ty: Type = parse_quote!(&mut u32);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "& mut u32");
}

#[test]
fn edge_array_wrap_wraps_whole() {
    let ty: Type = parse_quote!([bool; 16]);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [bool ; 16] >");
}

#[test]
fn edge_super_path_not_parameterized() {
    let ty = parse_ty("super::Foo");
    assert!(!is_parameterized(&ty));
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "super :: Foo");
}
