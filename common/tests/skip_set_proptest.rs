#![allow(clippy::needless_range_loop)]

//! Property-based tests for skip set handling in adze-common.
//!
//! Focuses on how different skip set configurations affect
//! `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{parse_str, Type};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn leaf_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize", "Foo", "Bar", "Baz", "Token", "Expr",
        ][..],
    )
}

fn wrapper_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Vec", "Option", "Arc", "Rc", "Cell", "Mutex"][..])
}

fn generic_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "MyWrapper",
            "Container",
            "Wrapper",
            "Handle",
            "Ref",
            "Guard",
            "Slot",
            "Entry",
        ][..],
    )
}

/// Build a skip set of exactly `n` elements drawn from wrappers.
fn skip_set_of_size(n: usize) -> impl Strategy<Value = HashSet<&'static str>> {
    prop::collection::hash_set(wrapper_name(), n..=n)
}

/// Large skip set: 4-7 elements including generic names.
fn large_skip_set() -> impl Strategy<Value = HashSet<&'static str>> {
    prop::collection::hash_set(
        prop_oneof![wrapper_name(), generic_type_name()],
        4..=7,
    )
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

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // ===== 1. Empty skip set passes everything through =====

    /// Empty skip set: filter leaves any Container<Leaf> unchanged.
    #[test]
    fn empty_skip_filter_passthrough(
        container in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let filtered = filter_inner_type(&ty, &HashSet::new());
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    /// Empty skip set: extraction only succeeds on direct target match.
    #[test]
    fn empty_skip_extract_only_direct(
        container in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let (result, ok) = try_extract_inner_type(&ty, container, &HashSet::new());
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), leaf);
    }

    /// Empty skip set: extract fails for mismatched target on nested type.
    #[test]
    fn empty_skip_no_reach_through(
        outer in wrapper_name(),
        inner in wrapper_name(),
        leaf in leaf_name(),
    ) {
        prop_assume!(outer != inner);
        let ty = parse_ty(&format!("{outer}<{inner}<{leaf}>>"));
        let (result, ok) = try_extract_inner_type(&ty, inner, &HashSet::new());
        prop_assert!(!ok);
        prop_assert_eq!(ty_str(&result), ty_str(&ty));
    }

    /// Empty skip set: wrap wraps the entire type including containers.
    #[test]
    fn empty_skip_wrap_entire(
        container in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let s = ty_str(&wrap_leaf_type(&ty, &HashSet::new()));
        prop_assert!(s.starts_with("adze :: WithLeaf <"), "got: {s}");
    }

    // ===== 2. Single-element skip set =====

    /// Single skip element: filter peels exactly that wrapper.
    #[test]
    fn single_skip_filter_peels_wrapper(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    /// Single skip element: extract reaches through the skip wrapper to target.
    #[test]
    fn single_skip_extract_reaches_target(
        wrapper in wrapper_name(),
        target in wrapper_name(),
        leaf in leaf_name(),
    ) {
        prop_assume!(wrapper != target);
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let ty = parse_ty(&format!("{wrapper}<{target}<{leaf}>>"));
        let (result, ok) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), leaf);
    }

    /// Single skip element: wrap preserves the skip container at front.
    #[test]
    fn single_skip_wrap_preserves_container(
        wrapper in wrapper_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = [wrapper].into_iter().collect();
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let s = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(s.starts_with(&format!("{wrapper} <")), "got: {s}");
        prop_assert!(s.contains("adze :: WithLeaf"), "got: {s}");
    }

    /// Single skip element: non-matching container is not peeled by filter.
    #[test]
    fn single_skip_non_matching_filter_noop(
        skip_name in wrapper_name(),
        other in wrapper_name(),
        leaf in leaf_name(),
    ) {
        prop_assume!(skip_name != other);
        let skip: HashSet<&str> = [skip_name].into_iter().collect();
        let ty = parse_ty(&format!("{other}<{leaf}>"));
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    // ===== 3. Multiple-element skip set =====

    /// Two-element skip set: filter peels both layers.
    #[test]
    fn multi_skip_filter_peels_two_layers(
        w1 in wrapper_name(),
        w2 in wrapper_name(),
        leaf in leaf_name(),
    ) {
        prop_assume!(w1 != w2);
        let skip: HashSet<&str> = [w1, w2].into_iter().collect();
        let ty = parse_ty(&format!("{w1}<{w2}<{leaf}>>"));
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    /// Two-element skip set: extract passes through both to reach target.
    #[test]
    fn multi_skip_extract_through_two(
        w1 in wrapper_name(),
        w2 in wrapper_name(),
        target in wrapper_name(),
        leaf in leaf_name(),
    ) {
        prop_assume!(w1 != w2 && w1 != target && w2 != target);
        let skip: HashSet<&str> = [w1, w2].into_iter().collect();
        let ty = parse_ty(&format!("{w1}<{w2}<{target}<{leaf}>>>"));
        let (result, ok) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), leaf);
    }

    /// Two-element skip set: wrap preserves both containers, wraps leaf.
    #[test]
    fn multi_skip_wrap_preserves_both(
        w1 in wrapper_name(),
        w2 in wrapper_name(),
        leaf in leaf_name(),
    ) {
        prop_assume!(w1 != w2);
        let skip: HashSet<&str> = [w1, w2].into_iter().collect();
        let ty = parse_ty(&format!("{w1}<{w2}<{leaf}>>"));
        let s = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(s.starts_with(&format!("{w1} <")), "outer: {s}");
        prop_assert!(s.contains(&format!("{w2} <")), "inner: {s}");
        prop_assert!(s.contains("adze :: WithLeaf"), "leaf: {s}");
    }

    // ===== 4. Skip set with generic types =====

    /// Generic type in skip set: filter peels custom wrapper names.
    #[test]
    fn generic_skip_filter_peels(
        generic in generic_type_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = [generic].into_iter().collect();
        let ty = parse_ty(&format!("{generic}<{leaf}>"));
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    /// Generic type in skip set: extract through custom wrapper.
    #[test]
    fn generic_skip_extract_through(
        generic in generic_type_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = [generic].into_iter().collect();
        let ty = parse_ty(&format!("{generic}<Vec<{leaf}>>"));
        let (result, ok) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), leaf);
    }

    /// Generic type in skip set: wrap preserves it and wraps inner leaf.
    #[test]
    fn generic_skip_wrap_preserves(
        generic in generic_type_name(),
        leaf in leaf_name(),
    ) {
        let skip: HashSet<&str> = [generic].into_iter().collect();
        let ty = parse_ty(&format!("{generic}<{leaf}>"));
        let s = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(s.starts_with(&format!("{generic} <")), "got: {s}");
        prop_assert!(s.contains("adze :: WithLeaf"), "got: {s}");
    }

    // ===== 5. Skip set intersection behavior =====

    /// Adding elements to skip set never increases filter output size.
    #[test]
    fn skip_superset_filter_monotonic(
        w1 in wrapper_name(),
        w2 in wrapper_name(),
        leaf in leaf_name(),
    ) {
        prop_assume!(w1 != w2);
        let small: HashSet<&str> = [w1].into_iter().collect();
        let big: HashSet<&str> = [w1, w2].into_iter().collect();
        let ty = parse_ty(&format!("{w1}<{w2}<{leaf}>>"));
        let small_out = ty_str(&filter_inner_type(&ty, &small)).len();
        let big_out = ty_str(&filter_inner_type(&ty, &big)).len();
        prop_assert!(big_out <= small_out, "big={big_out} > small={small_out}");
    }

    /// Extract succeeds with superset skip if it succeeds with subset.
    #[test]
    fn skip_superset_extract_succeeds(
        wrapper in wrapper_name(),
        target in wrapper_name(),
        leaf in leaf_name(),
    ) {
        prop_assume!(wrapper != target);
        let small: HashSet<&str> = [wrapper].into_iter().collect();
        let big: HashSet<&str> = [wrapper, "Arc", "Rc"].into_iter().collect();
        let ty = parse_ty(&format!("{wrapper}<{target}<{leaf}>>"));
        let (_, ok_small) = try_extract_inner_type(&ty, target, &small);
        let (_, ok_big) = try_extract_inner_type(&ty, target, &big);
        // If it worked with a subset, it must work with a superset.
        if ok_small {
            prop_assert!(ok_big);
        }
    }

    /// Disjoint skip set does not help extraction through non-skip wrapper.
    #[test]
    fn disjoint_skip_no_help(
        outer in wrapper_name(),
        target in wrapper_name(),
        leaf in leaf_name(),
    ) {
        prop_assume!(outer != target && outer != "Mutex" && outer != "Cell");
        let skip: HashSet<&str> = ["Mutex", "Cell"].into_iter().collect();
        let ty = parse_ty(&format!("{outer}<{target}<{leaf}>>"));
        let (_, ok) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(!ok);
    }

    // ===== 6. Skip set does not affect non-matching types =====

    /// Bare leaf type unaffected by any skip set for filter.
    #[test]
    fn skip_irrelevant_for_bare_leaf_filter(
        leaf in leaf_name(),
        skip in skip_set_of_size(3),
    ) {
        let ty = parse_ty(leaf);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    /// Bare leaf type unaffected by any skip set for extraction.
    #[test]
    fn skip_irrelevant_for_bare_leaf_extract(
        leaf in leaf_name(),
        target in wrapper_name(),
        skip in skip_set_of_size(3),
    ) {
        let ty = parse_ty(leaf);
        let (result, ok) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(!ok);
        prop_assert_eq!(ty_str(&result), leaf);
    }

    /// Reference types unaffected by skip set for all three functions.
    #[test]
    fn skip_irrelevant_for_references(
        leaf in leaf_name(),
        skip in skip_set_of_size(2),
    ) {
        let ty = parse_ty(&format!("& {leaf}"));
        // filter
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
        // extract
        let (result, ok) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(!ok);
        prop_assert_eq!(ty_str(&result), ty_str(&ty));
        // wrap always wraps non-path types
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains("adze :: WithLeaf"));
    }

    // ===== 7. Skip set with Option/Vec/Box types =====

    /// Option in skip set: filter peels Option, leaves inner.
    #[test]
    fn option_skip_filter(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option"].into_iter().collect();
        let ty = parse_ty(&format!("Option<{leaf}>"));
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    /// Vec in skip set: filter peels Vec, leaves inner.
    #[test]
    fn vec_skip_filter(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    /// Box in skip set: extract through Box to reach Option target.
    #[test]
    fn box_skip_extract_option(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let ty = parse_ty(&format!("Box<Option<{leaf}>>"));
        let (result, ok) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(ok);
        prop_assert_eq!(ty_str(&result), leaf);
    }

    /// Option+Vec+Box all in skip: filter peels all three layers.
    #[test]
    fn option_vec_box_skip_filter_all(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option", "Vec", "Box"].into_iter().collect();
        let ty = parse_ty(&format!("Option<Vec<Box<{leaf}>>>"));
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    /// Option+Vec in skip: wrap preserves both, wraps leaf.
    #[test]
    fn option_vec_skip_wrap(leaf in leaf_name()) {
        let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
        let ty = parse_ty(&format!("Option<Vec<{leaf}>>"));
        let s = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(s.starts_with("Option <"), "outer: {s}");
        prop_assert!(s.contains("Vec <"), "inner: {s}");
        let count = s.matches("WithLeaf").count();
        prop_assert!(count == 1, "expected 1 WithLeaf, got {} in: {}", count, s);
    }

    // ===== 8. Large skip sets =====

    /// Large skip set: filter output is always parseable.
    #[test]
    fn large_skip_filter_parseable(
        container in wrapper_name(),
        leaf in leaf_name(),
        skip in large_skip_set(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let s = ty_str(&filter_inner_type(&ty, &skip));
        prop_assert!(parse_str::<Type>(&s).is_ok(), "unparseable: {s}");
    }

    /// Large skip set: wrap output is always parseable.
    #[test]
    fn large_skip_wrap_parseable(
        container in wrapper_name(),
        leaf in leaf_name(),
        skip in large_skip_set(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let s = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(parse_str::<Type>(&s).is_ok(), "unparseable: {s}");
    }

    /// Large skip set: filter is still idempotent.
    #[test]
    fn large_skip_filter_idempotent(
        container in wrapper_name(),
        leaf in leaf_name(),
        skip in large_skip_set(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    /// Large skip set: extract is deterministic.
    #[test]
    fn large_skip_extract_deterministic(
        container in wrapper_name(),
        leaf in leaf_name(),
        target in wrapper_name(),
        skip in large_skip_set(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let (r1, e1) = try_extract_inner_type(&ty, target, &skip);
        let (r2, e2) = try_extract_inner_type(&ty, target, &skip);
        prop_assert_eq!(e1, e2);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }

    /// Large skip set: filter never increases output size.
    #[test]
    fn large_skip_filter_never_grows(
        container in wrapper_name(),
        leaf in leaf_name(),
        skip in large_skip_set(),
    ) {
        let ty = parse_ty(&format!("{container}<{leaf}>"));
        let orig_len = ty_str(&ty).len();
        let filtered_len = ty_str(&filter_inner_type(&ty, &skip)).len();
        prop_assert!(filtered_len <= orig_len, "{filtered_len} > {orig_len}");
    }
}
