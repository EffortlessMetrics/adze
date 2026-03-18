//! Comprehensive tests for type manipulation functions in adze-common.
//!
//! Covers `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! across primitive types, containers, nesting, skip sets, and roundtrips.

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
fn tm_v6_extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn tm_v6_extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn tm_v6_extract_i32_with_vec_returns_false() {
    let ty: Type = parse_quote!(i32);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn tm_v6_extract_box_u8() {
    let ty: Type = parse_quote!(Box<u8>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "u8");
}

#[test]
fn tm_v6_extract_arc_f64() {
    let ty: Type = parse_quote!(Arc<f64>);
    let (inner, found) = try_extract_inner_type(&ty, "Arc", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "f64");
}

// ============================================================================
// filter_inner_type — basic filtering
// ============================================================================

#[test]
fn tm_v6_filter_vec_i32_empty_skip_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), type_str(&ty));
}

#[test]
fn tm_v6_filter_vec_i32_skip_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn tm_v6_filter_option_i32_skip_option() {
    let ty: Type = parse_quote!(Option<i32>);
    let result = filter_inner_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn tm_v6_filter_vec_option_i32_skip_both() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = filter_inner_type(&ty, &skip_with(&["Vec", "Option"]));
    assert_eq!(type_str(&result), "i32");
}

// ============================================================================
// wrap_leaf_type — basic wrapping
// ============================================================================

#[test]
fn tm_v6_wrap_i32_empty_skip() {
    let ty: Type = parse_quote!(i32);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < i32 >");
}

#[test]
fn tm_v6_wrap_vec_i32_skip_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(type_str(&result), "Vec < adze :: WithLeaf < i32 > >");
}

// ============================================================================
// try_extract_inner_type — various primitive types
// ============================================================================

#[test]
fn tm_v6_extract_vec_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "u8");
}

#[test]
fn tm_v6_extract_vec_u16() {
    let ty: Type = parse_quote!(Vec<u16>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "u16");
}

#[test]
fn tm_v6_extract_vec_u32() {
    let ty: Type = parse_quote!(Vec<u32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "u32");
}

#[test]
fn tm_v6_extract_vec_u64() {
    let ty: Type = parse_quote!(Vec<u64>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "u64");
}

#[test]
fn tm_v6_extract_vec_i8() {
    let ty: Type = parse_quote!(Vec<i8>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "i8");
}

#[test]
fn tm_v6_extract_vec_i16() {
    let ty: Type = parse_quote!(Vec<i16>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "i16");
}

#[test]
fn tm_v6_extract_vec_i64() {
    let ty: Type = parse_quote!(Vec<i64>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "i64");
}

#[test]
fn tm_v6_extract_vec_f32() {
    let ty: Type = parse_quote!(Vec<f32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "f32");
}

#[test]
fn tm_v6_extract_vec_f64() {
    let ty: Type = parse_quote!(Vec<f64>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "f64");
}

#[test]
fn tm_v6_extract_vec_bool() {
    let ty: Type = parse_quote!(Vec<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "bool");
}

#[test]
fn tm_v6_extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn tm_v6_extract_vec_char() {
    let ty: Type = parse_quote!(Vec<char>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "char");
}

// ============================================================================
// try_extract_inner_type — container mismatch returns original
// ============================================================================

#[test]
fn tm_v6_extract_option_with_vec_returns_false() {
    let ty: Type = parse_quote!(Option<i32>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&result), "Option < i32 >");
}

#[test]
fn tm_v6_extract_string_with_vec_returns_false() {
    let ty: Type = parse_quote!(String);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&result), "String");
}

#[test]
fn tm_v6_extract_bool_with_option_returns_false() {
    let ty: Type = parse_quote!(bool);
    let (result, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&result), "bool");
}

#[test]
fn tm_v6_extract_box_with_arc_returns_false() {
    let ty: Type = parse_quote!(Box<i32>);
    let (result, found) = try_extract_inner_type(&ty, "Arc", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&result), "Box < i32 >");
}

#[test]
fn tm_v6_extract_u32_with_box_returns_false() {
    let ty: Type = parse_quote!(u32);
    let (result, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&result), "u32");
}

// ============================================================================
// try_extract_inner_type — nested types
// ============================================================================

#[test]
fn tm_v6_extract_vec_vec_i32_outer() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "Vec < i32 >");
}

#[test]
fn tm_v6_extract_option_option_i32_outer() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "Option < i32 >");
}

#[test]
fn tm_v6_extract_option_vec_i32_with_option() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "Vec < i32 >");
}

#[test]
fn tm_v6_extract_vec_option_i32_with_vec() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "Option < i32 >");
}

// ============================================================================
// try_extract_inner_type — with skip_over
// ============================================================================

#[test]
fn tm_v6_extract_box_vec_i32_skip_box_find_vec() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Box"]));
    assert!(found);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn tm_v6_extract_arc_option_string_skip_arc_find_option() {
    let ty: Type = parse_quote!(Arc<Option<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip_with(&["Arc"]));
    assert!(found);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn tm_v6_extract_box_arc_vec_i32_skip_box_arc_find_vec() {
    let ty: Type = parse_quote!(Box<Arc<Vec<i32>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Box", "Arc"]));
    assert!(found);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn tm_v6_extract_skip_but_target_not_found() {
    let ty: Type = parse_quote!(Box<i32>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Box"]));
    assert!(!found);
    assert_eq!(type_str(&result), "Box < i32 >");
}

#[test]
fn tm_v6_extract_skip_through_option_to_vec() {
    let ty: Type = parse_quote!(Option<Vec<f32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Option"]));
    assert!(found);
    assert_eq!(type_str(&inner), "f32");
}

// ============================================================================
// try_extract_inner_type — panics on malformed input
// ============================================================================

#[test]
fn tm_v6_extract_vec_no_angle_brackets_is_unchanged() {
    let ty: Type = parse_quote!(Vec);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&inner), "Vec");
}

// ============================================================================
// filter_inner_type — various scenarios
// ============================================================================

#[test]
fn tm_v6_filter_i32_empty_skip_unchanged() {
    let ty: Type = parse_quote!(i32);
    let result = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn tm_v6_filter_string_empty_skip_unchanged() {
    let ty: Type = parse_quote!(String);
    let result = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "String");
}

#[test]
fn tm_v6_filter_bool_empty_skip_unchanged() {
    let ty: Type = parse_quote!(bool);
    let result = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "bool");
}

#[test]
fn tm_v6_filter_option_string_skip_option() {
    let ty: Type = parse_quote!(Option<String>);
    let result = filter_inner_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&result), "String");
}

#[test]
fn tm_v6_filter_box_u64_skip_box() {
    let ty: Type = parse_quote!(Box<u64>);
    let result = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&result), "u64");
}

#[test]
fn tm_v6_filter_arc_f32_skip_arc() {
    let ty: Type = parse_quote!(Arc<f32>);
    let result = filter_inner_type(&ty, &skip_with(&["Arc"]));
    assert_eq!(type_str(&result), "f32");
}

#[test]
fn tm_v6_filter_box_arc_i32_skip_both() {
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    let result = filter_inner_type(&ty, &skip_with(&["Box", "Arc"]));
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn tm_v6_filter_vec_option_box_char_skip_all() {
    let ty: Type = parse_quote!(Vec<Option<Box<char>>>);
    let result = filter_inner_type(&ty, &skip_with(&["Vec", "Option", "Box"]));
    assert_eq!(type_str(&result), "char");
}

#[test]
fn tm_v6_filter_vec_i32_skip_option_no_change() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&result), "Vec < i32 >");
}

#[test]
fn tm_v6_filter_option_vec_i32_skip_option_only() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let result = filter_inner_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&result), "Vec < i32 >");
}

#[test]
fn tm_v6_filter_option_vec_i32_skip_vec_only_no_change() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let result = filter_inner_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(type_str(&result), "Option < Vec < i32 > >");
}

#[test]
fn tm_v6_filter_nested_vec_vec_i32_skip_vec() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let result = filter_inner_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn tm_v6_filter_option_option_bool_skip_option() {
    let ty: Type = parse_quote!(Option<Option<bool>>);
    let result = filter_inner_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&result), "bool");
}

// ============================================================================
// wrap_leaf_type — various scenarios
// ============================================================================

#[test]
fn tm_v6_wrap_string_empty_skip() {
    let ty: Type = parse_quote!(String);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < String >");
}

#[test]
fn tm_v6_wrap_bool_empty_skip() {
    let ty: Type = parse_quote!(bool);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < bool >");
}

#[test]
fn tm_v6_wrap_u8_empty_skip() {
    let ty: Type = parse_quote!(u8);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < u8 >");
}

#[test]
fn tm_v6_wrap_u16_empty_skip() {
    let ty: Type = parse_quote!(u16);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < u16 >");
}

#[test]
fn tm_v6_wrap_u32_empty_skip() {
    let ty: Type = parse_quote!(u32);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < u32 >");
}

#[test]
fn tm_v6_wrap_u64_empty_skip() {
    let ty: Type = parse_quote!(u64);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < u64 >");
}

#[test]
fn tm_v6_wrap_f32_empty_skip() {
    let ty: Type = parse_quote!(f32);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < f32 >");
}

#[test]
fn tm_v6_wrap_f64_empty_skip() {
    let ty: Type = parse_quote!(f64);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < f64 >");
}

#[test]
fn tm_v6_wrap_char_empty_skip() {
    let ty: Type = parse_quote!(char);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < char >");
}

#[test]
fn tm_v6_wrap_option_i32_skip_option() {
    let ty: Type = parse_quote!(Option<i32>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&result), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn tm_v6_wrap_box_string_skip_box() {
    let ty: Type = parse_quote!(Box<String>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&result), "Box < adze :: WithLeaf < String > >");
}

#[test]
fn tm_v6_wrap_vec_option_i32_skip_both() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Vec", "Option"]));
    assert_eq!(
        type_str(&result),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn tm_v6_wrap_arc_f64_skip_arc() {
    let ty: Type = parse_quote!(Arc<f64>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Arc"]));
    assert_eq!(type_str(&result), "Arc < adze :: WithLeaf < f64 > >");
}

#[test]
fn tm_v6_wrap_vec_i32_no_skip_wraps_whole() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn tm_v6_wrap_option_string_no_skip_wraps_whole() {
    let ty: Type = parse_quote!(Option<String>);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < Option < String > >");
}

#[test]
fn tm_v6_wrap_vec_vec_i32_skip_vec() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(
        type_str(&result),
        "Vec < Vec < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn tm_v6_wrap_option_option_bool_skip_option() {
    let ty: Type = parse_quote!(Option<Option<bool>>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Option"]));
    assert_eq!(
        type_str(&result),
        "Option < Option < adze :: WithLeaf < bool > > >"
    );
}

// ============================================================================
// try_extract_inner_type — more containers
// ============================================================================

#[test]
fn tm_v6_extract_rc_u32() {
    let ty: Type = parse_quote!(Rc<u32>);
    let (inner, found) = try_extract_inner_type(&ty, "Rc", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "u32");
}

#[test]
fn tm_v6_extract_cell_i16() {
    let ty: Type = parse_quote!(Cell<i16>);
    let (inner, found) = try_extract_inner_type(&ty, "Cell", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "i16");
}

#[test]
fn tm_v6_extract_refcell_bool() {
    let ty: Type = parse_quote!(RefCell<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "RefCell", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "bool");
}

// ============================================================================
// filter_inner_type — deeper nesting and mixed containers
// ============================================================================

#[test]
fn tm_v6_filter_box_option_vec_i32_skip_all_three() {
    let ty: Type = parse_quote!(Box<Option<Vec<i32>>>);
    let result = filter_inner_type(&ty, &skip_with(&["Box", "Option", "Vec"]));
    assert_eq!(type_str(&result), "i32");
}

#[test]
fn tm_v6_filter_arc_box_string_skip_arc_box() {
    let ty: Type = parse_quote!(Arc<Box<String>>);
    let result = filter_inner_type(&ty, &skip_with(&["Arc", "Box"]));
    assert_eq!(type_str(&result), "String");
}

#[test]
fn tm_v6_filter_rc_option_f64_skip_rc_only() {
    let ty: Type = parse_quote!(Rc<Option<f64>>);
    let result = filter_inner_type(&ty, &skip_with(&["Rc"]));
    assert_eq!(type_str(&result), "Option < f64 >");
}

// ============================================================================
// wrap_leaf_type — deeper nesting
// ============================================================================

#[test]
fn tm_v6_wrap_box_arc_i32_skip_both() {
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Box", "Arc"]));
    assert_eq!(
        type_str(&result),
        "Box < Arc < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn tm_v6_wrap_vec_option_box_char_skip_all() {
    let ty: Type = parse_quote!(Vec<Option<Box<char>>>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Vec", "Option", "Box"]));
    assert_eq!(
        type_str(&result),
        "Vec < Option < Box < adze :: WithLeaf < char > > > >"
    );
}

// ============================================================================
// Roundtrip consistency: filter(extract) and extract(filter) patterns
// ============================================================================

#[test]
fn tm_v6_roundtrip_extract_then_check_filter_consistency() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (extracted, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    let filtered = filter_inner_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(type_str(&extracted), type_str(&filtered));
}

#[test]
fn tm_v6_roundtrip_option_extract_and_filter_agree() {
    let ty: Type = parse_quote!(Option<String>);
    let (extracted, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    let filtered = filter_inner_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&extracted), type_str(&filtered));
}

#[test]
fn tm_v6_roundtrip_box_extract_and_filter_agree() {
    let ty: Type = parse_quote!(Box<u8>);
    let (extracted, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&extracted), type_str(&filtered));
}

#[test]
fn tm_v6_roundtrip_nested_extract_and_filter_agree() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (extracted, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    let filtered = filter_inner_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(type_str(&extracted), type_str(&filtered));
}

// ============================================================================
// extract — with skip_over targeting deeper layers
// ============================================================================

#[test]
fn tm_v6_extract_option_box_i32_skip_option_find_box() {
    let ty: Type = parse_quote!(Option<Box<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip_with(&["Option"]));
    assert!(found);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn tm_v6_extract_arc_vec_string_skip_arc_find_vec() {
    let ty: Type = parse_quote!(Arc<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Arc"]));
    assert!(found);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn tm_v6_extract_vec_box_option_i32_skip_vec_box_find_option() {
    let ty: Type = parse_quote!(Vec<Box<Option<i32>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip_with(&["Vec", "Box"]));
    assert!(found);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn tm_v6_extract_skip_target_not_present_returns_original() {
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Box", "Arc"]));
    assert!(!found);
    assert_eq!(type_str(&result), "Box < Arc < i32 > >");
}

// ============================================================================
// filter — primitives not in skip set returned unchanged
// ============================================================================

#[test]
fn tm_v6_filter_u8_empty_skip() {
    let ty: Type = parse_quote!(u8);
    let result = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "u8");
}

#[test]
fn tm_v6_filter_f64_skip_with_vec_unchanged() {
    let ty: Type = parse_quote!(f64);
    let result = filter_inner_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(type_str(&result), "f64");
}

#[test]
fn tm_v6_filter_char_skip_with_option_unchanged() {
    let ty: Type = parse_quote!(char);
    let result = filter_inner_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&result), "char");
}

// ============================================================================
// wrap — container not in skip wraps the whole container
// ============================================================================

#[test]
fn tm_v6_wrap_box_i32_skip_vec_wraps_whole() {
    let ty: Type = parse_quote!(Box<i32>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(type_str(&result), "adze :: WithLeaf < Box < i32 > >");
}

#[test]
fn tm_v6_wrap_arc_string_skip_option_wraps_whole() {
    let ty: Type = parse_quote!(Arc<String>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Option"]));
    assert_eq!(type_str(&result), "adze :: WithLeaf < Arc < String > >");
}

// ============================================================================
// Additional extract tests for coverage
// ============================================================================

#[test]
fn tm_v6_extract_option_u8() {
    let ty: Type = parse_quote!(Option<u8>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "u8");
}

#[test]
fn tm_v6_extract_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "bool");
}

#[test]
fn tm_v6_extract_option_char() {
    let ty: Type = parse_quote!(Option<char>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "char");
}

#[test]
fn tm_v6_extract_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn tm_v6_extract_box_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "bool");
}

// ============================================================================
// Wrap with i8/i16/i64 primitives
// ============================================================================

#[test]
fn tm_v6_wrap_i8_empty_skip() {
    let ty: Type = parse_quote!(i8);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < i8 >");
}

#[test]
fn tm_v6_wrap_i16_empty_skip() {
    let ty: Type = parse_quote!(i16);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < i16 >");
}

#[test]
fn tm_v6_wrap_i64_empty_skip() {
    let ty: Type = parse_quote!(i64);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < i64 >");
}

// ============================================================================
// Filter — mismatched skip has no effect on outer container
// ============================================================================

#[test]
fn tm_v6_filter_box_i32_skip_arc_no_effect() {
    let ty: Type = parse_quote!(Box<i32>);
    let result = filter_inner_type(&ty, &skip_with(&["Arc"]));
    assert_eq!(type_str(&result), "Box < i32 >");
}

#[test]
fn tm_v6_filter_arc_u16_skip_box_no_effect() {
    let ty: Type = parse_quote!(Arc<u16>);
    let result = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_str(&result), "Arc < u16 >");
}

// ============================================================================
// Wrap with custom type names
// ============================================================================

#[test]
fn tm_v6_wrap_mytype_empty_skip() {
    let ty: Type = parse_quote!(MyType);
    let result = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_str(&result), "adze :: WithLeaf < MyType >");
}

#[test]
fn tm_v6_wrap_vec_mytype_skip_vec() {
    let ty: Type = parse_quote!(Vec<MyType>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Vec"]));
    assert_eq!(type_str(&result), "Vec < adze :: WithLeaf < MyType > >");
}

// ============================================================================
// Extract — custom type containers
// ============================================================================

#[test]
fn tm_v6_extract_wrapper_inner_custom() {
    let ty: Type = parse_quote!(Wrapper<Inner>);
    let (inner, found) = try_extract_inner_type(&ty, "Wrapper", &empty_skip());
    assert!(found);
    assert_eq!(type_str(&inner), "Inner");
}

#[test]
fn tm_v6_extract_custom_not_matching() {
    let ty: Type = parse_quote!(Wrapper<Inner>);
    let (result, found) = try_extract_inner_type(&ty, "Other", &empty_skip());
    assert!(!found);
    assert_eq!(type_str(&result), "Wrapper < Inner >");
}

// ============================================================================
// Consistency: wrap then filter recovers wrapped leaf
// ============================================================================

#[test]
fn tm_v6_wrap_then_filter_vec_recovers_wrapped() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = skip_with(&["Vec"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let leaf = filter_inner_type(&wrapped, &skip);
    assert_eq!(type_str(&leaf), "adze :: WithLeaf < i32 >");
}

#[test]
fn tm_v6_wrap_then_filter_option_recovers_wrapped() {
    let ty: Type = parse_quote!(Option<bool>);
    let skip = skip_with(&["Option"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let leaf = filter_inner_type(&wrapped, &skip);
    assert_eq!(type_str(&leaf), "adze :: WithLeaf < bool >");
}

// ============================================================================
// Edge: same name in inner_of and skip_over
// ============================================================================

#[test]
fn tm_v6_extract_vec_with_vec_in_skip_matches_first() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_with(&["Vec"]));
    // inner_of check happens before skip_over check
    assert!(found);
    assert_eq!(type_str(&inner), "i32");
}

// ============================================================================
// More complex nesting with wrap
// ============================================================================

#[test]
fn tm_v6_wrap_box_option_vec_i32_skip_all() {
    let ty: Type = parse_quote!(Box<Option<Vec<i32>>>);
    let result = wrap_leaf_type(&ty, &skip_with(&["Box", "Option", "Vec"]));
    assert_eq!(
        type_str(&result),
        "Box < Option < Vec < adze :: WithLeaf < i32 > > > >"
    );
}
