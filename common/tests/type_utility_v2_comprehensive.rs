#![allow(clippy::needless_range_loop)]

//! Comprehensive v2 tests for type utility functions in adze-common:
//! `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`.

use std::collections::HashSet;

use adze_common::{filter_inner_type, try_extract_inner_type, wrap_leaf_type};
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

// ===========================================================================
// 1. try_extract_inner_type — Option<T>
// ===========================================================================

#[test]
fn extract_option_u8() {
    let ty: Type = parse_quote!(Option<u8>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_f64() {
    let ty: Type = parse_quote!(Option<f64>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn extract_option_bool() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_option_custom_type() {
    let ty: Type = parse_quote!(Option<MyStruct>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "MyStruct");
}

#[test]
fn extract_option_nested_vec() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Vec < i32 >");
}

#[test]
fn extract_option_usize() {
    let ty: Type = parse_quote!(Option<usize>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "usize");
}

// ===========================================================================
// 2. try_extract_inner_type — Vec<T>
// ===========================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_vec_u32() {
    let ty: Type = parse_quote!(Vec<u32>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn extract_vec_bool() {
    let ty: Type = parse_quote!(Vec<bool>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_vec_custom() {
    let ty: Type = parse_quote!(Vec<Token>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Token");
}

#[test]
fn extract_vec_nested_option() {
    let ty: Type = parse_quote!(Vec<Option<u8>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Option < u8 >");
}

#[test]
fn extract_vec_i64() {
    let ty: Type = parse_quote!(Vec<i64>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "i64");
}

// ===========================================================================
// 3. try_extract_inner_type — Box<T>
// ===========================================================================

#[test]
fn extract_box_i32() {
    let ty: Type = parse_quote!(Box<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_box_custom() {
    let ty: Type = parse_quote!(Box<Expr>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Expr");
}

#[test]
fn extract_box_vec_inner() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Vec < u8 >");
}

#[test]
fn extract_box_f32() {
    let ty: Type = parse_quote!(Box<f32>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "f32");
}

// ===========================================================================
// 4. try_extract_inner_type — non-matching types
// ===========================================================================

#[test]
fn extract_plain_i32_not_found() {
    let ty: Type = parse_quote!(i32);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn extract_string_looking_for_vec() {
    let ty: Type = parse_quote!(String);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn extract_vec_looking_for_option() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

#[test]
fn extract_box_looking_for_vec() {
    let ty: Type = parse_quote!(Box<String>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "Box < String >");
}

#[test]
fn extract_custom_type_not_found() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "HashMap < String , i32 >");
}

#[test]
fn extract_bool_not_found() {
    let ty: Type = parse_quote!(bool);
    let (result, found) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn extract_usize_not_found() {
    let ty: Type = parse_quote!(usize);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "usize");
}

// ===========================================================================
// 5. try_extract_inner_type — nested generics
// ===========================================================================

#[test]
fn extract_option_option_inner() {
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Option < i32 >");
}

#[test]
fn extract_vec_vec_inner() {
    let ty: Type = parse_quote!(Vec<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn extract_option_box_inner() {
    let ty: Type = parse_quote!(Option<Box<f64>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Box < f64 >");
}

#[test]
fn extract_vec_option_string() {
    let ty: Type = parse_quote!(Vec<Option<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Option < String >");
}

#[test]
fn extract_box_option_vec() {
    let ty: Type = parse_quote!(Box<Option<Vec<u8>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Option < Vec < u8 > >");
}

// ===========================================================================
// 6. try_extract_inner_type — skip_over
// ===========================================================================

#[test]
fn skip_box_extract_option() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn skip_box_extract_vec() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn skip_arc_extract_option() {
    let ty: Type = parse_quote!(Arc<Option<bool>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn skip_multiple_extract_vec() {
    let ty: Type = parse_quote!(Arc<Box<Vec<u16>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc", "Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "u16");
}

#[test]
fn skip_box_target_not_found() {
    let ty: Type = parse_quote!(Box<String>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!found);
    assert_eq!(ty_str(&result), "Box < String >");
}

#[test]
fn skip_irrelevant_container() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn skip_box_arc_extract_option() {
    let ty: Type = parse_quote!(Box<Arc<Option<f64>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "f64");
}

#[test]
fn skip_single_layer_no_match_inside() {
    let ty: Type = parse_quote!(Box<i32>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!found);
    assert_eq!(ty_str(&result), "Box < i32 >");
}

// ===========================================================================
// 7. filter_inner_type — with various skips
// ===========================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn filter_arc_i32() {
    let ty: Type = parse_quote!(Arc<i32>);
    let result = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_box_arc_nested() {
    let ty: Type = parse_quote!(Box<Arc<bool>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&result), "bool");
}

#[test]
fn filter_triple_nested() {
    let ty: Type = parse_quote!(Rc<Box<Arc<u64>>>);
    let result = filter_inner_type(&ty, &skip(&["Rc", "Box", "Arc"]));
    assert_eq!(ty_str(&result), "u64");
}

#[test]
fn filter_stops_at_non_skip() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Vec < i32 >");
}

#[test]
fn filter_option_not_in_skip() {
    let ty: Type = parse_quote!(Option<String>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Option < String >");
}

#[test]
fn filter_box_option_partial() {
    let ty: Type = parse_quote!(Box<Option<u8>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Option < u8 >");
}

#[test]
fn filter_box_box_nested() {
    let ty: Type = parse_quote!(Box<Box<f64>>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "f64");
}

// ===========================================================================
// 8. filter_inner_type — without skips (empty set)
// ===========================================================================

#[test]
fn filter_no_skip_i32() {
    let ty: Type = parse_quote!(i32);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "i32");
}

#[test]
fn filter_no_skip_string() {
    let ty: Type = parse_quote!(String);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "String");
}

#[test]
fn filter_no_skip_vec() {
    let ty: Type = parse_quote!(Vec<u32>);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "Vec < u32 >");
}

#[test]
fn filter_no_skip_option() {
    let ty: Type = parse_quote!(Option<bool>);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "Option < bool >");
}

#[test]
fn filter_no_skip_box() {
    let ty: Type = parse_quote!(Box<String>);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "Box < String >");
}

#[test]
fn filter_no_skip_custom() {
    let ty: Type = parse_quote!(MyWrapper<i32>);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "MyWrapper < i32 >");
}

#[test]
fn filter_no_skip_bool() {
    let ty: Type = parse_quote!(bool);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "bool");
}

// ===========================================================================
// 9. wrap_leaf_type — various types
// ===========================================================================

#[test]
fn wrap_i32_no_skip() {
    let ty: Type = parse_quote!(i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_string_no_skip() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_bool_no_skip() {
    let ty: Type = parse_quote!(bool);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_custom_no_skip() {
    let ty: Type = parse_quote!(Expr);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Expr >");
}

#[test]
fn wrap_option_in_skip_wraps_inner() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_vec_in_skip_wraps_inner() {
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_box_in_skip_wraps_inner() {
    let ty: Type = parse_quote!(Box<bool>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&wrapped), "Box < adze :: WithLeaf < bool > >");
}

#[test]
fn wrap_option_not_in_skip_wraps_whole() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Option < i32 > >");
}

#[test]
fn wrap_vec_not_in_skip_wraps_whole() {
    let ty: Type = parse_quote!(Vec<u8>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Vec < u8 > >");
}

#[test]
fn wrap_nested_option_vec_skip_both() {
    let ty: Type = parse_quote!(Option<Vec<f64>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < f64 > > >"
    );
}

#[test]
fn wrap_nested_vec_option_skip_both() {
    let ty: Type = parse_quote!(Vec<Option<u32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < u32 > > >"
    );
}

#[test]
fn wrap_box_option_skip_both() {
    let ty: Type = parse_quote!(Box<Option<String>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Box", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Box < Option < adze :: WithLeaf < String > > >"
    );
}

#[test]
fn wrap_f64_no_skip() {
    let ty: Type = parse_quote!(f64);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < f64 >");
}

#[test]
fn wrap_u8_no_skip() {
    let ty: Type = parse_quote!(u8);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u8 >");
}

// ===========================================================================
// 10. Edge cases — primitives, tuples, references, paths
// ===========================================================================

#[test]
fn extract_reference_type_not_found() {
    let ty: Type = parse_quote!(&str);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "& str");
}

#[test]
fn extract_tuple_type_not_found() {
    let ty: Type = parse_quote!((i32, String));
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "(i32 , String)");
}

#[test]
fn filter_reference_unchanged() {
    let ty: Type = parse_quote!(&str);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "& str");
}

#[test]
fn filter_tuple_unchanged() {
    let ty: Type = parse_quote!((u8, bool));
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "(u8 , bool)");
}

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, bool));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (i32 , bool) >");
}

#[test]
fn extract_path_type_qualified() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_isize_not_found() {
    let ty: Type = parse_quote!(isize);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "isize");
}

#[test]
fn filter_isize_no_skip() {
    let ty: Type = parse_quote!(isize);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "isize");
}

#[test]
fn wrap_isize_no_skip() {
    let ty: Type = parse_quote!(isize);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < isize >");
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < () >");
}

#[test]
fn extract_unit_type_not_found() {
    let ty: Type = parse_quote!(());
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "()");
}

#[test]
fn filter_unit_type_unchanged() {
    let ty: Type = parse_quote!(());
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "()");
}

#[test]
fn wrap_with_unrelated_skip() {
    let ty: Type = parse_quote!(u16);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < u16 >");
}

#[test]
fn extract_result_looking_for_result() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let (inner, found) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn filter_result_not_in_skip() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let result = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&result), "Result < i32 , String >");
}

#[test]
fn wrap_result_not_in_skip() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < Result < i32 , String > >"
    );
}

#[test]
fn extract_option_with_lifetime_param() {
    let ty: Type = parse_quote!(Option<Cow<'static, str>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Cow < 'static , str >");
}

#[test]
fn filter_deeply_nested_stops_correctly() {
    let ty: Type = parse_quote!(Box<Arc<Rc<String>>>);
    let result = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&result), "Rc < String >");
}

#[test]
fn wrap_option_vec_skip_option_only() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < adze :: WithLeaf < Vec < i32 > > >"
    );
}

#[test]
fn extract_phantom_data_not_found() {
    let ty: Type = parse_quote!(PhantomData<T>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "PhantomData < T >");
}

#[test]
fn extract_option_with_empty_skip_set() {
    let ty: Type = parse_quote!(Option<char>);
    let empty: HashSet<&str> = HashSet::new();
    let (inner, found) = try_extract_inner_type(&ty, "Option", &empty);
    assert!(found);
    assert_eq!(ty_str(&inner), "char");
}

#[test]
fn filter_char_no_skip() {
    let ty: Type = parse_quote!(char);
    let result = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&result), "char");
}

#[test]
fn wrap_char_no_skip() {
    let ty: Type = parse_quote!(char);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < char >");
}

#[test]
fn extract_skip_over_does_not_match_target() {
    // skip_over=["Box"], target="Vec", but type is Box<String> — no Vec inside
    let ty: Type = parse_quote!(Box<String>);
    let (result, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!found);
    assert_eq!(ty_str(&result), "Box < String >");
}

#[test]
fn wrap_vec_of_vec_skip_vec() {
    let ty: Type = parse_quote!(Vec<Vec<u8>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < Vec < adze :: WithLeaf < u8 > > >");
}

#[test]
fn filter_rc_in_skip() {
    let ty: Type = parse_quote!(Rc<u64>);
    let result = filter_inner_type(&ty, &skip(&["Rc"]));
    assert_eq!(ty_str(&result), "u64");
}
