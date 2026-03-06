//! Comprehensive tests for type extraction and manipulation utilities
//! in adze-common: `try_extract_inner_type`, `filter_inner_type`,
//! `wrap_leaf_type`, and a local `is_parameterized` helper.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use syn::{self, Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

/// Local helper: returns `true` when the type is a path whose last segment
/// carries angle-bracketed generic arguments.
fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

// ===========================================================================
// 1. try_extract_inner_type — Option<T> extraction
// ===========================================================================

#[test]
fn extract_option_string_succeeds() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_i32_succeeds() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_option_bool_succeeds() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_option_through_box_skip() {
    let ty: Type = parse_quote!(Box<Option<u64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn extract_option_through_arc_skip() {
    let ty: Type = parse_quote!(Arc<Option<f32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn extract_option_through_two_skips() {
    let ty: Type = parse_quote!(Arc<Box<Option<Token>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Token");
}

// ===========================================================================
// 2. try_extract_inner_type — Vec<T> extraction
// ===========================================================================

#[test]
fn extract_vec_string_succeeds() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_u8_succeeds() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_vec_through_box_skip() {
    let ty: Type = parse_quote!(Box<Vec<Stmt>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Stmt");
}

#[test]
fn extract_vec_preserves_complex_inner() {
    let ty: Type = parse_quote!(Vec<HashMap<String, i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

// ===========================================================================
// 3. try_extract_inner_type — Box<T> extraction
// ===========================================================================

#[test]
fn extract_box_directly() {
    let ty: Type = parse_quote!(Box<Expr>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Expr");
}

#[test]
fn extract_box_through_arc_skip() {
    let ty: Type = parse_quote!(Arc<Box<Node>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Node");
}

// ===========================================================================
// 4. try_extract_inner_type — non-matching / miss cases
// ===========================================================================

#[test]
fn extract_miss_simple_type() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_miss_wrong_generic() {
    let ty: Type = parse_quote!(Result<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Result < String >");
}

#[test]
fn extract_miss_skip_but_target_absent() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_miss_reference_type() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn extract_miss_tuple_type() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(i32 , u32)");
}

#[test]
fn extract_miss_unit_type() {
    let ty: Type = parse_quote!(());
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "()");
}

#[test]
fn extract_miss_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "[u8 ; 4]");
}

// ===========================================================================
// 5. filter_inner_type — filtering by outer type name
// ===========================================================================

#[test]
fn filter_box_strips_to_inner() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "String");
}

#[test]
fn filter_arc_strips_to_inner() {
    let ty: Type = parse_quote!(Arc<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Arc"]))), "i32");
}

#[test]
fn filter_nested_box_arc_strips_both() {
    let ty: Type = parse_quote!(Box<Arc<Token>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "Token"
    );
}

#[test]
fn filter_nested_triple_strips_all() {
    let ty: Type = parse_quote!(Arc<Box<Rc<Leaf>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Arc", "Box", "Rc"]))),
        "Leaf"
    );
}

#[test]
fn filter_stops_at_non_skip_type() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < String >"
    );
}

#[test]
fn filter_empty_skip_returns_original() {
    let ty: Type = parse_quote!(Box<Expr>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&[]))), "Box < Expr >");
}

#[test]
fn filter_simple_type_unchanged() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "i32");
}

#[test]
fn filter_non_path_tuple_unchanged() {
    let ty: Type = parse_quote!((bool, u8));
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "(bool , u8)"
    );
}

#[test]
fn filter_non_path_reference_unchanged() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "& str");
}

#[test]
fn filter_option_in_skip_strips() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Option"]))),
        "Vec < u8 >"
    );
}

#[test]
fn filter_option_vec_both_skipped() {
    let ty: Type = parse_quote!(Option<Vec<f64>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Option", "Vec"]))),
        "f64"
    );
}

// ===========================================================================
// 6. wrap_leaf_type — wrapping plain types
// ===========================================================================

#[test]
fn wrap_plain_string() {
    let ty: Type = parse_quote!(String);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_plain_i32() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_plain_bool() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < bool >"
    );
}

#[test]
fn wrap_plain_custom_type() {
    let ty: Type = parse_quote!(MyNode);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < MyNode >"
    );
}

// ===========================================================================
// 7. wrap_leaf_type — wrapping generic types (skip set)
// ===========================================================================

#[test]
fn wrap_vec_wraps_inner_element() {
    let ty: Type = parse_quote!(Vec<Token>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < Token > >"
    );
}

#[test]
fn wrap_option_wraps_inner_element() {
    let ty: Type = parse_quote!(Option<u32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < u32 > >"
    );
}

#[test]
fn wrap_nested_option_vec_wraps_leaf() {
    let ty: Type = parse_quote!(Option<Vec<Node>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option", "Vec"]))),
        "Option < Vec < adze :: WithLeaf < Node > > >"
    );
}

#[test]
fn wrap_triple_nesting_wraps_leaf() {
    let ty: Type = parse_quote!(Box<Option<Vec<Leaf>>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Box", "Option", "Vec"]))),
        "Box < Option < Vec < adze :: WithLeaf < Leaf > > > >"
    );
}

#[test]
fn wrap_result_wraps_both_args() {
    let ty: Type = parse_quote!(Result<Good, Bad>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Result"]))),
        "Result < adze :: WithLeaf < Good > , adze :: WithLeaf < Bad > >"
    );
}

#[test]
fn wrap_not_in_skip_wraps_entire_generic() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

// ===========================================================================
// 8. wrap_leaf_type — non-path types
// ===========================================================================

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, f64));
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < (i32 , f64) >"
    );
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < () >"
    );
}

#[test]
fn wrap_never_type() {
    let ty: Type = parse_quote!(!);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < ! >"
    );
}

// ===========================================================================
// 9. is_parameterized — simple types (not parameterized)
// ===========================================================================

#[test]
fn param_i32_not_parameterized() {
    let ty: Type = parse_quote!(i32);
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_string_not_parameterized() {
    let ty: Type = parse_quote!(String);
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_bool_not_parameterized() {
    let ty: Type = parse_quote!(bool);
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_usize_not_parameterized() {
    let ty: Type = parse_quote!(usize);
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_custom_ident_not_parameterized() {
    let ty: Type = parse_quote!(MyStruct);
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 10. is_parameterized — generic types (parameterized)
// ===========================================================================

#[test]
fn param_option_is_parameterized() {
    let ty: Type = parse_quote!(Option<i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn param_vec_is_parameterized() {
    let ty: Type = parse_quote!(Vec<String>);
    assert!(is_parameterized(&ty));
}

#[test]
fn param_box_is_parameterized() {
    let ty: Type = parse_quote!(Box<u8>);
    assert!(is_parameterized(&ty));
}

#[test]
fn param_hashmap_is_parameterized() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn param_result_is_parameterized() {
    let ty: Type = parse_quote!(Result<(), String>);
    assert!(is_parameterized(&ty));
}

// ===========================================================================
// 11. is_parameterized — nested generics
// ===========================================================================

#[test]
fn param_vec_option_is_parameterized() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    assert!(is_parameterized(&ty));
}

#[test]
fn param_option_box_is_parameterized() {
    let ty: Type = parse_quote!(Option<Box<String>>);
    assert!(is_parameterized(&ty));
}

#[test]
fn param_hashmap_vec_is_parameterized() {
    let ty: Type = parse_quote!(HashMap<String, Vec<u32>>);
    assert!(is_parameterized(&ty));
}

// ===========================================================================
// 12. is_parameterized — non-path types (not parameterized)
// ===========================================================================

#[test]
fn param_reference_not_parameterized() {
    let ty: Type = parse_quote!(&str);
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_tuple_not_parameterized() {
    let ty: Type = parse_quote!((i32, u32));
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_array_not_parameterized() {
    let ty: Type = parse_quote!([u8; 4]);
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_unit_not_parameterized() {
    let ty: Type = parse_quote!(());
    assert!(!is_parameterized(&ty));
}

#[test]
fn param_ref_to_generic_not_parameterized() {
    // Outer type is a reference, so is_parameterized is false.
    let ty: Type = parse_quote!(&Vec<u8>);
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 13. Type parsing edge cases — paths
// ===========================================================================

#[test]
fn parse_qualified_path_filter() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "std :: collections :: HashMap < String , i32 >"
    );
}

#[test]
fn parse_qualified_vec_extract() {
    let ty: Type = parse_quote!(std::vec::Vec<u16>);
    // The last segment is "Vec", so extraction works.
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn parse_crate_path_not_parameterized() {
    let ty: Type = parse_quote!(crate::ast::Node);
    assert!(!is_parameterized(&ty));
}

#[test]
fn parse_crate_generic_is_parameterized() {
    let ty: Type = parse_quote!(crate::Wrapper<u64>);
    assert!(is_parameterized(&ty));
}

// ===========================================================================
// 14. Nested generics — Vec<Option<T>>, Option<Box<T>>
// ===========================================================================

#[test]
fn nested_extract_option_inside_vec_not_found_without_skip() {
    // Looking for Option inside Vec<Option<i32>>, but Vec is not in skip set.
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < Option < i32 > >");
}

#[test]
fn nested_extract_option_inside_vec_with_skip() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Vec"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn nested_extract_vec_inside_option_with_skip() {
    let ty: Type = parse_quote!(Option<Vec<Stmt>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Stmt");
}

#[test]
fn nested_filter_option_box_both_skipped() {
    let ty: Type = parse_quote!(Option<Box<Leaf>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Option", "Box"]))),
        "Leaf"
    );
}

#[test]
fn nested_wrap_vec_option_both_skipped() {
    let ty: Type = parse_quote!(Vec<Option<Ident>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Option"]))),
        "Vec < Option < adze :: WithLeaf < Ident > > >"
    );
}

// ===========================================================================
// 15. Non-generic types — i32, String, custom types
// ===========================================================================

#[test]
fn non_generic_filter_returns_self() {
    let ty: Type = parse_quote!(Identifier);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "Identifier"
    );
}

#[test]
fn non_generic_extract_returns_false() {
    let ty: Type = parse_quote!(f64);
    let (_inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
}

#[test]
fn non_generic_wrap_wraps_entirely() {
    let ty: Type = parse_quote!(usize);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "adze :: WithLeaf < usize >"
    );
}

// ===========================================================================
// 16. Complex type expressions — Result<T, E>, HashMap<K, V>
// ===========================================================================

#[test]
fn complex_result_extract_directly() {
    let ty: Type = parse_quote!(Result<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn complex_hashmap_not_extracted_as_vec() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

#[test]
fn complex_btreemap_wrap_not_in_skip() {
    let ty: Type = parse_quote!(BTreeMap<u32, Vec<u8>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < BTreeMap < u32 , Vec < u8 > > >"
    );
}

#[test]
fn complex_result_wrap_in_skip_wraps_args() {
    let ty: Type = parse_quote!(Result<Parsed, ParseError>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Result"]))),
        "Result < adze :: WithLeaf < Parsed > , adze :: WithLeaf < ParseError > >"
    );
}

#[test]
fn complex_vec_of_result_partial_skip() {
    // Vec is in skip, Result is not — Result<...> gets wrapped as a whole.
    let ty: Type = parse_quote!(Vec<Result<String, Error>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < Result < String , Error > > >"
    );
}

// ===========================================================================
// 17. syn::parse_str edge cases
// ===========================================================================

#[test]
fn parse_str_vec_string() {
    let ty: Type = syn::parse_str("Vec<String>").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn parse_str_option_u32() {
    let ty: Type = syn::parse_str("Option<u32>").unwrap();
    assert!(is_parameterized(&ty));
}

#[test]
fn parse_str_plain_i64() {
    let ty: Type = syn::parse_str("i64").unwrap();
    assert!(!is_parameterized(&ty));
}

#[test]
fn parse_str_nested_option_vec() {
    let ty: Type = syn::parse_str("Option<Vec<bool>>").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}
