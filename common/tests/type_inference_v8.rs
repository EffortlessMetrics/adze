//! Comprehensive tests for type extraction and manipulation in adze-common.
//!
//! Covers `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! across a wide variety of type patterns, nesting depths, skip sets, and edge cases.

#![allow(unused_variables)]

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use std::collections::HashSet;
use syn::{Type, parse_quote};

fn type_str(ty: &Type) -> String {
    quote::quote!(#ty).to_string()
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

fn skip_with(names: &[&'static str]) -> HashSet<&'static str> {
    names.iter().copied().collect()
}

// ============================================================================
// try_extract_inner_type — basic extraction
// ============================================================================

#[test]
fn extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_box_u32() {
    let ty: Type = parse_quote!(Box<u32>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "u32");
}

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_option_i64() {
    let ty: Type = parse_quote!(Option<i64>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "i64");
}

#[test]
fn extract_vec_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "u8");
}

#[test]
fn extract_box_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "bool");
}

#[test]
fn extract_option_f64() {
    let ty: Type = parse_quote!(Option<f64>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "f64");
}

// ============================================================================
// try_extract_inner_type — non-matching types return (original, false)
// ============================================================================

#[test]
fn extract_i32_looking_for_vec_returns_false() {
    let ty: Type = parse_quote!(i32);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn extract_string_looking_for_option_returns_false() {
    let ty: Type = parse_quote!(String);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_bool_looking_for_box_returns_false() {
    let ty: Type = parse_quote!(bool);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "bool");
}

#[test]
fn extract_u64_looking_for_vec_returns_false() {
    let ty: Type = parse_quote!(u64);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "u64");
}

#[test]
fn extract_f32_looking_for_option_returns_false() {
    let ty: Type = parse_quote!(f32);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "f32");
}

#[test]
fn extract_usize_looking_for_box_returns_false() {
    let ty: Type = parse_quote!(usize);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "usize");
}

// ============================================================================
// try_extract_inner_type — wrong wrapper, returns (original, false)
// ============================================================================

#[test]
fn extract_option_string_looking_for_vec_returns_false() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "Option < String >");
}

#[test]
fn extract_box_i32_looking_for_option_returns_false() {
    let ty: Type = parse_quote!(Box<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "Box < i32 >");
}

#[test]
fn extract_vec_u32_looking_for_box_returns_false() {
    let ty: Type = parse_quote!(Vec<u32>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "Vec < u32 >");
}

// ============================================================================
// try_extract_inner_type — nested generics
// ============================================================================

#[test]
fn extract_vec_of_vec_string() {
    let ty: Type = parse_quote!(Vec<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "Vec < String >");
}

#[test]
fn extract_option_of_option_i32() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "Option < i32 >");
}

#[test]
fn extract_vec_of_option_bool() {
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "Option < bool >");
}

#[test]
fn extract_option_of_vec_u8() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "Vec < u8 >");
}

#[test]
fn extract_box_of_vec_i32() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "Vec < i32 >");
}

// ============================================================================
// try_extract_inner_type — with skip_over
// ============================================================================

#[test]
fn extract_through_box_to_vec() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Box"]));
    assert!(found);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn extract_through_option_to_box() {
    let ty: Type = parse_quote!(Option<Box<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip_with(&["Option"]));
    assert!(found);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_through_arc_to_vec() {
    let ty: Type = parse_quote!(Arc<Vec<f64>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Arc"]));
    assert!(found);
    assert_eq!(type_str(&inner), "f64");
}

#[test]
fn extract_through_box_to_option() {
    let ty: Type = parse_quote!(Box<Option<u16>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip_with(&["Box"]));
    assert!(found);
    assert_eq!(type_str(&inner), "u16");
}

#[test]
fn extract_skip_chain_box_arc_to_vec() {
    let ty: Type = parse_quote!(Box<Arc<Vec<String>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Box", "Arc"]));
    assert!(found);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_skip_not_found_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Box"]));
    assert!(!found);
    assert_eq!(type_str(&inner), "Box < String >");
}

#[test]
fn extract_skip_over_but_target_not_inside() {
    let ty: Type = parse_quote!(Arc<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip_with(&["Arc"]));
    assert!(!found);
    assert_eq!(type_str(&inner), "Arc < i32 >");
}

#[test]
fn extract_through_rc_to_option() {
    let ty: Type = parse_quote!(Rc<Option<bool>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip_with(&["Rc"]));
    assert!(found);
    assert_eq!(type_str(&inner), "bool");
}

// ============================================================================
// try_extract_inner_type — non-path types
// ============================================================================

#[test]
fn extract_reference_type_returns_false() {
    let ty: Type = parse_quote!(&str);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "& str");
}

#[test]
fn extract_mutable_reference_returns_false() {
    let ty: Type = parse_quote!(&mut i32);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "& mut i32");
}

#[test]
fn extract_tuple_type_returns_false() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "(i32 , u32)");
}

#[test]
fn extract_array_type_returns_false() {
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "[u8 ; 4]");
}

#[test]
fn extract_slice_reference_returns_false() {
    let ty: Type = parse_quote!(&[u8]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "& [u8]");
}

#[test]
fn extract_unit_type_returns_false() {
    let ty: Type = parse_quote!(());
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "()");
}

// ============================================================================
// try_extract_inner_type — custom and qualified types
// ============================================================================

#[test]
fn extract_custom_wrapper() {
    let ty: Type = parse_quote!(MyWrapper<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "MyWrapper", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn extract_custom_type_not_matching() {
    let ty: Type = parse_quote!(MyWrapper<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "MyWrapper < i32 >");
}

#[test]
fn extract_skip_custom_to_vec() {
    let ty: Type = parse_quote!(Wrapper<Vec<u64>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Wrapper"]));
    assert!(found);
    assert_eq!(type_str(&inner), "u64");
}

#[test]
fn extract_qualified_path_type() {
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    // Only the last segment is checked, which is "Vec"
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "i32");
}

// ============================================================================
// filter_inner_type — empty skip set
// ============================================================================

#[test]
fn filter_empty_skip_returns_same_primitive() {
    let ty: Type = parse_quote!(i32);
    let filtered = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_str(&filtered), "i32");
}

#[test]
fn filter_empty_skip_returns_same_string() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_str(&filtered), "String");
}

#[test]
fn filter_empty_skip_returns_same_generic() {
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_str(&filtered), "Vec < i32 >");
}

#[test]
fn filter_empty_skip_returns_same_box() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_str(&filtered), "Box < String >");
}

// ============================================================================
// filter_inner_type — single wrapper removal
// ============================================================================

#[test]
fn filter_box_i32() {
    let ty: Type = parse_quote!(Box<i32>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "i32");
}

#[test]
fn filter_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&filtered), "String");
}

#[test]
fn filter_arc_f64() {
    let ty: Type = parse_quote!(Arc<f64>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Arc"]));
    assert_eq!(type_str(&filtered), "f64");
}

#[test]
fn filter_rc_bool() {
    let ty: Type = parse_quote!(Rc<bool>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Rc"]));
    assert_eq!(type_str(&filtered), "bool");
}

// ============================================================================
// filter_inner_type — nested wrapper removal
// ============================================================================

#[test]
fn filter_box_arc_string() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box", "Arc"]));
    assert_eq!(type_str(&filtered), "String");
}

#[test]
fn filter_option_box_i32() {
    let ty: Type = parse_quote!(Option<Box<i32>>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Option", "Box"]));
    assert_eq!(type_str(&filtered), "i32");
}

#[test]
fn filter_arc_box_option_u8() {
    let ty: Type = parse_quote!(Arc<Box<Option<u8>>>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Arc", "Box", "Option"]));
    assert_eq!(type_str(&filtered), "u8");
}

#[test]
fn filter_box_box_string() {
    let ty: Type = parse_quote!(Box<Box<String>>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "String");
}

// ============================================================================
// filter_inner_type — partial skip (stops at non-skip wrapper)
// ============================================================================

#[test]
fn filter_box_vec_i32_only_skip_box() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "Vec < i32 >");
}

#[test]
fn filter_option_vec_string_only_skip_option() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&filtered), "Vec < String >");
}

#[test]
fn filter_arc_option_u16_only_skip_arc() {
    let ty: Type = parse_quote!(Arc<Option<u16>>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Arc"]));
    assert_eq!(type_str(&filtered), "Option < u16 >");
}

// ============================================================================
// filter_inner_type — non-matching types
// ============================================================================

#[test]
fn filter_non_matching_vec_with_box_skip() {
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "Vec < i32 >");
}

#[test]
fn filter_primitive_with_box_skip() {
    let ty: Type = parse_quote!(i32);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "i32");
}

#[test]
fn filter_reference_type_unchanged() {
    let ty: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "& str");
}

#[test]
fn filter_tuple_type_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "(i32 , u32)");
}

#[test]
fn filter_array_type_unchanged() {
    let ty: Type = parse_quote!([u8; 4]);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "[u8 ; 4]");
}

// ============================================================================
// wrap_leaf_type — primitive and simple types
// ============================================================================

#[test]
fn wrap_i32() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_string() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_bool() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_u8() {
    let ty: Type = parse_quote!(u8);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < u8 >");
}

#[test]
fn wrap_f64() {
    let ty: Type = parse_quote!(f64);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn wrap_usize() {
    let ty: Type = parse_quote!(usize);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < usize >");
}

// ============================================================================
// wrap_leaf_type — with skip_over (preserves container, wraps inner)
// ============================================================================

#[test]
fn wrap_vec_string_skip_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(type_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_i32_skip_option() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_box_bool_skip_box() {
    let ty: Type = parse_quote!(Box<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&wrapped), "Box < adze :: WithLeaf < bool > >");
}

#[test]
fn wrap_vec_u8_skip_vec() {
    let ty: Type = parse_quote!(Vec<u8>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(type_str(&wrapped), "Vec < adze :: WithLeaf < u8 > >");
}

// ============================================================================
// wrap_leaf_type — nested skip containers
// ============================================================================

#[test]
fn wrap_vec_option_string_skip_both() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Vec", "Option"]));
    assert_eq!(
        type_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_option_vec_i32_skip_both() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Option", "Vec"]));
    assert_eq!(
        type_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_box_vec_string_skip_both() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Box", "Vec"]));
    assert_eq!(
        type_str(&wrapped),
        "Box < Vec < adze :: WithLeaf < String > > >"
    );
}

// ============================================================================
// wrap_leaf_type — container not in skip set gets wrapped entirely
// ============================================================================

#[test]
fn wrap_vec_string_empty_skip_wraps_whole() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < Vec < String > >");
}

#[test]
fn wrap_option_i32_empty_skip_wraps_whole() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < Option < i32 > >");
}

#[test]
fn wrap_box_u32_empty_skip_wraps_whole() {
    let ty: Type = parse_quote!(Box<u32>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < Box < u32 > >");
}

// ============================================================================
// wrap_leaf_type — non-path types
// ============================================================================

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, u32));
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < (i32 , u32) >");
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_mutable_reference() {
    let ty: Type = parse_quote!(&mut i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < & mut i32 >");
}

// ============================================================================
// wrap_leaf_type — multiple generic args
// ============================================================================

#[test]
fn wrap_result_both_args_skip_result() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Result"]));
    assert_eq!(
        type_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_hashmap_skip_hashmap() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["HashMap"]));
    assert_eq!(
        type_str(&wrapped),
        "HashMap < adze :: WithLeaf < String > , adze :: WithLeaf < Vec < i32 > > >"
    );
}

#[test]
fn wrap_hashmap_skip_hashmap_and_vec() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["HashMap", "Vec"]));
    assert_eq!(
        type_str(&wrapped),
        "HashMap < adze :: WithLeaf < String > , Vec < adze :: WithLeaf < i32 > > >"
    );
}

// ============================================================================
// Interaction: extract then filter
// ============================================================================

#[test]
fn extract_then_filter_box_vec_i32() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (extracted, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Box"]));
    assert!(found);
    assert_eq!(type_str(&extracted), "i32");
    let filtered = filter_inner_type(&extracted, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "i32");
}

#[test]
fn filter_then_wrap() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "String");
    let wrapped = wrap_leaf_type(&filtered, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < String >");
}

// ============================================================================
// Interaction: extract then wrap
// ============================================================================

#[test]
fn extract_then_wrap_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (extracted, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    let wrapped = wrap_leaf_type(&extracted, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn extract_then_wrap_option_vec_i32() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (extracted, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    let wrapped = wrap_leaf_type(&extracted, &skip_with(&["Vec"]));
    assert_eq!(type_str(&wrapped), "Vec < adze :: WithLeaf < i32 > >");
}

// ============================================================================
// filter_inner_type — idempotency
// ============================================================================

#[test]
fn filter_idempotent_on_primitives() {
    let ty: Type = parse_quote!(i32);
    let skip = skip_with(&["Box"]);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(type_str(&once), type_str(&twice));
}

#[test]
fn filter_idempotent_after_unwrap() {
    let ty: Type = parse_quote!(Box<String>);
    let skip = skip_with(&["Box"]);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(type_str(&once), "String");
    assert_eq!(type_str(&once), type_str(&twice));
}

#[test]
fn filter_idempotent_nested() {
    let ty: Type = parse_quote!(Box<Arc<u64>>);
    let skip = skip_with(&["Box", "Arc"]);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(type_str(&once), "u64");
    assert_eq!(type_str(&once), type_str(&twice));
}

// ============================================================================
// try_extract_inner_type — deeply nested with skip
// ============================================================================

#[test]
fn extract_three_layer_skip_to_vec() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Vec<String>>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Box", "Arc", "Rc"]));
    assert!(found);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_two_layer_skip_target_in_middle() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip_with(&["Box"]));
    assert!(found);
    assert_eq!(type_str(&inner), "i32");
}

// ============================================================================
// wrap_leaf_type — deeply nested skip chain
// ============================================================================

#[test]
fn wrap_three_layer_skip() {
    let ty: Type = parse_quote!(Vec<Option<Box<String>>>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Vec", "Option", "Box"]));
    assert_eq!(
        type_str(&wrapped),
        "Vec < Option < Box < adze :: WithLeaf < String > > > >"
    );
}

#[test]
fn wrap_vec_of_vec_skip_vec() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(
        type_str(&wrapped),
        "Vec < Vec < adze :: WithLeaf < i32 > > >"
    );
}

// ============================================================================
// Edge cases: custom types and qualified paths
// ============================================================================

#[test]
fn filter_custom_wrapper_in_skip() {
    let ty: Type = parse_quote!(Wrapper<i32>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Wrapper"]));
    assert_eq!(type_str(&filtered), "i32");
}

#[test]
fn wrap_custom_type_no_skip() {
    let ty: Type = parse_quote!(MyType);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < MyType >");
}

#[test]
fn wrap_custom_generic_in_skip() {
    let ty: Type = parse_quote!(Container<Payload>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Container"]));
    assert_eq!(
        type_str(&wrapped),
        "Container < adze :: WithLeaf < Payload > >"
    );
}

#[test]
fn filter_qualified_path_box() {
    let ty: Type = parse_quote!(std::boxed::Box<i32>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "i32");
}

#[test]
fn extract_qualified_path_option() {
    let ty: Type = parse_quote!(core::option::Option<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "String");
}

// ============================================================================
// Consistency: extract found=false preserves original type exactly
// ============================================================================

#[test]
fn extract_false_preserves_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (returned, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&ty), type_str(&returned));
}

#[test]
fn extract_false_preserves_nested_generic() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let (returned, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&ty), type_str(&returned));
}

#[test]
fn extract_false_preserves_primitive() {
    let ty: Type = parse_quote!(u128);
    let (returned, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&ty), type_str(&returned));
}

// ============================================================================
// Additional wrap_leaf_type edge cases
// ============================================================================

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < () >");
}

#[test]
fn wrap_slice_reference() {
    let ty: Type = parse_quote!(&[u8]);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&wrapped), "adze :: WithLeaf < & [u8] >");
}

#[test]
fn wrap_option_of_option_skip_option() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip_with(&["Option"]));
    assert_eq!(
        type_str(&wrapped),
        "Option < Option < adze :: WithLeaf < i32 > > >"
    );
}

// ============================================================================
// Additional filter edge cases
// ============================================================================

#[test]
fn filter_unit_type_with_skip() {
    let ty: Type = parse_quote!(());
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&filtered), "()");
}

#[test]
fn filter_string_type_with_multiple_skips() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box", "Arc", "Rc"]));
    assert_eq!(type_str(&filtered), "String");
}

// ============================================================================
// Additional try_extract edge cases
// ============================================================================

#[test]
fn extract_vec_of_tuple() {
    let ty: Type = parse_quote!(Vec<(i32, String)>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "(i32 , String)");
}

#[test]
fn extract_option_of_reference() {
    let ty: Type = parse_quote!(Option<&str>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "& str");
}

#[test]
fn extract_vec_of_array() {
    let ty: Type = parse_quote!(Vec<[u8; 16]>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "[u8 ; 16]");
}

#[test]
fn extract_box_of_unit() {
    let ty: Type = parse_quote!(Box<()>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "()");
}
