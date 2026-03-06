//! Property-based tests (v4) for adze-common type operations.
//!
//! Covers: try_extract_inner_type, filter_inner_type, wrap_leaf_type,
//! and is_parameterized with 50+ proptest cases across 8 categories.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Valid Rust identifiers avoiding all keywords (including 2024-edition `gen`).
fn ident_name() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{1,8}".prop_filter("must not be Self or keyword", |s| {
        !matches!(
            s.as_str(),
            "Self" | "As" | "Box" | "Vec" | "Option" | "Arc" | "Rc" | "Result" | "Gen"
        )
    })
}

/// Known primitive leaf types.
fn leaf_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Common generic containers.
fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec", "Box", "Arc", "Rc"][..])
}

/// Nested type string with depth 0-3.
fn nested_type_string() -> impl Strategy<Value = String> {
    let d0 = leaf_type().prop_map(|s| s.to_string());
    let d1 = (container_name(), leaf_type()).prop_map(|(c, l)| format!("{c}<{l}>"));
    let d2 = (container_name(), container_name(), leaf_type())
        .prop_map(|(c1, c2, l)| format!("{c1}<{c2}<{l}>>"));
    let d3 = (
        container_name(),
        container_name(),
        container_name(),
        leaf_type(),
    )
        .prop_map(|(c1, c2, c3, l)| format!("{c1}<{c2}<{c3}<{l}>>>"));
    prop_oneof![d0, d1, d2, d3]
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

fn skip1(a: &str) -> HashSet<&str> {
    [a].into_iter().collect()
}

fn skip_static(items: &[&'static str]) -> HashSet<&'static str> {
    items.iter().copied().collect()
}

// ---------------------------------------------------------------------------
// 1. Extract + filter roundtrip properties (8 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Extracting from Container<Leaf> with matching target yields the leaf.
    #[test]
    fn extract_matching_container_yields_leaf(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, container, &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Filter on Container<Leaf> where container is in skip set yields leaf.
    #[test]
    fn filter_skip_container_yields_leaf(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip = skip1(container);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Extract then filter: if extraction succeeds, filtering the extracted
    /// type with empty skip set is identity.
    #[test]
    fn extract_then_filter_identity(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, container, &skip);
        prop_assert!(extracted);
        let filtered = filter_inner_type(&inner, &skip);
        prop_assert_eq!(to_str(&filtered), to_str(&inner));
    }

    /// Filter through Box<Vec<Leaf>> with Box in skip yields Vec<Leaf>.
    #[test]
    fn filter_box_vec_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Vec<{leaf}>>")).unwrap();
        let skip = skip_static(&["Box"]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), format!("Vec < {leaf} >"));
    }

    /// Filter through Box<Arc<Leaf>> with both in skip yields leaf.
    #[test]
    fn filter_double_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<{leaf}>>")).unwrap();
        let skip = skip_static(&["Box", "Arc"]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Extract through skip: Box<Option<Leaf>> with Box skipped, target Option.
    #[test]
    fn extract_through_skip_box_option(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Option<{leaf}>>")).unwrap();
        let skip = skip_static(&["Box"]);
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Filter and extract agree: filter(Box<Leaf>, {Box}) == extract(Box<Leaf>, "Box", {}).0
    #[test]
    fn filter_extract_agreement(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip_static(&["Box"]));
        let (extracted, success) = try_extract_inner_type(&ty, "Box", &HashSet::new());
        prop_assert!(success);
        prop_assert_eq!(to_str(&filtered), to_str(&extracted));
    }

    /// Extracting Arc<Vec<Leaf>> with Arc skip yields leaf from Vec.
    #[test]
    fn extract_arc_skip_vec_target(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Arc<Vec<{leaf}>>")).unwrap();
        let skip = skip_static(&["Arc"]);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }
}

// ---------------------------------------------------------------------------
// 2. Filter is idempotent (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Filtering a leaf type is idempotent.
    #[test]
    fn filter_leaf_idempotent(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip = skip_static(&["Box", "Arc"]);
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(to_str(&once), to_str(&twice));
    }

    /// Filtering Container<Leaf> is idempotent after first application.
    #[test]
    fn filter_container_leaf_idempotent(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip = skip1(container);
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(to_str(&once), to_str(&twice));
    }

    /// Filtering nested Box<Arc<Leaf>> is idempotent.
    #[test]
    fn filter_nested_idempotent(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<{leaf}>>")).unwrap();
        let skip = skip_static(&["Box", "Arc"]);
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(to_str(&once), to_str(&twice));
    }

    /// Filtering with empty skip set is always idempotent (identity).
    #[test]
    fn filter_empty_skip_idempotent(type_str in nested_type_string()) {
        let ty: Type = parse_str(&type_str).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(to_str(&once), to_str(&twice));
    }

    /// Triple application of filter equals single application.
    #[test]
    fn filter_triple_equals_single(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip = skip1(container);
        let once = filter_inner_type(&ty, &skip);
        let triple = filter_inner_type(&filter_inner_type(&once, &skip), &skip);
        prop_assert_eq!(to_str(&once), to_str(&triple));
    }
}

// ---------------------------------------------------------------------------
// 3. Wrap produces parameterized type (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Wrapping a leaf type always produces a parameterized type.
    #[test]
    fn wrap_leaf_is_parameterized(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        prop_assert!(is_parameterized(&wrapped));
    }

    /// Wrapping a CamelCase ident always produces a parameterized type.
    #[test]
    fn wrap_custom_ident_is_parameterized(name in ident_name()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        prop_assert!(is_parameterized(&wrapped));
    }

    /// Wrapping a container that is NOT in skip set wraps the whole thing.
    #[test]
    fn wrap_non_skip_container_is_parameterized(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        prop_assert!(is_parameterized(&wrapped));
        let s = to_str(&wrapped);
        prop_assert!(s.contains("WithLeaf"));
    }

    /// Wrapping Container<Leaf> with container in skip wraps the leaf inside.
    #[test]
    fn wrap_skip_container_wraps_inner(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip = skip1(container);
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = to_str(&wrapped);
        prop_assert!(s.contains("WithLeaf"));
        prop_assert!(s.starts_with(container));
    }

    /// Wrapping a leaf always adds "adze :: WithLeaf" prefix.
    #[test]
    fn wrap_leaf_has_adze_prefix(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = to_str(&wrapped);
        prop_assert!(s.starts_with("adze :: WithLeaf"));
    }
}

// ---------------------------------------------------------------------------
// 4. is_parameterized consistency (8 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Leaf types are not parameterized.
    #[test]
    fn leaf_not_parameterized(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        prop_assert!(!is_parameterized(&ty));
    }

    /// Container<Leaf> is parameterized.
    #[test]
    fn container_leaf_is_parameterized(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        prop_assert!(is_parameterized(&ty));
    }

    /// CamelCase ident alone is not parameterized.
    #[test]
    fn custom_ident_not_parameterized(name in ident_name()) {
        let ty: Type = parse_str(&name).unwrap();
        prop_assert!(!is_parameterized(&ty));
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

    /// After filtering away all containers, result may not be parameterized.
    #[test]
    fn filtered_leaf_not_parameterized(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip = skip1(container);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert!(!is_parameterized(&filtered));
    }

    /// After wrapping a leaf, result is always parameterized.
    #[test]
    fn wrapped_is_always_parameterized(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let wrapped = wrap_leaf_type(&ty, &HashSet::new());
        prop_assert!(is_parameterized(&wrapped));
    }

    /// Extraction success means inner type can be parsed.
    #[test]
    fn extracted_inner_is_valid_type(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let (inner, extracted) = try_extract_inner_type(&ty, container, &HashSet::new());
        prop_assert!(extracted);
        // The inner type should be a valid parseable type (leaf).
        prop_assert!(!is_parameterized(&inner));
    }

    /// is_parameterized agrees with string-level angle bracket check.
    #[test]
    fn parameterized_matches_string_check(type_str in nested_type_string()) {
        let ty: Type = parse_str(&type_str).unwrap();
        let has_angle = to_str(&ty).contains('<');
        prop_assert_eq!(is_parameterized(&ty), has_angle);
    }
}

// ---------------------------------------------------------------------------
// 5. Empty skip_over set behavior (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Empty skip set: filter returns type unchanged for any input.
    #[test]
    fn empty_skip_filter_identity(type_str in nested_type_string()) {
        let ty: Type = parse_str(&type_str).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), to_str(&ty));
    }

    /// Empty skip set: extract with matching target still works.
    #[test]
    fn empty_skip_extract_still_works(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, container, &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Empty skip set: extract with non-matching target returns false.
    #[test]
    fn empty_skip_extract_nomatch(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_inner, extracted) = try_extract_inner_type(&ty, "ZZZNever", &skip);
        prop_assert!(!extracted);
    }

    /// Empty skip set: wrap_leaf_type wraps everything with WithLeaf.
    #[test]
    fn empty_skip_wrap_always_wraps(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        prop_assert_eq!(
            to_str(&wrapped),
            format!("adze :: WithLeaf < {leaf} >")
        );
    }

    /// Empty skip set: wrap on Container<Leaf> wraps the entire thing.
    #[test]
    fn empty_skip_wrap_container_wraps_whole(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = to_str(&wrapped);
        prop_assert!(s.starts_with("adze :: WithLeaf"));
    }
}

// ---------------------------------------------------------------------------
// 6. Various type patterns (8 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Triple nesting: Box<Arc<Vec<Leaf>>> with all three in skip yields leaf.
    #[test]
    fn filter_triple_nesting(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<Vec<{leaf}>>>")).unwrap();
        let skip = skip_static(&["Box", "Arc", "Vec"]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Extract through double skip: Arc<Box<Option<Leaf>>>.
    #[test]
    fn extract_double_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Arc<Box<Option<{leaf}>>>")).unwrap();
        let skip = skip_static(&["Arc", "Box"]);
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Wrap nested container: Vec<Box<Leaf>> with Vec in skip wraps Box<Leaf>.
    #[test]
    fn wrap_nested_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<Box<{leaf}>>")).unwrap();
        let skip = skip_static(&["Vec"]);
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = to_str(&wrapped);
        // Vec stays, but Box<Leaf> gets wrapped
        prop_assert!(s.starts_with("Vec"));
        prop_assert!(s.contains("WithLeaf"));
    }

    /// Wrap with both outer and inner in skip: Vec<Option<Leaf>>.
    #[test]
    fn wrap_double_skip_nesting(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<Option<{leaf}>>")).unwrap();
        let skip = skip_static(&["Vec", "Option"]);
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = to_str(&wrapped);
        prop_assert!(s.starts_with("Vec"));
        prop_assert!(s.contains("Option"));
        prop_assert!(s.contains("WithLeaf"));
    }

    /// Filter with only inner container in skip: Box<Arc<Leaf>> skip Arc.
    #[test]
    fn filter_inner_only_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<{leaf}>>")).unwrap();
        let skip = skip_static(&["Arc"]);
        let filtered = filter_inner_type(&ty, &skip);
        // Box is not in skip, so type is returned unchanged.
        prop_assert_eq!(to_str(&filtered), to_str(&ty));
    }

    /// Rc<Leaf> filter with Rc in skip.
    #[test]
    fn filter_rc_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Rc<{leaf}>")).unwrap();
        let skip = skip_static(&["Rc"]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), leaf);
    }

    /// Extract from Rc<Leaf> with target Rc.
    #[test]
    fn extract_rc(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Rc<{leaf}>")).unwrap();
        let (inner, extracted) = try_extract_inner_type(&ty, "Rc", &HashSet::new());
        prop_assert!(extracted);
        prop_assert_eq!(to_str(&inner), leaf);
    }

    /// Wrap Rc<Leaf> with Rc in skip wraps only the leaf.
    #[test]
    fn wrap_rc_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Rc<{leaf}>")).unwrap();
        let skip = skip_static(&["Rc"]);
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = to_str(&wrapped);
        prop_assert!(s.starts_with("Rc"));
        prop_assert!(s.contains("WithLeaf"));
    }
}

// ---------------------------------------------------------------------------
// 7. Cross-function consistency (6 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Filter then wrap: filter(Box<Leaf>, {Box}) then wrap(Leaf, {}) == WithLeaf<Leaf>.
    #[test]
    fn filter_then_wrap(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<{leaf}>")).unwrap();
        let filtered = filter_inner_type(&ty, &skip_static(&["Box"]));
        let wrapped = wrap_leaf_type(&filtered, &HashSet::new());
        let s = to_str(&wrapped);
        prop_assert_eq!(s, format!("adze :: WithLeaf < {leaf} >"));
    }

    /// Extract and filter agree on simple Container<Leaf>.
    #[test]
    fn extract_filter_simple_agreement(
        container in container_name(),
        leaf in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{leaf}>")).unwrap();
        let (extracted, _) = try_extract_inner_type(&ty, container, &HashSet::new());
        let filtered = filter_inner_type(&ty, &skip1(container));
        prop_assert_eq!(to_str(&extracted), to_str(&filtered));
    }

    /// Wrap then check parameterized: wrapping any nested type is parameterized.
    #[test]
    fn wrap_nested_always_parameterized(type_str in nested_type_string()) {
        let ty: Type = parse_str(&type_str).unwrap();
        // Use empty skip so everything gets wrapped.
        let wrapped = wrap_leaf_type(&ty, &HashSet::new());
        prop_assert!(is_parameterized(&wrapped));
    }

    /// Filter preserves non-skip containers.
    #[test]
    fn filter_preserves_non_skip(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        // Skip set doesn't include Vec.
        let skip = skip_static(&["Box", "Arc"]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), to_str(&ty));
    }

    /// Extraction failure returns the original type unchanged.
    #[test]
    fn extract_failure_returns_original(type_str in nested_type_string()) {
        let ty: Type = parse_str(&type_str).unwrap();
        let (_inner, extracted) = try_extract_inner_type(&ty, "ZZZNever", &HashSet::new());
        prop_assert!(!extracted);
        prop_assert_eq!(to_str(&_inner), to_str(&ty));
    }

    /// Wrap idempotence check: wrapping twice nests WithLeaf twice.
    #[test]
    fn wrap_twice_double_nests(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let once = wrap_leaf_type(&ty, &skip);
        let twice = wrap_leaf_type(&once, &skip);
        let s = to_str(&twice);
        // Should contain nested WithLeaf.
        prop_assert!(s.starts_with("adze :: WithLeaf < adze :: WithLeaf"));
    }
}

// ---------------------------------------------------------------------------
// 8. Edge cases (5 tests)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    /// Non-path type (reference) is returned unchanged by filter.
    #[test]
    fn filter_reference_type_unchanged(_leaf in leaf_type()) {
        let ty: Type = parse_quote!(&str);
        let skip = skip_static(&["Box", "Arc"]);
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(to_str(&filtered), to_str(&ty));
    }

    /// Non-path type (reference) extract returns false.
    #[test]
    fn extract_reference_type_not_extracted(_leaf in leaf_type()) {
        let ty: Type = parse_quote!(&str);
        let (_inner, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
        prop_assert!(!extracted);
    }

    /// Non-path type gets wrapped by wrap_leaf_type.
    #[test]
    fn wrap_reference_type(_leaf in leaf_type()) {
        let ty: Type = parse_quote!(&str);
        let wrapped = wrap_leaf_type(&ty, &HashSet::new());
        let s = to_str(&wrapped);
        prop_assert!(s.contains("WithLeaf"));
    }

    /// Tuple type is not parameterized.
    #[test]
    fn tuple_type_not_parameterized(_leaf in leaf_type()) {
        let ty: Type = parse_quote!((i32, u32));
        prop_assert!(!is_parameterized(&ty));
    }

    /// Container with same name as target but different casing: no match.
    #[test]
    fn case_sensitive_extract(leaf in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{leaf}>")).unwrap();
        let (_inner, extracted) = try_extract_inner_type(&ty, "vec", &HashSet::new());
        prop_assert!(!extracted);
    }
}
