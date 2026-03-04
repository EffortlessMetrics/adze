//! Comprehensive tests for adze-common crate: re-exported types and utility functions.

use std::collections::HashSet;
use syn::Type;

#[test]
fn common_reexports_name_value_expr() {
    let _ = std::any::type_name::<adze_common::NameValueExpr>();
}

#[test]
fn common_reexports_field_then_params() {
    let _ = std::any::type_name::<adze_common::FieldThenParams>();
}

#[test]
fn try_extract_inner_type_option() {
    let ty: Type = syn::parse_str("Option<String>").unwrap();
    let skip = HashSet::new();
    let (result_ty, found) = adze_common::try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let _ = result_ty;
}

#[test]
fn try_extract_inner_type_vec() {
    let ty: Type = syn::parse_str("Vec<u32>").unwrap();
    let skip = HashSet::new();
    let (result_ty, found) = adze_common::try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let _ = result_ty;
}

#[test]
fn try_extract_inner_type_not_matching() {
    let ty: Type = syn::parse_str("String").unwrap();
    let skip = HashSet::new();
    let (_result_ty, found) = adze_common::try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
}

#[test]
fn try_extract_inner_type_box() {
    let ty: Type = syn::parse_str("Box<i32>").unwrap();
    let skip = HashSet::new();
    let (result_ty, found) = adze_common::try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let _ = result_ty;
}

#[test]
fn filter_inner_type_with_skip() {
    let ty: Type = syn::parse_str("Option<String>").unwrap();
    let mut skip = HashSet::new();
    skip.insert("Option");
    let filtered = adze_common::filter_inner_type(&ty, &skip);
    let _ = filtered;
}

#[test]
fn filter_inner_type_no_skip() {
    let ty: Type = syn::parse_str("String").unwrap();
    let skip = HashSet::new();
    let filtered = adze_common::filter_inner_type(&ty, &skip);
    let _ = filtered;
}

#[test]
fn wrap_leaf_type_simple() {
    let ty: Type = syn::parse_str("String").unwrap();
    let skip = HashSet::new();
    let wrapped = adze_common::wrap_leaf_type(&ty, &skip);
    let _ = wrapped;
}

#[test]
fn wrap_leaf_type_with_option_skip() {
    let ty: Type = syn::parse_str("Option<String>").unwrap();
    let mut skip = HashSet::new();
    skip.insert("Option");
    let wrapped = adze_common::wrap_leaf_type(&ty, &skip);
    let _ = wrapped;
}

#[test]
fn filter_inner_type_nested() {
    let ty: Type = syn::parse_str("Option<Vec<String>>").unwrap();
    let mut skip = HashSet::new();
    skip.insert("Option");
    let filtered = adze_common::filter_inner_type(&ty, &skip);
    let _ = filtered;
}

#[test]
fn filter_inner_type_double_skip() {
    let ty: Type = syn::parse_str("Option<Vec<String>>").unwrap();
    let mut skip = HashSet::new();
    skip.insert("Option");
    skip.insert("Vec");
    let filtered = adze_common::filter_inner_type(&ty, &skip);
    let _ = filtered;
}

#[test]
fn try_extract_inner_type_primitive() {
    let ty: Type = syn::parse_str("u32").unwrap();
    let skip = HashSet::new();
    let (_result_ty, found) = adze_common::try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
}

#[test]
fn try_extract_inner_type_with_skip() {
    let ty: Type = syn::parse_str("Option<Vec<String>>").unwrap();
    let mut skip = HashSet::new();
    skip.insert("Vec");
    let (result_ty, found) = adze_common::try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let _ = result_ty;
}

#[test]
fn wrap_leaf_type_vec() {
    let ty: Type = syn::parse_str("Vec<u32>").unwrap();
    let mut skip = HashSet::new();
    skip.insert("Vec");
    let wrapped = adze_common::wrap_leaf_type(&ty, &skip);
    let _ = wrapped;
}

#[test]
fn wrap_leaf_type_box() {
    let ty: Type = syn::parse_str("Box<String>").unwrap();
    let mut skip = HashSet::new();
    skip.insert("Box");
    let wrapped = adze_common::wrap_leaf_type(&ty, &skip);
    let _ = wrapped;
}
