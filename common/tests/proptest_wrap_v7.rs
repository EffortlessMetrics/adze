//! Property-based and unit tests for `wrap_leaf_type` in adze-common.

use adze_common::{try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn type_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn container(prim: &Type, name: &str) -> Type {
    match name {
        "Vec" => parse_quote!(Vec<#prim>),
        "Option" => parse_quote!(Option<#prim>),
        "Box" => parse_quote!(Box<#prim>),
        _ => unreachable!(),
    }
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_primitive() -> impl Strategy<Value = Type> {
    prop_oneof![
        Just(parse_quote!(i32)),
        Just(parse_quote!(u32)),
        Just(parse_quote!(String)),
        Just(parse_quote!(bool)),
        Just(parse_quote!(f64)),
        Just(parse_quote!(u8)),
    ]
}

fn arb_wrapper() -> impl Strategy<Value = &'static str> {
    prop_oneof![Just("Vec"), Just("Option"), Just("Box")]
}

// ---------------------------------------------------------------------------
// Property tests (34)
// ---------------------------------------------------------------------------

proptest! {
    // 1. wrap always produces non-empty type string
    #[test]
    fn pt01_wrap_nonempty(prim in arb_primitive()) {
        prop_assert!(!type_str(&wrap_leaf_type(&prim, &HashSet::new())).is_empty());
    }

    // 2. wrap output differs from input
    #[test]
    fn pt02_wrap_changes_type(prim in arb_primitive()) {
        let result = wrap_leaf_type(&prim, &HashSet::new());
        prop_assert_ne!(type_str(&result), type_str(&prim));
    }

    // 3. wrap output contains wrapper name (WithLeaf)
    #[test]
    fn pt03_wrap_contains_with_leaf(prim in arb_primitive()) {
        let result = wrap_leaf_type(&prim, &HashSet::new());
        prop_assert!(type_str(&result).contains("WithLeaf"));
    }

    // 4. wrap output contains inner type name
    #[test]
    fn pt04_wrap_contains_inner_name(prim in arb_primitive()) {
        let result = wrap_leaf_type(&prim, &HashSet::new());
        prop_assert!(type_str(&result).contains(&type_str(&prim)));
    }

    // 5. wrap then extract → roundtrip identity
    #[test]
    fn pt05_wrap_then_extract_roundtrip(prim in arb_primitive()) {
        let wrapped = wrap_leaf_type(&prim, &HashSet::new());
        let (extracted, found) = try_extract_inner_type(&wrapped, "WithLeaf", &HashSet::new());
        prop_assert!(found);
        prop_assert_eq!(type_str(&extracted), type_str(&prim));
    }

    // 6. double wrap → contains both wrappers
    #[test]
    fn pt06_double_wrap_two_with_leaf(prim in arb_primitive()) {
        let once = wrap_leaf_type(&prim, &HashSet::new());
        let twice = wrap_leaf_type(&once, &HashSet::new());
        prop_assert!(type_str(&twice).matches("WithLeaf").count() >= 2);
    }

    // 7. wrap is deterministic (same input → same output)
    #[test]
    fn pt07_wrap_deterministic(prim in arb_primitive()) {
        let s = HashSet::new();
        prop_assert_eq!(
            type_str(&wrap_leaf_type(&prim, &s)),
            type_str(&wrap_leaf_type(&prim, &s)),
        );
    }

    // 8. wrap with "Vec" → output starts with "Vec"
    #[test]
    fn pt08_vec_skip_starts_with_vec(prim in arb_primitive()) {
        let ty: Type = parse_quote!(Vec<#prim>);
        let result = wrap_leaf_type(&ty, &skip(&["Vec"]));
        prop_assert!(type_str(&result).starts_with("Vec"));
    }

    // 9. wrap with "Option" → output starts with "Option"
    #[test]
    fn pt09_option_skip_starts_with_option(prim in arb_primitive()) {
        let ty: Type = parse_quote!(Option<#prim>);
        let result = wrap_leaf_type(&ty, &skip(&["Option"]));
        prop_assert!(type_str(&result).starts_with("Option"));
    }

    // 10. wrap with "Box" → output starts with "Box"
    #[test]
    fn pt10_box_skip_starts_with_box(prim in arb_primitive()) {
        let ty: Type = parse_quote!(Box<#prim>);
        let result = wrap_leaf_type(&ty, &skip(&["Box"]));
        prop_assert!(type_str(&result).starts_with("Box"));
    }

    // 11. primitive always gets WithLeaf regardless of skip set
    #[test]
    fn pt11_primitive_always_wrapped(prim in arb_primitive(), w in arb_wrapper()) {
        let result = wrap_leaf_type(&prim, &skip(&[w]));
        prop_assert!(type_str(&result).contains("WithLeaf"));
    }

    // 12. container NOT in skip → entire container wrapped
    #[test]
    fn pt12_container_not_in_skip_wraps_whole(prim in arb_primitive(), w in arb_wrapper()) {
        let ty = container(&prim, w);
        let result = wrap_leaf_type(&ty, &HashSet::new());
        prop_assert!(type_str(&result).starts_with("adze"));
    }

    // 13. extract non-matching → found is false
    #[test]
    fn pt13_extract_non_matching_false(prim in arb_primitive()) {
        let (_, found) = try_extract_inner_type(&prim, "Vec", &HashSet::new());
        prop_assert!(!found);
    }

    // 14. extract non-matching → original preserved
    #[test]
    fn pt14_extract_non_matching_preserves(prim in arb_primitive()) {
        let (result, _) = try_extract_inner_type(&prim, "Vec", &HashSet::new());
        prop_assert_eq!(type_str(&result), type_str(&prim));
    }

    // 15. wrap output has angle brackets
    #[test]
    fn pt15_wrap_has_angle_brackets(prim in arb_primitive()) {
        let s = type_str(&wrap_leaf_type(&prim, &HashSet::new()));
        prop_assert!(s.contains('<'));
        prop_assert!(s.contains('>'));
    }

    // 16. skip container preserves container name
    #[test]
    fn pt16_skip_container_preserves_name(prim in arb_primitive(), w in arb_wrapper()) {
        let ty = container(&prim, w);
        let result = wrap_leaf_type(&ty, &skip(&[w]));
        prop_assert!(type_str(&result).contains(w));
    }

    // 17. double wrap differs from single wrap
    #[test]
    fn pt17_double_wrap_differs_from_single(prim in arb_primitive()) {
        let s = HashSet::new();
        let once = wrap_leaf_type(&prim, &s);
        let twice = wrap_leaf_type(&once, &s);
        prop_assert_ne!(type_str(&once), type_str(&twice));
    }

    // 18. different primitives → different output
    #[test]
    fn pt18_different_prims_different_output(p1 in arb_primitive(), p2 in arb_primitive()) {
        let s = HashSet::new();
        prop_assume!(type_str(&p1) != type_str(&p2));
        prop_assert_ne!(
            type_str(&wrap_leaf_type(&p1, &s)),
            type_str(&wrap_leaf_type(&p2, &s)),
        );
    }

    // 19. extract matching container → found is true
    #[test]
    fn pt19_extract_matching_true(prim in arb_primitive(), w in arb_wrapper()) {
        let ty = container(&prim, w);
        let (_, found) = try_extract_inner_type(&ty, w, &HashSet::new());
        prop_assert!(found);
    }

    // 20. extract matching container → unwraps one layer
    #[test]
    fn pt20_extract_matching_unwraps(prim in arb_primitive(), w in arb_wrapper()) {
        let ty = container(&prim, w);
        let (inner, _) = try_extract_inner_type(&ty, w, &HashSet::new());
        prop_assert_eq!(type_str(&inner), type_str(&prim));
    }

    // 21. empty skip always wraps with adze prefix
    #[test]
    fn pt21_empty_skip_starts_adze(prim in arb_primitive()) {
        let result = wrap_leaf_type(&prim, &HashSet::new());
        prop_assert!(type_str(&result).starts_with("adze"));
    }

    // 22. wrap output contains "adze"
    #[test]
    fn pt22_wrap_output_contains_adze(prim in arb_primitive()) {
        let result = wrap_leaf_type(&prim, &HashSet::new());
        prop_assert!(type_str(&result).contains("adze"));
    }

    // 23. wrap output is longer than input
    #[test]
    fn pt23_wrap_output_longer(prim in arb_primitive()) {
        let result = wrap_leaf_type(&prim, &HashSet::new());
        prop_assert!(type_str(&result).len() > type_str(&prim).len());
    }

    // 24. balanced angle brackets
    #[test]
    fn pt24_balanced_angle_brackets(prim in arb_primitive()) {
        let s = type_str(&wrap_leaf_type(&prim, &HashSet::new()));
        prop_assert_eq!(s.matches('<').count(), s.matches('>').count());
    }

    // 25. skip container wraps inner and keeps both names
    #[test]
    fn pt25_skip_container_wraps_inner(prim in arb_primitive(), w in arb_wrapper()) {
        let ty = container(&prim, w);
        let s = type_str(&wrap_leaf_type(&ty, &skip(&[w])));
        prop_assert!(s.contains(w));
        prop_assert!(s.contains("WithLeaf"));
        prop_assert!(s.contains(&type_str(&prim)));
    }

    // 26. full skip set still wraps primitive leaf
    #[test]
    fn pt26_full_skip_set_wraps_leaf(prim in arb_primitive()) {
        let result = wrap_leaf_type(&prim, &skip(&["Vec", "Option", "Box"]));
        prop_assert!(type_str(&result).contains("WithLeaf"));
    }

    // 27. wrap is not idempotent
    #[test]
    fn pt27_wrap_not_idempotent(prim in arb_primitive()) {
        let s = HashSet::new();
        let once = wrap_leaf_type(&prim, &s);
        let twice = wrap_leaf_type(&once, &s);
        prop_assert_ne!(type_str(&once), type_str(&twice));
    }

    // 28. extract through skip_over (Box<Vec<prim>> → prim via Vec with Box skipped)
    #[test]
    fn pt28_extract_through_skip(prim in arb_primitive()) {
        let ty: Type = parse_quote!(Box<Vec<#prim>>);
        let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
        prop_assert!(found);
        prop_assert_eq!(type_str(&inner), type_str(&prim));
    }

    // 29. balanced brackets on wrapped containers
    #[test]
    fn pt29_container_balanced_brackets(prim in arb_primitive(), w in arb_wrapper()) {
        let ty = container(&prim, w);
        let s = type_str(&wrap_leaf_type(&ty, &skip(&[w])));
        prop_assert_eq!(s.matches('<').count(), s.matches('>').count());
    }

    // 30. extract wrong wrapper returns false
    #[test]
    fn pt30_extract_wrong_wrapper_false(prim in arb_primitive()) {
        let ty: Type = parse_quote!(Vec<#prim>);
        let (_, found) = try_extract_inner_type(&ty, "Option", &HashSet::new());
        prop_assert!(!found);
    }

    // 31. extract wrong wrapper preserves type
    #[test]
    fn pt31_extract_wrong_wrapper_preserves(prim in arb_primitive()) {
        let ty: Type = parse_quote!(Vec<#prim>);
        let (result, _) = try_extract_inner_type(&ty, "Option", &HashSet::new());
        prop_assert_eq!(type_str(&result), type_str(&ty));
    }

    // 32. container not in skip starts with "adze"
    #[test]
    fn pt32_container_no_skip_starts_adze(prim in arb_primitive(), w in arb_wrapper()) {
        let ty = container(&prim, w);
        let result = wrap_leaf_type(&ty, &HashSet::new());
        prop_assert!(type_str(&result).starts_with("adze"));
    }

    // 33. wrap container then extract container → inner has WithLeaf
    #[test]
    fn pt33_wrap_then_extract_container(prim in arb_primitive(), w in arb_wrapper()) {
        let ty = container(&prim, w);
        let wrapped = wrap_leaf_type(&ty, &skip(&[w]));
        let (inner, found) = try_extract_inner_type(&wrapped, w, &HashSet::new());
        prop_assert!(found);
        prop_assert!(type_str(&inner).contains("WithLeaf"));
    }

    // 34. double wrap produces ≥ 2 "adze" occurrences
    #[test]
    fn pt34_double_wrap_double_adze(prim in arb_primitive()) {
        let s = HashSet::new();
        let once = wrap_leaf_type(&prim, &s);
        let twice = wrap_leaf_type(&once, &s);
        prop_assert!(type_str(&twice).matches("adze").count() >= 2);
    }
}

// ---------------------------------------------------------------------------
// Unit-test helpers
// ---------------------------------------------------------------------------

fn assert_wrap(ty: &Type, skip_over: &HashSet<&str>, expected: &str) {
    assert_eq!(type_str(&wrap_leaf_type(ty, skip_over)), expected);
}

fn assert_extract(
    ty: &Type,
    inner_of: &str,
    skip_over: &HashSet<&str>,
    expected_str: &str,
    expected_found: bool,
) {
    let (inner, found) = try_extract_inner_type(ty, inner_of, skip_over);
    assert_eq!(found, expected_found);
    assert_eq!(type_str(&inner), expected_str);
}

// ---------------------------------------------------------------------------
// Unit: wrap each primitive with no skip (6 tests)
// ---------------------------------------------------------------------------

#[test]
fn ut01_wrap_i32() {
    assert_wrap(
        &parse_quote!(i32),
        &HashSet::new(),
        "adze :: WithLeaf < i32 >",
    );
}

#[test]
fn ut02_wrap_u32() {
    assert_wrap(
        &parse_quote!(u32),
        &HashSet::new(),
        "adze :: WithLeaf < u32 >",
    );
}

#[test]
fn ut03_wrap_string() {
    assert_wrap(
        &parse_quote!(String),
        &HashSet::new(),
        "adze :: WithLeaf < String >",
    );
}

#[test]
fn ut04_wrap_bool() {
    assert_wrap(
        &parse_quote!(bool),
        &HashSet::new(),
        "adze :: WithLeaf < bool >",
    );
}

#[test]
fn ut05_wrap_f64() {
    assert_wrap(
        &parse_quote!(f64),
        &HashSet::new(),
        "adze :: WithLeaf < f64 >",
    );
}

#[test]
fn ut06_wrap_u8() {
    assert_wrap(
        &parse_quote!(u8),
        &HashSet::new(),
        "adze :: WithLeaf < u8 >",
    );
}

// ---------------------------------------------------------------------------
// Unit: wrap each primitive in Vec (Vec in skip) (6 tests)
// ---------------------------------------------------------------------------

#[test]
fn ut07_wrap_vec_i32() {
    assert_wrap(
        &parse_quote!(Vec<i32>),
        &skip(&["Vec"]),
        "Vec < adze :: WithLeaf < i32 > >",
    );
}

#[test]
fn ut08_wrap_vec_u32() {
    assert_wrap(
        &parse_quote!(Vec<u32>),
        &skip(&["Vec"]),
        "Vec < adze :: WithLeaf < u32 > >",
    );
}

#[test]
fn ut09_wrap_vec_string() {
    assert_wrap(
        &parse_quote!(Vec<String>),
        &skip(&["Vec"]),
        "Vec < adze :: WithLeaf < String > >",
    );
}

#[test]
fn ut10_wrap_vec_bool() {
    assert_wrap(
        &parse_quote!(Vec<bool>),
        &skip(&["Vec"]),
        "Vec < adze :: WithLeaf < bool > >",
    );
}

#[test]
fn ut11_wrap_vec_f64() {
    assert_wrap(
        &parse_quote!(Vec<f64>),
        &skip(&["Vec"]),
        "Vec < adze :: WithLeaf < f64 > >",
    );
}

#[test]
fn ut12_wrap_vec_u8() {
    assert_wrap(
        &parse_quote!(Vec<u8>),
        &skip(&["Vec"]),
        "Vec < adze :: WithLeaf < u8 > >",
    );
}

// ---------------------------------------------------------------------------
// Unit: wrap each primitive in Option (Option in skip) (6 tests)
// ---------------------------------------------------------------------------

#[test]
fn ut13_wrap_option_i32() {
    assert_wrap(
        &parse_quote!(Option<i32>),
        &skip(&["Option"]),
        "Option < adze :: WithLeaf < i32 > >",
    );
}

#[test]
fn ut14_wrap_option_u32() {
    assert_wrap(
        &parse_quote!(Option<u32>),
        &skip(&["Option"]),
        "Option < adze :: WithLeaf < u32 > >",
    );
}

#[test]
fn ut15_wrap_option_string() {
    assert_wrap(
        &parse_quote!(Option<String>),
        &skip(&["Option"]),
        "Option < adze :: WithLeaf < String > >",
    );
}

#[test]
fn ut16_wrap_option_bool() {
    assert_wrap(
        &parse_quote!(Option<bool>),
        &skip(&["Option"]),
        "Option < adze :: WithLeaf < bool > >",
    );
}

#[test]
fn ut17_wrap_option_f64() {
    assert_wrap(
        &parse_quote!(Option<f64>),
        &skip(&["Option"]),
        "Option < adze :: WithLeaf < f64 > >",
    );
}

#[test]
fn ut18_wrap_option_u8() {
    assert_wrap(
        &parse_quote!(Option<u8>),
        &skip(&["Option"]),
        "Option < adze :: WithLeaf < u8 > >",
    );
}

// ---------------------------------------------------------------------------
// Unit: wrap each primitive in Box (Box in skip) (6 tests)
// ---------------------------------------------------------------------------

#[test]
fn ut19_wrap_box_i32() {
    assert_wrap(
        &parse_quote!(Box<i32>),
        &skip(&["Box"]),
        "Box < adze :: WithLeaf < i32 > >",
    );
}

#[test]
fn ut20_wrap_box_u32() {
    assert_wrap(
        &parse_quote!(Box<u32>),
        &skip(&["Box"]),
        "Box < adze :: WithLeaf < u32 > >",
    );
}

#[test]
fn ut21_wrap_box_string() {
    assert_wrap(
        &parse_quote!(Box<String>),
        &skip(&["Box"]),
        "Box < adze :: WithLeaf < String > >",
    );
}

#[test]
fn ut22_wrap_box_bool() {
    assert_wrap(
        &parse_quote!(Box<bool>),
        &skip(&["Box"]),
        "Box < adze :: WithLeaf < bool > >",
    );
}

#[test]
fn ut23_wrap_box_f64() {
    assert_wrap(
        &parse_quote!(Box<f64>),
        &skip(&["Box"]),
        "Box < adze :: WithLeaf < f64 > >",
    );
}

#[test]
fn ut24_wrap_box_u8() {
    assert_wrap(
        &parse_quote!(Box<u8>),
        &skip(&["Box"]),
        "Box < adze :: WithLeaf < u8 > >",
    );
}

// ---------------------------------------------------------------------------
// Unit: wrap compound types (6 tests)
// ---------------------------------------------------------------------------

#[test]
fn ut25_wrap_vec_vec_i32() {
    assert_wrap(
        &parse_quote!(Vec<Vec<i32>>),
        &skip(&["Vec"]),
        "Vec < Vec < adze :: WithLeaf < i32 > > >",
    );
}

#[test]
fn ut26_wrap_option_vec_string() {
    assert_wrap(
        &parse_quote!(Option<Vec<String>>),
        &skip(&["Option", "Vec"]),
        "Option < Vec < adze :: WithLeaf < String > > >",
    );
}

#[test]
fn ut27_wrap_vec_option_bool() {
    assert_wrap(
        &parse_quote!(Vec<Option<bool>>),
        &skip(&["Vec", "Option"]),
        "Vec < Option < adze :: WithLeaf < bool > > >",
    );
}

#[test]
fn ut28_wrap_box_vec_u32() {
    assert_wrap(
        &parse_quote!(Box<Vec<u32>>),
        &skip(&["Box", "Vec"]),
        "Box < Vec < adze :: WithLeaf < u32 > > >",
    );
}

#[test]
fn ut29_wrap_box_option_f64() {
    assert_wrap(
        &parse_quote!(Box<Option<f64>>),
        &skip(&["Box", "Option"]),
        "Box < Option < adze :: WithLeaf < f64 > > >",
    );
}

#[test]
fn ut30_wrap_option_box_u8() {
    assert_wrap(
        &parse_quote!(Option<Box<u8>>),
        &skip(&["Option", "Box"]),
        "Option < Box < adze :: WithLeaf < u8 > > >",
    );
}

// ---------------------------------------------------------------------------
// Unit: wrap reference types (4 tests)
// ---------------------------------------------------------------------------

#[test]
fn ut31_wrap_ref_str() {
    assert_wrap(
        &parse_quote!(&str),
        &HashSet::new(),
        "adze :: WithLeaf < & str >",
    );
}

#[test]
fn ut32_wrap_ref_i32() {
    assert_wrap(
        &parse_quote!(&i32),
        &HashSet::new(),
        "adze :: WithLeaf < & i32 >",
    );
}

#[test]
fn ut33_wrap_ref_bool() {
    assert_wrap(
        &parse_quote!(&bool),
        &HashSet::new(),
        "adze :: WithLeaf < & bool >",
    );
}

#[test]
fn ut34_wrap_ref_u8_slice() {
    assert_wrap(
        &parse_quote!(&[u8]),
        &HashSet::new(),
        "adze :: WithLeaf < & [u8] >",
    );
}

// ---------------------------------------------------------------------------
// Unit: wrap tuple types (3 tests)
// ---------------------------------------------------------------------------

#[test]
fn ut35_wrap_tuple_i32_u32() {
    assert_wrap(
        &parse_quote!((i32, u32)),
        &HashSet::new(),
        "adze :: WithLeaf < (i32 , u32) >",
    );
}

#[test]
fn ut36_wrap_tuple_string_bool() {
    assert_wrap(
        &parse_quote!((String, bool)),
        &HashSet::new(),
        "adze :: WithLeaf < (String , bool) >",
    );
}

#[test]
fn ut37_wrap_tuple_f64_u8() {
    assert_wrap(
        &parse_quote!((f64, u8)),
        &HashSet::new(),
        "adze :: WithLeaf < (f64 , u8) >",
    );
}

// ---------------------------------------------------------------------------
// Unit: wrap array type (1 test)
// ---------------------------------------------------------------------------

#[test]
fn ut38_wrap_array_u8() {
    assert_wrap(
        &parse_quote!([u8; 4]),
        &HashSet::new(),
        "adze :: WithLeaf < [u8 ; 4] >",
    );
}

// ---------------------------------------------------------------------------
// Unit: nested skip and special cases (4 tests)
// ---------------------------------------------------------------------------

#[test]
fn ut39_wrap_vec_option_i32_full_skip() {
    assert_wrap(
        &parse_quote!(Vec<Option<i32>>),
        &skip(&["Vec", "Option", "Box"]),
        "Vec < Option < adze :: WithLeaf < i32 > > >",
    );
}

#[test]
fn ut40_wrap_result_both_args() {
    assert_wrap(
        &parse_quote!(Result<String, i32>),
        &skip(&["Result"]),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >",
    );
}

#[test]
fn ut41_wrap_vec_no_skip() {
    assert_wrap(
        &parse_quote!(Vec<i32>),
        &HashSet::new(),
        "adze :: WithLeaf < Vec < i32 > >",
    );
}

#[test]
fn ut42_wrap_option_no_skip() {
    assert_wrap(
        &parse_quote!(Option<String>),
        &HashSet::new(),
        "adze :: WithLeaf < Option < String > >",
    );
}

// ---------------------------------------------------------------------------
// Unit: container NOT in skip → whole type wrapped (1 test)
// ---------------------------------------------------------------------------

#[test]
fn ut43_wrap_box_no_skip() {
    assert_wrap(
        &parse_quote!(Box<bool>),
        &HashSet::new(),
        "adze :: WithLeaf < Box < bool > >",
    );
}

// ---------------------------------------------------------------------------
// Unit: extract tests (6 tests)
// ---------------------------------------------------------------------------

#[test]
fn ut44_extract_vec_string() {
    assert_extract(
        &parse_quote!(Vec<String>),
        "Vec",
        &HashSet::new(),
        "String",
        true,
    );
}

#[test]
fn ut45_extract_option_i32() {
    assert_extract(
        &parse_quote!(Option<i32>),
        "Option",
        &HashSet::new(),
        "i32",
        true,
    );
}

#[test]
fn ut46_extract_box_bool() {
    assert_extract(
        &parse_quote!(Box<bool>),
        "Box",
        &HashSet::new(),
        "bool",
        true,
    );
}

#[test]
fn ut47_extract_non_matching_vec() {
    assert_extract(
        &parse_quote!(Option<String>),
        "Vec",
        &HashSet::new(),
        "Option < String >",
        false,
    );
}

#[test]
fn ut48_extract_non_matching_option() {
    assert_extract(
        &parse_quote!(Vec<i32>),
        "Option",
        &HashSet::new(),
        "Vec < i32 >",
        false,
    );
}

#[test]
fn ut49_extract_non_matching_box() {
    assert_extract(
        &parse_quote!(Vec<u32>),
        "Box",
        &HashSet::new(),
        "Vec < u32 >",
        false,
    );
}

// ---------------------------------------------------------------------------
// Unit: extract through skip (4 tests)
// ---------------------------------------------------------------------------

#[test]
fn ut50_extract_through_box() {
    assert_extract(
        &parse_quote!(Box<Vec<String>>),
        "Vec",
        &skip(&["Box"]),
        "String",
        true,
    );
}

#[test]
fn ut51_extract_through_box_option() {
    assert_extract(
        &parse_quote!(Box<Option<i32>>),
        "Option",
        &skip(&["Box"]),
        "i32",
        true,
    );
}

#[test]
fn ut52_extract_from_non_path() {
    assert_extract(&parse_quote!(&str), "Vec", &HashSet::new(), "& str", false);
}

#[test]
fn ut53_extract_from_tuple() {
    assert_extract(
        &parse_quote!((i32, u32)),
        "Vec",
        &HashSet::new(),
        "(i32 , u32)",
        false,
    );
}
