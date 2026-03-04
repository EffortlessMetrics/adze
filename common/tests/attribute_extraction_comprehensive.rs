#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for attribute extraction and parsing in adze-common.
//!
//! Covers `NameValueExpr` and `FieldThenParams` parsing via the syn `Parse` trait,
//! including error cases, whitespace handling, complex patterns, and round-trips.

use adze_common::{FieldThenParams, NameValueExpr};
use quote::{ToTokens, quote};
use syn::{Expr, parse2};

// ---------------------------------------------------------------------------
// NameValueExpr – various value types
// ---------------------------------------------------------------------------

#[test]
fn nve_string_literal() {
    let nve: NameValueExpr = parse2(quote! { key = "hello" }).unwrap();
    assert_eq!(nve.path.to_string(), "key");
    assert!(matches!(nve.expr, Expr::Lit(_)));
}

#[test]
fn nve_integer_literal() {
    let nve: NameValueExpr = parse2(quote! { count = 42 }).unwrap();
    assert_eq!(nve.path.to_string(), "count");
    assert!(matches!(nve.expr, Expr::Lit(_)));
}

#[test]
fn nve_boolean_true() {
    let nve: NameValueExpr = parse2(quote! { enabled = true }).unwrap();
    assert_eq!(nve.path.to_string(), "enabled");
    assert!(matches!(nve.expr, Expr::Lit(_)));
}

#[test]
fn nve_boolean_false() {
    let nve: NameValueExpr = parse2(quote! { hidden = false }).unwrap();
    assert_eq!(nve.path.to_string(), "hidden");
    assert!(matches!(nve.expr, Expr::Lit(_)));
}

#[test]
fn nve_negative_integer() {
    let nve: NameValueExpr = parse2(quote! { offset = -7 }).unwrap();
    assert_eq!(nve.path.to_string(), "offset");
    // -7 is parsed as a Unary negation expression
    assert!(matches!(nve.expr, Expr::Unary(_)));
}

#[test]
fn nve_float_literal() {
    let nve: NameValueExpr = parse2(quote! { ratio = 3.14 }).unwrap();
    assert_eq!(nve.path.to_string(), "ratio");
    assert!(matches!(nve.expr, Expr::Lit(_)));
}

#[test]
fn nve_char_literal() {
    let nve: NameValueExpr = parse2(quote! { delimiter = 'x' }).unwrap();
    assert_eq!(nve.path.to_string(), "delimiter");
    assert!(matches!(nve.expr, Expr::Lit(_)));
}

#[test]
fn nve_closure_value() {
    let nve: NameValueExpr = parse2(quote! { transform = |x| x + 1 }).unwrap();
    assert_eq!(nve.path.to_string(), "transform");
    assert!(matches!(nve.expr, Expr::Closure(_)));
}

#[test]
fn nve_path_value() {
    let nve: NameValueExpr = parse2(quote! { parser = my_mod::custom_parser }).unwrap();
    assert_eq!(nve.path.to_string(), "parser");
    assert!(matches!(nve.expr, Expr::Path(_)));
}

#[test]
fn nve_array_value() {
    let nve: NameValueExpr = parse2(quote! { items = [1, 2, 3] }).unwrap();
    assert_eq!(nve.path.to_string(), "items");
    assert!(matches!(nve.expr, Expr::Array(_)));
}

#[test]
fn nve_tuple_value() {
    let nve: NameValueExpr = parse2(quote! { pair = (1, 2) }).unwrap();
    assert_eq!(nve.path.to_string(), "pair");
    assert!(matches!(nve.expr, Expr::Tuple(_)));
}

// ---------------------------------------------------------------------------
// NameValueExpr – error cases
// ---------------------------------------------------------------------------

#[test]
fn nve_missing_equals_is_err() {
    let result: syn::Result<NameValueExpr> = parse2(quote! { key "value" });
    assert!(result.is_err());
}

#[test]
fn nve_missing_value_is_err() {
    let result: syn::Result<NameValueExpr> = parse2(quote! { key = });
    assert!(result.is_err());
}

#[test]
fn nve_empty_input_is_err() {
    let result: syn::Result<NameValueExpr> = parse2(quote! {});
    assert!(result.is_err());
}

#[test]
fn nve_number_as_key_is_err() {
    // A literal where an Ident is expected should fail.
    let result: syn::Result<NameValueExpr> = parse2(quote! { 123 = "bad" });
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// NameValueExpr – round-trip / token fidelity
// ---------------------------------------------------------------------------

#[test]
fn nve_round_trip_string() {
    let nve: NameValueExpr = parse2(quote! { name = "hello" }).unwrap();
    assert_eq!(nve.path.to_string(), "name");
    let expr_str = nve.expr.to_token_stream().to_string();
    assert!(expr_str.contains("hello"));
    // Re-parse the individual pieces to verify they survive tokenisation
    let path = &nve.path;
    let expr = &nve.expr;
    let reconstructed = quote! { #path = #expr };
    let reparsed: NameValueExpr = parse2(reconstructed).unwrap();
    assert_eq!(reparsed.path.to_string(), "name");
}

#[test]
fn nve_round_trip_integer() {
    let nve: NameValueExpr = parse2(quote! { size = 256 }).unwrap();
    let expr_str = nve.expr.to_token_stream().to_string();
    assert!(expr_str.contains("256"));
}

// ---------------------------------------------------------------------------
// NameValueExpr – Clone / Eq / Debug trait coverage
// ---------------------------------------------------------------------------

#[test]
fn nve_clone_equals_original() {
    let nve: NameValueExpr = parse2(quote! { a = 1 }).unwrap();
    let cloned = nve.clone();
    assert_eq!(nve, cloned);
}

#[test]
fn nve_debug_contains_struct_name() {
    let nve: NameValueExpr = parse2(quote! { x = 0 }).unwrap();
    let dbg = format!("{nve:?}");
    assert!(dbg.contains("NameValueExpr"));
}

// ---------------------------------------------------------------------------
// FieldThenParams – basic field only
// ---------------------------------------------------------------------------

#[test]
fn ftp_simple_type_no_params() {
    let ftp: FieldThenParams = parse2(quote! { String }).unwrap();
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

#[test]
fn ftp_generic_type_no_params() {
    let ftp: FieldThenParams = parse2(quote! { Vec<u8> }).unwrap();
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    let ty_str = ftp.field.ty.to_token_stream().to_string();
    assert!(ty_str.contains("Vec"));
}

#[test]
fn ftp_nested_generic_no_params() {
    let ftp: FieldThenParams = parse2(quote! { Option<Vec<String>> }).unwrap();
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
}

// ---------------------------------------------------------------------------
// FieldThenParams – with parameters
// ---------------------------------------------------------------------------

#[test]
fn ftp_single_param() {
    let ftp: FieldThenParams = parse2(quote! { String, pattern = "abc" }).unwrap();
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "pattern");
}

#[test]
fn ftp_multiple_params() {
    let ftp: FieldThenParams = parse2(quote! { i32, min = 0, max = 100, step = 1 }).unwrap();
    assert_eq!(ftp.params.len(), 3);
    let names: Vec<String> = ftp.params.iter().map(|p| p.path.to_string()).collect();
    assert_eq!(names, vec!["min", "max", "step"]);
}

#[test]
fn ftp_param_values_are_accessible() {
    let ftp: FieldThenParams = parse2(quote! { bool, default = true }).unwrap();
    assert_eq!(ftp.params.len(), 1);
    assert!(matches!(ftp.params[0].expr, Expr::Lit(_)));
}

#[test]
fn ftp_complex_type_with_params() {
    let ftp: FieldThenParams =
        parse2(quote! { std::collections::HashMap<String, Vec<u8>>, capacity = 16 }).unwrap();
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "capacity");
    let ty_str = ftp.field.ty.to_token_stream().to_string();
    assert!(ty_str.contains("HashMap"));
}

// ---------------------------------------------------------------------------
// FieldThenParams – error cases
// ---------------------------------------------------------------------------

#[test]
fn ftp_empty_input_is_err() {
    let result: syn::Result<FieldThenParams> = parse2(quote! {});
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// FieldThenParams – Clone / Eq / Debug trait coverage
// ---------------------------------------------------------------------------

#[test]
fn ftp_clone_equals_original() {
    let ftp: FieldThenParams = parse2(quote! { u64, limit = 10 }).unwrap();
    let cloned = ftp.clone();
    assert_eq!(ftp, cloned);
}

#[test]
fn ftp_debug_contains_struct_name() {
    let ftp: FieldThenParams = parse2(quote! { u8 }).unwrap();
    let dbg = format!("{ftp:?}");
    assert!(dbg.contains("FieldThenParams"));
}

// ---------------------------------------------------------------------------
// Integration: parse2 round-trip for FieldThenParams
// ---------------------------------------------------------------------------

#[test]
fn ftp_field_type_preserved() {
    let ftp: FieldThenParams = parse2(quote! { Vec<String>, tag = "items" }).unwrap();
    let ty_tokens = ftp.field.ty.to_token_stream().to_string();
    assert!(ty_tokens.contains("Vec"));
    assert!(ty_tokens.contains("String"));
}

#[test]
fn ftp_trailing_comma_in_params() {
    // Punctuated::parse_terminated allows trailing commas
    let ftp: FieldThenParams = parse2(quote! { u32, a = 1, b = 2, }).unwrap();
    assert_eq!(ftp.params.len(), 2);
}
