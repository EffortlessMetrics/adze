//! Tests for `wrap_leaf_type` in adze-common — v4 suite.
//!
//! 55+ tests covering: wrap output validation, type transformation correctness,
//! container/skip interactions, deeply nested types, multi-arg generics,
//! non-path types, qualified paths, idempotency, and edge cases.

use std::collections::HashSet;

use adze_common::wrap_leaf_type;
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

/// Local helper: returns true when type has angle-bracketed generic args.
fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

// ===========================================================================
// 1. Basic leaf wrapping — simple types get WithLeaf wrapper
// ===========================================================================

#[test]
fn wrap_leaf_i8() {
    let ty: Type = parse_quote!(i8);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < i8 >"
    );
}

#[test]
fn wrap_leaf_u16() {
    let ty: Type = parse_quote!(u16);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < u16 >"
    );
}

#[test]
fn wrap_leaf_f32() {
    let ty: Type = parse_quote!(f32);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < f32 >"
    );
}

#[test]
fn wrap_leaf_char() {
    let ty: Type = parse_quote!(char);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < char >"
    );
}

#[test]
fn wrap_leaf_usize() {
    let ty: Type = parse_quote!(usize);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < usize >"
    );
}

#[test]
fn wrap_leaf_isize() {
    let ty: Type = parse_quote!(isize);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < isize >"
    );
}

// ===========================================================================
// 2. TokenStream output structure validation
// ===========================================================================

#[test]
fn output_starts_with_adze_prefix() {
    let ty: Type = parse_quote!(String);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert!(out.starts_with("adze :: WithLeaf <"));
}

#[test]
fn output_ends_with_closing_bracket() {
    let ty: Type = parse_quote!(u64);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert!(out.ends_with('>'));
}

#[test]
fn output_contains_original_type_name() {
    let ty: Type = parse_quote!(MyToken);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert!(out.contains("MyToken"));
}

#[test]
fn wrapped_type_is_path_type() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert!(matches!(wrapped, Type::Path(_)));
}

#[test]
fn wrapped_type_is_parameterized() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert!(is_parameterized(&wrapped));
}

#[test]
fn unwrapped_simple_type_is_not_parameterized() {
    let ty: Type = parse_quote!(i32);
    assert!(!is_parameterized(&ty));
}

// ===========================================================================
// 3. Skip-set container preservation
// ===========================================================================

#[test]
fn vec_in_skip_preserves_vec() {
    let ty: Type = parse_quote!(Vec<f64>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"])));
    assert!(out.starts_with("Vec <"));
    assert!(out.contains("adze :: WithLeaf < f64 >"));
}

#[test]
fn option_in_skip_preserves_option() {
    let ty: Type = parse_quote!(Option<char>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Option"])));
    assert_eq!(out, "Option < adze :: WithLeaf < char > >");
}

#[test]
fn box_in_skip_preserves_box() {
    let ty: Type = parse_quote!(Box<usize>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Box"])));
    assert_eq!(out, "Box < adze :: WithLeaf < usize > >");
}

#[test]
fn container_not_in_skip_wraps_entire_type() {
    let ty: Type = parse_quote!(Vec<i32>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn unrelated_skip_does_not_affect_wrapping() {
    let ty: Type = parse_quote!(Vec<i32>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Option", "Box"])));
    assert_eq!(out, "adze :: WithLeaf < Vec < i32 > >");
}

// ===========================================================================
// 4. Nested container wrapping
// ===========================================================================

#[test]
fn option_vec_skip_both() {
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Option", "Vec"])));
    assert_eq!(out, "Option < Vec < adze :: WithLeaf < u8 > > >");
}

#[test]
fn vec_option_skip_both() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Option"])));
    assert_eq!(out, "Vec < Option < adze :: WithLeaf < String > > >");
}

#[test]
fn box_vec_option_skip_all() {
    let ty: Type = parse_quote!(Box<Vec<Option<bool>>>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Box", "Vec", "Option"])));
    assert_eq!(out, "Box < Vec < Option < adze :: WithLeaf < bool > > > >");
}

#[test]
fn three_levels_skip_only_outer_two() {
    let ty: Type = parse_quote!(Option<Box<Vec<i32>>>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Option", "Box"])));
    assert_eq!(out, "Option < Box < adze :: WithLeaf < Vec < i32 > > > >");
}

#[test]
fn three_levels_skip_only_inner_two() {
    let ty: Type = parse_quote!(Option<Vec<Box<i32>>>);
    // Option not in skip, so entire thing is wrapped
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Box"])));
    assert_eq!(out, "adze :: WithLeaf < Option < Vec < Box < i32 > > > >");
}

// ===========================================================================
// 5. Custom / user-defined types
// ===========================================================================

#[test]
fn wrap_custom_struct_name() {
    let ty: Type = parse_quote!(Identifier);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < Identifier >");
}

#[test]
fn wrap_custom_type_with_generics_not_in_skip() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < HashMap < String , i32 > >");
}

#[test]
fn custom_container_in_skip_wraps_inner_args() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["HashMap"])));
    assert_eq!(
        out,
        "HashMap < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn custom_single_arg_container_in_skip() {
    let ty: Type = parse_quote!(Rc<MyNode>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Rc"])));
    assert_eq!(out, "Rc < adze :: WithLeaf < MyNode > >");
}

// ===========================================================================
// 6. Qualified / module paths
// ===========================================================================

#[test]
fn wrap_fully_qualified_type() {
    let ty: Type = parse_quote!(std::string::String);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < std :: string :: String >");
}

#[test]
fn qualified_path_last_segment_matched_for_skip() {
    // Skip matches last segment "Vec"
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"])));
    assert_eq!(out, "std :: vec :: Vec < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_deep_module_path() {
    let ty: Type = parse_quote!(a::b::c::MyType);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < a :: b :: c :: MyType >");
}

// ===========================================================================
// 7. Non-path types (references, tuples, arrays, slices)
// ===========================================================================

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_mutable_reference() {
    let ty: Type = parse_quote!(&mut u32);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < & mut u32 >");
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, bool));
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < (i32 , bool) >");
}

#[test]
fn wrap_unit_tuple() {
    let ty: Type = parse_quote!(());
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < () >");
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_slice_reference() {
    let ty: Type = parse_quote!(&[u8]);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < & [u8] >");
}

#[test]
fn non_path_type_ignores_skip_set() {
    let ty: Type = parse_quote!(&str);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["str"])));
    assert_eq!(out, "adze :: WithLeaf < & str >");
}

// ===========================================================================
// 8. Idempotency — wrapping is NOT idempotent
// ===========================================================================

#[test]
fn double_wrap_nests_with_leaf() {
    let ty: Type = parse_quote!(i32);
    let once = wrap_leaf_type(&ty, &skip(&[]));
    let twice = wrap_leaf_type(&once, &skip(&[]));
    assert_eq!(
        ty_str(&twice),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn triple_wrap_nests_three_deep() {
    let ty: Type = parse_quote!(bool);
    let once = wrap_leaf_type(&ty, &skip(&[]));
    let twice = wrap_leaf_type(&once, &skip(&[]));
    let thrice = wrap_leaf_type(&twice, &skip(&[]));
    assert_eq!(
        ty_str(&thrice),
        "adze :: WithLeaf < adze :: WithLeaf < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn double_wrap_container_in_skip() {
    let ty: Type = parse_quote!(Vec<String>);
    let ss = skip(&["Vec"]);
    let once = wrap_leaf_type(&ty, &ss);
    let twice = wrap_leaf_type(&once, &ss);
    assert_eq!(
        ty_str(&twice),
        "Vec < adze :: WithLeaf < adze :: WithLeaf < String > > >"
    );
}

// ===========================================================================
// 9. Multi-argument generics
// ===========================================================================

#[test]
fn result_in_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, MyError>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Result"])));
    assert_eq!(
        out,
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < MyError > >"
    );
}

#[test]
fn result_not_in_skip_wraps_whole() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < Result < i32 , String > >");
}

#[test]
fn hashmap_in_skip_wraps_both_key_value() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["HashMap"])));
    assert_eq!(
        out,
        "HashMap < adze :: WithLeaf < String > , adze :: WithLeaf < Vec < i32 > > >"
    );
}

#[test]
fn hashmap_in_skip_with_vec_in_skip_recurses() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["HashMap", "Vec"])));
    assert_eq!(
        out,
        "HashMap < adze :: WithLeaf < String > , Vec < adze :: WithLeaf < i32 > > >"
    );
}

// ===========================================================================
// 10. Empty / degenerate skip sets
// ===========================================================================

#[test]
fn empty_skip_wraps_everything() {
    let empty = skip(&[]);
    let types: Vec<Type> = vec![
        parse_quote!(i32),
        parse_quote!(String),
        parse_quote!(Vec<u8>),
        parse_quote!(Option<bool>),
    ];
    for ty in &types {
        let out = ty_str(&wrap_leaf_type(ty, &empty));
        assert!(
            out.starts_with("adze :: WithLeaf <"),
            "Expected wrapping for {}",
            ty_str(ty)
        );
    }
}

#[test]
fn large_skip_set_still_wraps_non_matching() {
    let ss = skip(&["Vec", "Option", "Box", "Rc", "Arc", "Cell", "RefCell"]);
    let ty: Type = parse_quote!(String);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &ss)),
        "adze :: WithLeaf < String >"
    );
}

// ===========================================================================
// 11. is_parameterized interactions with wrap output
// ===========================================================================

#[test]
fn simple_type_becomes_parameterized_after_wrap() {
    let ty: Type = parse_quote!(String);
    assert!(!is_parameterized(&ty));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert!(is_parameterized(&wrapped));
}

#[test]
fn already_parameterized_stays_parameterized() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert!(is_parameterized(&ty));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert!(is_parameterized(&wrapped));
}

#[test]
fn container_in_skip_output_is_still_parameterized() {
    let ty: Type = parse_quote!(Option<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert!(is_parameterized(&wrapped));
}

// ===========================================================================
// 12. Consistency: same input + skip → same output
// ===========================================================================

#[test]
fn deterministic_output_simple() {
    let ty: Type = parse_quote!(u32);
    let ss = skip(&[]);
    let a = ty_str(&wrap_leaf_type(&ty, &ss));
    let b = ty_str(&wrap_leaf_type(&ty, &ss));
    assert_eq!(a, b);
}

#[test]
fn deterministic_output_container() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let ss = skip(&["Vec", "Option"]);
    let a = ty_str(&wrap_leaf_type(&ty, &ss));
    let b = ty_str(&wrap_leaf_type(&ty, &ss));
    assert_eq!(a, b);
}

// ===========================================================================
// 13. Exact string output for all primitive types
// ===========================================================================

#[test]
fn wrap_all_integer_types() {
    let cases: Vec<(&str, Type)> = vec![
        ("i8", parse_quote!(i8)),
        ("i16", parse_quote!(i16)),
        ("i32", parse_quote!(i32)),
        ("i64", parse_quote!(i64)),
        ("i128", parse_quote!(i128)),
        ("u8", parse_quote!(u8)),
        ("u16", parse_quote!(u16)),
        ("u32", parse_quote!(u32)),
        ("u64", parse_quote!(u64)),
        ("u128", parse_quote!(u128)),
    ];
    let ss = skip(&[]);
    for (name, ty) in &cases {
        let expected = format!("adze :: WithLeaf < {} >", name);
        assert_eq!(ty_str(&wrap_leaf_type(ty, &ss)), expected);
    }
}

#[test]
fn wrap_float_types() {
    let ss = skip(&[]);
    let ty_f32: Type = parse_quote!(f32);
    let ty_f64: Type = parse_quote!(f64);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty_f32, &ss)),
        "adze :: WithLeaf < f32 >"
    );
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty_f64, &ss)),
        "adze :: WithLeaf < f64 >"
    );
}

// ===========================================================================
// 14. Four-level nesting
// ===========================================================================

#[test]
fn four_levels_all_in_skip() {
    let ty: Type = parse_quote!(Vec<Box<Option<Vec<u8>>>>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Box", "Option"])));
    assert_eq!(
        out,
        "Vec < Box < Option < Vec < adze :: WithLeaf < u8 > > > > >"
    );
}

#[test]
fn four_levels_only_alternating_in_skip() {
    // Vec in skip, Box NOT, Option in skip, Vec in skip
    // Vec<Box<Option<...>>> — Vec skipped, inner is Box which is NOT skipped → wrap Box<Option<...>>
    let ty: Type = parse_quote!(Vec<Box<Option<String>>>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Option"])));
    assert_eq!(
        out,
        "Vec < adze :: WithLeaf < Box < Option < String > > > >"
    );
}

// ===========================================================================
// 15. Specific TokenStream token count / structure
// ===========================================================================

#[test]
fn wrapped_simple_has_expected_token_count() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    let tokens = wrapped.to_token_stream();
    // adze :: WithLeaf < bool >  → 7 token trees
    let count = tokens.into_iter().count();
    assert_eq!(count, 7, "Expected 7 token trees for wrapped bool");
}

#[test]
fn container_in_skip_preserves_original_outer_ident() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    if let Type::Path(p) = &wrapped {
        let seg = p.path.segments.last().unwrap();
        assert_eq!(seg.ident, "Vec");
    } else {
        panic!("Expected Type::Path");
    }
}

#[test]
fn wrapped_leaf_outer_ident_is_adze() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    if let Type::Path(p) = &wrapped {
        let first_seg = p.path.segments.first().unwrap();
        assert_eq!(first_seg.ident, "adze");
    } else {
        panic!("Expected Type::Path");
    }
}

// ===========================================================================
// 16. parse_str round-trip validation
// ===========================================================================

#[test]
fn wrap_type_created_via_parse_str() {
    let ty: Type = syn::parse_str::<syn::Type>("String").unwrap();
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < String >");
}

#[test]
fn wrap_vec_via_parse_str() {
    let ty: Type = syn::parse_str::<syn::Type>("Vec<u32>").unwrap();
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"])));
    assert_eq!(out, "Vec < adze :: WithLeaf < u32 > >");
}

#[test]
fn wrap_nested_via_parse_str() {
    let ty = syn::parse_str::<syn::Type>("Option<Vec<bool>>").unwrap();
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Option", "Vec"])));
    assert_eq!(out, "Option < Vec < adze :: WithLeaf < bool > > >");
}

// ===========================================================================
// 17. Comparison: wrap with vs without skip for same container
// ===========================================================================

#[test]
fn vec_with_and_without_skip_differ() {
    let ty: Type = parse_quote!(Vec<i32>);
    let with_skip = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"])));
    let without_skip = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_ne!(with_skip, without_skip);
    assert_eq!(with_skip, "Vec < adze :: WithLeaf < i32 > >");
    assert_eq!(without_skip, "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn option_with_and_without_skip_differ() {
    let ty: Type = parse_quote!(Option<String>);
    let with_skip = ty_str(&wrap_leaf_type(&ty, &skip(&["Option"])));
    let without_skip = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_ne!(with_skip, without_skip);
}

// ===========================================================================
// 18. Lifetime and pointer types (non-path wrapped directly)
// ===========================================================================

#[test]
fn wrap_static_lifetime_ref() {
    let ty: Type = parse_quote!(&'static str);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < & 'static str >");
}

#[test]
fn wrap_raw_pointer() {
    let ty: Type = parse_quote!(*const u8);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < * const u8 >");
}

#[test]
fn wrap_mut_raw_pointer() {
    let ty: Type = parse_quote!(*mut u8);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&[])));
    assert_eq!(out, "adze :: WithLeaf < * mut u8 >");
}

// ===========================================================================
// 19. Batch validation: containers in skip have inner wrapped
// ===========================================================================

#[test]
fn batch_containers_inner_always_wrapped() {
    let ss = skip(&["Vec", "Option", "Box"]);
    let cases: Vec<(Type, &str)> = vec![
        (parse_quote!(Vec<i32>), "Vec < adze :: WithLeaf < i32 > >"),
        (
            parse_quote!(Vec<String>),
            "Vec < adze :: WithLeaf < String > >",
        ),
        (
            parse_quote!(Option<f64>),
            "Option < adze :: WithLeaf < f64 > >",
        ),
        (
            parse_quote!(Option<bool>),
            "Option < adze :: WithLeaf < bool > >",
        ),
        (parse_quote!(Box<char>), "Box < adze :: WithLeaf < char > >"),
        (
            parse_quote!(Box<usize>),
            "Box < adze :: WithLeaf < usize > >",
        ),
    ];
    for (ty, expected) in &cases {
        assert_eq!(ty_str(&wrap_leaf_type(ty, &ss)), *expected);
    }
}

// ===========================================================================
// 20. Nested containers: partial skip coverage
// ===========================================================================

#[test]
fn nested_skip_only_inner() {
    // Box NOT in skip, Vec in skip → entire Box<Vec<i32>> is wrapped
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"])));
    assert_eq!(out, "adze :: WithLeaf < Box < Vec < i32 > > >");
}

#[test]
fn nested_option_box_vec_skip_option_vec() {
    let ty: Type = parse_quote!(Option<Box<Vec<u16>>>);
    // Option skipped → recurse into Box. Box NOT skipped → wrap Box<Vec<u16>>
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Option", "Vec"])));
    assert_eq!(out, "Option < adze :: WithLeaf < Box < Vec < u16 > > > >");
}

// ===========================================================================
// 21. Edge case: type named same as skip entry but without generics
// ===========================================================================

// Note: wrap_leaf_type panics if a skip-set type lacks angle-bracket args,
// so a bare `Vec` (no generics) in skip would panic. We test that non-matching
// bare types are simply wrapped.

#[test]
fn bare_custom_name_matching_skip_entry_wraps_if_no_generics_not_in_path() {
    // A type named "Foo" not matching anything in skip → wrapped
    let ty: Type = parse_quote!(Foo);
    let out = ty_str(&wrap_leaf_type(&ty, &skip(&["Bar"])));
    assert_eq!(out, "adze :: WithLeaf < Foo >");
}

// ===========================================================================
// 22. Larger batch: 14 primitives all wrapped consistently
// ===========================================================================

#[test]
fn all_primitives_wrapped_consistently() {
    let primitives: Vec<Type> = vec![
        parse_quote!(i8),
        parse_quote!(i16),
        parse_quote!(i32),
        parse_quote!(i64),
        parse_quote!(i128),
        parse_quote!(u8),
        parse_quote!(u16),
        parse_quote!(u32),
        parse_quote!(u64),
        parse_quote!(u128),
        parse_quote!(f32),
        parse_quote!(f64),
        parse_quote!(bool),
        parse_quote!(char),
    ];
    let ss = skip(&[]);
    for ty in &primitives {
        let out = ty_str(&wrap_leaf_type(ty, &ss));
        assert!(
            out.starts_with("adze :: WithLeaf <"),
            "Failed for {}",
            ty_str(ty)
        );
        assert!(out.ends_with('>'), "Failed for {}", ty_str(ty));
    }
    assert!(!primitives.is_empty());
}
