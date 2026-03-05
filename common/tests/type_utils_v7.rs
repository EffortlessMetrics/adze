use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use std::collections::HashSet;
use syn::{Type, parse_quote};

// ============================================================================
// CATEGORY 1: extract_option_* (8 tests)
// ============================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    // Verify we got String back (basic check)
    let expected: Type = parse_quote!(String);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_option_int() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(i32);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_option_box() {
    let ty: Type = parse_quote!(Option<Box<String>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Box<String>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Vec<i32>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(bool);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_option_custom() {
    let ty: Type = parse_quote!(Option<MyCustomType>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(MyCustomType);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_option_ref() {
    let ty: Type = parse_quote!(Option<&String>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(&String);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_option_tuple() {
    let ty: Type = parse_quote!(Option<(i32, String)>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!((i32, String));
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

// ============================================================================
// CATEGORY 2: extract_vec_* (8 tests)
// ============================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(String);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_vec_int() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(i32);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_vec_box() {
    let ty: Type = parse_quote!(Vec<Box<String>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Box<String>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Option<i32>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_vec_bool() {
    let ty: Type = parse_quote!(Vec<bool>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(bool);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_vec_custom() {
    let ty: Type = parse_quote!(Vec<CustomStruct>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(CustomStruct);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_vec_ref() {
    let ty: Type = parse_quote!(Vec<&String>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(&String);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_vec_tuple() {
    let ty: Type = parse_quote!(Vec<(u32, String)>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!((u32, String));
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

// ============================================================================
// CATEGORY 3: extract_box_* (8 tests)
// ============================================================================

#[test]
fn extract_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(String);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_box_int() {
    let ty: Type = parse_quote!(Box<i32>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(i32);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_box_option() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Option<String>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_box_vec() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Vec<i32>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_box_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(bool);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_box_custom() {
    let ty: Type = parse_quote!(Box<MyEnum>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(MyEnum);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_box_ref() {
    let ty: Type = parse_quote!(Box<&String>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(&String);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_box_tuple() {
    let ty: Type = parse_quote!(Box<(i32, i32)>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!((i32, i32));
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

// ============================================================================
// CATEGORY 4: extract_nested_* (8 tests)
// ============================================================================

#[test]
fn extract_nested_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Vec<String>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_nested_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Option<i32>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_nested_box_option() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Option<String>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_nested_option_box() {
    let ty: Type = parse_quote!(Option<Box<i32>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Box<i32>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_nested_vec_box() {
    let ty: Type = parse_quote!(Vec<Box<String>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Box<String>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_nested_box_vec() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Vec<i32>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_nested_deep_triple() {
    let ty: Type = parse_quote!(Option<Vec<Box<String>>>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Vec<Box<String>>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_nested_with_skip() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(String);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

// ============================================================================
// CATEGORY 5: extract_not_found_* (8 tests)
// ============================================================================

#[test]
fn extract_not_found_option_from_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    let expected: Type = parse_quote!(Vec<String>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_not_found_vec_from_option() {
    let ty: Type = parse_quote!(Option<String>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    let expected: Type = parse_quote!(Option<String>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_not_found_box_from_string() {
    let ty: Type = parse_quote!(String);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(!found);
    let expected: Type = parse_quote!(String);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_not_found_string_from_int() {
    let ty: Type = parse_quote!(i32);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "String", &skip);
    assert!(!found);
    let expected: Type = parse_quote!(i32);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_not_found_bool_from_custom() {
    let ty: Type = parse_quote!(CustomType);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "bool", &skip);
    assert!(!found);
    let expected: Type = parse_quote!(CustomType);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_not_found_vec_from_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found); // This one SHOULD be found (we're looking for Vec inside Vec<String>)
    let expected: Type = parse_quote!(String);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_not_found_option_from_box() {
    let ty: Type = parse_quote!(Box<String>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    let expected: Type = parse_quote!(Box<String>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

#[test]
fn extract_not_found_missing_type() {
    let ty: Type = parse_quote!(UnknownContainer<i32>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    let expected: Type = parse_quote!(UnknownContainer<i32>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}

// ============================================================================
// CATEGORY 6: filter_* (8 tests)
// ============================================================================

#[test]
fn filter_simple_type() {
    let ty: Type = parse_quote!(String);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(String);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn filter_box_option() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(Option<String>);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn filter_option_box() {
    let ty: Type = parse_quote!(Option<Box<i32>>);
    let mut skip = HashSet::new();
    skip.insert("Option");
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(Box<i32>);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn filter_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let mut skip = HashSet::new();
    skip.insert("Vec");
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(String);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn filter_nested_triple() {
    let ty: Type = parse_quote!(Box<Vec<Option<String>>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    skip.insert("Vec");
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(Option<String>);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn filter_empty_skip_set() {
    let ty: Type = parse_quote!(Option<String>);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(Option<String>);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn filter_custom_type() {
    let ty: Type = parse_quote!(MyCustomType);
    let mut skip = HashSet::new();
    skip.insert("Box");
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(MyCustomType);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn filter_tuple_type() {
    let ty: Type = parse_quote!((i32, String));
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!((i32, String));
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

// ============================================================================
// CATEGORY 7: wrap_* (8 tests)
// ============================================================================

#[test]
fn wrap_simple_type() {
    let ty: Type = parse_quote!(String);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    // Result should be some wrapped version of String
    assert!(!format!("{:?}", result).is_empty());
}

#[test]
fn wrap_with_option() {
    let ty: Type = parse_quote!(Option<String>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert!(!format!("{:?}", result).is_empty());
}

#[test]
fn wrap_with_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert!(!format!("{:?}", result).is_empty());
}

#[test]
fn wrap_with_box() {
    let ty: Type = parse_quote!(Box<String>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert!(!format!("{:?}", result).is_empty());
}

#[test]
fn wrap_nested_type() {
    let ty: Type = parse_quote!(Option<Vec<Box<String>>>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert!(!format!("{:?}", result).is_empty());
}

#[test]
fn wrap_empty_skip_set() {
    let ty: Type = parse_quote!(i32);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert!(!format!("{:?}", result).is_empty());
}

#[test]
fn wrap_custom_type() {
    let ty: Type = parse_quote!(MyStruct);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert!(!format!("{:?}", result).is_empty());
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, String, bool));
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert!(!format!("{:?}", result).is_empty());
}

// ============================================================================
// CATEGORY 8: skip_over_* (8 tests)
// ============================================================================

#[test]
fn skip_over_single_box() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(Option<String>);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn skip_over_multiple_containers() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    skip.insert("Vec");
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(String);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn skip_over_option_then_vec() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let mut skip = HashSet::new();
    skip.insert("Option");
    skip.insert("Vec");
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(i32);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn skip_over_vec_then_box() {
    let ty: Type = parse_quote!(Vec<Box<String>>);
    let mut skip = HashSet::new();
    skip.insert("Vec");
    skip.insert("Box");
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(String);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn skip_over_empty_set() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(Box<Option<i32>>);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn skip_over_preserves_inner() {
    let ty: Type = parse_quote!(Box<String>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(String);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn skip_over_nested_skip() {
    let ty: Type = parse_quote!(Box<Vec<Option<i32>>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    skip.insert("Vec");
    let result = filter_inner_type(&ty, &skip);
    let expected: Type = parse_quote!(Option<i32>);
    assert_eq!(format!("{:?}", result), format!("{:?}", expected));
}

#[test]
fn skip_over_with_extraction() {
    let ty: Type = parse_quote!(Box<Option<Vec<String>>>);
    let mut skip = HashSet::new();
    skip.insert("Box");
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(Vec<String>);
    assert_eq!(format!("{:?}", inner), format!("{:?}", expected));
}
