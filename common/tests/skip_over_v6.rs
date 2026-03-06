use std::collections::HashSet;

use quote::quote;
use syn::{Type, parse_quote};

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};

/// Convert a syn Type to a normalized string for comparison.
fn ty_str(ty: &Type) -> String {
    quote!(#ty).to_string()
}

// =============================================================================
// Section 1: Empty skip_over — filter returns unchanged (tests 1–10)
// =============================================================================

#[test]
fn filter_empty_skip_returns_simple_type_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_empty_skip_returns_option_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_empty_skip_returns_vec_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_empty_skip_returns_box_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<u64>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_empty_skip_returns_nested_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_empty_skip_returns_arc_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Arc<Mutex<bool>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_empty_skip_returns_result_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Result<i32, String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_empty_skip_returns_string_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_empty_skip_returns_triple_nested_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<Vec<Option<u8>>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_empty_skip_returns_custom_type_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(MyCustomType<Foo>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

// =============================================================================
// Section 2: Empty skip_over — extract ignores wrappers (tests 11–18)
// =============================================================================

#[test]
fn extract_empty_skip_does_not_find_through_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&ty));
}

#[test]
fn extract_empty_skip_finds_direct_match() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<i32>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn extract_empty_skip_no_match() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<i32>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&ty));
}

#[test]
fn extract_empty_skip_simple_type_no_match() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(i32);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&ty));
}

#[test]
fn extract_empty_skip_cannot_reach_nested_option() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&ty));
}

#[test]
fn extract_empty_skip_direct_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<u8>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(u8);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn extract_empty_skip_direct_box() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<f64>);
    let (result, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(f64);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn extract_empty_skip_does_not_unwrap_double_nesting() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Arc<Vec<i32>>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&ty));
}

// =============================================================================
// Section 3: Empty skip_over — wrap wraps whole type (tests 19–24)
// =============================================================================

#[test]
fn wrap_empty_skip_wraps_simple() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(adze::WithLeaf<i32>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn wrap_empty_skip_wraps_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(adze::WithLeaf<Vec<i32>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn wrap_empty_skip_wraps_option() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<String>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(adze::WithLeaf<Option<String>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn wrap_empty_skip_wraps_nested() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(adze::WithLeaf<Vec<Option<i32>>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn wrap_empty_skip_wraps_string() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(adze::WithLeaf<String>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn wrap_empty_skip_wraps_box() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<u32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(adze::WithLeaf<Box<u32>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

// =============================================================================
// Section 4: Single entry skip — extract through wrapper (tests 25–32)
// =============================================================================

#[test]
fn extract_skip_vec_finds_option_inside_vec() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn extract_skip_box_finds_vec_inside_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(String);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn extract_skip_option_finds_vec_inside_option() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(bool);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn extract_skip_vec_with_direct_vec_still_extracts() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    // Vec is in skip_over, not inner_of — so it skips Vec looking for something else inside
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    // i32 is not Option, so extraction fails
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&ty));
}

#[test]
fn extract_skip_arc_finds_option_inside_arc() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<Option<u64>>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(u64);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn extract_direct_match_even_when_in_skip() {
    // If the target is also in skip_over, direct match takes precedence
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn extract_skip_mutex_finds_box_inside_mutex() {
    let skip: HashSet<&str> = ["Mutex"].into_iter().collect();
    let ty: Type = parse_quote!(Mutex<Box<i32>>);
    let (result, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn extract_skip_rwlock_finds_vec_inside() {
    let skip: HashSet<&str> = ["RwLock"].into_iter().collect();
    let ty: Type = parse_quote!(RwLock<Vec<String>>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(String);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

// =============================================================================
// Section 5: Single entry skip — filter strips wrapper (tests 33–40)
// =============================================================================

#[test]
fn filter_skip_option_strips_option() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<i32>);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn filter_skip_vec_strips_vec() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    let expected: Type = parse_quote!(String);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn filter_skip_box_strips_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<u8>);
    let expected: Type = parse_quote!(u8);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn filter_skip_arc_strips_arc() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<bool>);
    let expected: Type = parse_quote!(bool);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn filter_skip_option_leaves_vec_alone() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_skip_vec_leaves_option_alone() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Option<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_skip_option_recursive_double_option() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn filter_skip_vec_recursive_double_vec() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Vec<String>>);
    let expected: Type = parse_quote!(String);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

// =============================================================================
// Section 6: Multiple skip entries (tests 41–50)
// =============================================================================

#[test]
fn filter_multi_skip_vec_option_strips_both() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn filter_multi_skip_option_vec_strips_both() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let expected: Type = parse_quote!(String);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn filter_multi_skip_box_arc_strips_both() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<u32>>);
    let expected: Type = parse_quote!(u32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn filter_multi_skip_three_layers() {
    let skip: HashSet<&str> = ["Box", "Arc", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Option<f64>>>);
    let expected: Type = parse_quote!(f64);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn extract_multi_skip_vec_option_finds_box() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let (result, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn extract_multi_skip_box_arc_finds_vec() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Vec<u8>>>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(u8);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn filter_multi_skip_stops_at_non_skip_type() {
    let skip: HashSet<&str> = ["Box", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Option<Vec<i32>>>);
    let expected: Type = parse_quote!(Vec<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn wrap_multi_skip_vec_option_wraps_leaf() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(Vec<Option<adze::WithLeaf<i32>>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn wrap_multi_skip_option_vec_wraps_leaf() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(Option<Vec<adze::WithLeaf<String>>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn extract_multi_skip_not_found_returns_original() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&ty));
}

// =============================================================================
// Section 7: Skip entry not matching type — ignored (tests 51–56)
// =============================================================================

#[test]
fn filter_skip_vec_on_option_type_no_effect() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Option<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_skip_box_on_string_no_effect() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(String);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn extract_skip_arc_on_vec_type_no_reach() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&ty));
}

#[test]
fn wrap_skip_box_on_option_type_wraps_whole() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(adze::WithLeaf<Option<i32>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn filter_skip_nonexistent_type_no_effect() {
    let skip: HashSet<&str> = ["FooBar"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn extract_skip_nonexistent_type_no_effect() {
    let skip: HashSet<&str> = ["NoSuchType"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&ty));
}

// =============================================================================
// Section 8: Case sensitivity (tests 57–62)
// =============================================================================

#[test]
fn filter_skip_is_case_sensitive_lowercase_vec() {
    let skip: HashSet<&str> = ["vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    // "vec" != "Vec" so no stripping occurs
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_skip_is_case_sensitive_uppercase_option() {
    let skip: HashSet<&str> = ["OPTION"].into_iter().collect();
    let ty: Type = parse_quote!(Option<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn extract_skip_is_case_sensitive_lowercase_box() {
    let skip: HashSet<&str> = ["box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&ty));
}

#[test]
fn wrap_skip_is_case_sensitive_lowercase_option() {
    let skip: HashSet<&str> = ["option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(adze::WithLeaf<Option<i32>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn filter_skip_mixed_case_no_match() {
    let skip: HashSet<&str> = ["VEC", "option", "BOX"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_correct_case_does_match() {
    let skip: HashSet<&str> = ["Vec", "Option", "Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

// =============================================================================
// Section 9: Large skip sets (tests 63–67)
// =============================================================================

#[test]
fn filter_large_skip_set_strips_matching_outer() {
    let skip: HashSet<&str> = ["Vec", "Option", "Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<i32>);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn filter_large_skip_set_strips_chain() {
    let skip: HashSet<&str> = ["Vec", "Option", "Box", "Arc", "Rc", "Mutex"]
        .into_iter()
        .collect();
    let ty: Type = parse_quote!(Arc<Mutex<String>>);
    let expected: Type = parse_quote!(String);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn extract_large_skip_set_reaches_deep_target() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc", "Mutex", "RwLock"]
        .into_iter()
        .collect();
    let ty: Type = parse_quote!(Box<Arc<Rc<Vec<i32>>>>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn wrap_large_skip_set_wraps_leaf() {
    let skip: HashSet<&str> = ["Vec", "Option", "Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(Vec<adze::WithLeaf<i32>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn filter_large_skip_set_non_matching_outer_stops() {
    let skip: HashSet<&str> = ["Vec", "Option", "Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

// =============================================================================
// Section 10: Nested same-type: Vec<Vec<i32>> with skip={"Vec"} (tests 68–72)
// =============================================================================

#[test]
fn filter_skip_vec_double_vec_strips_both() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn filter_skip_option_double_option_strips_both() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Option<String>>);
    let expected: Type = parse_quote!(String);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn extract_skip_vec_double_vec_finds_option_inside() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Vec<Option<i32>>>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn wrap_skip_vec_double_vec_wraps_inner_leaf() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(Vec<Vec<adze::WithLeaf<i32>>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn filter_skip_box_triple_box_strips_all() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Box<Box<u16>>>);
    let expected: Type = parse_quote!(u16);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

// =============================================================================
// Section 11: Deeply nested with various skip sets (tests 73–78)
// =============================================================================

#[test]
fn filter_deep_nesting_partial_skip() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let expected: Type = parse_quote!(Box<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn filter_deep_nesting_full_skip() {
    let skip: HashSet<&str> = ["Vec", "Option", "Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let expected: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&expected));
}

#[test]
fn extract_deep_nesting_skip_two_find_third() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<Box<String>>>);
    let (result, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    let expected: Type = parse_quote!(String);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn wrap_deep_nesting_skip_vec_option() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(Vec<Option<adze::WithLeaf<Box<i32>>>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn wrap_deep_nesting_skip_all_three() {
    let skip: HashSet<&str> = ["Vec", "Option", "Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(Vec<Option<Box<adze::WithLeaf<i32>>>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn extract_deep_nesting_four_layers() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<u32>>>>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected: Type = parse_quote!(u32);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

// =============================================================================
// Section 12: wrap with skip wraps INSIDE vs OUTSIDE (tests 79–83)
// =============================================================================

#[test]
fn wrap_skip_vec_wraps_inside_vec() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(Vec<adze::WithLeaf<i32>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn wrap_skip_option_wraps_inside_option() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<f32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(Option<adze::WithLeaf<f32>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn wrap_no_skip_wraps_outside_everything() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(adze::WithLeaf<Vec<i32>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn wrap_skip_vec_not_option_wraps_option_as_leaf() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(Vec<adze::WithLeaf<Option<i32>>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

#[test]
fn wrap_skip_option_not_vec_wraps_vec_as_leaf() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let expected: Type = parse_quote!(Option<adze::WithLeaf<Vec<i32>>>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected));
}

// =============================================================================
// Section 13: Consistency / round-trip (tests 84–88)
// =============================================================================

#[test]
fn filter_then_wrap_with_same_skip() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &skip);
    let expected_filtered: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filtered), ty_str(&expected_filtered));

    let wrapped = wrap_leaf_type(&filtered, &skip);
    // i32 is not in skip, so gets wrapped
    let expected_wrapped: Type = parse_quote!(adze::WithLeaf<i32>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected_wrapped));
}

#[test]
fn extract_then_wrap_round_trip() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<u32>>);
    let (extracted, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let expected_extracted: Type = parse_quote!(u32);
    assert_eq!(ty_str(&extracted), ty_str(&expected_extracted));

    let wrapped = wrap_leaf_type(&extracted, &skip);
    let expected_wrapped: Type = parse_quote!(adze::WithLeaf<u32>);
    assert_eq!(ty_str(&wrapped), ty_str(&expected_wrapped));
}

#[test]
fn filter_idempotent_after_full_strip() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<i32>);
    let first = filter_inner_type(&ty, &skip);
    let second = filter_inner_type(&first, &skip);
    assert_eq!(ty_str(&first), ty_str(&second));
}

#[test]
fn filter_noop_is_idempotent() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    let first = filter_inner_type(&ty, &skip);
    let second = filter_inner_type(&first, &skip);
    assert_eq!(ty_str(&first), ty_str(&second));
    assert_eq!(ty_str(&ty), ty_str(&first));
}

#[test]
fn extract_not_found_returns_original_unchanged() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    let (result, found) = try_extract_inner_type(&ty, "HashMap", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), ty_str(&ty));
}
