#![allow(clippy::needless_range_loop)]

//! Property-based tests for type utility functions in adze-common.
//!
//! Focuses on type detection patterns (Option, Vec, Box), generic type
//! parameter extraction, path type handling, parse→format→parse roundtrips,
//! and complex nested type compositions.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn leaf_type() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Generates type strings with up to 3 levels of nesting using common containers.
fn nested_type_string() -> impl Strategy<Value = String> {
    let depth0 = leaf_type().prop_map(|s| s.to_string());
    let depth1 = (
        prop::sample::select(&["Option", "Vec", "Box"][..]),
        leaf_type(),
    )
        .prop_map(|(c, l)| format!("{c}<{l}>"));
    let depth2 = (
        prop::sample::select(&["Option", "Vec", "Box"][..]),
        prop::sample::select(&["Option", "Vec", "Box"][..]),
        leaf_type(),
    )
        .prop_map(|(c1, c2, l)| format!("{c1}<{c2}<{l}>>"));
    let depth3 = (
        prop::sample::select(&["Option", "Vec", "Box"][..]),
        prop::sample::select(&["Option", "Vec", "Box"][..]),
        prop::sample::select(&["Option", "Vec", "Box"][..]),
        leaf_type(),
    )
        .prop_map(|(c1, c2, c3, l)| format!("{c1}<{c2}<{c3}<{l}>>>"));
    prop_oneof![depth0, depth1, depth2, depth3]
}

/// Generates qualified path types like `std::collections::HashMap`.
fn qualified_path_type() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("std::string::String".to_string()),
        Just("std::vec::Vec<i32>".to_string()),
        Just("std::option::Option<bool>".to_string()),
        Just("std::boxed::Box<u8>".to_string()),
        Just("std::collections::HashMap<String, i32>".to_string()),
        Just("std::result::Result<i32, String>".to_string()),
    ]
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // ===== Option<T> detection =====

    // 1. Option<T> is always detected when target is "Option".
    #[test]
    fn option_always_detected(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted, "Option<{inner}> should be detected");
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 2. Option<T> is not detected when target is "Vec".
    #[test]
    fn option_not_detected_as_vec(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(!extracted, "Option should not match Vec target");
    }

    // 3. Nested Option<Option<T>> extracts outer layer first.
    #[test]
    fn nested_option_extracts_outer(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Option<{inner}>>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), format!("Option < {inner} >"));
    }

    // ===== Vec<T> detection =====

    // 4. Vec<T> is always detected when target is "Vec".
    #[test]
    fn vec_always_detected(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted, "Vec<{inner}> should be detected");
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 5. Vec<T> is not detected when target is "Option".
    #[test]
    fn vec_not_detected_as_option(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted, "Vec should not match Option target");
    }

    // 6. Vec<Vec<T>> extracts outer Vec, yielding Vec<T>.
    #[test]
    fn nested_vec_extracts_outer(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<Vec<{inner}>>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), format!("Vec < {inner} >"));
    }

    // ===== Box<T> detection =====

    // 7. Box<T> is always detected when target is "Box".
    #[test]
    fn box_always_detected(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, "Box", &skip);
        prop_assert!(extracted, "Box<{inner}> should be detected");
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 8. Box<T> is unwrapped by filter_inner_type when Box is in skip set.
    #[test]
    fn box_unwrapped_by_filter(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<{inner}>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), inner);
    }

    // 9. Box<Box<T>> is fully unwrapped by filter when Box is in skip set.
    #[test]
    fn nested_box_fully_unwrapped(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Box<{inner}>>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), inner);
    }

    // ===== Generic type parameter extraction =====

    // 10. Extracting through Box skip finds inner Option.
    #[test]
    fn extract_option_through_box(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Option<{inner}>>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 11. Extracting through Box skip finds inner Vec.
    #[test]
    fn extract_vec_through_box(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Vec<{inner}>>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 12. Multiple skip layers: Box<Arc<Option<T>>> extracts T with Box+Arc skip.
    #[test]
    fn extract_through_multiple_skips(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Arc<Option<{inner}>>>")).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 13. When skip contains the target itself, direct match still works.
    #[test]
    fn skip_containing_target_still_matches(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<{inner}>")).unwrap();
        let skip: HashSet<&str> = ["Option"].into_iter().collect();
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        // Direct match takes priority over skip-through
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), inner);
    }

    // ===== Path type handling =====

    // 14. Qualified path types are not matched by short container names.
    #[test]
    fn qualified_path_not_matched_by_short_name(qp in qualified_path_type()) {
        let ty: Type = parse_str(&qp).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        // "Vec" won't match "std::vec::Vec" because last segment is checked
        // but "Vec" IS the last segment for "std::vec::Vec<i32>", so this
        // tests the last-segment matching behavior.
        let (_, extracted) = try_extract_inner_type(&ty, "NONEXISTENT", &skip);
        prop_assert!(!extracted, "NONEXISTENT should never match");
    }

    // 15. Leaf types (no generics) are never extracted for any target.
    #[test]
    fn leaf_type_never_extracted(
        leaf in leaf_type(),
        target in prop::sample::select(&["Option", "Vec", "Box"][..]),
    ) {
        let ty: Type = parse_str(leaf).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, target, &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&result), leaf);
    }

    // 16. filter_inner_type on a leaf type is always identity regardless of skip set.
    #[test]
    fn filter_leaf_always_identity(
        leaf in leaf_type(),
        skip_names in prop::collection::hash_set(
            prop::sample::select(&["Box", "Vec", "Option", "Arc"][..]),
            0..=4,
        ),
    ) {
        let ty: Type = parse_str(leaf).unwrap();
        let filtered = filter_inner_type(&ty, &skip_names);
        prop_assert_eq!(ty_str(&filtered), leaf);
    }

    // ===== Type roundtrip: parse → format → parse =====

    // 17. Simple leaf types survive parse→format→parse roundtrip.
    #[test]
    fn roundtrip_leaf(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let formatted = ty_str(&ty);
        let reparsed: Type = parse_str(&formatted).unwrap();
        prop_assert_eq!(ty_str(&reparsed), formatted);
    }

    // 18. Container<Leaf> types survive roundtrip.
    #[test]
    fn roundtrip_single_container(
        container in prop::sample::select(&["Option", "Vec", "Box"][..]),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let formatted = ty_str(&ty);
        let reparsed: Type = parse_str(&formatted).unwrap();
        prop_assert_eq!(ty_str(&reparsed), formatted);
    }

    // 19. Nested types survive roundtrip.
    #[test]
    fn roundtrip_nested(ty_s in nested_type_string()) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let formatted = ty_str(&ty);
        let reparsed: Type = parse_str(&formatted).unwrap();
        prop_assert_eq!(ty_str(&reparsed), formatted);
    }

    // 20. Extraction result survives roundtrip.
    #[test]
    fn roundtrip_after_extraction(
        container in prop::sample::select(&["Option", "Vec", "Box"][..]),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, container, &skip);
        prop_assert!(extracted);
        let formatted = ty_str(&result);
        let reparsed: Type = parse_str(&formatted).unwrap();
        prop_assert_eq!(ty_str(&reparsed), formatted);
    }

    // 21. Wrapped result survives roundtrip.
    #[test]
    fn roundtrip_after_wrapping(leaf in leaf_type()) {
        let ty: Type = parse_str(leaf).unwrap();
        let wrapped = wrap_leaf_type(&ty, &HashSet::new());
        let formatted = ty_str(&wrapped);
        let reparsed: Type = parse_str(&formatted).unwrap();
        prop_assert_eq!(ty_str(&reparsed), formatted);
    }

    // 22. Filtered result survives roundtrip.
    #[test]
    fn roundtrip_after_filtering(
        container in prop::sample::select(&["Box", "Arc"][..]),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let skip: HashSet<&str> = [container].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        let formatted = ty_str(&filtered);
        let reparsed: Type = parse_str(&formatted).unwrap();
        prop_assert_eq!(ty_str(&reparsed), formatted);
    }

    // ===== Complex nested types =====

    // 23. Option<Vec<Box<T>>>: extract Option yields Vec<Box<T>>.
    #[test]
    fn complex_option_vec_box_extract(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Vec<Box<{inner}>>>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), format!("Vec < Box < {inner} > >"));
    }

    // 24. Vec<Option<T>>: extract Vec yields Option<T>.
    #[test]
    fn vec_option_extract(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Vec<Option<{inner}>>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&result), format!("Option < {inner} >"));
    }

    // 25. Box<Vec<Option<T>>>: filter through Box+Vec yields Option<T> when
    //     both Box and Vec are in skip set.
    //     (Actually filter_inner_type only strips wrappers from skip set, not targets.)
    #[test]
    fn filter_strips_all_skip_layers(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Vec<Option<{inner}>>>")).unwrap();
        let skip: HashSet<&str> = ["Box", "Vec"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), format!("Option < {inner} >"));
    }

    // 26. wrap_leaf_type on Option<Vec<T>> with both in skip wraps only inner T.
    #[test]
    fn wrap_nested_containers_in_skip(inner in leaf_type()) {
        let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
        let ty: Type = parse_str(&format!("Option<Vec<{inner}>>")).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = ty_str(&wrapped);
        prop_assert_eq!(
            s,
            format!("Option < Vec < adze :: WithLeaf < {inner} > > >")
        );
    }

    // 27. Deep nesting: Box<Box<Box<T>>> filter removes all Box layers.
    #[test]
    fn triple_box_filter(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Box<Box<Box<{inner}>>>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), inner);
    }

    // 28. Extracting Vec from Option<Vec<T>> fails without skip set (Option blocks).
    #[test]
    fn extract_blocked_without_skip(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Vec<{inner}>>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(!extracted, "Vec inside Option not reachable without skip");
    }

    // 29. Extracting Vec from Option<Vec<T>> succeeds with Option in skip set.
    #[test]
    fn extract_succeeds_with_skip(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Vec<{inner}>>")).unwrap();
        let skip: HashSet<&str> = ["Option"].into_iter().collect();
        let (result, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted, "Vec inside Option reachable with skip");
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 30. Consistency: extract + filter of same container both yield same inner type.
    #[test]
    fn extract_and_filter_consistent(
        container in prop::sample::select(&["Option", "Vec", "Box"][..]),
        inner in leaf_type(),
    ) {
        let ty: Type = parse_str(&format!("{container}<{inner}>")).unwrap();
        let skip_extract: HashSet<&str> = HashSet::new();
        let (extracted, ok) = try_extract_inner_type(&ty, container, &skip_extract);
        prop_assert!(ok);
        let skip_filter: HashSet<&str> = [container].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip_filter);
        prop_assert_eq!(ty_str(&extracted), ty_str(&filtered));
    }

    // 31. Wrapping deeply nested type produces exactly one WithLeaf per leaf.
    #[test]
    fn wrap_deeply_nested_single_withleaf(
        c1 in prop::sample::select(&["Option", "Vec", "Box"][..]),
        c2 in prop::sample::select(&["Option", "Vec", "Box"][..]),
        inner in leaf_type(),
    ) {
        let skip: HashSet<&str> = [c1, c2].into_iter().collect();
        let ty: Type = parse_str(&format!("{c1}<{c2}<{inner}>>")).unwrap();
        let s = ty_str(&wrap_leaf_type(&ty, &skip));
        let count = s.matches("WithLeaf").count();
        prop_assert!(count == 1, "exactly 1 WithLeaf in: {}", s);
    }

    // 32. All nested type strings parse and all three functions run without panic.
    #[test]
    fn all_functions_no_panic_on_nested(ty_s in nested_type_string()) {
        let ty: Type = parse_str(&ty_s).unwrap();
        let skip: HashSet<&str> = ["Box", "Option", "Vec"].into_iter().collect();
        let _ = try_extract_inner_type(&ty, "Option", &skip);
        let _ = filter_inner_type(&ty, &skip);
        let _ = wrap_leaf_type(&ty, &skip);
    }

    // 33. Qualified path types: last segment matching still works for extraction.
    #[test]
    fn qualified_option_last_segment_matches(inner in leaf_type()) {
        // std::option::Option<T> — last segment is "Option"
        let ty: Type = parse_str(&format!("std::option::Option<{inner}>")).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (result, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted, "last segment should match Option");
        prop_assert_eq!(ty_str(&result), inner);
    }

    // 34. Qualified path types: last segment matching works for filter.
    #[test]
    fn qualified_box_filter_last_segment(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("std::boxed::Box<{inner}>")).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), inner);
    }

    // 35. Complex: extract → filter → wrap pipeline produces valid type.
    #[test]
    fn pipeline_extract_filter_wrap(inner in leaf_type()) {
        let ty: Type = parse_str(&format!("Option<Box<{inner}>>")).unwrap();
        // Extract Option
        let skip: HashSet<&str> = HashSet::new();
        let (after_extract, ok) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(ok);
        // Filter Box
        let filter_skip: HashSet<&str> = ["Box"].into_iter().collect();
        let after_filter = filter_inner_type(&after_extract, &filter_skip);
        prop_assert_eq!(ty_str(&after_filter), inner);
        // Wrap leaf
        let wrapped = wrap_leaf_type(&after_filter, &HashSet::new());
        let s = ty_str(&wrapped);
        let expected = format!("adze :: WithLeaf < {inner} >");
        prop_assert_eq!(&s, &expected);
        // Roundtrip
        let reparsed: Type = parse_str(&s).unwrap();
        prop_assert_eq!(ty_str(&reparsed), s);
    }
}
