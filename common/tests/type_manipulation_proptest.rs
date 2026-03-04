#![allow(clippy::needless_range_loop)]

//! Property-based tests for type manipulation functions in adze-common:
//! `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`.
//!
//! Covers extraction with various wrappers, filter with different skip sets,
//! wrap_leaf_type roundtrip properties, and edge cases with deep nesting.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
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
    prop::sample::select(&["Box", "Vec", "Option", "Arc", "Rc"][..])
}

fn extract_target() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Vec", "Option"][..])
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

fn skip<'a>(names: &[&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ===========================================================================
// 1. try_extract_inner_type: extraction succeeds when target matches
// ===========================================================================

proptest! {
    #[test]
    fn extract_finds_direct_target(
        target in extract_target(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&[]);
        let ty = parse_ty(&format!("{target}<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, target, &skip_set);
        prop_assert!(extracted, "should extract {target}<{leaf}>");
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    #[test]
    fn extract_returns_false_when_target_absent(
        target in extract_target(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&[]);
        let ty = parse_ty(leaf);
        let (_inner, extracted) = try_extract_inner_type(&ty, target, &skip_set);
        prop_assert!(!extracted, "plain type has no {target} to extract");
    }

    #[test]
    fn extract_returns_original_when_wrong_wrapper(
        target in extract_target(),
        leaf in leaf_name(),
    ) {
        // Use a wrapper that is NOT the target and NOT in skip set
        let skip_set = skip(&[]);
        let other = if target == "Vec" { "Option" } else { "Vec" };
        let ty = parse_ty(&format!("{other}<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, target, &skip_set);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }
}

// ===========================================================================
// 2. try_extract_inner_type: skipping through containers
// ===========================================================================

proptest! {
    #[test]
    fn extract_skips_box_to_find_target(
        target in extract_target(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&["Box"]);
        let ty = parse_ty(&format!("Box<{target}<{leaf}>>"));
        let (inner, extracted) = try_extract_inner_type(&ty, target, &skip_set);
        prop_assert!(extracted, "should skip Box and find {target}");
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    #[test]
    fn extract_skips_arc_to_find_target(
        target in extract_target(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&["Arc"]);
        let ty = parse_ty(&format!("Arc<{target}<{leaf}>>"));
        let (inner, extracted) = try_extract_inner_type(&ty, target, &skip_set);
        prop_assert!(extracted, "should skip Arc and find {target}");
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    #[test]
    fn extract_skip_but_target_not_inside(
        leaf in leaf_name(),
    ) {
        // Box<String> with skip=Box, target=Vec → not found, returns original
        let skip_set = skip(&["Box"]);
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }
}

// ===========================================================================
// 3. try_extract_inner_type: non-path types
// ===========================================================================

proptest! {
    #[test]
    fn extract_non_path_returns_unchanged(
        target in extract_target(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&[]);
        let ty = parse_ty(&format!("&{leaf}"));
        let (inner, extracted) = try_extract_inner_type(&ty, target, &skip_set);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }
}

// ===========================================================================
// 4. try_extract_inner_type: determinism
// ===========================================================================

proptest! {
    #[test]
    fn extract_deterministic(
        target in extract_target(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&["Box"]);
        let ty = parse_ty(&format!("Box<{target}<{leaf}>>"));
        let (a, ea) = try_extract_inner_type(&ty, target, &skip_set);
        let (b, eb) = try_extract_inner_type(&ty, target, &skip_set);
        prop_assert_eq!(ea, eb);
        prop_assert_eq!(ty_str(&a), ty_str(&b));
    }
}

// ===========================================================================
// 5. filter_inner_type: removes skip-set wrappers
// ===========================================================================

proptest! {
    #[test]
    fn filter_removes_single_wrapper(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&[wrapper]);
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    #[test]
    fn filter_preserves_non_skip_wrapper(
        leaf in leaf_name(),
    ) {
        // Box not in skip set → preserved
        let skip_set = skip(&["Vec", "Option"]);
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(ty_str(&filtered), format!("Box < {leaf} >"));
    }

    #[test]
    fn filter_plain_type_unchanged(
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&["Box", "Vec", "Option"]);
        let ty = parse_ty(leaf);
        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }
}

// ===========================================================================
// 6. filter_inner_type: nested unwrapping
// ===========================================================================

proptest! {
    #[test]
    fn filter_removes_two_nested_wrappers(leaf in leaf_name()) {
        let skip_set = skip(&["Box", "Arc"]);
        let ty = parse_ty(&format!("Box<Arc<{leaf}>>"));
        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    #[test]
    fn filter_stops_at_first_non_skip(leaf in leaf_name()) {
        // Box is skipped, Vec is NOT → result is Vec<leaf>
        let skip_set = skip(&["Box"]);
        let ty = parse_ty(&format!("Box<Vec<{leaf}>>"));
        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(ty_str(&filtered), format!("Vec < {leaf} >"));
    }

    #[test]
    fn filter_three_layers_all_skipped(leaf in leaf_name()) {
        let skip_set = skip(&["Box", "Arc", "Rc"]);
        let ty = parse_ty(&format!("Box<Arc<Rc<{leaf}>>>"));
        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }
}

// ===========================================================================
// 7. filter_inner_type: idempotence
// ===========================================================================

proptest! {
    #[test]
    fn filter_idempotent(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&[wrapper]);
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let once = filter_inner_type(&ty, &skip_set);
        let twice = filter_inner_type(&once, &skip_set);
        prop_assert_eq!(ty_str(&once), ty_str(&twice), "filter should be idempotent");
    }

    #[test]
    fn filter_empty_skip_is_identity(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip_set: HashSet<&str> = HashSet::new();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty), "empty skip = identity");
    }
}

// ===========================================================================
// 8. wrap_leaf_type: wraps leaf in adze::WithLeaf
// ===========================================================================

proptest! {
    #[test]
    fn wrap_bare_type_wraps_with_leaf(leaf in leaf_name()) {
        let skip_set: HashSet<&str> = HashSet::new();
        let ty = parse_ty(leaf);
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert_eq!(wrapped, format!("adze :: WithLeaf < {leaf} >"));
    }

    #[test]
    fn wrap_skip_container_preserves_outer(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&[wrapper]);
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert!(wrapped.starts_with(&format!("{wrapper} <")), "outer preserved, got: {wrapped}");
        prop_assert!(wrapped.contains("adze :: WithLeaf"), "leaf wrapped, got: {wrapped}");
    }

    #[test]
    fn wrap_non_skip_container_wraps_whole(
        leaf in leaf_name(),
    ) {
        // Box not in skip → entire Box<leaf> is wrapped
        let skip_set = skip(&["Vec", "Option"]);
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert!(wrapped.starts_with("adze :: WithLeaf <"), "whole type wrapped, got: {wrapped}");
    }
}

// ===========================================================================
// 9. wrap_leaf_type: nested skip containers
// ===========================================================================

proptest! {
    #[test]
    fn wrap_nested_skip_preserves_all_containers(leaf in leaf_name()) {
        let skip_set = skip(&["Option", "Vec"]);
        let ty = parse_ty(&format!("Option<Vec<{leaf}>>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert!(wrapped.contains("Option"), "Option preserved");
        prop_assert!(wrapped.contains("Vec"), "Vec preserved");
        prop_assert!(wrapped.contains(&format!("adze :: WithLeaf < {leaf} >")), "leaf wrapped");
    }

    #[test]
    fn wrap_triple_nested_all_skipped(leaf in leaf_name()) {
        let skip_set = skip(&["Option", "Vec", "Box"]);
        let ty = parse_ty(&format!("Option<Vec<Box<{leaf}>>>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert!(wrapped.starts_with("Option <"));
        prop_assert!(wrapped.contains("Vec <"));
        prop_assert!(wrapped.contains("Box <"));
        let expected_leaf = format!("adze :: WithLeaf < {} >", leaf);
        prop_assert!(wrapped.contains(&expected_leaf));
    }
}

// ===========================================================================
// 10. wrap_leaf_type: output is always valid Rust type
// ===========================================================================

proptest! {
    #[test]
    fn wrap_output_parses_as_valid_type(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&[wrapper]);
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert!(parse_str::<Type>(&wrapped).is_ok(), "must be valid type: {wrapped}");
    }

    #[test]
    fn wrap_bare_output_parses(leaf in leaf_name()) {
        let skip_set: HashSet<&str> = HashSet::new();
        let ty = parse_ty(leaf);
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert!(parse_str::<Type>(&wrapped).is_ok(), "must be valid type: {wrapped}");
    }
}

// ===========================================================================
// 11. wrap_leaf_type: determinism
// ===========================================================================

proptest! {
    #[test]
    fn wrap_deterministic(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&[wrapper]);
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let a = ty_str(&wrap_leaf_type(&ty, &skip_set));
        let b = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert_eq!(a, b);
    }
}

// ===========================================================================
// 12. Composability: filter then extract
// ===========================================================================

proptest! {
    #[test]
    fn filter_then_extract_on_nested(
        target in extract_target(),
        leaf in leaf_name(),
    ) {
        // Box<Vec<leaf>> → filter(Box) → Vec<leaf> → extract(Vec) → leaf
        let filter_skip = skip(&["Box"]);
        let extract_skip = skip(&[]);
        let ty = parse_ty(&format!("Box<{target}<{leaf}>>"));
        let filtered = filter_inner_type(&ty, &filter_skip);
        let (inner, extracted) = try_extract_inner_type(&filtered, target, &extract_skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }
}

// ===========================================================================
// 13. extract + wrap consistency: wrap always contains leaf name
// ===========================================================================

proptest! {
    #[test]
    fn wrap_always_preserves_leaf_name(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&[wrapper]);
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert!(wrapped.contains(leaf), "leaf name preserved in: {wrapped}");
    }
}

// ===========================================================================
// 14. Edge case: deeply nested (4 levels)
// ===========================================================================

proptest! {
    #[test]
    fn extract_through_multiple_skip_layers(leaf in leaf_name()) {
        let skip_set = skip(&["Box", "Arc"]);
        let ty = parse_ty(&format!("Box<Arc<Vec<{leaf}>>>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set);
        prop_assert!(extracted, "should skip Box+Arc and find Vec");
        prop_assert_eq!(ty_str(&inner), leaf);
    }

    #[test]
    fn filter_deeply_nested_removes_all_skip(leaf in leaf_name()) {
        let skip_set = skip(&["Box", "Arc", "Rc", "Option"]);
        let ty = parse_ty(&format!("Box<Arc<Rc<Option<{leaf}>>>>"));
        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }
}

// ===========================================================================
// 15. Edge case: same wrapper used as both skip and target
// ===========================================================================

proptest! {
    #[test]
    fn extract_target_also_in_skip_extracts_inner(leaf in leaf_name()) {
        // Vec is both target and in skip set; first match as target wins
        let skip_set = skip(&["Vec"]);
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set);
        // The function checks target first, so it extracts
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }
}

// ===========================================================================
// 16. Non-path types pass through all functions
// ===========================================================================

proptest! {
    #[test]
    fn all_functions_handle_reference_types(leaf in leaf_name()) {
        let skip_set = skip(&["Box", "Vec"]);
        let ty = parse_ty(&format!("&{leaf}"));

        let (extract_result, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&extract_result), ty_str(&ty));

        let filtered = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));

        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert!(wrapped.starts_with("adze :: WithLeaf <"));
    }
}

// ===========================================================================
// 17. wrap_leaf_type: output always contains "adze :: WithLeaf"
// ===========================================================================

proptest! {
    #[test]
    fn wrap_output_always_has_with_leaf(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        // Whether wrapper is in skip or not, WithLeaf must appear somewhere
        let skip_set = skip(&[wrapper]);
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert!(wrapped.contains("adze :: WithLeaf"), "must contain WithLeaf: {wrapped}");
    }

    #[test]
    fn wrap_empty_skip_still_has_with_leaf(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip_set: HashSet<&str> = HashSet::new();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert!(wrapped.contains("adze :: WithLeaf"), "must contain WithLeaf: {wrapped}");
    }
}

// ===========================================================================
// 18. filter followed by wrap: filtered type gets wrapped
// ===========================================================================

proptest! {
    #[test]
    fn filter_then_wrap_produces_wrapped_leaf(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip_set = skip(&[wrapper]);
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let filtered = filter_inner_type(&ty, &skip_set);
        // filtered should be the leaf
        prop_assert_eq!(ty_str(&filtered), leaf);
        // wrapping with empty skip should give WithLeaf<leaf>
        let empty_skip: HashSet<&str> = HashSet::new();
        let wrapped = ty_str(&wrap_leaf_type(&filtered, &empty_skip));
        prop_assert_eq!(wrapped, format!("adze :: WithLeaf < {leaf} >"));
    }
}

// ===========================================================================
// 19. extract returns original type unchanged when not extracted
// ===========================================================================

proptest! {
    #[test]
    fn extract_not_found_returns_exact_original(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip_set: HashSet<&str> = HashSet::new();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let (result, extracted) = try_extract_inner_type(&ty, "NonExistent", &skip_set);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&result), ty_str(&ty));
    }
}

// ===========================================================================
// 20. Partial skip in wrap: outer skipped, inner not
// ===========================================================================

proptest! {
    #[test]
    fn wrap_partial_skip_outer_only(leaf in leaf_name()) {
        // Option is skipped, Box is NOT → Box<leaf> gets wrapped as a whole
        let skip_set = skip(&["Option"]);
        let ty = parse_ty(&format!("Option<Box<{leaf}>>"));
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip_set));
        prop_assert!(wrapped.starts_with("Option <"), "outer preserved: {wrapped}");
        prop_assert!(wrapped.contains("adze :: WithLeaf < Box"), "inner wrapped as whole: {wrapped}");
    }
}
