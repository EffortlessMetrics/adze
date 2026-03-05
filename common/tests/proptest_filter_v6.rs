//! Property-based tests for type filtering in adze-common.
//!
//! Covers `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! across varied base types, wrapper types, skip sets, and compositions.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_str};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

fn skip_set(names: &[&'static str]) -> HashSet<&'static str> {
    names.iter().copied().collect()
}

fn parse_ty(s: &str) -> Type {
    parse_str(s).unwrap()
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn base_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["i32", "u32", "String", "bool", "f64", "u8", "i64"][..])
}

fn wrapper_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Vec", "Option", "Box"][..])
}

fn two_distinct_wrappers() -> impl Strategy<Value = (&'static str, &'static str)> {
    prop::sample::select(
        &[
            ("Vec", "Option"),
            ("Vec", "Box"),
            ("Option", "Vec"),
            ("Option", "Box"),
            ("Box", "Vec"),
            ("Box", "Option"),
        ][..],
    )
}

// ===========================================================================
// 1. Extract from plain type with inner_of="Vec" → (type, false)
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_plain_with_vec_returns_false(base in base_type()) {
        let ty = parse_ty(base);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), base);
    }

    #[test]
    fn prop_extract_plain_with_option_returns_false(base in base_type()) {
        let ty = parse_ty(base);
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &empty_skip());
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), base);
    }

    #[test]
    fn prop_extract_plain_with_box_returns_false(base in base_type()) {
        let ty = parse_ty(base);
        let (inner, extracted) = try_extract_inner_type(&ty, "Box", &empty_skip());
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), base);
    }
}

// ===========================================================================
// 2. Filter with empty skip → type unchanged
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_empty_skip_plain_unchanged(base in base_type()) {
        let ty = parse_ty(base);
        let filtered = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(ty_str(&filtered), base);
    }

    #[test]
    fn prop_filter_empty_skip_wrapped_unchanged(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let filtered = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }
}

// ===========================================================================
// 3. Wrap then filter with wrapper in skip → recovers original approximately
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_then_filter_recovers_base(base in base_type()) {
        // Wrapping a plain type gives adze::WithLeaf<T>. The "inner" type of that
        // is not in any skip set, so filter returns unchanged. But we can verify
        // the round-trip for container types.
        let ty = parse_ty(base);
        let skip = skip_set(&["Vec", "Option"]);
        let wrapped = wrap_leaf_type(&ty, &skip);
        let ws = ty_str(&wrapped);
        prop_assert!(ws.contains("WithLeaf"), "Expected WithLeaf in: {}", ws);
        prop_assert!(ws.contains(base), "Expected base type {} in: {}", base, ws);
    }

    #[test]
    fn prop_wrap_container_then_filter_inner_matches(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let skip = skip_set(&[w]);
        let wrapped = wrap_leaf_type(&ty, &skip);
        let ws = ty_str(&wrapped);
        // The wrapper should be preserved, inner should have WithLeaf
        prop_assert!(ws.starts_with(w), "Expected wrapper {w} in: {ws}");
        prop_assert!(ws.contains("WithLeaf"), "Expected WithLeaf in: {ws}");
        prop_assert!(ws.contains(base), "Expected base {base} in: {ws}");
    }
}

// ===========================================================================
// 4. Extract Vec<T> with "Vec" always returns (T, true)
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_vec_returns_inner(base in base_type()) {
        let ty = parse_ty(&format!("Vec<{base}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), base);
    }
}

// ===========================================================================
// 5. Extract Option<T> with "Option" always returns (T, true)
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_option_returns_inner(base in base_type()) {
        let ty = parse_ty(&format!("Option<{base}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &empty_skip());
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), base);
    }
}

// ===========================================================================
// 6. Filter with skip={"Vec"} on Vec<T> → T
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_vec_skip_unwraps(base in base_type()) {
        let ty = parse_ty(&format!("Vec<{base}>"));
        let filtered = filter_inner_type(&ty, &skip_set(&["Vec"]));
        prop_assert_eq!(ty_str(&filtered), base);
    }

    #[test]
    fn prop_filter_option_skip_unwraps(base in base_type()) {
        let ty = parse_ty(&format!("Option<{base}>"));
        let filtered = filter_inner_type(&ty, &skip_set(&["Option"]));
        prop_assert_eq!(ty_str(&filtered), base);
    }

    #[test]
    fn prop_filter_box_skip_unwraps(base in base_type()) {
        let ty = parse_ty(&format!("Box<{base}>"));
        let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
        prop_assert_eq!(ty_str(&filtered), base);
    }
}

// ===========================================================================
// 7. Different base types all preserve under empty skip filter
// ===========================================================================

proptest! {
    #[test]
    fn prop_all_bases_preserved_under_empty_filter(base in base_type()) {
        let ty = parse_ty(base);
        let filtered = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }
}

// ===========================================================================
// 8. Wrap always produces non-empty type string
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_leaf_produces_nonempty(base in base_type()) {
        let ty = parse_ty(base);
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        prop_assert!(!ty_str(&wrapped).is_empty());
    }

    #[test]
    fn prop_wrap_container_produces_nonempty(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let wrapped = wrap_leaf_type(&ty, &skip_set(&[w]));
        prop_assert!(!ty_str(&wrapped).is_empty());
    }
}

// ===========================================================================
// 9. Extract with wrong inner_of → false
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_vec_with_option_target_returns_false(base in base_type()) {
        let ty = parse_ty(&format!("Vec<{base}>"));
        let (_, extracted) = try_extract_inner_type(&ty, "Option", &empty_skip());
        prop_assert!(!extracted);
    }

    #[test]
    fn prop_extract_option_with_vec_target_returns_false(base in base_type()) {
        let ty = parse_ty(&format!("Option<{base}>"));
        let (_, extracted) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        prop_assert!(!extracted);
    }

    #[test]
    fn prop_extract_box_with_vec_target_returns_false(base in base_type()) {
        let ty = parse_ty(&format!("Box<{base}>"));
        let (_, extracted) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        prop_assert!(!extracted);
    }

    #[test]
    fn prop_extract_wrong_target_preserves_original(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty_s = format!("{w}<{base}>");
        let ty = parse_ty(&ty_s);
        // Use a target that does not match the wrapper
        let wrong = if w == "Vec" { "Option" } else { "Vec" };
        let (inner, extracted) = try_extract_inner_type(&ty, wrong, &empty_skip());
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }
}

// ===========================================================================
// 10. Filter with irrelevant skip → unchanged
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_irrelevant_skip_on_plain(base in base_type()) {
        let ty = parse_ty(base);
        let filtered = filter_inner_type(&ty, &skip_set(&["Arc", "Rc", "Mutex"]));
        prop_assert_eq!(ty_str(&filtered), base);
    }

    #[test]
    fn prop_filter_irrelevant_skip_on_container(
        w in wrapper_type(),
        base in base_type(),
    ) {
        // Skip set contains only types that don't match the wrapper
        let irrelevant = skip_set(&["Cow", "Pin", "Mutex"]);
        let ty = parse_ty(&format!("{w}<{base}>"));
        let filtered = filter_inner_type(&ty, &irrelevant);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }
}

// ===========================================================================
// 11. Wrap with empty skip wraps whole type
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_empty_skip_wraps_plain(base in base_type()) {
        let ty = parse_ty(base);
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        let expected = format!("adze :: WithLeaf < {base} >");
        prop_assert_eq!(ty_str(&wrapped), expected);
    }

    #[test]
    fn prop_wrap_empty_skip_wraps_container_entirely(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty_s = format!("{w}<{base}>");
        let ty = parse_ty(&ty_s);
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        let ws = ty_str(&wrapped);
        // Entire container should be inside WithLeaf
        prop_assert!(ws.starts_with("adze :: WithLeaf <"),
            "Expected full wrap, got: {ws}");
    }
}

// ===========================================================================
// 12. Determinism: same input → same output
// ===========================================================================

proptest! {
    #[test]
    fn prop_determinism_extract(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let (r1, e1) = try_extract_inner_type(&ty, w, &empty_skip());
        let (r2, e2) = try_extract_inner_type(&ty, w, &empty_skip());
        prop_assert_eq!(e1, e2);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }

    #[test]
    fn prop_determinism_filter(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let skip = skip_set(&[w]);
        let r1 = ty_str(&filter_inner_type(&ty, &skip));
        let r2 = ty_str(&filter_inner_type(&ty, &skip));
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn prop_determinism_wrap(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let skip = skip_set(&[w]);
        let r1 = ty_str(&wrap_leaf_type(&ty, &skip));
        let r2 = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn prop_determinism_plain_all_ops(base in base_type()) {
        let ty = parse_ty(base);
        let skip = skip_set(&["Vec"]);

        let (e1, x1) = try_extract_inner_type(&ty, "Vec", &skip);
        let (e2, x2) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert_eq!(x1, x2);
        prop_assert_eq!(ty_str(&e1), ty_str(&e2));

        let f1 = ty_str(&filter_inner_type(&ty, &skip));
        let f2 = ty_str(&filter_inner_type(&ty, &skip));
        prop_assert_eq!(f1, f2);

        let w1 = ty_str(&wrap_leaf_type(&ty, &skip));
        let w2 = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(w1, w2);
    }
}

// ===========================================================================
// 13. Extract through skip layers
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_through_skip_layer(
        (outer, inner) in two_distinct_wrappers(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{outer}<{inner}<{base}>>"));
        let skip = skip_set(&[outer]);
        let (result, extracted) = try_extract_inner_type(&ty, inner, &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), base);
    }
}

// ===========================================================================
// 14. Filter nested skip layers unwraps all
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_double_skip_unwraps_to_base(
        (outer, inner) in two_distinct_wrappers(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{outer}<{inner}<{base}>>"));
        let skip = skip_set(&[outer, inner]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), base);
    }
}

// ===========================================================================
// 15. Wrap nested containers preserves structure
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_nested_preserves_both_containers(
        (outer, inner) in two_distinct_wrappers(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{outer}<{inner}<{base}>>"));
        let skip = skip_set(&[outer, inner]);
        let wrapped = wrap_leaf_type(&ty, &skip);
        let ws = ty_str(&wrapped);
        prop_assert!(ws.starts_with(outer),
            "Expected outer {outer}, got: {ws}");
        prop_assert!(ws.contains(inner),
            "Expected inner {inner}, got: {ws}");
        prop_assert!(ws.contains("WithLeaf"),
            "Expected WithLeaf, got: {ws}");
        prop_assert!(ws.contains(base),
            "Expected base {base}, got: {ws}");
    }
}

// ===========================================================================
// 16. Extract returns original on non-matching wrapper
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_nonmatch_returns_original_str(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty_s = format!("{w}<{base}>");
        let ty = parse_ty(&ty_s);
        let (inner, extracted) = try_extract_inner_type(&ty, "HashMap", &empty_skip());
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }
}

// ===========================================================================
// 17. Filter idempotence: filtering twice yields same result
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_idempotent(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let skip = skip_set(&[w]);
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }
}

// ===========================================================================
// 18. Wrap idempotence: wrapping already-wrapped leaf still valid
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_double_produces_nested_withleaf(base in base_type()) {
        let ty = parse_ty(base);
        let skip = empty_skip();
        let once = wrap_leaf_type(&ty, &skip);
        let twice = wrap_leaf_type(&once, &skip);
        let ws = ty_str(&twice);
        // Should have two layers of WithLeaf
        let count = ws.matches("WithLeaf").count();
        prop_assert!(count >= 2, "Expected 2+ WithLeaf, got {count} in: {ws}");
    }
}

// ===========================================================================
// 19. Extract with matching inner_of always succeeds on wrapped type
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_matching_always_succeeds(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, w, &empty_skip());
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), base);
    }
}

// ===========================================================================
// 20. Filter with single-element skip set
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_single_skip_matches_wrapper(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let skip = skip_set(&[w]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), base);
    }
}

// ===========================================================================
// 21. Extract and filter agree on base type
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_and_filter_agree(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let skip = skip_set(&[w]);
        let (extracted_ty, _) = try_extract_inner_type(&ty, w, &empty_skip());
        let filtered_ty = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&extracted_ty), ty_str(&filtered_ty));
    }
}

// ===========================================================================
// 22. Wrap with skip containing wrapper preserves wrapper name
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_skip_preserves_wrapper_name(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let skip = skip_set(&[w]);
        let wrapped = wrap_leaf_type(&ty, &skip);
        let ws = ty_str(&wrapped);
        // The wrapper name should appear at the start
        prop_assert!(ws.starts_with(w), "Wrapper {w} not at start of: {ws}");
    }
}

// ===========================================================================
// 23. Filter on plain type is identity regardless of skip set
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_plain_is_identity(base in base_type()) {
        let ty = parse_ty(base);
        let skip = skip_set(&["Vec", "Option", "Box"]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), base);
    }
}

// ===========================================================================
// 24. Extract on plain type never extracts regardless of target
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_plain_never_extracts(
        base in base_type(),
        target in wrapper_type(),
    ) {
        let ty = parse_ty(base);
        let (_, extracted) = try_extract_inner_type(&ty, target, &empty_skip());
        prop_assert!(!extracted);
    }
}

// ===========================================================================
// 25. Wrap plain type always produces exactly adze::WithLeaf<T>
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_plain_exact_format(base in base_type()) {
        let ty = parse_ty(base);
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {base} >"));
    }
}

// ===========================================================================
// 26. Extract preserves type string on non-extraction
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_preserves_on_no_match(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty_s = format!("{w}<{base}>");
        let ty = parse_ty(&ty_s);
        let wrong = if w == "Vec" { "Option" } else { "Vec" };
        let (inner, extracted) = try_extract_inner_type(&ty, wrong, &empty_skip());
        prop_assert!(!extracted);
        // Original type string preserved
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }
}

// ===========================================================================
// 27. Wrap container not in skip wraps entire thing
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_container_not_in_skip_wraps_entirely(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty_s = format!("{w}<{base}>");
        let ty = parse_ty(&ty_s);
        // Use skip set that does NOT contain w
        let wrong_skip = if w == "Vec" {
            skip_set(&["Option"])
        } else {
            skip_set(&["Vec"])
        };
        let wrapped = wrap_leaf_type(&ty, &wrong_skip);
        let ws = ty_str(&wrapped);
        prop_assert!(ws.starts_with("adze :: WithLeaf <"),
            "Expected full wrap for {w}, got: {ws}");
    }
}

// ===========================================================================
// 28. Filter and extract consistency on nested types
// ===========================================================================

proptest! {
    #[test]
    fn prop_nested_filter_then_extract(
        (outer, inner) in two_distinct_wrappers(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{outer}<{inner}<{base}>>"));
        // Filter removes outer, then extract on inner should succeed
        let skip_outer = skip_set(&[outer]);
        let after_filter = filter_inner_type(&ty, &skip_outer);
        let (result, extracted) = try_extract_inner_type(&after_filter, inner, &empty_skip());
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), base);
    }
}

// ===========================================================================
// 29. Multiple wrappers in skip: extract through all
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_through_two_skips(base in base_type()) {
        // Box<Option<Vec<base>>> with skip={Box, Option}, target=Vec
        let ty = parse_ty(&format!("Box<Option<Vec<{base}>>>"));
        let skip = skip_set(&["Box", "Option"]);
        let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), base);
    }
}

// ===========================================================================
// 30. Filter three layers
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_three_layers(base in base_type()) {
        let ty = parse_ty(&format!("Box<Option<Vec<{base}>>>"));
        let skip = skip_set(&["Box", "Option", "Vec"]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), base);
    }
}

// ===========================================================================
// 31. Extract returns correct boolean regardless of skip contents
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_bool_correct_when_target_matches(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let (_, extracted) = try_extract_inner_type(&ty, w, &skip_set(&["Box", "Option", "Vec"]));
        prop_assert!(extracted);
    }
}

// ===========================================================================
// 32. Filter with superset skip set still unwraps correctly
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_superset_skip(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let big_skip = skip_set(&["Vec", "Option", "Box", "Arc", "Rc"]);
        let filtered = filter_inner_type(&ty, &big_skip);
        prop_assert_eq!(ty_str(&filtered), base);
    }
}

// ===========================================================================
// 33. Wrap with all wrappers in skip preserves outermost
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_all_skip_preserves_outer(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let all_skip = skip_set(&["Vec", "Option", "Box"]);
        let wrapped = wrap_leaf_type(&ty, &all_skip);
        let ws = ty_str(&wrapped);
        prop_assert!(ws.starts_with(w), "Expected {w} at start, got: {ws}");
    }
}

// ===========================================================================
// 34. Extract returns same type object on non-match
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_nonmatch_type_equals_original(base in base_type()) {
        let ty = parse_ty(base);
        let (inner, _) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }
}

// ===========================================================================
// 35. Wrap contains base type string in output
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_output_contains_base(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let skip = skip_set(&[w]);
        let wrapped = wrap_leaf_type(&ty, &skip);
        let ws = ty_str(&wrapped);
        prop_assert!(ws.contains(base), "Base {base} not in: {ws}");
    }
}

// ===========================================================================
// 36. Filter does not change the base type string
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_result_matches_base_string(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let skip = skip_set(&[w]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), base);
    }
}

// ===========================================================================
// 37. Extract through skip with non-matching inner returns original
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_skip_no_inner_match(base in base_type()) {
        // Box<base> — skip Box, target Vec, but base is not Vec<...>
        let ty = parse_ty(&format!("Box<{base}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Box"]));
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }
}

// ===========================================================================
// 38. Wrap output string length always greater than input
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_output_longer_than_input(base in base_type()) {
        let ty = parse_ty(base);
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        prop_assert!(ty_str(&wrapped).len() > ty_str(&ty).len());
    }
}

// ===========================================================================
// 39. Filter plain type with full skip set is still identity
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_plain_full_skip_identity(base in base_type()) {
        let ty = parse_ty(base);
        let full = skip_set(&["Vec", "Option", "Box", "Arc", "Rc", "Mutex"]);
        let filtered = filter_inner_type(&ty, &full);
        prop_assert_eq!(ty_str(&filtered), base);
    }
}

// ===========================================================================
// 40. Extract Vec<T> via Box skip: Box<Vec<T>> → (T, true)
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_vec_via_box_skip(base in base_type()) {
        let ty = parse_ty(&format!("Box<Vec<{base}>>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Box"]));
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), base);
    }
}

// ===========================================================================
// 41. Wrap plain with various skip sets always gives same result
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_plain_skip_irrelevant(base in base_type()) {
        let ty = parse_ty(base);
        let w1 = ty_str(&wrap_leaf_type(&ty, &empty_skip()));
        let w2 = ty_str(&wrap_leaf_type(&ty, &skip_set(&["Vec"])));
        let w3 = ty_str(&wrap_leaf_type(&ty, &skip_set(&["Option", "Box"])));
        prop_assert_eq!(&w1, &w2);
        prop_assert_eq!(&w2, &w3);
    }
}

// ===========================================================================
// 42. Filter then wrap produces WithLeaf around base
// ===========================================================================

proptest! {
    #[test]
    fn prop_filter_then_wrap(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let skip = skip_set(&[w]);
        let filtered = filter_inner_type(&ty, &skip);
        let wrapped = wrap_leaf_type(&filtered, &empty_skip());
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {base} >"));
    }
}

// ===========================================================================
// 43. Extract Option via Box skip: Box<Option<T>> → (T, true)
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_option_via_box_skip(base in base_type()) {
        let ty = parse_ty(&format!("Box<Option<{base}>>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_set(&["Box"]));
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), base);
    }
}

// ===========================================================================
// 44. Wrap with skip produces string containing wrapper exactly once at start
// ===========================================================================

proptest! {
    #[test]
    fn prop_wrap_skip_wrapper_once_at_start(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let skip = skip_set(&[w]);
        let wrapped = wrap_leaf_type(&ty, &skip);
        let ws = ty_str(&wrapped);
        // The output starts with the wrapper, and WithLeaf appears exactly once
        prop_assert!(ws.starts_with(w));
        prop_assert_eq!(ws.matches("WithLeaf").count(), 1);
    }
}

// ===========================================================================
// 45. Extract and filter both return base from single-wrapper type
// ===========================================================================

proptest! {
    #[test]
    fn prop_extract_filter_both_yield_base(
        w in wrapper_type(),
        base in base_type(),
    ) {
        let ty = parse_ty(&format!("{w}<{base}>"));
        let (extracted_ty, extracted) = try_extract_inner_type(&ty, w, &empty_skip());
        let filtered_ty = filter_inner_type(&ty, &skip_set(&[w]));
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&extracted_ty), base);
        prop_assert_eq!(ty_str(&filtered_ty), base);
    }
}
