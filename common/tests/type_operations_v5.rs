//! Comprehensive tests for type operations in adze-common (v5).
//!
//! 55+ tests covering `try_extract_inner_type`, `filter_inner_type`,
//! `wrap_leaf_type`, and `is_parameterized` (local helper) across
//! extraction, filtering, wrapping, parameterization, edge cases,
//! consistency checks, and qualified paths.

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{self, Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Local parameterization check — mirrors the pipeline heuristic for detecting
/// whether a type has angle-bracketed generic parameters.
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
// 1. Extraction from standard containers (7 tests)
// ===========================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_box_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_result_first_arg() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_arc_f64() {
    let ty: Type = parse_quote!(Arc<f64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Arc", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_rc_usize() {
    let ty: Type = parse_quote!(Rc<usize>);
    let (inner, ok) = try_extract_inner_type(&ty, "Rc", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn extract_mismatch_returns_original() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

// ===========================================================================
// 2. Nested extraction (6 tests)
// ===========================================================================

#[test]
fn extract_option_from_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn extract_vec_from_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn extract_through_box_to_vec() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_through_arc_box_to_option() {
    let ty: Type = parse_quote!(Arc<Box<Option<u16>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn extract_through_three_skips() {
    let ty: Type = parse_quote!(Rc<Arc<Box<Vec<bool>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Rc", "Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_skip_present_but_target_absent() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

// ===========================================================================
// 3. Filter with specific outer types (8 tests)
// ===========================================================================

#[test]
fn filter_box_to_inner() {
    let ty: Type = parse_quote!(Box<i64>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "i64");
}

#[test]
fn filter_arc_to_inner() {
    let ty: Type = parse_quote!(Arc<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Arc"]))), "String");
}

#[test]
fn filter_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<u32>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "u32"
    );
}

#[test]
fn filter_triple_nesting_rc_arc_box() {
    let ty: Type = parse_quote!(Rc<Arc<Box<f32>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Rc", "Arc", "Box"]))),
        "f32"
    );
}

#[test]
fn filter_stops_at_non_skip() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
}

#[test]
fn filter_empty_skip_unchanged() {
    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Option < String >"
    );
}

#[test]
fn filter_non_matching_skip_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "Vec < i32 >"
    );
}

#[test]
fn filter_non_path_type_tuple() {
    let ty: Type = parse_quote!((i32, bool));
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "(i32 , bool)"
    );
}

// ===========================================================================
// 4. Wrapping generates valid token streams (8 tests)
// ===========================================================================

#[test]
fn wrap_simple_type_string() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_i32_produces_valid_type() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    let tokens = wrapped.to_token_stream().to_string();
    assert!(syn::parse_str::<Type>(&tokens).is_ok());
}

#[test]
fn wrap_vec_with_skip_wraps_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_vec_nested_both_skipped() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wrap_result_skips_both_args_wrapped() {
    let ty: Type = parse_quote!(Result<u32, String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < u32 > , adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_no_skip_wraps_entire_container() {
    let ty: Type = parse_quote!(Vec<u8>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < u8 > >");
}

#[test]
fn wrap_output_is_reparseable() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    let tokens = wrapped.to_token_stream().to_string();
    let reparsed = syn::parse_str::<Type>(&tokens).unwrap();
    assert_eq!(ty_str(&reparsed), ty_str(&wrapped));
}

#[test]
fn wrap_output_contains_with_leaf() {
    let ty: Type = parse_quote!(f64);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    let tokens = wrapped.to_token_stream().to_string();
    assert!(tokens.contains("WithLeaf"));
}

// ===========================================================================
// 5. Parameterized detection (10 tests)
// ===========================================================================

#[test]
fn param_vec_string_is_parameterized() {
    let ty: Type = parse_quote!(Vec<String>);
    assert!(is_parameterized(&ty));
}

#[test]
fn param_option_i32_is_parameterized() {
    let ty: Type = parse_quote!(Option<i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn param_hashmap_is_parameterized() {
    let ty: Type = parse_quote!(HashMap<String, u64>);
    assert!(is_parameterized(&ty));
}

#[test]
fn param_result_is_parameterized() {
    let ty: Type = parse_quote!(Result<Vec<u8>, Error>);
    assert!(is_parameterized(&ty));
}

#[test]
fn param_box_dyn_is_parameterized() {
    let ty: Type = parse_quote!(Box<dyn Display>);
    assert!(is_parameterized(&ty));
}

#[test]
fn param_plain_string_not_parameterized() {
    let ty: Type = parse_quote!(String);
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_i32_not_parameterized() {
    let ty: Type = parse_quote!(i32);
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_bool_not_parameterized() {
    let ty: Type = parse_quote!(bool);
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_reference_not_parameterized() {
    let ty: Type = parse_quote!(&str);
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_unit_not_parameterized() {
    let ty: Type = parse_quote!(());
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 6. Edge cases: primitives, references, arrays, tuples, fn ptrs (8 tests)
// ===========================================================================

#[test]
fn edge_all_numeric_primitives_not_parameterized() {
    for name in &[
        "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128",
    ] {
        let ty = syn::parse_str::<Type>(name).unwrap();
        assert!(!is_parameterized(&ty), "{name} should not be parameterized");
    }
}

#[test]
fn edge_reference_type_extract_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn edge_array_type_extract_unchanged() {
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "[u8 ; 4]");
}

#[test]
fn edge_tuple_extract_unchanged() {
    let ty: Type = parse_quote!((i32, bool, String));
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(i32 , bool , String)");
}

#[test]
fn edge_fn_pointer_not_parameterized() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    assert!(!is_parameterized(&ty));
}

#[test]
fn edge_wrap_array_type() {
    let ty: Type = parse_quote!([u8; 8]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 8] >");
}

#[test]
fn edge_wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, String));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (i32 , String) >");
}

#[test]
fn edge_wrap_fn_pointer() {
    let ty: Type = parse_quote!(fn(u8) -> u8);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < fn (u8) -> u8 >");
}

// ===========================================================================
// 7. Consistency: extract matches filter behavior (5 tests)
// ===========================================================================

#[test]
fn consistency_extract_and_filter_box() {
    let ty: Type = parse_quote!(Box<i32>);
    let (extracted, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}

#[test]
fn consistency_extract_and_filter_nested() {
    let ty: Type = parse_quote!(Arc<Box<String>>);
    let (extracted, ok) = try_extract_inner_type(&ty, "Box", &skip(&["Arc"]));
    assert!(ok);
    // extract target=Box through Arc gives String
    assert_eq!(ty_str(&extracted), "String");
    // filter through Arc,Box also gives String
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn consistency_extract_then_wrap_roundtrip() {
    let ty: Type = parse_quote!(Vec<f64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn consistency_filter_then_wrap_roundtrip() {
    let ty: Type = parse_quote!(Box<Arc<u16>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "u16");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u16 >");
}

#[test]
fn consistency_extract_inner_is_parameterized() {
    let ty: Type = parse_quote!(Box<Option<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert!(is_parameterized(&inner));
    assert_eq!(ty_str(&inner), "Option < u8 >");
}

// ===========================================================================
// 8. Complex qualified paths (3 tests)
// ===========================================================================

#[test]
fn qualified_std_option() {
    let ty: Type = parse_quote!(std::option::Option<u32>);
    // try_extract checks last segment, so "Option" still matches
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn qualified_std_vec() {
    let ty: Type = parse_quote!(std::vec::Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn qualified_path_filter() {
    let ty: Type = parse_quote!(std::boxed::Box<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "i32");
}

// ===========================================================================
// 9. Deep nesting and multi-parameter types (5 tests)
// ===========================================================================

#[test]
fn deep_four_level_wrap() {
    let ty: Type = parse_quote!(Option<Vec<Box<Arc<i32>>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec", "Box", "Arc"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < Box < Arc < adze :: WithLeaf < i32 > > > > >"
    );
}

#[test]
fn deep_five_level_filter() {
    let ty: Type = parse_quote!(A<B<C<D<E<u8>>>>>);
    let filtered = filter_inner_type(&ty, &skip(&["A", "B", "C", "D", "E"]));
    assert_eq!(ty_str(&filtered), "u8");
}

#[test]
fn deep_nested_parameterized_check() {
    let ty: Type = parse_quote!(Vec<Option<Box<String>>>);
    assert!(is_parameterized(&ty));
}

#[test]
fn deep_extract_through_four_skips() {
    let ty: Type = parse_quote!(W<X<Y<Z<Vec<bool>>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["W", "X", "Y", "Z"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn wrap_result_with_nested_containers() {
    let ty: Type = parse_quote!(Result<Option<u8>, Vec<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result", "Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < Option < adze :: WithLeaf < u8 > > , Vec < adze :: WithLeaf < String > > >"
    );
}

// ===========================================================================
// 10. Wrap preserves type name in output (3 tests)
// ===========================================================================

#[test]
fn wrap_preserves_custom_type_name() {
    let ty: Type = parse_quote!(MyCustomType);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    let tokens = wrapped.to_token_stream().to_string();
    assert!(tokens.contains("MyCustomType"));
}

#[test]
fn wrap_preserves_container_name_when_skipped() {
    let ty: Type = parse_quote!(Vec<MyType>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    let tokens = wrapped.to_token_stream().to_string();
    assert!(tokens.contains("Vec"));
    assert!(tokens.contains("MyType"));
}

#[test]
fn wrap_never_type() {
    let ty: Type = parse_quote!(!);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < ! >");
}
