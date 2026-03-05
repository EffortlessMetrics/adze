use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use std::collections::HashSet;
use syn::{parse_quote, Type};

/// Helper: convert a `syn::Type` to its token string for assertion.
fn ty_str(ty: &Type) -> String {
    quote::quote!(#ty).to_string()
}

// ============================================================================
// Category 1: wrap_prim_* — Wrap primitive leaf types (10 tests)
// ============================================================================

#[test]
fn wrap_prim_i32() {
    let ty: Type = parse_quote!(i32);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_prim_i8() {
    let ty: Type = parse_quote!(i8);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < i8 >");
}

#[test]
fn wrap_prim_i16() {
    let ty: Type = parse_quote!(i16);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < i16 >");
}

#[test]
fn wrap_prim_i64() {
    let ty: Type = parse_quote!(i64);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < i64 >");
}

#[test]
fn wrap_prim_u8() {
    let ty: Type = parse_quote!(u8);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < u8 >");
}

#[test]
fn wrap_prim_u16() {
    let ty: Type = parse_quote!(u16);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < u16 >");
}

#[test]
fn wrap_prim_u32() {
    let ty: Type = parse_quote!(u32);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < u32 >");
}

#[test]
fn wrap_prim_u64() {
    let ty: Type = parse_quote!(u64);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < u64 >");
}

#[test]
fn wrap_prim_f32() {
    let ty: Type = parse_quote!(f32);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < f32 >");
}

#[test]
fn wrap_prim_f64() {
    let ty: Type = parse_quote!(f64);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < f64 >");
}

// ============================================================================
// Category 2: wrap_named_* — Wrap named / custom types (8 tests)
// ============================================================================

#[test]
fn wrap_named_string() {
    let ty: Type = parse_quote!(String);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_named_bool() {
    let ty: Type = parse_quote!(bool);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_named_usize() {
    let ty: Type = parse_quote!(usize);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < usize >");
}

#[test]
fn wrap_named_isize() {
    let ty: Type = parse_quote!(isize);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < isize >");
}

#[test]
fn wrap_named_char() {
    let ty: Type = parse_quote!(char);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < char >");
}

#[test]
fn wrap_named_custom_struct() {
    let ty: Type = parse_quote!(MyWrapper);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < MyWrapper >");
}

#[test]
fn wrap_named_custom_enum() {
    let ty: Type = parse_quote!(MyEnum);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < MyEnum >");
}

#[test]
fn wrap_named_qualified_path() {
    let ty: Type = parse_quote!(std::string::String);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "adze :: WithLeaf < std :: string :: String >"
    );
}

// ============================================================================
// Category 3: wrap_skip_single_* — Single skip-over container (8 tests)
// ============================================================================

#[test]
fn wrap_skip_single_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Vec < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_skip_single_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_skip_single_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_skip_single_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Option < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_skip_single_box_u8() {
    let ty: Type = parse_quote!(Box<u8>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Box < adze :: WithLeaf < u8 > >");
}

#[test]
fn wrap_skip_single_box_bool() {
    let ty: Type = parse_quote!(Box<bool>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Box < adze :: WithLeaf < bool > >");
}

#[test]
fn wrap_skip_single_rc_f64() {
    let ty: Type = parse_quote!(Rc<f64>);
    let skip: HashSet<&str> = HashSet::from(["Rc"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Rc < adze :: WithLeaf < f64 > >");
}

#[test]
fn wrap_skip_single_arc_f64() {
    let ty: Type = parse_quote!(Arc<f64>);
    let skip: HashSet<&str> = HashSet::from(["Arc"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Arc < adze :: WithLeaf < f64 > >");
}

// ============================================================================
// Category 4: wrap_skip_multi_* — Multiple skip-over containers (8 tests)
// ============================================================================

#[test]
fn wrap_skip_multi_vec_option_i32() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_skip_multi_option_vec_string() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Option < Vec < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_skip_multi_box_option_u32() {
    let ty: Type = parse_quote!(Box<Option<u32>>);
    let skip: HashSet<&str> = HashSet::from(["Box", "Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Box < Option < adze :: WithLeaf < u32 > > >"
    );
}

#[test]
fn wrap_skip_multi_box_vec_i64() {
    let ty: Type = parse_quote!(Box<Vec<i64>>);
    let skip: HashSet<&str> = HashSet::from(["Box", "Vec"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Box < Vec < adze :: WithLeaf < i64 > > >"
    );
}

#[test]
fn wrap_skip_multi_arc_option_bool() {
    let ty: Type = parse_quote!(Arc<Option<bool>>);
    let skip: HashSet<&str> = HashSet::from(["Arc", "Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Arc < Option < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wrap_skip_multi_vec_box_u8() {
    let ty: Type = parse_quote!(Vec<Box<u8>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Box"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Vec < Box < adze :: WithLeaf < u8 > > >"
    );
}

#[test]
fn wrap_skip_multi_option_box_f32() {
    let ty: Type = parse_quote!(Option<Box<f32>>);
    let skip: HashSet<&str> = HashSet::from(["Option", "Box"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Option < Box < adze :: WithLeaf < f32 > > >"
    );
}

#[test]
fn wrap_skip_multi_rc_vec_char() {
    let ty: Type = parse_quote!(Rc<Vec<char>>);
    let skip: HashSet<&str> = HashSet::from(["Rc", "Vec"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Rc < Vec < adze :: WithLeaf < char > > >"
    );
}

// ============================================================================
// Category 5: wrap_deep_* — Triple+ nesting with skip (8 tests)
// ============================================================================

#[test]
fn wrap_deep_vec_option_box_i32() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Vec < Option < Box < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn wrap_deep_option_vec_box_string() {
    let ty: Type = parse_quote!(Option<Vec<Box<String>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Option < Vec < Box < adze :: WithLeaf < String > > > >"
    );
}

#[test]
fn wrap_deep_box_box_box_u64() {
    let ty: Type = parse_quote!(Box<Box<Box<u64>>>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Box < Box < Box < adze :: WithLeaf < u64 > > > >"
    );
}

#[test]
fn wrap_deep_vec_vec_vec_i32() {
    let ty: Type = parse_quote!(Vec<Vec<Vec<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Vec < Vec < Vec < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn wrap_deep_option_option_option_bool() {
    let ty: Type = parse_quote!(Option<Option<Option<bool>>>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Option < Option < Option < adze :: WithLeaf < bool > > > >"
    );
}

#[test]
fn wrap_deep_four_levels() {
    let ty: Type = parse_quote!(Vec<Option<Box<Rc<u16>>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box", "Rc"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Vec < Option < Box < Rc < adze :: WithLeaf < u16 > > > > >"
    );
}

#[test]
fn wrap_deep_partial_skip_stops_early() {
    // Only Vec is skipped; Option<Box<i32>> is a leaf from Vec's perspective.
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Vec < adze :: WithLeaf < Option < Box < i32 > > > >"
    );
}

#[test]
fn wrap_deep_partial_skip_mid_layer() {
    // Skip Vec and Box but not Option; Option becomes the leaf inside Vec.
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Box"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Vec < adze :: WithLeaf < Option < i32 > > >"
    );
}

// ============================================================================
// Category 6: wrap_noskip_* — Containers NOT in skip set (8 tests)
// ============================================================================

#[test]
fn wrap_noskip_vec_i32_empty_skip() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn wrap_noskip_option_string_empty_skip() {
    let ty: Type = parse_quote!(Option<String>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Option < String > >");
}

#[test]
fn wrap_noskip_box_u8_empty_skip() {
    let ty: Type = parse_quote!(Box<u8>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Box < u8 > >");
}

#[test]
fn wrap_noskip_rc_bool_empty_skip() {
    let ty: Type = parse_quote!(Rc<bool>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Rc < bool > >");
}

#[test]
fn wrap_noskip_arc_f64_empty_skip() {
    let ty: Type = parse_quote!(Arc<f64>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Arc < f64 > >");
}

#[test]
fn wrap_noskip_vec_option_wrong_skip() {
    // Skip has "Box" but type is Vec<Option<..>>; nothing is skipped.
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Box"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "adze :: WithLeaf < Vec < Option < i32 > > >"
    );
}

#[test]
fn wrap_noskip_custom_generic() {
    let ty: Type = parse_quote!(MyWrapper<i32>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < MyWrapper < i32 > >");
}

#[test]
fn wrap_noskip_custom_generic_in_skip() {
    // Custom wrapper IS in skip set: inner type gets wrapped.
    let ty: Type = parse_quote!(MyWrapper<i32>);
    let skip: HashSet<&str> = HashSet::from(["MyWrapper"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "MyWrapper < adze :: WithLeaf < i32 > >");
}

// ============================================================================
// Category 7: wrap_nonpath_* — Non-path types (references, tuples, unit) (8 tests)
// ============================================================================

#[test]
fn wrap_nonpath_reference_i32() {
    let ty: Type = parse_quote!(&i32);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < & i32 >");
}

#[test]
fn wrap_nonpath_reference_mut_i32() {
    let ty: Type = parse_quote!(&mut i32);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < & mut i32 >");
}

#[test]
fn wrap_nonpath_static_str() {
    let ty: Type = parse_quote!(&'static str);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < & 'static str >");
}

#[test]
fn wrap_nonpath_tuple_pair() {
    let ty: Type = parse_quote!((i32, String));
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < (i32 , String) >");
}

#[test]
fn wrap_nonpath_tuple_triple() {
    let ty: Type = parse_quote!((u8, u16, u32));
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < (u8 , u16 , u32) >");
}

#[test]
fn wrap_nonpath_unit() {
    let ty: Type = parse_quote!(());
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < () >");
}

#[test]
fn wrap_nonpath_slice() {
    let ty: Type = parse_quote!([u8]);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < [u8] >");
}

#[test]
fn wrap_nonpath_array() {
    let ty: Type = parse_quote!([u8; 4]);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < [u8 ; 4] >");
}

// ============================================================================
// Category 8: wrap_roundtrip_* — Wrap then extract roundtrip (8 tests)
// ============================================================================

#[test]
fn wrap_roundtrip_extract_from_skipped_vec() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    // The inner type of Vec should now be wrapped.
    let skip_extract = HashSet::new();
    let (inner, found) = try_extract_inner_type(&wrapped, "Vec", &skip_extract);
    assert!(found);
    assert_eq!(ty_str(&inner), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_roundtrip_extract_from_skipped_option() {
    let ty: Type = parse_quote!(Option<String>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let skip_extract = HashSet::new();
    let (inner, found) = try_extract_inner_type(&wrapped, "Option", &skip_extract);
    assert!(found);
    assert_eq!(ty_str(&inner), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_roundtrip_filter_then_wrap() {
    let ty: Type = parse_quote!(Box<i32>);
    let skip_filter: HashSet<&str> = HashSet::from(["Box"]);
    let filtered = filter_inner_type(&ty, &skip_filter);
    assert_eq!(ty_str(&filtered), "i32");
    let skip_wrap = HashSet::new();
    let wrapped = wrap_leaf_type(&filtered, &skip_wrap);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_roundtrip_wrap_then_filter() {
    let ty: Type = parse_quote!(u64);
    let skip = HashSet::new();
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u64 >");
    // Filtering with "adze" in skip drills into adze::WithLeaf.
    let skip_filter: HashSet<&str> = HashSet::from(["WithLeaf"]);
    let filtered = filter_inner_type(&wrapped, &skip_filter);
    // adze::WithLeaf is a path type; last segment is "WithLeaf".
    assert_eq!(ty_str(&filtered), "u64");
}

#[test]
fn wrap_roundtrip_nested_extract() {
    let ty: Type = parse_quote!(Vec<Option<u8>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let skip_extract = HashSet::new();
    let (vec_inner, found) = try_extract_inner_type(&wrapped, "Vec", &skip_extract);
    assert!(found);
    assert_eq!(
        ty_str(&vec_inner),
        "Option < adze :: WithLeaf < u8 > >"
    );
}

#[test]
fn wrap_roundtrip_double_wrap_leaf() {
    // Wrapping an already-wrapped leaf wraps it again.
    let ty: Type = parse_quote!(i32);
    let skip = HashSet::new();
    let once = wrap_leaf_type(&ty, &skip);
    let twice = wrap_leaf_type(&once, &skip);
    assert_eq!(
        ty_str(&twice),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_roundtrip_skip_preserves_outer() {
    let ty: Type = parse_quote!(Vec<bool>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    let outer_str = ty_str(&wrapped);
    assert!(outer_str.starts_with("Vec"));
}

#[test]
fn wrap_roundtrip_noskip_wraps_entire_container() {
    let ty: Type = parse_quote!(Vec<bool>);
    let skip = HashSet::new();
    let wrapped = wrap_leaf_type(&ty, &skip);
    let outer_str = ty_str(&wrapped);
    assert!(outer_str.starts_with("adze :: WithLeaf"));
}

// ============================================================================
// Category 9: wrap_nesting_order_* — Verify correct nesting order (8 tests)
// ============================================================================

#[test]
fn wrap_nesting_order_vec_before_leaf() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let result = wrap_leaf_type(&ty, &skip);
    let s = ty_str(&result);
    let vec_pos = s.find("Vec").expect("should contain Vec");
    let leaf_pos = s.find("WithLeaf").expect("should contain WithLeaf");
    assert!(vec_pos < leaf_pos);
}

#[test]
fn wrap_nesting_order_option_before_leaf() {
    let ty: Type = parse_quote!(Option<String>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    let s = ty_str(&result);
    let opt_pos = s.find("Option").expect("should contain Option");
    let leaf_pos = s.find("WithLeaf").expect("should contain WithLeaf");
    assert!(opt_pos < leaf_pos);
}

#[test]
fn wrap_nesting_order_vec_option_leaf() {
    let ty: Type = parse_quote!(Vec<Option<f32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    let s = ty_str(&result);
    let vec_pos = s.find("Vec").expect("should contain Vec");
    let opt_pos = s.find("Option").expect("should contain Option");
    let leaf_pos = s.find("WithLeaf").expect("should contain WithLeaf");
    assert!(vec_pos < opt_pos);
    assert!(opt_pos < leaf_pos);
}

#[test]
fn wrap_nesting_order_box_vec_leaf() {
    let ty: Type = parse_quote!(Box<Vec<u32>>);
    let skip: HashSet<&str> = HashSet::from(["Box", "Vec"]);
    let result = wrap_leaf_type(&ty, &skip);
    let s = ty_str(&result);
    let box_pos = s.find("Box").expect("should contain Box");
    let vec_pos = s.find("Vec").expect("should contain Vec");
    let leaf_pos = s.find("WithLeaf").expect("should contain WithLeaf");
    assert!(box_pos < vec_pos);
    assert!(vec_pos < leaf_pos);
}

#[test]
fn wrap_nesting_order_leaf_wraps_innermost() {
    let ty: Type = parse_quote!(Vec<Option<Box<char>>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option", "Box"]);
    let result = wrap_leaf_type(&ty, &skip);
    let s = ty_str(&result);
    // WithLeaf should appear right before char.
    assert!(s.contains("WithLeaf < char >"));
}

#[test]
fn wrap_nesting_order_no_skip_wraps_outermost() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    let s = ty_str(&result);
    assert!(s.starts_with("adze :: WithLeaf < Vec"));
}

#[test]
fn wrap_nesting_order_partial_skip_correct_depth() {
    // Only Option is skipped; Vec<u8> inside becomes the leaf.
    let ty: Type = parse_quote!(Option<Vec<u8>>);
    let skip: HashSet<&str> = HashSet::from(["Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Option < adze :: WithLeaf < Vec < u8 > > >"
    );
}

#[test]
fn wrap_nesting_order_same_container_repeated() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "Vec < Vec < adze :: WithLeaf < i32 > > >"
    );
}

// ============================================================================
// Category 10: wrap_edge_* — Edge cases and special scenarios (8 tests)
// ============================================================================

#[test]
fn wrap_edge_result_two_type_params_skip() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let skip: HashSet<&str> = HashSet::from(["Result"]);
    let result = wrap_leaf_type(&ty, &skip);
    // Both type params should be wrapped.
    let s = ty_str(&result);
    assert!(s.contains("WithLeaf < i32 >"));
    assert!(s.contains("WithLeaf < String >"));
}

#[test]
fn wrap_edge_hashmap_two_type_params_skip() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let skip: HashSet<&str> = HashSet::from(["HashMap"]);
    let result = wrap_leaf_type(&ty, &skip);
    let s = ty_str(&result);
    assert!(s.contains("WithLeaf < String >"));
    assert!(s.contains("WithLeaf < i32 >"));
}

#[test]
fn wrap_edge_idempotent_with_skip_on_nongeneric() {
    // Wrapping a non-generic type always yields WithLeaf, regardless of skip.
    let ty: Type = parse_quote!(i32);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_edge_skip_set_irrelevant_entries() {
    // Skip set contains names not present in the type; has no effect.
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> = HashSet::from(["Option", "Box", "Rc"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn wrap_edge_large_skip_set() {
    let ty: Type = parse_quote!(Vec<i32>);
    let skip: HashSet<&str> =
        HashSet::from(["Vec", "Option", "Box", "Rc", "Arc", "Cell", "RefCell"]);
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(ty_str(&result), "Vec < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_edge_preserves_inner_type_exactly() {
    let ty: Type = parse_quote!(std::vec::Vec<std::string::String>);
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    let result = wrap_leaf_type(&ty, &skip);
    let s = ty_str(&result);
    assert!(s.contains("adze :: WithLeaf < std :: string :: String >"));
}

#[test]
fn wrap_edge_result_valid_type() {
    // Verify the result parses as a valid Type by round-tripping through tokens.
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip: HashSet<&str> = HashSet::from(["Vec", "Option"]);
    let result = wrap_leaf_type(&ty, &skip);
    let tokens = quote::quote!(#result);
    let reparsed: Type = syn::parse2(tokens).expect("wrapped type should be valid syn::Type");
    assert_eq!(ty_str(&result), ty_str(&reparsed));
}

#[test]
fn wrap_edge_vec_of_tuples_no_skip() {
    let ty: Type = parse_quote!(Vec<(i32, String)>);
    let skip = HashSet::new();
    let result = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        ty_str(&result),
        "adze :: WithLeaf < Vec < (i32 , String) > >"
    );
}

// ============================================================================
// Category 11: wrap_interplay_* — Interplay with filter_inner_type (6 tests)
// ============================================================================

#[test]
fn wrap_interplay_filter_box_then_wrap() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let skip_filter: HashSet<&str> = HashSet::from(["Box"]);
    let filtered = filter_inner_type(&ty, &skip_filter);
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
    let skip_wrap: HashSet<&str> = HashSet::from(["Vec"]);
    let wrapped = wrap_leaf_type(&filtered, &skip_wrap);
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < u8 > >");
}

#[test]
fn wrap_interplay_filter_double_box_then_wrap() {
    let ty: Type = parse_quote!(Box<Box<i32>>);
    let skip_filter: HashSet<&str> = HashSet::from(["Box"]);
    let filtered = filter_inner_type(&ty, &skip_filter);
    assert_eq!(ty_str(&filtered), "i32");
    let skip_wrap = HashSet::new();
    let wrapped = wrap_leaf_type(&filtered, &skip_wrap);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_interplay_extract_then_wrap() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip_extract = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip_extract);
    assert!(found);
    assert_eq!(ty_str(&inner), "Vec < String >");
    let skip_wrap: HashSet<&str> = HashSet::from(["Vec"]);
    let wrapped = wrap_leaf_type(&inner, &skip_wrap);
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_interplay_extract_skip_then_wrap() {
    let ty: Type = parse_quote!(Box<Vec<f64>>);
    let skip_extract: HashSet<&str> = HashSet::from(["Box"]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip_extract);
    assert!(found);
    assert_eq!(ty_str(&inner), "f64");
    let skip_wrap = HashSet::new();
    let wrapped = wrap_leaf_type(&inner, &skip_wrap);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn wrap_interplay_filter_preserves_wrapping() {
    let ty: Type = parse_quote!(Box<adze::WithLeaf<i32>>);
    let skip_filter: HashSet<&str> = HashSet::from(["Box"]);
    let filtered = filter_inner_type(&ty, &skip_filter);
    assert_eq!(ty_str(&filtered), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_interplay_wrap_all_primitives_in_vec() {
    // Verify wrapping works identically for different primitives inside Vec.
    let skip: HashSet<&str> = HashSet::from(["Vec"]);
    for prim in ["i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64"] {
        let ty: Type = syn::parse_str(&format!("Vec<{prim}>")).expect("valid type");
        let result = wrap_leaf_type(&ty, &skip);
        let s = ty_str(&result);
        assert!(
            s.contains(&format!("WithLeaf < {prim} >")),
            "expected WithLeaf wrapping for {prim}, got: {s}"
        );
        assert!(
            s.starts_with("Vec"),
            "expected Vec outer for {prim}, got: {s}"
        );
    }
}
