//! Comprehensive tests for grammar-level type utilities in adze-common (v7).
//!
//! Covers try_extract_inner_type, filter_inner_type, wrap_leaf_type,
//! and their interactions with grammar-level patterns including leaf types,
//! optional fields, repeated fields, boxed fields, complex nesting, and edge cases.

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote};

// ═══════════════════════════════════════════════════════════════════
// Helper utilities
// ═══════════════════════════════════════════════════════════════════

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn to_s(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ═══════════════════════════════════════════════════════════════════
// 1. Grammar leaf type extraction (8 tests)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn leaf_extract_string_from_option() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "String");
}

#[test]
fn leaf_extract_u32_from_vec() {
    let ty: Type = parse_quote!(Vec<u32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "u32");
}

#[test]
fn leaf_extract_bool_from_box() {
    let ty: Type = parse_quote!(Box<bool>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "bool");
}

#[test]
fn leaf_extract_custom_type_from_option() {
    let ty: Type = parse_quote!(Option<MyNode>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "MyNode");
}

#[test]
fn leaf_extract_no_match_returns_original() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "HashMap < String , i32 >");
}

#[test]
fn leaf_extract_plain_type_not_extracted() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "String");
}

#[test]
fn leaf_extract_first_arg_from_result() {
    let ty: Type = parse_quote!(Result<TokenTree, ParseError>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "TokenTree");
}

#[test]
fn leaf_extract_usize_from_option() {
    let ty: Type = parse_quote!(Option<usize>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "usize");
}

// ═══════════════════════════════════════════════════════════════════
// 2. Grammar optional field types (8 tests)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn optional_extract_through_box() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "String");
}

#[test]
fn optional_extract_through_arc() {
    let ty: Type = parse_quote!(Arc<Option<i64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "i64");
}

#[test]
fn optional_not_found_in_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "Box < String >");
}

#[test]
fn optional_extract_through_multiple_skips() {
    let ty: Type = parse_quote!(Box<Arc<Option<f64>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "f64");
}

#[test]
fn optional_direct_option_with_skip_set() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Vec < u8 >");
}

#[test]
fn optional_skip_type_not_in_set_blocks_extraction() {
    let ty: Type = parse_quote!(Rc<Option<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "Rc < Option < String > >");
}

#[test]
fn optional_extract_nested_generic_inner() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Vec < String >");
}

#[test]
fn optional_extract_option_of_option() {
    let ty: Type = parse_quote!(Option<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Option < bool >");
}

// ═══════════════════════════════════════════════════════════════════
// 3. Grammar repeated field types (8 tests)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn repeated_extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "String");
}

#[test]
fn repeated_extract_vec_through_box() {
    let ty: Type = parse_quote!(Box<Vec<Token>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Token");
}

#[test]
fn repeated_extract_vec_through_arc_box() {
    let ty: Type = parse_quote!(Arc<Box<Vec<Statement>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc", "Box"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Statement");
}

#[test]
fn repeated_vec_of_option() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Option < i32 >");
}

#[test]
fn repeated_vec_of_box() {
    let ty: Type = parse_quote!(Vec<Box<Expr>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Box < Expr >");
}

#[test]
fn repeated_not_vec_returns_original() {
    let ty: Type = parse_quote!(HashSet<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "HashSet < String >");
}

#[test]
fn repeated_vec_of_tuple() {
    let ty: Type = parse_quote!(Vec<(String, u32)>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "(String , u32)");
}

#[test]
fn repeated_vec_of_path_type() {
    let ty: Type = parse_quote!(Vec<std::string::String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "std :: string :: String");
}

// ═══════════════════════════════════════════════════════════════════
// 4. Grammar boxed field types (5 tests)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn boxed_filter_removes_box() {
    let ty: Type = parse_quote!(Box<Expression>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_s(&filtered), "Expression");
}

#[test]
fn boxed_filter_removes_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<Statement>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(to_s(&filtered), "Statement");
}

#[test]
fn boxed_filter_preserves_non_skip_container() {
    let ty: Type = parse_quote!(Rc<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_s(&filtered), "Rc < String >");
}

#[test]
fn boxed_filter_empty_skip_set_preserves_all() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(to_s(&filtered), "Box < Vec < String > >");
}

#[test]
fn boxed_filter_deep_nesting() {
    let ty: Type = parse_quote!(Arc<Box<Arc<u8>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(to_s(&filtered), "u8");
}

// ═══════════════════════════════════════════════════════════════════
// 5. Complex grammar type patterns (8 tests)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn complex_option_of_vec_extract_option() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Vec < String >");
}

#[test]
fn complex_vec_of_option_extract_vec() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Option < i32 >");
}

#[test]
fn complex_box_vec_option_extract_option_through_box() {
    let ty: Type = parse_quote!(Box<Vec<Option<String>>>);
    // Box is skipped, Vec is found but we look for Option — so no match through Vec
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "Box < Vec < Option < String > > >");
}

#[test]
fn complex_filter_then_extract() {
    let ty: Type = parse_quote!(Arc<Box<Vec<Token>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(to_s(&filtered), "Vec < Token >");
    let (inner, ok) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "Token");
}

#[test]
fn complex_wrap_option_of_string() {
    let ty: Type = parse_quote!(Option<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(to_s(&wrapped), "Option < adze :: WithLeaf < String > >");
}

#[test]
fn complex_wrap_vec_of_option_of_type() {
    let ty: Type = parse_quote!(Vec<Option<Identifier>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        to_s(&wrapped),
        "Vec < Option < adze :: WithLeaf < Identifier > > >"
    );
}

#[test]
fn complex_extract_then_wrap() {
    let ty: Type = parse_quote!(Vec<MyNode>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(to_s(&wrapped), "adze :: WithLeaf < MyNode >");
}

#[test]
fn complex_double_option_wrap() {
    let ty: Type = parse_quote!(Option<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(
        to_s(&wrapped),
        "Option < Option < adze :: WithLeaf < String > > >"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 6. Type predicates on grammar types (5 tests)
//    Implemented as extraction-based checks since there are no
//    standalone is_option/is_vec/is_box predicates.
// ═══════════════════════════════════════════════════════════════════

#[test]
fn predicate_is_option_via_extraction() {
    let ty: Type = parse_quote!(Option<u8>);
    let (_, is_option) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(is_option);
}

#[test]
fn predicate_is_vec_via_extraction() {
    let ty: Type = parse_quote!(Vec<char>);
    let (_, is_vec) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(is_vec);
}

#[test]
fn predicate_is_box_via_extraction() {
    let ty: Type = parse_quote!(Box<f32>);
    let (_, is_box) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(is_box);
}

#[test]
fn predicate_string_is_not_option() {
    let ty: Type = parse_quote!(String);
    let (_, is_option) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!is_option);
}

#[test]
fn predicate_vec_is_not_option() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (_, is_option) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!is_option);
}

// ═══════════════════════════════════════════════════════════════════
// 7. Wrapping for code generation (5 tests)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn wrap_plain_primitive() {
    let ty: Type = parse_quote!(i64);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(to_s(&wrapped), "adze :: WithLeaf < i64 >");
}

#[test]
fn wrap_vec_skipped_inner_wrapped() {
    let ty: Type = parse_quote!(Vec<Stmt>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(to_s(&wrapped), "Vec < adze :: WithLeaf < Stmt > >");
}

#[test]
fn wrap_result_in_skip_set_wraps_both_args() {
    let ty: Type = parse_quote!(Result<OkType, ErrType>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        to_s(&wrapped),
        "Result < adze :: WithLeaf < OkType > , adze :: WithLeaf < ErrType > >"
    );
}

#[test]
fn wrap_non_skip_container_wraps_entirely() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        to_s(&wrapped),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

#[test]
fn wrap_deeply_nested_skip_types() {
    let ty: Type = parse_quote!(Vec<Option<Box<Leaf>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(
        to_s(&wrapped),
        "Vec < Option < Box < adze :: WithLeaf < Leaf > > > >"
    );
}

// ═══════════════════════════════════════════════════════════════════
// 8. Edge cases (8 tests)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn edge_reference_type_not_extracted() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "& str");
}

#[test]
fn edge_tuple_type_not_extracted() {
    let ty: Type = parse_quote!((i32, f64));
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(to_s(&inner), "(i32 , f64)");
}

#[test]
fn edge_array_type_wrapped() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(to_s(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn edge_filter_reference_type_unchanged() {
    let ty: Type = parse_quote!(&mut Vec<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_s(&filtered), "& mut Vec < String >");
}

#[test]
fn edge_filter_tuple_type_unchanged() {
    let ty: Type = parse_quote!((String, i32));
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(to_s(&filtered), "(String , i32)");
}

#[test]
fn edge_qualified_path_type_not_extracted() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    // The last segment is "Vec" so it should match
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(to_s(&inner), "u8");
}

#[test]
fn edge_wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(to_s(&wrapped), "adze :: WithLeaf < () >");
}

#[test]
fn edge_same_type_in_extract_and_skip() {
    // If "Vec" is both the target and in the skip set,
    // the target match takes priority (checked first in the function).
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Vec"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "String");
}

// ═══════════════════════════════════════════════════════════════════
// Additional tests for complete coverage
// ═══════════════════════════════════════════════════════════════════

#[test]
fn wrap_slice_type() {
    let ty: Type = parse_quote!(&[u8]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(to_s(&wrapped), "adze :: WithLeaf < & [u8] >");
}

#[test]
fn filter_arc_box_chain() {
    let ty: Type = parse_quote!(Arc<Box<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(to_s(&filtered), "String");
}

#[test]
fn extract_option_through_box_arc_chain() {
    let ty: Type = parse_quote!(Box<Arc<Option<MyNode>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(to_s(&inner), "MyNode");
}

#[test]
fn wrap_option_vec_layered() {
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        to_s(&wrapped),
        "Option < Vec < adze :: WithLeaf < u32 > > >"
    );
}

#[test]
fn filter_single_box_preserves_inner_option() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(to_s(&filtered), "Option < String >");
}
