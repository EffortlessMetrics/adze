//! Snapshot tests for adze-common expansion helpers.
//!
//! Validates that `try_extract_inner_type`, `filter_inner_type`, and
//! `wrap_leaf_type` produce deterministic, expected output via insta snapshots.

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
// 1. try_extract_inner_type snapshots (8 tests)
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_no_match() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_through_box() {
    let ty: Type = parse_quote!(Box<Vec<u64>>);
    let (inner, found) = extract(&ty, "Vec", &["Box"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_through_arc() {
    let ty: Type = parse_quote!(Arc<Option<bool>>);
    let (inner, found) = extract(&ty, "Option", &["Arc"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_skip_no_target_inside() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, found) = extract(&ty, "Vec", &["Box"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_non_path_type_reference() {
    let ty: Type = parse_quote!(&str);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_non_path_type_tuple() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

// ===========================================================================
// 2. filter_inner_type snapshots (8 tests)
// ===========================================================================

#[test]
fn filter_box_string() {
    insta::assert_snapshot!(filter(&parse_quote!(Box<String>), &["Box"]));
}

#[test]
fn filter_arc_i32() {
    insta::assert_snapshot!(filter(&parse_quote!(Arc<i32>), &["Arc"]));
}

#[test]
fn filter_nested_box_arc() {
    insta::assert_snapshot!(filter(&parse_quote!(Box<Arc<u8>>), &["Box", "Arc"]));
}

#[test]
fn filter_no_match_plain() {
    insta::assert_snapshot!(filter(&parse_quote!(String), &["Box"]));
}

#[test]
fn filter_empty_skip_set() {
    insta::assert_snapshot!(filter(&parse_quote!(Box<String>), &[]));
}

#[test]
fn filter_non_path_type() {
    insta::assert_snapshot!(filter(&parse_quote!(&str), &["Box"]));
}

#[test]
fn filter_triple_nesting() {
    insta::assert_snapshot!(filter(
        &parse_quote!(Box<Arc<Rc<bool>>>),
        &["Box", "Arc", "Rc"]
    ));
}

#[test]
fn filter_partial_skip() {
    // Only Box is in skip set; Arc stays.
    insta::assert_snapshot!(filter(&parse_quote!(Box<Arc<u32>>), &["Box"]));
}

// ===========================================================================
// 3. wrap_leaf_type snapshots (8 tests)
// ===========================================================================

#[test]
fn wrap_plain_string() {
    insta::assert_snapshot!(wrap(&parse_quote!(String), &[]));
}

#[test]
fn wrap_vec_inner() {
    insta::assert_snapshot!(wrap(&parse_quote!(Vec<Expr>), &["Vec"]));
}

#[test]
fn wrap_option_inner() {
    insta::assert_snapshot!(wrap(&parse_quote!(Option<Token>), &["Option"]));
}

#[test]
fn wrap_no_skip_on_container() {
    // Vec is NOT in skip set so the whole Vec<T> gets wrapped.
    insta::assert_snapshot!(wrap(&parse_quote!(Vec<Expr>), &[]));
}

#[test]
fn wrap_nested_option_vec() {
    insta::assert_snapshot!(wrap(&parse_quote!(Option<Vec<Item>>), &["Option", "Vec"]));
}

#[test]
fn wrap_result_both_args() {
    insta::assert_snapshot!(wrap(&parse_quote!(Result<String, i32>), &["Result"]));
}

#[test]
fn wrap_reference_type() {
    insta::assert_snapshot!(wrap(&parse_quote!(&str), &[]));
}

#[test]
fn wrap_array_type() {
    insta::assert_snapshot!(wrap(&parse_quote!([u8; 4]), &[]));
}

// ===========================================================================
// 4. Complex type transformations (8 tests)
// ===========================================================================

#[test]
fn complex_option_vec_all_ops() {
    let ty: Type = parse_quote!(Option<Vec<Node>>);
    let skip = &["Box", "Arc"];
    let wrap_skip = &["Option", "Vec"];
    let (ext_inner, ext_found) = extract(&ty, "Option", skip);
    let filtered = filter(&ty, skip);
    let wrapped = wrap(&ty, wrap_skip);
    insta::assert_snapshot!(format!(
        "extract: found={ext_found} inner={ext_inner}\nfilter: {filtered}\nwrap: {wrapped}"
    ));
}

#[test]
fn complex_box_option_string() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let skip = &["Box"];
    let wrap_skip = &["Option"];
    let (ext_inner, ext_found) = extract(&ty, "Option", skip);
    let filtered = filter(&ty, skip);
    let wrapped = wrap(&ty, wrap_skip);
    insta::assert_snapshot!(format!(
        "extract: found={ext_found} inner={ext_inner}\nfilter: {filtered}\nwrap: {wrapped}"
    ));
}

#[test]
fn complex_vec_of_box_expr() {
    let ty: Type = parse_quote!(Vec<Box<Expr>>);
    let skip = &["Box"];
    let wrap_skip = &["Vec"];
    let (ext_inner, ext_found) = extract(&ty, "Vec", skip);
    let filtered = filter(&ty, skip);
    let wrapped = wrap(&ty, wrap_skip);
    insta::assert_snapshot!(format!(
        "extract: found={ext_found} inner={ext_inner}\nfilter: {filtered}\nwrap: {wrapped}"
    ));
}

#[test]
fn complex_arc_vec_token() {
    let ty: Type = parse_quote!(Arc<Vec<Token>>);
    let skip = &["Arc"];
    let wrap_skip = &["Vec"];
    let (ext_inner, ext_found) = extract(&ty, "Vec", skip);
    let filtered = filter(&ty, skip);
    let wrapped = wrap(&ty, wrap_skip);
    insta::assert_snapshot!(format!(
        "extract: found={ext_found} inner={ext_inner}\nfilter: {filtered}\nwrap: {wrapped}"
    ));
}

#[test]
fn complex_plain_type_all_ops() {
    let ty: Type = parse_quote!(Identifier);
    let skip = &["Box"];
    let wrap_skip = &["Vec", "Option"];
    let (ext_inner, ext_found) = extract(&ty, "Vec", skip);
    let filtered = filter(&ty, skip);
    let wrapped = wrap(&ty, wrap_skip);
    insta::assert_snapshot!(format!(
        "extract: found={ext_found} inner={ext_inner}\nfilter: {filtered}\nwrap: {wrapped}"
    ));
}

#[test]
fn complex_result_type_wrap() {
    let ty: Type = parse_quote!(Result<Ast, ParseError>);
    let wrapped = wrap(&ty, &["Result"]);
    let filtered = filter(&ty, &["Result"]);
    insta::assert_snapshot!(format!("wrap: {wrapped}\nfilter: {filtered}"));
}

#[test]
fn complex_deeply_nested_filter() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Vec<u8>>>>);
    let filtered = filter(&ty, &["Box", "Arc", "Rc"]);
    let wrapped = wrap(&ty, &["Box", "Arc", "Rc", "Vec"]);
    insta::assert_snapshot!(format!("filter: {filtered}\nwrap: {wrapped}"));
}

#[test]
fn complex_extract_option_through_two_skips() {
    let ty: Type = parse_quote!(Box<Arc<Option<f64>>>);
    let (inner, found) = extract(&ty, "Option", &["Box", "Arc"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

// ===========================================================================
// 5. Nested generics (5 tests)
// ===========================================================================

#[test]
fn nested_vec_of_vec() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let wrapped = wrap(&ty, &["Vec"]);
    insta::assert_snapshot!(wrapped);
}

#[test]
fn nested_option_of_option() {
    let ty: Type = parse_quote!(Option<Option<bool>>);
    let wrapped = wrap(&ty, &["Option"]);
    insta::assert_snapshot!(wrapped);
}

#[test]
fn nested_extract_only_outer_vec() {
    let ty: Type = parse_quote!(Vec<Vec<String>>);
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn nested_hashmap_wrap() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let wrapped = wrap(&ty, &["HashMap", "Vec"]);
    insta::assert_snapshot!(wrapped);
}

#[test]
fn nested_option_vec_box_deep() {
    let ty: Type = parse_quote!(Option<Vec<Box<Node>>>);
    let wrapped = wrap(&ty, &["Option", "Vec", "Box"]);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!(
        "wrap: {wrapped}\nextract: found={found} inner={inner}"
    ));
}

// ===========================================================================
// 6. Edge cases (3 tests)
// ===========================================================================

#[test]
fn edge_qualified_path_type() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    // The last segment is Vec, so extraction should still work.
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn edge_unit_like_type() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap(&ty, &[]);
    let filtered = filter(&ty, &["Box"]);
    insta::assert_snapshot!(format!("wrap: {wrapped}\nfilter: {filtered}"));
}

#[test]
fn edge_single_char_type_name() {
    let ty: Type = parse_quote!(T);
    let wrapped = wrap(&ty, &[]);
    let (inner, found) = extract(&ty, "Option", &[]);
    let filtered = filter(&ty, &["Box"]);
    insta::assert_snapshot!(format!(
        "wrap: {wrapped}\nextract: found={found} inner={inner}\nfilter: {filtered}"
    ));
}
