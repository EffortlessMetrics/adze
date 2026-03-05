use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use std::collections::HashSet;
use syn::{Type, parse_quote};

/// Helper: convert a `syn::Type` to its token string for assertion.
fn ty_str(ty: &Type) -> String {
    quote::quote!(#ty).to_string()
}

// ============================================================================
// Category 1: try_extract — Vec<T> extraction (8 tests)
// ============================================================================

#[test]
fn wl_v8_extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn wl_v8_extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn wl_v8_extract_vec_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "u8");
}

#[test]
fn wl_v8_extract_vec_u16() {
    let ty: Type = parse_quote!(Vec<u16>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "u16");
}

#[test]
fn wl_v8_extract_vec_u32() {
    let ty: Type = parse_quote!(Vec<u32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "u32");
}

#[test]
fn wl_v8_extract_vec_u64() {
    let ty: Type = parse_quote!(Vec<u64>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "u64");
}

#[test]
fn wl_v8_extract_vec_f32() {
    let ty: Type = parse_quote!(Vec<f32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "f32");
}

#[test]
fn wl_v8_extract_vec_f64() {
    let ty: Type = parse_quote!(Vec<f64>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "f64");
}

// ============================================================================
// Category 2: try_extract — Option<T> extraction (6 tests)
// ============================================================================

#[test]
fn wl_v8_extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn wl_v8_extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn wl_v8_extract_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn wl_v8_extract_option_u64() {
    let ty: Type = parse_quote!(Option<u64>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "u64");
}

#[test]
fn wl_v8_extract_option_nested_vec() {
    let ty: Type = parse_quote!(Option<Vec<u32>>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "Vec < u32 >");
}

#[test]
fn wl_v8_extract_option_custom() {
    let ty: Type = parse_quote!(Option<MyStruct>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "MyStruct");
}

// ============================================================================
// Category 3: try_extract — non-matching (returns original, false) (8 tests)
// ============================================================================

#[test]
fn wl_v8_extract_plain_i32_not_vec() {
    let ty: Type = parse_quote!(i32);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn wl_v8_extract_plain_string_not_option() {
    let ty: Type = parse_quote!(String);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn wl_v8_extract_plain_bool_not_box() {
    let ty: Type = parse_quote!(bool);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn wl_v8_extract_plain_u8_not_arc() {
    let ty: Type = parse_quote!(u8);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Arc", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), "u8");
}

#[test]
fn wl_v8_extract_plain_f32_not_rc() {
    let ty: Type = parse_quote!(f32);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Rc", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), "f32");
}

#[test]
fn wl_v8_extract_option_looking_for_vec() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), "Option < i32 >");
}

#[test]
fn wl_v8_extract_vec_looking_for_option() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), "Vec < String >");
}

#[test]
fn wl_v8_extract_unit_not_matched() {
    let ty: Type = parse_quote!(());
    let skip = HashSet::new();
    let (_, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
}

// ============================================================================
// Category 4: try_extract — with skip_over (10 tests)
// ============================================================================

#[test]
fn wl_v8_extract_skip_option_over_box() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn wl_v8_extract_skip_vec_over_option() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn wl_v8_extract_skip_two_layers() {
    let ty: Type = parse_quote!(Box<Option<Vec<u8>>>);
    let skip: HashSet<&str> = ["Box", "Option"].into_iter().collect();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "u8");
}

#[test]
fn wl_v8_extract_skip_triple_box() {
    let ty: Type = parse_quote!(Box<Box<Box<Vec<u64>>>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "u64");
}

#[test]
fn wl_v8_extract_skip_not_matching_returns_false() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), "Box < Option < i32 > >");
}

#[test]
fn wl_v8_extract_skip_empty_no_traversal() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
    assert_eq!(ty_str(&result), "Box < Option < i32 > >");
}

#[test]
fn wl_v8_extract_skip_wrong_wrapper() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let (_result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!found);
}

#[test]
fn wl_v8_extract_skip_middle_match() {
    let ty: Type = parse_quote!(Box<Vec<Option<String>>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "Option < String >");
}

#[test]
fn wl_v8_extract_skip_arc_over_rc() {
    let ty: Type = parse_quote!(Rc<Arc<Option<bool>>>);
    let skip: HashSet<&str> = ["Rc", "Arc"].into_iter().collect();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn wl_v8_extract_skip_all_three_wrappers() {
    let ty: Type = parse_quote!(Arc<Box<Option<Vec<f64>>>>);
    let skip: HashSet<&str> = ["Arc", "Box", "Option"].into_iter().collect();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "f64");
}

// ============================================================================
// Category 5: try_extract — Box and other wrappers (4 tests)
// ============================================================================

#[test]
fn wl_v8_extract_box_i32() {
    let ty: Type = parse_quote!(Box<i32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Box", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn wl_v8_extract_arc_string() {
    let ty: Type = parse_quote!(Arc<String>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Arc", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn wl_v8_extract_rc_u32() {
    let ty: Type = parse_quote!(Rc<u32>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Rc", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "u32");
}

#[test]
fn wl_v8_extract_result_first_generic() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Result", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "i32");
}

// ============================================================================
// Category 6: filter_inner_type — empty skip returns original (8 tests)
// ============================================================================

#[test]
fn wl_v8_filter_empty_skip_i32() {
    let ty: Type = parse_quote!(i32);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn wl_v8_filter_empty_skip_string() {
    let ty: Type = parse_quote!(String);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn wl_v8_filter_empty_skip_vec_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

#[test]
fn wl_v8_filter_empty_skip_option_unchanged() {
    let ty: Type = parse_quote!(Option<String>);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Option < String >");
}

#[test]
fn wl_v8_filter_empty_skip_box_unchanged() {
    let ty: Type = parse_quote!(Box<u8>);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Box < u8 >");
}

#[test]
fn wl_v8_filter_empty_skip_bool() {
    let ty: Type = parse_quote!(bool);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn wl_v8_filter_empty_skip_f64() {
    let ty: Type = parse_quote!(f64);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "f64");
}

#[test]
fn wl_v8_filter_empty_skip_custom_type() {
    let ty: Type = parse_quote!(MyStruct);
    let skip = HashSet::new();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "MyStruct");
}

// ============================================================================
// Category 7: filter_inner_type — stripping single wrapper (8 tests)
// ============================================================================

#[test]
fn wl_v8_filter_strip_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn wl_v8_filter_strip_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn wl_v8_filter_strip_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn wl_v8_filter_strip_arc_u32() {
    let ty: Type = parse_quote!(Arc<u32>);
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "u32");
}

#[test]
fn wl_v8_filter_strip_rc_bool() {
    let ty: Type = parse_quote!(Rc<bool>);
    let skip: HashSet<&str> = ["Rc"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn wl_v8_filter_strip_vec_f32() {
    let ty: Type = parse_quote!(Vec<f32>);
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "f32");
}

#[test]
fn wl_v8_filter_strip_option_u16() {
    let ty: Type = parse_quote!(Option<u16>);
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "u16");
}

#[test]
fn wl_v8_filter_strip_box_f64() {
    let ty: Type = parse_quote!(Box<f64>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "f64");
}

// ============================================================================
// Category 8: filter_inner_type — nested / multiple skips (8 tests)
// ============================================================================

#[test]
fn wl_v8_filter_nested_vec_option_i32() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn wl_v8_filter_nested_option_vec_string() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn wl_v8_filter_nested_box_option_u8() {
    let ty: Type = parse_quote!(Box<Option<u8>>);
    let skip: HashSet<&str> = ["Box", "Option"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "u8");
}

#[test]
fn wl_v8_filter_nested_triple_box() {
    let ty: Type = parse_quote!(Box<Box<Box<i32>>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn wl_v8_filter_nested_partial_skip() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Option < i32 >");
}

#[test]
fn wl_v8_filter_nested_inner_only() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Vec < Option < i32 > >");
}

#[test]
fn wl_v8_filter_nested_arc_box_vec() {
    let ty: Type = parse_quote!(Arc<Box<Vec<u64>>>);
    let skip: HashSet<&str> = ["Arc", "Box", "Vec"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "u64");
}

#[test]
fn wl_v8_filter_nested_deep_four_layers() {
    let ty: Type = parse_quote!(Rc<Arc<Box<Option<bool>>>>);
    let skip: HashSet<&str> = ["Rc", "Arc", "Box", "Option"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "bool");
}

// ============================================================================
// Category 9: wrap_leaf_type — basic wrapping (no skip) (8 tests)
// ============================================================================

#[test]
fn wl_v8_wrap_i32_no_skip() {
    let ty: Type = parse_quote!(i32);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < i32 >");
}

#[test]
fn wl_v8_wrap_string_no_skip() {
    let ty: Type = parse_quote!(String);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < String >");
}

#[test]
fn wl_v8_wrap_bool_no_skip() {
    let ty: Type = parse_quote!(bool);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < bool >");
}

#[test]
fn wl_v8_wrap_u8_no_skip() {
    let ty: Type = parse_quote!(u8);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < u8 >");
}

#[test]
fn wl_v8_wrap_u32_no_skip() {
    let ty: Type = parse_quote!(u32);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < u32 >");
}

#[test]
fn wl_v8_wrap_f64_no_skip() {
    let ty: Type = parse_quote!(f64);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < f64 >");
}

#[test]
fn wl_v8_wrap_custom_struct_no_skip() {
    let ty: Type = parse_quote!(MyStruct);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < MyStruct >");
}

#[test]
fn wl_v8_wrap_custom_enum_no_skip() {
    let ty: Type = parse_quote!(MyEnum);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < MyEnum >");
}

// ============================================================================
// Category 10: wrap_leaf_type — with skip (wraps inner) (10 tests)
// ============================================================================

#[test]
fn wl_v8_wrap_vec_i32_skip_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Vec < adze :: WithLeaf < i32 > >");
}

#[test]
fn wl_v8_wrap_option_string_skip_option() {
    let ty: Type = parse_quote!(Option<String>);
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Option < adze :: WithLeaf < String > >");
}

#[test]
fn wl_v8_wrap_box_u32_skip_box() {
    let ty: Type = parse_quote!(Box<u32>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Box < adze :: WithLeaf < u32 > >");
}

#[test]
fn wl_v8_wrap_arc_f64_skip_arc() {
    let ty: Type = parse_quote!(Arc<f64>);
    let skip: HashSet<&str> = ["Arc"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Arc < adze :: WithLeaf < f64 > >");
}

#[test]
fn wl_v8_wrap_rc_bool_skip_rc() {
    let ty: Type = parse_quote!(Rc<bool>);
    let skip: HashSet<&str> = ["Rc"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Rc < adze :: WithLeaf < bool > >");
}

#[test]
fn wl_v8_wrap_vec_option_skip_both() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wl_v8_wrap_option_vec_skip_both() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Option < Vec < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wl_v8_wrap_box_option_skip_both() {
    let ty: Type = parse_quote!(Box<Option<u8>>);
    let skip: HashSet<&str> = ["Box", "Option"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Box < Option < adze :: WithLeaf < u8 > > >"
    );
}

#[test]
fn wl_v8_wrap_vec_skip_only_outer() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Vec < adze :: WithLeaf < Option < i32 > > >"
    );
}

#[test]
fn wl_v8_wrap_no_skip_wraps_whole_generic() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Vec < i32 > >");
}

// ============================================================================
// Category 11: wrap_leaf_type — deeply nested with skip (6 tests)
// ============================================================================

#[test]
fn wl_v8_wrap_triple_box_skip_box() {
    let ty: Type = parse_quote!(Box<Box<Box<i32>>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Box < Box < Box < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn wl_v8_wrap_arc_box_option_skip_all() {
    let ty: Type = parse_quote!(Arc<Box<Option<u64>>>);
    let skip: HashSet<&str> = ["Arc", "Box", "Option"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Arc < Box < Option < adze :: WithLeaf < u64 > > > >"
    );
}

#[test]
fn wl_v8_wrap_rc_arc_box_vec_skip_all() {
    let ty: Type = parse_quote!(Rc<Arc<Box<Vec<f32>>>>);
    let skip: HashSet<&str> = ["Rc", "Arc", "Box", "Vec"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Rc < Arc < Box < Vec < adze :: WithLeaf < f32 > > > > >"
    );
}

#[test]
fn wl_v8_wrap_vec_vec_skip_vec() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Vec < Vec < adze :: WithLeaf < i32 > > >");
}

#[test]
fn wl_v8_wrap_option_option_skip_option() {
    let ty: Type = parse_quote!(Option<Option<bool>>);
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Option < Option < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wl_v8_wrap_deep_four_skip_all() {
    let ty: Type = parse_quote!(Box<Option<Vec<Arc<u16>>>>);
    let skip: HashSet<&str> = ["Box", "Option", "Vec", "Arc"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Box < Option < Vec < Arc < adze :: WithLeaf < u16 > > > > >"
    );
}

// ============================================================================
// Category 12: wrap_leaf_type — non-path types (4 tests)
// ============================================================================

#[test]
fn wl_v8_wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < () >");
}

#[test]
fn wl_v8_wrap_reference_type() {
    let ty: Type = parse_quote!(&'static str);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < & 'static str >");
}

#[test]
fn wl_v8_wrap_mutable_ref_type() {
    let ty: Type = parse_quote!(&mut i32);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < & mut i32 >");
}

#[test]
fn wl_v8_wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, String));
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < (i32 , String) >");
}

// ============================================================================
// Category 13: roundtrip — extract then wrap consistency (6 tests)
// ============================================================================

#[test]
fn wl_v8_roundtrip_extract_vec_then_wrap() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
    let wrapped = wrap_leaf_type(&inner, &skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wl_v8_roundtrip_extract_option_then_wrap() {
    let ty: Type = parse_quote!(Option<String>);
    let skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    let wrapped = wrap_leaf_type(&inner, &skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wl_v8_roundtrip_filter_then_wrap() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&filtered), "i32");
    let empty_skip = HashSet::new();
    let wrapped = wrap_leaf_type(&filtered, &empty_skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wl_v8_roundtrip_filter_nested_then_wrap() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&filtered), "bool");
    let empty_skip = HashSet::new();
    let wrapped = wrap_leaf_type(&filtered, &empty_skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < bool >");
}

#[test]
fn wl_v8_roundtrip_extract_with_skip_then_wrap() {
    let ty: Type = parse_quote!(Box<Vec<u64>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "u64");
    let empty_skip = HashSet::new();
    let wrapped = wrap_leaf_type(&inner, &empty_skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u64 >");
}

#[test]
fn wl_v8_roundtrip_wrap_preserves_structure() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip_ve: HashSet<&str> = ["Vec"].into_iter().collect();
    let skip_vo: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let partial = wrap_leaf_type(&ty, &skip_ve);
    assert_eq!(
        ty_str(&partial),
        "Vec < adze :: WithLeaf < Option < i32 > > >"
    );
    let full = wrap_leaf_type(&ty, &skip_vo);
    assert_eq!(ty_str(&full), "Vec < Option < adze :: WithLeaf < i32 > > >");
}

// ============================================================================
// Category 14: filter + extract composition (4 tests)
// ============================================================================

#[test]
fn wl_v8_filter_then_extract_succeeds() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let filter_skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &filter_skip);
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
    let empty_skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&filtered, "Vec", &empty_skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn wl_v8_filter_then_extract_fails() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let filter_skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &filter_skip);
    assert_eq!(ty_str(&filtered), "Option < i32 >");
    let empty_skip = HashSet::new();
    let (_, found) = try_extract_inner_type(&filtered, "Vec", &empty_skip);
    assert!(!found);
}

#[test]
fn wl_v8_extract_then_filter() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let empty_skip = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "Option < Box < i32 > >");
    let filter_skip: HashSet<&str> = ["Option", "Box"].into_iter().collect();
    let filtered = filter_inner_type(&inner, &filter_skip);
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn wl_v8_full_pipeline_extract_filter_wrap() {
    let ty: Type = parse_quote!(Vec<Box<Option<u16>>>);
    let empty: HashSet<&str> = HashSet::new();
    let (after_extract, found) = try_extract_inner_type(&ty, "Vec", &empty);
    assert!(found);
    assert_eq!(ty_str(&after_extract), "Box < Option < u16 > >");
    let strip: HashSet<&str> = ["Box", "Option"].into_iter().collect();
    let after_filter = filter_inner_type(&after_extract, &strip);
    assert_eq!(ty_str(&after_filter), "u16");
    let wrapped = wrap_leaf_type(&after_filter, &empty);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u16 >");
}

// ============================================================================
// Category 15: edge cases — various scenarios (6 tests)
// ============================================================================

#[test]
fn wl_v8_edge_nested_same_wrapper_extract() {
    let ty: Type = parse_quote!(Vec<Vec<Vec<i32>>>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "Vec < Vec < i32 > >");
}

#[test]
fn wl_v8_edge_filter_stops_at_non_skip() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

#[test]
fn wl_v8_edge_wrap_skip_partial_chain() {
    let ty: Type = parse_quote!(Box<Arc<i32>>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Box < adze :: WithLeaf < Arc < i32 > > >");
}

#[test]
fn wl_v8_edge_extract_preserves_nested() {
    let ty: Type = parse_quote!(Option<Vec<Box<String>>>);
    let skip = HashSet::new();
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(ty_str(&result), "Vec < Box < String > >");
}

#[test]
fn wl_v8_edge_wrap_idempotent_check() {
    let ty: Type = parse_quote!(i32);
    let skip = HashSet::new();
    let once = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&once), "adze :: WithLeaf < i32 >");
    let twice = wrap_leaf_type(&once, &skip);
    assert_eq!(
        ty_str(&twice),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wl_v8_edge_filter_with_irrelevant_skip() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = ["Option", "Box", "Arc"].into_iter().collect();
    let result = filter_inner_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Vec < i32 >");
}
