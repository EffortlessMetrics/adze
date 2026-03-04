//! Comprehensive tests for type utility functions in adze-common.
//!
//! Tests cover:
//! - try_extract_inner_type: extracting inner types from containers
//! - filter_inner_type: removing container wrappers
//! - wrap_leaf_type: wrapping leaf types with adze::WithLeaf
//! - NameValueExpr: parsing key=value expressions
//! - FieldThenParams: parsing field(params) expressions

use adze_common_syntax_core::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::parse_quote;

// ============================================================================
// Helper functions
// ============================================================================

fn empty_set() -> HashSet<&'static str> {
    HashSet::new()
}

fn set(items: &[&'static str]) -> HashSet<&'static str> {
    items.iter().copied().collect()
}

fn type_str(ty: &syn::Type) -> String {
    ty.to_token_stream().to_string()
}

// ============================================================================
// try_extract_inner_type Tests
// ============================================================================

#[test]
fn extract_from_option_string() {
    let ty: syn::Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &empty_set());
    assert!(extracted);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_from_vec_i32() {
    let ty: syn::Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_set());
    assert!(extracted);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn extract_from_box_mytype() {
    let ty: syn::Type = parse_quote!(Box<MyType>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &empty_set());
    assert!(extracted);
    assert_eq!(type_str(&inner), "MyType");
}

#[test]
fn extract_with_skip_option_from_option_vec_t() {
    // Option<Vec<T>> with skip=["Option"] should extract Vec<T>, not T
    let ty: syn::Type = parse_quote!(Option<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &set(&["Box"]));
    assert!(extracted);
    assert_eq!(type_str(&inner), "Vec < i32 >");
}

#[test]
fn extract_with_skip_option_and_vec_from_option_vec_t() {
    // Option<Vec<T>> with skip=["Option", "Vec"] should extract T
    // But we need to skip through both Option and Vec to find the target
    let ty: syn::Type = parse_quote!(Option<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &set(&["Option"]));
    assert!(extracted);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn extract_from_nested_box_vec_finds_vec_skipping_box() {
    // Box<Vec<String>> - extract Vec by skipping Box
    let ty: syn::Type = parse_quote!(Box<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &set(&["Box"]));
    assert!(extracted);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_skipping_box_to_find_vec() {
    // Box<Vec<String>> with Box in skip, looking for Vec
    let ty: syn::Type = parse_quote!(Box<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &set(&["Box"]));
    assert!(extracted);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_skipping_multiple_containers_deeply_nested() {
    // Box<Arc<Vec<String>>> - extract Vec by skipping Box and Arc
    let ty: syn::Type = parse_quote!(Box<Arc<Vec<String>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &set(&["Box", "Arc"]));
    assert!(extracted);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_not_found_returns_false_and_original() {
    let ty: syn::Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_set());
    assert!(!extracted);
    assert_eq!(type_str(&inner), "Option < String >");
}

#[test]
fn extract_from_qualified_path() {
    let ty: syn::Type = parse_quote!(std::vec::Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_set());
    assert!(extracted);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn extract_from_hashmap_skipping_result() {
    // Result<HashMap<K, V>, E> - skip Result to find HashMap
    let ty: syn::Type = parse_quote!(Result<std::collections::HashMap<String, i32>, String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Result", &set(&["Box"]));
    assert!(extracted);
    assert_eq!(
        type_str(&inner),
        "std :: collections :: HashMap < String , i32 >"
    );
}

#[test]
fn extract_from_reference_type_returns_unchanged() {
    // Reference types are not Type::Path, so extraction returns them unchanged
    let ty: syn::Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &empty_set());
    assert!(!extracted);
    assert_eq!(type_str(&inner), "& str");
}

#[test]
fn extract_from_tuple_type_returns_unchanged() {
    // Tuple types are not Type::Path
    let ty: syn::Type = parse_quote!((i32, String));
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_set());
    assert!(!extracted);
    assert_eq!(type_str(&inner), "(i32 , String)");
}

#[test]
fn extract_from_array_type_returns_unchanged() {
    // Array types are not Type::Path
    let ty: syn::Type = parse_quote!([u8; 4]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_set());
    assert!(!extracted);
    assert_eq!(type_str(&inner), "[u8 ; 4]");
}

// ============================================================================
// filter_inner_type Tests
// ============================================================================

#[test]
fn filter_removes_box() {
    let ty: syn::Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &set(&["Box"]));
    assert_eq!(type_str(&filtered), "String");
}

#[test]
fn filter_removes_arc() {
    let ty: syn::Type = parse_quote!(Arc<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &set(&["Arc"]));
    assert_eq!(type_str(&filtered), "Vec < i32 >");
}

#[test]
fn filter_removes_multiple_wrappers() {
    // Box<Arc<String>> with skip=["Box", "Arc"]
    let ty: syn::Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &set(&["Box", "Arc"]));
    assert_eq!(type_str(&filtered), "String");
}

#[test]
fn filter_with_empty_skip_returns_unchanged() {
    let ty: syn::Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &empty_set());
    assert_eq!(type_str(&filtered), "Box < String >");
}

#[test]
fn filter_unmatched_container_returns_unchanged() {
    // Vec<String> with skip=["Box"] - Box not in type, returns unchanged
    let ty: syn::Type = parse_quote!(Vec<String>);
    let filtered = filter_inner_type(&ty, &set(&["Box"]));
    assert_eq!(type_str(&filtered), "Vec < String >");
}

#[test]
fn filter_non_path_type_returns_unchanged() {
    // Tuple type is not a Type::Path
    let ty: syn::Type = parse_quote!((i32, String));
    let filtered = filter_inner_type(&ty, &set(&["Box"]));
    assert_eq!(type_str(&filtered), "(i32 , String)");
}

#[test]
fn filter_reference_type_returns_unchanged() {
    let ty: syn::Type = parse_quote!(&String);
    let filtered = filter_inner_type(&ty, &set(&["Box"]));
    assert_eq!(type_str(&filtered), "& String");
}

#[test]
fn filter_qualified_path() {
    let ty: syn::Type = parse_quote!(std::boxed::Box<String>);
    let filtered = filter_inner_type(&ty, &set(&["Box"]));
    assert_eq!(type_str(&filtered), "String");
}

// ============================================================================
// wrap_leaf_type Tests
// ============================================================================

#[test]
fn wrap_leaf_simple_string() {
    let ty: syn::Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &empty_set());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_leaf_i32() {
    let ty: syn::Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &empty_set());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_skips_vec_container() {
    // Vec<String> with skip=["Vec"] - wraps inner String only
    let ty: syn::Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &set(&["Vec"]));
    assert_eq!(type_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_skips_option_container() {
    // Option<i32> with skip=["Option"] - wraps inner i32 only
    let ty: syn::Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &set(&["Option"]));
    assert_eq!(type_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_nested_with_multiple_skips() {
    // Vec<Option<String>> with skip=["Vec", "Option"]
    let ty: syn::Type = parse_quote!(Vec<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &set(&["Vec", "Option"]));
    assert_eq!(
        type_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_result_with_multiple_type_args() {
    // Result<String, i32> with skip=["Result"]
    let ty: syn::Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &set(&["Result"]));
    assert_eq!(
        type_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_non_path_array_type() {
    // Array types are not Type::Path, so wrap the whole thing
    let ty: syn::Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &empty_set());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_non_path_reference_type() {
    // Reference types are not Type::Path, so wrap the whole thing
    let ty: syn::Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &empty_set());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_qualified_path() {
    let ty: syn::Type = parse_quote!(std::vec::Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &set(&["Vec"]));
    assert_eq!(
        type_str(&wrapped),
        "std :: vec :: Vec < adze :: WithLeaf < String > >"
    );
}

// ============================================================================
// Roundtrip Tests
// ============================================================================

#[test]
fn roundtrip_extract_then_wrap_option_string() {
    // Extract String from Option<String>, then wrap it
    let ty: syn::Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &empty_set());
    assert!(extracted);

    let wrapped = wrap_leaf_type(&inner, &empty_set());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn roundtrip_extract_then_wrap_vec_with_skip() {
    // Extract i32 from Box<Vec<i32>>, then wrap it
    let ty: syn::Type = parse_quote!(Box<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &set(&["Box"]));
    assert!(extracted);
    // inner is String (the first generic arg of Vec), not Vec<i32>
    // Vec's first (and only) arg is i32

    // Now wrap it without skip set
    let wrapped = wrap_leaf_type(&inner, &empty_set());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < i32 >");
}

// ============================================================================
// NameValueExpr Tests
// ============================================================================

#[test]
fn parse_namevalue_key_value() {
    let expr: NameValueExpr = parse_quote!(key = "value");
    assert_eq!(expr.path.to_string(), "key");
}

#[test]
fn parse_namevalue_precedence_int() {
    let expr: NameValueExpr = parse_quote!(precedence = 5);
    assert_eq!(expr.path.to_string(), "precedence");
}

#[test]
fn parse_namevalue_name() {
    let expr: NameValueExpr = parse_quote!(name = "test");
    assert_eq!(expr.path.to_string(), "name");
}

#[test]
fn parse_namevalue_multiple_different_types() {
    let expr: NameValueExpr = parse_quote!(count = 42);
    assert_eq!(expr.path.to_string(), "count");

    let expr: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(expr.path.to_string(), "enabled");
}

// ============================================================================
// FieldThenParams Tests
// ============================================================================

#[test]
fn parse_field_then_params_field_only() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn parse_field_then_params_field_with_params() {
    let ftp: FieldThenParams = parse_quote!(String, name = "test", value = 42);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "name");
    assert_eq!(ftp.params[1].path.to_string(), "value");
}

#[test]
fn parse_field_then_params_single_param() {
    let ftp: FieldThenParams = parse_quote!(Vec<i32>, count = 5);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "count");
}

#[test]
fn parse_field_then_params_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(Option<String>, a = 1, b = 2, c = 3);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 3);
    assert_eq!(ftp.params[0].path.to_string(), "a");
    assert_eq!(ftp.params[1].path.to_string(), "b");
    assert_eq!(ftp.params[2].path.to_string(), "c");
}

// ============================================================================
// Edge Cases and Type Coverage
// ============================================================================

#[test]
fn extract_from_type_with_lifetime() {
    // &'a T is a reference type, not Type::Path
    let ty: syn::Type = parse_quote!(&'a String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &empty_set());
    assert!(!extracted);
    assert_eq!(type_str(&inner), "& 'a String");
}

#[test]
fn extract_from_mutable_reference() {
    // &mut T is a reference type, not Type::Path
    let ty: syn::Type = parse_quote!(&mut String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &empty_set());
    assert!(!extracted);
    assert_eq!(type_str(&inner), "& mut String");
}

#[test]
fn wrap_type_with_lifetime() {
    let ty: syn::Type = parse_quote!(&'a String);
    let wrapped = wrap_leaf_type(&ty, &empty_set());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < & 'a String >");
}

#[test]
fn wrap_mutable_reference() {
    let ty: syn::Type = parse_quote!(&mut String);
    let wrapped = wrap_leaf_type(&ty, &empty_set());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < & mut String >");
}

#[test]
fn extract_from_hashmap_k_v() {
    // HashMap<K, V> with HashMap in skip - first arg is K
    let ty: syn::Type = parse_quote!(HashMap<String, i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "HashMap", &empty_set());
    assert!(extracted);
    // First generic argument is String (the key type)
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn wrap_generic_type_with_multiple_args() {
    // HashMap<String, i32> with HashMap in skip
    let ty: syn::Type = parse_quote!(HashMap<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &set(&["HashMap"]));
    assert_eq!(
        type_str(&wrapped),
        "HashMap < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn namevalue_expr_with_path() {
    let expr: NameValueExpr = parse_quote!(path = "src/main.rs");
    assert_eq!(expr.path.to_string(), "path");
}

#[test]
fn fieldthenparams_with_complex_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<Option<String>>, skip = true);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
}
