// Comprehensive v2 tests for macro expansion patterns via adze-common.
// Covers NameValueExpr, FieldThenParams, try_extract_inner_type,
// filter_inner_type, and wrap_leaf_type with 60+ focused tests.

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote, parse_str};

// =====================================================================
// Helpers
// =====================================================================

fn ts(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip_set<'a>(names: &[&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// =====================================================================
// 1. NameValueExpr – Parse implementation
// =====================================================================

#[test]
fn nve_simple_string_value() {
    let nve: NameValueExpr = parse_str("name = \"hello\"").unwrap();
    assert_eq!(nve.path.to_string(), "name");
}

#[test]
fn nve_integer_value() {
    let nve: NameValueExpr = parse_str("count = 42").unwrap();
    assert_eq!(nve.path.to_string(), "count");
}

#[test]
fn nve_boolean_true() {
    let nve: NameValueExpr = parse_str("flag = true").unwrap();
    assert_eq!(nve.path.to_string(), "flag");
}

#[test]
fn nve_boolean_false() {
    let nve: NameValueExpr = parse_str("enabled = false").unwrap();
    assert_eq!(nve.path.to_string(), "enabled");
}

#[test]
fn nve_negative_integer() {
    let nve: NameValueExpr = parse_str("offset = -10").unwrap();
    assert_eq!(nve.path.to_string(), "offset");
}

#[test]
fn nve_float_value() {
    let nve: NameValueExpr = parse_str("weight = 1.5").unwrap();
    assert_eq!(nve.path.to_string(), "weight");
}

#[test]
fn nve_char_value() {
    let nve: NameValueExpr = parse_str("delim = ','").unwrap();
    assert_eq!(nve.path.to_string(), "delim");
}

#[test]
fn nve_path_value() {
    let nve: NameValueExpr = parse_str("kind = SomeEnum::Variant").unwrap();
    assert_eq!(nve.path.to_string(), "kind");
}

#[test]
fn nve_underscore_ident_name() {
    let nve: NameValueExpr = parse_str("my_param = 7").unwrap();
    assert_eq!(nve.path.to_string(), "my_param");
}

#[test]
fn nve_raw_ident_gen() {
    // `gen` is a reserved keyword in Rust 2024 edition; use raw identifier
    let nve: NameValueExpr = parse_str("r#gen = 1").unwrap();
    assert_eq!(nve.path.to_string(), "r#gen");
}

#[test]
fn nve_block_expr_value() {
    let nve: NameValueExpr = parse_str("val = { 2 + 3 }").unwrap();
    assert_eq!(nve.path.to_string(), "val");
}

#[test]
fn nve_array_expr_value() {
    let nve: NameValueExpr = parse_str("items = [1, 2]").unwrap();
    assert_eq!(nve.path.to_string(), "items");
}

#[test]
fn nve_missing_equals_is_error() {
    assert!(parse_str::<NameValueExpr>("key value").is_err());
}

#[test]
fn nve_missing_value_is_error() {
    assert!(parse_str::<NameValueExpr>("key =").is_err());
}

#[test]
fn nve_empty_input_is_error() {
    assert!(parse_str::<NameValueExpr>("").is_err());
}

// =====================================================================
// 2. FieldThenParams – Parse implementation
// =====================================================================

#[test]
fn ftp_bare_primitive() {
    let ftp: FieldThenParams = parse_str("u32").unwrap();
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ts(&ftp.field.ty), "u32");
}

#[test]
fn ftp_bare_string() {
    let ftp: FieldThenParams = parse_str("String").unwrap();
    assert!(ftp.params.is_empty());
    assert_eq!(ts(&ftp.field.ty), "String");
}

#[test]
fn ftp_generic_vec() {
    let ftp: FieldThenParams = parse_str("Vec<u8>").unwrap();
    assert!(ftp.params.is_empty());
    assert_eq!(ts(&ftp.field.ty), "Vec < u8 >");
}

#[test]
fn ftp_one_param() {
    let ftp: FieldThenParams = parse_str("i64, label = \"id\"").unwrap();
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "label");
}

#[test]
fn ftp_two_params() {
    let ftp: FieldThenParams = parse_str("bool, a = 1, b = 2").unwrap();
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "a");
    assert_eq!(ftp.params[1].path.to_string(), "b");
}

#[test]
fn ftp_three_params() {
    let ftp: FieldThenParams = parse_str("f64, x = 1, y = 2, z = 3").unwrap();
    assert_eq!(ftp.params.len(), 3);
    assert_eq!(ftp.params[2].path.to_string(), "z");
}

#[test]
fn ftp_nested_generic_type() {
    let ftp: FieldThenParams = parse_str("Option<Vec<i32>>").unwrap();
    assert!(ftp.params.is_empty());
    assert_eq!(ts(&ftp.field.ty), "Option < Vec < i32 > >");
}

#[test]
fn ftp_nested_generic_with_param() {
    let ftp: FieldThenParams = parse_str("HashMap<String, i32>, ordered = true").unwrap();
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "ordered");
}

#[test]
fn ftp_reference_type() {
    let ftp: FieldThenParams = parse_str("&str").unwrap();
    assert!(ftp.params.is_empty());
    assert_eq!(ts(&ftp.field.ty), "& str");
}

#[test]
fn ftp_qualified_path() {
    let ftp: FieldThenParams = parse_str("std::string::String").unwrap();
    assert!(ftp.params.is_empty());
    assert!(ts(&ftp.field.ty).contains("std"));
}

// =====================================================================
// 3. try_extract_inner_type – complex nested types
// =====================================================================

#[test]
fn extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_box_as_target() {
    let ty: Type = parse_quote!(Box<f64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "f64");
}

#[test]
fn extract_skip_box_to_vec() {
    let ty: Type = parse_quote!(Box<Vec<u16>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "u16");
}

#[test]
fn extract_skip_arc_to_option() {
    let ty: Type = parse_quote!(Arc<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&["Arc"]));
    assert!(ok);
    assert_eq!(ts(&inner), "bool");
}

#[test]
fn extract_double_skip_box_arc_to_vec() {
    let ty: Type = parse_quote!(Box<Arc<Vec<u8>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(ts(&inner), "u8");
}

#[test]
fn extract_option_vec_box_deep_chain() {
    // Option is target, Vec and Box are in skip — but Option is outermost so it matches first
    let ty: Type = parse_quote!(Option<Vec<Box<i32>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&["Vec", "Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "Vec < Box < i32 > >");
}

#[test]
fn extract_target_not_present_returns_original() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "HashMap < String , i32 >");
}

#[test]
fn extract_skip_present_but_target_missing() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Box"]));
    assert!(!ok);
    assert_eq!(ts(&inner), "Box < String >");
}

#[test]
fn extract_plain_ident_not_extracted() {
    let ty: Type = parse_quote!(MyStruct);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "MyStruct");
}

#[test]
fn extract_reference_not_extracted() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "& str");
}

#[test]
fn extract_tuple_not_extracted() {
    let ty: Type = parse_quote!((u8, u16));
    let (_inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(!ok);
}

#[test]
fn extract_hashmap_first_arg() {
    // Extracting HashMap returns its first generic argument
    let ty: Type = parse_quote!(HashMap<String, Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "HashMap", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_result_first_arg() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn extract_vec_of_tuple() {
    let ty: Type = parse_quote!(Vec<(i32, bool)>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "(i32 , bool)");
}

// =====================================================================
// 4. filter_inner_type – preserves structure
// =====================================================================

#[test]
fn filter_box_to_inner() {
    let ty: Type = parse_quote!(Box<String>);
    let f = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(ts(&f), "String");
}

#[test]
fn filter_arc_to_inner() {
    let ty: Type = parse_quote!(Arc<u32>);
    let f = filter_inner_type(&ty, &skip_set(&["Arc"]));
    assert_eq!(ts(&f), "u32");
}

#[test]
fn filter_nested_box_arc_all_stripped() {
    let ty: Type = parse_quote!(Box<Arc<f64>>);
    let f = filter_inner_type(&ty, &skip_set(&["Box", "Arc"]));
    assert_eq!(ts(&f), "f64");
}

#[test]
fn filter_three_deep_rc_box_arc() {
    let ty: Type = parse_quote!(Rc<Box<Arc<bool>>>);
    let f = filter_inner_type(&ty, &skip_set(&["Rc", "Box", "Arc"]));
    assert_eq!(ts(&f), "bool");
}

#[test]
fn filter_stops_at_non_skip_container() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let f = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(ts(&f), "Vec < i32 >");
}

#[test]
fn filter_non_matching_unchanged() {
    let ty: Type = parse_quote!(Vec<String>);
    let f = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(ts(&f), "Vec < String >");
}

#[test]
fn filter_plain_type_unchanged() {
    let ty: Type = parse_quote!(i64);
    let f = filter_inner_type(&ty, &skip_set(&["Box", "Arc"]));
    assert_eq!(ts(&f), "i64");
}

#[test]
fn filter_empty_skip_set_unchanged() {
    let ty: Type = parse_quote!(Box<u8>);
    let f = filter_inner_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&f), "Box < u8 >");
}

#[test]
fn filter_reference_unchanged() {
    let ty: Type = parse_quote!(&[u8]);
    let f = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(ts(&f), "& [u8]");
}

#[test]
fn filter_tuple_unchanged() {
    let ty: Type = parse_quote!((i32, String));
    let f = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(ts(&f), "(i32 , String)");
}

#[test]
fn filter_qualified_path_unchanged() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    let f = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(ts(&f), "std :: collections :: HashMap < String , i32 >");
}

// =====================================================================
// 5. wrap_leaf_type – transforms correctly
// =====================================================================

#[test]
fn wrap_simple_string() {
    let ty: Type = parse_quote!(String);
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_primitive_i32() {
    let ty: Type = parse_quote!(i32);
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_primitive_bool() {
    let ty: Type = parse_quote!(bool);
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_vec_skipped_wraps_inner() {
    let ty: Type = parse_quote!(Vec<u8>);
    let w = wrap_leaf_type(&ty, &skip_set(&["Vec"]));
    assert_eq!(ts(&w), "Vec < adze :: WithLeaf < u8 > >");
}

#[test]
fn wrap_option_skipped_wraps_inner() {
    let ty: Type = parse_quote!(Option<String>);
    let w = wrap_leaf_type(&ty, &skip_set(&["Option"]));
    assert_eq!(ts(&w), "Option < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_vec_not_skipped_wraps_whole() {
    let ty: Type = parse_quote!(Vec<u8>);
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < Vec < u8 > >");
}

#[test]
fn wrap_nested_option_vec_both_skipped() {
    let ty: Type = parse_quote!(Option<Vec<f32>>);
    let w = wrap_leaf_type(&ty, &skip_set(&["Option", "Vec"]));
    assert_eq!(ts(&w), "Option < Vec < adze :: WithLeaf < f32 > > >");
}

#[test]
fn wrap_deeply_nested_three_skips() {
    let ty: Type = parse_quote!(Option<Vec<Option<i32>>>);
    let w = wrap_leaf_type(&ty, &skip_set(&["Option", "Vec"]));
    assert_eq!(
        ts(&w),
        "Option < Vec < Option < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn wrap_result_skipped_wraps_both_args() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let w = wrap_leaf_type(&ty, &skip_set(&["Result"]));
    assert_eq!(
        ts(&w),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_result_and_vec_skipped() {
    let ty: Type = parse_quote!(Result<Vec<u8>, String>);
    let w = wrap_leaf_type(&ty, &skip_set(&["Result", "Vec"]));
    assert_eq!(
        ts(&w),
        "Result < Vec < adze :: WithLeaf < u8 > > , adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_hashmap_skipped_wraps_both_args() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let w = wrap_leaf_type(&ty, &skip_set(&["HashMap"]));
    assert_eq!(
        ts(&w),
        "HashMap < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_hashmap_and_vec_skipped() {
    let ty: Type = parse_quote!(HashMap<String, Vec<u8>>);
    let w = wrap_leaf_type(&ty, &skip_set(&["HashMap", "Vec"]));
    assert_eq!(
        ts(&w),
        "HashMap < adze :: WithLeaf < String > , Vec < adze :: WithLeaf < u8 > > >"
    );
}

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < () >");
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, bool));
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < (i32 , bool) >");
}

// =====================================================================
// 6. Type patterns: Option<Vec<Box<T>>>
// =====================================================================

#[test]
fn extract_through_option_vec_box_chain() {
    // Skip Option and Vec, target is Box
    let ty: Type = parse_quote!(Option<Vec<Box<u32>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip_set(&["Option", "Vec"]));
    assert!(ok);
    assert_eq!(ts(&inner), "u32");
}

#[test]
fn filter_option_vec_box_only_option_skipped() {
    // Only Option is in skip set — filter unwraps Option, stops at Vec
    let ty: Type = parse_quote!(Option<Vec<Box<u32>>>);
    let f = filter_inner_type(&ty, &skip_set(&["Option"]));
    assert_eq!(ts(&f), "Vec < Box < u32 > >");
}

#[test]
fn wrap_option_vec_box_all_skipped() {
    let ty: Type = parse_quote!(Option<Vec<Box<u32>>>);
    let w = wrap_leaf_type(&ty, &skip_set(&["Option", "Vec", "Box"]));
    assert_eq!(
        ts(&w),
        "Option < Vec < Box < adze :: WithLeaf < u32 > > > >"
    );
}

// =====================================================================
// 7. Type patterns: Result<T, E>
// =====================================================================

#[test]
fn extract_result_returns_first_arg() {
    let ty: Type = parse_quote!(Result<Vec<u8>, String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Vec < u8 >");
}

#[test]
fn filter_result_not_in_skip_unchanged() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let f = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(ts(&f), "Result < i32 , String >");
}

#[test]
fn wrap_result_not_skipped_wraps_whole() {
    let ty: Type = parse_quote!(Result<i32, String>);
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < Result < i32 , String > >");
}

// =====================================================================
// 8. Primitive types pass through
// =====================================================================

#[test]
fn extract_primitive_u8_passthrough() {
    let ty: Type = parse_quote!(u8);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Box"]));
    assert!(!ok);
    assert_eq!(ts(&inner), "u8");
}

#[test]
fn extract_primitive_f64_passthrough() {
    let ty: Type = parse_quote!(f64);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), "f64");
}

#[test]
fn filter_primitive_bool_passthrough() {
    let ty: Type = parse_quote!(bool);
    let f = filter_inner_type(&ty, &skip_set(&["Box", "Arc"]));
    assert_eq!(ts(&f), "bool");
}

#[test]
fn filter_primitive_usize_passthrough() {
    let ty: Type = parse_quote!(usize);
    let f = filter_inner_type(&ty, &skip_set(&["Rc"]));
    assert_eq!(ts(&f), "usize");
}

#[test]
fn wrap_primitive_u64() {
    let ty: Type = parse_quote!(u64);
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < u64 >");
}

// =====================================================================
// 9. Complex paths like std::collections::HashMap
// =====================================================================

#[test]
fn extract_qualified_hashmap() {
    // Qualified paths match on the last segment
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "HashMap", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn filter_qualified_box() {
    let ty: Type = parse_quote!(std::boxed::Box<u8>);
    let f = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(ts(&f), "u8");
}

#[test]
fn wrap_qualified_vec_skipped() {
    let ty: Type = parse_quote!(std::vec::Vec<i32>);
    let w = wrap_leaf_type(&ty, &skip_set(&["Vec"]));
    assert_eq!(ts(&w), "std :: vec :: Vec < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_qualified_path_not_skipped() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(
        ts(&w),
        "adze :: WithLeaf < std :: collections :: HashMap < String , i32 > >"
    );
}

// =====================================================================
// 10. Edge cases and error conditions
// =====================================================================

#[test]
fn parse_str_never_type() {
    let ty: Type = parse_str("!").unwrap();
    assert_eq!(ts(&ty), "!");
}

#[test]
fn parse_str_raw_pointer() {
    let ty: Type = parse_str("*const u8").unwrap();
    assert_eq!(ts(&ty), "* const u8");
}

#[test]
fn parse_str_fn_pointer() {
    let ty: Type = parse_str("fn(i32) -> bool").unwrap();
    assert_eq!(ts(&ty), "fn (i32) -> bool");
}

#[test]
fn parse_str_impl_trait() {
    let ty: Type = parse_str("impl Clone").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), ts(&ty));
}

#[test]
fn parse_str_lifetime_reference() {
    let ty: Type = parse_str("&'a str").unwrap();
    assert_eq!(ts(&ty), "& 'a str");
}

#[test]
fn parse_str_dyn_trait() {
    let ty: Type = parse_str("Box<dyn Send>").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "dyn Send");
}

#[test]
fn filter_never_type_unchanged() {
    let ty: Type = parse_str("!").unwrap();
    let f = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(ts(&f), "!");
}

#[test]
fn wrap_never_type() {
    let ty: Type = parse_str("!").unwrap();
    let w = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < ! >");
}

#[test]
fn parse_str_invalid_type_errors() {
    assert!(parse_str::<Type>("123invalid").is_err());
}

// =====================================================================
// 11. Composability: extract → filter → wrap chains
// =====================================================================

#[test]
fn extract_then_wrap_leaf() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(ok);
    let w = wrap_leaf_type(&inner, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_wrap_leaf() {
    let ty: Type = parse_quote!(Box<i32>);
    let f = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(ts(&f), "i32");
    let w = wrap_leaf_type(&f, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < i32 >");
}

#[test]
fn extract_filter_wrap_full_chain() {
    let ty: Type = parse_quote!(Arc<Vec<Box<u8>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Arc"]));
    assert!(ok);
    assert_eq!(ts(&inner), "Box < u8 >");
    let f = filter_inner_type(&inner, &skip_set(&["Box"]));
    assert_eq!(ts(&f), "u8");
    let w = wrap_leaf_type(&f, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < u8 >");
}

#[test]
fn double_filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let f = filter_inner_type(&ty, &skip_set(&["Box", "Arc"]));
    assert_eq!(ts(&f), "String");
    let w = wrap_leaf_type(&f, &skip_set(&[]));
    assert_eq!(ts(&w), "adze :: WithLeaf < String >");
}

#[test]
fn extract_option_then_wrap_with_vec_skip() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Vec < i32 >");
    let w = wrap_leaf_type(&inner, &skip_set(&["Vec"]));
    assert_eq!(ts(&w), "Vec < adze :: WithLeaf < i32 > >");
}

// =====================================================================
// 12. Idempotency and symmetry properties
// =====================================================================

#[test]
fn filter_idempotent_after_first_unwrap() {
    let ty: Type = parse_quote!(Box<String>);
    let f1 = filter_inner_type(&ty, &skip_set(&["Box"]));
    let f2 = filter_inner_type(&f1, &skip_set(&["Box"]));
    // String is not Box, so second filter is a no-op
    assert_eq!(ts(&f1), ts(&f2));
}

#[test]
fn extract_idempotent_on_non_matching() {
    let ty: Type = parse_quote!(String);
    let (i1, ok1) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(!ok1);
    let (i2, ok2) = try_extract_inner_type(&i1, "Vec", &skip_set(&[]));
    assert!(!ok2);
    assert_eq!(ts(&i1), ts(&i2));
}

#[test]
fn filter_plain_type_fully_stable() {
    let ty: Type = parse_quote!(u32);
    let f = filter_inner_type(&ty, &skip_set(&["Box", "Arc", "Rc"]));
    assert_eq!(ts(&f), "u32");
    let f2 = filter_inner_type(&f, &skip_set(&["Box", "Arc", "Rc"]));
    assert_eq!(ts(&f), ts(&f2));
}
