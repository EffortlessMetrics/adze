//! Comprehensive tests for type expansion and grammar-related utilities.
//!
//! This test module covers 64 test cases across 8 categories:
//! 1. try_extract_inner_type with Option
//! 2. try_extract_inner_type with Vec
//! 3. try_extract_inner_type with Box
//! 4. filter_inner_type operations
//! 5. Type parameterization detection (when available)
//! 6. wrap_leaf_type operations
//! 7. Complex nested type handling
//! 8. Edge cases and special types

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote};

// ============================================================================
// Category 1: try_extract_inner_type with Option (8 tests)
// ============================================================================

#[test]
fn extract_option_string_returns_string() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_option_u32_returns_u32() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<u32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "u32");
}

#[test]
fn extract_option_vec_returns_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Vec < u8 >");
}

#[test]
fn extract_option_box_returns_box() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<Box<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Box < String >");
}

#[test]
fn extract_nested_option_with_skip() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_option_from_bare_string_returns_none() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let (result_ty, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(result_ty.to_token_stream().to_string(), "String");
}

#[test]
fn extract_option_from_bare_u32_returns_none() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(u32);
    let (result_ty, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(result_ty.to_token_stream().to_string(), "u32");
}

#[test]
fn extract_option_from_custom_type_returns_none() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(MyCustomType);
    let (result_ty, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(result_ty.to_token_stream().to_string(), "MyCustomType");
}

// ============================================================================
// Category 2: try_extract_inner_type with Vec (8 tests)
// ============================================================================

#[test]
fn extract_vec_string_returns_string() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_vec_u32_returns_u32() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<u32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "u32");
}

#[test]
fn extract_vec_option_returns_option() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Option < String >");
}

#[test]
fn extract_vec_box_returns_box() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Box<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Box < String >");
}

#[test]
fn extract_vec_with_skip_finds_nested_vec() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_vec_from_bare_string_returns_none() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let (result_ty, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(result_ty.to_token_stream().to_string(), "String");
}

#[test]
fn extract_vec_from_bare_type_returns_none() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(i64);
    let (result_ty, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(result_ty.to_token_stream().to_string(), "i64");
}

#[test]
fn extract_vec_from_different_container_returns_none() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<String>);
    let (result_ty, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
    assert_eq!(result_ty.to_token_stream().to_string(), "Option < String >");
}

// ============================================================================
// Category 3: try_extract_inner_type with Box (8 tests)
// ============================================================================

#[test]
fn extract_box_string_returns_string() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_box_u32_returns_u32() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<u32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "u32");
}

#[test]
fn extract_box_vec_returns_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Vec < String >");
}

#[test]
fn extract_box_option_returns_option() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Option < String >");
}

#[test]
fn extract_nested_box_with_skip() {
    let skip: HashSet<&str> = HashSet::from(["Arc"]);
    let ty: Type = parse_quote!(Arc<Box<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_box_from_bare_string_returns_none() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let (result_ty, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(!extracted);
    assert_eq!(result_ty.to_token_stream().to_string(), "String");
}

#[test]
fn extract_box_from_bare_type_returns_none() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(bool);
    let (result_ty, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(!extracted);
    assert_eq!(result_ty.to_token_stream().to_string(), "bool");
}

#[test]
fn extract_box_from_different_container_returns_none() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let (result_ty, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(!extracted);
    assert_eq!(result_ty.to_token_stream().to_string(), "Vec < String >");
}

// ============================================================================
// Category 4: filter_inner_type operations (8 tests)
// ============================================================================

#[test]
fn filter_option_strips_outer_wrapper() {
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn filter_vec_strips_outer_wrapper() {
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let ty: Type = parse_quote!(Vec<u32>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "u32");
}

#[test]
fn filter_box_strips_outer_wrapper() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn filter_bare_type_returns_itself() {
    let skip: HashSet<&str> = HashSet::from(["Box", "Vec", "Option"]);
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn filter_nested_wrappers_peels_all() {
    let skip: HashSet<&str> = HashSet::from(["Box", "Option"]);
    let ty: Type = parse_quote!(Box<Option<String>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn filter_idempotent_on_bare_type() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(i32);
    let filtered_once = filter_inner_type(&ty, &skip);
    let filtered_twice = filter_inner_type(&filtered_once, &skip);
    assert_eq!(
        filtered_once.to_token_stream().to_string(),
        filtered_twice.to_token_stream().to_string()
    );
}

#[test]
fn filter_complex_type_with_vec() {
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let ty: Type = parse_quote!(Vec<(String, u32)>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "(String , u32)");
}

#[test]
fn filter_unit_type_remains_unchanged() {
    let skip: HashSet<&str> = HashSet::from(["Box", "Option"]);
    let ty: Type = parse_quote!(());
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "()");
}

// ============================================================================
// Category 5: Type parameterization detection (8 tests)
// ============================================================================
// Note: The public API doesn't expose an is_parameterized function,
// but we test related patterns through wrapper functions.

#[test]
fn option_contains_generic_parameter() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<T>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "T");
}

#[test]
fn vec_contains_generic_parameter() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<T>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "T");
}

#[test]
fn box_contains_generic_parameter() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<T>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "T");
}

#[test]
fn concrete_string_is_not_parameterized() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let (_result_ty, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
}

#[test]
fn concrete_u32_is_not_parameterized() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(u32);
    let (_result_ty, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
}

#[test]
fn tuple_with_multiple_generics() {
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let ty: Type = parse_quote!((T, U));
    let filtered = filter_inner_type(&ty, &skip);
    // Tuples are not in skip set, so they return unchanged
    assert_eq!(filtered.to_token_stream().to_string(), "(T , U)");
}

#[test]
fn custom_generic_struct_with_parameter() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Generic<T>);
    // Generic<T> is not a recognized wrapper, so extraction fails
    let (_result_ty, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
}

#[test]
fn hashmap_with_two_type_parameters() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(HashMap<K, V>);
    // HashMap is not a recognized wrapper for extraction
    let (_result_ty, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
}

// ============================================================================
// Category 6: wrap_leaf_type operations (8 tests)
// ============================================================================

#[test]
fn wrap_bare_string_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_bare_u32_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(u32);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < u32 >"
    );
}

#[test]
fn wrap_bool_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < bool >"
    );
}

#[test]
fn wrap_custom_struct_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(MyStruct);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < MyStruct >"
    );
}

#[test]
fn wrap_respects_skip_set_for_vec() {
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_respects_skip_set_for_option() {
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let ty: Type = parse_quote!(Option<u32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < adze :: WithLeaf < u32 > >"
    );
}

#[test]
fn wrap_complex_path_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(std::collections::HashMap);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert!(
        wrapped
            .to_token_stream()
            .to_string()
            .contains("adze :: WithLeaf")
    );
}

#[test]
fn wrap_unit_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < () >"
    );
}

// ============================================================================
// Category 7: Complex nested type handling (8 tests)
// ============================================================================

#[test]
fn deeply_nested_option_vec_box() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Box<Vec<Option<String>>>);
    // First, Box is unwrapped, then we look for Vec
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Option < String >");
}

#[test]
fn type_with_multiple_generic_parameters() {
    let skip: HashSet<&str> = HashSet::from(["Result"]);
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // Both generic args should be wrapped
    let result_str = wrapped.to_token_stream().to_string();
    assert!(result_str.contains("adze :: WithLeaf < String >"));
    assert!(result_str.contains("adze :: WithLeaf < i32 >"));
}

#[test]
fn type_with_lifetime_parameter() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&'a str);
    // Reference types are not handled as special cases
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "& 'a str");
}

#[test]
fn filter_with_where_clause_simulation() {
    // Simulate filtering a type that might appear in where clause
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "String");
}

#[test]
fn tuple_type_extraction_attempt() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!((String, u32));
    // Tuples are not path types, so extraction returns None
    let (_result_ty, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
}

#[test]
fn array_type_extraction_attempt() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!([u8; 32]);
    // Array types are not path types
    let (_result_ty, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
}

#[test]
fn reference_type_extraction_attempt() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    let (_result_ty, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
}

#[test]
fn function_pointer_type_extraction_attempt() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(fn(u32) -> String);
    let (_result_ty, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!extracted);
}

// ============================================================================
// Category 8: Edge cases and special types (8 tests)
// ============================================================================

#[test]
fn very_long_type_name_extraction() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<VeryLongTypeNameForTesting>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(
        inner.to_token_stream().to_string(),
        "VeryLongTypeNameForTesting"
    );
}

#[test]
fn type_name_with_underscores() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<My_Custom_Type_Name>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "My_Custom_Type_Name");
}

#[test]
fn reserved_word_as_type_component() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<r#type>);
    let (_inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
}

#[test]
fn multiple_nested_extractors_in_sequence() {
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let ty: Type = parse_quote!(Box<Vec<String>>);
    // First extract Vec
    let (intermediate, extracted1) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted1);
    // Then try to extract Option from the result
    let (result, extracted2) = try_extract_inner_type(&intermediate, "Option", &skip);
    assert!(!extracted2);
    assert_eq!(result.to_token_stream().to_string(), "String");
}

#[test]
fn wrap_with_empty_skip_set() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // Vec is not in skip set, so entire type gets wrapped
    assert!(
        wrapped
            .to_token_stream()
            .to_string()
            .contains("adze :: WithLeaf")
    );
}

#[test]
fn filter_with_empty_skip_set() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip);
    // Box is not in skip set, so type returns unchanged
    assert_eq!(filtered.to_token_stream().to_string(), "Box < String >");
}

#[test]
fn chained_skip_operations() {
    let skip: HashSet<&str> = HashSet::from(["Box", "Arc"]);
    let ty: Type = parse_quote!(Arc<Box<Option<String>>>);
    // Arc is skipped, Box is skipped, Option is extracted
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn wrap_nested_containers_independently() {
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // Both Vec and Option are in skip set, String gets wrapped
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}
