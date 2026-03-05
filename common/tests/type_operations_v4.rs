//! Comprehensive tests for type operations in adze-common.
//!
//! Covers `wrap_leaf_type`, `is_parameterized` (local helper),
//! `try_extract_inner_type`, `filter_inner_type`, and edge cases
//! around primitive types, deeply nested generics, trait objects,
//! and TokenStream output validation.

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{self, Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Local helper — mirrors the heuristic used in the adze pipeline to detect
/// whether a type has generic (angle-bracketed) parameters.
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
// 1. wrap_leaf_type — basic wrapping (8 tests)
// ===========================================================================

#[test]
fn wrap_leaf_string() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_leaf_i32() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_leaf_bool() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_leaf_u64() {
    let ty: Type = parse_quote!(u64);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u64 >");
}

#[test]
fn wrap_leaf_f64() {
    let ty: Type = parse_quote!(f64);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn wrap_leaf_reference_str() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_leaf_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_leaf_tuple_type() {
    let ty: Type = parse_quote!((i32, u32));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (i32 , u32) >");
}

// ===========================================================================
// 2. wrap_leaf_type — skip-set containers (8 tests)
// ===========================================================================

#[test]
fn wrap_vec_string_skips_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_i32_skips_option() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_option_vec_nested_skips_both() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_vec_not_in_skip_wraps_entirely() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < String > >");
}

#[test]
fn wrap_box_string_skips_box() {
    let ty: Type = parse_quote!(Box<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&wrapped), "Box < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_result_skips_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_deeply_nested_option_vec_box() {
    let ty: Type = parse_quote!(Option<Vec<Box<u8>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec", "Box"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < Box < adze :: WithLeaf < u8 > > > >"
    );
}

#[test]
fn wrap_vec_option_skips_both() {
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < bool > > >"
    );
}

// ===========================================================================
// 3. is_parameterized — generic detection (10 tests)
// ===========================================================================

#[test]
fn parameterized_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_hashmap() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_result() {
    let ty: Type = parse_quote!(Result<String, Error>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_box_dyn_trait() {
    let ty: Type = parse_quote!(Box<dyn Display>);
    assert!(is_parameterized(&ty));
}

#[test]
fn not_parameterized_string() {
    let ty: Type = parse_quote!(String);
    assert!(!is_parameterized(&ty));
}

#[test]
fn not_parameterized_i32() {
    let ty: Type = parse_quote!(i32);
    assert!(!is_parameterized(&ty));
}

#[test]
fn not_parameterized_bool() {
    let ty: Type = parse_quote!(bool);
    assert!(!is_parameterized(&ty));
}

#[test]
fn not_parameterized_reference_str() {
    let ty: Type = parse_quote!(&str);
    assert!(!is_parameterized(&ty));
}

#[test]
fn not_parameterized_unit_type() {
    let ty: Type = parse_quote!(());
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 4. try_extract_inner_type — nested generics (8 tests)
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_u64() {
    let ty: Type = parse_quote!(Option<u64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn extract_vec_option_nested() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < String >");
}

#[test]
fn extract_through_box_skip() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_through_two_skips() {
    let ty: Type = parse_quote!(Arc<Box<Option<f64>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_no_match_returns_original() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

#[test]
fn extract_skip_but_no_inner_target() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_non_path_type_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

// ===========================================================================
// 5. filter_inner_type — filter combinations (8 tests)
// ===========================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "String");
}

#[test]
fn filter_arc_box_nested() {
    let ty: Type = parse_quote!(Arc<Box<i32>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Arc", "Box"]))),
        "i32"
    );
}

#[test]
fn filter_option_not_in_skip() {
    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Option < String >"
    );
}

#[test]
fn filter_empty_skip_set() {
    let ty: Type = parse_quote!(Vec<u8>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Vec < u8 >"
    );
}

#[test]
fn filter_triple_nesting() {
    let ty: Type = parse_quote!(Rc<Arc<Box<bool>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Rc", "Arc", "Box"]))),
        "bool"
    );
}

#[test]
fn filter_non_path_type_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "(i32 , u32)"
    );
}

#[test]
fn filter_stops_at_non_skip_container() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < String >");
}

#[test]
fn filter_single_layer_box() {
    let ty: Type = parse_quote!(Box<f32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "f32");
}

// ===========================================================================
// 6. TokenStream output — wrap_leaf_type produces valid TokenStream (5 tests)
// ===========================================================================

#[test]
fn wrap_tokenstream_is_parseable_simple() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    let tokens = wrapped.to_token_stream().to_string();
    assert!(syn::parse_str::<Type>(&tokens).is_ok());
}

#[test]
fn wrap_tokenstream_is_parseable_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    let tokens = wrapped.to_token_stream().to_string();
    assert!(syn::parse_str::<Type>(&tokens).is_ok());
}

#[test]
fn wrap_tokenstream_is_parseable_nested() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    let tokens = wrapped.to_token_stream().to_string();
    let reparsed = syn::parse_str::<Type>(&tokens).unwrap();
    assert_eq!(ty_str(&reparsed), ty_str(&wrapped));
}

#[test]
fn wrap_tokenstream_contains_with_leaf() {
    let ty: Type = parse_quote!(u8);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    let tokens = wrapped.to_token_stream().to_string();
    assert!(tokens.contains("WithLeaf"));
}

#[test]
fn wrap_tokenstream_preserves_inner_type_name() {
    let ty: Type = parse_quote!(MyCustomType);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    let tokens = wrapped.to_token_stream().to_string();
    assert!(tokens.contains("MyCustomType"));
}

// ===========================================================================
// 7. Type complexity — deeply nested, multiple parameters (6 tests)
// ===========================================================================

#[test]
fn wrap_four_levels_deep() {
    let ty: Type = parse_quote!(Option<Vec<Box<Arc<String>>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec", "Box", "Arc"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < Box < Arc < adze :: WithLeaf < String > > > > >"
    );
}

#[test]
fn extract_four_levels_through_skips() {
    let ty: Type = parse_quote!(Rc<Arc<Box<Option<u16>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Rc", "Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn filter_four_layers() {
    let ty: Type = parse_quote!(A<B<C<D<u8>>>>);
    let filtered = filter_inner_type(&ty, &skip(&["A", "B", "C", "D"]));
    assert_eq!(ty_str(&filtered), "u8");
}

#[test]
fn parameterized_deeply_nested() {
    let ty: Type = parse_quote!(Vec<Option<Box<String>>>);
    assert!(is_parameterized(&ty));
}

#[test]
fn wrap_result_with_nested_option() {
    let ty: Type = parse_quote!(Result<Option<String>, Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result", "Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < Option < adze :: WithLeaf < String > > , Vec < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn extract_from_custom_generic() {
    let ty: Type = parse_quote!(MyWrapper<Inner>);
    let (inner, ok) = try_extract_inner_type(&ty, "MyWrapper", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Inner");
}

// ===========================================================================
// 8. Primitive types — i32, u64, bool, String, &str (5 tests)
// ===========================================================================

#[test]
fn primitives_are_not_parameterized() {
    for ty_text in &["i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64"] {
        let ty = syn::parse_str::<Type>(ty_text).unwrap();
        assert!(!is_parameterized(&ty), "expected {ty_text} not parameterized");
    }
}

#[test]
fn primitive_bool_not_parameterized() {
    let ty: Type = parse_quote!(bool);
    assert!(!is_parameterized(&ty));
}

#[test]
fn primitive_string_not_parameterized() {
    let ty: Type = parse_quote!(String);
    assert!(!is_parameterized(&ty));
}

#[test]
fn wrap_all_numeric_primitives() {
    for ty_text in &["i32", "u64", "f32", "usize", "isize"] {
        let ty = syn::parse_str::<Type>(ty_text).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip(&[]));
        let result = ty_str(&wrapped);
        assert!(
            result.contains("WithLeaf"),
            "expected WithLeaf wrapper for {ty_text}, got: {result}"
        );
        assert!(
            result.contains(ty_text),
            "expected {ty_text} inside wrapper, got: {result}"
        );
    }
}

#[test]
fn extract_from_option_of_primitives() {
    for ty_text in &["Option<i32>", "Option<u64>", "Option<bool>", "Option<f64>"] {
        let ty = syn::parse_str::<Type>(ty_text).unwrap();
        let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        assert!(ok, "expected extraction from {ty_text}");
    }
}

// ===========================================================================
// 9. Edge cases — unit type, never type, trait objects, slices (7 tests)
// ===========================================================================

#[test]
fn unit_type_not_parameterized() {
    let ty: Type = parse_quote!(());
    assert!(!is_parameterized(&ty));
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < () >");
}

#[test]
fn never_type_not_parameterized() {
    let ty: Type = parse_quote!(!);
    assert!(!is_parameterized(&ty));
}

#[test]
fn wrap_never_type() {
    let ty: Type = parse_quote!(!);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < ! >");
}

#[test]
fn trait_object_not_parameterized() {
    let ty: Type = parse_quote!(dyn Display);
    assert!(!is_parameterized(&ty));
}

#[test]
fn wrap_trait_object() {
    let ty: Type = parse_quote!(dyn Display);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < dyn Display >");
}

#[test]
fn wrap_slice_type() {
    let ty: Type = parse_quote!([u8]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8] >");
}

// ===========================================================================
// 10. Round-trip: extract then wrap (3 tests)
// ===========================================================================

#[test]
fn extract_then_wrap_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Arc<i64>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "i64");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i64 >");
}

#[test]
fn extract_nested_then_check_parameterized() {
    let ty: Type = parse_quote!(Box<Vec<Option<u8>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    // Inner is Option<u8>, which is parameterized
    assert!(is_parameterized(&inner));
    assert_eq!(ty_str(&inner), "Option < u8 >");
}
