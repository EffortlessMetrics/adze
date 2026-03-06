//! Comprehensive tests for type parsing and manipulation using `syn::parse_quote`
//! in adze-common.
//!
//! Tests cover `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! across simple types, generics, references, tuples, arrays, qualified paths,
//! and deeply nested containers.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use std::collections::HashSet;
use syn::{Type, parse_quote};

fn ty_str(ty: &Type) -> String {
    quote::quote!(#ty).to_string()
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

fn skip(names: &[&'static str]) -> HashSet<&'static str> {
    names.iter().copied().collect()
}

// ============================================================================
// Section 1: parse_quote type construction verification
// ============================================================================

#[test]
fn pq_simple_i32() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(ty_str(&ty), "i32");
}

#[test]
fn pq_simple_bool() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(ty_str(&ty), "bool");
}

#[test]
fn pq_simple_u64() {
    let ty: Type = parse_quote!(u64);
    assert_eq!(ty_str(&ty), "u64");
}

#[test]
fn pq_generic_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(ty_str(&ty), "Vec < i32 >");
}

#[test]
fn pq_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(ty_str(&ty), "Option < String >");
}

#[test]
fn pq_nested_box_vec_i32() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    assert_eq!(ty_str(&ty), "Box < Vec < i32 > >");
}

#[test]
fn pq_reference_str() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(ty_str(&ty), "& str");
}

#[test]
fn pq_lifetime_reference() {
    let ty: Type = parse_quote!(&'a str);
    assert_eq!(ty_str(&ty), "& 'a str");
}

#[test]
fn pq_unit_type() {
    let ty: Type = parse_quote!(());
    assert_eq!(ty_str(&ty), "()");
}

#[test]
fn pq_tuple_type() {
    let ty: Type = parse_quote!((i32, String));
    assert_eq!(ty_str(&ty), "(i32 , String)");
}

#[test]
fn pq_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    assert_eq!(ty_str(&ty), "[u8 ; 4]");
}

#[test]
fn pq_fully_qualified_vec() {
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    assert_eq!(ty_str(&ty), "std :: vec :: Vec < i32 >");
}

#[test]
fn pq_multi_param_generic() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(ty_str(&ty), "HashMap < String , i32 >");
}

#[test]
fn pq_triple_nested() {
    let ty: Type = parse_quote!(Arc<Box<Option<u8>>>);
    assert_eq!(ty_str(&ty), "Arc < Box < Option < u8 > > >");
}

#[test]
fn pq_fn_pointer() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    assert_eq!(ty_str(&ty), "fn (i32) -> bool");
}

#[test]
fn pq_slice_type() {
    let ty: Type = parse_quote!([u8]);
    // Slice type
    assert_eq!(ty_str(&ty), "[u8]");
}

#[test]
fn pq_nested_tuple() {
    let ty: Type = parse_quote!((i32, (u8, bool)));
    assert_eq!(ty_str(&ty), "(i32 , (u8 , bool))");
}

// ============================================================================
// Section 2: try_extract_inner_type — basic extraction
// ============================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_box_u8() {
    let ty: Type = parse_quote!(Box<u8>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_target_not_found() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn extract_plain_type_not_found() {
    let ty: Type = parse_quote!(i32);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_vec_bool() {
    let ty: Type = parse_quote!(Vec<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_vec_f64() {
    let ty: Type = parse_quote!(Vec<f64>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "f64");
}

// ============================================================================
// Section 3: try_extract_inner_type — skip-over chaining
// ============================================================================

#[test]
fn extract_through_box_to_vec() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_through_arc_box_to_option() {
    let ty: Type = parse_quote!(Arc<Box<Option<u8>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_skip_but_target_missing() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!found);
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_skip_two_levels_to_vec() {
    let ty: Type = parse_quote!(Arc<Box<Vec<i32>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc", "Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_skip_single_level_to_option() {
    let ty: Type = parse_quote!(Box<Option<f32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn extract_skip_three_levels() {
    let ty: Type = parse_quote!(Rc<Arc<Box<Vec<u16>>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Rc", "Arc", "Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "u16");
}

// ============================================================================
// Section 4: try_extract_inner_type — non-path types
// ============================================================================

#[test]
fn extract_reference_type_unchanged() {
    let ty: Type = parse_quote!(&str);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn extract_lifetime_ref_unchanged() {
    let ty: Type = parse_quote!(&'a str);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&inner), "& 'a str");
}

#[test]
fn extract_tuple_type_unchanged() {
    let ty: Type = parse_quote!((i32, String));
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&inner), "(i32 , String)");
}

#[test]
fn extract_array_type_unchanged() {
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&inner), "[u8 ; 4]");
}

#[test]
fn extract_unit_type_unchanged() {
    let ty: Type = parse_quote!(());
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&inner), "()");
}

#[test]
fn extract_fn_pointer_unchanged() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&inner), "fn (i32) -> bool");
}

#[test]
fn extract_slice_unchanged() {
    let ty: Type = parse_quote!([u8]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&inner), "[u8]");
}

// ============================================================================
// Section 5: try_extract_inner_type — qualified paths
// ============================================================================

#[test]
fn extract_qualified_vec() {
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_qualified_option() {
    let ty: Type = parse_quote!(std::option::Option<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_qualified_box_skip() {
    let ty: Type = parse_quote!(std::boxed::Box<Vec<u32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "u32");
}

// ============================================================================
// Section 6: try_extract_inner_type — nested generics
// ============================================================================

#[test]
fn extract_vec_of_vec() {
    let ty: Type = parse_quote!(Vec<Vec<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn extract_option_of_vec() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn extract_vec_of_option() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn extract_vec_of_tuple() {
    let ty: Type = parse_quote!(Vec<(i32, String)>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "(i32 , String)");
}

// ============================================================================
// Section 7: filter_inner_type — single container unwrap
// ============================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_arc_i32() {
    let ty: Type = parse_quote!(Arc<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_rc_bool() {
    let ty: Type = parse_quote!(Rc<bool>);
    let filtered = filter_inner_type(&ty, &skip(&["Rc"]));
    assert_eq!(ty_str(&filtered), "bool");
}

// ============================================================================
// Section 8: filter_inner_type — multi-level unwrap
// ============================================================================

#[test]
fn filter_box_arc_string() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_three_levels() {
    let ty: Type = parse_quote!(Rc<Arc<Box<u64>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Rc", "Arc", "Box"]));
    assert_eq!(ty_str(&filtered), "u64");
}

#[test]
fn filter_four_levels() {
    let ty: Type = parse_quote!(Cow<Rc<Arc<Box<f32>>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Cow", "Rc", "Arc", "Box"]));
    assert_eq!(ty_str(&filtered), "f32");
}

// ============================================================================
// Section 9: filter_inner_type — no-op cases
// ============================================================================

#[test]
fn filter_empty_skip_noop() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &empty_skip());
    assert_eq!(ty_str(&filtered), "Box < String >");
}

#[test]
fn filter_plain_type_noop() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn filter_non_matching_container_noop() {
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
}

#[test]
fn filter_reference_type_noop() {
    let ty: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "& str");
}

#[test]
fn filter_tuple_noop() {
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(i32 , u32)");
}

#[test]
fn filter_array_noop() {
    let ty: Type = parse_quote!([u8; 16]);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "[u8 ; 16]");
}

#[test]
fn filter_unit_noop() {
    let ty: Type = parse_quote!(());
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "()");
}

// ============================================================================
// Section 10: filter_inner_type — stops at non-skip container
// ============================================================================

#[test]
fn filter_stops_at_vec() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
}

#[test]
fn filter_stops_at_option() {
    let ty: Type = parse_quote!(Arc<Option<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&filtered), "Option < String >");
}

#[test]
fn filter_skips_partial_chain() {
    let ty: Type = parse_quote!(Box<Arc<Vec<f64>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "Vec < f64 >");
}

// ============================================================================
// Section 11: wrap_leaf_type — primitives
// ============================================================================

#[test]
fn wrap_i32() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_string() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_bool() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_f64() {
    let ty: Type = parse_quote!(f64);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn wrap_u8() {
    let ty: Type = parse_quote!(u8);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u8 >");
}

// ============================================================================
// Section 12: wrap_leaf_type — containers in skip set
// ============================================================================

#[test]
fn wrap_vec_string_skips_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_i32_skips_option() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_option_vec_skips_both() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wrap_vec_not_in_skip_wraps_entire() {
    let ty: Type = parse_quote!(Vec<i32>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn wrap_box_skips_box() {
    let ty: Type = parse_quote!(Box<u32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&wrapped), "Box < adze :: WithLeaf < u32 > >");
}

// ============================================================================
// Section 13: wrap_leaf_type — multi-param generics in skip set
// ============================================================================

#[test]
fn wrap_result_in_skip_wraps_both_params() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_hashmap_in_skip_wraps_both_params() {
    let ty: Type = parse_quote!(HashMap<String, Vec<u8>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["HashMap"]));
    assert_eq!(
        ty_str(&wrapped),
        "HashMap < adze :: WithLeaf < String > , adze :: WithLeaf < Vec < u8 > > >"
    );
}

#[test]
fn wrap_hashmap_and_vec_in_skip() {
    let ty: Type = parse_quote!(HashMap<String, Vec<u8>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["HashMap", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "HashMap < adze :: WithLeaf < String > , Vec < adze :: WithLeaf < u8 > > >"
    );
}

// ============================================================================
// Section 14: wrap_leaf_type — non-path types
// ============================================================================

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_lifetime_ref() {
    let ty: Type = parse_quote!(&'a str);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & 'a str >");
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, String));
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (i32 , String) >");
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < () >");
}

#[test]
fn wrap_fn_pointer() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < fn (i32) -> bool >");
}

#[test]
fn wrap_slice_type() {
    let ty: Type = parse_quote!([u8]);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8] >");
}

// ============================================================================
// Section 15: wrap_leaf_type — deeply nested containers
// ============================================================================

#[test]
fn wrap_option_option_i32_skip_option() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_vec_vec_bool_skip_vec() {
    let ty: Type = parse_quote!(Vec<Vec<bool>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Vec < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wrap_three_nested_skip_all() {
    let ty: Type = parse_quote!(Vec<Option<Box<u16>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option", "Box"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < Box < adze :: WithLeaf < u16 > > > >"
    );
}

// ============================================================================
// Section 16: combined operations — extract then wrap
// ============================================================================

#[test]
fn extract_then_wrap_vec() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    let wrapped = wrap_leaf_type(&inner, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn extract_through_box_then_wrap() {
    let ty: Type = parse_quote!(Box<Option<u32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(found);
    let wrapped = wrap_leaf_type(&inner, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u32 >");
}

#[test]
fn filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "String");
    let wrapped = wrap_leaf_type(&filtered, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_extract() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < i32 >");
    let (inner, found) = try_extract_inner_type(&filtered, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

// ============================================================================
// Section 17: extract — multi-segment paths
// ============================================================================

#[test]
fn extract_alloc_vec() {
    let ty: Type = parse_quote!(alloc::vec::Vec<f32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn extract_core_option() {
    let ty: Type = parse_quote!(core::option::Option<u64>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn filter_qualified_box() {
    let ty: Type = parse_quote!(std::boxed::Box<i64>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "i64");
}

// ============================================================================
// Section 18: extract — complex inner types
// ============================================================================

#[test]
fn extract_vec_of_array() {
    let ty: Type = parse_quote!(Vec<[u8; 32]>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "[u8 ; 32]");
}

#[test]
fn extract_vec_of_reference() {
    let ty: Type = parse_quote!(Vec<&str>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn extract_option_of_tuple() {
    let ty: Type = parse_quote!(Option<(i32, bool)>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "(i32 , bool)");
}

#[test]
fn extract_option_of_hashmap() {
    let ty: Type = parse_quote!(Option<HashMap<String, i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

// ============================================================================
// Section 19: extract — same type in inner_of and skip_over
// ============================================================================

#[test]
fn extract_box_of_box_target_box_skip_box() {
    // When target equals skip, the outermost match wins since try_extract
    // checks inner_of before skip_over.
    let ty: Type = parse_quote!(Box<Box<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip(&["Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Box < i32 >");
}

// ============================================================================
// Section 20: filter — idempotence
// ============================================================================

#[test]
fn filter_idempotent_already_unwrapped() {
    let ty: Type = parse_quote!(String);
    let s = &skip(&["Box"]);
    let first = filter_inner_type(&ty, s);
    let second = filter_inner_type(&first, s);
    assert_eq!(ty_str(&first), ty_str(&second));
}

#[test]
fn filter_idempotent_after_unwrap() {
    let ty: Type = parse_quote!(Box<i32>);
    let s = &skip(&["Box"]);
    let first = filter_inner_type(&ty, s);
    let second = filter_inner_type(&first, s);
    assert_eq!(ty_str(&first), "i32");
    assert_eq!(ty_str(&first), ty_str(&second));
}

// ============================================================================
// Section 21: wrap — double wrapping (non-idempotent)
// ============================================================================

#[test]
fn wrap_double_wrapping_primitive() {
    let ty: Type = parse_quote!(i32);
    let once = wrap_leaf_type(&ty, &empty_skip());
    let twice = wrap_leaf_type(&once, &empty_skip());
    assert_eq!(
        ty_str(&twice),
        "adze :: WithLeaf < adze :: WithLeaf < i32 > >"
    );
}

// ============================================================================
// Section 22: type string comparison accuracy
// ============================================================================

#[test]
fn string_comparison_preserves_spacing_simple() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(ty_str(&ty), "i32");
}

#[test]
fn string_comparison_preserves_spacing_generic() {
    let ty: Type = parse_quote!(Vec<i32>);
    // quote adds spaces around angle brackets
    assert!(ty_str(&ty).contains("Vec"));
    assert!(ty_str(&ty).contains("i32"));
}

#[test]
fn string_comparison_qualified_path() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, Vec<u8>>);
    let s = ty_str(&ty);
    assert!(s.contains("std"));
    assert!(s.contains("collections"));
    assert!(s.contains("HashMap"));
    assert!(s.contains("String"));
    assert!(s.contains("Vec"));
}

// ============================================================================
// Section 23: wrap — qualified path types
// ============================================================================

#[test]
fn wrap_qualified_path_not_in_skip() {
    let ty: Type = parse_quote!(std::string::String);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < std :: string :: String >"
    );
}

#[test]
fn wrap_qualified_vec_in_skip() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "std :: vec :: Vec < adze :: WithLeaf < u8 > >"
    );
}

// ============================================================================
// Section 24: miscellaneous edge cases
// ============================================================================

#[test]
fn extract_with_large_skip_set() {
    let ty: Type = parse_quote!(Arc<Vec<u8>>);
    let big_skip = skip(&["Box", "Arc", "Rc", "Cow", "Cell", "RefCell", "Mutex"]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &big_skip);
    assert!(found);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn filter_with_large_skip_set() {
    let ty: Type = parse_quote!(Mutex<RefCell<Arc<Box<u8>>>>);
    let big_skip = skip(&["Mutex", "RefCell", "Arc", "Box"]);
    let filtered = filter_inner_type(&ty, &big_skip);
    assert_eq!(ty_str(&filtered), "u8");
}

#[test]
fn wrap_nested_tuple_in_vec() {
    let ty: Type = parse_quote!(Vec<(i32, (u8, bool))>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < adze :: WithLeaf < (i32 , (u8 , bool)) > >"
    );
}

#[test]
fn extract_option_string_not_vec() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&inner), "Option < String >");
}

#[test]
fn wrap_result_not_in_skip() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < Result < i32 , String > >"
    );
}

#[test]
fn extract_custom_container() {
    let ty: Type = parse_quote!(MyVec<f64>);
    let (inner, found) = try_extract_inner_type(&ty, "MyVec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn filter_custom_container() {
    let ty: Type = parse_quote!(MyBox<String>);
    let filtered = filter_inner_type(&ty, &skip(&["MyBox"]));
    assert_eq!(ty_str(&filtered), "String");
}

#[test]
fn wrap_custom_container_in_skip() {
    let ty: Type = parse_quote!(MyVec<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["MyVec"]));
    assert_eq!(ty_str(&wrapped), "MyVec < adze :: WithLeaf < i32 > >");
}

#[test]
fn extract_vec_usize() {
    let ty: Type = parse_quote!(Vec<usize>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "usize");
}

#[test]
fn wrap_mutable_reference() {
    let ty: Type = parse_quote!(&mut i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & mut i32 >");
}

#[test]
fn extract_ref_mut_unchanged() {
    let ty: Type = parse_quote!(&mut Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&inner), "& mut Vec < i32 >");
}

#[test]
fn filter_fn_pointer_noop() {
    let ty: Type = parse_quote!(fn(u8) -> u8);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "fn (u8) -> u8");
}

#[test]
fn wrap_nested_array() {
    let ty: Type = parse_quote!([[u8; 4]; 8]);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [[u8 ; 4] ; 8] >");
}

#[test]
fn extract_vec_of_fn_pointer() {
    let ty: Type = parse_quote!(Vec<fn(i32) -> bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "fn (i32) -> bool");
}
