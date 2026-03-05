use std::collections::HashSet;

use quote::quote;
use syn::{Type, parse_quote};

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};

/// Convert a syn Type to a normalized string for comparison.
fn ty_str(ty: &Type) -> String {
    quote!(#ty).to_string()
}

// =============================================================================
// Section 1: try_extract_inner_type — wrapper detection basics (tests 1–20)
// =============================================================================

#[test]
fn extract_vec_detected_as_wrapper() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_option_detected_as_wrapper() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_box_detected_as_wrapper() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<u64>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn extract_arc_detected_as_wrapper() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Arc<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Arc", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_rc_detected_as_wrapper() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Rc<f64>);
    let (inner, found) = try_extract_inner_type(&ty, "Rc", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_cell_detected_as_wrapper() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Cell<u8>);
    let (inner, found) = try_extract_inner_type(&ty, "Cell", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_refcell_detected_as_wrapper() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(RefCell<i16>);
    let (inner, found) = try_extract_inner_type(&ty, "RefCell", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "i16");
}

#[test]
fn extract_custom_wrapper_detected() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Custom<usize>);
    let (inner, found) = try_extract_inner_type(&ty, "Custom", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn extract_plain_i32_not_detected_as_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(i32);
    let (_inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
}

#[test]
fn extract_plain_string_not_detected_as_option() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let (_inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
}

#[test]
fn extract_plain_bool_not_detected_as_box() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(bool);
    let (_inner, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(!found);
}

#[test]
fn extract_mismatched_wrapper_not_detected() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<i32>);
    let (returned, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(ty_str(&returned), ty_str(&ty));
}

#[test]
fn extract_option_not_detected_as_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<i32>);
    let (_returned, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
}

#[test]
fn extract_box_not_detected_as_arc() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<u32>);
    let (_returned, found) = try_extract_inner_type(&ty, "Arc", &skip);
    assert!(!found);
}

#[test]
fn extract_returns_original_type_on_mismatch() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (returned, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(ty_str(&returned), ty_str(&ty));
}

#[test]
fn extract_reference_type_not_detected() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    let (_returned, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
}

#[test]
fn extract_tuple_type_not_detected() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!((i32, i32));
    let (_returned, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
}

#[test]
fn extract_vec_with_string_inner() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_with_bool_inner() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_vec_with_path_inner() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<std::string::String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "std :: string :: String");
}

// =============================================================================
// Section 2: try_extract_inner_type — skip_over behaviour (tests 21–35)
// =============================================================================

#[test]
fn extract_through_box_skip_to_find_vec() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_through_arc_skip_to_find_option() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<Option<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_through_rc_skip_to_find_vec() {
    let skip: HashSet<&str> = ["Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Rc<Vec<u8>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_through_multiple_skips() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Vec<i32>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_skip_does_not_match_target_inside_non_skip_wrapper() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (_returned, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
}

#[test]
fn extract_skip_returns_false_when_target_absent() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let (_returned, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
}

#[test]
fn extract_skip_returns_original_when_target_absent() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let (returned, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(ty_str(&returned), ty_str(&ty));
}

#[test]
fn extract_through_cell_skip() {
    let skip: HashSet<&str> = ["Cell"].into_iter().collect();
    let ty: Type = parse_quote!(Cell<Vec<f32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn extract_through_refcell_skip() {
    let skip: HashSet<&str> = ["RefCell"].into_iter().collect();
    let ty: Type = parse_quote!(RefCell<Option<u16>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn extract_skip_self_does_not_recurse_into_same_wrapper() {
    // When inner_of="Vec" and skip={"Vec"}, the outer Vec matches inner_of first
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_with_empty_skip_set_still_detects_wrapper() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<u32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn extract_deep_nesting_through_three_skip_layers() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Rc<Vec<bool>>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "bool");
}

// =============================================================================
// Section 3: try_extract_inner_type — nested wrappers (tests 36–45)
// =============================================================================

#[test]
fn extract_nested_vec_option_detects_outer_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn extract_nested_option_vec_detects_outer_option() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn extract_nested_vec_vec_only_outer_detected() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn extract_nested_option_option_only_outer_detected() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<Option<bool>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "Option < bool >");
}

#[test]
fn extract_vec_with_tuple_inner() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<(i32, String)>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "(i32 , String)");
}

#[test]
fn extract_option_with_tuple_inner() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<(i32, i32)>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "(i32 , i32)");
}

#[test]
fn extract_vec_with_reference_inner() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<&str>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn extract_option_with_reference_inner() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<&u8>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "& u8");
}

#[test]
fn extract_vec_with_complex_path_inner() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<std::collections::HashMap<String, i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(
        ty_str(&inner),
        "std :: collections :: HashMap < String , i32 >"
    );
}

#[test]
fn extract_progressive_nested_detection() {
    // First extract outer Vec, then extract inner Option from result
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<Option<i32>>);

    let (after_vec, found_vec) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found_vec);
    assert_eq!(ty_str(&after_vec), "Option < i32 >");

    let (after_opt, found_opt) = try_extract_inner_type(&after_vec, "Option", &skip);
    assert!(found_opt);
    assert_eq!(ty_str(&after_opt), "i32");
}

// =============================================================================
// Section 4: filter_inner_type — wrapper stripping (tests 46–60)
// =============================================================================

#[test]
fn filter_strips_vec_when_in_skip_set() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_strips_option_when_in_skip_set() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_strips_box_when_in_skip_set() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<f64>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "f64");
}

#[test]
fn filter_strips_arc_when_in_skip_set() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<bool>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "bool");
}

#[test]
fn filter_strips_rc_when_in_skip_set() {
    let skip: HashSet<&str> = ["Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Rc<u16>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "u16");
}

#[test]
fn filter_does_not_strip_non_matching_wrapper() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_does_not_strip_plain_type() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn filter_strips_nested_matching_wrappers() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<String>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn filter_strips_three_nested_matching_wrappers() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Rc<u32>>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "u32");
}

#[test]
fn filter_stops_at_non_matching_inner_wrapper() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "Vec < i32 >");
}

#[test]
fn filter_with_empty_skip_returns_unchanged() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), ty_str(&ty));
}

#[test]
fn filter_reference_type_returns_unchanged() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(&str);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "& str");
}

#[test]
fn filter_tuple_type_returns_unchanged() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!((i32, bool));
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "(i32 , bool)");
}

#[test]
fn filter_cell_in_skip_set() {
    let skip: HashSet<&str> = ["Cell"].into_iter().collect();
    let ty: Type = parse_quote!(Cell<u8>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "u8");
}

#[test]
fn filter_refcell_in_skip_set() {
    let skip: HashSet<&str> = ["RefCell"].into_iter().collect();
    let ty: Type = parse_quote!(RefCell<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "String");
}

// =============================================================================
// Section 5: wrap_leaf_type — wrapper creation (tests 61–75)
// =============================================================================

#[test]
fn wrap_plain_type_gets_wrapped() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(i32);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_string_type_gets_wrapped() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_vec_in_skip_wraps_inner_instead() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Vec < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_option_in_skip_wraps_inner_instead() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Option < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_box_in_skip_wraps_inner_instead() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<f64>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Box < adze :: WithLeaf < f64 > >"
    );
}

#[test]
fn wrap_nested_skip_wrappers_wraps_leaf() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_non_skip_wrapper_wraps_whole_thing() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Option<i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < Option < i32 > >"
    );
}

#[test]
fn wrap_vec_vec_in_skip_wraps_inner_leaf() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Vec < Vec < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_empty_skip_wraps_vec_entirely() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < Vec < i32 > >"
    );
}

#[test]
fn wrap_reference_type_gets_wrapped() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_tuple_type_gets_wrapped() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!((i32, bool));
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < (i32 , bool) >"
    );
}

#[test]
fn wrap_arc_in_skip_wraps_inner() {
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Arc<u64>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Arc < adze :: WithLeaf < u64 > >"
    );
}

#[test]
fn wrap_rc_in_skip_wraps_inner() {
    let skip: HashSet<&str> = ["Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Rc<bool>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Rc < adze :: WithLeaf < bool > >"
    );
}

#[test]
fn wrap_cell_in_skip_wraps_inner() {
    let skip: HashSet<&str> = ["Cell"].into_iter().collect();
    let ty: Type = parse_quote!(Cell<u8>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Cell < adze :: WithLeaf < u8 > >"
    );
}

#[test]
fn wrap_refcell_in_skip_wraps_inner() {
    let skip: HashSet<&str> = ["RefCell"].into_iter().collect();
    let ty: Type = parse_quote!(RefCell<i16>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "RefCell < adze :: WithLeaf < i16 > >"
    );
}

// =============================================================================
// Section 6: HashMap and multi-param types (tests 76–80)
// =============================================================================

#[test]
fn extract_hashmap_first_type_param_returned() {
    // HashMap<K,V> — when matched as inner_of, first generic arg is returned
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, found) = try_extract_inner_type(&ty, "HashMap", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn filter_hashmap_in_skip_returns_first_type_param() {
    let skip: HashSet<&str> = ["HashMap"].into_iter().collect();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn wrap_hashmap_in_skip_wraps_all_type_args() {
    let skip: HashSet<&str> = ["HashMap"].into_iter().collect();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&wrapped),
        "HashMap < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn extract_hashmap_not_detected_as_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (_returned, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
}

#[test]
fn wrap_hashmap_not_in_skip_wraps_entirely() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

// =============================================================================
// Section 7: Additional edge cases and combinations (tests 81–86)
// =============================================================================

#[test]
fn extract_fully_qualified_path_vec() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    // Last segment is still "Vec"
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn filter_fully_qualified_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(std::boxed::Box<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "String");
}

#[test]
fn wrap_custom_in_skip_wraps_inner() {
    let skip: HashSet<&str> = ["MyWrapper"].into_iter().collect();
    let ty: Type = parse_quote!(MyWrapper<u32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "MyWrapper < adze :: WithLeaf < u32 > >"
    );
}

#[test]
fn extract_vec_with_unit_inner() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<()>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "()");
}

#[test]
fn filter_deeply_nested_mixed_wrappers() {
    let skip: HashSet<&str> = ["Box", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Option<Box<i32>>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip)), "i32");
}

#[test]
fn wrap_deeply_nested_all_in_skip() {
    let skip: HashSet<&str> = ["Vec", "Option", "Box"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip)),
        "Vec < Option < Box < adze :: WithLeaf < i32 > > > >"
    );
}
