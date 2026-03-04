//! Comprehensive tests for adze-common-syntax-core re-exported APIs:
//! `try_extract_inner_type`, `filter_inner_type`, `wrap_leaf_type`,
//! `NameValueExpr`, and `FieldThenParams`.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn ty(s: &str) -> Type {
    syn::parse_str::<Type>(s).unwrap()
}

fn tok(t: &Type) -> String {
    t.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// ===========================================================================
// try_extract_inner_type
// ===========================================================================

#[test]
fn extract_option_string() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<String>"), "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(tok(&inner), "String");
}

#[test]
fn extract_vec_i32() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<i32>"), "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(tok(&inner), "i32");
}

#[test]
fn extract_target_mismatch_returns_original() {
    let t = ty("Vec<u8>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(tok(&inner), tok(&t));
}

#[test]
fn extract_primitive_no_generics_returns_original() {
    let t = ty("bool");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(tok(&inner), "bool");
}

#[test]
fn extract_skip_over_box_to_vec() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Vec<f64>>"), "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(tok(&inner), "f64");
}

#[test]
fn extract_skip_over_arc_to_option() {
    let (inner, ok) = try_extract_inner_type(&ty("Arc<Option<u16>>"), "Option", &skip(&["Arc"]));
    assert!(ok);
    assert_eq!(tok(&inner), "u16");
}

#[test]
fn extract_double_skip_box_arc_to_vec() {
    let (inner, ok) =
        try_extract_inner_type(&ty("Box<Arc<Vec<String>>>"), "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    assert_eq!(tok(&inner), "String");
}

#[test]
fn extract_skip_not_containing_target() {
    let t = ty("Box<String>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(tok(&inner), tok(&t));
}

#[test]
fn extract_non_path_type_reference() {
    let t: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(tok(&inner), "& str");
}

#[test]
fn extract_non_path_type_tuple() {
    let t: Type = parse_quote!((i32, u32));
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(tok(&inner), "(i32 , u32)");
}

#[test]
fn extract_nested_option_option() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<Option<u8>>"), "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(tok(&inner), "Option < u8 >");
}

#[test]
fn extract_empty_skip_set() {
    let t = ty("Box<Vec<i32>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(tok(&inner), tok(&t));
}

#[test]
fn extract_target_is_skip_type() {
    // Box is both in skip_over and is the target — target match takes precedence
    let (inner, ok) = try_extract_inner_type(&ty("Box<u64>"), "Box", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(tok(&inner), "u64");
}

#[test]
fn extract_qualified_path_last_segment_matches() {
    let t = ty("std::option::Option<bool>");
    let (inner, ok) = try_extract_inner_type(&t, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(tok(&inner), "bool");
}

#[test]
fn extract_deeply_nested_three_skips() {
    let t = ty("A<B<C<Vec<u8>>>>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["A", "B", "C"]));
    assert!(ok);
    assert_eq!(tok(&inner), "u8");
}

#[test]
fn extract_skip_outer_no_inner_target() {
    // A<String> where A is in skip and target is Vec — inner String is not Vec
    let t = ty("A<String>");
    let (inner, ok) = try_extract_inner_type(&t, "Vec", &skip(&["A"]));
    assert!(!ok);
    assert_eq!(tok(&inner), tok(&t));
}

#[test]
fn extract_option_with_complex_inner() {
    let (inner, ok) =
        try_extract_inner_type(&ty("Option<HashMap<String, i32>>"), "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(tok(&inner), "HashMap < String , i32 >");
}

// ===========================================================================
// filter_inner_type
// ===========================================================================

#[test]
fn filter_box_string() {
    let filtered = filter_inner_type(&ty("Box<String>"), &skip(&["Box"]));
    assert_eq!(tok(&filtered), "String");
}

#[test]
fn filter_arc_string() {
    let filtered = filter_inner_type(&ty("Arc<String>"), &skip(&["Arc"]));
    assert_eq!(tok(&filtered), "String");
}

#[test]
fn filter_double_box_arc() {
    let filtered = filter_inner_type(&ty("Box<Arc<u32>>"), &skip(&["Box", "Arc"]));
    assert_eq!(tok(&filtered), "u32");
}

#[test]
fn filter_not_in_skip_returns_original() {
    let t = ty("Vec<i32>");
    let filtered = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tok(&filtered), tok(&t));
}

#[test]
fn filter_empty_skip_returns_original() {
    let t = ty("Box<String>");
    let filtered = filter_inner_type(&t, &skip(&[]));
    assert_eq!(tok(&filtered), tok(&t));
}

#[test]
fn filter_primitive_returns_self() {
    let t = ty("i64");
    let filtered = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tok(&filtered), "i64");
}

#[test]
fn filter_non_path_returns_self() {
    let t: Type = parse_quote!(&[u8]);
    let filtered = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tok(&filtered), tok(&t));
}

#[test]
fn filter_three_layers() {
    let filtered = filter_inner_type(&ty("A<B<C<bool>>>"), &skip(&["A", "B", "C"]));
    assert_eq!(tok(&filtered), "bool");
}

#[test]
fn filter_stops_at_non_skip() {
    let filtered = filter_inner_type(&ty("Box<Vec<String>>"), &skip(&["Box"]));
    assert_eq!(tok(&filtered), "Vec < String >");
}

#[test]
fn filter_tuple_passthrough() {
    let t: Type = parse_quote!((u8, u16));
    let filtered = filter_inner_type(&t, &skip(&["Box"]));
    assert_eq!(tok(&filtered), "(u8 , u16)");
}

#[test]
fn filter_qualified_path() {
    let filtered = filter_inner_type(&ty("std::boxed::Box<u8>"), &skip(&["Box"]));
    assert_eq!(tok(&filtered), "u8");
}

// ===========================================================================
// wrap_leaf_type
// ===========================================================================

#[test]
fn wrap_plain_string() {
    let wrapped = wrap_leaf_type(&ty("String"), &skip(&[]));
    assert_eq!(tok(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_plain_i32() {
    let wrapped = wrap_leaf_type(&ty("i32"), &skip(&[]));
    assert_eq!(tok(&wrapped), "adze :: WithLeaf < i32 >");
}

#[test]
fn wrap_vec_string() {
    let wrapped = wrap_leaf_type(&ty("Vec<String>"), &skip(&["Vec"]));
    assert_eq!(tok(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_option_i32() {
    let wrapped = wrap_leaf_type(&ty("Option<i32>"), &skip(&["Option"]));
    assert_eq!(tok(&wrapped), "Option < adze :: WithLeaf < i32 > >");
}

#[test]
fn wrap_option_vec_nested() {
    let wrapped = wrap_leaf_type(&ty("Option<Vec<u8>>"), &skip(&["Option", "Vec"]));
    assert_eq!(tok(&wrapped), "Option < Vec < adze :: WithLeaf < u8 > > >");
}

#[test]
fn wrap_not_in_skip_wraps_whole() {
    let wrapped = wrap_leaf_type(&ty("Vec<u8>"), &skip(&["Option"]));
    assert_eq!(tok(&wrapped), "adze :: WithLeaf < Vec < u8 > >");
}

#[test]
fn wrap_empty_skip_wraps_whole() {
    let wrapped = wrap_leaf_type(&ty("Vec<u8>"), &skip(&[]));
    assert_eq!(tok(&wrapped), "adze :: WithLeaf < Vec < u8 > >");
}

#[test]
fn wrap_non_path_type() {
    let t: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(tok(&wrapped), "adze :: WithLeaf < & str >");
}

#[test]
fn wrap_result_both_args() {
    let wrapped = wrap_leaf_type(&ty("Result<String, i32>"), &skip(&["Result"]));
    assert_eq!(
        tok(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_deeply_nested_skip() {
    let wrapped = wrap_leaf_type(&ty("A<B<C<bool>>>"), &skip(&["A", "B", "C"]));
    assert_eq!(tok(&wrapped), "A < B < C < adze :: WithLeaf < bool > > > >");
}

#[test]
fn wrap_array_type() {
    let t: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(tok(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_tuple_type() {
    let t: Type = parse_quote!((u8, u16));
    let wrapped = wrap_leaf_type(&t, &skip(&[]));
    assert_eq!(tok(&wrapped), "adze :: WithLeaf < (u8 , u16) >");
}

#[test]
fn wrap_qualified_skip_type() {
    let wrapped = wrap_leaf_type(&ty("std::vec::Vec<bool>"), &skip(&["Vec"]));
    assert_eq!(
        tok(&wrapped),
        "std :: vec :: Vec < adze :: WithLeaf < bool > >"
    );
}

// ===========================================================================
// NameValueExpr parsing
// ===========================================================================

#[test]
fn nve_simple_string_value() {
    let nv: NameValueExpr = parse_quote!(name = "hello");
    assert_eq!(nv.path.to_string(), "name");
}

#[test]
fn nve_numeric_value() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path.to_string(), "precedence");
}

#[test]
fn nve_bool_value() {
    let nv: NameValueExpr = parse_quote!(flag = true);
    assert_eq!(nv.path.to_string(), "flag");
}

#[test]
fn nve_path_value() {
    let nv: NameValueExpr = parse_quote!(kind = MyEnum::Variant);
    assert_eq!(nv.path.to_string(), "kind");
}

#[test]
fn nve_negative_number() {
    let nv: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nv.path.to_string(), "offset");
}

// ===========================================================================
// FieldThenParams parsing
// ===========================================================================

#[test]
fn ftp_bare_type() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_type_with_one_param() {
    let ftp: FieldThenParams = parse_quote!(u32, name = "x");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "name");
}

#[test]
fn ftp_type_with_two_params() {
    let ftp: FieldThenParams = parse_quote!(Vec<u8>, name = "buf", size = 256);
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "name");
    assert_eq!(ftp.params[1].path.to_string(), "size");
}

#[test]
fn ftp_generic_type() {
    let ftp: FieldThenParams = parse_quote!(Option<String>);
    assert!(ftp.params.is_empty());
    let field_ty = &ftp.field.ty;
    assert_eq!(tok(field_ty), "Option < String >");
}

#[test]
fn ftp_three_params() {
    let ftp: FieldThenParams = parse_quote!(bool, a = 1, b = 2, c = 3);
    assert_eq!(ftp.params.len(), 3);
    assert_eq!(ftp.params[2].path.to_string(), "c");
}

// ===========================================================================
// Composition / cross-function tests
// ===========================================================================

#[test]
fn extract_then_wrap() {
    let s = skip(&["Box"]);
    let (inner, ok) = try_extract_inner_type(&ty("Box<Vec<String>>"), "Vec", &skip(&["Box"]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &s);
    assert_eq!(tok(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn filter_then_wrap() {
    let t = ty("Box<Arc<bool>>");
    let filtered = filter_inner_type(&t, &skip(&["Box", "Arc"]));
    assert_eq!(tok(&filtered), "bool");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(tok(&wrapped), "adze :: WithLeaf < bool >");
}

#[test]
fn wrap_then_filter_identity() {
    // wrap Vec<String> with Vec in skip, then filter Box (no-op)
    let wrapped = wrap_leaf_type(&ty("Vec<String>"), &skip(&["Vec"]));
    let filtered = filter_inner_type(&wrapped, &skip(&["Box"]));
    assert_eq!(tok(&filtered), tok(&wrapped));
}

#[test]
fn extract_option_vec_nested() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<Vec<String>>"), "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(tok(&inner), "Vec < String >");
}

#[test]
fn extract_with_skip_preserves_inner_nesting() {
    let (inner, ok) =
        try_extract_inner_type(&ty("Box<Option<Vec<u8>>>"), "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(tok(&inner), "Vec < u8 >");
}

#[test]
fn filter_single_layer_same_as_extract() {
    let t = ty("Box<f32>");
    let filtered = filter_inner_type(&t, &skip(&["Box"]));
    let (extracted, ok) = try_extract_inner_type(&t, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(tok(&filtered), tok(&extracted));
}
