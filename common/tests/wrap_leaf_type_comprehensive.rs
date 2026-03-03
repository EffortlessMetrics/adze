#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for `wrap_leaf_type` in adze-common.
//!
//! Tests cover: String wrapping, primitive type wrapping (i32, u32, f64),
//! bool wrapping, custom type wrapping, Option<T> wrapping, Vec<T> wrapping,
//! Box<T> wrapping, nested wrappers, idempotent behavior, and bulk type lists.

use std::collections::HashSet;

use adze_common::wrap_leaf_type;
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
// 1. String wrapping
// ===========================================================================

#[test]
fn wrap_string_no_skip() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_str_reference_no_skip() {
    // Non-path types are also wrapped
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_string_with_unrelated_skip() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

// ===========================================================================
// 2. Primitive type wrapping (i32, u32, f64)
// ===========================================================================

#[test]
fn wrap_i32() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_u32() {
    let ty: Type = parse_quote!(u32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u32 >");
}

#[test]
fn wrap_f64() {
    let ty: Type = parse_quote!(f64);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn wrap_i64_with_skip() {
    let ty: Type = parse_quote!(i64);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i64 >");
}

// ===========================================================================
// 3. Bool wrapping
// ===========================================================================

#[test]
fn wrap_bool_no_skip() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_bool_with_skip() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Box"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < bool >");
}

// ===========================================================================
// 4. Custom type wrapping
// ===========================================================================

#[test]
fn wrap_custom_type() {
    let ty: Type = parse_quote!(MyStruct);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < MyStruct >");
}

#[test]
fn wrap_qualified_custom_type() {
    let ty: Type = parse_quote!(my_module::MyStruct);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < my_module :: MyStruct >"
    );
}

#[test]
fn wrap_custom_type_not_in_skip() {
    let ty: Type = parse_quote!(Identifier);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Identifier >");
}

// ===========================================================================
// 5. Option<T> wrapping
// ===========================================================================

#[test]
fn wrap_option_string_skip_option() {
    let ty: Type = parse_quote!(Option<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_option_i32_skip_option() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_option_not_in_skip_wraps_whole() {
    // When Option is NOT in skip set, the entire Option<T> is wrapped
    let ty: Type = parse_quote!(Option<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < Option < String > >"
    );
}

// ===========================================================================
// 6. Vec<T> wrapping
// ===========================================================================

#[test]
fn wrap_vec_string_skip_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_vec_u32_skip_vec() {
    let ty: Type = parse_quote!(Vec<u32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < u32 > >");
}

#[test]
fn wrap_vec_not_in_skip_wraps_whole() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < i32 > >");
}

// ===========================================================================
// 7. Box<T> wrapping
// ===========================================================================

#[test]
fn wrap_box_string_skip_box() {
    let ty: Type = parse_quote!(Box<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    assert_eq!(
        ty_str(&wrapped),
        "Box < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_box_f64_skip_box() {
    let ty: Type = parse_quote!(Box<f64>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&wrapped), "Box < adze :: WithLeaf < f64 > >");
}

#[test]
fn wrap_box_not_in_skip_wraps_whole() {
    let ty: Type = parse_quote!(Box<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Box < bool > >");
}

// ===========================================================================
// 8. Nested wrappers
// ===========================================================================

#[test]
fn wrap_option_vec_string_skip_both() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_vec_option_i32_skip_both() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_option_box_custom_skip_all() {
    let ty: Type = parse_quote!(Option<Box<MyNode>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Box"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Box < adze :: WithLeaf < MyNode > > >"
    );
}

#[test]
fn wrap_vec_box_option_skip_all_three() {
    let ty: Type = parse_quote!(Vec<Box<Option<u8>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Box", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Box < Option < adze :: WithLeaf < u8 > > > >"
    );
}

#[test]
fn wrap_nested_skip_only_outer() {
    // Only Option is in skip, so Vec<String> is wrapped as a whole leaf
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < adze :: WithLeaf < Vec < String > > >"
    );
}

// ===========================================================================
// 9. Idempotent behavior
// ===========================================================================

#[test]
fn wrap_already_wrapped_type_wraps_again() {
    // Wrapping is NOT idempotent — applying it twice nests WithLeaf
    let ty: Type = parse_quote!(String);
    let once = wrap_leaf_type(&ty, &skip(&[]));
    let twice = wrap_leaf_type(&once, &skip(&[]));
    assert_eq!(
        ty_str(&twice),
        "adze :: WithLeaf < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_vec_twice_skip_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let once = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&once),
        "Vec < adze :: WithLeaf < String > >"
    );
    // Second wrap: Vec is still skipped, inner is wrapped again
    let twice = wrap_leaf_type(&once, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&twice),
        "Vec < adze :: WithLeaf < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_option_twice_skip_option() {
    let ty: Type = parse_quote!(Option<bool>);
    let once = wrap_leaf_type(&ty, &skip(&["Option"]));
    let twice = wrap_leaf_type(&once, &skip(&["Option"]));
    assert_eq!(
        ty_str(&twice),
        "Option < adze :: WithLeaf < adze :: WithLeaf < bool > > >"
    );
}

// ===========================================================================
// 10. Large number of types
// ===========================================================================

#[test]
fn wrap_many_primitive_types() {
    let types: Vec<Type> = vec![
        parse_quote!(i8),
        parse_quote!(i16),
        parse_quote!(i32),
        parse_quote!(i64),
        parse_quote!(u8),
        parse_quote!(u16),
        parse_quote!(u32),
        parse_quote!(u64),
        parse_quote!(f32),
        parse_quote!(f64),
        parse_quote!(bool),
        parse_quote!(char),
        parse_quote!(usize),
        parse_quote!(isize),
    ];
    let empty_skip = skip(&[]);
    for i in 0..types.len() {
        let wrapped = wrap_leaf_type(&types[i], &empty_skip);
        let s = ty_str(&wrapped);
        assert!(
            s.starts_with("adze :: WithLeaf <"),
            "type at index {} was not wrapped: {}",
            i,
            s,
        );
        assert!(s.ends_with('>'), "type at index {} missing closing: {}", i, s);
    }
}

#[test]
fn wrap_many_container_types_skip_container() {
    let skip_set = skip(&["Vec", "Option", "Box"]);
    let types: Vec<Type> = vec![
        parse_quote!(Vec<i32>),
        parse_quote!(Vec<String>),
        parse_quote!(Option<u64>),
        parse_quote!(Option<bool>),
        parse_quote!(Box<f32>),
        parse_quote!(Box<MyType>),
        parse_quote!(Vec<Option<i32>>),
        parse_quote!(Option<Vec<String>>),
        parse_quote!(Box<Option<u8>>),
        parse_quote!(Vec<Box<Option<char>>>),
    ];
    for i in 0..types.len() {
        let wrapped = wrap_leaf_type(&types[i], &skip_set);
        let s = ty_str(&wrapped);
        // Outer container should be preserved, not wrapped
        assert!(
            !s.starts_with("adze :: WithLeaf <"),
            "container at index {} should not be leaf-wrapped: {}",
            i,
            s,
        );
        // But the innermost type should be wrapped
        assert!(
            s.contains("adze :: WithLeaf"),
            "inner type at index {} should be wrapped: {}",
            i,
            s,
        );
    }
}

#[test]
fn wrap_non_path_tuple_type() {
    // Tuple types are not Type::Path, so they get wrapped directly
    let ty: Type = parse_quote!((i32, String));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < (i32 , String) >"
    );
}
