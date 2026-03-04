#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for type visiting/inspection in adze-common.
//!
//! Validates that `try_extract_inner_type`, `filter_inner_type`, and
//! `wrap_leaf_type` correctly inspect and transform simple types, generic types,
//! nested generics, reference types, tuple types, array types, path types with
//! segments, skip-set interactions, and sequences of multiple types.

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
// 1. Visit simple types (String, i32, bool, char, f64, usize)
// ===========================================================================

#[test]
fn visit_simple_string_extract_returns_unchanged() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn visit_simple_i32_filter_returns_unchanged() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "i32");
}

#[test]
fn visit_simple_char_wrap_produces_with_leaf() {
    let ty: Type = parse_quote!(char);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < char >"
    );
}

#[test]
fn visit_simple_usize_all_three_functions() {
    let ty: Type = parse_quote!(usize);
    let empty = skip(&[]);
    // extract: no match
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &empty);
    assert!(!ok);
    assert_eq!(ty_str(&inner), "usize");
    // filter: unchanged
    assert_eq!(ty_str(&filter_inner_type(&ty, &empty)), "usize");
    // wrap: wrapped
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &empty)),
        "adze :: WithLeaf < usize >"
    );
}

// ===========================================================================
// 2. Visit generic types (Option<T>, Vec<T>)
// ===========================================================================

#[test]
fn visit_option_i64_extract_inner() {
    let ty: Type = parse_quote!(Option<i64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i64");
}

#[test]
fn visit_vec_bool_filter_strips_when_in_skip() {
    let ty: Type = parse_quote!(Vec<bool>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Vec"]))), "bool");
}

#[test]
fn visit_option_char_wrap_preserves_option() {
    let ty: Type = parse_quote!(Option<char>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < char > >"
    );
}

#[test]
fn visit_generic_not_in_skip_wrap_wraps_whole() {
    let ty: Type = parse_quote!(Rc<Node>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "adze :: WithLeaf < Rc < Node > >"
    );
}

// ===========================================================================
// 3. Visit nested generics
// ===========================================================================

#[test]
fn visit_nested_option_vec_extract_through_skip() {
    let ty: Type = parse_quote!(Option<Vec<u16>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn visit_nested_box_arc_option_extract_deep() {
    let ty: Type = parse_quote!(Box<Arc<Option<Token>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Token");
}

#[test]
fn visit_nested_filter_peels_all_skip_layers() {
    let ty: Type = parse_quote!(Arc<Box<Rc<Leaf>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Arc", "Box", "Rc"]))),
        "Leaf"
    );
}

#[test]
fn visit_nested_wrap_preserves_all_skip_layers() {
    let ty: Type = parse_quote!(Box<Option<Vec<Atom>>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Box", "Option", "Vec"]))),
        "Box < Option < Vec < adze :: WithLeaf < Atom > > > >"
    );
}

// ===========================================================================
// 4. Visit reference types
// ===========================================================================

#[test]
fn visit_shared_ref_extract_returns_unchanged() {
    let ty: Type = parse_quote!(&String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& String");
}

#[test]
fn visit_mut_ref_filter_returns_unchanged() {
    let ty: Type = parse_quote!(&mut Vec<u8>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "& mut Vec < u8 >"
    );
}

#[test]
fn visit_ref_wrap_wraps_entire_ref() {
    let ty: Type = parse_quote!(&'static str);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < & 'static str >"
    );
}

// ===========================================================================
// 5. Visit tuple types
// ===========================================================================

#[test]
fn visit_pair_tuple_extract_returns_unchanged() {
    let ty: Type = parse_quote!((u32, u64));
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(u32 , u64)");
}

#[test]
fn visit_triple_tuple_filter_returns_unchanged() {
    let ty: Type = parse_quote!((String, i32, bool));
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Option"]))),
        "(String , i32 , bool)"
    );
}

#[test]
fn visit_unit_tuple_wrap() {
    let ty: Type = parse_quote!(());
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < () >"
    );
}

// ===========================================================================
// 6. Visit array types
// ===========================================================================

#[test]
fn visit_array_u8_extract_returns_unchanged() {
    let ty: Type = parse_quote!([u8; 32]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "[u8 ; 32]");
}

#[test]
fn visit_array_filter_returns_unchanged() {
    let ty: Type = parse_quote!([f32; 3]);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "[f32 ; 3]"
    );
}

#[test]
fn visit_array_wrap_wraps_entire_array() {
    let ty: Type = parse_quote!([i64; 2]);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < [i64 ; 2] >"
    );
}

// ===========================================================================
// 7. Visit path types with segments
// ===========================================================================

#[test]
fn visit_qualified_path_extract_matches_last_segment() {
    let ty: Type = parse_quote!(std::vec::Vec<f32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn visit_qualified_path_filter_matches_last_segment() {
    let ty: Type = parse_quote!(std::option::Option<u128>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Option"]))), "u128");
}

#[test]
fn visit_qualified_path_wrap_preserves_full_path() {
    let ty: Type = parse_quote!(std::collections::BTreeSet<Key>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < std :: collections :: BTreeSet < Key > >"
    );
}

#[test]
fn visit_deep_qualified_path_not_in_skip() {
    let ty: Type = parse_quote!(my::module::Custom<Inner>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "my :: module :: Custom < Inner >");
}

// ===========================================================================
// 8. Type visitor with skip set interaction
// ===========================================================================

#[test]
fn skip_set_empty_extract_only_matches_target_directly() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    // With empty skip set, Box is not skipped, so Option inside is unreachable
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < Option < String > >");
}

#[test]
fn skip_set_enables_deep_extraction() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    // With Box in skip set, we can reach through it to find Option
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn skip_set_filter_stops_at_non_skip_boundary() {
    let ty: Type = parse_quote!(Box<Vec<Arc<Leaf>>>);
    // Only Box in skip: peel Box, stop at Vec
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < Arc < Leaf > >"
    );
}

#[test]
fn skip_set_wrap_only_descends_into_skip_types() {
    let ty: Type = parse_quote!(Option<HashMap<K, V>>);
    // Option is in skip, HashMap is not — so HashMap gets wrapped as leaf
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option"]))),
        "Option < adze :: WithLeaf < HashMap < K , V > > >"
    );
}

#[test]
fn skip_set_large_set_peels_many_layers() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Cell<Payload>>>>);
    assert_eq!(
        ty_str(&filter_inner_type(
            &ty,
            &skip(&["Box", "Arc", "Rc", "Cell"])
        )),
        "Payload"
    );
}

// ===========================================================================
// 9. Multiple types in sequence
// ===========================================================================

#[test]
fn visit_sequence_extract_different_targets() {
    let types: Vec<Type> = vec![
        parse_quote!(Vec<i32>),
        parse_quote!(Option<String>),
        parse_quote!(Box<bool>),
    ];
    let targets = ["Vec", "Option", "Box"];
    let expected_inner = ["i32", "String", "bool"];

    for i in 0..types.len() {
        let (inner, ok) = try_extract_inner_type(&types[i], targets[i], &skip(&[]));
        assert!(ok, "extraction failed for target {}", targets[i]);
        assert_eq!(ty_str(&inner), expected_inner[i]);
    }
}

#[test]
fn visit_sequence_filter_with_shared_skip_set() {
    let skip_set = skip(&["Box", "Arc"]);
    let types: Vec<Type> = vec![
        parse_quote!(Box<Leaf>),
        parse_quote!(Arc<Leaf>),
        parse_quote!(Vec<Leaf>),
        parse_quote!(Leaf),
    ];
    let expected = ["Leaf", "Leaf", "Vec < Leaf >", "Leaf"];

    for i in 0..types.len() {
        assert_eq!(
            ty_str(&filter_inner_type(&types[i], &skip_set)),
            expected[i]
        );
    }
}

#[test]
fn visit_sequence_wrap_mixed_types() {
    let skip_set = skip(&["Vec", "Option"]);
    let types: Vec<Type> = vec![
        parse_quote!(String),
        parse_quote!(Vec<Token>),
        parse_quote!(Option<Node>),
        parse_quote!(&str),
        parse_quote!([u8; 4]),
    ];
    let expected = [
        "adze :: WithLeaf < String >",
        "Vec < adze :: WithLeaf < Token > >",
        "Option < adze :: WithLeaf < Node > >",
        "adze :: WithLeaf < & str >",
        "adze :: WithLeaf < [u8 ; 4] >",
    ];

    for i in 0..types.len() {
        assert_eq!(ty_str(&wrap_leaf_type(&types[i], &skip_set)), expected[i]);
    }
}

#[test]
fn visit_sequence_all_non_path_types_pass_through_extract() {
    let non_path_types: Vec<Type> = vec![
        parse_quote!(&str),
        parse_quote!(&mut i32),
        parse_quote!((u8, u16)),
        parse_quote!([f64; 10]),
        parse_quote!(()),
    ];

    for ty in &non_path_types {
        let (inner, ok) = try_extract_inner_type(ty, "Vec", &skip(&["Box"]));
        assert!(!ok, "non-path type should not match: {}", ty_str(ty));
        assert_eq!(ty_str(&inner), ty_str(ty));
    }
}

#[test]
fn visit_sequence_extract_same_target_across_varying_depths() {
    let skip_set = skip(&["Box", "Arc", "Rc"]);
    let cases: Vec<(Type, &str)> = vec![
        (parse_quote!(Option<i32>), "i32"),
        (parse_quote!(Box<Option<i32>>), "i32"),
        (parse_quote!(Arc<Box<Option<i32>>>), "i32"),
        (parse_quote!(Rc<Arc<Box<Option<i32>>>>), "i32"),
    ];

    for (ty, expected) in &cases {
        let (inner, ok) = try_extract_inner_type(ty, "Option", &skip_set);
        assert!(ok, "should extract Option from: {}", ty_str(ty));
        assert_eq!(ty_str(&inner), *expected);
    }
}
