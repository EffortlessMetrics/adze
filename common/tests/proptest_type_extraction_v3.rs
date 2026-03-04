//! Property-based and unit tests for type extraction functions (v3).
//!
//! Tests `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! covering: nested generics, skip-over sets, identity properties, edge cases,
//! tuple types, reference types, deeply nested types, and no-panic guarantees.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_str};

// ===========================================================================
// Strategies
// ===========================================================================

fn leaf_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize", "MyStruct", "Ident", "Span",
        ][..],
    )
}

fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Vec", "Option", "Arc", "Rc"][..])
}

fn skip_set_strategy() -> impl Strategy<Value = HashSet<&'static str>> {
    prop::collection::hash_set(container_name(), 0..=5)
}

fn type_string_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        leaf_type_name().prop_map(|s| s.to_string()),
        (container_name(), leaf_type_name()).prop_map(|(c, l)| format!("{c}<{l}>")),
        (container_name(), container_name(), leaf_type_name())
            .prop_map(|(c1, c2, l)| format!("{c1}<{c2}<{l}>>")),
    ]
}

fn parseable_type(s: &str) -> Type {
    parse_str::<Type>(s).unwrap()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// Unit tests — try_extract_inner_type
// ===========================================================================

#[test]
fn extract_option_string() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("Option<String>");
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_i32() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("Vec<i32>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_box_u8() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("Box<u8>");
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_wrong_target_returns_false() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("Vec<String>");
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn extract_plain_type_returns_false() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("String");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_skip_box_then_option() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty = parseable_type("Box<Option<i64>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "i64");
}

#[test]
fn extract_skip_arc_then_vec() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty = parseable_type("Arc<Vec<bool>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_skip_multiple_then_target() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty = parseable_type("Box<Arc<Vec<f32>>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn extract_skip_no_match_returns_original() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty = parseable_type("Box<String>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_reference_type_returns_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("&str");
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn extract_tuple_type_returns_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("(i32, u32)");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(i32 , u32)");
}

#[test]
fn extract_rc_target() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("Rc<char>");
    let (inner, ok) = try_extract_inner_type(&ty, "Rc", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn extract_nested_option_option() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty = parseable_type("Option<Option<u8>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    // Outer Option matches target directly, inner is Option<u8>
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < u8 >");
}

// ===========================================================================
// Unit tests — filter_inner_type
// ===========================================================================

#[test]
fn filter_box_string() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty = parseable_type("Box<String>");
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_arc_i32() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty = parseable_type("Arc<i32>");
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_nested_box_arc() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty = parseable_type("Box<Arc<String>>");
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_non_skip_returns_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty = parseable_type("Vec<String>");
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "Vec < String >");
}

#[test]
fn filter_empty_skip_set() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("Box<i32>");
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "Box < i32 >");
}

#[test]
fn filter_plain_type_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty = parseable_type("String");
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_reference_type_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty = parseable_type("&u8");
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "& u8");
}

#[test]
fn filter_tuple_type_unchanged() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty = parseable_type("(bool, char)");
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "(bool , char)");
}

#[test]
fn filter_triple_nested() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty = parseable_type("Box<Arc<Rc<u64>>>");
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "u64");
}

#[test]
fn filter_partial_skip_stops_early() {
    // Only Box is in skip; Arc is not, so filtering stops at Arc.
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty = parseable_type("Box<Arc<String>>");
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "Arc < String >");
}

#[test]
fn filter_rc_wrapping_option() {
    let skip: HashSet<&str> = ["Rc"].into_iter().collect();
    let ty = parseable_type("Rc<Option<i32>>");
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "Option < i32 >");
}

// ===========================================================================
// Unit tests — wrap_leaf_type
// ===========================================================================

#[test]
fn wrap_plain_string() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("String");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_vec_string() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty = parseable_type("Vec<String>");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_option_i32() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty = parseable_type("Option<i32>");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Option < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_nested_vec_option() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty = parseable_type("Vec<Option<bool>>");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Vec < Option < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wrap_plain_i32_no_skip() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("i32");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_reference_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("&str");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_tuple_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("(u8, u16)");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < (u8 , u16) >"
    );
}

#[test]
fn wrap_array_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("[u8; 4]");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

#[test]
fn wrap_result_both_args() {
    let skip: HashSet<&str> = ["Result"].into_iter().collect();
    let ty = parseable_type("Result<String, i32>");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_vec_not_in_skip_wraps_whole() {
    // When Vec is NOT in skip set, the entire Vec<T> is wrapped.
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("Vec<String>");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < Vec < String > >"
    );
}

#[test]
fn wrap_box_in_skip_wraps_inner() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty = parseable_type("Box<f64>");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Box < adze :: WithLeaf < f64 > >"
    );
}

#[test]
fn wrap_deeply_nested_all_skip() {
    let skip: HashSet<&str> = ["Vec", "Option", "Box"].into_iter().collect();
    let ty = parseable_type("Vec<Option<Box<usize>>>");
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Vec < Option < Box < adze :: WithLeaf < usize > > > >"
    );
}

// ===========================================================================
// Unit tests — cross-function interactions
// ===========================================================================

#[test]
fn extract_then_filter_consistency() {
    // Extracting inner type of Vec gives Box<String>, then filtering Box gives String.
    let extract_skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("Vec<Box<String>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &extract_skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < String >");
    let filter_skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filtered = filter_inner_type(&inner, &filter_skip);
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_then_wrap_roundtrip() {
    // filter removes Box, wrap adds WithLeaf
    let filter_skip: HashSet<&str> = ["Box"].into_iter().collect();
    let wrap_skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("Box<i32>");
    let filtered = filter_inner_type(&ty, &filter_skip);
    let wrapped = wrap_leaf_type(&filtered, &wrap_skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn extract_not_found_then_filter_is_identity() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("String");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!ok);
    let filtered = filter_inner_type(&inner, &skip);
    assert_eq!(ty_str(&filtered), "String");
}

// ===========================================================================
// Unit tests — additional edge cases
// ===========================================================================

#[test]
fn extract_with_qualified_path_segment() {
    // std::vec::Vec<i32> — last segment is Vec
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("std::vec::Vec<i32>");
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn filter_with_qualified_path() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty = parseable_type("std::sync::Arc<u32>");
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&filtered), "u32");
}

#[test]
fn wrap_with_qualified_path_not_in_skip() {
    let skip: HashSet<&str> = HashSet::new();
    let ty = parseable_type("std::string::String");
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < std :: string :: String >"
    );
}

#[test]
fn extract_skip_rc_then_option() {
    let skip: HashSet<&str> = ["Rc"].into_iter().collect();
    let ty = parseable_type("Rc<Option<char>>");
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(ok);
    assert_eq!(ty_str(&inner), "char");
}

// ===========================================================================
// Proptest — try_extract_inner_type properties
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_never_panics_on_valid_types(
        type_str in type_string_strategy(),
        target in container_name(),
        skip in skip_set_strategy(),
    ) {
        let ty = parse_str::<Type>(&type_str).unwrap();
        let _ = try_extract_inner_type(&ty, target, &skip);
    }

    #[test]
    fn prop_extract_leaf_always_false(
        leaf in leaf_type_name(),
        target in container_name(),
        skip in skip_set_strategy(),
    ) {
        let ty = parse_str::<Type>(leaf).unwrap();
        let (_, ok) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(!ok, "leaf type {leaf} should never extract as {target}");
    }

    #[test]
    fn prop_extract_matching_target_is_true(
        target in container_name(),
        leaf in leaf_type_name(),
    ) {
        let skip: HashSet<&str> = HashSet::new();
        let type_str = format!("{target}<{leaf}>");
        let ty = parse_str::<Type>(&type_str).unwrap();
        let (inner, ok) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(ok, "direct target should always extract");
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    #[test]
    fn prop_extract_mismatched_target_is_false_on_leaf(
        leaf in leaf_type_name(),
    ) {
        let skip: HashSet<&str> = HashSet::new();
        let ty = parse_str::<Type>(leaf).unwrap();
        let (_, ok) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(!ok);
    }

    #[test]
    fn prop_extract_skip_then_target(
        wrapper in container_name(),
        target in container_name(),
        leaf in leaf_type_name(),
    ) {
        // When wrapper != target and wrapper is in skip set
        prop_assume!(wrapper != target);
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let type_str = format!("{wrapper}<{target}<{leaf}>>");
        let ty = parse_str::<Type>(&type_str).unwrap();
        let (inner, ok) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(ok, "should skip {wrapper} and extract {target}");
        prop_assert_eq!(ty_str(&inner), leaf);
    }
}

// ===========================================================================
// Proptest — filter_inner_type properties
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_never_panics(
        type_str in type_string_strategy(),
        skip in skip_set_strategy(),
    ) {
        let ty = parse_str::<Type>(&type_str).unwrap();
        let _ = filter_inner_type(&ty, &skip);
    }

    #[test]
    fn prop_filter_leaf_is_identity(
        leaf in leaf_type_name(),
        skip in skip_set_strategy(),
    ) {
        let ty = parse_str::<Type>(leaf).unwrap();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    #[test]
    fn prop_filter_empty_skip_is_identity(
        type_str in type_string_strategy(),
    ) {
        let skip: HashSet<&str> = HashSet::new();
        let ty = parse_str::<Type>(&type_str).unwrap();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    #[test]
    fn prop_filter_idempotent(
        type_str in type_string_strategy(),
        skip in skip_set_strategy(),
    ) {
        let ty = parse_str::<Type>(&type_str).unwrap();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice), "filter should be idempotent");
    }

    #[test]
    fn prop_filter_strips_single_container(
        container in container_name(),
        leaf in leaf_type_name(),
    ) {
        let skip: HashSet<&str> = [container].into_iter().collect();
        let type_str = format!("{container}<{leaf}>");
        let ty = parse_str::<Type>(&type_str).unwrap();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }
}

// ===========================================================================
// Proptest — wrap_leaf_type properties
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_never_panics(
        type_str in type_string_strategy(),
        skip in skip_set_strategy(),
    ) {
        let ty = parse_str::<Type>(&type_str).unwrap();
        let _ = wrap_leaf_type(&ty, &skip);
    }

    #[test]
    fn prop_wrap_leaf_always_has_with_leaf(
        leaf in leaf_type_name(),
    ) {
        let skip: HashSet<&str> = HashSet::new();
        let ty = parse_str::<Type>(leaf).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = ty_str(&wrapped);
        prop_assert!(s.contains("WithLeaf"), "leaf should be wrapped: {s}");
    }

    #[test]
    fn prop_wrap_container_in_skip_preserves_outer(
        container in container_name(),
        leaf in leaf_type_name(),
    ) {
        let skip: HashSet<&str> = [container].into_iter().collect();
        let type_str = format!("{container}<{leaf}>");
        let ty = parse_str::<Type>(&type_str).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = ty_str(&wrapped);
        prop_assert!(s.starts_with(container), "should preserve {container}: {s}");
        prop_assert!(s.contains("WithLeaf"), "inner should be wrapped: {s}");
    }

    #[test]
    fn prop_wrap_container_not_in_skip_wraps_whole(
        container in container_name(),
        leaf in leaf_type_name(),
    ) {
        let skip: HashSet<&str> = HashSet::new();
        let type_str = format!("{container}<{leaf}>");
        let ty = parse_str::<Type>(&type_str).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = ty_str(&wrapped);
        prop_assert!(
            s.starts_with("adze :: WithLeaf"),
            "whole type should be wrapped when not in skip: {s}"
        );
    }

    #[test]
    fn prop_wrap_output_is_parseable(
        type_str in type_string_strategy(),
        skip in skip_set_strategy(),
    ) {
        let ty = parse_str::<Type>(&type_str).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = ty_str(&wrapped);
        prop_assert!(
            parse_str::<Type>(&s).is_ok(),
            "wrapped type should be parseable: {s}"
        );
    }
}

// ===========================================================================
// Proptest — cross-function properties
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_found_then_filter_is_noop(
        target in container_name(),
        leaf in leaf_type_name(),
    ) {
        let skip: HashSet<&str> = HashSet::new();
        let type_str = format!("{target}<{leaf}>");
        let ty = parse_str::<Type>(&type_str).unwrap();
        let (inner, ok) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(ok);
        // The extracted leaf is not a container, so filtering with any skip set leaves it unchanged.
        let filtered = filter_inner_type(&inner, &skip);
        prop_assert_eq!(ty_str(&inner), ty_str(&filtered));
    }

    #[test]
    fn prop_filter_then_wrap_always_parseable(
        type_str in type_string_strategy(),
        filter_skip in skip_set_strategy(),
        wrap_skip in skip_set_strategy(),
    ) {
        let ty = parse_str::<Type>(&type_str).unwrap();
        let filtered = filter_inner_type(&ty, &filter_skip);
        let wrapped = wrap_leaf_type(&filtered, &wrap_skip);
        let s = ty_str(&wrapped);
        prop_assert!(
            parse_str::<Type>(&s).is_ok(),
            "filter+wrap output should be parseable: {s}"
        );
    }

    #[test]
    fn prop_extract_not_found_preserves_type(
        leaf in leaf_type_name(),
        target in container_name(),
        skip in skip_set_strategy(),
    ) {
        let ty = parse_str::<Type>(leaf).unwrap();
        let (returned, ok) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(!ok);
        prop_assert_eq!(ty_str(&returned), leaf);
    }
}
