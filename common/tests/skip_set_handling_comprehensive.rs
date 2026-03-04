#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for skip set and type filtering in adze-common.
//!
//! Covers: default skip set behavior (Vec, Option, Box), custom skip sets,
//! multi-layer skipping, non-skippable types, mixed skip/non-skip,
//! skip set interaction with `wrap_leaf_type`, and edge cases.

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
// 1. Default skip set behavior (Vec, Option, Box)
// ===========================================================================

#[test]
fn default_skip_extract_vec_through_box() {
    let s = skip(&["Box", "Option"]);
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn default_skip_extract_vec_through_option() {
    let s = skip(&["Box", "Option"]);
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn default_skip_extract_option_through_box() {
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &s);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn default_skip_filter_box_option_nested() {
    let s = skip(&["Box", "Option"]);
    let ty: Type = parse_quote!(Box<Option<f64>>);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "f64");
}

#[test]
fn default_skip_filter_vec_not_in_skip() {
    // Vec is NOT in the skip set, so it stays.
    let s = skip(&["Box", "Option"]);
    let ty: Type = parse_quote!(Vec<String>);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "Vec < String >");
}

// ===========================================================================
// 2. Custom skip set patterns
// ===========================================================================

#[test]
fn custom_skip_arc() {
    let s = skip(&["Arc"]);
    let ty: Type = parse_quote!(Arc<Vec<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn custom_skip_rc_and_cell() {
    let s = skip(&["Rc", "Cell"]);
    let ty: Type = parse_quote!(Rc<Cell<Vec<u32>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn custom_skip_filter_mutex_rwlock() {
    let s = skip(&["Mutex", "RwLock"]);
    let ty: Type = parse_quote!(Mutex<RwLock<i64>>);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "i64");
}

#[test]
fn custom_skip_single_wrapper_filter() {
    let s = skip(&["Pin"]);
    let ty: Type = parse_quote!(Pin<Future>);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "Future");
}

// ===========================================================================
// 3. Skip through multiple layers
// ===========================================================================

#[test]
fn multi_layer_skip_three_deep_extract() {
    let s = skip(&["Box", "Arc", "Rc"]);
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<u16>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &s);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn multi_layer_skip_three_deep_filter() {
    let s = skip(&["Box", "Arc", "Rc"]);
    let ty: Type = parse_quote!(Box<Arc<Rc<String>>>);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn multi_layer_extract_stops_at_target() {
    // Skip Box and Option; extract Vec. The inner type of Vec is returned.
    let s = skip(&["Box", "Option"]);
    let ty: Type = parse_quote!(Box<Option<Vec<char>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn multi_layer_filter_partial_skip() {
    // Only Box is in skip set; Option stays.
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "Option < i32 >");
}

// ===========================================================================
// 4. Skip with non-skippable types
// ===========================================================================

#[test]
fn non_skippable_extract_returns_false() {
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

#[test]
fn non_skippable_filter_returns_unchanged() {
    let s = skip(&["Box", "Option"]);
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn extract_target_not_inside_skip_chain() {
    // Box<String> — looking for Vec through Box, String is not Vec.
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn non_path_type_extract_returns_unchanged() {
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn non_path_type_filter_returns_unchanged() {
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!((u8, u16));
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "(u8 , u16)");
}

// ===========================================================================
// 5. Mixed skip/non-skip types
// ===========================================================================

#[test]
fn mixed_skip_extracts_through_skip_only() {
    // Skip = [Box], type = Box<Result<Vec<u8>, Error>>
    // Box is skipped, Result is NOT in skip set and is not target Vec → not extracted.
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(Box<Result<Vec<u8>, Error>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < Result < Vec < u8 > , Error > >");
}

#[test]
fn mixed_filter_strips_only_skip_set_members() {
    // Skip = [Box]; Box<Vec<String>> → filter removes Box, Vec stays.
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "Vec < String >");
}

#[test]
fn mixed_skip_interleaved_skip_nonskip_extract_fails() {
    // Box<HashMap<String, Vec<i32>>> — Box skipped, HashMap not skippable → no extraction
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(Box<HashMap<String, Vec<i32>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < HashMap < String , Vec < i32 > > >");
}

// ===========================================================================
// 6. Skip set interaction with wrap_leaf_type
// ===========================================================================

#[test]
fn wrap_skips_vec_wraps_inner() {
    let s = skip(&["Vec"]);
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &s);
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_skips_option_wraps_inner() {
    let s = skip(&["Option"]);
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &s);
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_skips_nested_vec_option() {
    let s = skip(&["Vec", "Option"]);
    let ty: Type = parse_quote!(Vec<Option<Foo>>);
    let wrapped = wrap_leaf_type(&ty, &s);
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < Foo > > >"
    );
}

#[test]
fn wrap_no_skip_wraps_entire_vec() {
    // With empty skip set, Vec itself gets wrapped.
    let s = skip(&[]);
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &s);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < String > >");
}

#[test]
fn wrap_custom_skip_result_wraps_both_args() {
    let s = skip(&["Result"]);
    let ty: Type = parse_quote!(Result<String, Error>);
    let wrapped = wrap_leaf_type(&ty, &s);
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < Error > >"
    );
}

#[test]
fn wrap_skip_box_nested_in_vec() {
    let s = skip(&["Vec", "Box"]);
    let ty: Type = parse_quote!(Vec<Box<u64>>);
    let wrapped = wrap_leaf_type(&ty, &s);
    assert_eq!(ty_str(&wrapped), "Vec < Box < adze :: WithLeaf < u64 > > >");
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn edge_empty_skip_set_extract_direct_match() {
    // No skip set, but direct match on target.
    let s = skip(&[]);
    let ty: Type = parse_quote!(Vec<bool>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn edge_empty_skip_set_filter_noop() {
    let s = skip(&[]);
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "Box < String >");
}

#[test]
fn edge_single_skip_same_as_target_extract() {
    // Skip = [Vec], target = Vec → direct match wins over skip.
    let s = skip(&["Vec"]);
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    // The target match is checked first, so it extracts.
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn edge_all_skipped_filter_deeply_nested() {
    let s = skip(&["A", "B", "C"]);
    let ty: Type = parse_quote!(A<B<C<Leaf>>>);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "Leaf");
}

#[test]
fn edge_extract_from_plain_type_no_generics() {
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(u32);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn edge_wrap_non_path_reference_type() {
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(&mut i32);
    let wrapped = wrap_leaf_type(&ty, &s);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & mut i32 >");
}

#[test]
fn edge_filter_target_itself_in_skip_set() {
    // If the outermost type is in skip set, filter peels it.
    let s = skip(&["Option"]);
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &s);
    assert_eq!(ty_str(&filtered), "String");
}
