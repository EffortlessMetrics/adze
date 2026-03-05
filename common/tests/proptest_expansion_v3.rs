//! Property-based tests for adze-common expansion functions (v3).
//!
//! Covers: filter_inner_type idempotency, wrap_leaf_type validity,
//! try_extract_inner_type consistency, random type names, empty/large skip sets,
//! and cross-function consistency.

use adze_common::*;
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_str};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Set of all Rust keywords (2024 edition) that cannot be used as identifiers.
const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "gen", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try",
];

fn is_keyword(s: &str) -> bool {
    RUST_KEYWORDS.contains(&s)
}

/// Generates a valid Rust identifier that is not a keyword.
fn valid_ident() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{0,9}".prop_filter("must not be a Rust keyword", |s| !is_keyword(s))
}

/// Generates a lowercase identifier safe for use as a type name.
fn lower_ident() -> impl Strategy<Value = String> {
    "[a-z]{2,8}".prop_filter("must not be a Rust keyword", |s| !is_keyword(s))
}

/// Known leaf type names that are always valid.
fn leaf_type_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(
        &[
            "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "bool", "char",
            "String", "usize", "isize",
        ][..],
    )
}

/// Common container names.
fn container_name() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["Option", "Vec", "Box", "Arc", "Rc"][..])
}

/// Build a `Container<leaf>` type string.
fn single_wrapped() -> impl Strategy<Value = String> {
    (container_name(), leaf_type_name()).prop_map(|(c, l)| format!("{c}<{l}>"))
}

/// Build a `C1<C2<leaf>>` type string.
fn double_wrapped() -> impl Strategy<Value = String> {
    (container_name(), container_name(), leaf_type_name())
        .prop_map(|(c1, c2, l)| format!("{c1}<{c2}<{l}>>"))
}

/// Build a `C1<C2<C3<leaf>>>` type string.
fn triple_wrapped() -> impl Strategy<Value = String> {
    (
        container_name(),
        container_name(),
        container_name(),
        leaf_type_name(),
    )
        .prop_map(|(c1, c2, c3, l)| format!("{c1}<{c2}<{c3}<{l}>>>"))
}

/// Any nesting depth 0–3.
fn any_type_string() -> impl Strategy<Value = String> {
    prop_oneof![
        leaf_type_name().prop_map(|s| s.to_string()),
        single_wrapped(),
        double_wrapped(),
        triple_wrapped(),
    ]
}

/// Helper: stringify a type via token stream.
fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. filter_inner_type idempotency: filter(filter(x)) == filter(x)  (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn filter_idempotent_box(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    #[test]
    fn filter_idempotent_arc(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Arc"].into_iter().collect();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    #[test]
    fn filter_idempotent_rc(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Rc"].into_iter().collect();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    #[test]
    fn filter_idempotent_option(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Option"].into_iter().collect();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    #[test]
    fn filter_idempotent_vec(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    #[test]
    fn filter_idempotent_box_arc(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    #[test]
    fn filter_idempotent_all_containers(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc", "Rc", "Option", "Vec"].into_iter().collect();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    #[test]
    fn filter_idempotent_random_ident(name in valid_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }
}

// ===========================================================================
// 2. wrap_leaf_type always produces valid types  (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn wrap_produces_parseable_type_leaf(name in leaf_type_name()) {
        let ty: Type = parse_str(name).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        // Must be a valid type — re-parse the token stream.
        let s = ty_str(&wrapped);
        let _reparsed: Type = parse_str(&s).unwrap();
    }

    #[test]
    fn wrap_produces_parseable_type_single(ts in single_wrapped()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option", "Box", "Arc", "Rc"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = ty_str(&wrapped);
        let _reparsed: Type = parse_str(&s).unwrap();
    }

    #[test]
    fn wrap_produces_parseable_type_double(ts in double_wrapped()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option", "Box", "Arc", "Rc"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = ty_str(&wrapped);
        let _reparsed: Type = parse_str(&s).unwrap();
    }

    #[test]
    fn wrap_produces_parseable_type_triple(ts in triple_wrapped()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option", "Box", "Arc", "Rc"].into_iter().collect();
        let wrapped = wrap_leaf_type(&ty, &skip);
        let s = ty_str(&wrapped);
        let _reparsed: Type = parse_str(&s).unwrap();
    }

    #[test]
    fn wrap_deterministic(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
        let a = ty_str(&wrap_leaf_type(&ty, &skip));
        let b = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert_eq!(a, b);
    }

    #[test]
    fn wrap_leaf_contains_with_leaf(name in leaf_type_name()) {
        let ty: Type = parse_str(name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains("WithLeaf"), "leaf must be wrapped: {wrapped}");
    }

    #[test]
    fn wrap_skip_container_preserves_outer(cname in container_name(), lname in leaf_type_name()) {
        let ts = format!("{cname}<{lname}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = [cname].into_iter().collect();
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        // Outer container name is still present.
        prop_assert!(wrapped.contains(cname), "outer container preserved: {wrapped}");
        // Inner leaf is wrapped.
        prop_assert!(wrapped.contains("WithLeaf"), "inner leaf wrapped: {wrapped}");
    }

    #[test]
    fn wrap_random_ident_wraps(name in valid_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains("WithLeaf"));
    }
}

// ===========================================================================
// 3. try_extract_inner_type consistency  (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn extract_plain_ident_never_matches(name in valid_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted, "plain ident should not match Option");
    }

    #[test]
    fn extract_matching_outer_always_extracts(
        cname in container_name(),
        lname in leaf_type_name()
    ) {
        let ts = format!("{cname}<{lname}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, cname, &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), lname);
    }

    #[test]
    fn extract_non_matching_returns_original(lname in leaf_type_name()) {
        let ts = format!("Vec<{lname}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }

    #[test]
    fn extract_through_skip_finds_target(lname in leaf_type_name()) {
        let ts = format!("Box<Vec<{lname}>>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), lname);
    }

    #[test]
    fn extract_through_double_skip(lname in leaf_type_name()) {
        let ts = format!("Arc<Box<Option<{lname}>>>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Arc", "Box"].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), lname);
    }

    #[test]
    fn extract_skip_but_no_target_returns_original(lname in leaf_type_name()) {
        let ts = format!("Box<{lname}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }

    #[test]
    fn extract_is_deterministic(ts in single_wrapped()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let (a_inner, a_ext) = try_extract_inner_type(&ty, "Vec", &skip);
        let (b_inner, b_ext) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert_eq!(a_ext, b_ext);
        prop_assert_eq!(ty_str(&a_inner), ty_str(&b_inner));
    }

    #[test]
    fn extract_bool_matches_type_identity(
        cname in container_name(),
        lname in leaf_type_name()
    ) {
        let ts = format!("{cname}<{lname}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, cname, &skip);
        // If extracted, the inner must differ from the original.
        if extracted {
            prop_assert_ne!(ty_str(&inner), ty_str(&ty));
        }
    }
}

// ===========================================================================
// 4. Random type names maintain properties  (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn random_name_filter_no_crash(name in valid_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
        let _ = filter_inner_type(&ty, &skip);
    }

    #[test]
    fn random_name_wrap_no_crash(name in valid_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
        let _ = wrap_leaf_type(&ty, &skip);
    }

    #[test]
    fn random_name_extract_no_crash(name in valid_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let _ = try_extract_inner_type(&ty, "Option", &skip);
    }

    #[test]
    fn random_name_filter_preserves_ident(name in valid_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), name);
    }

    #[test]
    fn random_name_wrap_adds_with_leaf(name in valid_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains("WithLeaf"));
        prop_assert!(wrapped.contains(&name));
    }

    #[test]
    fn random_name_extract_returns_false(name in valid_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(!extracted);
    }

    #[test]
    fn random_lower_name_filter_preserves(name in lower_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), name);
    }

    #[test]
    fn random_lower_name_wrap_always_wraps(name in lower_ident()) {
        let ty: Type = parse_str(&name).unwrap();
        let skip: HashSet<&str> = ["Vec"].into_iter().collect();
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains("WithLeaf"));
    }
}

// ===========================================================================
// 5. Empty skip set behavior  (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn empty_skip_filter_is_identity(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), ty_str(&ty));
    }

    #[test]
    fn empty_skip_extract_no_skip_through(lname in leaf_type_name()) {
        let ts = format!("Box<Vec<{lname}>>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        // Box is not in skip set, and it's not "Vec", so extraction fails.
        let (_, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(!extracted);
    }

    #[test]
    fn empty_skip_wrap_always_wraps_everything(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains("WithLeaf"));
    }

    #[test]
    fn empty_skip_extract_direct_match(cname in container_name(), lname in leaf_type_name()) {
        let ts = format!("{cname}<{lname}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, cname, &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), lname);
    }

    #[test]
    fn empty_skip_filter_idempotent(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }
}

// ===========================================================================
// 6. Large skip sets  (5 tests)
// ===========================================================================

fn large_skip_set() -> HashSet<&'static str> {
    [
        "Box", "Arc", "Rc", "Option", "Vec", "Cell", "RefCell", "Mutex", "RwLock", "Pin",
    ]
    .into_iter()
    .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn large_skip_filter_strips_known(
        cname in prop::sample::select(&["Box", "Arc", "Rc"][..]),
        lname in leaf_type_name()
    ) {
        let ts = format!("{cname}<{lname}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip = large_skip_set();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), lname);
    }

    #[test]
    fn large_skip_filter_idempotent(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip = large_skip_set();
        let once = filter_inner_type(&ty, &skip);
        let twice = filter_inner_type(&once, &skip);
        prop_assert_eq!(ty_str(&once), ty_str(&twice));
    }

    #[test]
    fn large_skip_extract_through_many(lname in leaf_type_name()) {
        let ts = format!("Box<Arc<Vec<{lname}>>>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip = large_skip_set();
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), lname);
    }

    #[test]
    fn large_skip_wrap_preserves_skipped(
        cname in prop::sample::select(&["Box", "Arc", "Rc", "Option", "Vec"][..]),
        lname in leaf_type_name()
    ) {
        let ts = format!("{cname}<{lname}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip = large_skip_set();
        let wrapped = ty_str(&wrap_leaf_type(&ty, &skip));
        prop_assert!(wrapped.contains(cname), "container preserved: {wrapped}");
        prop_assert!(wrapped.contains("WithLeaf"), "inner wrapped: {wrapped}");
    }

    #[test]
    fn large_skip_leaf_unaffected_by_filter(name in leaf_type_name()) {
        let ty: Type = parse_str(name).unwrap();
        let skip = large_skip_set();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), name);
    }
}

// ===========================================================================
// 7. Cross-function consistency  (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn filter_then_extract_fails(lname in leaf_type_name()) {
        // After filtering Box away, extracting through Box should fail.
        let ts = format!("Box<Vec<{lname}>>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        // filtered = Vec<lname>, now try to extract Vec through Box — Box is gone.
        let (_, extracted) = try_extract_inner_type(&filtered, "Vec", &["Box"].into_iter().collect());
        // Direct match still works, so extracted should be true.
        prop_assert!(extracted);
    }

    #[test]
    fn extract_then_wrap_gives_with_leaf(
        cname in container_name(),
        lname in leaf_type_name()
    ) {
        let ts = format!("{cname}<{lname}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, cname, &skip);
        prop_assert!(extracted);
        let wrapped = ty_str(&wrap_leaf_type(&inner, &skip));
        prop_assert!(wrapped.contains("WithLeaf"));
    }

    #[test]
    fn wrap_after_filter_still_valid(ts in single_wrapped()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip_filter: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
        let skip_wrap: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip_filter);
        let wrapped = wrap_leaf_type(&filtered, &skip_wrap);
        let s = ty_str(&wrapped);
        let _reparsed: Type = parse_str(&s).unwrap();
    }

    #[test]
    fn filter_preserves_extractability(lname in leaf_type_name()) {
        // Box<Vec<T>> → filter(Box) → Vec<T> → extract(Vec) succeeds
        let ts = format!("Box<Vec<{lname}>>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip_filter: HashSet<&str> = ["Box"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip_filter);
        let skip_extract: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&filtered, "Vec", &skip_extract);
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), lname);
    }

    #[test]
    fn wrap_idempotent_on_already_wrapped(name in leaf_type_name()) {
        // Wrapping a plain type, then wrapping again should double-wrap consistently.
        let ty: Type = parse_str(name).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let once = wrap_leaf_type(&ty, &skip);
        let twice = wrap_leaf_type(&once, &skip);
        let s = ty_str(&twice);
        // Should have two WithLeaf occurrences.
        let count = s.matches("WithLeaf").count();
        prop_assert!(count >= 2, "double wrap should have >=2 WithLeaf: {s}");
    }

    #[test]
    fn filter_no_op_then_wrap_same_as_direct_wrap(name in leaf_type_name()) {
        let ty: Type = parse_str(name).unwrap();
        let skip_filter: HashSet<&str> = ["Box"].into_iter().collect();
        let skip_wrap: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip_filter);
        let wrap_filtered = ty_str(&wrap_leaf_type(&filtered, &skip_wrap));
        let wrap_direct = ty_str(&wrap_leaf_type(&ty, &skip_wrap));
        prop_assert_eq!(wrap_filtered, wrap_direct);
    }

    #[test]
    fn extract_false_means_original_returned(ts in any_type_string()) {
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = HashSet::new();
        let (inner, extracted) = try_extract_inner_type(&ty, "NoSuchType", &skip);
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }

    #[test]
    fn filter_strip_then_no_further_stripping(
        cname in prop::sample::select(&["Box", "Arc", "Rc"][..]),
        lname in leaf_type_name()
    ) {
        let ts = format!("{cname}<{lname}>");
        let ty: Type = parse_str(&ts).unwrap();
        let skip: HashSet<&str> = [cname].into_iter().collect();
        let filtered = filter_inner_type(&ty, &skip);
        prop_assert_eq!(ty_str(&filtered), lname);
        // Filtering again should be no-op.
        let again = filter_inner_type(&filtered, &skip);
        prop_assert_eq!(ty_str(&again), lname);
    }
}
