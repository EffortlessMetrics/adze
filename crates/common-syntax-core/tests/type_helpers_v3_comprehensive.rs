//! Comprehensive tests for common-syntax-core type helpers.

use adze_common_syntax_core::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote};

fn empty_set() -> HashSet<&'static str> {
    HashSet::new()
}

fn option_set() -> HashSet<&'static str> {
    let mut s = HashSet::new();
    s.insert("Option");
    s
}

fn box_set() -> HashSet<&'static str> {
    let mut s = HashSet::new();
    s.insert("Box");
    s
}

fn multi_set() -> HashSet<&'static str> {
    let mut s = HashSet::new();
    s.insert("Option");
    s.insert("Box");
    s.insert("Vec");
    s
}

// --- try_extract_inner_type ---

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &empty_set());
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_vec_u32() {
    let ty: Type = parse_quote!(Vec<u32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_set());
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "u32");
}

#[test]
fn extract_wrong_wrapper_returns_false() {
    let ty: Type = parse_quote!(Option<String>);
    let (_, extracted) = try_extract_inner_type(&ty, "Vec", &empty_set());
    assert!(!extracted);
}

#[test]
fn extract_plain_type_returns_false() {
    let ty: Type = parse_quote!(String);
    let (_, extracted) = try_extract_inner_type(&ty, "Option", &empty_set());
    assert!(!extracted);
}

#[test]
fn extract_with_skip_over() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &box_set());
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_skip_over_no_match() {
    let ty: Type = parse_quote!(Box<String>);
    let (_, extracted) = try_extract_inner_type(&ty, "Option", &box_set());
    assert!(!extracted);
}

// --- filter_inner_type ---

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let result = filter_inner_type(&ty, &box_set());
    assert_eq!(result.to_token_stream().to_string(), "String");
}

#[test]
fn filter_plain_type_unchanged() {
    let ty: Type = parse_quote!(i32);
    let result = filter_inner_type(&ty, &box_set());
    assert_eq!(result.to_token_stream().to_string(), "i32");
}

#[test]
fn filter_option_not_in_set() {
    let ty: Type = parse_quote!(Option<String>);
    let result = filter_inner_type(&ty, &box_set());
    // Option not in box_set, so unchanged
    assert!(result.to_token_stream().to_string().contains("Option"));
}

#[test]
fn filter_nested_containers() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let result = filter_inner_type(&ty, &multi_set());
    assert_eq!(result.to_token_stream().to_string(), "String");
}

// --- wrap_leaf_type ---

#[test]
fn wrap_plain_string() {
    let ty: Type = parse_quote!(String);
    let result = wrap_leaf_type(&ty, &empty_set());
    let s = result.to_token_stream().to_string();
    assert!(s.contains("WithLeaf"), "should wrap in WithLeaf: {}", s);
}

#[test]
fn wrap_option_skipped() {
    let ty: Type = parse_quote!(Option<String>);
    let result = wrap_leaf_type(&ty, &option_set());
    let s = result.to_token_stream().to_string();
    assert!(s.contains("Option"), "outer Option should remain");
    assert!(s.contains("WithLeaf"), "inner String should be wrapped");
}

#[test]
fn wrap_plain_u32() {
    let ty: Type = parse_quote!(u32);
    let result = wrap_leaf_type(&ty, &empty_set());
    let s = result.to_token_stream().to_string();
    assert!(s.contains("WithLeaf"));
    assert!(s.contains("u32"));
}
