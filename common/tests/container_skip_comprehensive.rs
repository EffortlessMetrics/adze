#![allow(clippy::needless_range_loop)]

//! Comprehensive integration tests for container type handling and skip sets
//! in adze-common.
//!
//! Tests the three main functions:
//! - `try_extract_inner_type(ty, target, skip_set)`: Extract a specific type from containers
//! - `filter_inner_type(ty, skip_set)`: Unwrap all container types in skip set
//! - `wrap_leaf_type(ty, skip_set)`: Wrap leaf types with adze::WithLeaf
//!
//! Tests cover:
//! - Basic container extraction (Option, Vec, Box)
//! - Nested container scenarios
//! - Skip set handling and chaining
//! - Non-container types (references, tuples, arrays)
//! - Type preservation during transformations
//! - Interaction between the three functions

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1-5. Basic extraction from single-layer containers
// ===========================================================================

#[test]
fn test_1_extract_from_option_string() {
    // Test: try_extract_inner_type from Option<String> → Some(String)
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_2_extract_from_vec_i32() {
    // Test: try_extract_inner_type from Vec<i32> → Some(i32)
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_3_extract_from_box_mytype() {
    // Test: try_extract_inner_type from Box<MyType> → Some(MyType)
    let ty: Type = parse_quote!(Box<MyType>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "MyType");
}

#[test]
fn test_4_plain_string_returns_none() {
    // Test: try_extract_inner_type from plain String → None
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_5_nested_option_vec_extracts_first_layer() {
    // Test: try_extract_inner_type from nested Option<Vec<T>>
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < bool >");
}

// ===========================================================================
// 6-8. Filter inner type tests
// ===========================================================================

#[test]
fn test_6_filter_matches_target_type() {
    // Test: filter_inner_type matches target type
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn test_7_filter_no_match_returns_original() {
    // Test: filter_inner_type no match → returns original unchanged
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&filtered), "Box < String >");
}

#[test]
fn test_8_filter_chained_unwrap() {
    // Test: filter_inner_type with chained skip set
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Vec"]));
    assert_eq!(ty_str(&filtered), "i32");
}

// ===========================================================================
// 9-10. Wrap leaf type tests
// ===========================================================================

#[test]
fn test_9_wrap_leaf_type_string() {
    // Test: wrap_leaf_type with String
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn test_10_wrap_leaf_type_custom_type() {
    // Test: wrap_leaf_type with custom type
    let ty: Type = parse_quote!(CustomType);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < CustomType >");
}

// ===========================================================================
// 11-12. NameValueExpr and FieldThenParams parsing
// ===========================================================================

#[test]
fn test_11_name_value_expr_simple() {
    // Test: NameValueExpr parsing simple name=value
    use adze_common::NameValueExpr;
    let expr: NameValueExpr = parse_quote!(key = "value");
    assert_eq!(expr.path.to_string(), "key");
}

#[test]
fn test_12_field_then_params_basic() {
    // Test: FieldThenParams parsing
    use adze_common::FieldThenParams;
    let ftp: FieldThenParams = parse_quote!(Type);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

// ===========================================================================
// 13-16. Container type detection
// ===========================================================================

#[test]
fn test_13_container_option_detection() {
    // Test: Container type Option detection
    let ty: Type = parse_quote!(Option<i32>);
    let (_, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
}

#[test]
fn test_14_container_vec_detection() {
    // Test: Container type Vec detection
    let ty: Type = parse_quote!(Vec<String>);
    let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
}

#[test]
fn test_15_container_box_detection() {
    // Test: Container type Box detection
    let ty: Type = parse_quote!(Box<bool>);
    let (_, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(extracted);
}

#[test]
fn test_16_non_container_type_handling() {
    // Test: Non-container type handling
    let ty: Type = parse_quote!(i32);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "i32");
}

// ===========================================================================
// 17-22. Advanced type scenarios
// ===========================================================================

#[test]
fn test_17_type_with_lifetime_parameter() {
    // Test: Type with lifetime parameter
    let ty: Type = parse_quote!(&'a str);
    let (_inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    // Reference types are not Type::Path, so they're not extracted
}

#[test]
fn test_18_type_with_multiple_generic_params() {
    // Test: Type with multiple generic params (Result<T, E>)
    let ty: Type = parse_quote!(Result<String, i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(extracted);
    // Extract gets the first arg
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_19_qualified_path_type_extraction() {
    // Test: Qualified path type extraction
    let ty: Type = parse_quote!(std::option::Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_20_reference_type_handling() {
    // Test: Reference type handling (&str)
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn test_21_tuple_type_handling() {
    // Test: Tuple type handling
    let ty: Type = parse_quote!((i32, u32));
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "(i32 , u32)");
}

#[test]
fn test_22_array_type_handling() {
    // Test: Array type handling
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "[u8 ; 4]");
}

// ===========================================================================
// 23-25. Interaction and composition tests
// ===========================================================================

#[test]
fn test_23_type_extraction_preserves_inner_generics() {
    // Test: Type extraction preserves inner generics
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    // Inner should be Vec<String> with its own generics preserved
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn test_24_wrap_then_extract_roundtrip() {
    // Test: Wrap then extract roundtrip
    let ty: Type = parse_quote!(String);
    let skip_vec = skip(&["Vec"]);

    // First wrap the type
    let wrapped = wrap_leaf_type(&ty, &skip_vec);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");

    // Wrapping with non-matching skip set should still wrap
    assert!(ty_str(&wrapped).contains("WithLeaf"));
}

#[test]
fn test_25_filter_skip_chaining_order() {
    // Test: Filter preserves original ordering in chained skip set
    let ty: Type = parse_quote!(Box<Vec<Option<i32>>>);

    // Unwrap in order: Box, then Vec, then Option
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Vec", "Option"]));
    assert_eq!(ty_str(&filtered), "i32");
}

// ===========================================================================
// Additional integration and edge case tests
// ===========================================================================

#[test]
fn test_26_skip_over_to_find_nested_target() {
    // Test: Skip set helps find nested targets
    let ty: Type = parse_quote!(Box<Arc<Vec<String>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_27_wrap_preserves_container_structure() {
    // Test: wrap_leaf_type preserves container structure
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    // Should wrap the inner String, keep Vec structure
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn test_28_filter_with_empty_skip_set() {
    // Test: filter_inner_type with empty skip set
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    // Nothing to unwrap
    assert_eq!(ty_str(&filtered), "Box < String >");
}

#[test]
fn test_29_extract_vs_filter_consistency() {
    // Test: Extract and filter are consistent
    let ty: Type = parse_quote!(Vec<Option<i32>>);

    // Extract Vec from Vec<Option<i32>> with empty skip
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);

    // Filter with Vec in skip set
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));

    // Both should get Option<i32>
    assert_eq!(ty_str(&inner), "Option < i32 >");
    assert_eq!(ty_str(&filtered), "Option < i32 >");
}

#[test]
fn test_30_wrap_then_filter_idempotent() {
    // Test: Wrapping is idempotent with filtering
    let ty: Type = parse_quote!(i32);
    let skip_vec = skip(&["Vec"]);

    // Wrap with no-skip
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert!(ty_str(&wrapped).contains("WithLeaf"));

    // Filter shouldn't affect wrapped types
    let filtered = filter_inner_type(&wrapped, &skip_vec);
    assert!(ty_str(&filtered).contains("WithLeaf"));
}

#[test]
fn test_31_complex_nested_structure() {
    // Test: Complex nested structure handling
    let ty: Type = parse_quote!(Option<Box<Vec<Result<String, i32>>>>);

    // Skip Option and Box to find Vec
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Option", "Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Result < String , i32 >");
}

#[test]
fn test_32_name_value_expr_with_complex_value() {
    // Test: NameValueExpr parsing complex value
    use adze_common::NameValueExpr;
    let expr: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(expr.path.to_string(), "precedence");
}

#[test]
fn test_33_field_then_params_with_multiple_params() {
    // Test: FieldThenParams with multiple parameters
    use adze_common::FieldThenParams;
    let ftp: FieldThenParams = parse_quote!(Type, name = "test", value = 42);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 2);
}

#[test]
fn test_34_arc_as_skip_container() {
    // Test: Arc (another smart pointer) as skip container
    let ty: Type = parse_quote!(Arc<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_35_wrap_option_skipped() {
    // Test: wrap_leaf_type skips Option container
    let ty: Type = parse_quote!(Option<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    // Option is skipped, so wrap its inner String
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < String > >");
}

#[test]
fn test_36_multiple_extractions_layered() {
    // Test: Multiple sequential extractions
    let ty: Type = parse_quote!(Option<Option<Option<i32>>>);

    let (inner1, ext1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ext1);
    assert_eq!(ty_str(&inner1), "Option < Option < i32 > >");

    let (inner2, ext2) = try_extract_inner_type(&inner1, "Option", &skip(&[]));
    assert!(ext2);
    assert_eq!(ty_str(&inner2), "Option < i32 >");

    let (inner3, ext3) = try_extract_inner_type(&inner2, "Option", &skip(&[]));
    assert!(ext3);
    assert_eq!(ty_str(&inner3), "i32");
}
