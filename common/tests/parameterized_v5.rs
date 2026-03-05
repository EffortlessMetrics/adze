//! Tests for `is_parameterized` (local helper) and the exported type
//! introspection functions: `try_extract_inner_type`, `filter_inner_type`,
//! and `wrap_leaf_type`.
//!
//! Eight categories × 8 tests each = 64 tests.

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{self, Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns `true` when `ty` is a path whose last segment has angle-bracketed
/// generic arguments — the heuristic the adze pipeline uses.
fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. Primitive types are NOT parameterized  (8 tests)
// ===========================================================================

#[test]
fn prim_i32_not_parameterized() {
    let ty: Type = syn::parse_str("i32").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn prim_u64_not_parameterized() {
    let ty: Type = syn::parse_str("u64").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn prim_f32_not_parameterized() {
    let ty: Type = syn::parse_str("f32").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn prim_bool_not_parameterized() {
    let ty: Type = syn::parse_str("bool").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn prim_char_not_parameterized() {
    let ty: Type = syn::parse_str("char").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn prim_usize_not_parameterized() {
    let ty: Type = syn::parse_str("usize").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn prim_isize_not_parameterized() {
    let ty: Type = syn::parse_str("isize").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn prim_string_not_parameterized() {
    let ty: Type = syn::parse_str("String").unwrap();
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 2. Container types ARE parameterized  (8 tests)
// ===========================================================================

#[test]
fn container_option_is_parameterized() {
    let ty: Type = syn::parse_str("Option<i32>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn container_vec_is_parameterized() {
    let ty: Type = syn::parse_str("Vec<String>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn container_box_is_parameterized() {
    let ty: Type = syn::parse_str("Box<u8>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn container_hashmap_is_parameterized() {
    let ty: Type = syn::parse_str("HashMap<String, i32>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn container_result_is_parameterized() {
    let ty: Type = syn::parse_str("Result<(), String>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn container_arc_is_parameterized() {
    let ty: Type = syn::parse_str("Arc<String>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn container_rc_is_parameterized() {
    let ty: Type = syn::parse_str("Rc<bool>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn container_btreeset_is_parameterized() {
    let ty: Type = syn::parse_str("BTreeSet<u32>").unwrap();
    assert!(is_parameterized(&ty));
}

// ===========================================================================
// 3. Nested parameterized types  (8 tests)
// ===========================================================================

#[test]
fn nested_option_vec_is_parameterized() {
    let ty: Type = syn::parse_str("Option<Vec<i32>>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_vec_option_is_parameterized() {
    let ty: Type = syn::parse_str("Vec<Option<String>>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_box_vec_option_is_parameterized() {
    let ty: Type = syn::parse_str("Box<Vec<Option<u8>>>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_result_option_is_parameterized() {
    let ty: Type = syn::parse_str("Result<Option<i32>, String>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_option_option_is_parameterized() {
    let ty: Type = syn::parse_str("Option<Option<bool>>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_vec_vec_is_parameterized() {
    let ty: Type = syn::parse_str("Vec<Vec<f64>>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_hashmap_vec_is_parameterized() {
    let ty: Type = syn::parse_str("HashMap<String, Vec<u32>>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn nested_extract_through_skip_box() {
    let ty: Type = syn::parse_str("Box<Vec<u8>>").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

// ===========================================================================
// 4. References are NOT parameterized  (8 tests)
// ===========================================================================

#[test]
fn ref_str_not_parameterized() {
    let ty: Type = syn::parse_str("&str").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_mut_i32_not_parameterized() {
    let ty: Type = syn::parse_str("&mut i32").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_slice_u8_not_parameterized() {
    let ty: Type = syn::parse_str("&[u8]").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_lifetime_str_not_parameterized() {
    let ty: Type = syn::parse_str("&'a str").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_lifetime_mut_not_parameterized() {
    let ty: Type = syn::parse_str("&'a mut u32").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_option_is_not_parameterized() {
    // Outermost type is a reference, so `is_parameterized` returns false.
    let ty: Type = syn::parse_str("&Option<i32>").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn ref_extract_returns_unchanged() {
    let ty: Type = syn::parse_str("&str").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn ref_mut_filter_returns_unchanged() {
    let ty: Type = syn::parse_str("&mut String").unwrap();
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "& mut String");
}

// ===========================================================================
// 5. Arrays and tuples  (8 tests)
// ===========================================================================

#[test]
fn array_u8_not_parameterized() {
    let ty: Type = syn::parse_str("[u8; 4]").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn array_i32_large_not_parameterized() {
    let ty: Type = syn::parse_str("[i32; 256]").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn array_wrap_leaf_wraps_entirely() {
    let ty: Type = syn::parse_str("[u8; 4]").unwrap();
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn tuple_pair_not_parameterized() {
    let ty: Type = syn::parse_str("(i32, String)").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn tuple_unit_not_parameterized() {
    let ty: Type = syn::parse_str("()").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn tuple_triple_not_parameterized() {
    let ty: Type = syn::parse_str("(bool, u8, f64)").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn tuple_filter_returns_unchanged() {
    let ty: Type = syn::parse_str("(i32, u32)").unwrap();
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(i32 , u32)");
}

#[test]
fn tuple_single_not_parameterized() {
    let ty: Type = syn::parse_str("(i32,)").unwrap();
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 6. Function pointer types  (8 tests)
// ===========================================================================

#[test]
fn fn_ptr_not_parameterized() {
    let ty: Type = syn::parse_str("fn() -> i32").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn fn_ptr_with_args_not_parameterized() {
    let ty: Type = syn::parse_str("fn(i32, u32) -> bool").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn fn_ptr_no_return_not_parameterized() {
    let ty: Type = syn::parse_str("fn(String)").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn fn_ptr_unit_not_parameterized() {
    let ty: Type = syn::parse_str("fn()").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn fn_ptr_extract_returns_unchanged() {
    let ty: Type = syn::parse_str("fn() -> i32").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "fn () -> i32");
}

#[test]
fn fn_ptr_filter_returns_unchanged() {
    let ty: Type = syn::parse_str("fn(u8) -> bool").unwrap();
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "fn (u8) -> bool");
}

#[test]
fn fn_ptr_wrap_leaf_wraps_entirely() {
    let ty: Type = syn::parse_str("fn() -> i32").unwrap();
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < fn () -> i32 >");
}

#[test]
fn fn_ptr_multi_arg_not_parameterized() {
    let ty: Type = syn::parse_str("fn(bool, char, u64) -> String").unwrap();
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 7. Qualified paths  (8 tests)
// ===========================================================================

#[test]
fn qualified_vec_is_parameterized() {
    let ty: Type = syn::parse_str("std::vec::Vec<i32>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn qualified_option_is_parameterized() {
    let ty: Type = syn::parse_str("std::option::Option<u8>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn qualified_hashmap_is_parameterized() {
    let ty: Type = syn::parse_str("std::collections::HashMap<String, i32>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn qualified_path_no_generics_not_parameterized() {
    let ty: Type = syn::parse_str("crate::MyType").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn qualified_module_path_not_parameterized() {
    let ty: Type = syn::parse_str("foo::bar::Baz").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn qualified_super_path_not_parameterized() {
    let ty: Type = syn::parse_str("super::Item").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn qualified_crate_generic_is_parameterized() {
    let ty: Type = syn::parse_str("crate::Wrapper<u64>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn qualified_self_not_parameterized() {
    let ty: Type = syn::parse_str("Self").unwrap();
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 8. Edge cases  (8 tests)
// ===========================================================================

#[test]
fn edge_unit_type_not_parameterized() {
    let ty: Type = parse_quote!(());
    assert!(!is_parameterized(&ty));
}

#[test]
fn edge_never_type_not_parameterized() {
    let ty: Type = syn::parse_str("!").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn edge_raw_ptr_const_not_parameterized() {
    let ty: Type = syn::parse_str("*const u8").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn edge_raw_ptr_mut_not_parameterized() {
    let ty: Type = syn::parse_str("*mut i32").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn edge_trait_object_not_parameterized() {
    let ty: Type = syn::parse_str("dyn std::fmt::Debug").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn edge_wrap_never_type() {
    let ty: Type = syn::parse_str("!").unwrap();
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < ! >");
}

#[test]
fn edge_extract_from_plain_type_no_match() {
    let ty: Type = syn::parse_str("String").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn edge_filter_empty_skip_returns_original() {
    let ty: Type = syn::parse_str("Box<String>").unwrap();
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), "Box < String >");
}
