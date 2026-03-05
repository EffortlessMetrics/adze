//! Property-based tests (v5) for `filter_inner_type` in adze-common.
//!
//! 48 proptest properties across 9 categories: Option filtering, Vec filtering,
//! Box filtering, non-matching, determinism, agreement with extract, nesting,
//! qualified paths, and edge cases.

use adze_common::{filter_inner_type, try_extract_inner_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Leaf primitives that are valid, non-keyword Rust type names.
fn leaf_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Common single-arg generic containers.
fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec", "Box", "Arc", "Rc"][..])
}

/// Containers disjoint from Option/Vec/Box for non-matching tests.
fn other_container() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Arc", "Rc"][..])
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn to_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip<'a>(items: &'a [&'a str]) -> HashSet<&'a str> {
    items.iter().copied().collect()
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

// ===========================================================================
// 1. Filter "Option" extracts from Option<T> (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Filtering Option<leaf> with {"Option"} yields the leaf.
    #[test]
    fn option_filter_extracts_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering Option<Option<leaf>> with {"Option"} strips both layers.
    #[test]
    fn option_filter_strips_nested_option(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Option<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering Option<C<leaf>> with {"Option"} strips only Option.
    #[test]
    fn option_filter_keeps_inner_container(
        inner in other_container(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Option<{inner}<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option"]));
        prop_assert_eq!(to_str(&filtered), format!("{inner} < {leaf} >"));
    }

    /// Filtering Option<leaf> with {"Option", "Box"} still extracts leaf.
    #[test]
    fn option_filter_with_extra_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option", "Box"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering Option<Box<leaf>> with {"Option", "Box"} strips both.
    #[test]
    fn option_box_filter_strips_both(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Box<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option", "Box"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }
}

// ===========================================================================
// 2. Filter "Vec" extracts from Vec<T> (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Filtering Vec<leaf> with {"Vec"} yields the leaf.
    #[test]
    fn vec_filter_extracts_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering Vec<Vec<leaf>> with {"Vec"} strips both layers.
    #[test]
    fn vec_filter_strips_nested_vec(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<Vec<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering Vec<Option<leaf>> with {"Vec"} strips only Vec.
    #[test]
    fn vec_filter_keeps_inner_option(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<Option<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
        prop_assert_eq!(to_str(&filtered), format!("Option < {leaf} >"));
    }

    /// Filtering Vec<leaf> with {"Vec", "Arc"} still extracts leaf.
    #[test]
    fn vec_filter_with_extra_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Vec", "Arc"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering Vec<Arc<leaf>> with {"Vec", "Arc"} strips both.
    #[test]
    fn vec_arc_filter_strips_both(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<Arc<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Vec", "Arc"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }
}

// ===========================================================================
// 3. Filter "Box" extracts from Box<T> (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Filtering Box<leaf> with {"Box"} yields the leaf.
    #[test]
    fn box_filter_extracts_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering Box<Box<leaf>> with {"Box"} strips both layers.
    #[test]
    fn box_filter_strips_nested_box(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Box<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering Box<Vec<leaf>> with {"Box"} strips only Box.
    #[test]
    fn box_filter_keeps_inner_vec(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Vec<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(to_str(&filtered), format!("Vec < {leaf} >"));
    }

    /// Filtering Box<leaf> with {"Box", "Rc"} still extracts leaf.
    #[test]
    fn box_filter_with_extra_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box", "Rc"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering Box<Rc<leaf>> with {"Box", "Rc"} strips both.
    #[test]
    fn box_rc_filter_strips_both(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Rc<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box", "Rc"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }
}

// ===========================================================================
// 4. Filter non-matching returns None / unchanged (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Filtering a plain leaf with any skip set returns the leaf unchanged.
    #[test]
    fn nonmatch_plain_leaf_unchanged(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option", "Vec", "Box"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering a container not in skip set returns it unchanged.
    #[test]
    fn nonmatch_container_not_in_skip(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(to_str(&filtered), format!("{container} < {leaf} >"));
    }

    /// Filtering Vec<leaf> with {"Option"} keeps Vec<leaf>.
    #[test]
    fn nonmatch_vec_with_option_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option"]));
        prop_assert_eq!(to_str(&filtered), format!("Vec < {leaf} >"));
    }

    /// Filtering Option<leaf> with {"Vec"} keeps Option<leaf>.
    #[test]
    fn nonmatch_option_with_vec_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
        prop_assert_eq!(to_str(&filtered), format!("Option < {leaf} >"));
    }

    /// Filtering a nested type where outer is not in skip set is unchanged.
    #[test]
    fn nonmatch_nested_outer_not_in_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Arc<Box<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option"]));
        prop_assert_eq!(to_str(&filtered), format!("Arc < Box < {leaf} > >"));
    }
}

// ===========================================================================
// 5. Filter is deterministic (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Running filter twice on the same input yields the same result.
    #[test]
    fn deterministic_same_result_twice(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let arr = [container];
        let skip_set = skip(&arr);
        let a = filter_inner_type(&ty, &skip_set);
        let b = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(to_str(&a), to_str(&b));
    }

    /// Filtering a plain leaf is deterministic.
    #[test]
    fn deterministic_plain_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let a = filter_inner_type(&ty, &empty_skip());
        let b = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(to_str(&a), to_str(&b));
    }

    /// Filtering a double-wrapped type is deterministic.
    #[test]
    fn deterministic_double_wrapped(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Option<{leaf}>>")).unwrap();
        let skip_set = skip(&["Box", "Option"]);
        let a = filter_inner_type(&ty, &skip_set);
        let b = filter_inner_type(&ty, &skip_set);
        prop_assert_eq!(to_str(&a), to_str(&b));
    }

    /// Filter result is idempotent: filtering the output again is a no-op.
    #[test]
    fn deterministic_idempotent(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let arr = [container];
        let skip_set = skip(&arr);
        let once = filter_inner_type(&ty, &skip_set);
        let twice = filter_inner_type(&once, &skip_set);
        prop_assert_eq!(to_str(&once), to_str(&twice));
    }

    /// Filtering with empty skip set is always a no-op.
    #[test]
    fn deterministic_empty_skip_noop(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(to_str(&filtered), format!("{container} < {leaf} >"));
    }
}

// ===========================================================================
// 6. Filter agrees with extract for known containers (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// When container is in skip set, filter and extract yield the same leaf.
    #[test]
    fn agree_single_wrapper(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let (extracted, ok) = try_extract_inner_type(&ty, container, &empty_skip());
        let filtered = filter_inner_type(&ty, &skip(&[container]));
        prop_assert!(ok);
        prop_assert_eq!(to_str(&extracted), to_str(&filtered));
    }

    /// Extract through skip layer agrees with filter stripping both.
    #[test]
    fn agree_skip_then_extract(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Vec<{leaf}>>")).unwrap();
        let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
        let filtered = filter_inner_type(&ty, &skip(&["Box", "Vec"]));
        prop_assert!(ok);
        prop_assert_eq!(to_str(&extracted), to_str(&filtered));
    }

    /// Both agree on a plain leaf: extract fails, filter returns unchanged.
    #[test]
    fn agree_plain_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &empty_skip());
        let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
        prop_assert!(!ok);
        prop_assert_eq!(to_str(&extracted), to_str(&filtered));
    }

    /// Both agree on Arc<Rc<leaf>> when skipping both.
    #[test]
    fn agree_arc_rc(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Arc<Rc<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Arc", "Rc"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// For Option<leaf>, extract as Option and filter over Option give same result.
    #[test]
    fn agree_option_extract_vs_filter(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let (extracted, ok) = try_extract_inner_type(&ty, "Option", &empty_skip());
        let filtered = filter_inner_type(&ty, &skip(&["Option"]));
        prop_assert!(ok);
        prop_assert_eq!(to_str(&extracted), to_str(&filtered));
    }
}

// ===========================================================================
// 7. Nested filtering (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Filtering 3-deep nesting strips all skip layers.
    #[test]
    fn nested_three_deep_all_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<Rc<{leaf}>>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering 3-deep but only skipping outer two keeps innermost wrapper.
    #[test]
    fn nested_three_deep_partial_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<Vec<{leaf}>>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
        prop_assert_eq!(to_str(&filtered), format!("Vec < {leaf} >"));
    }

    /// Mixed: Option<Vec<Box<leaf>>> skip all three.
    #[test]
    fn nested_option_vec_box(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Vec<Box<{leaf}>>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option", "Vec", "Box"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering stops at first non-skip layer in a deep chain.
    #[test]
    fn nested_stops_at_non_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Option<Arc<{leaf}>>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(to_str(&filtered), format!("Option < Arc < {leaf} > >"));
    }

    /// Alternating skip/non-skip: only outermost skip layer is peeled.
    #[test]
    fn nested_alternating_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Arc<Vec<Arc<{leaf}>>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
        prop_assert_eq!(to_str(&filtered), format!("Vec < Arc < {leaf} > >"));
    }
}

// ===========================================================================
// 8. Filter with qualified paths (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Qualified std::option::Option<leaf> is matched by last segment "Option".
    #[test]
    fn qualified_std_option(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("std::option::Option<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Qualified std::vec::Vec<leaf> is matched by "Vec".
    #[test]
    fn qualified_std_vec(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("std::vec::Vec<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Vec"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Qualified std::boxed::Box<leaf> is matched by "Box".
    #[test]
    fn qualified_std_box(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("std::boxed::Box<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Qualified path not in skip set is unchanged.
    #[test]
    fn qualified_not_in_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("std::sync::Arc<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(to_str(&filtered), format!("std :: sync :: Arc < {leaf} >"));
    }

    /// Nested qualified: std::boxed::Box<std::option::Option<leaf>> skip both.
    #[test]
    fn qualified_nested_box_option(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("std::boxed::Box<std::option::Option<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Box", "Option"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }
}

// ===========================================================================
// 9. Edge cases (8 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Filtering with a superset skip set that includes the container works.
    #[test]
    fn edge_superset_skip(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option", "Vec", "Box", "Arc", "Rc"]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering a random container with itself in the skip set always yields the leaf.
    #[test]
    fn edge_self_in_skip(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[container]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering a deeply homogeneous chain: C<C<C<leaf>>> with {C} → leaf.
    #[test]
    fn edge_homogeneous_chain(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{container}<{container}<{leaf}>>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[container]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filtering leaf "String" (multi-char) is unchanged with any skip set.
    #[test]
    fn edge_string_leaf_unchanged(
        container in container_name(),
    ) {
        let ty: Type = parse_str("String").unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[container]));
        prop_assert_eq!(to_str(&filtered), "String");
    }

    /// Two different containers: filter only strips the matching one.
    #[test]
    fn edge_two_containers_filter_one(
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Option<Vec<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&["Option"]));
        prop_assert_eq!(to_str(&filtered), format!("Vec < {leaf} >"));
    }

    /// Filtering with single-element skip set on a 2-layer same-container nesting.
    #[test]
    fn edge_same_container_two_layers(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{container}<{leaf}>>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[container]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filter output always parses as a valid syn::Type.
    #[test]
    fn edge_output_is_valid_type(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[container]));
        let reparsed: Result<Type, _> = parse_str(&to_str(&filtered));
        prop_assert!(reparsed.is_ok());
    }

    /// Filter on bool leaf with every container is well-behaved.
    #[test]
    fn edge_bool_leaf_roundtrip(container in container_name()) {
        let ty: Type = parse_str(&format!("{container}<bool>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip(&[container]));
        prop_assert_eq!(to_str(&filtered), "bool");
    }
}
