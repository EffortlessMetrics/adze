#![allow(clippy::needless_range_loop)]

//! Property-based and deterministic tests for `try_extract_inner_type` in adze-common.

use std::collections::HashSet;

use adze_common::try_extract_inner_type;
use proptest::prelude::*;
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ---------------------------------------------------------------------------
// Catalog of concrete types used by proptest strategies
// ---------------------------------------------------------------------------

const LEAF_TYPES: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64", "bool", "char", "String",
];

const WRAPPER_NAMES: &[&str] = &["Option", "Vec", "Box", "Arc", "Rc", "Cell", "RefCell"];

fn parse_type(s: &str) -> Type {
    syn::parse_str(s).unwrap()
}

// ===========================================================================
// 1. Extract from Option<T> → T (various leaf types)
// ===========================================================================

proptest! {
    #[test]
    fn option_leaf_always_extracts(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_type(&format!("Option<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
        prop_assert!(extracted, "Option<{leaf}> should extract");
        prop_assert_eq!(ty_str(&inner), leaf);
    }
}

// ===========================================================================
// 2. Extract from Vec<T> → T (various leaf types)
// ===========================================================================

proptest! {
    #[test]
    fn vec_leaf_always_extracts(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_type(&format!("Vec<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
        prop_assert!(extracted, "Vec<{leaf}> should extract");
        prop_assert_eq!(ty_str(&inner), leaf);
    }
}

// ===========================================================================
// 3. Extract from Box<T> → T (various leaf types)
// ===========================================================================

proptest! {
    #[test]
    fn box_leaf_always_extracts(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_type(&format!("Box<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
        prop_assert!(extracted, "Box<{leaf}> should extract");
        prop_assert_eq!(ty_str(&inner), leaf);
    }
}

// ===========================================================================
// 4. Extract from nested Option<Vec<T>> → T (skip Option to target Vec)
// ===========================================================================

proptest! {
    #[test]
    fn option_vec_leaf_skip_option_extracts(idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[idx];
        let ty = parse_type(&format!("Option<Vec<{leaf}>>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
        prop_assert!(extracted, "Option<Vec<{leaf}>> with skip Option should extract");
        prop_assert_eq!(ty_str(&inner), leaf);
    }
}

// ===========================================================================
// 5. No extraction on plain types
// ===========================================================================

proptest! {
    #[test]
    fn plain_leaf_never_extracts(
        leaf_idx in 0..LEAF_TYPES.len(),
        target_idx in 0..WRAPPER_NAMES.len(),
    ) {
        let leaf = LEAF_TYPES[leaf_idx];
        let target = WRAPPER_NAMES[target_idx];
        let ty = parse_type(leaf);
        let (inner, extracted) = try_extract_inner_type(&ty, target, &skip(&[]));
        prop_assert!(!extracted, "plain {leaf} should not extract for target {target}");
        prop_assert_eq!(ty_str(&inner), leaf);
    }
}

// ===========================================================================
// 6. Extract with target type matching — only matching wrapper extracts
// ===========================================================================

proptest! {
    #[test]
    fn only_matching_target_extracts(
        wrapper_idx in 0..WRAPPER_NAMES.len(),
        target_idx in 0..WRAPPER_NAMES.len(),
    ) {
        let wrapper = WRAPPER_NAMES[wrapper_idx];
        let target = WRAPPER_NAMES[target_idx];
        let ty = parse_type(&format!("{wrapper}<i32>"));
        let (inner, extracted) = try_extract_inner_type(&ty, target, &skip(&[]));
        if wrapper == target {
            prop_assert!(extracted, "{wrapper}<i32> should extract when target is {target}");
            prop_assert_eq!(ty_str(&inner), "i32");
        } else {
            prop_assert!(!extracted, "{wrapper}<i32> should NOT extract when target is {target}");
        }
    }
}

// ===========================================================================
// 7. Extract with skip set — skip set wrappers are traversed
// ===========================================================================

#[test]
fn skip_arc_rc_to_find_vec() {
    let ty: Type = parse_quote!(Arc<Rc<Vec<f64>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc", "Rc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn skip_cell_to_find_option() {
    let ty: Type = parse_quote!(Cell<Option<usize>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Cell"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn skip_refcell_box_to_find_vec() {
    let ty: Type = parse_quote!(RefCell<Box<Vec<u16>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["RefCell", "Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn skip_set_with_target_still_extracts_directly() {
    // When the target itself is also in the skip set, the target match happens
    // first (line 92 of the source), so it extracts directly.
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Option"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn large_skip_set_traversal() {
    let ty: Type = parse_type("Arc<Box<Rc<Cell<RefCell<Option<i128>>>>>>");
    let (inner, extracted) = try_extract_inner_type(
        &ty,
        "Option",
        &skip(&["Arc", "Box", "Rc", "Cell", "RefCell"]),
    );
    assert!(extracted);
    assert_eq!(ty_str(&inner), "i128");
}

#[test]
fn skip_set_miss_at_intermediate_layer() {
    // Arc is skippable, but inner Mutex is NOT in skip set and is NOT the target.
    // So extraction stops and returns original.
    let ty: Type = parse_quote!(Arc<Mutex<Option<u8>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Arc < Mutex < Option < u8 > > >");
}

// ===========================================================================
// 8. Extract determinism — repeated calls yield identical results
// ===========================================================================

proptest! {
    #[test]
    fn extraction_is_deterministic(
        wrapper_idx in 0..WRAPPER_NAMES.len(),
        leaf_idx in 0..LEAF_TYPES.len(),
    ) {
        let wrapper = WRAPPER_NAMES[wrapper_idx];
        let leaf = LEAF_TYPES[leaf_idx];
        let ty = parse_type(&format!("{wrapper}<{leaf}>"));
        let s = skip(&[]);

        let (inner1, ext1) = try_extract_inner_type(&ty, wrapper, &s);
        let (inner2, ext2) = try_extract_inner_type(&ty, wrapper, &s);
        let (inner3, ext3) = try_extract_inner_type(&ty, wrapper, &s);

        prop_assert_eq!(ext1, ext2);
        prop_assert_eq!(ext2, ext3);
        prop_assert_eq!(ty_str(&inner1), ty_str(&inner2));
        prop_assert_eq!(ty_str(&inner2), ty_str(&inner3));
    }
}

proptest! {
    #[test]
    fn non_extraction_is_deterministic(leaf_idx in 0..LEAF_TYPES.len()) {
        let leaf = LEAF_TYPES[leaf_idx];
        let ty = parse_type(leaf);
        let s = skip(&[]);

        let (r1, e1) = try_extract_inner_type(&ty, "Option", &s);
        let (r2, e2) = try_extract_inner_type(&ty, "Option", &s);

        prop_assert!(!e1);
        prop_assert!(!e2);
        prop_assert_eq!(ty_str(&r1), ty_str(&r2));
    }
}

// ===========================================================================
// 9. Property: extracted=false ⟹ returned type equals original
// ===========================================================================

proptest! {
    #[test]
    fn false_extraction_preserves_original(
        leaf_idx in 0..LEAF_TYPES.len(),
        target_idx in 0..WRAPPER_NAMES.len(),
    ) {
        let leaf = LEAF_TYPES[leaf_idx];
        let target = WRAPPER_NAMES[target_idx];
        let ty = parse_type(leaf);
        let (inner, extracted) = try_extract_inner_type(&ty, target, &skip(&[]));
        prop_assert!(!extracted);
        prop_assert_eq!(ty_str(&inner), ty_str(&ty));
    }
}

// ===========================================================================
// 10. Property: extracted=true ⟹ inner type token stream differs from outer
// ===========================================================================

proptest! {
    #[test]
    fn true_extraction_changes_type(
        wrapper_idx in 0..WRAPPER_NAMES.len(),
        leaf_idx in 0..LEAF_TYPES.len(),
    ) {
        let wrapper = WRAPPER_NAMES[wrapper_idx];
        let leaf = LEAF_TYPES[leaf_idx];
        let ty = parse_type(&format!("{wrapper}<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, wrapper, &skip(&[]));
        prop_assert!(extracted);
        // The extracted inner type must differ from the original wrapper<leaf>
        prop_assert_ne!(ty_str(&inner), ty_str(&ty));
    }
}

// ===========================================================================
// 11. Mixed: skip set interaction with target present at different depths
// ===========================================================================

#[test]
fn target_at_depth_zero_ignores_skip_set() {
    let ty: Type = parse_quote!(Vec<u32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn skip_single_layer_to_nested_option_vec() {
    // Box<Option<Vec<i8>>> — skip Box, target Option → extracts Vec<i8>
    let ty: Type = parse_quote!(Box<Option<Vec<i8>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Vec < i8 >");
}

#[test]
fn option_box_vec_skip_option_box_target_vec() {
    let ty: Type = parse_quote!(Option<Box<Vec<f32>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Option", "Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "f32");
}

// ===========================================================================
// 12. Concrete types with complex inner types
// ===========================================================================

#[test]
fn extract_option_of_path_type() {
    let ty: Type = parse_quote!(Option<std::collections::HashMap<String, i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(
        ty_str(&inner),
        "std :: collections :: HashMap < String , i32 >"
    );
}

#[test]
fn extract_vec_of_tuple_type() {
    let ty: Type = parse_quote!(Vec<(u8, u16)>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "(u8 , u16)");
}

#[test]
fn extract_box_of_reference_type() {
    let ty: Type = parse_quote!(Box<&str>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn extract_option_of_array_type() {
    let ty: Type = parse_quote!(Option<[u8; 32]>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "[u8 ; 32]");
}

// ===========================================================================
// 13. Additional coverage: empty skip set, self-named types, re-extraction
// ===========================================================================

#[test]
fn empty_skip_set_with_non_target_wrapper_returns_original() {
    let ty: Type = parse_quote!(Rc<Vec<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    // Rc is NOT in skip set and is NOT the target, so no extraction happens.
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Rc < Vec < String > >");
}

#[test]
fn extract_then_re_extract_nested_option() {
    // First extraction peels outer Option, second peels inner Option.
    let ty: Type = parse_quote!(Option<Option<Vec<u8>>>);
    let s = skip(&[]);
    let (first, e1) = try_extract_inner_type(&ty, "Option", &s);
    assert!(e1);
    assert_eq!(ty_str(&first), "Option < Vec < u8 > >");
    let (second, e2) = try_extract_inner_type(&first, "Option", &s);
    assert!(e2);
    assert_eq!(ty_str(&second), "Vec < u8 >");
}

proptest! {
    #[test]
    fn skip_set_containing_target_still_extracts(leaf_idx in 0..LEAF_TYPES.len()) {
        // When the target name is also in the skip set, the target check
        // (line 92) fires first, so extraction always succeeds.
        let leaf = LEAF_TYPES[leaf_idx];
        let ty = parse_type(&format!("Vec<{leaf}>"));
        let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&["Vec"]));
        prop_assert!(extracted);
        prop_assert_eq!(ty_str(&inner), leaf);
    }
}

#[test]
fn extract_vec_of_option_without_skip() {
    // Target is Vec, inner is Option<bool>. Extracts Option<bool> directly.
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < bool >");
}

#[test]
fn mismatched_target_with_full_skip_set_returns_original() {
    // All wrappers in skip set but target "Foo" is never present.
    let ty: Type = parse_quote!(Box<Arc<Rc<String>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Foo", &skip(&["Box", "Arc", "Rc"]));
    assert!(!extracted);
    assert_eq!(ty_str(&inner), "Box < Arc < Rc < String > > >");
}
