#![allow(clippy::needless_range_loop)]

//! Property-based tests for type extraction, filtering, and wrapping in adze-common.
//!
//! Covers `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! across a wide variety of leaf types, wrappers, and skip-set configurations.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use proptest::prelude::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn type_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn parse_ty(s: &str) -> Type {
    syn::parse_str(s).unwrap()
}

// ---------------------------------------------------------------------------
// Catalogs for proptest strategies
// ---------------------------------------------------------------------------

const LEAF_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "u8", "u16", "u32", "u64", "u128", "usize", "isize", "f32",
    "f64", "bool", "char", "String", "Foo", "Bar", "Token",
];

const WRAPPER_NAMES: &[&str] = &["Vec", "Option", "Box", "Arc", "Rc", "RefCell"];

// ===========================================================================
// 1. try_extract_inner_type — Vec<T> always extracts inner type
// ===========================================================================

proptest! {
    #[test]
    fn extract_vec_always_succeeds(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
        prop_assert!(extracted, "Vec<{leaf}> should extract");
        prop_assert_eq!(type_str(&inner), leaf);
    }

    #[test]
    fn extract_option_always_succeeds(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Option<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        prop_assert!(extracted, "Option<{leaf}> should extract");
        prop_assert_eq!(type_str(&inner), leaf);
    }

    #[test]
    fn extract_box_always_succeeds(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
        prop_assert!(extracted, "Box<{leaf}> should extract");
        prop_assert_eq!(type_str(&inner), leaf);
    }
}

// ===========================================================================
// 2. try_extract_inner_type — plain type with inner_of="Vec" returns false
// ===========================================================================

proptest! {
    #[test]
    fn extract_plain_type_returns_false(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(leaf);
        let (returned, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
        prop_assert!(!extracted, "plain {leaf} should not extract for Vec");
        prop_assert_eq!(type_str(&returned), leaf);
    }

    #[test]
    fn extract_mismatched_wrapper_returns_false(
        lidx in 0..LEAF_TYPES.len(),
    ) {
        // Option<T> with inner_of="Vec" should not extract
        let leaf = LEAF_TYPES[lidx];
        let ty = parse_ty(&format!("Option<{leaf}>"));
        let (returned, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
        prop_assert!(!extracted, "Option<{leaf}> should not extract for Vec");
        prop_assert_eq!(type_str(&returned), format!("Option < {leaf} >"));
    }

    #[test]
    fn extract_vec_mismatched_as_option_returns_false(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let (returned, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        prop_assert!(!extracted, "Vec<{leaf}> should not extract for Option");
        prop_assert_eq!(type_str(&returned), format!("Vec < {leaf} >"));
    }
}

// ===========================================================================
// 3. filter_inner_type — empty skip returns same type
// ===========================================================================

proptest! {
    #[test]
    fn filter_empty_skip_preserves_leaf(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(leaf);
        let result = filter_inner_type(&ty, &skip(&[]));
        prop_assert_eq!(type_str(&result), leaf);
    }

    #[test]
    fn filter_empty_skip_preserves_wrapped(
        lidx in 0..LEAF_TYPES.len(),
        widx in 0..WRAPPER_NAMES.len(),
    ) {
        let leaf = LEAF_TYPES[lidx];
        let wrapper = WRAPPER_NAMES[widx];
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let result = filter_inner_type(&ty, &skip(&[]));
        prop_assert_eq!(type_str(&result), format!("{wrapper} < {leaf} >"));
    }
}

// ===========================================================================
// 4. filter_inner_type — Box in skip removes Box wrapper
// ===========================================================================

proptest! {
    #[test]
    fn filter_removes_box(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let result = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(type_str(&result), leaf);
    }

    #[test]
    fn filter_removes_arc(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Arc<{leaf}>"));
        let result = filter_inner_type(&ty, &skip(&["Arc"]));
        prop_assert_eq!(type_str(&result), leaf);
    }

    #[test]
    fn filter_removes_rc(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Rc<{leaf}>"));
        let result = filter_inner_type(&ty, &skip(&["Rc"]));
        prop_assert_eq!(type_str(&result), leaf);
    }

    #[test]
    fn filter_removes_refcell(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("RefCell<{leaf}>"));
        let result = filter_inner_type(&ty, &skip(&["RefCell"]));
        prop_assert_eq!(type_str(&result), leaf);
    }
}

// ===========================================================================
// 5. wrap_leaf_type — empty skip wraps everything
// ===========================================================================

proptest! {
    #[test]
    fn wrap_empty_skip_wraps_leaf(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(leaf);
        let result = type_str(&wrap_leaf_type(&ty, &skip(&[])));
        prop_assert!(
            result.contains("adze :: WithLeaf"),
            "plain {leaf} should be wrapped, got: {result}"
        );
    }

    #[test]
    fn wrap_empty_skip_wraps_wrapper(
        lidx in 0..LEAF_TYPES.len(),
        widx in 0..WRAPPER_NAMES.len(),
    ) {
        let leaf = LEAF_TYPES[lidx];
        let wrapper = WRAPPER_NAMES[widx];
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let result = type_str(&wrap_leaf_type(&ty, &skip(&[])));
        prop_assert!(
            result.starts_with("adze :: WithLeaf <"),
            "wrapper {wrapper}<{leaf}> with empty skip should wrap entire type, got: {result}"
        );
    }
}

// ===========================================================================
// 6. Roundtrip: extract then wrap produces consistent results
// ===========================================================================

proptest! {
    #[test]
    fn roundtrip_extract_then_wrap_vec(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
        prop_assert!(extracted);
        // Wrapping the extracted inner type should produce a WithLeaf wrapper
        let wrapped = type_str(&wrap_leaf_type(&inner, &skip(&[])));
        prop_assert!(
            wrapped.contains("adze :: WithLeaf"),
            "extracted {leaf} should wrap to WithLeaf, got: {wrapped}"
        );
    }

    #[test]
    fn roundtrip_extract_then_wrap_option(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Option<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        prop_assert!(extracted);
        let wrapped = type_str(&wrap_leaf_type(&inner, &skip(&[])));
        prop_assert!(
            wrapped.contains("adze :: WithLeaf"),
            "extracted {leaf} from Option should wrap, got: {wrapped}"
        );
    }

    #[test]
    fn roundtrip_filter_then_wrap_box(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let filtered = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(type_str(&filtered), leaf);
        let wrapped = type_str(&wrap_leaf_type(&filtered, &skip(&[])));
        prop_assert!(
            wrapped.contains("adze :: WithLeaf"),
            "filtered {leaf} should wrap, got: {wrapped}"
        );
    }
}

// ===========================================================================
// 7. Various inner types: i32, u32, String, bool with all wrappers
// ===========================================================================

proptest! {
    #[test]
    fn extract_all_wrappers_with_i32(widx in 0..WRAPPER_NAMES.len()) {
        let wrapper = WRAPPER_NAMES[widx];
        let ty = parse_ty(&format!("{wrapper}<i32>"));
        let (inner, extracted) = try_extract_inner_type(&ty, wrapper, &skip(&[]));
        prop_assert!(extracted, "{wrapper}<i32> should extract");
        prop_assert_eq!(type_str(&inner), "i32");
    }

    #[test]
    fn extract_all_wrappers_with_u32(widx in 0..WRAPPER_NAMES.len()) {
        let wrapper = WRAPPER_NAMES[widx];
        let ty = parse_ty(&format!("{wrapper}<u32>"));
        let (inner, extracted) = try_extract_inner_type(&ty, wrapper, &skip(&[]));
        prop_assert!(extracted, "{wrapper}<u32> should extract");
        prop_assert_eq!(type_str(&inner), "u32");
    }

    #[test]
    fn extract_all_wrappers_with_string(widx in 0..WRAPPER_NAMES.len()) {
        let wrapper = WRAPPER_NAMES[widx];
        let ty = parse_ty(&format!("{wrapper}<String>"));
        let (inner, extracted) = try_extract_inner_type(&ty, wrapper, &skip(&[]));
        prop_assert!(extracted, "{wrapper}<String> should extract");
        prop_assert_eq!(type_str(&inner), "String");
    }

    #[test]
    fn extract_all_wrappers_with_bool(widx in 0..WRAPPER_NAMES.len()) {
        let wrapper = WRAPPER_NAMES[widx];
        let ty = parse_ty(&format!("{wrapper}<bool>"));
        let (inner, extracted) = try_extract_inner_type(&ty, wrapper, &skip(&[]));
        prop_assert!(extracted, "{wrapper}<bool> should extract");
        prop_assert_eq!(type_str(&inner), "bool");
    }
}

// ===========================================================================
// 8. Various wrappers: Vec, Option, Box — filter and wrap interactions
// ===========================================================================

proptest! {
    #[test]
    fn filter_each_wrapper_removes_it(
        widx in 0..WRAPPER_NAMES.len(),
        lidx in 0..LEAF_TYPES.len(),
    ) {
        let wrapper = WRAPPER_NAMES[widx];
        let leaf = LEAF_TYPES[lidx];
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let result = filter_inner_type(&ty, &skip(&[wrapper]));
        prop_assert_eq!(type_str(&result), leaf);
    }

    #[test]
    fn wrap_with_skip_preserves_outer(
        widx in 0..WRAPPER_NAMES.len(),
        lidx in 0..LEAF_TYPES.len(),
    ) {
        let wrapper = WRAPPER_NAMES[widx];
        let leaf = LEAF_TYPES[lidx];
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let result = type_str(&wrap_leaf_type(&ty, &skip(&[wrapper])));
        prop_assert!(
            result.starts_with(&format!("{wrapper} <")),
            "wrap with {wrapper} in skip should preserve outer, got: {result}"
        );
        prop_assert!(
            result.contains("adze :: WithLeaf"),
            "inner {leaf} should be wrapped, got: {result}"
        );
    }
}

// ===========================================================================
// 9. Skip sets of different sizes (0, 1, 2, 3)
// ===========================================================================

proptest! {
    #[test]
    fn skip_size_0_extract_returns_identity_for_plain(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(leaf);
        let (returned, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
        prop_assert!(!extracted);
        prop_assert_eq!(type_str(&returned), leaf);
    }

    #[test]
    fn skip_size_1_extract_through_box(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Box<Vec<{leaf}>>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
        prop_assert!(extracted, "should extract Vec through Box skip");
        prop_assert_eq!(type_str(&inner), leaf);
    }

    #[test]
    fn skip_size_2_extract_through_box_arc(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Box<Arc<Vec<{leaf}>>>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
        prop_assert!(extracted, "should extract Vec through Box+Arc skip");
        prop_assert_eq!(type_str(&inner), leaf);
    }

    #[test]
    fn skip_size_3_extract_through_box_arc_rc(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Box<Arc<Rc<Option<{leaf}>>>>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc", "Rc"]));
        prop_assert!(extracted, "should extract Option through Box+Arc+Rc skip");
        prop_assert_eq!(type_str(&inner), leaf);
    }

    #[test]
    fn filter_skip_size_2_removes_nested(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Box<Arc<{leaf}>>"));
        let result = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
        prop_assert_eq!(type_str(&result), leaf);
    }

    #[test]
    fn filter_skip_size_3_removes_all_layers(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Box<Arc<Rc<{leaf}>>>"));
        let result = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
        prop_assert_eq!(type_str(&result), leaf);
    }
}

// ===========================================================================
// 10. Nested types maintain structure
// ===========================================================================

proptest! {
    #[test]
    fn extract_vec_of_option_preserves_inner_structure(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Vec<Option<{leaf}>>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
        prop_assert!(extracted, "Vec<Option<{leaf}>> should extract");
        prop_assert_eq!(type_str(&inner), format!("Option < {leaf} >"));
    }

    #[test]
    fn extract_option_of_vec_preserves_inner_structure(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Option<Vec<{leaf}>>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        prop_assert!(extracted, "Option<Vec<{leaf}>> should extract");
        prop_assert_eq!(type_str(&inner), format!("Vec < {leaf} >"));
    }

    #[test]
    fn filter_nested_only_removes_skip_layers(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Box<Vec<{leaf}>>"));
        // Only Box in skip — Vec layer should remain
        let result = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(type_str(&result), format!("Vec < {leaf} >"));
    }

    #[test]
    fn wrap_nested_skip_wraps_innermost(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Vec<Option<{leaf}>>"));
        let result = type_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Option"])));
        prop_assert!(
            result.starts_with("Vec <"),
            "outer Vec should be preserved, got: {result}"
        );
        prop_assert!(
            result.contains("Option <"),
            "Option layer should be preserved, got: {result}"
        );
        prop_assert!(
            result.contains("adze :: WithLeaf"),
            "innermost {leaf} should be wrapped, got: {result}"
        );
    }

    #[test]
    fn wrap_triple_nested_wraps_leaf_only(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Vec<Box<Option<{leaf}>>>"));
        let result = type_str(&wrap_leaf_type(&ty, &skip(&["Vec", "Box", "Option"])));
        prop_assert!(result.starts_with("Vec <"), "Vec preserved, got: {result}");
        prop_assert!(result.contains("Box <"), "Box preserved, got: {result}");
        prop_assert!(result.contains("Option <"), "Option preserved, got: {result}");
        prop_assert!(result.contains("adze :: WithLeaf"), "leaf wrapped, got: {result}");
    }
}

// ===========================================================================
// 11. Determinism — same input always same output
// ===========================================================================

proptest! {
    #[test]
    fn extract_is_deterministic(
        lidx in 0..LEAF_TYPES.len(),
        widx in 0..WRAPPER_NAMES.len(),
    ) {
        let leaf = LEAF_TYPES[lidx];
        let wrapper = WRAPPER_NAMES[widx];
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let (inner1, ext1) = try_extract_inner_type(&ty, wrapper, &skip(&[]));
        let (inner2, ext2) = try_extract_inner_type(&ty, wrapper, &skip(&[]));
        prop_assert_eq!(ext1, ext2);
        prop_assert_eq!(type_str(&inner1), type_str(&inner2));
    }

    #[test]
    fn filter_is_deterministic(
        lidx in 0..LEAF_TYPES.len(),
        widx in 0..WRAPPER_NAMES.len(),
    ) {
        let leaf = LEAF_TYPES[lidx];
        let wrapper = WRAPPER_NAMES[widx];
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let r1 = type_str(&filter_inner_type(&ty, &skip(&[wrapper])));
        let r2 = type_str(&filter_inner_type(&ty, &skip(&[wrapper])));
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn wrap_is_deterministic(
        lidx in 0..LEAF_TYPES.len(),
        widx in 0..WRAPPER_NAMES.len(),
    ) {
        let leaf = LEAF_TYPES[lidx];
        let wrapper = WRAPPER_NAMES[widx];
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        let r1 = type_str(&wrap_leaf_type(&ty, &skip(&[wrapper])));
        let r2 = type_str(&wrap_leaf_type(&ty, &skip(&[wrapper])));
        prop_assert_eq!(r1, r2);
    }
}

// ===========================================================================
// 12. parse_quote! based tests for exact type construction
// ===========================================================================

proptest! {
    #[test]
    fn parse_quote_vec_extract(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let leaf_ty: Type = syn::parse_str(leaf).unwrap();
        let ty: Type = parse_quote!(Vec<#leaf_ty>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
        prop_assert!(extracted);
        prop_assert_eq!(type_str(&inner), leaf);
    }

    #[test]
    fn parse_quote_option_extract(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let leaf_ty: Type = syn::parse_str(leaf).unwrap();
        let ty: Type = parse_quote!(Option<#leaf_ty>);
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        prop_assert!(extracted);
        prop_assert_eq!(type_str(&inner), leaf);
    }
}

// ===========================================================================
// 13. Non-matching skip entries have no effect
// ===========================================================================

proptest! {
    #[test]
    fn irrelevant_skip_entries_do_not_affect_extract(
        lidx in 0..LEAF_TYPES.len(),
        widx in 0..WRAPPER_NAMES.len(),
    ) {
        let leaf = LEAF_TYPES[lidx];
        let wrapper = WRAPPER_NAMES[widx];
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        // Skip set contains unrelated names that are not in the type
        let (inner1, ext1) = try_extract_inner_type(&ty, wrapper, &skip(&[]));
        let (inner2, ext2) = try_extract_inner_type(&ty, wrapper, &skip(&["Mutex", "RwLock"]));
        prop_assert_eq!(ext1, ext2);
        prop_assert_eq!(type_str(&inner1), type_str(&inner2));
    }

    #[test]
    fn irrelevant_skip_entries_do_not_affect_filter(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let r1 = type_str(&filter_inner_type(&ty, &skip(&["Box"])));
        let r2 = type_str(&filter_inner_type(&ty, &skip(&["Box", "Mutex", "RwLock"])));
        prop_assert_eq!(r1, r2);
    }
}

// ===========================================================================
// 14. wrap_leaf_type with parse_quote! types
// ===========================================================================

proptest! {
    #[test]
    fn wrap_parse_quote_leaf(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty: Type = syn::parse_str(leaf).unwrap();
        let result = type_str(&wrap_leaf_type(&ty, &skip(&[])));
        prop_assert!(
            result.starts_with("adze :: WithLeaf <"),
            "leaf {leaf} should be fully wrapped, got: {result}"
        );
        prop_assert!(
            result.contains(leaf),
            "wrapped result should contain original type name {leaf}, got: {result}"
        );
    }
}

// ===========================================================================
// 15. Extracted type from filter equals original leaf
// ===========================================================================

proptest! {
    #[test]
    fn filter_double_wrap_removes_both(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Arc<Box<{leaf}>>"));
        let result = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
        prop_assert_eq!(type_str(&result), leaf);
    }

    #[test]
    fn extract_through_filter_consistency(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        // Box<Vec<T>> — filter Box, then extract Vec
        let ty = parse_ty(&format!("Box<Vec<{leaf}>>"));
        let filtered = filter_inner_type(&ty, &skip(&["Box"]));
        prop_assert_eq!(type_str(&filtered), format!("Vec < {leaf} >"));
        let (inner, extracted) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
        prop_assert!(extracted);
        prop_assert_eq!(type_str(&inner), leaf);
    }

    #[test]
    fn filter_preserves_non_skip_wrapper(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        // Vec is NOT in skip, so it should be preserved
        let result = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
        prop_assert_eq!(type_str(&result), format!("Vec < {leaf} >"));
    }

    #[test]
    fn wrap_then_filter_noop_on_leaf(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty: Type = syn::parse_str(leaf).unwrap();
        let wrapped = wrap_leaf_type(&ty, &skip(&[]));
        // Filter with empty skip should not change wrapped type
        let filtered = filter_inner_type(&wrapped, &skip(&[]));
        prop_assert_eq!(type_str(&filtered), type_str(&wrapped));
    }

    #[test]
    fn extract_option_vec_nested_with_skip(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Option<Vec<{leaf}>>"));
        // Extract Vec through Option skip
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
        prop_assert!(extracted);
        prop_assert_eq!(type_str(&inner), leaf);
    }

    #[test]
    fn filter_single_layer_identity_when_not_in_skip(
        widx in 0..WRAPPER_NAMES.len(),
        lidx in 0..LEAF_TYPES.len(),
    ) {
        let wrapper = WRAPPER_NAMES[widx];
        let leaf = LEAF_TYPES[lidx];
        let ty = parse_ty(&format!("{wrapper}<{leaf}>"));
        // Use a skip set that does NOT contain this wrapper
        let other_skip: HashSet<&str> = WRAPPER_NAMES.iter()
            .copied()
            .filter(|&w| w != wrapper)
            .collect();
        let result = type_str(&filter_inner_type(&ty, &other_skip));
        prop_assert_eq!(result, format!("{wrapper} < {leaf} >"));
    }
}

// ===========================================================================
// 16. Additional coverage — extract with self-referencing wrapper in skip
// ===========================================================================

proptest! {
    #[test]
    fn extract_ignores_inner_of_in_skip(idx in 0..LEAF_TYPES.len()) {
        // When inner_of wrapper is also in skip, the skip branch fires first
        // but the inner_of match on the first segment takes priority
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Vec<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Vec"]));
        // inner_of match happens before skip check
        prop_assert!(extracted);
        prop_assert_eq!(type_str(&inner), leaf);
    }

    #[test]
    fn wrap_box_in_skip_wraps_inner(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("Box<{leaf}>"));
        let result = type_str(&wrap_leaf_type(&ty, &skip(&["Box"])));
        prop_assert!(result.starts_with("Box <"), "Box preserved, got: {result}");
        prop_assert!(result.contains("adze :: WithLeaf"), "inner wrapped, got: {result}");
    }

    #[test]
    fn extract_refcell_always_succeeds(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_ty(&format!("RefCell<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "RefCell", &skip(&[]));
        prop_assert!(extracted, "RefCell<{leaf}> should extract");
        prop_assert_eq!(type_str(&inner), leaf);
    }
}
