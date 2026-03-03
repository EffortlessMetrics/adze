#![allow(clippy::needless_range_loop)]

//! Comprehensive composability tests for `wrap_leaf_type`, `filter_inner_type`,
//! and `try_extract_inner_type` in adze-common.
//!
//! Tests cover: wrap-then-filter, filter-then-wrap, extract-then-wrap,
//! double-wrap, double-filter, compositions across Vec/Option/Box patterns,
//! identity compositions, and error-resilient (non-path type) compositions.

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
// 1. Wrap then filter
// ===========================================================================

#[test]
fn wrap_then_filter_plain_type_box_skip() {
    // wrap(String, skip=[Box]) => adze::WithLeaf<String>
    // filter(adze::WithLeaf<String>, skip=[Box]) => adze::WithLeaf<String>  (no Box to strip)
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    let filtered = filter_inner_type(&wrapped, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_then_filter_vec_string() {
    // wrap(Vec<String>, skip=[Vec]) => Vec<adze::WithLeaf<String>>
    // filter(Vec<adze::WithLeaf<String>>, skip=[Vec]) => adze::WithLeaf<String>
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < adze :: WithLeaf < String > >"
    );
    let filtered = filter_inner_type(&wrapped, &skip(&["Vec"]));
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_then_filter_option_i32() {
    // wrap(Option<i32>, skip=[Option]) => Option<adze::WithLeaf<i32>>
    // filter(Option<...>, skip=[Option]) => adze::WithLeaf<i32>
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    let filtered = filter_inner_type(&wrapped, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_then_filter_box_bool() {
    // wrap(Box<bool>, skip=[Box]) => Box<adze::WithLeaf<bool>>
    // filter(Box<...>, skip=[Box]) => adze::WithLeaf<bool>
    let ty: Type = parse_quote!(Box<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    let filtered = filter_inner_type(&wrapped, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < bool >");
}

// ===========================================================================
// 2. Filter then wrap
// ===========================================================================

#[test]
fn filter_then_wrap_box_string() {
    // filter(Box<String>, skip=[Box]) => String
    // wrap(String, skip=[Box]) => adze::WithLeaf<String>
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Box"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_wrap_nested_box_arc() {
    // filter(Box<Arc<u64>>, skip=[Box, Arc]) => u64
    // wrap(u64, skip=[Vec]) => adze::WithLeaf<u64>
    let ty: Type = parse_quote!(Box<Arc<u64>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "u64");
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u64 >");
}

#[test]
fn filter_then_wrap_option_vec_string() {
    // filter(Option<Vec<String>>, skip=[Option]) => Vec<String>
    // wrap(Vec<String>, skip=[Vec]) => Vec<adze::WithLeaf<String>>
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "Vec < String >");
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

// ===========================================================================
// 3. Extract then wrap
// ===========================================================================

#[test]
fn extract_vec_then_wrap() {
    // extract(Vec<String>, "Vec", skip=[]) => (String, true)
    // wrap(String, skip=[]) => adze::WithLeaf<String>
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn extract_option_then_wrap_with_option_skip() {
    // extract(Option<u32>, "Option", skip=[]) => (u32, true)
    // wrap(u32, skip=[Option]) => adze::WithLeaf<u32>  (u32 not in skip, so wrapped)
    let ty: Type = parse_quote!(Option<u32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    let wrapped = wrap_leaf_type(&inner, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u32 >");
}

#[test]
fn extract_through_box_then_wrap() {
    // extract(Box<Vec<f64>>, "Vec", skip=[Box]) => (f64, true)
    // wrap(f64, skip=[Vec]) => adze::WithLeaf<f64>
    let ty: Type = parse_quote!(Box<Vec<f64>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f64");
    let wrapped = wrap_leaf_type(&inner, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn extract_not_found_then_wrap_original() {
    // extract(String, "Vec", skip=[]) => (String, false)
    // wrap(String, skip=[]) => adze::WithLeaf<String>
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

// ===========================================================================
// 4. Double wrap
// ===========================================================================

#[test]
fn double_wrap_plain_type() {
    // wrap(wrap(i32)) => adze::WithLeaf<adze::WithLeaf<i32>>
    let ty: Type = parse_quote!(i32);
    let once = wrap_leaf_type(&ty, &skip(&[]));
    let twice = wrap_leaf_type(&once, &skip(&[]));
    assert_eq!(
        ty_str(&twice),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn double_wrap_vec_string() {
    // First wrap: Vec<adze::WithLeaf<String>>
    // Second wrap (skip Vec again): Vec<adze::WithLeaf<adze::WithLeaf<String>>>
    // because inner is adze::WithLeaf<String> which is a Path not in skip
    let ty: Type = parse_quote!(Vec<String>);
    let s = skip(&["Vec"]);
    let once = wrap_leaf_type(&ty, &s);
    let twice = wrap_leaf_type(&once, &s);
    assert_eq!(
        ty_str(&twice),
        "Vec < adze :: WithLeaf < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn double_wrap_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let s = skip(&["Option"]);
    let once = wrap_leaf_type(&ty, &s);
    let twice = wrap_leaf_type(&once, &s);
    assert_eq!(
        ty_str(&twice),
        "Option < adze :: WithLeaf < adze :: WithLeaf < bool > > >"
    );
}

// ===========================================================================
// 5. Double filter
// ===========================================================================

#[test]
fn double_filter_nested_box() {
    // filter(Box<Box<String>>, skip=[Box]) strips both layers
    let ty: Type = parse_quote!(Box<Box<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
    // Applying again is idempotent on a plain type
    let filtered2 = filter_inner_type(&filtered, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered2), "String");
}

#[test]
fn double_filter_option_option_i32() {
    // filter(Option<Option<i32>>, skip=[Option]) => i32 (both stripped)
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "i32");
    let filtered2 = filter_inner_type(&filtered, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered2), "i32");
}

#[test]
fn double_filter_different_skip_sets() {
    // First filter removes Box, second removes Arc
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let step1 = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&step1), "Arc < String >");
    let step2 = filter_inner_type(&step1, &skip(&["Arc"]));
    assert_eq!(ty_str(&step2), "String");
}

// ===========================================================================
// 6. Compose with all supported type patterns (Vec, Option, Box)
// ===========================================================================

#[test]
fn compose_vec_option_box_wrap_filter() {
    // wrap(Vec<Option<Box<u8>>>, skip=[Vec, Option, Box])
    //   => Vec<Option<Box<adze::WithLeaf<u8>>>>
    let ty: Type = parse_quote!(Vec<Option<Box<u8>>>);
    let s = skip(&["Vec", "Option", "Box"]);
    let wrapped = wrap_leaf_type(&ty, &s);
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < Box < adze :: WithLeaf < u8 > > > >"
    );
    // filter Box layer away
    // filter with skip=[Box] on the inner Option<Box<adze::WithLeaf<u8>>> won't reach
    // because the outermost is Vec. We filter the full thing with skip=[Vec, Option, Box]
    let filtered = filter_inner_type(&wrapped, &s);
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < u8 >");
}

#[test]
fn compose_option_vec_extract_then_wrap() {
    // extract Option from Option<Vec<String>>
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < String >");
    // Now wrap, skipping Vec
    let wrapped = wrap_leaf_type(&inner, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn compose_box_vec_filter_then_extract() {
    // filter(Box<Vec<i32>>, skip=[Box]) => Vec<i32>
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
    // extract Vec
    let (inner, extracted) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn compose_all_three_ops_pipeline() {
    // Start: Box<Option<Vec<String>>>
    // 1. filter Box => Option<Vec<String>>
    // 2. extract Option => Vec<String>
    // 3. wrap with Vec skip => Vec<adze::WithLeaf<String>>
    let ty: Type = parse_quote!(Box<Option<Vec<String>>>);
    let step1 = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&step1), "Option < Vec < String > >");
    let (step2, extracted) = try_extract_inner_type(&step1, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&step2), "Vec < String >");
    let step3 = wrap_leaf_type(&step2, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&step3),
        "Vec < adze :: WithLeaf < String > >"
    );
}

// ===========================================================================
// 7. Identity compositions
// ===========================================================================

#[test]
fn filter_identity_no_matching_skip() {
    // filter with empty skip set is identity
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), ty_str(&ty));
}

#[test]
fn filter_identity_type_not_in_skip() {
    // filter(Vec<String>, skip=[Box]) leaves Vec untouched
    let ty: Type = parse_quote!(Vec<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < String >");
}

#[test]
fn extract_identity_when_target_absent() {
    // extract "HashMap" from Vec<String> => not found, returns original
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "HashMap", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn wrap_then_filter_then_extract_roundtrip() {
    // wrap(String, skip=[]) => adze::WithLeaf<String>
    // filter(adze::WithLeaf<String>, skip=[]) => adze::WithLeaf<String> (no skip match)
    // extract "WithLeaf" from the result is not in skip => (adze::WithLeaf<String>, false)
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    let filtered = filter_inner_type(&wrapped, &skip(&[]));
    let (inner, extracted) = try_extract_inner_type(&filtered, "HashMap", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "adze :: WithLeaf < String >");
}

// ===========================================================================
// 8. Error-resilient compositions (non-path types)
// ===========================================================================

#[test]
fn wrap_then_filter_reference_type() {
    // wrap(&str) => adze::WithLeaf<&str>  (non-path wrapped wholesale)
    // filter(adze::WithLeaf<&str>, skip=[Box]) => adze::WithLeaf<&str>  (WithLeaf not in skip)
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
    let filtered = filter_inner_type(&wrapped, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < & str >");
}

#[test]
fn filter_then_wrap_tuple_type() {
    // filter((i32, u32), skip=[Box]) => (i32, u32)  (non-path, identity)
    // wrap((i32, u32), skip=[]) => adze::WithLeaf<(i32, u32)>
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(i32 , u32)");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (i32 , u32) >");
}

#[test]
fn extract_then_wrap_non_path_type() {
    // extract from &str => not extracted (non-path)
    // wrap result => adze::WithLeaf<&str>
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn double_wrap_array_type() {
    // [u8; 4] is non-path => wrapped wholesale both times
    let ty: Type = parse_quote!([u8; 4]);
    let once = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&once), "adze :: WithLeaf < [u8 ; 4] >");
    let twice = wrap_leaf_type(&once, &skip(&[]));
    assert_eq!(
        ty_str(&twice),
        "adze :: WithLeaf < adze :: WithLeaf < [u8 ; 4] > >"
    );
}

#[test]
fn double_filter_non_path_is_identity() {
    let ty: Type = parse_quote!(&mut Vec<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    // &mut Vec<String> is Type::Reference, not Type::Path, so filter is identity
    assert_eq!(ty_str(&filtered), "& mut Vec < String >");
    let filtered2 = filter_inner_type(&filtered, &skip(&["Vec"]));
    assert_eq!(ty_str(&filtered2), "& mut Vec < String >");
}

// ===========================================================================
// 9. Mixed multi-step compositions
// ===========================================================================

#[test]
fn extract_option_filter_box_wrap_vec() {
    // Start: Option<Box<Vec<f32>>>
    // extract Option => Box<Vec<f32>>
    // filter Box => Vec<f32>
    // wrap with Vec skip => Vec<adze::WithLeaf<f32>>
    let ty: Type = parse_quote!(Option<Box<Vec<f32>>>);
    let (step1, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&step1), "Box < Vec < f32 > >");
    let step2 = filter_inner_type(&step1, &skip(&["Box"]));
    assert_eq!(ty_str(&step2), "Vec < f32 >");
    let step3 = wrap_leaf_type(&step2, &skip(&["Vec"]));
    assert_eq!(ty_str(&step3), "Vec < adze :: WithLeaf < f32 > >");
}

#[test]
fn wrap_filter_wrap_sandwich() {
    // wrap(String, skip=[]) => adze::WithLeaf<String>
    // filter(adze::WithLeaf<String>, skip=[]) => adze::WithLeaf<String>
    // wrap again => adze::WithLeaf<adze::WithLeaf<String>>
    let ty: Type = parse_quote!(String);
    let w1 = wrap_leaf_type(&ty, &skip(&[]));
    let f = filter_inner_type(&w1, &skip(&[]));
    let w2 = wrap_leaf_type(&f, &skip(&[]));
    assert_eq!(
        ty_str(&w2),
        "adze :: WithLeaf < adze :: WithLeaf < String > >"
    );
}

#[test]
fn filter_extract_on_already_plain_type() {
    // filter(u64, skip=[Box]) => u64 (identity)
    // extract "Vec" from u64 => (u64, false)
    let ty: Type = parse_quote!(u64);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "u64");
    let (inner, extracted) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn wrap_vec_option_nested_then_filter_both() {
    // wrap(Vec<Option<String>>, skip=[Vec, Option])
    //   => Vec<Option<adze::WithLeaf<String>>>
    // filter with skip=[Vec, Option] strips both
    //   => adze::WithLeaf<String>
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let s = skip(&["Vec", "Option"]);
    let wrapped = wrap_leaf_type(&ty, &s);
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
    let filtered = filter_inner_type(&wrapped, &s);
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < String >");
}
