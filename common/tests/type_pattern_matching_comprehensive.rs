#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for type pattern matching in adze-common.
//!
//! Exercises `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! across Vec<T>, Option<T>, Box<T>, nested patterns, qualified paths, reference
//! types, array types, tuple types, and non-matching types.

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
// 1. Vec<T> pattern matching
// ===========================================================================

#[test]
fn extract_vec_of_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_of_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn filter_vec_in_skip_set_unwraps() {
    let ty: Type = parse_quote!(Vec<Token>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Vec"]))), "Token");
}

#[test]
fn wrap_vec_in_skip_preserves_vec_wraps_inner() {
    let ty: Type = parse_quote!(Vec<Leaf>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < Leaf > >"
    );
}

// ===========================================================================
// 2. Option<T> pattern matching
// ===========================================================================

#[test]
fn extract_option_of_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_option_of_f64() {
    let ty: Type = parse_quote!(Option<f64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn filter_option_in_skip_set_unwraps() {
    let ty: Type = parse_quote!(Option<Ident>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Option"]))), "Ident");
}

#[test]
fn wrap_option_in_skip_preserves_option_wraps_inner() {
    let ty: Type = parse_quote!(Option<Node>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < Node > >"
    );
}

// ===========================================================================
// 3. Box<T> pattern matching
// ===========================================================================

#[test]
fn extract_box_of_expr() {
    let ty: Type = parse_quote!(Box<Expr>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Expr");
}

#[test]
fn filter_box_in_skip_set_unwraps() {
    let ty: Type = parse_quote!(Box<Statement>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Statement"
    );
}

#[test]
fn filter_nested_box_box_unwraps_both() {
    let ty: Type = parse_quote!(Box<Box<Inner>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "Inner");
}

#[test]
fn extract_vec_through_box_skip() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

// ===========================================================================
// 4. Nested patterns (Vec<Option<T>>, Option<Vec<T>>, etc.)
// ===========================================================================

#[test]
fn extract_vec_of_option_extracts_option() {
    let ty: Type = parse_quote!(Vec<Option<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < u32 >");
}

#[test]
fn extract_option_through_vec_skip_not_found() {
    // Vec is not in skip set, so we cannot see through it to find Option
    let ty: Type = parse_quote!(Vec<Option<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < Option < u32 > >");
}

#[test]
fn extract_option_through_vec_skip_found() {
    let ty: Type = parse_quote!(Vec<Option<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Vec"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn filter_three_level_nesting_all_skipped() {
    let ty: Type = parse_quote!(Box<Option<Vec<Atom>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Option", "Vec"]))),
        "Atom"
    );
}

#[test]
fn wrap_nested_vec_option_both_skipped() {
    let ty: Type = parse_quote!(Vec<Option<Leaf>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Option"]))),
        "Vec < Option < adze :: WithLeaf < Leaf > > >"
    );
}

// ===========================================================================
// 5. Non-matching types
// ===========================================================================

#[test]
fn extract_vec_from_hashmap_fails() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

#[test]
fn extract_option_from_result_fails() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Result < String , Error >");
}

#[test]
fn extract_vec_from_plain_type_fails() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn filter_non_matching_generic_unchanged() {
    let ty: Type = parse_quote!(BTreeMap<String, u32>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Option"]))),
        "BTreeMap < String , u32 >"
    );
}

// ===========================================================================
// 6. Qualified paths (std::vec::Vec<T>, etc.)
// ===========================================================================

#[test]
fn extract_qualified_vec_matches_last_segment() {
    // try_extract_inner_type checks the *last* path segment
    let ty: Type = parse_quote!(std::vec::Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn filter_qualified_option_matches_last_segment() {
    let ty: Type = parse_quote!(std::option::Option<u64>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Option"]))), "u64");
}

#[test]
fn wrap_qualified_vec_in_skip_preserves_path() {
    let ty: Type = parse_quote!(std::vec::Vec<Item>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "std :: vec :: Vec < adze :: WithLeaf < Item > >"
    );
}

// ===========================================================================
// 7. Type aliases (custom names that are NOT the target)
// ===========================================================================

#[test]
fn extract_type_alias_not_recognized() {
    // A type alias like `MyVec<T>` won't match "Vec"
    let ty: Type = parse_quote!(MyVec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "MyVec < String >");
}

#[test]
fn filter_type_alias_not_in_skip_unchanged() {
    let ty: Type = parse_quote!(Wrapper<Payload>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Wrapper < Payload >"
    );
}

// ===========================================================================
// 8. Reference types
// ===========================================================================

#[test]
fn extract_from_reference_type_returns_unchanged() {
    let ty: Type = parse_quote!(&Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& Vec < u8 >");
}

#[test]
fn filter_reference_type_returns_unchanged() {
    let ty: Type = parse_quote!(&mut String);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "& mut String"
    );
}

#[test]
fn wrap_reference_type_wraps_entire_ref() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < & str >"
    );
}

// ===========================================================================
// 9. Array types
// ===========================================================================

#[test]
fn extract_from_array_type_returns_unchanged() {
    let ty: Type = parse_quote!([u8; 16]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "[u8 ; 16]");
}

#[test]
fn filter_array_type_returns_unchanged() {
    let ty: Type = parse_quote!([i32; 4]);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "[i32 ; 4]"
    );
}

#[test]
fn wrap_array_type_wraps_entire_array() {
    let ty: Type = parse_quote!([f64; 3]);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < [f64 ; 3] >"
    );
}

// ===========================================================================
// 10. Tuple types
// ===========================================================================

#[test]
fn extract_from_tuple_type_returns_unchanged() {
    let ty: Type = parse_quote!((i32, String));
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(i32 , String)");
}

#[test]
fn filter_tuple_type_returns_unchanged() {
    let ty: Type = parse_quote!((bool, u8, f32));
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Option"]))),
        "(bool , u8 , f32)"
    );
}

#[test]
fn wrap_tuple_type_wraps_entire_tuple() {
    let ty: Type = parse_quote!((u32, u64));
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < (u32 , u64) >"
    );
}
