//! Snapshot tests for adze-common type wrapping, extraction, and filtering.
//!
//! Organized by operation: wrap simple, wrap container, wrap nested,
//! extract-then-snapshot, filter-then-snapshot.

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fmt(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn wrap(ty: &Type, skip: &[&str]) -> String {
    let skip_set: HashSet<&str> = skip.iter().copied().collect();
    fmt(&wrap_leaf_type(ty, &skip_set))
}

fn extract(ty: &Type, target: &str, skip: &[&str]) -> String {
    let skip_set: HashSet<&str> = skip.iter().copied().collect();
    let (inner, found) = try_extract_inner_type(ty, target, &skip_set);
    format!("found={found} inner={}", fmt(&inner))
}

fn filter(ty: &Type, skip: &[&str]) -> String {
    let skip_set: HashSet<&str> = skip.iter().copied().collect();
    fmt(&filter_inner_type(ty, &skip_set))
}

// ===========================================================================
// 1. Wrap simple types (8 tests)
// ===========================================================================

#[test]
fn wrap_simple_string() {
    let ty: Type = parse_quote!(String);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_simple_i32() {
    let ty: Type = parse_quote!(i32);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_simple_bool() {
    let ty: Type = parse_quote!(bool);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_simple_u8() {
    let ty: Type = parse_quote!(u8);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_simple_f64() {
    let ty: Type = parse_quote!(f64);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_simple_usize() {
    let ty: Type = parse_quote!(usize);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_simple_char() {
    let ty: Type = parse_quote!(char);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_simple_unit() {
    let ty: Type = parse_quote!(());
    insta::assert_snapshot!(wrap(&ty, &[]));
}

// ===========================================================================
// 2. Wrap container types (8 tests)
// ===========================================================================

#[test]
fn wrap_container_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    insta::assert_snapshot!(wrap(&ty, &["Vec"]));
}

#[test]
fn wrap_container_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    insta::assert_snapshot!(wrap(&ty, &["Option"]));
}

#[test]
fn wrap_container_box_u8() {
    let ty: Type = parse_quote!(Box<u8>);
    insta::assert_snapshot!(wrap(&ty, &["Box"]));
}

#[test]
fn wrap_container_hashmap_string_i32() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    insta::assert_snapshot!(wrap(&ty, &["HashMap"]));
}

#[test]
fn wrap_container_result_string_error() {
    let ty: Type = parse_quote!(Result<String, Error>);
    insta::assert_snapshot!(wrap(&ty, &["Result"]));
}

#[test]
fn wrap_container_vec_no_skip() {
    let ty: Type = parse_quote!(Vec<String>);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_container_option_no_skip() {
    let ty: Type = parse_quote!(Option<bool>);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_container_box_no_skip() {
    let ty: Type = parse_quote!(Box<f64>);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

// ===========================================================================
// 3. Wrap nested types (8 tests)
// ===========================================================================

#[test]
fn wrap_nested_vec_option_string() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    insta::assert_snapshot!(wrap(&ty, &["Vec", "Option"]));
}

#[test]
fn wrap_nested_option_vec_i32() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    insta::assert_snapshot!(wrap(&ty, &["Vec", "Option"]));
}

#[test]
fn wrap_nested_box_vec_option_u8() {
    let ty: Type = parse_quote!(Box<Vec<Option<u8>>>);
    insta::assert_snapshot!(wrap(&ty, &["Box", "Vec", "Option"]));
}

#[test]
fn wrap_nested_vec_vec_bool() {
    let ty: Type = parse_quote!(Vec<Vec<bool>>);
    insta::assert_snapshot!(wrap(&ty, &["Vec"]));
}

#[test]
fn wrap_nested_option_option_string() {
    let ty: Type = parse_quote!(Option<Option<String>>);
    insta::assert_snapshot!(wrap(&ty, &["Option"]));
}

#[test]
fn wrap_nested_result_option_i32_string() {
    let ty: Type = parse_quote!(Result<Option<i32>, String>);
    insta::assert_snapshot!(wrap(&ty, &["Result", "Option"]));
}

#[test]
fn wrap_nested_vec_box_f64() {
    let ty: Type = parse_quote!(Vec<Box<f64>>);
    insta::assert_snapshot!(wrap(&ty, &["Vec", "Box"]));
}

#[test]
fn wrap_nested_option_vec_partial_skip() {
    let ty: Type = parse_quote!(Option<Vec<char>>);
    insta::assert_snapshot!(wrap(&ty, &["Option"]));
}

// ===========================================================================
// 4. Extract then snapshot (8 tests)
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    insta::assert_snapshot!(extract(&ty, "Vec", &[]));
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    insta::assert_snapshot!(extract(&ty, "Option", &[]));
}

#[test]
fn extract_vec_not_found() {
    let ty: Type = parse_quote!(Option<String>);
    insta::assert_snapshot!(extract(&ty, "Vec", &[]));
}

#[test]
fn extract_through_box() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    insta::assert_snapshot!(extract(&ty, "Vec", &["Box"]));
}

#[test]
fn extract_through_arc_box() {
    let ty: Type = parse_quote!(Arc<Box<Option<f64>>>);
    insta::assert_snapshot!(extract(&ty, "Option", &["Arc", "Box"]));
}

#[test]
fn extract_plain_type_no_match() {
    let ty: Type = parse_quote!(String);
    insta::assert_snapshot!(extract(&ty, "Vec", &[]));
}

#[test]
fn extract_reference_type() {
    let ty: Type = parse_quote!(&str);
    insta::assert_snapshot!(extract(&ty, "Option", &[]));
}

#[test]
fn extract_nested_skip_no_target() {
    let ty: Type = parse_quote!(Box<String>);
    insta::assert_snapshot!(extract(&ty, "Vec", &["Box"]));
}

// ===========================================================================
// 5. Filter then snapshot (8 tests)
// ===========================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    insta::assert_snapshot!(filter(&ty, &["Box"]));
}

#[test]
fn filter_arc_i32() {
    let ty: Type = parse_quote!(Arc<i32>);
    insta::assert_snapshot!(filter(&ty, &["Arc"]));
}

#[test]
fn filter_box_arc_bool() {
    let ty: Type = parse_quote!(Box<Arc<bool>>);
    insta::assert_snapshot!(filter(&ty, &["Box", "Arc"]));
}

#[test]
fn filter_no_match_vec() {
    let ty: Type = parse_quote!(Vec<u8>);
    insta::assert_snapshot!(filter(&ty, &["Box"]));
}

#[test]
fn filter_plain_type() {
    let ty: Type = parse_quote!(String);
    insta::assert_snapshot!(filter(&ty, &["Box", "Arc"]));
}

#[test]
fn filter_tuple_type() {
    let ty: Type = parse_quote!((i32, u32));
    insta::assert_snapshot!(filter(&ty, &["Box"]));
}

#[test]
fn filter_empty_skip_set() {
    let ty: Type = parse_quote!(Box<String>);
    insta::assert_snapshot!(filter(&ty, &[]));
}

#[test]
fn filter_triple_nesting() {
    let ty: Type = parse_quote!(Box<Arc<Rc<usize>>>);
    insta::assert_snapshot!(filter(&ty, &["Box", "Arc", "Rc"]));
}
