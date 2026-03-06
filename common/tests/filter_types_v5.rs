//! Tests for `filter_inner_type`, `try_extract_inner_type`, `wrap_leaf_type`,
//! and a local `is_parameterized` helper covering eight categories of type
//! analysis scenarios (60+ tests total).

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

/// Returns `true` when the outermost type carries angle-bracketed generic
/// arguments (i.e. is a parameterized `Type::Path`).
fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

// ===========================================================================
// 1. Filter "Option" — extracts from Option<T>  (8 tests)
// ===========================================================================

#[test]
fn filter_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn filter_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn filter_option_u8() {
    let ty: Type = parse_quote!(Option<u8>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "u8");
}

#[test]
fn filter_option_f64() {
    let ty: Type = parse_quote!(Option<f64>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "f64");
}

#[test]
fn filter_option_usize() {
    let ty: Type = parse_quote!(Option<usize>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "usize");
}

#[test]
fn filter_option_char() {
    let ty: Type = parse_quote!(Option<char>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "char");
}

#[test]
fn filter_option_custom_type() {
    let ty: Type = parse_quote!(Option<MyStruct>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "MyStruct");
}

// ===========================================================================
// 2. Filter "Vec" — extracts from Vec<T>  (8 tests)
// ===========================================================================

#[test]
fn filter_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn filter_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_vec_bool() {
    let ty: Type = parse_quote!(Vec<bool>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn filter_vec_u64() {
    let ty: Type = parse_quote!(Vec<u64>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "u64");
}

#[test]
fn filter_vec_f32() {
    let ty: Type = parse_quote!(Vec<f32>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "f32");
}

#[test]
fn filter_vec_usize() {
    let ty: Type = parse_quote!(Vec<usize>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "usize");
}

#[test]
fn filter_vec_char() {
    let ty: Type = parse_quote!(Vec<char>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "char");
}

#[test]
fn filter_vec_custom_type() {
    let ty: Type = parse_quote!(Vec<Token>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "Token");
}

// ===========================================================================
// 3. Filter "Box" — extracts from Box<T>  (8 tests)
// ===========================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn filter_box_i32() {
    let ty: Type = parse_quote!(Box<i32>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_box_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn filter_box_u16() {
    let ty: Type = parse_quote!(Box<u16>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "u16");
}

#[test]
fn filter_box_f64() {
    let ty: Type = parse_quote!(Box<f64>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "f64");
}

#[test]
fn filter_box_usize() {
    let ty: Type = parse_quote!(Box<usize>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "usize");
}

#[test]
fn filter_box_char() {
    let ty: Type = parse_quote!(Box<char>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "char");
}

#[test]
fn filter_box_custom_type() {
    let ty: Type = parse_quote!(Box<AstNode>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "AstNode");
}

// ===========================================================================
// 4. Filter mismatched outer — returns unchanged  (8 tests)
// ===========================================================================

#[test]
fn mismatch_vec_skip_box_unchanged() {
    let ty: Type = parse_quote!(Vec<String>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Vec < String >");
}

#[test]
fn mismatch_option_skip_vec_unchanged() {
    let ty: Type = parse_quote!(Option<i32>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "Option < i32 >");
}

#[test]
fn mismatch_box_skip_option_unchanged() {
    let ty: Type = parse_quote!(Box<bool>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "Box < bool >");
}

#[test]
fn mismatch_result_skip_vec_unchanged() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "Result < String , i32 >");
}

#[test]
fn mismatch_arc_skip_box_unchanged() {
    let ty: Type = parse_quote!(Arc<u64>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Arc < u64 >");
}

#[test]
fn mismatch_plain_type_skip_option_unchanged() {
    let ty: Type = parse_quote!(String);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn mismatch_empty_skip_set_unchanged() {
    let ty: Type = parse_quote!(Option<String>);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "Option < String >");
}

#[test]
fn mismatch_tuple_skip_box_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "(i32 , u32)");
}

// ===========================================================================
// 5. Filter with nested types — Option<Vec<T>>, Box<Arc<T>>, etc.  (8 tests)
// ===========================================================================

#[test]
fn nested_option_vec_strip_option() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "Vec < String >");
}

#[test]
fn nested_box_arc_strip_both() {
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn nested_box_vec_strip_box_only() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Vec < u8 >");
}

#[test]
fn nested_arc_option_strip_arc() {
    let ty: Type = parse_quote!(Arc<Option<bool>>);
    let result = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&result), "Option < bool >");
}

#[test]
fn nested_option_option_strip_both() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn nested_box_box_strip_both() {
    let ty: Type = parse_quote!(Box<Box<String>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn nested_triple_box_arc_rc_strip_all() {
    let ty: Type = parse_quote!(Box<Arc<Rc<f64>>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ty_str(&result), "f64");
}

#[test]
fn nested_vec_not_in_skip_stops_early() {
    let ty: Type = parse_quote!(Box<Vec<Option<u32>>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Vec < Option < u32 > >");
}

// ===========================================================================
// 6. Filter vs extract agreement  (8 tests)
// ===========================================================================

#[test]
fn agreement_vec_string_extract_matches_filter_identity() {
    let ty: Type = parse_quote!(Vec<String>);
    let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    // filter with Vec in skip set also strips it
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn agreement_option_i32_extract_matches_filter() {
    let ty: Type = parse_quote!(Option<i32>);
    let (extracted, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn agreement_box_bool_extract_matches_filter() {
    let ty: Type = parse_quote!(Box<bool>);
    let (extracted, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn agreement_mismatch_both_return_original() {
    let ty: Type = parse_quote!(String);
    let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    // Neither changes the type
    assert_eq!(ty_str(&extracted), "String");
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn agreement_extract_through_skip_matches_filter_chain() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    // filter strips Box then Vec
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Vec"]));
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn agreement_non_path_both_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (extracted, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn agreement_arc_extract_vs_filter() {
    let ty: Type = parse_quote!(Arc<usize>);
    let (extracted, ok) = try_extract_inner_type(&ty, "Arc", &skip(&[]));
    assert!(ok);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn agreement_nested_skip_extract_vs_double_filter() {
    let ty: Type = parse_quote!(Arc<Box<f32>>);
    let (extracted, ok) = try_extract_inner_type(&ty, "Box", &skip(&["Arc"]));
    assert!(ok);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

// ===========================================================================
// 7. Complex types: Result<T, E>, HashMap<K, V>, wrap_leaf_type  (8 tests)
// ===========================================================================

#[test]
fn complex_result_filter_unchanged_when_not_in_skip() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Result < String , i32 >");
}

#[test]
fn complex_hashmap_filter_unchanged() {
    let ty: Type = parse_quote!(HashMap<String, Vec<u8>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "HashMap < String , Vec < u8 > >");
}

#[test]
fn complex_result_is_parameterized() {
    let ty: Type = parse_quote!(Result<(), String>);
    assert!(is_parameterized(&ty));
}

#[test]
fn complex_hashmap_is_parameterized() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn complex_wrap_result_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn complex_wrap_hashmap_wraps_both_args() {
    let ty: Type = parse_quote!(HashMap<String, u64>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["HashMap"]));
    assert_eq!(
        ty_str(&wrapped),
        "HashMap < adze :: WithLeaf < String > , adze :: WithLeaf < u64 > >"
    );
}

#[test]
fn complex_wrap_vec_option_recursive() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn complex_wrap_plain_type_wraps_entirely() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

// ===========================================================================
// 8. Edge cases: qualified paths, no generics, non-path types  (8 tests)
// ===========================================================================

#[test]
fn edge_qualified_vec_filter_uses_last_segment() {
    // filter_inner_type checks only the last path segment
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn edge_qualified_option_filter_uses_last_segment() {
    let ty: Type = parse_quote!(std::option::Option<u8>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "u8");
}

#[test]
fn edge_no_generics_filter_unchanged() {
    let ty: Type = parse_quote!(String);
    let result = filter_inner_type(&ty, &skip(&["Option", "Vec", "Box"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn edge_reference_filter_unchanged() {
    let ty: Type = parse_quote!(&str);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "& str");
}

#[test]
fn edge_array_filter_unchanged() {
    let ty: Type = parse_quote!([u8; 4]);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "[u8 ; 4]");
}

#[test]
fn edge_tuple_is_not_parameterized() {
    let ty: Type = parse_quote!((i32, String));
    assert!(!is_parameterized(&ty));
}

#[test]
fn edge_unqualified_option_is_parameterized() {
    let ty: Type = parse_quote!(Option<i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn edge_wrap_non_path_array_wraps_entirely() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}
