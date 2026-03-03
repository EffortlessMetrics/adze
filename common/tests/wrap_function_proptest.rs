#![allow(clippy::needless_range_loop)]

//! Property-based tests for `wrap_leaf_type` in adze-common.
//!
//! Covers: wrapping plain types in Option/Vec/Box, double wrapping,
//! skip-set interaction, inner-type preservation, valid Rust output,
//! and determinism.

use adze_common::wrap_leaf_type;
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn leaf_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i32", "u64", "f64", "bool", "char", "String", "usize", "Foo", "Bar", "Token",
        ][..],
    )
}

fn wrapper_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Vec", "Option", "Arc", "Rc", "Cell", "Mutex"][..])
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn parse_ty(s: &str) -> Type {
    parse_str(s).unwrap()
}

// ===========================================================================
// 1. Wrap plain type in Option (skip set contains Option)
// ===========================================================================

proptest! {
    #[test]
    fn wrap_plain_in_option_preserves_option(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option"].into_iter().collect();
        let ty = parse_ty(&format!("Option<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("Option <"), "expected Option wrapper, got: {wrapped}");
        prop_assert!(wrapped.contains("adze :: WithLeaf"), "leaf should be wrapped, got: {wrapped}");
    }

    #[test]
    fn wrap_plain_not_in_skip_wraps_entire_option(leaf in leaf_name()) {
        let skip: HashSet<&str> = HashSet::new();
        let ty = parse_ty(&format!("Option<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("adze :: WithLeaf <"), "entire type should be wrapped, got: {wrapped}");
    }
}

// ===========================================================================
// 2. Wrap plain type in Vec (skip set contains Vec)
// ===========================================================================

proptest! {
    #[test]
    fn wrap_plain_in_vec_preserves_vec(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("Vec <"), "expected Vec wrapper, got: {wrapped}");
        prop_assert!(wrapped.contains("adze :: WithLeaf"), "leaf should be wrapped, got: {wrapped}");
    }

    #[test]
    fn wrap_plain_not_in_skip_wraps_entire_vec(leaf in leaf_name()) {
        let skip: HashSet<&str> = HashSet::new();
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("adze :: WithLeaf <"), "entire type should be wrapped, got: {wrapped}");
    }
}

// ===========================================================================
// 3. Wrap plain type in Box (skip set contains Box)
// ===========================================================================

proptest! {
    #[test]
    fn wrap_plain_in_box_preserves_box(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("Box <"), "expected Box wrapper, got: {wrapped}");
        prop_assert!(wrapped.contains("adze :: WithLeaf"), "leaf should be wrapped, got: {wrapped}");
    }

    #[test]
    fn wrap_bare_type_without_box(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let ty = parse_ty(leaf);
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(wrapped, format!("adze :: WithLeaf < {leaf} >"));
    }
}

// ===========================================================================
// 4. Double wrap (already-wrapped type)
// ===========================================================================

proptest! {
    #[test]
    fn double_wrap_option_vec(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
        let ty = parse_ty(&format!("Option<Vec<{leaf}>>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("Option <"), "outer Option preserved, got: {wrapped}");
        prop_assert!(wrapped.contains("Vec <"), "inner Vec preserved, got: {wrapped}");
        prop_assert!(wrapped.contains("adze :: WithLeaf"), "leaf wrapped, got: {wrapped}");
    }

    #[test]
    fn double_wrap_vec_box(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Vec", "Box"].into_iter().collect();
        let ty = parse_ty(&format!("Vec<Box<{leaf}>>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("Vec <"), "outer Vec preserved, got: {wrapped}");
        prop_assert!(wrapped.contains("Box <"), "inner Box preserved, got: {wrapped}");
        prop_assert!(wrapped.contains("adze :: WithLeaf"), "leaf wrapped, got: {wrapped}");
    }

    #[test]
    fn double_wrap_box_option(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Box", "Option"].into_iter().collect();
        let ty = parse_ty(&format!("Box<Option<{leaf}>>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("Box <"), "outer Box preserved, got: {wrapped}");
        prop_assert!(wrapped.contains("Option <"), "inner Option preserved, got: {wrapped}");
        prop_assert!(wrapped.contains(&format!("adze :: WithLeaf < {leaf} >")), "leaf wrapped, got: {wrapped}");
    }
}

// ===========================================================================
// 5. Wrap with skip set containing the wrapper
// ===========================================================================

proptest! {
    #[test]
    fn skip_containing_wrapper_preserves_container(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with(&format!("{wrapper} <")), "container preserved, got: {wrapped}");
        prop_assert!(wrapped.contains("adze :: WithLeaf"), "leaf wrapped, got: {wrapped}");
    }

    #[test]
    fn skip_not_containing_wrapper_wraps_whole(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = HashSet::new();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("adze :: WithLeaf <"), "whole type wrapped, got: {wrapped}");
    }

    #[test]
    fn skip_with_different_wrapper_wraps_whole(leaf in leaf_name()) {
        // Skip contains Vec but we wrap with Box → Box is not skipped
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("adze :: WithLeaf <"), "Box not in skip, got: {wrapped}");
    }
}

// ===========================================================================
// 6. Wrap preserves inner type
// ===========================================================================

proptest! {
    #[test]
    fn wrap_preserves_leaf_name(leaf in leaf_name()) {
        let skip: HashSet<&str> = HashSet::new();
        let ty = parse_ty(leaf);
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains(leaf), "leaf name must appear in output, got: {wrapped}");
    }

    #[test]
    fn wrap_skip_preserves_inner_leaf(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains(leaf), "inner leaf name preserved, got: {wrapped}");
    }

    #[test]
    fn wrap_nested_preserves_all_names(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
        let ty = parse_ty(&format!("Option<Vec<{leaf}>>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains("Option"), "Option preserved, got: {wrapped}");
        prop_assert!(wrapped.contains("Vec"), "Vec preserved, got: {wrapped}");
        prop_assert!(wrapped.contains(leaf), "leaf preserved, got: {wrapped}");
    }
}

// ===========================================================================
// 7. Wrap output is valid Rust type
// ===========================================================================

proptest! {
    #[test]
    fn wrap_output_parses_as_type_plain(leaf in leaf_name()) {
        let skip: HashSet<&str> = HashSet::new();
        let ty = parse_ty(leaf);
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(parse_str::<Type>(&wrapped).is_ok(), "unparseable: {wrapped}");
    }

    #[test]
    fn wrap_output_parses_as_type_container(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(parse_str::<Type>(&wrapped).is_ok(), "unparseable: {wrapped}");
    }

    #[test]
    fn wrap_output_parses_as_type_nested(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
        let ty = parse_ty(&format!("Option<Vec<{leaf}>>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(parse_str::<Type>(&wrapped).is_ok(), "unparseable: {wrapped}");
    }

    #[test]
    fn wrap_output_parses_empty_skip(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = HashSet::new();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(parse_str::<Type>(&wrapped).is_ok(), "unparseable: {wrapped}");
    }

    #[test]
    fn wrap_output_nonempty(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(!wrapped.is_empty(), "wrap output must not be empty");
    }
}

// ===========================================================================
// 8. Wrap determinism
// ===========================================================================

proptest! {
    #[test]
    fn wrap_deterministic_plain(leaf in leaf_name()) {
        let skip: HashSet<&str> = HashSet::new();
        let ty = parse_ty(leaf);
        let a = ty_str(&wrap_leaf_type(&ty, &skip));
        let b = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(a, b, "wrap must be deterministic");
    }

    #[test]
    fn wrap_deterministic_container(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let a = ty_str(&wrap_leaf_type(&ty, &skip));
        let b = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(a, b, "wrap must be deterministic");
    }

    #[test]
    fn wrap_deterministic_nested(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option", "Vec", "Box"].into_iter().collect();
        let ty = parse_ty(&format!("Option<Vec<Box<{leaf}>>>"));
        let a = ty_str(&wrap_leaf_type(&ty, &skip));
        let b = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(a, b, "wrap must be deterministic for nested types");
    }

    #[test]
    fn wrap_deterministic_empty_skip_container(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = HashSet::new();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let a = ty_str(&wrap_leaf_type(&ty, &skip));
        let b = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(a, b, "wrap must be deterministic with empty skip");
    }
}

// ===========================================================================
// Additional coverage
// ===========================================================================

proptest! {
    /// Triple nesting: all three containers in skip set
    #[test]
    fn triple_wrap_all_skipped(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option", "Vec", "Box"].into_iter().collect();
        let ty = parse_ty(&format!("Option<Vec<Box<{leaf}>>>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("Option <"), "outer Option, got: {wrapped}");
        prop_assert!(wrapped.contains("Vec <"), "middle Vec, got: {wrapped}");
        prop_assert!(wrapped.contains("Box <"), "inner Box, got: {wrapped}");
        prop_assert!(wrapped.contains(&format!("adze :: WithLeaf < {leaf} >")), "leaf wrapped, got: {wrapped}");
    }

    /// Wrapping a bare type (no container) always produces exactly `adze::WithLeaf<T>`
    #[test]
    fn wrap_bare_type_exact_output(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option", "Vec", "Box"].into_iter().collect();
        let ty = parse_ty(leaf);
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(wrapped, format!("adze :: WithLeaf < {leaf} >"));
    }

    /// Non-path type (reference) gets wrapped entirely
    #[test]
    fn wrap_reference_type_wraps_entirely(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option"].into_iter().collect();
        let ty: Type = syn::parse_str(&format!("&{leaf}")).unwrap();
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("adze :: WithLeaf <"), "ref type wrapped entirely, got: {wrapped}");
        prop_assert!(parse_str::<Type>(&wrapped).is_ok(), "output parseable, got: {wrapped}");
    }

    /// Partial skip: outer in skip, inner not → outer preserved, inner+leaf wrapped together
    #[test]
    fn partial_skip_outer_only(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option"].into_iter().collect();
        let ty = parse_ty(&format!("Option<Box<{leaf}>>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.starts_with("Option <"), "outer preserved, got: {wrapped}");
        // Box is not in skip, so the whole Box<leaf> gets wrapped as a leaf
        prop_assert!(wrapped.contains("adze :: WithLeaf < Box"), "Box wrapped as leaf, got: {wrapped}");
    }

    /// Wrapping the same type twice yields identical results (idempotent input check)
    #[test]
    fn wrap_called_five_times_same_result(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let results: Vec<String> = (0..5).map(|_| ty_str(&wrap_leaf_type(&ty, &skip))).collect();
        for i in 1..results.len() {
            prop_assert_eq!(&results[0], &results[i], "call {} differs", i);
        }
    }

    /// Wrap output always contains "adze :: WithLeaf" somewhere
    #[test]
    fn wrap_always_contains_with_leaf(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains("adze :: WithLeaf"), "must contain WithLeaf, got: {wrapped}");
    }
}
