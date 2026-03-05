//! Property-based tests (v4) for adze-common extraction utilities.
//!
//! 40+ proptest properties covering: extraction consistency, filter specificity,
//! wrapping determinism, and parameterized detection correctness.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Leaf primitives that are always valid, non-keyword Rust types.
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

/// Members suitable for skip sets.
fn skip_member() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Box", "Arc", "Rc"][..])
}

/// A second set of containers disjoint from skip_member for mismatch tests.
fn non_skip_container() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec"][..])
}

/// Depth-0 or depth-1 type string.
fn type_string_shallow() -> impl Strategy<Value = String> {
    prop_oneof![
        leaf_type().prop_map(|s| s.to_string()),
        (container_name(), leaf_type()).prop_map(|(c, l)| format!("{c}<{l}>")),
    ]
}

/// Depth-2 nested type string.
fn type_string_deep() -> impl Strategy<Value = String> {
    (container_name(), container_name(), leaf_type())
        .prop_map(|(c1, c2, l)| format!("{c1}<{c2}<{l}>>"))
}

/// Depth 0–2 random type string.
fn any_type_string() -> impl Strategy<Value = String> {
    prop_oneof![
        leaf_type().prop_map(|s| s.to_string()),
        (container_name(), leaf_type()).prop_map(|(c, l)| format!("{c}<{l}>")),
        (container_name(), container_name(), leaf_type())
            .prop_map(|(c1, c2, l)| format!("{c1}<{c2}<{l}>>")),
    ]
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn to_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn is_parameterized(ty: &Type) -> bool {
    if let Type::Path(p) = ty
        && let Some(seg) = p.path.segments.last()
    {
        return matches!(seg.arguments, syn::PathArguments::AngleBracketed(_));
    }
    false
}

fn skip_set<'a>(items: &'a [&'a str]) -> HashSet<&'a str> {
    items.iter().copied().collect()
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

// ===========================================================================
// 1. Extraction consistency (8 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Extracting Container<Leaf> with matching target always succeeds.
    #[test]
    fn extract_matching_always_succeeds(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let (inner, extracted) = try_extract_inner_type(&ty, container, &empty_skip());
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Extracting a plain leaf never succeeds regardless of target.
    #[test]
    fn extract_plain_leaf_never_succeeds(
        leaf in leaf_type(),
        target in container_name(),
    ) {
        let ty: Type = parse_str(leaf).unwrap();
        let (result, extracted) = try_extract_inner_type(&ty, target, &empty_skip());
        prop_assert!(!extracted);
        prop_assert_eq!(to_str(&result), leaf);
    }

    /// Extraction is deterministic: calling twice yields identical results.
    #[test]
    fn extract_is_deterministic(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let (inner1, ok1) = try_extract_inner_type(&ty, container, &empty_skip());
        let (inner2, ok2) = try_extract_inner_type(&ty, container, &empty_skip());
        prop_assert_eq!(ok1, ok2);
        prop_assert_eq!(to_str(&inner1), to_str(&inner2));
    }

    /// Mismatched target name does not extract.
    #[test]
    fn extract_mismatched_target_fails(
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let (_result, extracted) = try_extract_inner_type(&ty, "Option", &empty_skip());
        prop_assert!(!extracted);
    }

    /// Extraction through a skip layer reaches inner target.
    #[test]
    fn extract_through_skip_layer(
        skipper in skip_member(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{skipper}<Vec<{leaf}>>")).unwrap();
        let arr = [skipper];
        let s = skip_set(&arr);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &s);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Skip layer without matching target returns original unchanged.
    #[test]
    fn extract_skip_without_target_returns_original(
        skipper in skip_member(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{skipper}<{leaf}>")).unwrap();
        let arr = [skipper];
        let s = skip_set(&arr);
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &s);
        prop_assert!(!extracted);
        prop_assert_eq!(to_str(&result), format!("{skipper} < {leaf} >"));
    }

    /// Extracting from Option<Leaf> with various skip sets always succeeds.
    #[test]
    fn extract_option_ignores_skip_set(
        leaf in leaf_type(),
        skipper in skip_member(),
    ) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let arr = [skipper];
        let s = skip_set(&arr);
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &s);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Extract and filter agree: both yield the same leaf from Wrapper<Leaf>.
    #[test]
    fn extract_and_filter_agree(
        wrapper in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{wrapper}<{leaf}>")).unwrap();
        let (extracted_inner, ok) = try_extract_inner_type(&ty, wrapper, &empty_skip());
        let filtered = filter_inner_type(&ty, &skip_set(&[wrapper]));
        prop_assert!(ok);
        prop_assert_eq!(to_str(&extracted_inner), to_str(&filtered));
    }
}

// ===========================================================================
// 2. Filter specificity (10 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Filter removes a single skip container to expose the leaf.
    #[test]
    fn filter_single_skip_exposes_leaf(
        container in skip_member(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip_set(&[container]));
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filter with empty skip set is always identity.
    #[test]
    fn filter_empty_skip_is_identity(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let filtered = filter_inner_type(&ty, &empty_skip());
        prop_assert_eq!(to_str(&filtered), to_str(&ty));
    }

    /// Filter is idempotent: applying twice gives the same result.
    #[test]
    fn filter_is_idempotent(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let arr = [container];
        let s = skip_set(&arr);
        let once = filter_inner_type(&ty, &s);
        let twice = filter_inner_type(&once, &s);
        prop_assert_eq!(to_str(&once), to_str(&twice));
    }

    /// Filter on leaf types is idempotent.
    #[test]
    fn filter_leaf_is_idempotent(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let s = skip_set(&["Box", "Arc", "Rc"]);
        let once = filter_inner_type(&ty, &s);
        let twice = filter_inner_type(&once, &s);
        prop_assert_eq!(to_str(&once), to_str(&twice));
    }

    /// Filter strips Box<Arc<Leaf>> with both in skip set.
    #[test]
    fn filter_double_skip_strips_both(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<{leaf}>>")).unwrap();
        let s = skip_set(&["Box", "Arc"]);
        let filtered = filter_inner_type(&ty, &s);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filter non-skip container returns original.
    #[test]
    fn filter_non_skip_container_unchanged(
        non_skip in non_skip_container(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{non_skip}<{leaf}>")).unwrap();
        let s = skip_set(&["Box", "Arc", "Rc"]);
        let filtered = filter_inner_type(&ty, &s);
        prop_assert_eq!(to_str(&filtered), format!("{non_skip} < {leaf} >"));
    }

    /// Filter Box<NonSkip<Leaf>> with only Box in skip yields NonSkip<Leaf>.
    #[test]
    fn filter_outer_skip_preserves_inner(
        non_skip in non_skip_container(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Box<{non_skip}<{leaf}>>")).unwrap();
        let s = skip_set(&["Box"]);
        let filtered = filter_inner_type(&ty, &s);
        prop_assert_eq!(to_str(&filtered), format!("{non_skip} < {leaf} >"));
    }

    /// Filter determinism: same input always produces same output.
    #[test]
    fn filter_is_deterministic(ts in type_string_shallow()) {
        let ty: Type = parse_str(&ts).unwrap();
        let s = skip_set(&["Box", "Arc"]);
        let r1 = filter_inner_type(&ty, &s);
        let r2 = filter_inner_type(&ty, &s);
        prop_assert_eq!(to_str(&r1), to_str(&r2));
    }

    /// Filter Rc<Leaf> with Rc in skip yields leaf.
    #[test]
    fn filter_rc_in_skip_yields_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Rc<{leaf}>")).unwrap();
        let s = skip_set(&["Rc"]);
        let filtered = filter_inner_type(&ty, &s);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Filter Rc<Box<Leaf>> with both in skip yields leaf.
    #[test]
    fn filter_rc_box_both_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Rc<Box<{leaf}>>")).unwrap();
        let s = skip_set(&["Rc", "Box"]);
        let filtered = filter_inner_type(&ty, &s);
        prop_assert_eq!(to_str(&filtered), leaf);
    }
}

// ===========================================================================
// 3. Wrapping determinism (10 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Wrapping a plain leaf always produces `adze::WithLeaf<Leaf>`.
    #[test]
    fn wrap_plain_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        prop_assert_eq!(to_str(&wrapped), format!("adze :: WithLeaf < {leaf} >"));
    }

    /// Wrapping is deterministic: same input same output.
    #[test]
    fn wrap_is_deterministic(ts in type_string_shallow()) {
        let ty: Type = parse_str(&ts).unwrap();
        let s = skip_set(&["Vec", "Option", "Box", "Arc", "Rc"]);
        let w1 = wrap_leaf_type(&ty, &s);
        let w2 = wrap_leaf_type(&ty, &s);
        prop_assert_eq!(to_str(&w1), to_str(&w2));
    }

    /// Wrapping a skip container wraps the inner leaf, not the container.
    #[test]
    fn wrap_skip_container_wraps_inner(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let arr = [container];
        let s = skip_set(&arr);
        let wrapped = wrap_leaf_type(&ty, &s);
        let expected = format!("{container} < adze :: WithLeaf < {leaf} > >");
        prop_assert_eq!(to_str(&wrapped), expected);
    }

    /// Wrapping a non-skip container wraps the whole type.
    #[test]
    fn wrap_non_skip_wraps_entirely(
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let s = empty_skip();
        let wrapped = wrap_leaf_type(&ty, &s);
        prop_assert!(to_str(&wrapped).starts_with("adze :: WithLeaf <"));
    }

    /// Wrapping Vec<Leaf> with Vec in skip puts WithLeaf around leaf only.
    #[test]
    fn wrap_vec_skip_wraps_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let s = skip_set(&["Vec"]);
        let wrapped = wrap_leaf_type(&ty, &s);
        prop_assert_eq!(
            to_str(&wrapped),
            format!("Vec < adze :: WithLeaf < {leaf} > >")
        );
    }

    /// Wrapping Option<Leaf> with Option in skip.
    #[test]
    fn wrap_option_skip_wraps_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{leaf}>")).unwrap();
        let s = skip_set(&["Option"]);
        let wrapped = wrap_leaf_type(&ty, &s);
        prop_assert_eq!(
            to_str(&wrapped),
            format!("Option < adze :: WithLeaf < {leaf} > >")
        );
    }

    /// Wrapping Vec<Option<Leaf>> with both in skip wraps innermost leaf.
    #[test]
    fn wrap_nested_skip_wraps_innermost(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<Option<{leaf}>>")).unwrap();
        let s = skip_set(&["Vec", "Option"]);
        let wrapped = wrap_leaf_type(&ty, &s);
        prop_assert_eq!(
            to_str(&wrapped),
            format!("Vec < Option < adze :: WithLeaf < {leaf} > > >")
        );
    }

    /// wrap_leaf_type output always contains "adze :: WithLeaf" somewhere.
    #[test]
    fn wrap_always_contains_with_leaf(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let s = skip_set(&["Vec", "Option", "Box", "Arc", "Rc"]);
        let wrapped = wrap_leaf_type(&ty, &s);
        prop_assert!(to_str(&wrapped).contains("adze :: WithLeaf"));
    }

    /// Wrapping a deep nested type with all containers in skip wraps only leaf.
    #[test]
    fn wrap_deep_nested_wraps_leaf(ts in type_string_deep()) {
        let ty: Type = parse_str(&ts).unwrap();
        let s = skip_set(&["Vec", "Option", "Box", "Arc", "Rc"]);
        let wrapped = wrap_leaf_type(&ty, &s);
        let output = to_str(&wrapped);
        // The innermost leaf should be wrapped
        prop_assert!(output.contains("adze :: WithLeaf"));
    }

    /// Wrapping Result<A, B> with Result in skip wraps both args.
    #[test]
    fn wrap_result_wraps_both_type_args(
        leaf_a in leaf_type(),
        leaf_b in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("Result<{leaf_a}, {leaf_b}>")).unwrap();
        let s = skip_set(&["Result"]);
        let wrapped = wrap_leaf_type(&ty, &s);
        let output = to_str(&wrapped);
        let expected_a = format!("adze :: WithLeaf < {} >", leaf_a);
        let expected_b = format!("adze :: WithLeaf < {} >", leaf_b);
        prop_assert!(output.contains(&expected_a));
        prop_assert!(output.contains(&expected_b));
    }
}

// ===========================================================================
// 4. Parameterized detection correctness (12 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// All leaf primitives are not parameterized.
    #[test]
    fn leaf_primitives_not_parameterized(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        prop_assert!(!is_parameterized(&ty));
    }

    /// All Container<Leaf> types are parameterized.
    #[test]
    fn container_leaf_is_parameterized(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        prop_assert!(is_parameterized(&ty));
    }

    /// Nested Container<Container<Leaf>> is parameterized.
    #[test]
    fn nested_container_is_parameterized(
        c1 in container_name(),
        c2 in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{c1}<{c2}<{leaf}>>")).unwrap();
        prop_assert!(is_parameterized(&ty));
    }

    /// Tuples are never parameterized (not Type::Path).
    #[test]
    fn tuple_types_not_parameterized(
        a in leaf_type(),
        b in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("({a}, {b})")).unwrap();
        prop_assert!(!is_parameterized(&ty));
    }

    /// Reference types are not parameterized.
    #[test]
    fn reference_types_not_parameterized(leaf in leaf_type()) {
        // &leaf is not always valid syntax; use &i32 etc.
        let ty: Type = parse_str(&format!("&{leaf}")).unwrap();
        prop_assert!(!is_parameterized(&ty));
    }

    /// Array types are not parameterized.
    #[test]
    fn array_types_not_parameterized(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("[{leaf}; 4]")).unwrap();
        prop_assert!(!is_parameterized(&ty));
    }

    /// Parameterized detection is deterministic.
    #[test]
    fn parameterized_is_deterministic(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let r1 = is_parameterized(&ty);
        let r2 = is_parameterized(&ty);
        prop_assert_eq!(r1, r2);
    }

    /// Parameterized agrees with extraction: if parameterized and extraction
    /// with the correct target succeeds, we get the inner type.
    #[test]
    fn parameterized_implies_extractable(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        prop_assert!(is_parameterized(&ty));
        let (_inner, extracted) = try_extract_inner_type(&ty, container, &empty_skip());
        prop_assert!(extracted);
    }

    /// Non-parameterized leaf implies extraction fails for any target.
    #[test]
    fn non_parameterized_implies_extract_fails(
        leaf in leaf_type(),
        target in container_name(),
    ) {
        let ty: Type = parse_str(leaf).unwrap();
        prop_assert!(!is_parameterized(&ty));
        let (_result, extracted) = try_extract_inner_type(&ty, target, &empty_skip());
        prop_assert!(!extracted);
    }

    /// Extracted inner type from Container<Leaf> is not parameterized when leaf is primitive.
    #[test]
    fn extracted_inner_is_not_parameterized(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let (inner, ok) = try_extract_inner_type(&ty, container, &empty_skip());
        prop_assert!(ok);
        prop_assert!(!is_parameterized(&inner));
    }

    /// Filtered type from skip Container<Leaf> is not parameterized.
    #[test]
    fn filtered_leaf_not_parameterized(
        container in skip_member(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip_set(&[container]));
        prop_assert!(!is_parameterized(&filtered));
    }

    /// Wrapped leaf is always parameterized (because adze::WithLeaf<T> has generics).
    #[test]
    fn wrapped_leaf_is_parameterized(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let wrapped = wrap_leaf_type(&ty, &empty_skip());
        prop_assert!(is_parameterized(&wrapped));
    }
}

// ===========================================================================
// 5. Cross-function invariants (4 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Filter then wrap: filter removes skipped wrapper, wrap adds WithLeaf.
    #[test]
    fn filter_then_wrap_adds_with_leaf(
        container in skip_member(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip_set(&[container]));
        let wrapped = wrap_leaf_type(&filtered, &empty_skip());
        prop_assert_eq!(
            to_str(&wrapped),
            format!("adze :: WithLeaf < {leaf} >")
        );
    }

    /// Extract then wrap: extracted leaf gets WithLeaf wrapper.
    #[test]
    fn extract_then_wrap(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let (inner, ok) = try_extract_inner_type(&ty, container, &empty_skip());
        prop_assert!(ok);
        let wrapped = wrap_leaf_type(&inner, &empty_skip());
        prop_assert_eq!(to_str(&wrapped), format!("adze :: WithLeaf < {leaf} >"));
    }

    /// Parameterized detection on wrapped output is always true.
    #[test]
    fn wrap_output_always_parameterized(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let s = skip_set(&["Vec", "Option", "Box", "Arc", "Rc"]);
        let wrapped = wrap_leaf_type(&ty, &s);
        prop_assert!(is_parameterized(&wrapped));
    }

    /// Filter idempotence on deep types: filtering twice gives same result.
    #[test]
    fn filter_deep_idempotent(ts in type_string_deep()) {
        let ty: Type = parse_str(&ts).unwrap();
        let s = skip_set(&["Box", "Arc", "Rc"]);
        let once = filter_inner_type(&ty, &s);
        let twice = filter_inner_type(&once, &s);
        prop_assert_eq!(to_str(&once), to_str(&twice));
    }
}
