#![allow(clippy::needless_range_loop)]

//! Property-based and unit tests for `filter_inner_type` in adze-common.

use std::collections::HashSet;

use adze_common::filter_inner_type;
use quote::ToTokens;
use syn::{Type, parse_quote};

/// Helper: render a `Type` to its token string for comparison.
fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

/// Helper: build a `HashSet<&str>` from a slice.
fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ===========================================================================
// 1. Filter removes skip-set types
// ===========================================================================

#[test]
fn filter_removes_box_wrapper() {
    let ty: Type = parse_quote!(Box<u32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "u32");
}

#[test]
fn filter_removes_arc_wrapper() {
    let ty: Type = parse_quote!(Arc<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Arc"]))), "String");
}

#[test]
fn filter_removes_rc_wrapper() {
    let ty: Type = parse_quote!(Rc<bool>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Rc"]))), "bool");
}

#[test]
fn filter_removes_cell_wrapper() {
    let ty: Type = parse_quote!(Cell<i64>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Cell"]))), "i64");
}

#[test]
fn filter_removes_mutex_wrapper() {
    let ty: Type = parse_quote!(Mutex<Data>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Mutex"]))), "Data");
}

// ===========================================================================
// 2. Filter preserves non-skip types
// ===========================================================================

#[test]
fn filter_preserves_plain_string() {
    let ty: Type = parse_quote!(String);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "String");
}

#[test]
fn filter_preserves_u64_with_box_skip() {
    let ty: Type = parse_quote!(u64);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "u64");
}

#[test]
fn filter_preserves_hashmap_not_in_skip() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Option"]))),
        "HashMap < String , i32 >"
    );
}

#[test]
fn filter_preserves_result_not_in_skip() {
    let ty: Type = parse_quote!(Result<(), Error>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Vec"]))),
        "Result < () , Error >"
    );
}

#[test]
fn filter_preserves_custom_generic_not_in_skip() {
    let ty: Type = parse_quote!(MyWrapper<Payload>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc"]))),
        "MyWrapper < Payload >"
    );
}

// ===========================================================================
// 3. Filter with empty skip set (identity)
// ===========================================================================

#[test]
fn filter_empty_skip_returns_box_unchanged() {
    let ty: Type = parse_quote!(Box<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&[]))), "Box < i32 >");
}

#[test]
fn filter_empty_skip_returns_vec_unchanged() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Vec < String >"
    );
}

#[test]
fn filter_empty_skip_returns_nested_unchanged() {
    let ty: Type = parse_quote!(Box<Option<Vec<u8>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Box < Option < Vec < u8 > > >"
    );
}

#[test]
fn filter_empty_skip_returns_plain_type_unchanged() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&[]))), "bool");
}

// ===========================================================================
// 4. Filter with Option type
// ===========================================================================

#[test]
fn filter_option_in_skip_unwraps_to_inner() {
    let ty: Type = parse_quote!(Option<u8>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Option"]))), "u8");
}

#[test]
fn filter_option_not_in_skip_preserved() {
    let ty: Type = parse_quote!(Option<u8>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Option < u8 >"
    );
}

#[test]
fn filter_option_wrapping_box_both_skipped() {
    let ty: Type = parse_quote!(Option<Box<Token>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Option", "Box"]))),
        "Token"
    );
}

// ===========================================================================
// 5. Filter with Vec type
// ===========================================================================

#[test]
fn filter_vec_in_skip_unwraps_to_inner() {
    let ty: Type = parse_quote!(Vec<Expr>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Vec"]))), "Expr");
}

#[test]
fn filter_vec_not_in_skip_preserved() {
    let ty: Type = parse_quote!(Vec<Expr>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Option"]))),
        "Vec < Expr >"
    );
}

#[test]
fn filter_vec_of_option_vec_skipped() {
    let ty: Type = parse_quote!(Vec<Option<Node>>);
    // Vec is skipped, revealing Option<Node>; Option is NOT skipped.
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Vec"]))),
        "Option < Node >"
    );
}

// ===========================================================================
// 6. Filter idempotent (filter twice == filter once)
// ===========================================================================

#[test]
fn filter_idempotent_box_string() {
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(Box<String>);
    let once = filter_inner_type(&ty, &s);
    let twice = filter_inner_type(&once, &s);
    assert_eq!(ty_str(&once), ty_str(&twice));
}

#[test]
fn filter_idempotent_nested_arc_box() {
    let s = skip(&["Arc", "Box"]);
    let ty: Type = parse_quote!(Arc<Box<Foo>>);
    let once = filter_inner_type(&ty, &s);
    let twice = filter_inner_type(&once, &s);
    assert_eq!(ty_str(&once), ty_str(&twice));
    assert_eq!(ty_str(&once), "Foo");
}

#[test]
fn filter_idempotent_plain_type() {
    let s = skip(&["Box", "Option", "Vec"]);
    let ty: Type = parse_quote!(i32);
    let once = filter_inner_type(&ty, &s);
    let twice = filter_inner_type(&once, &s);
    assert_eq!(ty_str(&once), ty_str(&twice));
}

#[test]
fn filter_idempotent_option_vec_string() {
    let s = skip(&["Option", "Vec"]);
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let once = filter_inner_type(&ty, &s);
    let twice = filter_inner_type(&once, &s);
    assert_eq!(ty_str(&once), "String");
    assert_eq!(ty_str(&once), ty_str(&twice));
}

// ===========================================================================
// 7. Filter with nested types
// ===========================================================================

#[test]
fn filter_three_layers_all_skipped() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Leaf>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]))),
        "Leaf"
    );
}

#[test]
fn filter_stops_at_first_non_skip_layer() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    // Box is skipped, Vec is NOT — should stop at Vec<i32>.
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < i32 >"
    );
}

#[test]
fn filter_nested_option_option_both_skipped() {
    let ty: Type = parse_quote!(Option<Option<bool>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Option"]))), "bool");
}

#[test]
fn filter_four_layers_mixed_skip() {
    // Skip Box and Rc but NOT Arc — should stop at Arc<Leaf>.
    let ty: Type = parse_quote!(Box<Rc<Arc<Leaf>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box", "Rc"]))),
        "Arc < Leaf >"
    );
}

// ===========================================================================
// 8. Filter determinism (same input always same output)
// ===========================================================================

#[test]
fn filter_deterministic_repeated_calls() {
    let s = skip(&["Box", "Option"]);
    let ty: Type = parse_quote!(Box<Option<Atom>>);
    let results: Vec<String> = (0..10)
        .map(|_| ty_str(&filter_inner_type(&ty, &s)))
        .collect();
    for i in 1..results.len() {
        assert_eq!(results[0], results[i]);
    }
    assert_eq!(results[0], "Atom");
}

#[test]
fn filter_deterministic_across_skip_set_constructions() {
    // Two independently constructed skip sets with same contents.
    let s1: HashSet<&str> = ["Box", "Arc"].iter().copied().collect();
    let s2: HashSet<&str> = ["Arc", "Box"].iter().copied().collect();
    let ty: Type = parse_quote!(Box<Arc<Value>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &s1)),
        ty_str(&filter_inner_type(&ty, &s2))
    );
}

#[test]
fn filter_deterministic_plain_type_repeated() {
    let s = skip(&["Box"]);
    let ty: Type = parse_quote!(f64);
    let a = ty_str(&filter_inner_type(&ty, &s));
    let b = ty_str(&filter_inner_type(&ty, &s));
    assert_eq!(a, b);
    assert_eq!(a, "f64");
}

// ===========================================================================
// Additional edge cases
// ===========================================================================

#[test]
fn filter_reference_type_passthrough() {
    let ty: Type = parse_quote!(&Box<u8>);
    // Reference is not a Path type at top level — returned as-is.
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "& Box < u8 >"
    );
}

#[test]
fn filter_tuple_type_passthrough() {
    let ty: Type = parse_quote!((i32, String));
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "(i32 , String)"
    );
}

#[test]
fn filter_array_type_passthrough() {
    let ty: Type = parse_quote!([u8; 32]);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Vec"]))),
        "[u8 ; 32]"
    );
}

#[test]
fn filter_qualified_path_option_unwraps() {
    let ty: Type = parse_quote!(core::option::Option<usize>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Option"]))), "usize");
}

#[test]
fn filter_skip_set_superset_of_present_wrappers() {
    // Skip set has many entries, but only Box is present.
    let s = skip(&["Box", "Arc", "Rc", "Cell", "Mutex", "Option", "Vec"]);
    let ty: Type = parse_quote!(Box<Data>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &s)), "Data");
}
