//! Comprehensive tests for type utility functions in adze-common (v6).
//!
//! Covers try_extract_inner_type, filter_inner_type, wrap_leaf_type,
//! and their interactions with nested, complex, and edge-case types.

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote};

// ═══════════════════════════════════════════════════════════════════
// Helper: build a skip set from a slice of strs
// ═══════════════════════════════════════════════════════════════════

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn to_s(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ═══════════════════════════════════════════════════════════════════
// 1. try_extract_inner_type — 10 tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "String");
}

#[test]
fn extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "i32");
}

#[test]
fn extract_box_u64() {
    let ty: Type = parse_quote!(Box<u64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "u64");
}

#[test]
fn extract_result_first_arg() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(ok);
    // Extracts only the first generic argument
    assert_eq!(to_s(&inner), "String");
}

#[test]
fn extract_hashmap_first_arg() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "HashMap", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "String");
}

#[test]
fn extract_target_not_present_returns_false() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "String");
}

#[test]
fn extract_wrong_container_returns_false() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "Vec < String >");
}

#[test]
fn extract_through_skip_box_to_vec() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "u8");
}

#[test]
fn extract_through_nested_skip_arc_box_to_option() {
    let ty: Type = parse_quote!(Arc<Box<Option<bool>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "bool");
}

#[test]
fn extract_skip_present_but_target_absent_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "Box < String >");
}

// ═══════════════════════════════════════════════════════════════════
// 2. filter_inner_type — 8 tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn filter_box_removes_wrapper() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_s(&filtered), "String");
}

#[test]
fn filter_arc_removes_wrapper() {
    let ty: Type = parse_quote!(Arc<u32>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(to_s(&filtered), "u32");
}

#[test]
fn filter_nested_box_arc_removes_both() {
    let ty: Type = parse_quote!(Box<Arc<i64>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(to_s(&filtered), "i64");
}

#[test]
fn filter_triple_nested_wrappers() {
    let ty: Type = parse_quote!(Rc<Arc<Box<f64>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Rc", "Arc", "Box"]));
    assert_eq!(to_s(&filtered), "f64");
}

#[test]
fn filter_stops_at_non_skip_container() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_s(&filtered), "Vec < String >");
}

#[test]
fn filter_no_skip_returns_original() {
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(to_s(&filtered), "Vec < i32 >");
}

#[test]
fn filter_plain_type_returns_unchanged() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(to_s(&filtered), "String");
}

#[test]
fn filter_non_path_type_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_s(&filtered), "& str");
}

// ═══════════════════════════════════════════════════════════════════
// 3. wrap_leaf_type — 8 tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn wrap_plain_type() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(to_s(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_numeric_type() {
    let ty: Type = parse_quote!(u32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(to_s(&wrapped), "adze :: WithLeaf < u32 >");
}

#[test]
fn wrap_skips_vec_wraps_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(to_s(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_skips_option_wraps_inner() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(to_s(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_nested_skip_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        to_s(&wrapped),
        "Vec < Option < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wrap_non_skip_container_wraps_entire() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        to_s(&wrapped),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

#[test]
fn wrap_result_in_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        to_s(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < Error > >"
    );
}

#[test]
fn wrap_non_path_type_wraps_entirely() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(to_s(&wrapped), "adze :: WithLeaf < & str >");
}

// ═══════════════════════════════════════════════════════════════════
// 4. Parameterized type checks via try_extract — 5 tests
//    (is_parameterized etc. do not exist; use extract to probe)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn parameterized_option_is_extractable() {
    let ty: Type = parse_quote!(Option<String>);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok, "Option<String> should be extractable as Option");
}

#[test]
fn parameterized_vec_is_extractable() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok, "Vec<u8> should be extractable as Vec");
}

#[test]
fn parameterized_box_is_extractable() {
    let ty: Type = parse_quote!(Box<dyn Fn()>);
    let (_, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok, "Box<dyn Fn()> should be extractable as Box");
}

#[test]
fn plain_type_is_not_extractable_as_option() {
    let ty: Type = parse_quote!(String);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok, "plain String is not Option");
}

#[test]
fn primitive_is_not_extractable_as_vec() {
    let ty: Type = parse_quote!(bool);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok, "bool is not Vec");
}

// ═══════════════════════════════════════════════════════════════════
// 5. is_option / is_vec / is_box probed via try_extract — 8 tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn probe_is_option_true() {
    let ty: Type = parse_quote!(Option<()>);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
}

#[test]
fn probe_is_option_false_for_vec() {
    let ty: Type = parse_quote!(Vec<()>);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
}

#[test]
fn probe_is_vec_true() {
    let ty: Type = parse_quote!(Vec<f32>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
}

#[test]
fn probe_is_vec_false_for_option() {
    let ty: Type = parse_quote!(Option<f32>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
}

#[test]
fn probe_is_box_true() {
    let ty: Type = parse_quote!(Box<String>);
    let (_, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
}

#[test]
fn probe_is_box_false_for_rc() {
    let ty: Type = parse_quote!(Rc<String>);
    let (_, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!ok);
}

#[test]
fn probe_option_nested_in_box_via_skip() {
    let ty: Type = parse_quote!(Box<Option<u16>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "u16");
}

#[test]
fn probe_vec_nested_in_arc_via_skip() {
    let ty: Type = parse_quote!(Arc<Vec<char>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "char");
}

// ═══════════════════════════════════════════════════════════════════
// 6. Nested type operations — 8 tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn extract_from_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Vec < u8 >");
}

#[test]
fn extract_vec_from_option_vec_via_skip() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "u8");
}

#[test]
fn filter_box_option_leaves_option() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_s(&filtered), "Option < String >");
}

#[test]
fn filter_box_option_removes_both() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Option"]));
    assert_eq!(to_s(&filtered), "String");
}

#[test]
fn wrap_option_vec_leaf() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        to_s(&wrapped),
        "Option < Vec < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_box_option_leaf() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box", "Option"]));
    assert_eq!(
        to_s(&wrapped),
        "Box < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn extract_then_filter_roundtrip() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let skip_set = skip(&["Box"]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set);
    assert!(ok);
    assert_eq!(to_s(&inner), "String");

    // Filtering the same type with Box in skip set gives Vec<String>
    let filtered = filter_inner_type(&ty, &skip_set);
    assert_eq!(to_s(&filtered), "Vec < String >");
}

#[test]
fn wrap_then_extract_consistency() {
    let ty: Type = parse_quote!(Vec<bool>);
    let skip_set = skip(&["Vec"]);
    let wrapped = wrap_leaf_type(&ty, &skip_set);
    assert_eq!(to_s(&wrapped), "Vec < adze :: WithLeaf < bool > >");
    // The outer container is still Vec, so extraction should work
    let (inner, ok) = try_extract_inner_type(&wrapped, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "adze :: WithLeaf < bool >");
}

// ═══════════════════════════════════════════════════════════════════
// 7. Complex type patterns — 5 tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn extract_custom_container() {
    let ty: Type = parse_quote!(MyContainer<Payload>);
    let (inner, ok) = try_extract_inner_type(&ty, "MyContainer", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Payload");
}

#[test]
fn filter_custom_wrapper() {
    let ty: Type = parse_quote!(Wrapper<Inner>);
    let filtered = filter_inner_type(&ty, &skip(&["Wrapper"]));
    assert_eq!(to_s(&filtered), "Inner");
}

#[test]
fn wrap_qualified_path_type() {
    let ty: Type = parse_quote!(std::string::String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        to_s(&wrapped),
        "adze :: WithLeaf < std :: string :: String >"
    );
}

#[test]
fn extract_from_fully_qualified_option() {
    // Last segment is "Option", which matches
    let ty: Type = parse_quote!(std::option::Option<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "u8");
}

#[test]
fn filter_with_large_skip_set() {
    let ty: Type = parse_quote!(Arc<Mutex<RefCell<String>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Mutex", "RefCell"]));
    assert_eq!(to_s(&filtered), "String");
}

// ═══════════════════════════════════════════════════════════════════
// 8. Edge cases — 3 tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn edge_unit_type_not_extractable() {
    let ty: Type = parse_quote!(());
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "()");
}

#[test]
fn edge_reference_type_filter_returns_unchanged() {
    let ty: Type = parse_quote!(&'static str);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_s(&filtered), "& 'static str");
}

#[test]
fn edge_tuple_type_wrap_wraps_entirely() {
    let ty: Type = parse_quote!((i32, u32));
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(to_s(&wrapped), "adze :: WithLeaf < (i32 , u32) >");
}
