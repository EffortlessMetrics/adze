//! Comprehensive tests for type filtering and extraction in adze-common.
//!
//! Covers:
//! 1. try_extract_inner_type with Option (8 tests)
//! 2. try_extract_inner_type with Vec (8 tests)
//! 3. try_extract_inner_type with Box (5 tests)
//! 4. filter_inner_type comprehensive (8 tests)
//! 5. wrap_leaf_type comprehensive (8 tests)
//! 6. Type identity / is-parameterized style checks (5 tests)
//! 7. Interaction of filter/extract/wrap (5 tests)
//! 8. Edge cases (3+ tests)

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
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

// ===========================================================================
// 1. try_extract_inner_type — Option (8 tests)
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
fn extract_option_nested_vec() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < u8 >");
}

#[test]
fn extract_option_not_found_returns_original() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn extract_option_through_box_skip() {
    let ty: Type = parse_quote!(Box<Option<f64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_option_through_arc_skip() {
    let ty: Type = parse_quote!(Arc<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_option_skip_box_no_option_inside() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_option_double_skip() {
    let ty: Type = parse_quote!(Box<Arc<Option<u32>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u32");
}

// ===========================================================================
// 2. try_extract_inner_type — Vec (8 tests)
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
fn extract_vec_nested_option() {
    let ty: Type = parse_quote!(Vec<Option<i64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < i64 >");
}

#[test]
fn extract_vec_not_found_on_option() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Option < String >");
}

#[test]
fn extract_vec_through_box_skip() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_through_arc_box_skip() {
    let ty: Type = parse_quote!(Arc<Box<Vec<usize>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn extract_vec_plain_type_returns_original() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_skip_without_match_inside() {
    let ty: Type = parse_quote!(Box<Option<u16>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < Option < u16 > >");
}

// ===========================================================================
// 3. try_extract_inner_type — Box (5 tests)
// ===========================================================================

#[test]
fn extract_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_box_nested_vec() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn extract_box_through_arc_skip() {
    let ty: Type = parse_quote!(Arc<Box<u64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn extract_box_not_found_returns_original() {
    let ty: Type = parse_quote!(Rc<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Rc < String >");
}

#[test]
fn extract_box_from_plain_primitive() {
    let ty: Type = parse_quote!(u32);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "u32");
}

// ===========================================================================
// 4. filter_inner_type — comprehensive (8 tests)
// ===========================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "String");
}

#[test]
fn filter_arc_i32() {
    let ty: Type = parse_quote!(Arc<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Arc"]))), "i32");
}

#[test]
fn filter_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<f64>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "f64"
    );
}

#[test]
fn filter_not_in_skip_set_unchanged() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < String >"
    );
}

#[test]
fn filter_plain_type_unchanged() {
    let ty: Type = parse_quote!(String);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "String");
}

#[test]
fn filter_empty_skip_set_noop() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Box < String >"
    );
}

#[test]
fn filter_triple_nesting() {
    let ty: Type = parse_quote!(Box<Arc<Rc<bool>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]))),
        "bool"
    );
}

#[test]
fn filter_stops_at_non_skip_wrapper() {
    let ty: Type = parse_quote!(Box<Option<u8>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Option < u8 >"
    );
}

// ===========================================================================
// 5. wrap_leaf_type — comprehensive (8 tests)
// ===========================================================================

#[test]
fn wrap_plain_string() {
    let ty: Type = parse_quote!(String);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_plain_i32() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_vec_skip_wraps_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_option_skip_wraps_inner() {
    let ty: Type = parse_quote!(Option<u32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < u32 > >"
    );
}

#[test]
fn wrap_nested_vec_option_both_skip() {
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Option"]))),
        "Vec < Option < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wrap_not_in_skip_wraps_whole_type() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "adze :: WithLeaf < Vec < String > >"
    );
}

#[test]
fn wrap_result_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Result"]))),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_option_vec_only_option_skipped() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < Vec < u8 > > >"
    );
}

// ===========================================================================
// 6. Type identity / parameterized-style checks (5 tests)
//    (is_parameterized does not exist; test parameterized detection via
//     try_extract_inner_type returning extracted=true for known containers)
// ===========================================================================

#[test]
fn parameterized_vec_detected() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok, "Vec<u8> should be detected as parameterized Vec");
}

#[test]
fn parameterized_option_detected() {
    let ty: Type = parse_quote!(Option<bool>);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(
        ok,
        "Option<bool> should be detected as parameterized Option"
    );
}

#[test]
fn plain_type_not_parameterized_vec() {
    let ty: Type = parse_quote!(String);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok, "String should not be detected as parameterized Vec");
}

#[test]
fn plain_type_not_parameterized_option() {
    let ty: Type = parse_quote!(u64);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok, "u64 should not be detected as parameterized Option");
}

#[test]
fn different_container_not_detected() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok, "HashMap should not be detected as Vec");
}

// ===========================================================================
// 7. Interaction of filter / extract / wrap (5 tests)
// ===========================================================================

#[test]
fn filter_then_extract_vec() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let (inner, ok) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_then_wrap() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < u8 > >");
}

#[test]
fn extract_option_filter_arc_wrap() {
    let ty: Type = parse_quote!(Arc<Option<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn roundtrip_filter_noop_then_wrap() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

// ===========================================================================
// 8. Edge cases (3+ tests)
// ===========================================================================

#[test]
fn reference_type_extract_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn tuple_type_filter_returns_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(i32 , u32)");
}

#[test]
fn array_type_wrap_wraps_entirely() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn reference_type_wrap_wraps_entirely() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn extract_with_qualified_path_type() {
    let ty: Type = parse_quote!(Option<std::string::String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "std :: string :: String");
}
