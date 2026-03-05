//! Comprehensive tests for type-to-string conversion and comparison in adze-common.
//!
//! Uses `quote::quote!(#ty).to_string()` as the canonical way to produce
//! deterministic string representations of `syn::Type` values, then exercises
//! `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type` through
//! that lens.

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use std::collections::HashSet;
use syn::{parse_quote, Type};

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
// Section 1: Primitive type strings (tests 1–12)
// ============================================================================

#[test]
fn str_primitive_i32() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(ty_str(&ty), "i32");
}

#[test]
fn str_primitive_u32() {
    let ty: Type = parse_quote!(u32);
    assert_eq!(ty_str(&ty), "u32");
}

#[test]
fn str_primitive_string() {
    let ty: Type = parse_quote!(String);
    assert_eq!(ty_str(&ty), "String");
}

#[test]
fn str_primitive_bool() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(ty_str(&ty), "bool");
}

#[test]
fn str_primitive_u8() {
    let ty: Type = parse_quote!(u8);
    assert_eq!(ty_str(&ty), "u8");
}

#[test]
fn str_primitive_u16() {
    let ty: Type = parse_quote!(u16);
    assert_eq!(ty_str(&ty), "u16");
}

#[test]
fn str_primitive_u64() {
    let ty: Type = parse_quote!(u64);
    assert_eq!(ty_str(&ty), "u64");
}

#[test]
fn str_primitive_i8() {
    let ty: Type = parse_quote!(i8);
    assert_eq!(ty_str(&ty), "i8");
}

#[test]
fn str_primitive_i16() {
    let ty: Type = parse_quote!(i16);
    assert_eq!(ty_str(&ty), "i16");
}

#[test]
fn str_primitive_i64() {
    let ty: Type = parse_quote!(i64);
    assert_eq!(ty_str(&ty), "i64");
}

#[test]
fn str_primitive_f32() {
    let ty: Type = parse_quote!(f32);
    assert_eq!(ty_str(&ty), "f32");
}

#[test]
fn str_primitive_f64() {
    let ty: Type = parse_quote!(f64);
    assert_eq!(ty_str(&ty), "f64");
}

// ============================================================================
// Section 2: Generic container type strings (tests 13–22)
// ============================================================================

#[test]
fn str_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    assert_eq!(ty_str(&ty), "Vec < i32 >");
}

#[test]
fn str_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(ty_str(&ty), "Option < String >");
}

#[test]
fn str_box_u8() {
    let ty: Type = parse_quote!(Box<u8>);
    assert_eq!(ty_str(&ty), "Box < u8 >");
}

#[test]
fn str_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(ty_str(&ty), "Vec < String >");
}

#[test]
fn str_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    assert_eq!(ty_str(&ty), "Option < i32 >");
}

#[test]
fn str_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(ty_str(&ty), "Box < String >");
}

#[test]
fn str_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    assert_eq!(ty_str(&ty), "Option < bool >");
}

#[test]
fn str_vec_u8() {
    let ty: Type = parse_quote!(Vec<u8>);
    assert_eq!(ty_str(&ty), "Vec < u8 >");
}

#[test]
fn str_box_vec_i32() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    assert_eq!(ty_str(&ty), "Box < Vec < i32 > >");
}

#[test]
fn str_arc_string() {
    let ty: Type = parse_quote!(Arc<String>);
    assert_eq!(ty_str(&ty), "Arc < String >");
}

// ============================================================================
// Section 3: Nested generic type strings (tests 23–30)
// ============================================================================

#[test]
fn str_vec_option_i32() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    assert_eq!(ty_str(&ty), "Vec < Option < i32 > >");
}

#[test]
fn str_option_vec_string() {
    let ty: Type = parse_quote!(Option<Vec<String>>);
    assert_eq!(ty_str(&ty), "Option < Vec < String > >");
}

#[test]
fn str_box_option_u8() {
    let ty: Type = parse_quote!(Box<Option<u8>>);
    assert_eq!(ty_str(&ty), "Box < Option < u8 > >");
}

#[test]
fn str_arc_box_option_u8() {
    let ty: Type = parse_quote!(Arc<Box<Option<u8>>>);
    assert_eq!(ty_str(&ty), "Arc < Box < Option < u8 > > >");
}

#[test]
fn str_option_option_i32() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    assert_eq!(ty_str(&ty), "Option < Option < i32 > >");
}

#[test]
fn str_vec_vec_u8() {
    let ty: Type = parse_quote!(Vec<Vec<u8>>);
    assert_eq!(ty_str(&ty), "Vec < Vec < u8 > >");
}

#[test]
fn str_result_string_i32() {
    let ty: Type = parse_quote!(Result<String, i32>);
    assert_eq!(ty_str(&ty), "Result < String , i32 >");
}

#[test]
fn str_hashmap_string_vec_i32() {
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    assert_eq!(ty_str(&ty), "HashMap < String , Vec < i32 > >");
}

// ============================================================================
// Section 4: Reference type strings (tests 31–36)
// ============================================================================

#[test]
fn str_ref_str() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(ty_str(&ty), "& str");
}

#[test]
fn str_ref_i32() {
    let ty: Type = parse_quote!(&i32);
    assert_eq!(ty_str(&ty), "& i32");
}

#[test]
fn str_ref_mut_i32() {
    let ty: Type = parse_quote!(&mut i32);
    assert_eq!(ty_str(&ty), "& mut i32");
}

#[test]
fn str_lifetime_ref() {
    let ty: Type = parse_quote!(&'a str);
    assert_eq!(ty_str(&ty), "& 'a str");
}

#[test]
fn str_ref_vec_i32() {
    let ty: Type = parse_quote!(&Vec<i32>);
    assert_eq!(ty_str(&ty), "& Vec < i32 >");
}

#[test]
fn str_ref_option_string() {
    let ty: Type = parse_quote!(&Option<String>);
    assert_eq!(ty_str(&ty), "& Option < String >");
}

// ============================================================================
// Section 5: Tuple type strings (tests 37–42)
// ============================================================================

#[test]
fn str_tuple_i32_string() {
    let ty: Type = parse_quote!((i32, String));
    assert_eq!(ty_str(&ty), "(i32 , String)");
}

#[test]
fn str_unit_type() {
    let ty: Type = parse_quote!(());
    assert_eq!(ty_str(&ty), "()");
}

#[test]
fn str_tuple_triple() {
    let ty: Type = parse_quote!((i32, bool, String));
    assert_eq!(ty_str(&ty), "(i32 , bool , String)");
}

#[test]
fn str_nested_tuple() {
    let ty: Type = parse_quote!((i32, (u8, bool)));
    assert_eq!(ty_str(&ty), "(i32 , (u8 , bool))");
}

#[test]
fn str_tuple_single() {
    let ty: Type = parse_quote!((i32,));
    assert_eq!(ty_str(&ty), "(i32 ,)");
}

#[test]
fn str_tuple_with_generic() {
    let ty: Type = parse_quote!((Vec<i32>, Option<String>));
    assert_eq!(ty_str(&ty), "(Vec < i32 > , Option < String >)");
}

// ============================================================================
// Section 6: Array and slice type strings (tests 43–48)
// ============================================================================

#[test]
fn str_array_u8_4() {
    let ty: Type = parse_quote!([u8; 4]);
    assert_eq!(ty_str(&ty), "[u8 ; 4]");
}

#[test]
fn str_array_i32_16() {
    let ty: Type = parse_quote!([i32; 16]);
    assert_eq!(ty_str(&ty), "[i32 ; 16]");
}

#[test]
fn str_slice_u8() {
    let ty: Type = parse_quote!([u8]);
    assert_eq!(ty_str(&ty), "[u8]");
}

#[test]
fn str_array_bool_1() {
    let ty: Type = parse_quote!([bool; 1]);
    assert_eq!(ty_str(&ty), "[bool ; 1]");
}

#[test]
fn str_ref_slice_u8() {
    let ty: Type = parse_quote!(&[u8]);
    assert_eq!(ty_str(&ty), "& [u8]");
}

#[test]
fn str_array_string_2() {
    let ty: Type = parse_quote!([String; 2]);
    assert_eq!(ty_str(&ty), "[String ; 2]");
}

// ============================================================================
// Section 7: Fully qualified path strings (tests 49–52)
// ============================================================================

#[test]
fn str_fully_qualified_vec() {
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    assert_eq!(ty_str(&ty), "std :: vec :: Vec < i32 >");
}

#[test]
fn str_fully_qualified_option() {
    let ty: Type = parse_quote!(std::option::Option<String>);
    assert_eq!(ty_str(&ty), "std :: option :: Option < String >");
}

#[test]
fn str_fully_qualified_hashmap() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    assert_eq!(ty_str(&ty), "std :: collections :: HashMap < String , i32 >");
}

#[test]
fn str_fn_pointer() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    assert_eq!(ty_str(&ty), "fn (i32) -> bool");
}

// ============================================================================
// Section 8: Type comparison — same type yields same string (tests 53–58)
// ============================================================================

#[test]
fn compare_same_i32() {
    let a: Type = parse_quote!(i32);
    let b: Type = parse_quote!(i32);
    assert_eq!(ty_str(&a), ty_str(&b));
}

#[test]
fn compare_same_vec_string() {
    let a: Type = parse_quote!(Vec<String>);
    let b: Type = parse_quote!(Vec<String>);
    assert_eq!(ty_str(&a), ty_str(&b));
}

#[test]
fn compare_same_option_i32() {
    let a: Type = parse_quote!(Option<i32>);
    let b: Type = parse_quote!(Option<i32>);
    assert_eq!(ty_str(&a), ty_str(&b));
}

#[test]
fn compare_same_tuple() {
    let a: Type = parse_quote!((i32, bool));
    let b: Type = parse_quote!((i32, bool));
    assert_eq!(ty_str(&a), ty_str(&b));
}

#[test]
fn compare_same_ref() {
    let a: Type = parse_quote!(&str);
    let b: Type = parse_quote!(&str);
    assert_eq!(ty_str(&a), ty_str(&b));
}

#[test]
fn compare_same_nested() {
    let a: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let b: Type = parse_quote!(Vec<Option<Box<i32>>>);
    assert_eq!(ty_str(&a), ty_str(&b));
}

// ============================================================================
// Section 9: Type comparison — different types yield different strings (tests 59–64)
// ============================================================================

#[test]
fn compare_diff_i32_u32() {
    let a: Type = parse_quote!(i32);
    let b: Type = parse_quote!(u32);
    assert_ne!(ty_str(&a), ty_str(&b));
}

#[test]
fn compare_diff_vec_option() {
    let a: Type = parse_quote!(Vec<i32>);
    let b: Type = parse_quote!(Option<i32>);
    assert_ne!(ty_str(&a), ty_str(&b));
}

#[test]
fn compare_diff_string_bool() {
    let a: Type = parse_quote!(String);
    let b: Type = parse_quote!(bool);
    assert_ne!(ty_str(&a), ty_str(&b));
}

#[test]
fn compare_diff_inner_type() {
    let a: Type = parse_quote!(Vec<i32>);
    let b: Type = parse_quote!(Vec<u32>);
    assert_ne!(ty_str(&a), ty_str(&b));
}

#[test]
fn compare_diff_ref_vs_owned() {
    let a: Type = parse_quote!(&i32);
    let b: Type = parse_quote!(i32);
    assert_ne!(ty_str(&a), ty_str(&b));
}

#[test]
fn compare_diff_tuple_order() {
    let a: Type = parse_quote!((i32, String));
    let b: Type = parse_quote!((String, i32));
    assert_ne!(ty_str(&a), ty_str(&b));
}

// ============================================================================
// Section 10: Extract then stringify (tests 65–72)
// ============================================================================

#[test]
fn extract_vec_i32_str() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_option_string_str() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_box_bool_str() {
    let ty: Type = parse_quote!(Box<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_vec_option_i32_str() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn extract_not_found_str() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (returned, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(!found);
    assert_eq!(ty_str(&returned), "Vec < i32 >");
}

#[test]
fn extract_skip_box_to_vec_str() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_skip_arc_box_to_option_str() {
    let ty: Type = parse_quote!(Arc<Box<Option<u8>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_option_vec_inner_str() {
    let ty: Type = parse_quote!(Option<Vec<f64>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    assert_eq!(ty_str(&inner), "Vec < f64 >");
}

// ============================================================================
// Section 11: Filter then stringify (tests 73–80)
// ============================================================================

#[test]
fn filter_vec_skip_vec_str() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_box_skip_box_str() {
    let ty: Type = parse_quote!(Box<String>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn filter_option_skip_option_str() {
    let ty: Type = parse_quote!(Option<bool>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn filter_box_vec_skip_both_str() {
    let ty: Type = parse_quote!(Box<Vec<u16>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Vec"]));
    assert_eq!(ty_str(&result), "u16");
}

#[test]
fn filter_no_skip_unchanged_str() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &empty_skip());
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

#[test]
fn filter_non_matching_skip_str() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = filter_inner_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

#[test]
fn filter_primitive_unchanged_str() {
    let ty: Type = parse_quote!(i32);
    let result = filter_inner_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_arc_box_skip_both_str() {
    let ty: Type = parse_quote!(Arc<Box<u64>>);
    let result = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(ty_str(&result), "u64");
}

// ============================================================================
// Section 12: Wrap then stringify (tests 81–88)
// ============================================================================

#[test]
fn wrap_i32_str() {
    let ty: Type = parse_quote!(i32);
    let result = wrap_leaf_type(&ty, &empty_skip());
    let s = ty_str(&result);
    assert!(s.contains("adze"));
    assert!(s.contains("i32"));
}

#[test]
fn wrap_string_str() {
    let ty: Type = parse_quote!(String);
    let result = wrap_leaf_type(&ty, &empty_skip());
    let s = ty_str(&result);
    assert!(s.contains("adze"));
    assert!(s.contains("String"));
}

#[test]
fn wrap_vec_skip_vec_str() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = wrap_leaf_type(&ty, &skip(&["Vec"]));
    let s = ty_str(&result);
    assert!(s.contains("Vec"));
    assert!(s.contains("i32"));
    assert!(s.contains("adze"));
}

#[test]
fn wrap_option_skip_option_str() {
    let ty: Type = parse_quote!(Option<String>);
    let result = wrap_leaf_type(&ty, &skip(&["Option"]));
    let s = ty_str(&result);
    assert!(s.contains("Option"));
    assert!(s.contains("String"));
    assert!(s.contains("adze"));
}

#[test]
fn wrap_box_skip_box_str() {
    let ty: Type = parse_quote!(Box<u8>);
    let result = wrap_leaf_type(&ty, &skip(&["Box"]));
    let s = ty_str(&result);
    assert!(s.contains("Box"));
    assert!(s.contains("u8"));
    assert!(s.contains("adze"));
}

#[test]
fn wrap_bool_no_skip_str() {
    let ty: Type = parse_quote!(bool);
    let result = wrap_leaf_type(&ty, &empty_skip());
    let s = ty_str(&result);
    assert!(s.contains("adze"));
    assert!(s.contains("bool"));
}

#[test]
fn wrap_vec_no_skip_wraps_outer_str() {
    let ty: Type = parse_quote!(Vec<i32>);
    let result = wrap_leaf_type(&ty, &empty_skip());
    let s = ty_str(&result);
    assert!(s.contains("adze"));
    assert!(s.contains("Vec"));
}

#[test]
fn wrap_nested_skip_both_str() {
    let ty: Type = parse_quote!(Vec<Option<i32>>);
    let result = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    let s = ty_str(&result);
    assert!(s.contains("Vec"));
    assert!(s.contains("Option"));
    assert!(s.contains("i32"));
    assert!(s.contains("adze"));
}

// ============================================================================
// Section 13: Roundtrip — extract and compare strings (tests 89–93)
// ============================================================================

#[test]
fn roundtrip_extract_matches_expected_type_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &empty_skip());
    assert!(found);
    let expected: Type = parse_quote!(String);
    assert_eq!(ty_str(&inner), ty_str(&expected));
}

#[test]
fn roundtrip_filter_matches_expected_type_string() {
    let ty: Type = parse_quote!(Box<i64>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    let expected: Type = parse_quote!(i64);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn roundtrip_extract_nested_matches_string() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty_skip());
    assert!(found);
    let expected: Type = parse_quote!(Vec<bool>);
    assert_eq!(ty_str(&inner), ty_str(&expected));
}

#[test]
fn roundtrip_filter_double_unwrap_string() {
    let ty: Type = parse_quote!(Arc<Box<f32>>);
    let result = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    let expected: Type = parse_quote!(f32);
    assert_eq!(ty_str(&result), ty_str(&expected));
}

#[test]
fn roundtrip_extract_with_skip_string() {
    let ty: Type = parse_quote!(Box<Option<u16>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(found);
    let expected: Type = parse_quote!(u16);
    assert_eq!(ty_str(&inner), ty_str(&expected));
}
