//! Comprehensive tests for `try_extract_inner_type` covering extraction from
//! generic containers, nested types, qualified paths, and edge cases.

use std::collections::HashSet;

use adze_common::try_extract_inner_type;
use quote::ToTokens;
use syn::Type;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

fn skip(names: &[&'static str]) -> HashSet<&'static str> {
    names.iter().copied().collect()
}

fn ty(s: &str) -> Type {
    syn::parse_str::<Type>(s).unwrap()
}

fn ty_str(t: &Type) -> String {
    t.to_token_stream().to_string()
}

// ===========================================================================
// 1. Extract from Option<T> for various T (8 tests)
// ===========================================================================

#[test]
fn option_string() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<String>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn option_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<i32>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn option_bool() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<bool>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn option_u64() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<u64>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn option_f64() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<f64>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn option_usize() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<usize>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn option_char() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<char>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn option_custom_struct() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<MyStruct>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "MyStruct");
}

// ===========================================================================
// 2. Extract from Vec<T> for various T (8 tests)
// ===========================================================================

#[test]
fn vec_string() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<String>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn vec_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<i32>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn vec_bool() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<bool>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn vec_u8() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<u8>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn vec_f32() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<f32>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn vec_isize() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<isize>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "isize");
}

#[test]
fn vec_u128() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<u128>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "u128");
}

#[test]
fn vec_custom_enum() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<MyEnum>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "MyEnum");
}

// ===========================================================================
// 3. Extract from Box<T> for various T (8 tests)
// ===========================================================================

#[test]
fn box_string() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<String>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn box_i64() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<i64>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "i64");
}

#[test]
fn box_bool() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<bool>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn box_u16() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<u16>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn box_f64() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<f64>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn box_char() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<char>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn box_usize() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<usize>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn box_custom_type() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Widget>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Widget");
}

// ===========================================================================
// 4. Non-extractable types return false (8 tests)
// ===========================================================================

#[test]
fn plain_string_not_extractable() {
    let input = ty("String");
    let (returned, ok) = try_extract_inner_type(&input, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), "String");
}

#[test]
fn plain_i32_not_extractable() {
    let input = ty("i32");
    let (returned, ok) = try_extract_inner_type(&input, "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), "i32");
}

#[test]
fn plain_bool_not_extractable() {
    let input = ty("bool");
    let (returned, ok) = try_extract_inner_type(&input, "Box", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), "bool");
}

#[test]
fn vec_not_option() {
    let input = ty("Vec<i32>");
    let (returned, ok) = try_extract_inner_type(&input, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), "Vec < i32 >");
}

#[test]
fn option_not_vec() {
    let input = ty("Option<String>");
    let (returned, ok) = try_extract_inner_type(&input, "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), "Option < String >");
}

#[test]
fn box_not_option() {
    let input = ty("Box<u8>");
    let (returned, ok) = try_extract_inner_type(&input, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), "Box < u8 >");
}

#[test]
fn hashmap_not_extractable_as_option() {
    let input = ty("HashMap<String, i32>");
    let (returned, ok) = try_extract_inner_type(&input, "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), "HashMap < String , i32 >");
}

#[test]
fn result_not_extractable_as_vec() {
    let input = ty("Result<String, Error>");
    let (returned, ok) = try_extract_inner_type(&input, "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&returned), "Result < String , Error >");
}

// ===========================================================================
// 5. Nested extraction: Option<Vec<T>> extracts Vec<T> (8 tests)
// ===========================================================================

#[test]
fn option_vec_string() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<Vec<String>>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn option_vec_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<Vec<i32>>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn option_box_bool() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<Box<bool>>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < bool >");
}

#[test]
fn vec_option_string() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Option<String>>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < String >");
}

#[test]
fn box_vec_u8() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Vec<u8>>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < u8 >");
}

#[test]
fn option_option_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<Option<i32>>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn vec_box_f64() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Box<f64>>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < f64 >");
}

#[test]
fn box_option_char() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Option<char>>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < char >");
}

// ===========================================================================
// 6. Double extraction: extract twice for nested containers (8 tests)
// ===========================================================================

#[test]
fn double_option_vec_string() {
    let (mid, ok1) = try_extract_inner_type(&ty("Option<Vec<String>>"), "Option", &empty_skip());
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Vec", &empty_skip());
    assert!(ok2);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn double_vec_option_i32() {
    let (mid, ok1) = try_extract_inner_type(&ty("Vec<Option<i32>>"), "Vec", &empty_skip());
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Option", &empty_skip());
    assert!(ok2);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn double_box_option_bool() {
    let (mid, ok1) = try_extract_inner_type(&ty("Box<Option<bool>>"), "Box", &empty_skip());
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Option", &empty_skip());
    assert!(ok2);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn double_option_box_u64() {
    let (mid, ok1) = try_extract_inner_type(&ty("Option<Box<u64>>"), "Option", &empty_skip());
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Box", &empty_skip());
    assert!(ok2);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn double_option_option_f32() {
    let (mid, ok1) = try_extract_inner_type(&ty("Option<Option<f32>>"), "Option", &empty_skip());
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Option", &empty_skip());
    assert!(ok2);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn double_vec_vec_u8() {
    let (mid, ok1) = try_extract_inner_type(&ty("Vec<Vec<u8>>"), "Vec", &empty_skip());
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Vec", &empty_skip());
    assert!(ok2);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn double_box_vec_char() {
    let (mid, ok1) = try_extract_inner_type(&ty("Box<Vec<char>>"), "Box", &empty_skip());
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Vec", &empty_skip());
    assert!(ok2);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn double_extract_second_fails_on_plain() {
    let (mid, ok1) = try_extract_inner_type(&ty("Option<String>"), "Option", &empty_skip());
    assert!(ok1);
    assert_eq!(ty_str(&mid), "String");
    let (_returned, ok2) = try_extract_inner_type(&mid, "Vec", &empty_skip());
    assert!(!ok2);
}

// ===========================================================================
// 7. Qualified path extraction: std::option::Option<T> (8 tests)
// ===========================================================================

#[test]
fn qualified_option_string() {
    let (inner, ok) =
        try_extract_inner_type(&ty("std::option::Option<String>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn qualified_option_i32() {
    let (inner, ok) =
        try_extract_inner_type(&ty("std::option::Option<i32>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn qualified_vec_bool() {
    let (inner, ok) = try_extract_inner_type(&ty("std::vec::Vec<bool>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn qualified_vec_u8() {
    let (inner, ok) = try_extract_inner_type(&ty("std::vec::Vec<u8>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn qualified_box_f64() {
    let (inner, ok) = try_extract_inner_type(&ty("std::boxed::Box<f64>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn qualified_box_char() {
    let (inner, ok) = try_extract_inner_type(&ty("std::boxed::Box<char>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn qualified_option_custom() {
    let (inner, ok) =
        try_extract_inner_type(&ty("core::option::Option<MyType>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "MyType");
}

#[test]
fn qualified_vec_custom() {
    let (inner, ok) = try_extract_inner_type(&ty("alloc::vec::Vec<Widget>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Widget");
}

// ===========================================================================
// 8. Edge cases: unit, never, raw pointers, references, tuples (8 tests)
// ===========================================================================

#[test]
fn unit_type_not_extractable() {
    let input = ty("()");
    let (_returned, ok) = try_extract_inner_type(&input, "Option", &empty_skip());
    assert!(!ok);
}

#[test]
fn reference_type_not_extractable() {
    let input = ty("&str");
    let (_returned, ok) = try_extract_inner_type(&input, "Option", &empty_skip());
    assert!(!ok);
}

#[test]
fn mutable_reference_not_extractable() {
    let input = ty("&mut i32");
    let (_returned, ok) = try_extract_inner_type(&input, "Vec", &empty_skip());
    assert!(!ok);
}

#[test]
fn raw_pointer_not_extractable() {
    let input = ty("*const u8");
    let (_returned, ok) = try_extract_inner_type(&input, "Box", &empty_skip());
    assert!(!ok);
}

#[test]
fn raw_mut_pointer_not_extractable() {
    let input = ty("*mut f64");
    let (_returned, ok) = try_extract_inner_type(&input, "Option", &empty_skip());
    assert!(!ok);
}

#[test]
fn tuple_not_extractable() {
    let input = ty("(i32, String)");
    let (_returned, ok) = try_extract_inner_type(&input, "Option", &empty_skip());
    assert!(!ok);
}

#[test]
fn never_type_not_extractable() {
    let input = ty("!");
    let (_returned, ok) = try_extract_inner_type(&input, "Vec", &empty_skip());
    assert!(!ok);
}

#[test]
fn skip_over_extracts_through_wrapper() {
    // Using skip_over to reach Option through a Box wrapper
    let (inner, ok) = try_extract_inner_type(&ty("Box<Option<String>>"), "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}
