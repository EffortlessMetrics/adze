//! Snapshot tests for adze-common extraction operations (v5).
//!
//! 40+ insta snapshot tests covering:
//! 1. Extract from standard library types
//! 2. Extract from custom types
//! 3. Extract chains (nested unwrapping)
//! 4. Parameterized detection via extraction probes
//! 5. Wrap-then-snapshot round trips

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
// 1. Extract from standard library types (8 tests)
// ===========================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_box_u8() {
    let ty: Type = parse_quote!(Box<u8>);
    let (inner, found) = extract(&ty, "Box", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_arc_mutex_t() {
    let ty: Type = parse_quote!(Arc<Mutex<T>>);
    let (inner, found) = extract(&ty, "Arc", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_option_vec_u8() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_vec_option_bool() {
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let (inner, found) = extract(&ty, "Vec", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_box_through_arc() {
    let ty: Type = parse_quote!(Arc<Box<u8>>);
    let (inner, found) = extract(&ty, "Box", &["Arc"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_option_not_present_in_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

// ===========================================================================
// 2. Extract from custom types (8 tests)
// ===========================================================================

#[test]
fn extract_my_wrapper_t() {
    let ty: Type = parse_quote!(MyWrapper<T>);
    let (inner, found) = extract(&ty, "MyWrapper", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_custom_opt_string() {
    let ty: Type = parse_quote!(CustomOpt<String>);
    let (inner, found) = extract(&ty, "CustomOpt", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_result_first_type_arg() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, found) = extract(&ty, "Result", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_custom_through_box() {
    let ty: Type = parse_quote!(Box<Wrapper<i64>>);
    let (inner, found) = extract(&ty, "Wrapper", &["Box"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_custom_miss_wrong_name() {
    let ty: Type = parse_quote!(Foo<u32>);
    let (inner, found) = extract(&ty, "Bar", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_custom_nested_wrapper() {
    let ty: Type = parse_quote!(Outer<Inner<String>>);
    let (inner, found) = extract(&ty, "Outer", &[]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_custom_through_two_skips() {
    let ty: Type = parse_quote!(Arc<Box<Container<f64>>>);
    let (inner, found) = extract(&ty, "Container", &["Arc", "Box"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

#[test]
fn extract_custom_skip_no_match_inside() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, found) = extract(&ty, "Container", &["Box"]);
    insta::assert_snapshot!(format!("found={found} inner={inner}"));
}

// ===========================================================================
// 3. Extract chain — nested unwrapping (8 tests)
// ===========================================================================

#[test]
fn chain_option_vec_string_step1() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, found) = extract(&ty, "Option", &[]);
    insta::assert_snapshot!(format!("step1: found={found} inner={inner}"));
}

#[test]
fn chain_option_vec_string_step2() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip: HashSet<&str> = HashSet::new();
    let (step1, _) = try_extract_inner_type(&ty, "Option", &skip);
    let (step2, found) = try_extract_inner_type(&step1, "Vec", &skip);
    insta::assert_snapshot!(format!("found={found} inner={}", fmt(&step2)));
}

#[test]
fn chain_box_option_i32_full() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let skip: HashSet<&str> = HashSet::new();
    let (step1, f1) = try_extract_inner_type(&ty, "Box", &skip);
    let (step2, f2) = try_extract_inner_type(&step1, "Option", &skip);
    insta::assert_snapshot!(format!(
        "step1: found={f1} ty={} | step2: found={f2} ty={}",
        fmt(&step1),
        fmt(&step2)
    ));
}

#[test]
fn chain_arc_box_vec_string_all_steps() {
    let ty: Type = parse_quote!(Arc<Box<Vec<String>>>);
    let skip: HashSet<&str> = HashSet::new();
    let (s1, f1) = try_extract_inner_type(&ty, "Arc", &skip);
    let (s2, f2) = try_extract_inner_type(&s1, "Box", &skip);
    let (s3, f3) = try_extract_inner_type(&s2, "Vec", &skip);
    insta::assert_snapshot!(format!(
        "f1={f1} s1={} | f2={f2} s2={} | f3={f3} s3={}",
        fmt(&s1),
        fmt(&s2),
        fmt(&s3)
    ));
}

#[test]
fn chain_extract_skip_vs_manual() {
    let ty: Type = parse_quote!(Box<Vec<u16>>);
    // Using skip to reach Vec directly
    let (via_skip, found_skip) = extract(&ty, "Vec", &["Box"]);
    // Manual two-step extraction
    let skip: HashSet<&str> = HashSet::new();
    let (s1, _) = try_extract_inner_type(&ty, "Box", &skip);
    let (s2, found_manual) = try_extract_inner_type(&s1, "Vec", &skip);
    insta::assert_snapshot!(format!(
        "skip: found={found_skip} inner={via_skip} | manual: found={found_manual} inner={}",
        fmt(&s2)
    ));
}

#[test]
fn chain_filter_then_extract() {
    let ty: Type = parse_quote!(Box<Arc<Vec<Token>>>);
    let filtered_str = filter(&ty, &["Box", "Arc"]);
    let filtered_ty: Type = syn::parse_str(&filtered_str).unwrap();
    let (inner, found) = extract(&filtered_ty, "Vec", &[]);
    insta::assert_snapshot!(format!(
        "filtered={filtered_str} then_found={found} inner={inner}"
    ));
}

#[test]
fn chain_extract_then_filter() {
    let ty: Type = parse_quote!(Option<Box<Arc<u32>>>);
    let skip: HashSet<&str> = HashSet::new();
    let (after_extract, found) = try_extract_inner_type(&ty, "Option", &skip);
    let after_filter = filter(&after_extract, &["Box", "Arc"]);
    insta::assert_snapshot!(format!(
        "extracted={found} ty={} | filtered={after_filter}",
        fmt(&after_extract)
    ));
}

#[test]
fn chain_triple_option_unwrap() {
    let ty: Type = parse_quote!(Option<Option<Option<bool>>>);
    let skip: HashSet<&str> = HashSet::new();
    let (s1, f1) = try_extract_inner_type(&ty, "Option", &skip);
    let (s2, f2) = try_extract_inner_type(&s1, "Option", &skip);
    let (s3, f3) = try_extract_inner_type(&s2, "Option", &skip);
    insta::assert_snapshot!(format!(
        "f1={f1} s1={} | f2={f2} s2={} | f3={f3} s3={}",
        fmt(&s1),
        fmt(&s2),
        fmt(&s3)
    ));
}

// ===========================================================================
// 4. Parameterized detection via extraction probes (8 tests)
// ===========================================================================

fn is_generic_container(ty: &Type, name: &str) -> bool {
    let (_, found) = extract(ty, name, &[]);
    found
}

#[test]
fn param_option_u32_is_option() {
    let ty: Type = parse_quote!(Option<u32>);
    insta::assert_snapshot!(format!("is_option={}", is_generic_container(&ty, "Option")));
}

#[test]
fn param_string_is_not_option() {
    let ty: Type = parse_quote!(String);
    insta::assert_snapshot!(format!("is_option={}", is_generic_container(&ty, "Option")));
}

#[test]
fn param_vec_f64_is_vec() {
    let ty: Type = parse_quote!(Vec<f64>);
    insta::assert_snapshot!(format!("is_vec={}", is_generic_container(&ty, "Vec")));
}

#[test]
fn param_i32_is_not_vec() {
    let ty: Type = parse_quote!(i32);
    insta::assert_snapshot!(format!("is_vec={}", is_generic_container(&ty, "Vec")));
}

#[test]
fn param_box_string_is_box() {
    let ty: Type = parse_quote!(Box<String>);
    insta::assert_snapshot!(format!("is_box={}", is_generic_container(&ty, "Box")));
}

#[test]
fn param_ref_str_is_not_box() {
    let ty: Type = parse_quote!(&str);
    insta::assert_snapshot!(format!("is_box={}", is_generic_container(&ty, "Box")));
}

#[test]
fn param_result_is_result() {
    let ty: Type = parse_quote!(Result<String, Error>);
    insta::assert_snapshot!(format!("is_result={}", is_generic_container(&ty, "Result")));
}

#[test]
fn param_tuple_is_not_result() {
    let ty: Type = parse_quote!((i32, i32));
    insta::assert_snapshot!(format!("is_result={}", is_generic_container(&ty, "Result")));
}

// ===========================================================================
// 5. Wrap then snapshot (8 tests)
// ===========================================================================

#[test]
fn wrap_plain_string() {
    let ty: Type = parse_quote!(String);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_plain_i32() {
    let ty: Type = parse_quote!(i32);
    insta::assert_snapshot!(wrap(&ty, &[]));
}

#[test]
fn wrap_vec_string_skip_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    insta::assert_snapshot!(wrap(&ty, &["Vec"]));
}

#[test]
fn wrap_option_u64_skip_option() {
    let ty: Type = parse_quote!(Option<u64>);
    insta::assert_snapshot!(wrap(&ty, &["Option"]));
}

#[test]
fn wrap_option_vec_bool_skip_both() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    insta::assert_snapshot!(wrap(&ty, &["Option", "Vec"]));
}

#[test]
fn wrap_box_vec_u8_skip_both() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    insta::assert_snapshot!(wrap(&ty, &["Box", "Vec"]));
}

#[test]
fn wrap_result_skip_result() {
    let ty: Type = parse_quote!(Result<String, i32>);
    insta::assert_snapshot!(wrap(&ty, &["Result"]));
}

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    insta::assert_snapshot!(wrap(&ty, &[]));
}
