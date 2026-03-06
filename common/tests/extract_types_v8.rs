//! Comprehensive tests for type extraction and manipulation in adze-common.
//!
//! Covers `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! across a wide variety of type patterns, nesting depths, and edge cases.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use std::collections::HashSet;
use syn::{Type, parse_quote};

fn type_to_string(ty: &Type) -> String {
    quote::quote!(#ty).to_string()
}

fn empty_skip() -> HashSet<&'static str> {
    HashSet::new()
}

fn skip_with(names: &[&'static str]) -> HashSet<&'static str> {
    names.iter().copied().collect()
}

// ============================================================================
// try_extract_inner_type — Vec<T>
// ============================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "i32");
}

#[test]
fn extract_vec_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "u8");
}

#[test]
fn extract_vec_bool() {
    let ty: Type = parse_quote!(Vec<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "bool");
}

#[test]
fn extract_vec_f64() {
    let ty: Type = parse_quote!(Vec<f64>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "f64");
}

#[test]
fn extract_vec_custom_type() {
    let ty: Type = parse_quote!(Vec<MyStruct>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "MyStruct");
}

// ============================================================================
// try_extract_inner_type — Option<T>
// ============================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "i32");
}

#[test]
fn extract_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "bool");
}

#[test]
fn extract_option_vec_string() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "Vec < String >");
}

// ============================================================================
// try_extract_inner_type — Box<T>
// ============================================================================

#[test]
fn extract_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn extract_box_i32() {
    let ty: Type = parse_quote!(Box<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "i32");
}

#[test]
fn extract_box_vec_u8() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "Vec < u8 >");
}

// ============================================================================
// try_extract_inner_type — plain types (no wrapper, returns false)
// ============================================================================

#[test]
fn extract_plain_i32_returns_false() {
    let ty: Type = parse_quote!(i32);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "i32");
}

#[test]
fn extract_plain_string_returns_false() {
    let ty: Type = parse_quote!(String);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn extract_plain_bool_returns_false() {
    let ty: Type = parse_quote!(bool);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "bool");
}

#[test]
fn extract_plain_usize_returns_false() {
    let ty: Type = parse_quote!(usize);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "usize");
}

#[test]
fn extract_plain_unit_returns_false() {
    let ty: Type = parse_quote!(());
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    // Unit type is not a Path, so it comes back unchanged
    assert_eq!(type_to_string(&inner), "()");
}

// ============================================================================
// try_extract_inner_type — nested types (Vec<Vec<T>>, etc.)
// ============================================================================

#[test]
fn extract_vec_of_vec_string() {
    let ty: Type = parse_quote!(Vec<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "Vec < String >");
}

#[test]
fn extract_vec_of_option_i32() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "Option < i32 >");
}

#[test]
fn extract_option_of_option_string() {
    let ty: Type = parse_quote!(Option<Option<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "Option < String >");
}

#[test]
fn extract_vec_of_vec_of_vec() {
    let ty: Type = parse_quote!(Vec<Vec<Vec<u32>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "Vec < Vec < u32 > >");
}

// ============================================================================
// try_extract_inner_type — with skip_over
// ============================================================================

#[test]
fn extract_vec_through_box_skip() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let skip = skip_with(&["Box"]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn extract_option_through_arc_skip() {
    let ty: Type = parse_quote!(Arc<Option<i32>>);
    let skip = skip_with(&["Arc"]);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(found);
    assert_eq!(type_to_string(&inner), "i32");
}

#[test]
fn extract_vec_through_box_and_arc_skip() {
    let ty: Type = parse_quote!(Box<Arc<Vec<String>>>);
    let skip = skip_with(&["Box", "Arc"]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn extract_skip_container_no_target_inside() {
    let ty: Type = parse_quote!(Box<String>);
    let skip = skip_with(&["Box"]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(!found);
    assert_eq!(type_to_string(&inner), "Box < String >");
}

#[test]
fn extract_skip_not_matching_outermost() {
    let ty: Type = parse_quote!(Rc<Vec<String>>);
    let skip = skip_with(&["Box"]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    // Rc is not in skip and is not Vec, so no extraction
    assert!(!found);
    assert_eq!(type_to_string(&inner), "Rc < Vec < String > >");
}

#[test]
fn extract_skip_empty_set_does_not_skip() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    // Box is not skipped, and it's not the target "Vec"
    assert!(!found);
    assert_eq!(type_to_string(&inner), "Box < Vec < String > >");
}

// ============================================================================
// try_extract_inner_type — multiple wrappers (Option<Vec<T>>)
// ============================================================================

#[test]
fn extract_option_of_vec_string_target_option() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "Vec < String >");
}

#[test]
fn extract_option_of_vec_string_target_vec_no_skip() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    // Option is outermost, not "Vec", and Option is not in skip set
    assert!(!found);
    assert_eq!(type_to_string(&inner), "Option < Vec < String > >");
}

#[test]
fn extract_option_of_vec_string_target_vec_with_option_skip() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip = skip_with(&["Option"]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

// ============================================================================
// try_extract_inner_type — reference types and non-path types
// ============================================================================

#[test]
fn extract_reference_type_returns_false() {
    let ty: Type = parse_quote!(&str);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "& str");
}

#[test]
fn extract_mutable_reference_returns_false() {
    let ty: Type = parse_quote!(&mut String);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "& mut String");
}

#[test]
fn extract_tuple_type_returns_false() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "(i32 , u32)");
}

#[test]
fn extract_array_type_returns_false() {
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "[u8 ; 4]");
}

#[test]
fn extract_slice_reference_returns_false() {
    let ty: Type = parse_quote!(&[u8]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "& [u8]");
}

// ============================================================================
// try_extract_inner_type — path types (std::string::String)
// ============================================================================

#[test]
fn extract_std_path_no_match() {
    let ty: Type = parse_quote!(std::string::String);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "std :: string :: String");
}

#[test]
fn extract_std_vec_by_last_segment() {
    // The function checks last segment, so std::vec::Vec<i32> should match "Vec"
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "i32");
}

// ============================================================================
// try_extract_inner_type — various generic containers
// ============================================================================

#[test]
fn extract_hashset_string() {
    let ty: Type = parse_quote!(HashSet<String>);
    let (inner, found) = try_extract_inner_type(&ty, "HashSet", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn extract_rc_i32() {
    let ty: Type = parse_quote!(Rc<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Rc", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "i32");
}

#[test]
fn extract_arc_string() {
    let ty: Type = parse_quote!(Arc<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Arc", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn extract_cell_u64() {
    let ty: Type = parse_quote!(Cell<u64>);
    let (inner, found) = try_extract_inner_type(&ty, "Cell", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "u64");
}

#[test]
fn extract_refcell_bool() {
    let ty: Type = parse_quote!(RefCell<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "RefCell", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "bool");
}

// ============================================================================
// try_extract_inner_type — multi-arg generics (Result<T, E>)
// ============================================================================

#[test]
fn extract_result_extracts_first_arg() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, found) = try_extract_inner_type(&ty, "Result", &empty_skip());
    assert!(found);
    // Extracts first generic argument only
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn extract_hashmap_extracts_first_arg() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, found) = try_extract_inner_type(&ty, "HashMap", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

// ============================================================================
// filter_inner_type — basic usage
// ============================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_to_string(&filtered), "String");
}

#[test]
fn filter_arc_i32() {
    let ty: Type = parse_quote!(Arc<i32>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Arc"]));
    assert_eq!(type_to_string(&filtered), "i32");
}

#[test]
fn filter_rc_bool() {
    let ty: Type = parse_quote!(Rc<bool>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Rc"]));
    assert_eq!(type_to_string(&filtered), "bool");
}

#[test]
fn filter_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box", "Arc"]));
    assert_eq!(type_to_string(&filtered), "String");
}

#[test]
fn filter_triple_nested() {
    let ty: Type = parse_quote!(Box<Arc<Rc<u32>>>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box", "Arc", "Rc"]));
    assert_eq!(type_to_string(&filtered), "u32");
}

// ============================================================================
// filter_inner_type — empty skip set
// ============================================================================

#[test]
fn filter_empty_skip_leaves_box_unchanged() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&filtered), "Box < String >");
}

#[test]
fn filter_empty_skip_leaves_vec_unchanged() {
    let ty: Type = parse_quote!(Vec<i32>);
    let filtered = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&filtered), "Vec < i32 >");
}

#[test]
fn filter_empty_skip_leaves_plain_unchanged() {
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&filtered), "String");
}

// ============================================================================
// filter_inner_type — populated skip set (partial match)
// ============================================================================

#[test]
fn filter_skip_box_but_not_arc() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    // Box is stripped, Arc remains because it's not in skip
    assert_eq!(type_to_string(&filtered), "Arc < String >");
}

#[test]
fn filter_skip_arc_but_not_box() {
    let ty: Type = parse_quote!(Arc<String>);
    let skip = skip_with(&["Box"]);
    let filtered = filter_inner_type(&ty, &skip);
    // Arc is not in skip, remains
    assert_eq!(type_to_string(&filtered), "Arc < String >");
}

#[test]
fn filter_non_matching_type_unchanged() {
    let ty: Type = parse_quote!(Vec<String>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box", "Arc"]));
    assert_eq!(type_to_string(&filtered), "Vec < String >");
}

// ============================================================================
// filter_inner_type — non-path types
// ============================================================================

#[test]
fn filter_reference_returns_unchanged() {
    let ty: Type = parse_quote!(&str);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_to_string(&filtered), "& str");
}

#[test]
fn filter_tuple_returns_unchanged() {
    let ty: Type = parse_quote!((i32, u32));
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_to_string(&filtered), "(i32 , u32)");
}

#[test]
fn filter_array_returns_unchanged() {
    let ty: Type = parse_quote!([u8; 16]);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_to_string(&filtered), "[u8 ; 16]");
}

// ============================================================================
// wrap_leaf_type — primitive types
// ============================================================================

#[test]
fn wrap_i32() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_string() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_bool() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_u8() {
    let ty: Type = parse_quote!(u8);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < u8 >");
}

#[test]
fn wrap_f64() {
    let ty: Type = parse_quote!(f64);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn wrap_usize() {
    let ty: Type = parse_quote!(usize);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < usize >");
}

// ============================================================================
// wrap_leaf_type — complex types
// ============================================================================

#[test]
fn wrap_custom_struct() {
    let ty: Type = parse_quote!(MyStruct);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < MyStruct >");
}

#[test]
fn wrap_path_type() {
    let ty: Type = parse_quote!(std::string::String);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(
        type_to_string(&wrapped),
        "adze :: WithLeaf < std :: string :: String >"
    );
}

#[test]
fn wrap_vec_in_skip_set() {
    let ty: Type = parse_quote!(Vec<String>);
    let skip = skip_with(&["Vec"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_option_in_skip_set() {
    let ty: Type = parse_quote!(Option<i32>);
    let skip = skip_with(&["Option"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "Option < adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_vec_not_in_skip_wraps_whole_thing() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(
        type_to_string(&wrapped),
        "adze :: WithLeaf < Vec < String > >"
    );
}

#[test]
fn wrap_nested_vec_option_both_in_skip() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let skip = skip_with(&["Vec", "Option"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "Vec < Option < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_option_vec_both_in_skip() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    let skip = skip_with(&["Vec", "Option"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "Option < Vec < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_result_in_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let skip = skip_with(&["Result"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

// ============================================================================
// wrap_leaf_type — non-path types
// ============================================================================

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_mutable_reference() {
    let ty: Type = parse_quote!(&mut i32);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < & mut i32 >");
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, u32));
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < (i32 , u32) >");
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < () >");
}

#[test]
fn wrap_slice_reference() {
    let ty: Type = parse_quote!(&[u8]);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < & [u8] >");
}

// ============================================================================
// Type string round-trip fidelity
// ============================================================================

#[test]
fn roundtrip_plain_type_through_extract() {
    let ty: Type = parse_quote!(i32);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&result), type_to_string(&ty));
}

#[test]
fn roundtrip_non_matching_generic_through_extract() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&result), type_to_string(&ty));
}

#[test]
fn roundtrip_filter_non_skip_type() {
    let ty: Type = parse_quote!(Vec<String>);
    let result = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_to_string(&result), type_to_string(&ty));
}

#[test]
fn roundtrip_filter_plain_type() {
    let ty: Type = parse_quote!(String);
    let result = filter_inner_type(&ty, &skip_with(&["Box", "Arc"]));
    assert_eq!(type_to_string(&result), type_to_string(&ty));
}

#[test]
fn roundtrip_reference_through_filter() {
    let ty: Type = parse_quote!(&str);
    let result = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_to_string(&result), type_to_string(&ty));
}

// ============================================================================
// Deeply nested generics
// ============================================================================

#[test]
fn extract_deeply_nested_skip_chain() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Vec<String>>>>);
    let skip = skip_with(&["Box", "Arc", "Rc"]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn filter_deeply_nested_all_skippable() {
    let ty: Type = parse_quote!(Box<Arc<Rc<i32>>>);
    let skip = skip_with(&["Box", "Arc", "Rc"]);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(type_to_string(&filtered), "i32");
}

#[test]
fn wrap_deeply_nested_all_in_skip() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let skip = skip_with(&["Vec", "Option", "Box"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "Vec < Option < Box < adze :: WithLeaf < i32 > > > >"
    );
}

// ============================================================================
// Edge cases: single generic arg containers
// ============================================================================

#[test]
fn extract_single_element_vec() {
    let ty: Type = parse_quote!(Vec<()>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "()");
}

#[test]
fn filter_single_layer_box() {
    let ty: Type = parse_quote!(Box<()>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_to_string(&filtered), "()");
}

#[test]
fn wrap_single_arg_in_skip() {
    let ty: Type = parse_quote!(Option<()>);
    let skip = skip_with(&["Option"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "Option < adze :: WithLeaf < () > >"
    );
}

// ============================================================================
// Edge cases: types not in any generic
// ============================================================================

#[test]
fn extract_no_generics_custom_type() {
    let ty: Type = parse_quote!(Foo);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "Foo");
}

#[test]
fn filter_no_generics_custom_type() {
    let ty: Type = parse_quote!(Foo);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    assert_eq!(type_to_string(&filtered), "Foo");
}

#[test]
fn wrap_no_generics_custom_type() {
    let ty: Type = parse_quote!(Foo);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < Foo >");
}

// ============================================================================
// Interactions between functions
// ============================================================================

#[test]
fn extract_then_wrap_leaf() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    let wrapped = wrap_leaf_type(&inner, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_wrap_leaf() {
    let ty: Type = parse_quote!(Box<i32>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    let wrapped = wrap_leaf_type(&filtered, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn extract_then_filter() {
    let ty: Type = parse_quote!(Vec<Box<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    let filtered = filter_inner_type(&inner, &skip_with(&["Box"]));
    assert_eq!(type_to_string(&filtered), "String");
}

#[test]
fn filter_then_extract() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let filtered = filter_inner_type(&ty, &skip_with(&["Box"]));
    let (inner, found) = try_extract_inner_type(&filtered, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

// ============================================================================
// Additional edge cases
// ============================================================================

#[test]
fn extract_wrong_target_name_returns_false() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(type_to_string(&inner), "Vec < String >");
}

#[test]
fn extract_with_inner_of_matching_inner_type_name() {
    // inner_of="String" but String has no angle brackets — would panic
    // Instead we test a non-generic type used as target
    let ty: Type = parse_quote!(Wrapper<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Wrapper", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}

#[test]
fn filter_multiple_layers_partial_skip() {
    let ty: Type = parse_quote!(Box<Rc<Arc<String>>>);
    let skip = skip_with(&["Box", "Arc"]);
    let filtered = filter_inner_type(&ty, &skip);
    // Box stripped -> Rc not in skip -> stops at Rc
    assert_eq!(type_to_string(&filtered), "Rc < Arc < String > >");
}

#[test]
fn wrap_hashmap_in_skip_wraps_all_type_args() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let skip = skip_with(&["HashMap"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "HashMap < adze :: WithLeaf < String > , adze :: WithLeaf < Vec < i32 > > >"
    );
}

#[test]
fn wrap_hashmap_and_vec_in_skip() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let skip = skip_with(&["HashMap", "Vec"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "HashMap < adze :: WithLeaf < String > , Vec < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn extract_idempotent_on_non_matching() {
    let ty: Type = parse_quote!(i32);
    let (r1, f1) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    let (r2, f2) = try_extract_inner_type(&r1, "Vec", &empty_skip());
    assert!(!f1);
    assert!(!f2);
    assert_eq!(type_to_string(&r1), type_to_string(&r2));
}

#[test]
fn filter_idempotent_on_plain_type() {
    let ty: Type = parse_quote!(String);
    let skip = skip_with(&["Box"]);
    let r1 = filter_inner_type(&ty, &skip);
    let r2 = filter_inner_type(&r1, &skip);
    assert_eq!(type_to_string(&r1), type_to_string(&r2));
}

#[test]
fn wrap_vec_option_result_all_skip() {
    let ty: Type = parse_quote!(Vec<Option<Result<i32, String>>>);
    let skip = skip_with(&["Vec", "Option", "Result"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "Vec < Option < Result < adze :: WithLeaf < i32 > , adze :: WithLeaf < String > > > >"
    );
}

#[test]
fn extract_through_multiple_same_skip_types() {
    let ty: Type = parse_quote!(Box<Box<Vec<i32>>>);
    let skip = skip_with(&["Box"]);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(found);
    assert_eq!(type_to_string(&inner), "i32");
}

#[test]
fn filter_through_multiple_same_skip_types() {
    let ty: Type = parse_quote!(Box<Box<Box<i32>>>);
    let skip = skip_with(&["Box"]);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(type_to_string(&filtered), "i32");
}

#[test]
fn wrap_generic_not_in_skip_wraps_entirely() {
    let ty: Type = parse_quote!(Rc<String>);
    let skip = skip_with(&["Vec", "Option"]);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        type_to_string(&wrapped),
        "adze :: WithLeaf < Rc < String > >"
    );
}

#[test]
fn extract_option_u128() {
    let ty: Type = parse_quote!(Option<u128>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "u128");
}

#[test]
fn extract_vec_char() {
    let ty: Type = parse_quote!(Vec<char>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "char");
}

#[test]
fn wrap_never_type_as_leaf() {
    // `!` is the never type — parsed as a Type
    let ty: Type = parse_quote!(MyNever);
    let wrapped = wrap_leaf_type(&ty, &empty_skip());
    assert_eq!(type_to_string(&wrapped), "adze :: WithLeaf < MyNever >");
}

#[test]
fn extract_cow_string() {
    let ty: Type = parse_quote!(Cow<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Cow", &empty_skip());
    assert!(found);
    assert_eq!(type_to_string(&inner), "String");
}
