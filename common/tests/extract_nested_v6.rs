//! Comprehensive tests for nested type extraction, filtering, and wrapping
//! patterns in `adze-common`. Covers double/triple/quadruple nesting,
//! mixed container chains, skip-over interactions, and cross-function
//! composition.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
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
// 1. try_extract_inner_type — double nesting (14 tests)
// ===========================================================================

#[test]
fn extract_vec_vec_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Vec<i32>>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn extract_option_option_string() {
    let (inner, ok) =
        try_extract_inner_type(&ty("Option<Option<String>>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < String >");
}

#[test]
fn extract_vec_from_vec_option_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Option<i32>>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn extract_option_from_option_vec_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<Vec<i32>>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn extract_vec_from_box_vec_option_i32_skip_box() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Vec<Option<i32>>>"), "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn extract_option_from_box_option_string_skip_box() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Option<String>>"), "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_from_arc_vec_u8_skip_arc() {
    let (inner, ok) = try_extract_inner_type(&ty("Arc<Vec<u8>>"), "Vec", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_vec_from_box_arc_vec_bool_skip_box_arc() {
    let (inner, ok) =
        try_extract_inner_type(&ty("Box<Arc<Vec<bool>>>"), "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_option_from_box_arc_option_u64_skip_box_arc() {
    let (inner, ok) = try_extract_inner_type(
        &ty("Box<Arc<Option<u64>>>"),
        "Option",
        &skip(&["Box", "Arc"]),
    );
    assert!(ok);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn extract_vec_vec_string() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Vec<String>>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn extract_option_vec_option_i32_extracts_outer_option() {
    let (inner, ok) =
        try_extract_inner_type(&ty("Option<Vec<Option<i32>>>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < Option < i32 > >");
}

#[test]
fn extract_vec_option_vec_u32_extracts_outer_vec() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Option<Vec<u32>>>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < Vec < u32 > >");
}

#[test]
fn extract_box_from_box_box_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Box<i32>>"), "Box", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < i32 >");
}

#[test]
fn extract_hashmap_nested_in_option() {
    let (inner, ok) =
        try_extract_inner_type(&ty("Option<HashMap<String, i32>>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

// ===========================================================================
// 2. try_extract_inner_type — triple nesting (8 tests)
// ===========================================================================

#[test]
fn extract_vec_vec_vec_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Vec<Vec<i32>>>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < Vec < i32 > >");
}

#[test]
fn extract_option_option_option_bool() {
    let (inner, ok) =
        try_extract_inner_type(&ty("Option<Option<Option<bool>>>"), "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < Option < bool > >");
}

#[test]
fn extract_vec_through_box_arc_skip_both() {
    let (inner, ok) =
        try_extract_inner_type(&ty("Box<Arc<Vec<f64>>>"), "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_option_through_three_skip_layers() {
    let (inner, ok) = try_extract_inner_type(
        &ty("Box<Arc<Rc<Option<String>>>>"),
        "Option",
        &skip(&["Box", "Arc", "Rc"]),
    );
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_from_triple_box_vec_skip_box() {
    // Box<Box<Box<Vec<i32>>>> with Box in skip → drills through all 3 Box layers
    let (inner, ok) =
        try_extract_inner_type(&ty("Box<Box<Box<Vec<i32>>>>"), "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_vec_from_box_option_vec_i32_skip_box_option() {
    let (inner, ok) = try_extract_inner_type(
        &ty("Box<Option<Vec<i32>>>"),
        "Vec",
        &skip(&["Box", "Option"]),
    );
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_option_from_vec_option_string_no_skip_fails() {
    // Vec is NOT in skip set, so we can't reach Option inside it
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Option<String>>"), "Option", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < Option < String > >");
}

#[test]
fn extract_option_from_vec_option_string_skip_vec() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Option<String>>"), "Option", &skip(&["Vec"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

// ===========================================================================
// 3. try_extract_inner_type — quadruple / deep nesting (4 tests)
// ===========================================================================

#[test]
fn extract_vec_vec_vec_vec_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Vec<Vec<Vec<i32>>>>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < Vec < Vec < i32 > > >");
}

#[test]
fn extract_through_four_skip_layers() {
    let (inner, ok) = try_extract_inner_type(
        &ty("A<B<C<D<Vec<u8>>>>>"),
        "Vec",
        &skip(&["A", "B", "C", "D"]),
    );
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_option_nested_four_deep() {
    let (inner, ok) = try_extract_inner_type(
        &ty("Option<Option<Option<Option<i32>>>>"),
        "Option",
        &empty_skip(),
    );
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < Option < Option < i32 > > >");
}

#[test]
fn extract_deep_mixed_chain() {
    let (inner, ok) = try_extract_inner_type(
        &ty("Box<Arc<Rc<Cow<Vec<String>>>>>"),
        "Vec",
        &skip(&["Box", "Arc", "Rc", "Cow"]),
    );
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

// ===========================================================================
// 4. try_extract_inner_type — non-matching / false cases (6 tests)
// ===========================================================================

#[test]
fn extract_from_plain_type_not_found() {
    let (inner, ok) = try_extract_inner_type(&ty("i32"), "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_wrong_container_not_found() {
    let (inner, ok) = try_extract_inner_type(&ty("HashMap<String, i32>"), "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

#[test]
fn extract_skip_but_target_absent() {
    let (inner, ok) =
        try_extract_inner_type(&ty("Box<Arc<String>>"), "Vec", &skip(&["Box", "Arc"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < Arc < String > >");
}

#[test]
fn extract_non_path_reference_type() {
    let (inner, ok) = try_extract_inner_type(&ty("&Vec<i32>"), "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& Vec < i32 >");
}

#[test]
fn extract_non_path_tuple_type() {
    let (inner, ok) = try_extract_inner_type(&ty("(Vec<i32>, u8)"), "Vec", &empty_skip());
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(Vec < i32 > , u8)");
}

#[test]
fn extract_skip_only_partial_chain_fails() {
    // Box<Rc<Vec<i32>>> with skip={"Box"} — Rc is not skipped, so can't reach Vec
    let (inner, ok) = try_extract_inner_type(&ty("Box<Rc<Vec<i32>>>"), "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < Rc < Vec < i32 > > >");
}

// ===========================================================================
// 5. filter_inner_type — nested filtering (20 tests)
// ===========================================================================

#[test]
fn filter_vec_option_i32_skip_vec() {
    let filtered = filter_inner_type(&ty("Vec<Option<i32>>"), &skip(&["Vec"]));
    assert_eq!(ty_str(&filtered), "Option < i32 >");
}

#[test]
fn filter_vec_option_i32_skip_vec_option() {
    let filtered = filter_inner_type(&ty("Vec<Option<i32>>"), &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_vec_option_i32_skip_option_only() {
    // Vec is outermost and NOT in skip → returned unchanged
    let filtered = filter_inner_type(&ty("Vec<Option<i32>>"), &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "Vec < Option < i32 > >");
}

#[test]
fn filter_empty_skip_returns_original() {
    let filtered = filter_inner_type(&ty("Vec<Option<i32>>"), &empty_skip());
    assert_eq!(ty_str(&filtered), "Vec < Option < i32 > >");
}

#[test]
fn filter_box_arc_string_skip_box_arc() {
    let filtered = filter_inner_type(&ty("Box<Arc<String>>"), &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_box_arc_string_skip_box_only() {
    let filtered = filter_inner_type(&ty("Box<Arc<String>>"), &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Arc < String >");
}

#[test]
fn filter_box_arc_string_skip_arc_only() {
    // Box is outermost, not in skip → unchanged
    let filtered = filter_inner_type(&ty("Box<Arc<String>>"), &skip(&["Arc"]));
    assert_eq!(ty_str(&filtered), "Box < Arc < String > >");
}

#[test]
fn filter_triple_box_i32_skip_box() {
    let filtered = filter_inner_type(&ty("Box<Box<Box<i32>>>"), &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_option_option_option_bool_skip_option() {
    let filtered = filter_inner_type(&ty("Option<Option<Option<bool>>>"), &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "bool");
}

#[test]
fn filter_vec_vec_vec_i32_skip_vec() {
    let filtered = filter_inner_type(&ty("Vec<Vec<Vec<i32>>>"), &skip(&["Vec"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_box_option_vec_i32_skip_box_option_vec() {
    let filtered = filter_inner_type(
        &ty("Box<Option<Vec<i32>>>"),
        &skip(&["Box", "Option", "Vec"]),
    );
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_box_option_vec_i32_skip_box_vec() {
    // Box stripped, then Option is not in skip → stop at Option
    let filtered = filter_inner_type(&ty("Box<Option<Vec<i32>>>"), &skip(&["Box", "Vec"]));
    assert_eq!(ty_str(&filtered), "Option < Vec < i32 > >");
}

#[test]
fn filter_box_option_vec_i32_skip_box_option() {
    // Box stripped, then Option stripped → Vec<i32>
    let filtered = filter_inner_type(&ty("Box<Option<Vec<i32>>>"), &skip(&["Box", "Option"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
}

#[test]
fn filter_plain_type_skip_anything() {
    let filtered = filter_inner_type(&ty("String"), &skip(&["Box", "Vec"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_non_path_reference_unchanged() {
    let filtered = filter_inner_type(&ty("&str"), &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "& str");
}

#[test]
fn filter_non_path_tuple_unchanged() {
    let filtered = filter_inner_type(&ty("(i32, u32)"), &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(i32 , u32)");
}

#[test]
fn filter_quadruple_wrapper_skip_all() {
    let filtered = filter_inner_type(&ty("A<B<C<D<u8>>>>"), &skip(&["A", "B", "C", "D"]));
    assert_eq!(ty_str(&filtered), "u8");
}

#[test]
fn filter_vec_vec_i32_skip_vec_yields_i32() {
    let filtered = filter_inner_type(&ty("Vec<Vec<i32>>"), &skip(&["Vec"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_option_vec_i32_skip_option_only() {
    let filtered = filter_inner_type(&ty("Option<Vec<i32>>"), &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
}

#[test]
fn filter_option_vec_i32_skip_vec_only_unchanged() {
    // Option is outermost and not in skip → unchanged
    let filtered = filter_inner_type(&ty("Option<Vec<i32>>"), &skip(&["Vec"]));
    assert_eq!(ty_str(&filtered), "Option < Vec < i32 > >");
}

// ===========================================================================
// 6. wrap_leaf_type — nested wrapping (20 tests)
// ===========================================================================

#[test]
fn wrap_plain_i32_empty_skip() {
    let wrapped = wrap_leaf_type(&ty("i32"), &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_plain_string_empty_skip() {
    let wrapped = wrap_leaf_type(&ty("String"), &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_vec_i32_skip_vec() {
    let wrapped = wrap_leaf_type(&ty("Vec<i32>"), &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_option_i32_skip_option() {
    let wrapped = wrap_leaf_type(&ty("Option<i32>"), &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_vec_option_i32_skip_vec_option() {
    let wrapped = wrap_leaf_type(&ty("Vec<Option<i32>>"), &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_option_vec_i32_skip_option_vec() {
    let wrapped = wrap_leaf_type(&ty("Option<Vec<i32>>"), &skip(&["Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_vec_vec_i32_skip_vec() {
    let wrapped = wrap_leaf_type(&ty("Vec<Vec<i32>>"), &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < Vec < adze :: WithLeaf < i32 > > >");
}

#[test]
fn wrap_vec_vec_vec_i32_skip_vec() {
    let wrapped = wrap_leaf_type(&ty("Vec<Vec<Vec<i32>>>"), &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Vec < Vec < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn wrap_option_option_string_skip_option() {
    let wrapped = wrap_leaf_type(&ty("Option<Option<String>>"), &skip(&["Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_vec_i32_no_skip_wraps_entire() {
    // Vec not in skip → entire type is the leaf
    let wrapped = wrap_leaf_type(&ty("Vec<i32>"), &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn wrap_option_i32_no_skip_wraps_entire() {
    let wrapped = wrap_leaf_type(&ty("Option<i32>"), &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Option < i32 > >");
}

#[test]
fn wrap_vec_option_i32_skip_vec_only() {
    // Vec skipped but Option is not → Option<i32> becomes the leaf
    let wrapped = wrap_leaf_type(&ty("Vec<Option<i32>>"), &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < adze :: WithLeaf < Option < i32 > > >"
    );
}

#[test]
fn wrap_vec_option_i32_skip_option_only() {
    // Vec not in skip → entire Vec<Option<i32>> is the leaf
    let wrapped = wrap_leaf_type(&ty("Vec<Option<i32>>"), &skip(&["Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < Vec < Option < i32 > > >"
    );
}

#[test]
fn wrap_box_vec_option_skip_all_three() {
    let wrapped = wrap_leaf_type(
        &ty("Box<Vec<Option<i32>>>"),
        &skip(&["Box", "Vec", "Option"]),
    );
    assert_eq!(
        ty_str(&wrapped),
        "Box < Vec < Option < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn wrap_quadruple_nesting_skip_all() {
    let wrapped = wrap_leaf_type(&ty("A<B<C<D<u8>>>>"), &skip(&["A", "B", "C", "D"]));
    assert_eq!(
        ty_str(&wrapped),
        "A < B < C < D < adze :: WithLeaf < u8 > > > > >"
    );
}

#[test]
fn wrap_reference_type_wraps_entirely() {
    let wrapped = wrap_leaf_type(&ty("&str"), &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_tuple_type_wraps_entirely() {
    let wrapped = wrap_leaf_type(&ty("(i32, u32)"), &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (i32 , u32) >");
}

#[test]
fn wrap_result_skip_result_wraps_both_args() {
    let wrapped = wrap_leaf_type(&ty("Result<String, i32>"), &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_result_vec_skip_result_vec() {
    let wrapped = wrap_leaf_type(&ty("Result<Vec<i32>, String>"), &skip(&["Result", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < Vec < adze :: WithLeaf < i32 > > , adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_vec_vec_vec_vec_i32_skip_vec() {
    let wrapped = wrap_leaf_type(&ty("Vec<Vec<Vec<Vec<i32>>>>"), &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Vec < Vec < Vec < adze :: WithLeaf < i32 > > > > >"
    );
}

// ===========================================================================
// 7. Cross-function composition (12 tests)
// ===========================================================================

#[test]
fn extract_then_filter_vec_option_i32() {
    // Extract Vec from Vec<Option<i32>> → Option<i32>, then filter Option
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Option<i32>>"), "Vec", &empty_skip());
    assert!(ok);
    let filtered = filter_inner_type(&inner, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn extract_then_wrap_vec_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<i32>"), "Vec", &empty_skip());
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn filter_then_wrap_box_option_i32() {
    let filtered = filter_inner_type(&ty("Box<Option<i32>>"), &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Option < i32 >");
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn extract_filter_wrap_triple_chain() {
    // Box<Vec<Option<i32>>> → extract Vec (skip Box) → Option<i32>
    let (inner, ok) = try_extract_inner_type(&ty("Box<Vec<Option<i32>>>"), "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < i32 >");
    // filter Option → i32
    let filtered = filter_inner_type(&inner, &skip(&["Option"]));
    assert_eq!(ty_str(&filtered), "i32");
    // wrap → adze::WithLeaf<i32>
    let wrapped = wrap_leaf_type(&filtered, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn extract_vec_then_wrap_nested_option() {
    // Vec<Option<String>> → extract Vec → Option<String> → wrap with Option in skip
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Option<String>>"), "Vec", &empty_skip());
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < String > >");
}

#[test]
fn filter_box_then_extract_option() {
    let filtered = filter_inner_type(&ty("Box<Option<Vec<i32>>>"), &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Option < Vec < i32 > >");
    let (inner, ok) = try_extract_inner_type(&filtered, "Option", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn double_extract_vec_vec_vec_i32() {
    // First extract: Vec<Vec<Vec<i32>>> → Vec<Vec<i32>>
    let (inner1, ok1) = try_extract_inner_type(&ty("Vec<Vec<Vec<i32>>>"), "Vec", &empty_skip());
    assert!(ok1);
    assert_eq!(ty_str(&inner1), "Vec < Vec < i32 > >");
    // Second extract: Vec<Vec<i32>> → Vec<i32>
    let (inner2, ok2) = try_extract_inner_type(&inner1, "Vec", &empty_skip());
    assert!(ok2);
    assert_eq!(ty_str(&inner2), "Vec < i32 >");
}

#[test]
fn triple_extract_vec_vec_vec_vec_i32() {
    let original = ty("Vec<Vec<Vec<Vec<i32>>>>");
    let (a, ok1) = try_extract_inner_type(&original, "Vec", &empty_skip());
    assert!(ok1);
    let (b, ok2) = try_extract_inner_type(&a, "Vec", &empty_skip());
    assert!(ok2);
    let (c, ok3) = try_extract_inner_type(&b, "Vec", &empty_skip());
    assert!(ok3);
    assert_eq!(ty_str(&c), "Vec < i32 >");
    // Fourth extraction reaches i32
    let (d, ok4) = try_extract_inner_type(&c, "Vec", &empty_skip());
    assert!(ok4);
    assert_eq!(ty_str(&d), "i32");
}

#[test]
fn extract_fails_then_wrap_original() {
    let (inner, ok) = try_extract_inner_type(&ty("HashMap<String, i32>"), "Vec", &empty_skip());
    assert!(!ok);
    let wrapped = wrap_leaf_type(&inner, &empty_skip());
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

#[test]
fn filter_double_then_wrap() {
    // Arc<Box<Vec<i32>>> filter {Arc, Box} → Vec<i32>, then wrap with Vec in skip
    let filtered = filter_inner_type(&ty("Arc<Box<Vec<i32>>>"), &skip(&["Arc", "Box"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < i32 > >");
}

#[test]
fn filter_keeps_inner_nesting_intact() {
    // Box<Vec<Option<i32>>> filter {Box} → Vec<Option<i32>> — inner nesting preserved
    let filtered = filter_inner_type(&ty("Box<Vec<Option<i32>>>"), &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < Option < i32 > >");
}

#[test]
fn extract_preserves_deeply_nested_args() {
    // Vec<HashMap<String, Vec<i32>>> extract Vec → HashMap<String, Vec<i32>>
    let (inner, ok) =
        try_extract_inner_type(&ty("Vec<HashMap<String, Vec<i32>>>"), "Vec", &empty_skip());
    assert!(ok);
    assert_eq!(ty_str(&inner), "HashMap < String , Vec < i32 > >");
}

// ===========================================================================
// 8. Edge cases: skip_over partial matches and identity (6 tests)
// ===========================================================================

#[test]
fn extract_skip_does_not_substring_match() {
    // "Ve" is not "Vec", so skip={"Ve"} doesn't skip Vec
    let (inner, ok) = try_extract_inner_type(&ty("Vec<i32>"), "Option", &skip(&["Ve"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn filter_skip_does_not_substring_match() {
    let filtered = filter_inner_type(&ty("Vec<i32>"), &skip(&["Ve"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
}

#[test]
fn wrap_skip_does_not_substring_match() {
    // "Ve" doesn't match "Vec", so Vec<i32> is treated as a leaf
    let wrapped = wrap_leaf_type(&ty("Vec<i32>"), &skip(&["Ve"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn extract_target_same_as_skip_extracts_immediately() {
    // If inner_of == "Vec" and skip also has "Vec", the inner_of branch triggers first
    let (inner, ok) = try_extract_inner_type(&ty("Vec<i32>"), "Vec", &skip(&["Vec"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn filter_single_matching_layer() {
    let filtered = filter_inner_type(&ty("Box<i32>"), &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_non_matching_outermost_preserves_all() {
    // Rc not in skip → entire type unchanged even though Box is in skip
    let filtered = filter_inner_type(&ty("Rc<Box<i32>>"), &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Rc < Box < i32 > >");
}
