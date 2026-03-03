#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for `filter_inner_type` (and its relationship with
//! `try_extract_inner_type`) in adze-common.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type};
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
// 1. Single-wrapper unwrapping
// ===========================================================================

#[test]
fn option_string_unwraps_to_string() {
    let ty: Type = parse_quote!(Option<String>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn vec_i32_unwraps_to_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn box_bool_unwraps_to_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn arc_u64_unwraps_to_u64() {
    let ty: Type = parse_quote!(Arc<u64>);
    let result = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&result), "u64");
}

#[test]
fn rc_f32_unwraps_to_f32() {
    let ty: Type = parse_quote!(Rc<f32>);
    let result = filter_inner_type(&ty, &skip(&["Rc"]));
    assert_eq!(ty_str(&result), "f32");
}

// ===========================================================================
// 2. Plain type (no wrapper) — returned unchanged
// ===========================================================================

#[test]
fn plain_string_returned_unchanged() {
    let ty: Type = parse_quote!(String);
    let result = filter_inner_type(&ty, &skip(&["Option", "Vec", "Box"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn plain_i32_returned_unchanged() {
    let ty: Type = parse_quote!(i32);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn plain_bool_empty_skip_set() {
    let ty: Type = parse_quote!(bool);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "bool");
}

// ===========================================================================
// 3. Wrapper NOT in skip set — returned unchanged
// ===========================================================================

#[test]
fn option_not_in_skip_set_returned_unchanged() {
    let ty: Type = parse_quote!(Option<String>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "Option < String >");
}

#[test]
fn vec_not_in_skip_set_returned_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

// ===========================================================================
// 4. Nested wrappers — multiple levels
// ===========================================================================

#[test]
fn option_vec_both_in_skip_unwraps_fully() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let result = filter_inner_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn box_arc_option_triple_nesting_unwraps() {
    let ty: Type = parse_quote!(Box<Arc<Option<u8>>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Arc", "Option"]));
    assert_eq!(ty_str(&result), "u8");
}

#[test]
fn nested_partial_skip_stops_at_first_non_skip() {
    // Only Box is in skip, so unwrap Box then stop at Vec
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Vec < String >");
}

#[test]
fn option_option_double_nesting() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn vec_vec_double_nesting() {
    let ty: Type = parse_quote!(Vec<Vec<bool>>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn deeply_nested_four_levels() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<char>>>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc", "Option"]));
    assert_eq!(ty_str(&result), "char");
}

// ===========================================================================
// 5. Custom wrapper types
// ===========================================================================

#[test]
fn custom_wrapper_mybox_unwraps() {
    let ty: Type = parse_quote!(MyBox<String>);
    let result = filter_inner_type(&ty, &skip(&["MyBox"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn custom_wrapper_not_in_skip_unchanged() {
    let ty: Type = parse_quote!(MyBox<String>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "MyBox < String >");
}

#[test]
fn custom_nested_wrappers() {
    let ty: Type = parse_quote!(Wrapper<Container<u32>>);
    let result = filter_inner_type(&ty, &skip(&["Wrapper", "Container"]));
    assert_eq!(ty_str(&result), "u32");
}

// ===========================================================================
// 6. Generic types with multiple params (only first arg extracted)
// ===========================================================================

#[test]
fn result_in_skip_extracts_first_param() {
    // filter_inner_type uses .first().unwrap() so it extracts the first generic arg
    let ty: Type = parse_quote!(Result<String, Error>);
    let result = filter_inner_type(&ty, &skip(&["Result"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn hashmap_in_skip_extracts_first_param() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let result = filter_inner_type(&ty, &skip(&["HashMap"]));
    assert_eq!(ty_str(&result), "String");
}

// ===========================================================================
// 7. Non-path types — returned unchanged
// ===========================================================================

#[test]
fn reference_type_returned_unchanged() {
    let ty: Type = parse_quote!(&str);
    let result = filter_inner_type(&ty, &skip(&["Option", "Box"]));
    assert_eq!(ty_str(&result), "& str");
}

#[test]
fn tuple_type_returned_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "(i32 , u32)");
}

#[test]
fn array_type_returned_unchanged() {
    let ty: Type = parse_quote!([u8; 4]);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "[u8 ; 4]");
}

// ===========================================================================
// 8. filter_inner_type vs try_extract_inner_type — behavioral comparison
// ===========================================================================

#[test]
fn try_extract_option_string_extracts() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn try_extract_wrong_target_not_extracted() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Option < String >");
}

#[test]
fn try_extract_skips_over_box_to_find_vec() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn try_extract_skip_but_target_not_inside() {
    // Box is in skip_over, but inside is String, not Vec
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!extracted);
    // Returns original when target not found
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn filter_and_extract_agree_on_simple_unwrap() {
    // When skip_over contains "Option" and target is "Option",
    // both functions should yield the same inner type.
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    let (extracted, was) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(was);
    assert_eq!(ty_str(&filtered), ty_str(&extracted));
}

#[test]
fn try_extract_reference_type_not_extracted() {
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn try_extract_deep_skip_chain() {
    // Arc<Box<Option<u8>>> — skip Arc and Box, target Option
    let ty: Type = parse_quote!(Arc<Box<Option<u8>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u8");
}

// ===========================================================================
// 9. Edge cases and mixed scenarios
// ===========================================================================

#[test]
fn qualified_path_inner_type_not_stripped() {
    // std::option::Option is a multi-segment path; filter checks last segment
    let ty: Type = parse_quote!(std::option::Option<String>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn filter_idempotent_on_plain_type() {
    let ty: Type = parse_quote!(String);
    let s = &skip(&["Option", "Vec", "Box"]);
    let first = filter_inner_type(&ty, s);
    let second = filter_inner_type(&first, s);
    assert_eq!(ty_str(&first), ty_str(&second));
}

#[test]
fn filter_idempotent_after_full_unwrap() {
    let ty: Type = parse_quote!(Box<i32>);
    let s = &skip(&["Box"]);
    let first = filter_inner_type(&ty, s);
    let second = filter_inner_type(&first, s);
    assert_eq!(ty_str(&first), "i32");
    assert_eq!(ty_str(&second), "i32");
}
