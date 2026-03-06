//! Comprehensive edge-case tests for syntax processing in adze-common.
//!
//! 100+ tests covering `try_extract_inner_type`, `wrap_leaf_type`, and
//! `filter_inner_type` across a wide range of type shapes.
//!
//! Sections:
//!   1–10   Bare primitives — extract returns (original, false)
//!  11–20   Single-layer extraction (Vec, Option, Box, etc.)
//!  21–30   Non-matching / mismatch extraction
//!  31–40   skip_over variations (empty, single, multiple)
//!  41–50   Nested container extraction
//!  51–60   wrap_leaf_type basics
//!  61–70   wrap_leaf_type with containers & nesting
//!  71–80   filter_inner_type basics
//!  81–90   filter_inner_type multi-layer & nesting
//!  91–100  Non-path types (references, tuples, arrays, slices)
//! 101–110  Complex / exotic types

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
// 1–10  Bare primitives — extract always returns (original, false)
// ===========================================================================

#[test]
fn test_01_extract_bare_i32() {
    let ty: Type = parse_quote!(i32);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_02_extract_bare_u64() {
    let ty: Type = parse_quote!(u64);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn test_03_extract_bare_bool() {
    let ty: Type = parse_quote!(bool);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn test_04_extract_bare_string() {
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_05_extract_bare_f64() {
    let ty: Type = parse_quote!(f64);
    let (inner, extracted) = try_extract_inner_type(&ty, "Rc", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn test_06_extract_bare_usize() {
    let ty: Type = parse_quote!(usize);
    let (inner, extracted) = try_extract_inner_type(&ty, "Arc", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn test_07_extract_bare_char() {
    let ty: Type = parse_quote!(char);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn test_08_extract_bare_isize() {
    let ty: Type = parse_quote!(isize);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "isize");
}

#[test]
fn test_09_extract_bare_u8() {
    let ty: Type = parse_quote!(u8);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn test_10_extract_bare_i128() {
    let ty: Type = parse_quote!(i128);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "i128");
}

// ===========================================================================
// 11–20  Single-layer extraction
// ===========================================================================

#[test]
fn test_11_extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_12_extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_13_extract_box_i32() {
    let ty: Type = parse_quote!(Box<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_14_extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_15_extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_16_extract_arc_u32() {
    let ty: Type = parse_quote!(Arc<u32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Arc", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn test_17_extract_rc_bool() {
    let ty: Type = parse_quote!(Rc<bool>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Rc", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn test_18_extract_cell_f32() {
    let ty: Type = parse_quote!(Cell<f32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Cell", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn test_19_extract_refcell_u8() {
    let ty: Type = parse_quote!(RefCell<u8>);
    let (inner, extracted) = try_extract_inner_type(&ty, "RefCell", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn test_20_extract_custom_wrapper() {
    let ty: Type = parse_quote!(MyWrapper<i64>);
    let (inner, extracted) = try_extract_inner_type(&ty, "MyWrapper", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i64");
}

// ===========================================================================
// 21–30  Non-matching / mismatch extraction
// ===========================================================================

#[test]
fn test_21_extract_vec_but_target_option() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn test_22_extract_option_but_target_vec() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Option < String >");
}

#[test]
fn test_23_extract_box_but_target_arc() {
    let ty: Type = parse_quote!(Box<u64>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Arc", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < u64 >");
}

#[test]
fn test_24_extract_hashmap_but_target_vec() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

#[test]
fn test_25_extract_case_sensitive_vec_vs_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn test_26_extract_case_sensitive_option_vs_option() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, extracted) = try_extract_inner_type(&ty, "OPTION", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Option < bool >");
}

#[test]
fn test_27_extract_custom_not_matching() {
    let ty: Type = parse_quote!(Foo<Bar>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Baz", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Foo < Bar >");
}

#[test]
fn test_28_extract_result_not_matching_vec() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Result < i32 , String >");
}

#[test]
fn test_29_extract_plain_type_with_skip_set() {
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_30_extract_skip_but_no_target_inside() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < String >");
}

// ===========================================================================
// 31–40  skip_over variations
// ===========================================================================

#[test]
fn test_31_extract_empty_skip_set_with_match() {
    let ty: Type = parse_quote!(Vec<u32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn test_32_extract_single_skip_box_wrapping_vec() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_33_extract_single_skip_arc_wrapping_option() {
    let ty: Type = parse_quote!(Arc<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_34_extract_multi_skip_box_arc_wrapping_vec() {
    let ty: Type = parse_quote!(Box<Arc<Vec<bool>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn test_35_extract_multi_skip_three_layers() {
    let ty: Type = parse_quote!(Rc<Box<Arc<Option<f64>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Rc", "Box", "Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn test_36_extract_skip_only_first_not_second() {
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    // Box is skippable, Arc is not — Arc doesn't match "Vec" target
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < Arc < i32 > >");
}

#[test]
fn test_37_extract_skip_irrelevant_entries() {
    let ty: Type = parse_quote!(Vec<u8>);
    // skip_over contains types not in the chain
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Foo", "Bar", "Baz"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn test_38_extract_skip_matches_target_not_in_skip() {
    let ty: Type = parse_quote!(Arc<Vec<char>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn test_39_extract_skip_over_same_as_target_extracts() {
    // When the outer type is both the target and in skip_over, target match takes priority
    // because the target check runs first.
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Vec"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_40_extract_deeply_nested_skip_chain() {
    let ty: Type = parse_quote!(A<B<C<D<Vec<u16>>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["A", "B", "C", "D"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u16");
}

// ===========================================================================
// 41–50  Nested container extraction
// ===========================================================================

#[test]
fn test_41_extract_vec_of_vec() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn test_42_extract_option_of_option() {
    let ty: Type = parse_quote!(Option<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < String >");
}

#[test]
fn test_43_extract_option_of_vec() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < u8 >");
}

#[test]
fn test_44_extract_vec_of_option() {
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < bool >");
}

#[test]
fn test_45_extract_box_of_vec_of_option() {
    let ty: Type = parse_quote!(Box<Vec<Option<i64>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < i64 >");
}

#[test]
fn test_46_extract_option_vec_with_option_skip() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_47_extract_nested_same_target_peels_one() {
    let ty: Type = parse_quote!(Vec<Vec<Vec<i32>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < Vec < i32 > >");
}

#[test]
fn test_48_extract_box_option_vec_multi_skip() {
    let ty: Type = parse_quote!(Box<Option<Vec<String>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Option"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn test_49_extract_result_first_arg() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_50_extract_custom_nested() {
    let ty: Type = parse_quote!(Outer<Inner<Leaf>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Inner", &skip(&["Outer"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Leaf");
}

// ===========================================================================
// 51–60  wrap_leaf_type basics
// ===========================================================================

#[test]
fn test_51_wrap_bare_i32() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn test_52_wrap_bare_string() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn test_53_wrap_bare_bool() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < bool >");
}

#[test]
fn test_54_wrap_bare_u64() {
    let ty: Type = parse_quote!(u64);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u64 >");
}

#[test]
fn test_55_wrap_bare_f32() {
    let ty: Type = parse_quote!(f32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f32 >");
}

#[test]
fn test_56_wrap_bare_char() {
    let ty: Type = parse_quote!(char);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < char >");
}

#[test]
fn test_57_wrap_bare_usize() {
    let ty: Type = parse_quote!(usize);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < usize >");
}

#[test]
fn test_58_wrap_custom_type_as_leaf() {
    let ty: Type = parse_quote!(MyType);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < MyType >");
}

#[test]
fn test_59_wrap_generic_not_in_skip() {
    let ty: Type = parse_quote!(Foo<Bar>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Foo < Bar > >");
}

#[test]
fn test_60_wrap_hashmap_not_in_skip() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

// ===========================================================================
// 61–70  wrap_leaf_type with containers & nesting
// ===========================================================================

#[test]
fn test_61_wrap_vec_string_with_vec_skip() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn test_62_wrap_option_i32_with_option_skip() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn test_63_wrap_vec_option_both_skip() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn test_64_wrap_option_vec_both_skip() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn test_65_wrap_nested_vec_vec_with_vec_skip() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < Vec < adze :: WithLeaf < i32 > > >");
}

#[test]
fn test_66_wrap_result_with_result_skip() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn test_67_wrap_box_not_in_skip_wraps_whole() {
    let ty: Type = parse_quote!(Box<u8>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Box < u8 > >");
}

#[test]
fn test_68_wrap_vec_with_empty_skip_wraps_whole() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn test_69_wrap_triple_nested_all_skip() {
    let ty: Type = parse_quote!(A<B<C<i32>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["A", "B", "C"]));
    assert_eq!(
        ty_str(&wrapped),
        "A < B < C < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn test_70_wrap_mixed_skip_and_non_skip() {
    let ty: Type = parse_quote!(Vec<HashMap<String, i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < adze :: WithLeaf < HashMap < String , i32 > > >"
    );
}

// ===========================================================================
// 71–80  filter_inner_type basics
// ===========================================================================

#[test]
fn test_71_filter_empty_skip_identity() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), "Box < String >");
}

#[test]
fn test_72_filter_bare_type_identity() {
    let ty: Type = parse_quote!(i32);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn test_73_filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn test_74_filter_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn test_75_filter_arc_u64() {
    let ty: Type = parse_quote!(Arc<u64>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&filtered), "u64");
}

#[test]
fn test_76_filter_rc_bool() {
    let ty: Type = parse_quote!(Rc<bool>);
    let filtered = filter_inner_type(&ty, &skip(&["Rc"]));
    assert_eq!(ty_str(&filtered), "bool");
}

#[test]
fn test_77_filter_non_matching_skip() {
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
}

#[test]
fn test_78_filter_bare_string_identity() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn test_79_filter_custom_wrapper() {
    let ty: Type = parse_quote!(MyBox<char>);
    let filtered = filter_inner_type(&ty, &skip(&["MyBox"]));
    assert_eq!(ty_str(&filtered), "char");
}

#[test]
fn test_80_filter_case_sensitive_no_match() {
    let ty: Type = parse_quote!(Box<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["box"]));
    assert_eq!(ty_str(&filtered), "Box < i32 >");
}

// ===========================================================================
// 81–90  filter_inner_type multi-layer & nesting
// ===========================================================================

#[test]
fn test_81_filter_box_arc_string() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn test_82_filter_three_layers() {
    let ty: Type = parse_quote!(Rc<Box<Arc<u32>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Rc", "Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "u32");
}

#[test]
fn test_83_filter_stops_at_non_skip() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
}

#[test]
fn test_84_filter_option_option_both_skip() {
    let ty: Type = parse_quote!(Option<Option<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn test_85_filter_deeply_nested() {
    let ty: Type = parse_quote!(A<B<C<D<i32>>>>);
    let filtered = filter_inner_type(&ty, &skip(&["A", "B", "C", "D"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn test_86_filter_partial_chain() {
    let ty: Type = parse_quote!(A<B<C<i32>>>);
    // Only A and B are skipped; C is not
    let filtered = filter_inner_type(&ty, &skip(&["A", "B"]));
    assert_eq!(ty_str(&filtered), "C < i32 >");
}

#[test]
fn test_87_filter_box_option_vec() {
    let ty: Type = parse_quote!(Box<Option<Vec<u8>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Option"]));
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
}

#[test]
fn test_88_filter_skip_irrelevant_entries() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Foo", "Bar"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn test_89_filter_vec_vec_with_vec_skip() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn test_90_filter_five_layers() {
    let ty: Type = parse_quote!(W<X<Y<Z<Q<bool>>>>>);
    let filtered = filter_inner_type(&ty, &skip(&["W", "X", "Y", "Z", "Q"]));
    assert_eq!(ty_str(&filtered), "bool");
}

// ===========================================================================
// 91–100  Non-path types (references, tuples, arrays, slices)
// ===========================================================================

#[test]
fn test_91_extract_ref_str_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn test_92_extract_ref_i32_returns_unchanged() {
    let ty: Type = parse_quote!(&i32);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& i32");
}

#[test]
fn test_93_extract_tuple_returns_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "(i32 , u32)");
}

#[test]
fn test_94_extract_array_returns_unchanged() {
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "[u8 ; 4]");
}

#[test]
fn test_95_filter_ref_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "& str");
}

#[test]
fn test_96_filter_tuple_returns_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(i32 , u32)");
}

#[test]
fn test_97_filter_array_returns_unchanged() {
    let ty: Type = parse_quote!([u8; 4]);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "[u8 ; 4]");
}

#[test]
fn test_98_wrap_ref_wraps_entirely() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn test_99_wrap_tuple_wraps_entirely() {
    let ty: Type = parse_quote!((i32, u32));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (i32 , u32) >");
}

#[test]
fn test_100_wrap_array_wraps_entirely() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

// ===========================================================================
// 101–110  Complex / exotic types
// ===========================================================================

#[test]
fn test_101_extract_qualified_path_type() {
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn test_102_filter_qualified_path() {
    let ty: Type = parse_quote!(std::boxed::Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn test_103_wrap_qualified_path_not_in_skip() {
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < std :: vec :: Vec < i32 > >"
    );
}

#[test]
fn test_104_wrap_qualified_path_in_skip() {
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "std :: vec :: Vec < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn test_105_extract_mut_ref_returns_unchanged() {
    let ty: Type = parse_quote!(&mut i32);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& mut i32");
}

#[test]
fn test_106_wrap_mut_ref_wraps_entirely() {
    let ty: Type = parse_quote!(&mut String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & mut String >");
}

#[test]
fn test_107_extract_slice_returns_unchanged() {
    let ty: Type = parse_quote!(&[u8]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "& [u8]");
}

#[test]
fn test_108_wrap_slice_wraps_entirely() {
    let ty: Type = parse_quote!(&[u8]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & [u8] >");
}

#[test]
fn test_109_extract_two_step_sequential() {
    // First extract Vec from Option<Vec<i32>> (skipping Option), then extract again
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (mid, extracted1) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(extracted1);
    assert_eq!(ty_str(&mid), "i32");

    // Extracting Vec again from i32 yields no match
    let (final_ty, extracted2) = try_extract_inner_type(&mid, "Vec", &skip(&[]));
    assert!(!extracted2);
    assert_eq!(ty_str(&final_ty), "i32");
}

#[test]
fn test_110_filter_then_wrap_roundtrip() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "String");

    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn test_111_extract_and_wrap_composition() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn test_112_wrap_option_option_nested_skip() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn test_113_filter_single_layer_same_as_skip() {
    let ty: Type = parse_quote!(Box<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn test_114_extract_empty_tuple_returns_unchanged() {
    let ty: Type = parse_quote!(());
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "()");
}

#[test]
fn test_115_wrap_empty_tuple() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < () >");
}
