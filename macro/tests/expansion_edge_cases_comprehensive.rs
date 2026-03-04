//! Comprehensive edge-case tests for the expansion helpers in adze-common.
//!
//! The macro crate delegates to `adze_common` (re-exported from
//! `adze_common_syntax_core`). These tests exercise `try_extract_inner_type`,
//! `filter_inner_type`, `wrap_leaf_type`, `NameValueExpr`, and
//! `FieldThenParams` with corner-case inputs.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn ty(s: &str) -> Type {
    syn::parse_str::<Type>(s).unwrap()
}

fn ts(t: &Type) -> String {
    t.to_token_stream().to_string()
}

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

// =====================================================================
// 1. try_extract_inner_type – basic extraction
// =====================================================================

#[test]
fn extract_vec_string() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<String>"), "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "String");
}

#[test]
fn extract_option_u32() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<u32>"), "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "u32");
}

#[test]
fn extract_miss_returns_original() {
    let orig = ty("HashMap<String, i32>");
    let (inner, ok) = try_extract_inner_type(&orig, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), ts(&orig));
}

// =====================================================================
// 2. try_extract_inner_type – skip-over containers
// =====================================================================

#[test]
fn extract_through_single_skip() {
    let (inner, ok) = try_extract_inner_type(&ty("Box<Vec<u8>>"), "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "u8");
}

#[test]
fn extract_through_double_skip() {
    let (inner, ok) = try_extract_inner_type(
        &ty("Arc<Box<Option<bool>>>"),
        "Option",
        &skip(&["Arc", "Box"]),
    );
    assert!(ok);
    assert_eq!(ts(&inner), "bool");
}

#[test]
fn extract_skip_without_target_returns_original() {
    let orig = ty("Box<String>");
    let (inner, ok) = try_extract_inner_type(&orig, "Vec", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ts(&inner), ts(&orig));
}

// =====================================================================
// 3. try_extract_inner_type – nested generics
// =====================================================================

#[test]
fn extract_vec_of_option() {
    let (inner, ok) = try_extract_inner_type(&ty("Vec<Option<i32>>"), "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Option < i32 >");
}

#[test]
fn extract_option_of_vec() {
    let (inner, ok) = try_extract_inner_type(&ty("Option<Vec<String>>"), "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Vec < String >");
}

// =====================================================================
// 4. try_extract_inner_type – non-path types
// =====================================================================

#[test]
fn extract_reference_type_unchanged() {
    let orig = ty("&str");
    let (inner, ok) = try_extract_inner_type(&orig, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), ts(&orig));
}

#[test]
fn extract_tuple_type_unchanged() {
    let orig = ty("(i32, u64)");
    let (inner, ok) = try_extract_inner_type(&orig, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), ts(&orig));
}

#[test]
fn extract_array_type_unchanged() {
    let orig = ty("[u8; 16]");
    let (inner, ok) = try_extract_inner_type(&orig, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ts(&inner), ts(&orig));
}

// =====================================================================
// 5. try_extract_inner_type – qualified / multi-segment paths
// =====================================================================

#[test]
fn extract_last_segment_match() {
    // `std::vec::Vec<i32>` – last segment is `Vec`
    let (inner, ok) = try_extract_inner_type(&ty("std::vec::Vec<i32>"), "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "i32");
}

#[test]
fn extract_qualified_skip() {
    // skip recognises "Box" from last segment
    let (inner, ok) =
        try_extract_inner_type(&ty("std::boxed::Box<Vec<f64>>"), "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "f64");
}

// =====================================================================
// 6. filter_inner_type
// =====================================================================

#[test]
fn filter_single_layer() {
    let filtered = filter_inner_type(&ty("Box<String>"), &skip(&["Box"]));
    assert_eq!(ts(&filtered), "String");
}

#[test]
fn filter_two_layers() {
    let filtered = filter_inner_type(&ty("Box<Arc<i32>>"), &skip(&["Box", "Arc"]));
    assert_eq!(ts(&filtered), "i32");
}

#[test]
fn filter_stops_at_non_skip() {
    let filtered = filter_inner_type(&ty("Box<Vec<u8>>"), &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Vec < u8 >");
}

#[test]
fn filter_empty_skip_set_noop() {
    let orig = ty("Rc<String>");
    let filtered = filter_inner_type(&orig, &skip(&[]));
    assert_eq!(ts(&filtered), ts(&orig));
}

#[test]
fn filter_non_path_type_noop() {
    let orig = ty("(bool, char)");
    let filtered = filter_inner_type(&orig, &skip(&["Box"]));
    assert_eq!(ts(&filtered), ts(&orig));
}

#[test]
fn filter_reference_type_noop() {
    let orig = ty("&mut String");
    let filtered = filter_inner_type(&orig, &skip(&["Box"]));
    assert_eq!(ts(&filtered), ts(&orig));
}

// =====================================================================
// 7. wrap_leaf_type
// =====================================================================

#[test]
fn wrap_plain_type() {
    let wrapped = wrap_leaf_type(&ty("String"), &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < String >");
}

#[test]
fn wrap_skips_vec_wraps_inner() {
    let wrapped = wrap_leaf_type(&ty("Vec<String>"), &skip(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < String > >");
}

#[test]
fn wrap_skips_nested_containers() {
    let wrapped = wrap_leaf_type(&ty("Option<Vec<i32>>"), &skip(&["Option", "Vec"]));
    assert_eq!(ts(&wrapped), "Option < Vec < adze :: WithLeaf < i32 > > >");
}

#[test]
fn wrap_non_path_type() {
    let wrapped = wrap_leaf_type(&ty("[u8; 4]"), &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_result_wraps_both_args() {
    let wrapped = wrap_leaf_type(&ty("Result<String, i32>"), &skip(&["Result"]));
    assert_eq!(
        ts(&wrapped),
        "Result < adze :: WithLeaf < String > , adze :: WithLeaf < i32 > >"
    );
}

#[test]
fn wrap_reference_type() {
    let wrapped = wrap_leaf_type(&ty("&str"), &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < & str >");
}

// =====================================================================
// 8. NameValueExpr parsing
// =====================================================================

#[test]
fn name_value_simple_string() {
    let nv: NameValueExpr = parse_quote!(language = "rust");
    assert_eq!(nv.path.to_string(), "language");
}

#[test]
fn name_value_integer_literal() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path.to_string(), "precedence");
    assert_eq!(nv.expr.to_token_stream().to_string(), "42");
}

#[test]
fn name_value_bool_literal() {
    let nv: NameValueExpr = parse_quote!(enabled = true);
    assert_eq!(nv.path.to_string(), "enabled");
    assert_eq!(nv.expr.to_token_stream().to_string(), "true");
}

#[test]
fn name_value_path_expr() {
    let nv: NameValueExpr = parse_quote!(target = std::io::Error);
    assert_eq!(nv.path.to_string(), "target");
    assert_eq!(nv.expr.to_token_stream().to_string(), "std :: io :: Error");
}

// =====================================================================
// 9. FieldThenParams parsing
// =====================================================================

#[test]
fn field_only_no_params() {
    let ftp: FieldThenParams = parse_quote!(String);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn field_with_single_param() {
    let ftp: FieldThenParams = parse_quote!(String, rename = "s");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "rename");
}

#[test]
fn field_with_multiple_params() {
    let ftp: FieldThenParams = parse_quote!(Vec<u8>, min = 1, max = 10);
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "min");
    assert_eq!(ftp.params[1].path.to_string(), "max");
}

#[test]
fn field_generic_type_with_param() {
    let ftp: FieldThenParams = parse_quote!(Option<String>, default = "none");
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "default");
}
