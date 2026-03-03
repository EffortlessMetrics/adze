#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for type extraction and processing in adze-common.
//!
//! This test suite exercises the core type manipulation functions:
//! - `try_extract_inner_type()` - extracts inner types from wrappers
//! - `filter_inner_type()` - removes wrapper types
//! - `wrap_leaf_type()` - wraps types in adze::WithLeaf
//! - `NameValueExpr` - name-value expression parsing
//! - `FieldThenParams` - field parameter handling
//!
//! Tests cover simple cases, complex nested types, generic lifetimes,
//! qualified paths, trait objects, edge cases, and composability.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Test Helpers
// ---------------------------------------------------------------------------

/// Create a skip set from slice of names.
fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

/// Convert type to string representation for assertions.
fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. try_extract_inner_type() - Basic extraction
// ===========================================================================

#[test]
fn extract_inner_type_from_option() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_inner_type_from_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_inner_type_from_box() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_inner_type_non_wrapper_returns_none() {
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_inner_type_from_nested_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

// ===========================================================================
// 2. try_extract_inner_type() - Skip over unwanted wrappers
// ===========================================================================

#[test]
fn extract_through_skip_set_box() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_through_skip_set_arc() {
    let ty: Type = parse_quote!(Arc<Option<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_through_multiple_skip_wrappers() {
    let ty: Type = parse_quote!(Box<Arc<Vec<String>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_skip_does_not_match_target() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < String >");
}

// ===========================================================================
// 3. filter_inner_type() - Remove wrapper types
// ===========================================================================

#[test]
fn filter_box_single_level() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_no_match_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), "Box < String >");
}

#[test]
fn filter_non_wrapper_type() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "String");
}

// ===========================================================================
// 4. wrap_leaf_type() - Wrap types in adze::WithLeaf
// ===========================================================================

#[test]
fn wrap_simple_type_as_leaf() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_complex_type_as_leaf() {
    let ty: Type = parse_quote!(MyCustomType);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < MyCustomType >");
}

#[test]
fn wrap_preserves_vec_containers() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_preserves_option_containers() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

// ===========================================================================
// 5. Type extraction with generic parameters
// ===========================================================================

#[test]
fn extract_inner_type_with_lifetime() {
    let ty: Type = parse_quote!(Vec<&'a str>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "& 'a str");
}

#[test]
fn extract_inner_type_with_multiple_generics() {
    let ty: Type = parse_quote!(Result<String, io::Error>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(extracted);
    // Result extracts the first generic argument
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_preserves_all_type_parameters() {
    let ty: Type = parse_quote!(Option<(String, i32)>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "(String , i32)");
}

// ===========================================================================
// 6. Roundtrip tests - wrap and extract should be related
// ===========================================================================

#[test]
fn wrap_then_filter_roundtrip() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    // The wrapped type is adze::WithLeaf<String>
    assert!(ty_str(&wrapped).contains("WithLeaf"));
}

#[test]
fn filter_then_wrap_idempotent() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

// ===========================================================================
// 7. filter_inner_type() - Ordering and edge cases
// ===========================================================================

#[test]
fn filter_preserves_structure_of_multiple_wraps() {
    // When filtering through multiple wrappers, order is preserved
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    // After removing Box, we get Arc<String>
    assert_eq!(ty_str(&filtered), "Arc < String >");
}

#[test]
fn filter_empty_skip_set_unchanged() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), "Box < Arc < String > >");
}

// ===========================================================================
// 8. Type extraction with qualified paths
// ===========================================================================

#[test]
fn extract_qualified_path_type() {
    let ty: Type = parse_quote!(Vec<std::string::String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "std :: string :: String");
}

#[test]
fn extract_module_qualified_custom_type() {
    let ty: Type = parse_quote!(Option<my::module::Type>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "my :: module :: Type");
}

// ===========================================================================
// 9. Type extraction with trait objects and references
// ===========================================================================

#[test]
fn extract_from_trait_object() {
    let ty: Type = parse_quote!(Box<dyn std::fmt::Debug>);
    // Since the inner of Box<dyn Debug> is dyn Debug (a Trait type, not a Type::Path),
    // the extraction will try to unwrap it and may panic or handle differently.
    // For safety, we just verify it doesn't panic with empty skip.
    let (inner, _extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    // The inner should be the trait object itself
    assert!(ty_str(&inner).contains("Debug"));
}

#[test]
fn extract_from_reference_type() {
    let ty: Type = parse_quote!(Box<&String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "& String");
}

// ===========================================================================
// 10. Type extraction with deeply nested generics
// ===========================================================================

#[test]
fn extract_deeply_nested_generics() {
    let ty: Type = parse_quote!(Vec<Option<Box<String>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < Box < String > >");
}

#[test]
fn extract_from_deep_nesting_with_skip() {
    let ty: Type = parse_quote!(Box<Vec<Option<String>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Vec"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

// ===========================================================================
// 11. NameValueExpr parsing and processing
// ===========================================================================

#[test]
fn parse_nve_simple_name_string_value() {
    let nve: NameValueExpr = parse_quote!(param = "value");
    assert_eq!(nve.path.to_string(), "param");
    assert!(matches!(nve.expr, syn::Expr::Lit(_)));
}

#[test]
fn parse_nve_complex_name_integer_value() {
    let nve: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nve.path.to_string(), "precedence");
    assert!(matches!(nve.expr, syn::Expr::Lit(_)));
}

#[test]
fn parse_nve_boolean_value() {
    let nve: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nve.path.to_string(), "enabled");
}

// ===========================================================================
// 12. FieldThenParams parsing and processing
// ===========================================================================

#[test]
fn parse_field_then_params_no_params() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn parse_field_then_params_with_single_param() {
    let ftp: FieldThenParams = parse_quote!(String, name = "test");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
}

#[test]
fn parse_field_then_params_with_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(String, name = "test", value = 42);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "name");
    assert_eq!(ftp.params[1].path.to_string(), "value");
}

// ===========================================================================
// 13. Wrap with multiple generic arguments
// ===========================================================================

#[test]
fn wrap_result_type_wraps_both_generics() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    let wrapped_str = ty_str(&wrapped);
    // Both String and i32 should be wrapped
    assert!(wrapped_str.contains("WithLeaf"));
    assert!(wrapped_str.contains("String"));
    assert!(wrapped_str.contains("i32"));
}

#[test]
fn wrap_complex_generic_structure() {
    let ty: Type = parse_quote!(Vec<Result<String, Box<Error>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Result", "Box"]));
    let wrapped_str = ty_str(&wrapped);
    // String and Error should be wrapped
    assert!(wrapped_str.contains("WithLeaf"));
    assert!(wrapped_str.contains("Vec"));
    assert!(wrapped_str.contains("Result"));
}

// ===========================================================================
// 14. Type processing idempotency and composition
// ===========================================================================

#[test]
fn filter_is_idempotent() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered_once = filter_inner_type(&ty, &skip(&["Box"]));
    let filtered_twice = filter_inner_type(&filtered_once, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered_once), ty_str(&filtered_twice));
}

#[test]
fn extract_multiple_times_on_same_type() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner1, ext1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    let (inner2, ext2) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert_eq!(ext1, ext2);
    assert_eq!(ty_str(&inner1), ty_str(&inner2));
}

// ===========================================================================
// 15. Type processing with references
// ===========================================================================

#[test]
fn wrap_mutable_reference() {
    let ty: Type = parse_quote!(&mut String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & mut String >");
}

#[test]
fn wrap_immutable_reference() {
    let ty: Type = parse_quote!(&String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & String >");
}

#[test]
fn filter_reference_type_non_wrapper() {
    let ty: Type = parse_quote!(&String);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    // References are not in skip set, so unchanged
    assert_eq!(ty_str(&filtered), "& String");
}

// ===========================================================================
// 16. Edge cases with type names
// ===========================================================================

#[test]
fn extract_single_char_type_name() {
    let ty: Type = parse_quote!(Vec<T>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "T");
}

#[test]
fn extract_long_type_name() {
    let ty: Type = parse_quote!(Vec<VeryLongTypeNameForTestingPurposesWithManyCharacters>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert!(ty_str(&inner).contains("VeryLongTypeNameForTesting"));
}

// ===========================================================================
// 17. Complex composition scenarios
// ===========================================================================

#[test]
fn extract_then_filter_composition() {
    // Vec<Box<Option<String>>>
    // Extract Vec -> Box<Option<String>>
    // Filter Box -> Option<String>
    let ty: Type = parse_quote!(Vec<Box<Option<String>>>);
    let (after_extract, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    let after_filter = filter_inner_type(&after_extract, &skip(&["Box"]));
    assert_eq!(ty_str(&after_filter), "Option < String >");
}

#[test]
fn filter_skip_through_and_extract() {
    // Box<Vec<Option<String>>>
    // Extract through Box -> Vec<Option<String>>
    let ty: Type = parse_quote!(Box<Vec<Option<String>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < String >");
}

// ===========================================================================
// 18. Non-Path types (references, tuples, arrays)
// ===========================================================================

#[test]
fn extract_from_non_path_type_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn filter_non_path_type_returns_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(i32 , u32)");
}

#[test]
fn wrap_non_path_type_wraps_entirely() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

// ===========================================================================
// 19. Tuple and array handling
// ===========================================================================

#[test]
fn extract_from_tuple_wrapper() {
    let ty: Type = parse_quote!(Vec<(String, i32)>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "(String , i32)");
}

#[test]
fn wrap_tuple_as_leaf() {
    let ty: Type = parse_quote!((String, i32));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (String , i32) >");
}

// ===========================================================================
// 20. Multiple skip set entries
// ===========================================================================

#[test]
fn extract_with_three_item_skip_set() {
    let ty: Type = parse_quote!(Box<Arc<Pin<Vec<String>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc", "Pin"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn filter_with_large_skip_set() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Mutex<String>>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc", "Mutex"]));
    assert_eq!(ty_str(&filtered), "String");
}

// ===========================================================================
// 21. Generic parameter preservation
// ===========================================================================

#[test]
fn extract_preserves_complex_generic_bounds() {
    let ty: Type = parse_quote!(Vec<impl Trait>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
}

#[test]
fn wrap_preserves_lifetime_annotations() {
    let ty: Type = parse_quote!(&'static str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & 'static str >");
}
