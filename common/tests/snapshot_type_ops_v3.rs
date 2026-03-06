//! Snapshot tests for adze-common type transformation operations.
//!
//! Validates that `try_extract_inner_type`, `filter_inner_type`, and
//! `wrap_leaf_type` produce deterministic, stable output via insta snapshots.

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

fn extract(ty: &Type, target: &str, skip: &[&str]) -> (String, bool) {
    let skip_set: HashSet<&str> = skip.iter().copied().collect();
    let (inner, found) = try_extract_inner_type(ty, target, &skip_set);
    (fmt(&inner), found)
}

fn filter(ty: &Type, skip: &[&str]) -> String {
    let skip_set: HashSet<&str> = skip.iter().copied().collect();
    fmt(&filter_inner_type(ty, &skip_set))
}

fn wrap(ty: &Type, skip: &[&str]) -> String {
    let skip_set: HashSet<&str> = skip.iter().copied().collect();
    fmt(&wrap_leaf_type(ty, &skip_set))
}

// ===========================================================================
// 1. Extract inner from Option (5 snapshots)
// ===========================================================================

#[test]
fn option_extract_simple_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn option_extract_nested_vec() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn option_extract_through_box() {
    let ty: Type = parse_quote!(Box<Option<i64>>);
    let (inner, found) = extract(&ty, "Option", &["Box"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn option_extract_not_present() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn option_extract_with_tuple_inner() {
    let ty: Type = parse_quote!(Option<(u32, u32)>);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

// ===========================================================================
// 2. Extract inner from Vec (5 snapshots)
// ===========================================================================

#[test]
fn vec_extract_simple_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn vec_extract_nested_option() {
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn vec_extract_through_box() {
    let ty: Type = parse_quote!(Box<Vec<f64>>);
    let (inner, found) = extract(&ty, "Vec", &["Box"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn vec_extract_not_present() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn vec_extract_of_custom_type() {
    let ty: Type = parse_quote!(Vec<MyStruct>);
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

// ===========================================================================
// 3. Extract inner from Box (5 snapshots)
// ===========================================================================

#[test]
fn box_extract_simple_u64() {
    let ty: Type = parse_quote!(Box<u64>);
    let (inner, found) = extract(&ty, "Box", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn box_extract_nested_vec() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, found) = extract(&ty, "Box", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn box_extract_through_arc() {
    let ty: Type = parse_quote!(Arc<Box<u8>>);
    let (inner, found) = extract(&ty, "Box", &["Arc"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn box_extract_not_present() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = extract(&ty, "Box", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn box_extract_with_option_inner() {
    let ty: Type = parse_quote!(Box<Option<f32>>);
    let (inner, found) = extract(&ty, "Box", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

// ===========================================================================
// 4. Wrap leaf types (5 snapshots)
// ===========================================================================

#[test]
fn wrap_plain_string() {
    let ty: Type = parse_quote!(String);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_vec_of_string() {
    let ty: Type = parse_quote!(Vec<String>);
    insta::assert_snapshot!(wrap(&ty, &["Vec"]));
}

#[test]
fn wrap_option_of_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    insta::assert_snapshot!(wrap(&ty, &["Option"]));
}

#[test]
fn wrap_box_of_vec_of_u8() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    insta::assert_snapshot!(wrap(&ty, &["Box", "Vec"]));
}

#[test]
fn wrap_nested_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<Expr>>);
    insta::assert_snapshot!(wrap(&ty, &["Option", "Vec"]));
}

// ===========================================================================
// 5. Filter inner types (5 snapshots)
// ===========================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    insta::assert_snapshot!(filter(&ty, &["Box"]));
}

#[test]
fn filter_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    insta::assert_snapshot!(filter(&ty, &["Option"]));
}

#[test]
fn filter_box_option_vec_u8() {
    let ty: Type = parse_quote!(Box<Option<Vec<u8>>>);
    insta::assert_snapshot!(filter(&ty, &["Box", "Option"]));
}

#[test]
fn filter_no_match_plain() {
    let ty: Type = parse_quote!(String);
    insta::assert_snapshot!(filter(&ty, &["Box"]));
}

#[test]
fn filter_double_box() {
    let ty: Type = parse_quote!(Box<Box<f64>>);
    insta::assert_snapshot!(filter(&ty, &["Box"]));
}

// ===========================================================================
// 6. Complex nested operations (5 snapshots)
// ===========================================================================

#[test]
fn complex_extract_then_wrap() {
    let ty: Type = parse_quote!(Option<Vec<Token>>);
    let skip_set: HashSet<&str> = HashSet::new();
    let (inner_ty, found) = try_extract_inner_type(&ty, "Option", &skip_set);
    let inner = fmt(&inner_ty);
    let wrapped = wrap(&inner_ty, &["Vec"]);
    insta::assert_snapshot!(format!("found={found} inner={inner} wrapped={wrapped}"));
}

#[test]
fn complex_filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Option<Ident>>);
    let filtered = filter(&ty, &["Box", "Option"]);
    let filtered_ty: Type = syn::parse_str(&filtered).unwrap();
    let wrapped = wrap(&filtered_ty, &[]);
    insta::assert_snapshot!(format!("filtered={filtered} wrapped={wrapped}"));
}

#[test]
fn complex_extract_vec_through_two_skips() {
    let ty: Type = parse_quote!(Arc<Box<Vec<String>>>);
    let (inner, found) = extract(&ty, "Vec", &["Arc", "Box"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn complex_wrap_deeply_nested() {
    let ty: Type = parse_quote!(Option<Box<Vec<Stmt>>>);
    insta::assert_snapshot!(wrap(&ty, &["Option", "Box", "Vec"]));
}

#[test]
fn complex_all_three_ops() {
    let ty: Type = parse_quote!(Box<Vec<MyNode>>);
    let (ext_inner, ext_found) = extract(&ty, "Vec", &["Box"]);
    let filtered = filter(&ty, &["Box"]);
    let wrapped = wrap(&ty, &["Box", "Vec"]);
    insta::assert_snapshot!(format!(
        "extract_found={ext_found} extract_inner={ext_inner} filtered={filtered} wrapped={wrapped}"
    ));
}

// ===========================================================================
// 7. Type predicate combinations (5 snapshots)
// ===========================================================================

#[test]
fn predicate_is_option_true() {
    let ty: Type = parse_quote!(Option<u32>);
    let (_, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("is_option={found}"));
}

#[test]
fn predicate_is_vec_true() {
    let ty: Type = parse_quote!(Vec<String>);
    let (_, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("is_vec={found}"));
}

#[test]
fn predicate_is_box_false_for_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (_, found) = extract(&ty, "Box", &[]);
    insta::assert_snapshot!(format!("is_box={found}"));
}

#[test]
fn predicate_option_through_box_true() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (_, found_direct) = extract(&ty, "Option", &[]);
    let (_, found_skip) = extract(&ty, "Option", &["Box"]);
    insta::assert_snapshot!(format!("direct={found_direct} with_skip={found_skip}"));
}

#[test]
fn predicate_vec_through_two_layers() {
    let ty: Type = parse_quote!(Arc<Box<Vec<u16>>>);
    let (_, found_no_skip) = extract(&ty, "Vec", &[]);
    let (_, found_one_skip) = extract(&ty, "Vec", &["Arc"]);
    let (_, found_both_skip) = extract(&ty, "Vec", &["Arc", "Box"]);
    insta::assert_snapshot!(format!(
        "no_skip={found_no_skip} one_skip={found_one_skip} both_skip={found_both_skip}"
    ));
}

// ===========================================================================
// 8. Edge case outputs (5 snapshots)
// ===========================================================================

#[test]
fn edge_reference_type_extract() {
    let ty: Type = parse_quote!(&str);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn edge_tuple_type_filter() {
    let ty: Type = parse_quote!((i32, i32));
    insta::assert_snapshot!(filter(&ty, &["Box"]));
}

#[test]
fn edge_unit_type_wrap() {
    let ty: Type = parse_quote!(());
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn edge_qualified_path_extract() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn edge_single_char_type_wrap() {
    let ty: Type = parse_quote!(T);
    insta::assert_snapshot!(wrap(&ty, &[]));
}
