//! Tests for the common crate's syn-based parsing utilities.

use adze_common::*;
use quote::quote;
use syn::parse2;

#[test]
fn name_value_expr_parses() {
    let tokens = quote! { key = "value" };
    let result: syn::Result<NameValueExpr> = parse2(tokens);
    assert!(result.is_ok());
    let nve = result.unwrap();
    assert_eq!(nve.path.to_string(), "key");
}

#[test]
fn name_value_expr_with_number() {
    let tokens = quote! { count = 42 };
    let result: syn::Result<NameValueExpr> = parse2(tokens);
    assert!(result.is_ok());
}

#[test]
fn name_value_expr_with_bool() {
    let tokens = quote! { enabled = true };
    let result: syn::Result<NameValueExpr> = parse2(tokens);
    assert!(result.is_ok());
}

#[test]
fn name_value_expr_debug() {
    let tokens = quote! { name = "test" };
    let nve: NameValueExpr = parse2(tokens).unwrap();
    let debug = format!("{nve:?}");
    assert!(debug.contains("NameValueExpr"));
}

#[test]
fn name_value_expr_clone() {
    let tokens = quote! { name = "test" };
    let nve: NameValueExpr = parse2(tokens).unwrap();
    let cloned = nve.clone();
    assert_eq!(cloned.path.to_string(), nve.path.to_string());
}

#[test]
fn field_then_params_basic() {
    let tokens = quote! { String };
    let result: syn::Result<FieldThenParams> = parse2(tokens);
    assert!(result.is_ok());
}

#[test]
fn field_then_params_debug() {
    let tokens = quote! { String };
    let ftp: FieldThenParams = parse2(tokens).unwrap();
    let debug = format!("{ftp:?}");
    assert!(debug.contains("FieldThenParams"));
}
