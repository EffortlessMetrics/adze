#![allow(clippy::needless_range_loop)]

use adze_common::NameValueExpr;
use quote::ToTokens;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn parse_nve(src: &str) -> NameValueExpr {
    syn::parse_str(src).unwrap()
}

fn parse_nve_list(src: &str) -> Punctuated<NameValueExpr, syn::Token![,]> {
    let parser = Punctuated::<NameValueExpr, syn::Token![,]>::parse_terminated;
    parser.parse_str(src).unwrap()
}

// ===========================================================================
// 1. String literal value extraction
// ===========================================================================

#[test]
fn string_literal_simple() {
    let nve = parse_nve(r#"name = "hello""#);
    assert_eq!(nve.path.to_string(), "name");
    match &nve.expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => assert_eq!(s.value(), "hello"),
            other => panic!("expected string literal, got {other:?}"),
        },
        other => panic!("expected Expr::Lit, got {other:?}"),
    }
}

#[test]
fn string_literal_empty() {
    let nve = parse_nve(r#"key = """#);
    match &nve.expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => assert_eq!(s.value(), ""),
            other => panic!("expected string literal, got {other:?}"),
        },
        other => panic!("expected Expr::Lit, got {other:?}"),
    }
}

#[test]
fn string_literal_with_spaces() {
    let nve = parse_nve(r#"msg = "hello world""#);
    match &nve.expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => assert_eq!(s.value(), "hello world"),
            other => panic!("expected string literal, got {other:?}"),
        },
        other => panic!("expected Expr::Lit, got {other:?}"),
    }
}

#[test]
fn string_literal_with_escapes() {
    let nve = parse_nve(r#"pat = "line\nnext""#);
    match &nve.expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => assert_eq!(s.value(), "line\nnext"),
            other => panic!("expected string literal, got {other:?}"),
        },
        other => panic!("expected Expr::Lit, got {other:?}"),
    }
}

// ===========================================================================
// 2. Integer literal value extraction
// ===========================================================================

#[test]
fn integer_literal_positive() {
    let nve = parse_nve("precedence = 42");
    assert_eq!(nve.path.to_string(), "precedence");
    assert_eq!(nve.expr.to_token_stream().to_string(), "42");
}

#[test]
fn integer_literal_zero() {
    let nve = parse_nve("level = 0");
    assert_eq!(nve.expr.to_token_stream().to_string(), "0");
}

#[test]
fn integer_literal_negative() {
    let nve = parse_nve("offset = -7");
    let tokens = nve.expr.to_token_stream().to_string();
    assert!(tokens.contains("7"), "tokens should contain 7: {tokens}");
}

// ===========================================================================
// 3. Boolean literal value extraction
// ===========================================================================

#[test]
fn bool_literal_true() {
    let nve = parse_nve("enabled = true");
    assert_eq!(nve.path.to_string(), "enabled");
    assert_eq!(nve.expr.to_token_stream().to_string(), "true");
}

#[test]
fn bool_literal_false() {
    let nve = parse_nve("visible = false");
    assert_eq!(nve.path.to_string(), "visible");
    assert_eq!(nve.expr.to_token_stream().to_string(), "false");
}

// ===========================================================================
// 4. Path value extraction
// ===========================================================================

#[test]
fn path_value_simple_ident() {
    let nve = parse_nve("parser = my_parser");
    assert_eq!(nve.path.to_string(), "parser");
    assert_eq!(nve.expr.to_token_stream().to_string(), "my_parser");
}

#[test]
fn path_value_qualified() {
    let nve = parse_nve("handler = std::io::stdin");
    assert_eq!(nve.path.to_string(), "handler");
    let tokens = nve.expr.to_token_stream().to_string();
    assert!(tokens.contains("std"), "should contain std: {tokens}");
    assert!(tokens.contains("stdin"), "should contain stdin: {tokens}");
}

// ===========================================================================
// 5. Complex expression values
// ===========================================================================

#[test]
fn complex_expr_function_call() {
    let nve = parse_nve("init = default_value()");
    assert_eq!(nve.path.to_string(), "init");
    let tokens = nve.expr.to_token_stream().to_string();
    assert!(
        tokens.contains("default_value"),
        "should contain fn name: {tokens}"
    );
}

#[test]
fn complex_expr_array() {
    let nve = parse_nve("items = [1, 2, 3]");
    assert_eq!(nve.path.to_string(), "items");
    let tokens = nve.expr.to_token_stream().to_string();
    assert!(tokens.contains("1"), "should contain 1: {tokens}");
    assert!(tokens.contains("3"), "should contain 3: {tokens}");
}

#[test]
fn complex_expr_tuple() {
    let nve = parse_nve("pair = (1, 2)");
    assert_eq!(nve.path.to_string(), "pair");
    let tokens = nve.expr.to_token_stream().to_string();
    assert!(tokens.contains("1"));
    assert!(tokens.contains("2"));
}

// ===========================================================================
// 6. Multiple NameValueExpr from attribute (comma-separated list)
// ===========================================================================

#[test]
fn multiple_nve_two_params() {
    let list = parse_nve_list(r#"name = "test", value = 42"#);
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].path.to_string(), "name");
    assert_eq!(list[1].path.to_string(), "value");
}

#[test]
fn multiple_nve_three_params() {
    let list = parse_nve_list(r#"a = 1, b = "two", c = true"#);
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].path.to_string(), "a");
    assert_eq!(list[1].path.to_string(), "b");
    assert_eq!(list[2].path.to_string(), "c");
}

#[test]
fn multiple_nve_single_param() {
    let list = parse_nve_list("only = 99");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].path.to_string(), "only");
}

#[test]
fn multiple_nve_preserves_order() {
    let names = ["alpha", "beta", "gamma", "delta"];
    let src = names
        .iter()
        .enumerate()
        .map(|(i, n)| format!("{n} = {i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let list = parse_nve_list(&src);
    assert_eq!(list.len(), names.len());
    for i in 0..names.len() {
        assert_eq!(list[i].path.to_string(), names[i]);
    }
}

// ===========================================================================
// 7. NameValueExpr equality
// ===========================================================================

#[test]
fn equality_same_input() {
    let a = parse_nve("key = 1");
    let b = parse_nve("key = 1");
    assert_eq!(a, b);
}

#[test]
fn equality_clone() {
    let a = parse_nve(r#"name = "value""#);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn inequality_different_name() {
    let a = parse_nve("foo = 1");
    let b = parse_nve("bar = 1");
    assert_ne!(a, b);
}

#[test]
fn inequality_different_value() {
    let a = parse_nve("key = 1");
    let b = parse_nve("key = 2");
    assert_ne!(a, b);
}

// ===========================================================================
// 8. NameValueExpr debug output
// ===========================================================================

#[test]
fn debug_contains_struct_name() {
    let nve = parse_nve("x = 10");
    let dbg = format!("{nve:?}");
    assert!(
        dbg.contains("NameValueExpr"),
        "Debug should contain type name: {dbg}"
    );
}

#[test]
fn debug_non_empty() {
    let nve = parse_nve(r#"label = "abc""#);
    let dbg = format!("{nve:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn debug_contains_path_field() {
    let nve = parse_nve("myfield = 5");
    let dbg = format!("{nve:?}");
    assert!(
        dbg.contains("path"),
        "Debug should contain field 'path': {dbg}"
    );
}

// ===========================================================================
// 9. Missing value handling (parse errors)
// ===========================================================================

#[test]
fn error_empty_input() {
    assert!(syn::parse_str::<NameValueExpr>("").is_err());
}

#[test]
fn error_missing_equals_and_value() {
    assert!(syn::parse_str::<NameValueExpr>("name").is_err());
}

#[test]
fn error_missing_value_after_equals() {
    assert!(syn::parse_str::<NameValueExpr>("name =").is_err());
}

#[test]
fn error_missing_equals_sign() {
    assert!(syn::parse_str::<NameValueExpr>("name 42").is_err());
}

#[test]
fn error_leading_number_as_name() {
    assert!(syn::parse_str::<NameValueExpr>("123 = 1").is_err());
}

// ===========================================================================
// 10. Empty / unusual name handling
// ===========================================================================

#[test]
fn underscore_name() {
    // `_` is a keyword, not a valid identifier for syn::Ident parsing
    assert!(syn::parse_str::<NameValueExpr>("_ = 1").is_err());
}

#[test]
fn single_char_name() {
    let nve = parse_nve("x = 0");
    assert_eq!(nve.path.to_string(), "x");
}

#[test]
fn long_snake_case_name() {
    let nve = parse_nve("very_long_parameter_name = 100");
    assert_eq!(nve.path.to_string(), "very_long_parameter_name");
}

#[test]
fn name_with_leading_underscore() {
    let nve = parse_nve("_private = true");
    assert_eq!(nve.path.to_string(), "_private");
}

#[test]
fn keyword_raw_ident_as_name() {
    let nve = parse_nve("r#type = 5");
    // syn preserves the r# prefix in to_string()
    assert_eq!(nve.path.to_string(), "r#type");
}
