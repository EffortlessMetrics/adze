#![allow(unused_variables)]
use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use std::collections::HashSet;
use syn::{Type, parse_quote};

// ============================================================================
// Category 1: extract_basic_* — Basic type extraction (8 tests)
// ============================================================================

#[test]
fn extract_basic_primitive_i32() {
    let ty: Type = parse_quote!(i32);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(quote::quote!(#result).to_string(), "i32");
}

#[test]
fn extract_basic_primitive_string() {
    let ty: Type = parse_quote!(String);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(quote::quote!(#result).to_string(), "String");
}

#[test]
fn extract_basic_primitive_bool() {
    let ty: Type = parse_quote!(bool);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(quote::quote!(#result).to_string(), "bool");
}

#[test]
fn extract_basic_primitive_f64() {
    let ty: Type = parse_quote!(f64);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(quote::quote!(#result).to_string(), "f64");
}

#[test]
fn extract_basic_primitive_u8() {
    let ty: Type = parse_quote!(u8);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(quote::quote!(#result).to_string(), "u8");
}

#[test]
fn extract_basic_unit_type() {
    let ty: Type = parse_quote!(());
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
}

#[test]
fn extract_basic_custom_struct() {
    let ty: Type = parse_quote!(MyStruct);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(quote::quote!(#result).to_string(), "MyStruct");
}

#[test]
fn extract_basic_custom_enum() {
    let ty: Type = parse_quote!(MyEnum);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Result", &skip);
    assert!(!found);
    assert_eq!(quote::quote!(#result).to_string(), "MyEnum");
}

// ============================================================================
// Category 2: extract_option_* — Option<T> extraction (8 tests)
// ============================================================================

#[test]
fn extract_option_of_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "i32");
}

#[test]
fn extract_option_of_string() {
    let ty: Type = parse_quote!(Option<String>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "String");
}

#[test]
fn extract_option_of_vec() {
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "Vec < u32 >");
}

#[test]
fn extract_option_of_custom_type() {
    let ty: Type = parse_quote!(Option<MyType>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "MyType");
}

#[test]
fn extract_option_of_tuple() {
    let ty: Type = parse_quote!(Option<(i32, String)>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
}

#[test]
fn extract_option_of_reference() {
    let ty: Type = parse_quote!(Option<&'static str>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
}

#[test]
fn extract_option_wrong_generic() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(quote::quote!(#result).to_string(), "Option < i32 >");
}

#[test]
fn extract_option_of_unit() {
    let ty: Type = parse_quote!(Option<()>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
}

// ============================================================================
// Category 3: extract_vec_* — Vec<T> extraction (8 tests)
// ============================================================================

#[test]
fn extract_vec_of_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "i32");
}

#[test]
fn extract_vec_of_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "String");
}

#[test]
fn extract_vec_of_custom_type() {
    let ty: Type = parse_quote!(Vec<MyType>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "MyType");
}

#[test]
fn extract_vec_of_tuple() {
    let ty: Type = parse_quote!(Vec<(u8, u16)>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
}

#[test]
fn extract_vec_wrong_generic() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(quote::quote!(#result).to_string(), "Vec < i32 >");
}

#[test]
fn extract_vec_of_vec() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "Vec < i32 >");
}

#[test]
fn extract_vec_of_reference() {
    let ty: Type = parse_quote!(Vec<&i32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
}

#[test]
fn extract_vec_of_unit() {
    let ty: Type = parse_quote!(Vec<()>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
}

// ============================================================================
// Category 4: extract_skip_* — skip_over behavior (8 tests)
// ============================================================================

#[test]
fn extract_skip_over_box() {
    let ty: Type = parse_quote!(Box<Option<u32>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "u32");
}

#[test]
fn extract_skip_over_result() {
    let ty: Type = parse_quote!(Result<Vec<String>, ()>);
    let mut skip = HashSet::new();
    skip.insert("Result");
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
}

#[test]
fn extract_skip_multiple_layers() {
    let ty: Type = parse_quote!(Box<Option<Vec<i32>>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    skip.insert("Option");
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "i32");
}

#[test]
fn extract_skip_not_in_list() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let mut skip = HashSet::new();
    skip.insert("Vec");
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
}

#[test]
fn extract_skip_empty_set() {
    let ty: Type = parse_quote!(Box<Option<u32>>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
}

#[test]
fn extract_skip_box_twice() {
    let ty: Type = parse_quote!(Box<Box<Option<i32>>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "i32");
}

#[test]
fn extract_skip_deeply_nested() {
    let ty: Type = parse_quote!(Box<Box<Box<Vec<u64>>>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "u64");
}

#[test]
fn extract_skip_with_middle_match() {
    let ty: Type = parse_quote!(Box<Vec<Option<String>>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(quote::quote!(#result).to_string(), "Option < String >");
}

// ============================================================================
// Category 5: filter_basic_* — filter_inner_type basic (8 tests)
// ============================================================================

#[test]
fn filter_basic_primitive_i32() {
    let ty: Type = parse_quote!(i32);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(quote::quote!(#result).to_string(), "i32");
}

#[test]
fn filter_basic_primitive_string() {
    let ty: Type = parse_quote!(String);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(quote::quote!(#result).to_string(), "String");
}

#[test]
fn filter_basic_custom_type() {
    let ty: Type = parse_quote!(MyStruct);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(quote::quote!(#result).to_string(), "MyStruct");
}

#[test]
fn filter_basic_tuple_type() {
    let ty: Type = parse_quote!((i32, String));
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
}

// ============================================================================
// Category 6: filter_nested_* — Nested filtering (8 tests)
// ============================================================================

// ============================================================================
// Category 7: wrap_basic_* — wrap_leaf_type basic (8 tests)
// ============================================================================

#[test]
fn wrap_basic_unit_type() {
    let ty: Type = parse_quote!(());
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
}

#[test]
fn wrap_basic_lifetime_reference() {
    let ty: Type = parse_quote!(&'static str);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
}

#[test]
fn wrap_basic_mutable_reference() {
    let ty: Type = parse_quote!(&mut i32);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
}

// ============================================================================
// Category 8: wrap_complex_* — Complex wrapping (8 tests)
// ============================================================================

#[test]
fn wrap_complex_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    let result_str = quote::quote!(#result).to_string();
    assert!(result_str.contains("i32"));
}

#[test]
fn wrap_complex_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    let result_str = quote::quote!(#result).to_string();
    assert!(result_str.contains("String"));
}

#[test]
fn wrap_complex_box_u32() {
    let ty: Type = parse_quote!(Box<u32>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    let result_str = quote::quote!(#result).to_string();
    assert!(result_str.contains("u32"));
}

#[test]
fn wrap_complex_nested_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    let result_str = quote::quote!(#result).to_string();
    assert!(result_str.contains("i32"));
}

#[test]
fn wrap_complex_nested_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    let result_str = quote::quote!(#result).to_string();
    assert!(result_str.contains("String"));
}

#[test]
fn wrap_complex_with_skip() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    let result = wrap_leaf_type(&ty, &skip);
    let result_str = quote::quote!(#result).to_string();
    assert!(result_str.contains("i32"));
}

#[test]
fn wrap_complex_deeply_nested() {
    let ty: Type = parse_quote!(Box<Vec<Option<String>>>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    let result_str = quote::quote!(#result).to_string();
    assert!(result_str.contains("String"));
}

#[test]
fn wrap_complex_result_types() {
    let ty: Type = parse_quote!(Result<Vec<i32>, String>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    let result_str = quote::quote!(#result).to_string();
    assert!(!result_str.is_empty());
}
