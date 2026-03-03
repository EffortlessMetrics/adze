#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for `try_extract_inner_type` in adze-common.

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
// 1. Basic extraction — Option, Vec, Box
// ===========================================================================

#[test]
fn option_string_extracts_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn vec_i32_extracts_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn box_bool_extracts_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn option_u64_extracts_u64() {
    let ty: Type = parse_quote!(Option<u64>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn vec_char_extracts_char() {
    let ty: Type = parse_quote!(Vec<char>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "char");
}

// ===========================================================================
// 2. Plain type — no extraction
// ===========================================================================

#[test]
fn plain_string_not_extracted() {
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn plain_i32_not_extracted() {
    let ty: Type = parse_quote!(i32);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn plain_bool_empty_skip_set() {
    let ty: Type = parse_quote!(bool);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "bool");
}

// ===========================================================================
// 3. Wrong target — wrapper present but looking for different target
// ===========================================================================

#[test]
fn option_string_target_vec_not_extracted() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Option < String >");
}

#[test]
fn vec_i32_target_option_not_extracted() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn box_u8_target_vec_not_extracted() {
    let ty: Type = parse_quote!(Box<u8>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < u8 >");
}

// ===========================================================================
// 4. Skip-over — skipping wrappers to find target inside
// ===========================================================================

#[test]
fn skip_box_to_find_vec() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn skip_arc_to_find_option() {
    let ty: Type = parse_quote!(Arc<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn skip_box_target_not_inside_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn skip_chain_arc_box_to_find_option() {
    let ty: Type = parse_quote!(Arc<Box<Option<u8>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn skip_chain_three_levels_target_not_found() {
    let ty: Type = parse_quote!(Arc<Box<Rc<String>>>);
    let (inner, extracted) =
        try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box", "Rc"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Arc < Box < Rc < String > > >");
}

// ===========================================================================
// 5. Nested wrappers — target wrapping target, or mixed nesting
// ===========================================================================

#[test]
fn option_option_extracts_outer_only() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn vec_vec_extracts_outer_only() {
    let ty: Type = parse_quote!(Vec<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn option_vec_target_option_extracts_vec() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < bool >");
}

#[test]
fn option_vec_target_vec_skip_option() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

// ===========================================================================
// 6. Result<T, E> — multi-generic target (extracts first arg)
// ===========================================================================

#[test]
fn result_as_target_extracts_first_arg() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn skip_box_to_find_result() {
    let ty: Type = parse_quote!(Box<Result<u32, std::io::Error>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Result", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u32");
}

// ===========================================================================
// 7. Non-path types — references, tuples, arrays
// ===========================================================================

#[test]
fn reference_type_not_extracted() {
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn tuple_type_not_extracted() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "(i32 , u32)");
}

#[test]
fn array_type_not_extracted() {
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "[u8 ; 4]");
}

// ===========================================================================
// 8. Qualified paths — std::option::Option etc.
// ===========================================================================

#[test]
fn qualified_option_extracts_by_last_segment() {
    let ty: Type = parse_quote!(std::option::Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn qualified_vec_extracts_by_last_segment() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn qualified_box_in_skip_set() {
    let ty: Type = parse_quote!(std::boxed::Box<Option<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

// ===========================================================================
// 9. try_extract vs filter_inner_type comparison
// ===========================================================================

#[test]
fn extract_and_filter_agree_on_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    let (extracted, was) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(was);
    assert_eq!(ty_str(&filtered), ty_str(&extracted));
}

#[test]
fn extract_and_filter_agree_on_box_vec_i32() {
    // filter with skip={Box,Vec} unwraps both layers to i32
    // extract with target=Vec skip={Box} also yields i32
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Vec"]));
    let (extracted, was) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(was);
    assert_eq!(ty_str(&filtered), ty_str(&extracted));
}

#[test]
fn extract_returns_original_when_filter_would_not_unwrap() {
    // filter with skip={Box} on plain String returns String
    // extract with target=Box on String returns (String, false)
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let (extracted, was) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!was);
    assert_eq!(ty_str(&filtered), ty_str(&extracted));
}

#[test]
fn filter_unwraps_but_extract_does_not_when_target_differs() {
    // filter with skip={Option} unwraps Option<String> → String
    // extract with target=Vec on Option<String> does NOT extract
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    let (extracted, was) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!was);
    assert_eq!(ty_str(&filtered), "String");
    assert_eq!(ty_str(&extracted), "Option < String >");
}

// ===========================================================================
// 10. Custom wrapper types
// ===========================================================================

#[test]
fn custom_wrapper_as_target() {
    let ty: Type = parse_quote!(MyList<f64>);
    let (inner, extracted) = try_extract_inner_type(&ty, "MyList", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn custom_wrapper_in_skip_set() {
    let ty: Type = parse_quote!(Wrapper<Option<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Wrapper"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

// ===========================================================================
// 11. Edge cases
// ===========================================================================

#[test]
fn extracted_flag_is_false_for_non_matching_generic() {
    // HashMap is not target and not in skip
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

#[test]
fn extract_idempotent_on_plain_type() {
    let ty: Type = parse_quote!(u32);
    let s = &skip(&[]);
    let (first, ext1) = try_extract_inner_type(&ty, "Option", s);
    let (second, ext2) = try_extract_inner_type(&first, "Option", s);
    assert!(!ext1);
    assert!(!ext2);
    assert_eq!(ty_str(&first), ty_str(&second));
}

#[test]
fn double_extract_peels_one_layer_at_a_time() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let s = &skip(&[]);
    let (first, ext1) = try_extract_inner_type(&ty, "Option", s);
    assert!(ext1);
    assert_eq!(ty_str(&first), "Option < i32 >");
    let (second, ext2) = try_extract_inner_type(&first, "Option", s);
    assert!(ext2);
    assert_eq!(ty_str(&second), "i32");
}
