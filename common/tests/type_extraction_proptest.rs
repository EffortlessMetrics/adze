#![allow(clippy::needless_range_loop)]

//! Property-based tests for type extraction functions in adze-common.
//!
//! Tests `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! with random inputs to verify no-panic guarantees, idempotency, composition,
//! and output validity properties.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{parse_str, Type};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Simple leaf type names that are never container names.
fn leaf_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize", "Token", "Expr", "Stmt", "Node", "Leaf",
        ][..],
    )
}

/// Container type names used for wrapping.
fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Vec", "Option", "Arc", "Rc"][..])
}

/// Random subsets of container names for skip-over sets.
fn skip_set_strategy() -> impl Strategy<Value = HashSet<&'static str>> {
    prop::collection::hash_set(container_name(), 0..=5)
}

/// Type strings of varying nesting depth.
fn type_string_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Leaf
        leaf_type_name().prop_map(|s| s.to_string()),
        // Container<Leaf>
        (container_name(), leaf_type_name()).prop_map(|(c, l)| format!("{c}<{l}>")),
        // Container<Container<Leaf>>
        (container_name(), container_name(), leaf_type_name())
            .prop_map(|(c1, c2, l)| format!("{c1}<{c2}<{l}>>")),
    ]
}

/// Non-path type strings (references, tuples, arrays, slices).
fn non_path_type_string() -> impl Strategy<Value = String> {
    prop_oneof![
        leaf_type_name().prop_map(|l| format!("& {l}")),
        (leaf_type_name(), leaf_type_name()).prop_map(|(a, b)| format!("({a} , {b})")),
        leaf_type_name().prop_map(|l| format!("[{l} ; 4]")),
        leaf_type_name().prop_map(|l| format!("& [{l}]")),
    ]
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // ===== try_extract_inner_type: no-panic & correctness =====

    // 1. Tuple types are non-path and never cause extraction.
    #[test]
    fn extract_tuple_never_extracts(
        a in leaf_type_name(),
        b in leaf_type_name(),
        target in container_name(),
    ) {
        let ty: Type = parse_str(&format!("({a}, {b})")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_result, extracted) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(!extracted);
    }

    // 2. Array types are non-path and never cause extraction.
    #[test]
    fn extract_array_never_extracts(
        leaf in leaf_type_name(),
        target in container_name(),
    ) {
        let ty: Type = parse_str(&format!("[{leaf}; 4]")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_result, extracted) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(!extracted);
    }

    // 3. When extraction fails on a leaf, result equals original.
    #[test]
    fn extract_fail_preserves_original(
        leaf in leaf_type_name(),
        target in container_name(),
    ) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&result), ty_str(&ty));
    }

    // 4. Direct match on container always succeeds and returns the inner type.
    #[test]
    fn extract_direct_match_returns_inner(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, container, &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 5. Successful extraction produces a result with strictly fewer characters.
    #[test]
    fn extract_success_shrinks_output(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, container, &skip);
        prop_assert!(extracted);
        prop_assert!(ty_str(&result).len() < ty_str(&ty).len());
    }

    // 6. Double extraction peels two identical container layers.
    #[test]
    fn extract_double_layer(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{container}<{inner}>>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (first, ok1) = try_extract_inner_type(&ty, container, &skip);
        prop_assert!(ok1);
        let (second, ok2) = try_extract_inner_type(&first, container, &skip);
        prop_assert!(ok2);
        prop_assert_eq!(ty_str(&second), inner);
    }

    // 7. Extraction with disjoint skip set still extracts a direct top-level match.
    #[test]
    fn extract_disjoint_skip_still_extracts(inner in leaf_type_name()) {
        let ty: Type = parse_str(&format!("Vec<{inner}>")).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 8. Extraction through a skip wrapper reaches nested target.
    #[test]
    fn extract_through_skip_wrapper(
        wrapper in prop::sample::select(&["Box", "Arc"][..]),
        inner in leaf_type_name(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<Option<{inner}>>")).unwrap();
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 9. Non-path types return the exact original on extraction.
    #[test]
    fn extract_non_path_returns_original(
        non_path in non_path_type_string(),
        target in container_name(),
    ) {
        let ty: Type = parse_str(&non_path).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&result), ty_str(&ty));
    }

    // 10. Extraction result is always a parseable Type.
    #[test]
    fn extract_result_is_parseable(
        ty_s in type_string_strategy(),
        target in container_name(),
        skip in skip_set_strategy(),
    ) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let (result, _) = try_extract_inner_type(&ty, target, &skip);
        let s = ty_str(&result);
        prop_assert!(parse_str::<Type>(&s).is_ok(), "unparseable: {s}");
    }

    // 11. Extraction is deterministic.
    #[test]
    fn extract_deterministic(
        ty_s in type_string_strategy(),
        target in container_name(),
        skip in skip_set_strategy(),
    ) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let (r1, e1) = try_extract_inner_type(&ty, target, &skip);
        let (r2, e2) = try_extract_inner_type(&ty, target, &skip);
        prop_assert_eq!(e1, e2);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }

    // ===== filter_inner_type: identity, idempotency, validity =====

    // 12. Filtering a tuple type is always identity.
    #[test]
    fn filter_tuple_is_identity(
        a in leaf_type_name(),
        b in leaf_type_name(),
        skip in skip_set_strategy(),
    ) {
        let ty: Type = parse_str(&format!("({a}, {b})")).unwrap();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    // 13. Filtering an array type is always identity.
    #[test]
    fn filter_array_is_identity(
        leaf in leaf_type_name(),
        skip in skip_set_strategy(),
    ) {
        let ty: Type = parse_str(&format!("[{leaf}; 4]")).unwrap();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    // 14. Filtering any non-path type is identity.
    #[test]
    fn filter_non_path_is_identity(
        non_path in non_path_type_string(),
        skip in skip_set_strategy(),
    ) {
        let ty: Type = parse_str(&non_path).unwrap();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    // 15. filter_inner_type is idempotent on arbitrary valid types.
    #[test]
    fn filter_idempotent(
        ty_s in type_string_strategy(),
        skip in skip_set_strategy(),
    ) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    // 16. Filtering with a superset skip set produces same-or-smaller output.
    #[test]
    fn filter_superset_skip_no_larger(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let small_skip: HashSet<&str> = HashSet::new();
        let big_skip: HashSet<&str> = [container].into_iter().collect();
        let small_len = ty_str(&filter_inner_type(&ty, &small_skip)).len();
        let big_len = ty_str(&filter_inner_type(&ty, &big_skip)).len();
        prop_assert!(big_len <= small_len);
    }

    // 17. Filtered output is always parseable.
    #[test]
    fn filter_output_parseable(
        ty_s in type_string_strategy(),
        skip in skip_set_strategy(),
    ) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let s = ty_str(&filter_inner_type(&ty, &skip));
        prop_assert!(parse_str::<Type>(&s).is_ok(), "unparseable: {s}");
    }

    // 18. filter_inner_type is deterministic.
    #[test]
    fn filter_deterministic(
        ty_s in type_string_strategy(),
        skip in skip_set_strategy(),
    ) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let f1 = ty_str(&filter_inner_type(&ty, &skip));
        let f2 = ty_str(&filter_inner_type(&ty, &skip));
        prop_assert_eq!(f1, f2);
    }

    // ===== wrap_leaf_type: wrapping, preservation, validity =====

    // 19. Wrapping a leaf type always produces output containing WithLeaf.
    #[test]
    fn wrap_leaf_contains_with_leaf(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &HashSet::new()));
        prop_assert!(s.contains("adze :: WithLeaf"), "expected WithLeaf: {s}");
    }

    // 20. Wrapping a non-path type always produces output containing WithLeaf.
    #[test]
    fn wrap_non_path_contains_with_leaf(non_path in non_path_type_string()) {
        let ty: Type = parse_str(&non_path).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &HashSet::new()));
        prop_assert!(s.contains("adze :: WithLeaf"), "expected WithLeaf: {s}");
    }

    // 21. Wrapping a leaf type increases character count.
    #[test]
    fn wrap_leaf_increases_length(leaf in leaf_type_name()) {
        let ty: Type = parse_str(leaf).unwrap();
        let orig_len = ty_str(&ty).len();
        let wrapped_len = ty_str(&wrap_leaf_type(&ty, &HashSet::new())).len();
        prop_assert!(wrapped_len > orig_len);
    }

    // 22. Wrapping with a skip set preserves the container name at the front.
    #[test]
    fn wrap_preserves_skip_container_name(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let skip: HashSet<&str> = [container].into_iter().collect();
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(
            s.starts_with(&format!("{container} <")),
            "expected {container} at front: {s}"
        );
    }

    // 23. Wrapping without a skip set wraps the entire container.
    #[test]
    fn wrap_no_skip_wraps_entirely(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &HashSet::new()));
        prop_assert!(
            s.starts_with("adze :: WithLeaf"),
            "expected adze::WithLeaf at front: {s}"
        );
    }

    // 24. Wrapped output is always parseable.
    #[test]
    fn wrap_output_parseable(
        ty_s in type_string_strategy(),
        skip in skip_set_strategy(),
    ) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(parse_str::<Type>(&s).is_ok(), "unparseable: {s}");
    }

    // 25. wrap_leaf_type is deterministic.
    #[test]
    fn wrap_deterministic(
        ty_s in type_string_strategy(),
        skip in skip_set_strategy(),
    ) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let w1 = ty_str(&wrap_leaf_type(&ty, &skip));
        let w2 = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(w1, w2);
    }

    // ===== Composition properties =====

    // 26. filter(extract_fail.0) equals original when extraction did not succeed.
    #[test]
    fn filter_after_failed_extract_is_identity(
        leaf in leaf_type_name(),
        target in container_name(),
    ) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(!extracted);
        let filtered = filter_inner_type(&result, &skip);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    // 27. wrap(filter(C<T>, {C})) equals wrap(T) for single-layer containers.
    #[test]
    fn wrap_of_filtered_equals_wrap_of_inner(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let skip: HashSet<&str> = [container].into_iter().collect();
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip);
        let wrapped_filtered = wrap_leaf_type(&filtered, &HashSet::new());
        let inner_ty: Type = parse_str(inner).unwrap();
        let wrapped_inner = wrap_leaf_type(&inner_ty, &HashSet::new());
        prop_assert_eq!(ty_str(&wrapped_filtered), ty_str(&wrapped_inner));
    }

    // 28. extract then wrap gives adze::WithLeaf<inner> for single-layer targets.
    #[test]
    fn extract_then_wrap_gives_with_leaf(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (extracted, ok) = try_extract_inner_type(&ty, container, &skip);
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&extracted, &HashSet::new());
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }

    // 29. Extracting through skip then wrapping produces adze::WithLeaf<inner>.
    #[test]
    fn extract_through_skip_then_wrap(
        wrapper in prop::sample::select(&["Box", "Arc"][..]),
        inner in leaf_type_name(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<Option<{inner}>>")).unwrap();
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let (extracted, ok) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&extracted, &HashSet::new());
        prop_assert_eq!(ty_str(&wrapped), format!("adze :: WithLeaf < {inner} >"));
    }

    // 30. All three functions handle slice references without panicking.
    #[test]
    fn slice_ref_no_panic_all_functions(
        leaf in leaf_type_name(),
        target in container_name(),
        skip in skip_set_strategy(),
    ) {
        let ty: Type = parse_str(&format!("&[{leaf}]")).unwrap();
        let _ = try_extract_inner_type(&ty, target, &skip);
        let _ = filter_inner_type(&ty, &skip);
        let _ = wrap_leaf_type(&ty, &skip);
    }

    // 31. Wrapping nested skip containers wraps only the innermost leaf.
    #[test]
    fn wrap_nested_skips_wraps_leaf_only(inner in leaf_type_name()) {
        let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
        let ty: Type = parse_str(&format!("Option<Vec<{inner}>>")).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(s.starts_with("Option <"), "outer preserved: {s}");
        prop_assert!(s.contains("Vec <"), "middle preserved: {s}");
        prop_assert!(s.contains("adze :: WithLeaf"), "leaf wrapped: {s}");
        // Exactly one WithLeaf occurrence
        let count = s.matches("WithLeaf").count();
        prop_assert!(count == 1, "expected 1 WithLeaf, got {} in: {}", count, s);
    }

    // 32. filter(ty) with empty skip equals identity for any type_string.
    #[test]
    fn filter_empty_skip_is_identity(ty_s in type_string_strategy()) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let filtered = filter_inner_type(&ty, &HashSet::new());
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    // 33. Extracting a mismatched target from a container returns the container unchanged.
    #[test]
    fn extract_mismatch_returns_container_unchanged(
        container in container_name(),
        inner in leaf_type_name(),
    ) {
        // Only test when container != "Option"
        prop_assume!(container != "Option");
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&result), ty_str(&ty));
    }

    // 34. filter then filter then filter = filter (triple idempotency).
    #[test]
    fn filter_triple_idempotent(ty_s in type_string_strategy(), skip in skip_set_strategy()) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let f1 = filter_inner_type(&ty, &skip);
        let f2 = filter_inner_type(&f1, &skip);
        let f3 = filter_inner_type(&f2, &skip);
        prop_assert_eq!(ty_str(&f1), ty_str(&f2));
        prop_assert_eq!(ty_str(&f2), ty_str(&f3));
    }
}
