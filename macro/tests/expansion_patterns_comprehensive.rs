// Comprehensive tests for macro expansion patterns and type handling
// Tests adze_common types: NameValueExpr, FieldThenParams, try_extract_inner_type,
// filter_inner_type, wrap_leaf_type

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Type, parse_quote, parse_str};

// =====================================================================
// Helper
// =====================================================================

fn type_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// =====================================================================
// 1. syn::parse_str for various Rust types
// =====================================================================

#[test]
fn parse_str_simple_ident() {
    let ty: Type = parse_str("i32").unwrap();
    assert_eq!(type_str(&ty), "i32");
}

#[test]
fn parse_str_string_type() {
    let ty: Type = parse_str("String").unwrap();
    assert_eq!(type_str(&ty), "String");
}

#[test]
fn parse_str_bool() {
    let ty: Type = parse_str("bool").unwrap();
    assert_eq!(type_str(&ty), "bool");
}

#[test]
fn parse_str_unit_type() {
    let ty: Type = parse_str("()").unwrap();
    assert_eq!(type_str(&ty), "()");
}

#[test]
fn parse_str_reference() {
    let ty: Type = parse_str("&str").unwrap();
    assert_eq!(type_str(&ty), "& str");
}

#[test]
fn parse_str_mutable_reference() {
    let ty: Type = parse_str("&mut i32").unwrap();
    assert_eq!(type_str(&ty), "& mut i32");
}

#[test]
fn parse_str_slice() {
    let ty: Type = parse_str("&[u8]").unwrap();
    assert_eq!(type_str(&ty), "& [u8]");
}

#[test]
fn parse_str_array() {
    let ty: Type = parse_str("[u8; 4]").unwrap();
    assert_eq!(type_str(&ty), "[u8 ; 4]");
}

#[test]
fn parse_str_tuple() {
    let ty: Type = parse_str("(i32, String)").unwrap();
    assert_eq!(type_str(&ty), "(i32 , String)");
}

#[test]
fn parse_str_option() {
    let ty: Type = parse_str("Option<i32>").unwrap();
    assert_eq!(type_str(&ty), "Option < i32 >");
}

#[test]
fn parse_str_vec() {
    let ty: Type = parse_str("Vec<String>").unwrap();
    assert_eq!(type_str(&ty), "Vec < String >");
}

#[test]
fn parse_str_result() {
    let ty: Type = parse_str("Result<i32, String>").unwrap();
    assert_eq!(type_str(&ty), "Result < i32 , String >");
}

#[test]
fn parse_str_nested_generic() {
    let ty: Type = parse_str("Vec<Option<i32>>").unwrap();
    assert_eq!(type_str(&ty), "Vec < Option < i32 > >");
}

#[test]
fn parse_str_qualified_path() {
    let ty: Type = parse_str("std::collections::HashMap<String, i32>").unwrap();
    assert_eq!(
        type_str(&ty),
        "std :: collections :: HashMap < String , i32 >"
    );
}

#[test]
fn parse_str_fn_pointer() {
    let ty: Type = parse_str("fn(i32) -> bool").unwrap();
    assert_eq!(type_str(&ty), "fn (i32) -> bool");
}

#[test]
fn parse_str_triple_nested() {
    let ty: Type = parse_str("Box<Vec<Option<u8>>>").unwrap();
    assert_eq!(type_str(&ty), "Box < Vec < Option < u8 > > >");
}

// =====================================================================
// 2. try_extract_inner_type — basic extraction
// =====================================================================

#[test]
fn extract_vec_string() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(type_str(&inner), "i32");
}

#[test]
fn extract_not_matching_returns_original() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(type_str(&inner), type_str(&ty));
}

#[test]
fn extract_plain_type_not_extracted() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_through_box_skip() {
    let ty: Type = parse_quote!(Box<Vec<u64>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(type_str(&inner), "u64");
}

#[test]
fn extract_through_arc_skip() {
    let ty: Type = parse_quote!(Arc<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(type_str(&inner), "bool");
}

#[test]
fn extract_through_nested_skips() {
    let ty: Type = parse_quote!(Box<Arc<Vec<f32>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(type_str(&inner), "f32");
}

#[test]
fn extract_skip_present_but_target_absent() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(type_str(&inner), "Box < String >");
}

#[test]
fn extract_reference_type_not_extracted() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(type_str(&inner), "& str");
}

#[test]
fn extract_tuple_type_not_extracted() {
    let ty: Type = parse_quote!((i32, u32));
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(type_str(&inner), "(i32 , u32)");
}

// =====================================================================
// 3. filter_inner_type — unwrapping containers
// =====================================================================

#[test]
fn filter_box_string() {
    let ty: Type = parse_quote!(Box<String>);
    let f = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(type_str(&f), "String");
}

#[test]
fn filter_arc_i32() {
    let ty: Type = parse_quote!(Arc<i32>);
    let f = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(type_str(&f), "i32");
}

#[test]
fn filter_nested_box_arc() {
    let ty: Type = parse_quote!(Box<Arc<u8>>);
    let f = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(type_str(&f), "u8");
}

#[test]
fn filter_not_in_skip_set_unchanged() {
    let ty: Type = parse_quote!(Vec<String>);
    let f = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(type_str(&f), "Vec < String >");
}

#[test]
fn filter_plain_type_unchanged() {
    let ty: Type = parse_quote!(u64);
    let f = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(type_str(&f), "u64");
}

#[test]
fn filter_empty_skip_set() {
    let ty: Type = parse_quote!(Box<i32>);
    let f = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(type_str(&f), "Box < i32 >");
}

#[test]
fn filter_reference_unchanged() {
    let ty: Type = parse_quote!(&str);
    let f = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(type_str(&f), "& str");
}

#[test]
fn filter_tuple_unchanged() {
    let ty: Type = parse_quote!((i32, bool));
    let f = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(type_str(&f), "(i32 , bool)");
}

#[test]
fn filter_three_layers() {
    let ty: Type = parse_quote!(Rc<Box<Arc<f64>>>);
    let f = filter_inner_type(&ty, &skip(&["Rc", "Box", "Arc"]));
    assert_eq!(type_str(&f), "f64");
}

#[test]
fn filter_stops_at_non_skip_container() {
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    let f = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(type_str(&f), "Vec < i32 >");
}

// =====================================================================
// 4. wrap_leaf_type — wrapping in adze::WithLeaf
// =====================================================================

#[test]
fn wrap_simple_type() {
    let ty: Type = parse_quote!(String);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(type_str(&w), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_i32() {
    let ty: Type = parse_quote!(i32);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(type_str(&w), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_vec_skipped() {
    let ty: Type = parse_quote!(Vec<String>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(type_str(&w), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_skipped() {
    let ty: Type = parse_quote!(Option<i32>);
    let w = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(type_str(&w), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_vec_option_both_skipped() {
    let ty: Type = parse_quote!(Vec<Option<u8>>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(type_str(&w), "Vec < Option < adze :: WithLeaf < u8 > > >");
}

#[test]
fn wrap_not_in_skip_wraps_whole() {
    let ty: Type = parse_quote!(Vec<String>);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(type_str(&w), "adze :: WithLeaf < Vec < String > >");
}

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&str);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(type_str(&w), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(type_str(&w), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_result_both_args_when_skipped() {
    let ty: Type = parse_quote!(Result<String, i32>);
    let w = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        type_str(&w),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_nested_skip_chain() {
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    let w = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(type_str(&w), "Option < Vec < adze :: WithLeaf < bool > > >");
}

// =====================================================================
// 5. NameValueExpr parsing
// =====================================================================

#[test]
fn nve_string_literal() {
    let nve: NameValueExpr = parse_str("key = \"hello\"").unwrap();
    assert_eq!(nve.path.to_string(), "key");
}

#[test]
fn nve_integer_literal() {
    let nve: NameValueExpr = parse_str("precedence = 5").unwrap();
    assert_eq!(nve.path.to_string(), "precedence");
}

#[test]
fn nve_bool_literal() {
    let nve: NameValueExpr = parse_str("enabled = true").unwrap();
    assert_eq!(nve.path.to_string(), "enabled");
}

#[test]
fn nve_path_expression() {
    let nve: NameValueExpr = parse_str("mode = Mode::Fast").unwrap();
    assert_eq!(nve.path.to_string(), "mode");
}

#[test]
fn nve_negative_number() {
    let nve: NameValueExpr = parse_str("offset = -1").unwrap();
    assert_eq!(nve.path.to_string(), "offset");
}

#[test]
fn nve_underscore_ident() {
    let nve: NameValueExpr = parse_str("my_field = 42").unwrap();
    assert_eq!(nve.path.to_string(), "my_field");
}

#[test]
fn nve_float_literal() {
    let nve: NameValueExpr = parse_str("ratio = 3.14").unwrap();
    assert_eq!(nve.path.to_string(), "ratio");
}

#[test]
fn nve_char_literal() {
    let nve: NameValueExpr = parse_str("sep = '/'").unwrap();
    assert_eq!(nve.path.to_string(), "sep");
}

// =====================================================================
// 6. FieldThenParams parsing
// =====================================================================

#[test]
fn ftp_type_only() {
    let ftp: FieldThenParams = parse_str("i32").unwrap();
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_type_with_one_param() {
    let ftp: FieldThenParams = parse_str("String, name = \"test\"").unwrap();
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "name");
}

#[test]
fn ftp_type_with_two_params() {
    let ftp: FieldThenParams = parse_str("bool, x = 1, y = 2").unwrap();
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "x");
    assert_eq!(ftp.params[1].path.to_string(), "y");
}

#[test]
fn ftp_generic_type() {
    let ftp: FieldThenParams = parse_str("Vec<i32>").unwrap();
    assert!(ftp.params.is_empty());
    let ty_str = ftp.field.ty.to_token_stream().to_string();
    assert_eq!(ty_str, "Vec < i32 >");
}

#[test]
fn ftp_generic_type_with_params() {
    let ftp: FieldThenParams = parse_str("Option<String>, default = \"none\"").unwrap();
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "default");
}

#[test]
fn ftp_qualified_path_type() {
    let ftp: FieldThenParams = parse_str("std::string::String").unwrap();
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_three_params() {
    let ftp: FieldThenParams = parse_str("u8, a = 1, b = 2, c = 3").unwrap();
    assert_eq!(ftp.params.len(), 3);
    assert_eq!(ftp.params[2].path.to_string(), "c");
}

// =====================================================================
// 7. Edge cases
// =====================================================================

#[test]
fn parse_str_never_type() {
    let ty: Type = parse_str("!").unwrap();
    assert_eq!(type_str(&ty), "!");
}

#[test]
fn parse_str_raw_pointer() {
    let ty: Type = parse_str("*const u8").unwrap();
    assert_eq!(type_str(&ty), "* const u8");
}

#[test]
fn parse_str_dyn_trait() {
    let ty: Type = parse_str("Box<dyn Send>").unwrap();
    assert_eq!(type_str(&ty), "Box < dyn Send >");
}

#[test]
fn parse_str_impl_trait() {
    let ty: Type = parse_str("impl Iterator<Item = i32>").unwrap();
    assert_eq!(type_str(&ty), "impl Iterator < Item = i32 >");
}

#[test]
fn parse_str_lifetime_ref() {
    let ty: Type = parse_str("&'a str").unwrap();
    assert_eq!(type_str(&ty), "& 'a str");
}

#[test]
fn parse_str_nested_tuple() {
    let ty: Type = parse_str("((i32, i32), (u8, u8))").unwrap();
    assert_eq!(type_str(&ty), "((i32 , i32) , (u8 , u8))");
}

#[test]
fn extract_on_impl_trait_not_extracted() {
    let ty: Type = parse_str("impl Clone").unwrap();
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(type_str(&inner), type_str(&ty));
}

#[test]
fn filter_on_never_type_unchanged() {
    let ty: Type = parse_str("!").unwrap();
    let f = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(type_str(&f), "!");
}

#[test]
fn wrap_unit_type() {
    let ty: Type = parse_quote!(());
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(type_str(&w), "adze :: WithLeaf < () >");
}

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, bool));
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(type_str(&w), "adze :: WithLeaf < (i32 , bool) >");
}

// =====================================================================
// 8. Property: type functions preserve non-container types
// =====================================================================

#[test]
fn extract_preserves_plain_u8() {
    let ty: Type = parse_quote!(u8);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(type_str(&inner), "u8");
}

#[test]
fn extract_preserves_plain_bool() {
    let ty: Type = parse_quote!(bool);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc"]));
    assert!(!ok);
    assert_eq!(type_str(&inner), "bool");
}

#[test]
fn filter_preserves_plain_string() {
    let ty: Type = parse_quote!(String);
    let f = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(type_str(&f), "String");
}

#[test]
fn filter_preserves_qualified_path() {
    let ty: Type = parse_quote!(std::string::String);
    let f = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(type_str(&f), "std :: string :: String");
}

#[test]
fn wrap_preserves_identity_through_skip() {
    // Vec is skipped, inner String gets wrapped — the Vec container stays.
    let ty: Type = parse_quote!(Vec<String>);
    let w = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert!(type_str(&w).starts_with("Vec"));
    assert!(type_str(&w).contains("adze :: WithLeaf < String >"));
}

#[test]
fn extract_non_generic_path_not_extracted() {
    let ty: Type = parse_quote!(MyStruct);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(type_str(&inner), "MyStruct");
}

#[test]
fn filter_non_generic_path_unchanged() {
    let ty: Type = parse_quote!(MyStruct);
    let f = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(type_str(&f), "MyStruct");
}

// =====================================================================
// 9. Additional extraction / filter / wrap combinations
// =====================================================================

#[test]
fn extract_hashmap_as_target() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "HashMap", &skip(&[]));
    assert!(ok);
    // extracts the first generic argument
    assert_eq!(type_str(&inner), "String");
}

#[test]
fn extract_through_single_skip_to_option() {
    let ty: Type = parse_quote!(Rc<Option<u16>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Rc"]));
    assert!(ok);
    assert_eq!(type_str(&inner), "u16");
}

#[test]
fn filter_rc_in_skip_set() {
    let ty: Type = parse_quote!(Rc<Vec<u8>>);
    let f = filter_inner_type(&ty, &skip(&["Rc"]));
    assert_eq!(type_str(&f), "Vec < u8 >");
}

#[test]
fn wrap_option_vec_chain_all_skipped() {
    let ty: Type = parse_quote!(Option<Vec<f64>>);
    let w = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(type_str(&w), "Option < Vec < adze :: WithLeaf < f64 > > >");
}

#[test]
fn wrap_hashmap_not_skipped() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let w = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        type_str(&w),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

#[test]
fn wrap_hashmap_skipped_wraps_both_args() {
    let ty: Type = parse_quote!(HashMap<String, Vec<u8>>);
    let w = wrap_leaf_type(&ty, &skip(&["HashMap"]));
    assert_eq!(
        type_str(&w),
        "HashMap < adze :: WithLeaf < String > , adze :: WithLeaf < Vec < u8 > > >"
    );
}

#[test]
fn wrap_hashmap_and_vec_skipped() {
    let ty: Type = parse_quote!(HashMap<String, Vec<u8>>);
    let w = wrap_leaf_type(&ty, &skip(&["HashMap", "Vec"]));
    assert_eq!(
        type_str(&w),
        "HashMap < adze :: WithLeaf < String > , Vec < adze :: WithLeaf < u8 > > >"
    );
}

// =====================================================================
// 10. NameValueExpr expression diversity
// =====================================================================

#[test]
fn nve_array_expression() {
    let nve: NameValueExpr = parse_str("data = [1, 2, 3]").unwrap();
    assert_eq!(nve.path.to_string(), "data");
}

#[test]
fn nve_tuple_expression() {
    let nve: NameValueExpr = parse_str("pair = (1, 2)").unwrap();
    assert_eq!(nve.path.to_string(), "pair");
}

#[test]
fn nve_closure_expression() {
    let nve: NameValueExpr = parse_str("func = |x| x + 1").unwrap();
    assert_eq!(nve.path.to_string(), "func");
}

#[test]
fn nve_block_expression() {
    let nve: NameValueExpr = parse_str("val = { 1 + 2 }").unwrap();
    assert_eq!(nve.path.to_string(), "val");
}

// =====================================================================
// 11. Composability: extract then filter, filter then wrap
// =====================================================================

#[test]
fn extract_then_wrap() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let w = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(type_str(&w), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_wrap() {
    let ty: Type = parse_quote!(Box<i32>);
    let f = filter_inner_type(&ty, &skip(&["Box"]));
    let w = wrap_leaf_type(&f, &skip(&[]));
    assert_eq!(type_str(&w), "adze :: WithLeaf < i32 >");
}

#[test]
fn extract_filter_wrap_chain() {
    let ty: Type = parse_quote!(Arc<Vec<Box<u8>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Arc"]));
    assert!(ok);
    // inner is Box<u8>
    let f = filter_inner_type(&inner, &skip(&["Box"]));
    assert_eq!(type_str(&f), "u8");
    let w = wrap_leaf_type(&f, &skip(&[]));
    assert_eq!(type_str(&w), "adze :: WithLeaf < u8 >");
}

#[test]
fn double_filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Arc<String>>);
    let f = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(type_str(&f), "String");
    let w = wrap_leaf_type(&f, &skip(&[]));
    assert_eq!(type_str(&w), "adze :: WithLeaf < String >");
}

// =====================================================================
// 12. FieldThenParams field type checking
// =====================================================================

#[test]
fn ftp_reference_type() {
    let ftp: FieldThenParams = parse_str("&str").unwrap();
    assert!(ftp.params.is_empty());
    assert_eq!(type_str(&ftp.field.ty), "& str");
}

#[test]
fn ftp_tuple_type() {
    let ftp: FieldThenParams = parse_str("(i32, bool)").unwrap();
    assert!(ftp.params.is_empty());
    assert_eq!(type_str(&ftp.field.ty), "(i32 , bool)");
}

#[test]
fn ftp_nested_generic_with_param() {
    let ftp: FieldThenParams = parse_str("Vec<Option<i32>>, label = \"items\"").unwrap();
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(type_str(&ftp.field.ty), "Vec < Option < i32 > >");
}

// =====================================================================
// 13. parse_str failure cases (ensure invalid syntax is rejected)
// =====================================================================

#[test]
fn parse_str_invalid_type_fails() {
    assert!(parse_str::<Type>("123abc").is_err());
}

#[test]
fn nve_missing_equals_fails() {
    assert!(parse_str::<NameValueExpr>("key \"value\"").is_err());
}

#[test]
fn nve_missing_value_fails() {
    assert!(parse_str::<NameValueExpr>("key =").is_err());
}

// =====================================================================
// 14. Additional miscellaneous
// =====================================================================

#[test]
fn extract_box_directly_as_target() {
    let ty: Type = parse_quote!(Box<f32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(type_str(&inner), "f32");
}

#[test]
fn wrap_deeply_nested_skip_chain() {
    let ty: Type = parse_quote!(Option<Vec<Option<i32>>>);
    let w = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        type_str(&w),
        "Option < Vec < Option < adze :: WithLeaf < i32 > > > >"
    );
}

#[test]
fn filter_only_outermost_when_inner_not_in_skip() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let f = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(type_str(&f), "Option < i32 >");
}

#[test]
fn extract_vec_of_tuples() {
    let ty: Type = parse_quote!(Vec<(i32, String)>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(type_str(&inner), "(i32 , String)");
}

#[test]
fn nve_reserved_like_ident() {
    // `r#gen` is a raw identifier for the reserved keyword `gen`
    let nve: NameValueExpr = parse_str("r#gen = 42").unwrap();
    assert_eq!(nve.path.to_string(), "r#gen");
}
