//! Advanced proc-macro attribute pattern tests for adze macros.
//!
//! Tests attribute parsing helpers, type extraction, type wrapping, and edge
//! cases from `adze_common` (re-exported from `adze_common_syntax_core`).
//! Since proc macros cannot be invoked directly in unit tests, we exercise the
//! shared parsing and type-manipulation utilities that the macros rely on.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{Token, Type, parse_quote};

// ═══════════════════════════════════════════════════════════════════════════
// Helper
// ═══════════════════════════════════════════════════════════════════════════

fn ty(s: &str) -> Type {
    syn::parse_str::<Type>(s).unwrap_or_else(|e| panic!("failed to parse type `{s}`: {e}"))
}

fn tokens(t: &Type) -> String {
    t.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. NameValueExpr parsing  (tests 1–15)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn nve_string_literal() {
    let nve: NameValueExpr = parse_quote!(name = "hello");
    assert_eq!(nve.path.to_string(), "name");
}

#[test]
fn nve_integer_literal() {
    let nve: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nve.path.to_string(), "precedence");
}

#[test]
fn nve_bool_true() {
    let nve: NameValueExpr = parse_quote!(non_empty = true);
    assert_eq!(nve.path.to_string(), "non_empty");
}

#[test]
fn nve_bool_false() {
    let nve: NameValueExpr = parse_quote!(visible = false);
    assert_eq!(nve.path.to_string(), "visible");
}

#[test]
fn nve_raw_string_literal() {
    let nve: NameValueExpr = parse_quote!(pattern = r"\d+");
    assert_eq!(nve.path.to_string(), "pattern");
}

#[test]
fn nve_closure_value() {
    let nve: NameValueExpr = parse_quote!(transform = |v| v.parse().unwrap());
    assert_eq!(nve.path.to_string(), "transform");
}

#[test]
fn nve_negative_integer() {
    let nve: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nve.path.to_string(), "offset");
}

#[test]
fn nve_float_literal() {
    let nve: NameValueExpr = parse_quote!(weight = 3.14);
    assert_eq!(nve.path.to_string(), "weight");
}

#[test]
fn nve_char_literal() {
    let nve: NameValueExpr = parse_quote!(delimiter = 'x');
    assert_eq!(nve.path.to_string(), "delimiter");
}

#[test]
fn nve_path_value() {
    let nve: NameValueExpr = parse_quote!(kind = MyEnum::Variant);
    assert_eq!(nve.path.to_string(), "kind");
}

#[test]
fn nve_array_value() {
    let nve: NameValueExpr = parse_quote!(sizes = [1, 2, 3]);
    assert_eq!(nve.path.to_string(), "sizes");
}

#[test]
fn nve_tuple_value() {
    let nve: NameValueExpr = parse_quote!(pair = (1, 2));
    assert_eq!(nve.path.to_string(), "pair");
}

#[test]
fn nve_underscore_ident() {
    let nve: NameValueExpr = parse_quote!(my_long_param = 99);
    assert_eq!(nve.path.to_string(), "my_long_param");
}

#[test]
fn nve_punctuated_multiple() {
    let params: Punctuated<NameValueExpr, Token![,]> = parse_quote!(a = 1, b = "two", c = true);
    assert_eq!(params.len(), 3);
    assert_eq!(params[0].path.to_string(), "a");
    assert_eq!(params[1].path.to_string(), "b");
    assert_eq!(params[2].path.to_string(), "c");
}

#[test]
fn nve_single_in_punctuated() {
    let params: Punctuated<NameValueExpr, Token![,]> = parse_quote!(only = 1);
    assert_eq!(params.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. FieldThenParams parsing  (tests 16–27)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ftp_bare_type() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_type_with_one_param() {
    let ftp: FieldThenParams = parse_quote!(i32, min = 0);
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "min");
}

#[test]
fn ftp_type_with_two_params() {
    let ftp: FieldThenParams = parse_quote!(u64, min = 0, max = 100);
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[1].path.to_string(), "max");
}

#[test]
fn ftp_generic_type() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>);
    assert!(ftp.params.is_empty());
    assert_eq!(tokens(&ftp.field.ty), "Vec < String >");
}

#[test]
fn ftp_reference_type() {
    let ftp: FieldThenParams = parse_quote!(&str);
    assert!(ftp.params.is_empty());
    assert_eq!(tokens(&ftp.field.ty), "& str");
}

#[test]
fn ftp_option_type_with_params() {
    let ftp: FieldThenParams = parse_quote!(Option<u32>, default = 0);
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(tokens(&ftp.field.ty), "Option < u32 >");
}

#[test]
fn ftp_box_type() {
    let ftp: FieldThenParams = parse_quote!(Box<Expr>);
    assert!(ftp.params.is_empty());
    assert_eq!(tokens(&ftp.field.ty), "Box < Expr >");
}

#[test]
fn ftp_unit_type() {
    let ftp: FieldThenParams = parse_quote!(());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_tuple_type() {
    let ftp: FieldThenParams = parse_quote!((i32, u32));
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_array_type() {
    let ftp: FieldThenParams = parse_quote!([u8; 4]);
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_qualified_path_type() {
    let ftp: FieldThenParams = parse_quote!(std::string::String);
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_three_params() {
    let ftp: FieldThenParams = parse_quote!(f64, min = 0.0, max = 1.0, step = 0.1);
    assert_eq!(ftp.params.len(), 3);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. try_extract_inner_type  (tests 28–52)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extract_vec_string() {
    let t = ty("Vec<String>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(tokens(&inner), "String");
}

#[test]
fn extract_option_i32() {
    let t = ty("Option<i32>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(tokens(&inner), "i32");
}

#[test]
fn extract_not_matching() {
    let t = ty("HashMap<String, i32>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(tokens(&inner), tokens(&t));
}

#[test]
fn extract_skip_box_to_vec() {
    let t = ty("Box<Vec<u8>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(tokens(&inner), "u8");
}

#[test]
fn extract_skip_arc_to_option() {
    let t = ty("Arc<Option<bool>>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(tokens(&inner), "bool");
}

#[test]
fn extract_double_skip() {
    let t = ty("Box<Arc<Vec<f64>>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(tokens(&inner), "f64");
}

#[test]
fn extract_skip_no_match_inside() {
    let t = ty("Box<String>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(tokens(&inner), "Box < String >");
}

#[test]
fn extract_reference_type_unchanged() {
    let t = ty("&str");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(tokens(&inner), "& str");
}

#[test]
fn extract_tuple_type_unchanged() {
    let t = ty("(i32, u32)");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(tokens(&inner), "(i32 , u32)");
}

#[test]
fn extract_plain_type_unchanged() {
    let t = ty("String");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(tokens(&inner), "String");
}

#[test]
fn extract_option_of_vec() {
    let t = ty("Option<Vec<u8>>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(tokens(&inner), "Vec < u8 >");
}

#[test]
fn extract_vec_of_option() {
    let t = ty("Vec<Option<u8>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(tokens(&inner), "Option < u8 >");
}

#[test]
fn extract_nested_option_skip_option() {
    let t = ty("Option<Option<i32>>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(tokens(&inner), "Option < i32 >");
}

#[test]
fn extract_cow_not_in_skip() {
    let t = ty("Cow<str>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(tokens(&inner), "Cow < str >");
}

#[test]
fn extract_rc_in_skip() {
    let t = ty("Rc<Vec<char>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Rc"]));
    assert!(ok);
    assert_eq!(tokens(&inner), "char");
}

#[test]
fn extract_with_qualified_path() {
    let t = ty("std::vec::Vec<u32>");
    // The last segment is `Vec` so this should match
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(tokens(&inner), "u32");
}

#[test]
fn extract_custom_wrapper() {
    let t = ty("MyWrapper<Inner>");
    let (inner, ok) = try_extract_inner_type(&t, "MyWrapper", &skip(&[]));
    assert!(ok);
    assert_eq!(tokens(&inner), "Inner");
}

#[test]
fn extract_empty_skip_set() {
    let t = ty("Box<i32>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(tokens(&inner), "Box < i32 >");
}

#[test]
fn extract_slice_type_unchanged() {
    let t = ty("[u8]");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(tokens(&inner), "[u8]");
}

#[test]
fn extract_array_type_unchanged() {
    let t = ty("[u8; 4]");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(tokens(&inner), "[u8 ; 4]");
}

#[test]
fn extract_fn_pointer_unchanged() {
    let t = ty("fn(i32) -> bool");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(tokens(&inner), tokens(&t));
}

#[test]
fn extract_mutable_ref_unchanged() {
    let t = ty("&mut String");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(tokens(&inner), "& mut String");
}

#[test]
fn extract_lifetime_ref_unchanged() {
    let t = ty("&'static str");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(tokens(&inner), "& 'static str");
}

#[test]
fn extract_triple_skip_chain() {
    let t = ty("Box<Arc<Rc<Option<u8>>>>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&["Box", "Arc", "Rc"]));
    assert!(ok);
    assert_eq!(tokens(&inner), "u8");
}

#[test]
fn extract_target_is_first_not_skip() {
    // Vec is the target, not in skip. Direct match.
    let t = ty("Vec<bool>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(tokens(&inner), "bool");
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. filter_inner_type  (tests 53–66)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn filter_box_string() {
    let t = ty("Box<String>");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tokens(&f), "String");
}

#[test]
fn filter_arc_i32() {
    let t = ty("Arc<i32>");
    let f = filter_inner_type(&t, &skip(&["Arc"]));
    assert_eq!(tokens(&f), "i32");
}

#[test]
fn filter_double_unwrap() {
    let t = ty("Box<Arc<u64>>");
    let f = filter_inner_type(&t, &skip(&["Box", "Arc"]));
    assert_eq!(tokens(&f), "u64");
}

#[test]
fn filter_triple_unwrap() {
    let t = ty("Box<Arc<Rc<bool>>>");
    let f = filter_inner_type(&t, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(tokens(&f), "bool");
}

#[test]
fn filter_not_in_skip_unchanged() {
    let t = ty("Vec<String>");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tokens(&f), "Vec < String >");
}

#[test]
fn filter_plain_type_unchanged() {
    let t = ty("String");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tokens(&f), "String");
}

#[test]
fn filter_empty_skip_unchanged() {
    let t = ty("Box<i32>");
    let f = filter_inner_type(&t, &skip(&[]));
    assert_eq!(tokens(&f), "Box < i32 >");
}

#[test]
fn filter_reference_unchanged() {
    let t = ty("&str");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tokens(&f), "& str");
}

#[test]
fn filter_tuple_unchanged() {
    let t = ty("(i32, u32)");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tokens(&f), "(i32 , u32)");
}

#[test]
fn filter_qualified_box() {
    // Only the last segment is checked, so `std::boxed::Box` matches "Box"
    let t = ty("std::boxed::Box<f32>");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tokens(&f), "f32");
}

#[test]
fn filter_stops_at_non_skip() {
    // Box is in skip, but Vec is not—so we unwrap Box then stop.
    let t = ty("Box<Vec<u8>>");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tokens(&f), "Vec < u8 >");
}

#[test]
fn filter_array_type_unchanged() {
    let t = ty("[u8; 16]");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tokens(&f), "[u8 ; 16]");
}

#[test]
fn filter_fn_pointer_unchanged() {
    let t = ty("fn() -> i32");
    let f = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tokens(&f), tokens(&t));
}

#[test]
fn filter_custom_wrapper() {
    let t = ty("MyBox<Inner>");
    let f = filter_inner_type(&t, &skip(&["MyBox"]));
    assert_eq!(tokens(&f), "Inner");
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. wrap_leaf_type  (tests 67–83)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn wrap_plain_string() {
    let t = ty("String");
    let w = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(tokens(&w), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_plain_i32() {
    let t = ty("i32");
    let w = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(tokens(&w), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_vec_skipped() {
    let t = ty("Vec<String>");
    let w = wrap_leaf_type(&t, &skip(&["Vec"]));
    assert_eq!(tokens(&w), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_skipped() {
    let t = ty("Option<String>");
    let w = wrap_leaf_type(&t, &skip(&["Option"]));
    assert_eq!(tokens(&w), "Option < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_vec_option_both_skipped() {
    let t = ty("Vec<Option<i32>>");
    let w = wrap_leaf_type(&t, &skip(&["Vec", "Option"]));
    assert_eq!(tokens(&w), "Vec < Option < adze :: WithLeaf < i32 > > >");
}

#[test]
fn wrap_option_vec_both_skipped() {
    let t = ty("Option<Vec<u8>>");
    let w = wrap_leaf_type(&t, &skip(&["Vec", "Option"]));
    assert_eq!(tokens(&w), "Option < Vec < adze :: WithLeaf < u8 > > >");
}

#[test]
fn wrap_not_in_skip_wraps_entire() {
    let t = ty("Vec<i32>");
    let w = wrap_leaf_type(&t, &skip(&["Option"]));
    assert_eq!(tokens(&w), "adze :: WithLeaf < Vec < i32 > >");
}

#[test]
fn wrap_reference_type() {
    let t = ty("&str");
    let w = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(tokens(&w), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_tuple_type() {
    let t = ty("(i32, u32)");
    let w = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(tokens(&w), "adze :: WithLeaf < (i32 , u32) >");
}

#[test]
fn wrap_array_type() {
    let t = ty("[u8; 4]");
    let w = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(tokens(&w), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_result_both_args() {
    let t = ty("Result<String, i32>");
    let w = wrap_leaf_type(&t, &skip(&["Result"]));
    assert_eq!(
        tokens(&w),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_empty_skip_wraps_everything() {
    let t = ty("Option<Vec<String>>");
    let w = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(tokens(&w), "adze :: WithLeaf < Option < Vec < String > > >");
}

#[test]
fn wrap_qualified_vec_skipped() {
    // Last segment is Vec, so it matches the skip set
    let t = ty("std::vec::Vec<u32>");
    let w = wrap_leaf_type(&t, &skip(&["Vec"]));
    assert_eq!(tokens(&w), "std :: vec :: Vec < adze :: WithLeaf < u32 > >");
}

#[test]
fn wrap_custom_skip_type() {
    let t = ty("MyContainer<Foo>");
    let w = wrap_leaf_type(&t, &skip(&["MyContainer"]));
    assert_eq!(tokens(&w), "MyContainer < adze :: WithLeaf < Foo > >");
}

#[test]
fn wrap_deeply_nested_skip() {
    let t = ty("Vec<Option<Vec<bool>>>");
    let w = wrap_leaf_type(&t, &skip(&["Vec", "Option"]));
    assert_eq!(
        tokens(&w),
        "Vec < Option < Vec < adze :: WithLeaf < bool > > > >"
    );
}

#[test]
fn wrap_hashmap_in_skip_wraps_both_args() {
    let t = ty("HashMap<String, Vec<i32>>");
    let w = wrap_leaf_type(&t, &skip(&["HashMap", "Vec"]));
    assert_eq!(
        tokens(&w),
        "HashMap < adze :: WithLeaf < String > , Vec < adze :: WithLeaf < i32 > > >"
    );
}

#[test]
fn wrap_box_not_in_skip() {
    let t = ty("Box<i32>");
    let w = wrap_leaf_type(&t, &skip(&["Vec", "Option"]));
    assert_eq!(tokens(&w), "adze :: WithLeaf < Box < i32 > >");
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Combined / cross-function tests  (tests 84–90+)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn extract_then_wrap() {
    let t = ty("Vec<String>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(tokens(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_wrap() {
    let t = ty("Box<String>");
    let filtered = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tokens(&filtered), "String");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(tokens(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn extract_through_skip_then_filter() {
    let t = ty("Arc<Vec<Box<u32>>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(tokens(&inner), "Box < u32 >");
    let filtered = filter_inner_type(&inner, &skip(&["Box"]));
    assert_eq!(tokens(&filtered), "u32");
}

#[test]
fn ftp_field_type_extraction() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>, non_empty = true);
    let (inner, ok) = try_extract_inner_type(&ftp.field.ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(tokens(&inner), "String");
    assert_eq!(ftp.params[0].path.to_string(), "non_empty");
}

#[test]
fn ftp_field_type_wrap() {
    let ftp: FieldThenParams = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ftp.field.ty, &skip(&["Option"]));
    assert_eq!(tokens(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn extract_returns_original_for_non_generic() {
    let t = ty("u64");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(tokens(&inner), "u64");
}

#[test]
fn wrap_preserves_never_type() {
    let t = ty("!");
    let w = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(tokens(&w), "adze :: WithLeaf < ! >");
}

#[test]
fn extract_dyn_trait_unchanged() {
    let t: Type = parse_quote!(dyn Iterator<Item = i32>);
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(tokens(&inner), tokens(&t));
}

#[test]
fn nve_method_call_value() {
    let nve: NameValueExpr = parse_quote!(factory = String::new());
    assert_eq!(nve.path.to_string(), "factory");
}

#[test]
fn ftp_with_string_param_value() {
    let ftp: FieldThenParams = parse_quote!(String, label = "name");
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "label");
}
