//! Comprehensive v5 tests for adze-common expansion utilities.
//!
//! Covers:
//! 1. Type extraction with nested generics (10 tests)
//! 2. Filter with multiple skip_over types (8 tests)
//! 3. Wrap patterns (8 tests)
//! 4. Parameterized type detection via extraction probing (8 tests)
//! 5. Complex real-world type patterns (8 tests)
//! 6. Determinism (5 tests)
//! 7. Edge cases (8 tests)

use adze_common::*;
use quote::ToTokens;
use std::collections::HashSet;
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

/// Checks whether a type has angle-bracketed generic arguments at the top level.
fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

// ===========================================================================
// 1. Type extraction with nested generics (10 tests)
// ===========================================================================

#[test]
fn extract_option_of_vec_of_string() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn extract_vec_inside_box_skip_box() {
    let ty: Type = parse_quote!(Box<Vec<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn extract_option_inside_arc_skip_arc() {
    let ty: Type = parse_quote!(Arc<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_through_two_skip_layers() {
    let ty: Type = parse_quote!(Box<Arc<Vec<f64>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_fails_when_target_absent() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < Arc < String > >");
}

#[test]
fn extract_option_of_option() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn extract_skip_does_not_match_target_directly() {
    // Rc is in skip set, target is Vec, but inner is String — no Vec found
    let ty: Type = parse_quote!(Rc<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Rc"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Rc < String >");
}

#[test]
fn extract_hashmap_is_not_extracted_as_vec() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
}

#[test]
fn extract_nested_option_vec_box() {
    let ty: Type = parse_quote!(Option<Vec<Box<Node>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < Box < Node > >");
}

#[test]
fn extract_vec_deeply_nested_in_three_skips() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Vec<u8>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc", "Rc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

// ===========================================================================
// 2. Filter with multiple skip_over types (8 tests)
// ===========================================================================

#[test]
fn filter_single_box() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "String");
}

#[test]
fn filter_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<u64>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "u64"
    );
}

#[test]
fn filter_three_layers() {
    let ty: Type = parse_quote!(Box<Arc<Rc<bool>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]))),
        "bool"
    );
}

#[test]
fn filter_stops_at_non_skip_type() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    // Vec is not in skip set, so filtering stops there
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < String >"
    );
}

#[test]
fn filter_no_match_returns_original() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "Vec < i32 >"
    );
}

#[test]
fn filter_empty_skip_set_returns_original() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Box < String >"
    );
}

#[test]
fn filter_non_path_type_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "& str");
}

#[test]
fn filter_tuple_type_returns_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "(i32 , u32)"
    );
}

// ===========================================================================
// 3. Wrap patterns (8 tests)
// ===========================================================================

#[test]
fn wrap_plain_type() {
    let ty: Type = parse_quote!(String);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_vec_wraps_inner() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_option_wraps_inner() {
    let ty: Type = parse_quote!(Option<bool>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < bool > >"
    );
}

#[test]
fn wrap_nested_vec_option() {
    let ty: Type = parse_quote!(Vec<Option<Node>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Option"]))),
        "Vec < Option < adze :: WithLeaf < Node > > >"
    );
}

#[test]
fn wrap_non_skip_generic_wraps_whole_type() {
    // HashMap is not in skip set, so the whole thing gets wrapped
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

#[test]
fn wrap_result_both_args_when_skipped() {
    let ty: Type = parse_quote!(Result<String, Error>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Result"]))),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < Error > >"
    );
}

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

// ===========================================================================
// 4. Parameterized type detection via extraction probing (8 tests)
// ===========================================================================

#[test]
fn parameterized_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_plain_string() {
    let ty: Type = parse_quote!(String);
    assert!(!is_parameterized(&ty));
}

#[test]
fn parameterized_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_reference_not_detected() {
    let ty: Type = parse_quote!(&str);
    assert!(!is_parameterized(&ty));
}

#[test]
fn parameterized_tuple_not_detected() {
    let ty: Type = parse_quote!((i32, u32));
    assert!(!is_parameterized(&ty));
}

#[test]
fn parameterized_nested_generics() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_hashmap() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert!(is_parameterized(&ty));
}

#[test]
fn parameterized_unit_type() {
    let ty: Type = parse_quote!(());
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 5. Complex real-world type patterns (8 tests)
// ===========================================================================

#[test]
fn real_world_optional_vec_boxed_node() {
    // Typical grammar field: Option<Vec<Box<AstNode>>>
    let ty: Type = parse_quote!(Option<Vec<Box<AstNode>>>);

    let (after_opt, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);

    let (after_vec, ok) = try_extract_inner_type(&after_opt, "Vec", &skip(&[]));
    assert!(ok);

    let filtered = filter_inner_type(&after_vec, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "AstNode");

    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < AstNode >");
}

#[test]
fn real_world_vec_of_option() {
    let ty: Type = parse_quote!(Vec<Option<Token>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < Token >");
}

#[test]
fn real_world_arc_mutex_inner() {
    let ty: Type = parse_quote!(Arc<Mutex<ParseState>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Mutex", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "ParseState");
}

#[test]
fn real_world_box_dyn_trait_not_extracted() {
    // Box<dyn Fn()> — not a simple generic extraction target
    let ty: Type = parse_quote!(Box<dyn Fn()>);
    let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    // dyn Fn() is not a Type::Path with Vec, so skip through Box but fail to find Vec
    assert!(!ok);
}

#[test]
fn real_world_filter_then_wrap_pipeline() {
    let ty: Type = parse_quote!(Box<Arc<Expr>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "Expr");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Expr >");
}

#[test]
fn real_world_wrap_vec_of_option_of_node() {
    let ty: Type = parse_quote!(Vec<Option<Node>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < Node > > >"
    );
}

#[test]
fn real_world_extract_and_wrap_result() {
    let ty: Type = parse_quote!(Result<Vec<Token>, ParseError>);
    // Result is not Vec, but we can wrap it if Result is in skip
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < Vec < adze :: WithLeaf < Token > > , adze :: WithLeaf < ParseError > >"
    );
}

#[test]
fn real_world_cow_str_not_filtered() {
    let ty: Type = parse_quote!(Cow<str>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "Cow < str >");
}

// ===========================================================================
// 6. Determinism (5 tests)
// ===========================================================================

#[test]
fn determinism_extract_repeated() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip_set = skip(&[]);
    let results: Vec<_> = (0..10)
        .map(|_| {
            let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set);
            (ty_str(&inner), ok)
        })
        .collect();
    assert!(results.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn determinism_filter_repeated() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let skip_set = skip(&["Box", "Arc"]);
    let results: Vec<_> = (0..10)
        .map(|_| ty_str(&filter_inner_type(&ty, &skip_set)))
        .collect();
    assert!(results.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn determinism_wrap_repeated() {
    let ty: Type = parse_quote!(Vec<Option<Node>>);
    let skip_set = skip(&["Vec", "Option"]);
    let results: Vec<_> = (0..10)
        .map(|_| ty_str(&wrap_leaf_type(&ty, &skip_set)))
        .collect();
    assert!(results.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn determinism_extract_with_skip_repeated() {
    let ty: Type = parse_quote!(Arc<Box<Vec<u8>>>);
    let skip_set = skip(&["Arc", "Box"]);
    let results: Vec<_> = (0..10)
        .map(|_| {
            let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set);
            (ty_str(&inner), ok)
        })
        .collect();
    assert!(results.windows(2).all(|w| w[0] == w[1]));
    assert!(results[0].1);
    assert_eq!(results[0].0, "u8");
}

#[test]
fn determinism_full_pipeline_repeated() {
    let ty: Type = parse_quote!(Option<Box<Vec<Expr>>>);
    let results: Vec<_> = (0..10)
        .map(|_| {
            let (a, _) = try_extract_inner_type(&ty, "Option", &skip(&[]));
            let b = filter_inner_type(&a, &skip(&["Box"]));
            let c = wrap_leaf_type(&b, &skip(&["Vec"]));
            ty_str(&c)
        })
        .collect();
    assert!(results.windows(2).all(|w| w[0] == w[1]));
}

// ===========================================================================
// 7. Edge cases (8 tests)
// ===========================================================================

#[test]
fn edge_extract_plain_type_not_found() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn edge_filter_plain_type_unchanged() {
    let ty: Type = parse_quote!(u64);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "u64");
}

#[test]
fn edge_wrap_already_qualified_path() {
    let ty: Type = parse_quote!(std::string::String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < std :: string :: String >"
    );
}

#[test]
fn edge_extract_from_slice_type() {
    let ty: Type = parse_quote!([u8]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "[u8]");
}

#[test]
fn edge_filter_never_type() {
    let ty: Type = parse_quote!(!);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "!");
}

#[test]
fn edge_wrap_never_type() {
    let ty: Type = parse_quote!(!);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < ! >"
    );
}

#[test]
fn edge_extract_target_same_as_skip() {
    // When the target is also in skip_over, the target match takes priority
    // because the target check happens first in try_extract_inner_type.
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Vec"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn edge_wrap_primitive_i32() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < i32 >"
    );
}
