//! Comprehensive tests for complex type extraction scenarios in adze-common.
//!
//! Covers the three core type-transformation functions:
//! - `try_extract_inner_type(ty, target, skip_set)` — peel one target container
//! - `filter_inner_type(ty, skip_set)` — recursively unwrap containers in skip set
//! - `wrap_leaf_type(ty, skip_set)` — wrap leaf types with `adze::WithLeaf`
//!
//! 80+ tests organised into sections:
//!   1–10  Basic extraction
//!  11–20  Extraction with skip sets
//!  21–30  Nested / multi-layer extraction
//!  31–40  filter_inner_type
//!  41–50  wrap_leaf_type
//!  51–60  Roundtrip / composition
//!  61–70  Complex generic types
//!  71–80+ Edge cases and exotic types

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
// 1–10  Basic extraction (try_extract_inner_type)
// ===========================================================================

#[test]
fn test_01_extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_02_extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_03_extract_option_u8() {
    let ty: Type = parse_quote!(Option<u8>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn test_04_extract_box_f64() {
    let ty: Type = parse_quote!(Box<f64>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn test_05_no_match_vec_from_i32() {
    let ty: Type = parse_quote!(i32);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_06_no_match_option_from_string() {
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_07_extract_vec_of_vec_one_layer() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn test_08_extract_option_of_option_one_layer() {
    let ty: Type = parse_quote!(Option<Option<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < bool >");
}

#[test]
fn test_09_extract_option_from_vec_no_match() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn test_10_extract_box_from_option_no_match() {
    let ty: Type = parse_quote!(Option<u64>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Option < u64 >");
}

// ===========================================================================
// 11–20  Extraction with skip sets
// ===========================================================================

#[test]
fn test_11_skip_box_extract_vec() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_12_skip_option_extract_vec() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_13_skip_arc_extract_option() {
    let ty: Type = parse_quote!(Arc<Option<f32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn test_14_skip_box_and_arc_extract_vec() {
    let ty: Type = parse_quote!(Box<Arc<Vec<u16>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn test_15_skip_set_not_present_no_match() {
    let ty: Type = parse_quote!(Rc<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Rc < Vec < i32 > >");
}

#[test]
fn test_16_skip_over_inner_of_same_type() {
    // skip={"Vec"} inner_of="Vec": outer Vec is matched first, not skipped
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Vec"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_17_skip_does_not_match_target_returns_false() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    // Box is skipped, String doesn't match Vec
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn test_18_skip_three_layers() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Vec<u8>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc", "Rc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn test_19_skip_option_no_target_inside() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn test_20_skip_multiple_no_target() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < Arc < String > >");
}

// ===========================================================================
// 21–30  Nested / multi-layer extraction (chained calls)
// ===========================================================================

#[test]
fn test_21_two_stage_extract_vec_vec_i32() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let (mid, ok1) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Vec", &skip(&[]));
    assert!(ok2);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_22_three_stage_extract_vec_vec_vec() {
    let ty: Type = parse_quote!(Vec<Vec<Vec<bool>>>);
    let (l1, ok1) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok1);
    let (l2, ok2) = try_extract_inner_type(&l1, "Vec", &skip(&[]));
    assert!(ok2);
    let (l3, ok3) = try_extract_inner_type(&l2, "Vec", &skip(&[]));
    assert!(ok3);
    assert_eq!(ty_str(&l3), "bool");
}

#[test]
fn test_23_extract_option_then_vec() {
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    let (mid, ok1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Vec", &skip(&[]));
    assert!(ok2);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn test_24_extract_vec_then_option() {
    let ty: Type = parse_quote!(Vec<Option<i64>>);
    let (mid, ok1) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Option", &skip(&[]));
    assert!(ok2);
    assert_eq!(ty_str(&inner), "i64");
}

#[test]
fn test_25_extract_box_then_option_then_vec() {
    let ty: Type = parse_quote!(Box<Option<Vec<f32>>>);
    let (l1, ok1) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok1);
    let (l2, ok2) = try_extract_inner_type(&l1, "Option", &skip(&[]));
    assert!(ok2);
    let (l3, ok3) = try_extract_inner_type(&l2, "Vec", &skip(&[]));
    assert!(ok3);
    assert_eq!(ty_str(&l3), "f32");
}

#[test]
fn test_26_chained_stops_when_no_match() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (mid, ok1) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Vec", &skip(&[]));
    assert!(!ok2);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_27_extract_with_skip_and_chained() {
    let ty: Type = parse_quote!(Box<Vec<Option<u8>>>);
    let (mid, ok1) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok1);
    let (inner, ok2) = try_extract_inner_type(&mid, "Option", &skip(&[]));
    assert!(ok2);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn test_28_nested_option_option_option() {
    let ty: Type = parse_quote!(Option<Option<Option<char>>>);
    let (l1, ok1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok1);
    let (l2, ok2) = try_extract_inner_type(&l1, "Option", &skip(&[]));
    assert!(ok2);
    let (l3, ok3) = try_extract_inner_type(&l2, "Option", &skip(&[]));
    assert!(ok3);
    assert_eq!(ty_str(&l3), "char");
}

#[test]
fn test_29_deep_skip_chain_extract() {
    let ty: Type = parse_quote!(Arc<Box<Rc<Option<i128>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box", "Rc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i128");
}

#[test]
fn test_30_extract_hashmap_first_type_arg() {
    // HashMap<K,V> — extraction picks the first generic argument
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "HashMap", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

// ===========================================================================
// 31–40  filter_inner_type
// ===========================================================================

#[test]
fn test_31_filter_empty_skip_returns_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

#[test]
fn test_32_filter_skip_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn test_33_filter_skip_option() {
    let ty: Type = parse_quote!(Option<i32>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn test_34_filter_skip_box() {
    let ty: Type = parse_quote!(Box<f64>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "f64");
}

#[test]
fn test_35_filter_nested_skip_both() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn test_36_filter_nested_skip_outer_only() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "Option < i32 >");
}

#[test]
fn test_37_filter_nested_skip_inner_only() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    // Vec not in skip set → returned as-is
    assert_eq!(ty_str(&result), "Vec < Option < i32 > >");
}

#[test]
fn test_38_filter_plain_type_unchanged() {
    let ty: Type = parse_quote!(String);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn test_39_filter_three_layers() {
    let ty: Type = parse_quote!(Box<Arc<Rc<u32>>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ty_str(&result), "u32");
}

#[test]
fn test_40_filter_stops_at_non_skip() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

// ===========================================================================
// 41–50  wrap_leaf_type
// ===========================================================================

#[test]
fn test_41_wrap_plain_i32() {
    let ty: Type = parse_quote!(i32);
    let result = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "adze :: WithLeaf < i32 >");
}

#[test]
fn test_42_wrap_plain_string() {
    let ty: Type = parse_quote!(String);
    let result = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "adze :: WithLeaf < String >");
}

#[test]
fn test_43_wrap_vec_not_in_skip() {
    // Vec not in skip → wrapped wholesale
    let ty: Type = parse_quote!(Vec<i32>);
    let result = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn test_44_wrap_vec_in_skip() {
    // Vec in skip → Vec preserved, inner wrapped
    let ty: Type = parse_quote!(Vec<i32>);
    let result = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "Vec < adze :: WithLeaf < i32 > >");
}

#[test]
fn test_45_wrap_option_in_skip() {
    let ty: Type = parse_quote!(Option<String>);
    let result = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "Option < adze :: WithLeaf < String > >");
}

#[test]
fn test_46_wrap_box_in_skip() {
    let ty: Type = parse_quote!(Box<f64>);
    let result = wrap_leaf_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Box < adze :: WithLeaf < f64 > >");
}

#[test]
fn test_47_wrap_nested_vec_option_both_skip() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&result),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn test_48_wrap_nested_skip_outer_only() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = wrap_leaf_type(&ty, &skip(&["Vec"]));
    // Vec skipped, but Option not in skip → Option<i32> wrapped wholesale
    assert_eq!(
        ty_str(&result),
        "Vec < adze :: WithLeaf < Option < i32 > > >"
    );
}

#[test]
fn test_49_wrap_nested_skip_inner_only() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = wrap_leaf_type(&ty, &skip(&["Option"]));
    // Vec not in skip → entire Vec<Option<i32>> wrapped
    assert_eq!(
        ty_str(&result),
        "adze :: WithLeaf < Vec < Option < i32 > > >"
    );
}

#[test]
fn test_50_wrap_three_layer_all_skip() {
    let ty: Type = parse_quote!(Box<Vec<Option<u8>>>);
    let result = wrap_leaf_type(&ty, &skip(&["Box", "Vec", "Option"]));
    assert_eq!(
        ty_str(&result),
        "Box < Vec < Option < adze :: WithLeaf < u8 > > > >"
    );
}

// ===========================================================================
// 51–60  Roundtrip / composition
// ===========================================================================

#[test]
fn test_51_extract_then_wrap_roundtrip() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn test_52_filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Option"]));
    assert_eq!(ty_str(&filtered), "i32");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn test_53_wrap_then_extract() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    // i32 not in skip → wrapped
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
    // Last segment is "WithLeaf", so extracting "WithLeaf" succeeds
    let (inner, extracted) = try_extract_inner_type(&wrapped, "WithLeaf", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_54_extract_and_filter_same_result() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (extracted_inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let filtered_inner = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&extracted_inner), ty_str(&filtered_inner));
}

#[test]
fn test_55_double_wrap_via_skip() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    // Both Vec and Option skipped, i32 wrapped
    assert_eq!(
        ty_str(&result),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn test_56_extract_skip_filter_chain() {
    let ty: Type = parse_quote!(Box<Vec<Option<u16>>>);
    let (after_extract, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    // after_extract = Option<u16>
    let filtered = filter_inner_type(&after_extract, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "u16");
}

#[test]
fn test_57_filter_idempotent() {
    let ty: Type = parse_quote!(i32);
    let once = filter_inner_type(&ty, &skip(&["Vec", "Option"]));
    let twice = filter_inner_type(&once, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&once), ty_str(&twice));
}

#[test]
fn test_58_wrap_idempotent_with_empty_skip() {
    // wrap with no skip always wraps → not idempotent, but deterministic
    let ty: Type = parse_quote!(i32);
    let once = wrap_leaf_type(&ty, &skip(&[]));
    let twice = wrap_leaf_type(&once, &skip(&[]));
    assert_eq!(
        ty_str(&twice),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn test_59_extract_preserves_non_matching_structure() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Result < String , Error >");
}

#[test]
fn test_60_filter_preserves_non_matching() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "Result < String , Error >");
}

// ===========================================================================
// 61–70  Complex generic types
// ===========================================================================

#[test]
fn test_61_extract_from_hashmap() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "HashMap", &skip(&[]));
    assert!(extracted);
    // First generic arg
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_62_extract_from_btreemap() {
    let ty: Type = parse_quote!(BTreeMap<u32, String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "BTreeMap", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn test_63_extract_result_ok_type() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_64_extract_custom_generic() {
    let ty: Type = parse_quote!(MyWrapper<CustomType>);
    let (inner, extracted) = try_extract_inner_type(&ty, "MyWrapper", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "CustomType");
}

#[test]
fn test_65_extract_cow() {
    let ty: Type = parse_quote!(Cow<str>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Cow", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "str");
}

#[test]
fn test_66_wrap_with_multiarg_in_skip() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let result = wrap_leaf_type(&ty, &skip(&["HashMap"]));
    // HashMap in skip → both generic args wrapped
    assert_eq!(
        ty_str(&result),
        "HashMap < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn test_67_wrap_result_in_skip() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let result = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&result),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < Error > >"
    );
}

#[test]
fn test_68_extract_vec_of_tuple_like_type() {
    let ty: Type = parse_quote!(Vec<Pair<A, B>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Pair < A , B >");
}

#[test]
fn test_69_nested_hashmap_vec() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "HashMap", &skip(&[]));
    assert!(extracted);
    // First arg only
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_70_deeply_nested_generic() {
    let ty: Type = parse_quote!(Vec<Option<Box<Arc<i32>>>>);
    let (l1, ok1) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok1);
    let (l2, ok2) = try_extract_inner_type(&l1, "Option", &skip(&[]));
    assert!(ok2);
    let (l3, ok3) = try_extract_inner_type(&l2, "Box", &skip(&[]));
    assert!(ok3);
    let (l4, ok4) = try_extract_inner_type(&l3, "Arc", &skip(&[]));
    assert!(ok4);
    assert_eq!(ty_str(&l4), "i32");
}

// ===========================================================================
// 71–80+ Edge cases and exotic types
// ===========================================================================

#[test]
fn test_71_extract_from_unit_type_no_match() {
    let ty: Type = parse_quote!(());
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "()");
}

#[test]
fn test_72_filter_primitive_unchanged() {
    let ty: Type = parse_quote!(bool);
    let result = filter_inner_type(&ty, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn test_73_wrap_bool() {
    let ty: Type = parse_quote!(bool);
    let result = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "adze :: WithLeaf < bool >");
}

#[test]
fn test_74_wrap_unit() {
    let ty: Type = parse_quote!(());
    let result = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "adze :: WithLeaf < () >");
}

#[test]
fn test_75_extract_with_qualified_path_no_match() {
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    // Extraction checks last segment only
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_76_filter_qualified_path() {
    let ty: Type = parse_quote!(std::option::Option<i32>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn test_77_extract_cell_refcell() {
    let ty: Type = parse_quote!(RefCell<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["RefCell"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_78_filter_four_layers() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Cell<u64>>>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc", "Cell"]));
    assert_eq!(ty_str(&result), "u64");
}

#[test]
fn test_79_wrap_nested_vec_vec_both_skip() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let result = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "Vec < Vec < adze :: WithLeaf < i32 > > >");
}

#[test]
fn test_80_extract_no_match_different_case() {
    let ty: Type = parse_quote!(vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "vec < i32 >");
}

#[test]
fn test_81_extract_vec_of_reference_type() {
    let ty: Type = parse_quote!(Vec<MyStruct>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "MyStruct");
}

#[test]
fn test_82_filter_mixed_skip_and_non_skip() {
    let ty: Type = parse_quote!(Box<HashMap<String, i32>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "HashMap < String , i32 >");
}

#[test]
fn test_83_wrap_with_all_common_containers_skip() {
    let ty: Type = parse_quote!(Box<Vec<Option<i32>>>);
    let result = wrap_leaf_type(&ty, &skip(&["Box", "Vec", "Option"]));
    assert_eq!(
        ty_str(&result),
        "Box < Vec < Option < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn test_84_extract_returns_clone_on_no_match() {
    let ty: Type = parse_quote!(MyType);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), ty_str(&ty));
}

#[test]
fn test_85_filter_single_skip_on_non_container() {
    let ty: Type = parse_quote!(usize);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "usize");
}

#[test]
fn test_86_wrap_custom_type() {
    let ty: Type = parse_quote!(Foo);
    let result = wrap_leaf_type(&ty, &skip(&["Bar"]));
    // Foo not in skip → wrapped
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Foo >");
}

#[test]
fn test_87_extract_skip_rc_to_option() {
    let ty: Type = parse_quote!(Rc<Option<Vec<u32>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Rc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < u32 >");
}

#[test]
fn test_88_filter_only_outermost() {
    let ty: Type = parse_quote!(Option<Vec<Option<i32>>>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    // Peels outer Option, then Vec not in skip → stops
    assert_eq!(ty_str(&result), "Vec < Option < i32 > >");
}

#[test]
fn test_89_wrap_preserves_multiarg_order() {
    let ty: Type = parse_quote!(Either<Left, Right>);
    let result = wrap_leaf_type(&ty, &skip(&["Either"]));
    assert_eq!(
        ty_str(&result),
        "Either < adze :: WithLeaf < Left > , adze :: WithLeaf < Right > >"
    );
}

#[test]
fn test_90_extract_from_pin() {
    let ty: Type = parse_quote!(Pin<Box<Future>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&["Pin"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Future");
}

#[test]
fn test_91_filter_chain_all_removed() {
    let ty: Type = parse_quote!(Arc<Mutex<RwLock<u8>>>);
    let result = filter_inner_type(&ty, &skip(&["Arc", "Mutex", "RwLock"]));
    assert_eq!(ty_str(&result), "u8");
}

#[test]
fn test_92_wrap_deeply_nested_skip() {
    let ty: Type = parse_quote!(Arc<Mutex<Vec<i32>>>);
    let result = wrap_leaf_type(&ty, &skip(&["Arc", "Mutex", "Vec"]));
    assert_eq!(
        ty_str(&result),
        "Arc < Mutex < Vec < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn test_93_extract_vec_u128() {
    let ty: Type = parse_quote!(Vec<u128>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u128");
}

#[test]
fn test_94_extract_option_isize() {
    let ty: Type = parse_quote!(Option<isize>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "isize");
}

#[test]
fn test_95_filter_empty_skip_on_plain_type() {
    let ty: Type = parse_quote!(f32);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "f32");
}
