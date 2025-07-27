use rust_sitter_common::{FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use syn::{parse_quote, Expr, Type};
use std::collections::HashSet;

#[test]
fn test_name_value_expr_parsing() {
    // Test basic name=value parsing
    let expr: NameValueExpr = parse_quote!(foo = 42);
    assert_eq!(expr.path.to_string(), "foo");
    assert!(matches!(expr.expr, Expr::Lit(_)));

    // Test with string value
    let expr: NameValueExpr = parse_quote!(name = "value");
    assert_eq!(expr.path.to_string(), "name");
    assert!(matches!(expr.expr, Expr::Lit(_)));

    // Test with complex expression
    let expr: NameValueExpr = parse_quote!(transform = |x| x + 1);
    assert_eq!(expr.path.to_string(), "transform");
    assert!(matches!(expr.expr, Expr::Closure(_)));
}

#[test]
fn test_field_then_params_parsing() {
    // Test field with no params
    let parsed: FieldThenParams = parse_quote!(String);
    assert!(parsed.comma.is_none());
    assert_eq!(parsed.params.len(), 0);

    // Test field with single param
    let parsed: FieldThenParams = parse_quote!(String, pattern = r"\d+");
    assert!(parsed.comma.is_some());
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "pattern");

    // Test field with multiple params
    let parsed: FieldThenParams = parse_quote!(Vec<String>, min = 1, max = 10);
    assert!(parsed.comma.is_some());
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "min");
    assert_eq!(parsed.params[1].path.to_string(), "max");
}

#[test]
fn test_try_extract_inner_type() {
    let mut skip_over = HashSet::new();
    skip_over.insert("Box");
    skip_over.insert("Vec");

    // Test direct Option extraction
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(extracted);
    assert_eq!(quote::quote!(#inner).to_string(), "String");

    // Test skipping over Box to find Option
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(extracted);
    assert_eq!(quote::quote!(#inner).to_string(), "i32");

    // Test skipping over Vec to find Option
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(extracted);
    assert_eq!(quote::quote!(#inner).to_string(), "bool");

    // Test no match returns original
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(!extracted);
    assert_eq!(quote::quote!(#inner).to_string(), "String");
}

#[test]
fn test_filter_inner_type() {
    let mut skip_over = HashSet::new();
    skip_over.insert("Box");
    skip_over.insert("Vec");
    skip_over.insert("Option");

    // Test filtering Box
    let ty: Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip_over);
    assert_eq!(quote::quote!(#filtered).to_string(), "String");

    // Test filtering nested types
    let ty: Type = parse_quote!(Box<Vec<Option<i32>>>);
    let filtered = filter_inner_type(&ty, &skip_over);
    assert_eq!(quote::quote!(#filtered).to_string(), "i32");

    // Test no filtering needed
    let ty: Type = parse_quote!(String);
    let filtered = filter_inner_type(&ty, &skip_over);
    assert_eq!(quote::quote!(#filtered).to_string(), "String");
}

#[test]
fn test_wrap_leaf_type() {
    let mut skip_over = HashSet::new();
    skip_over.insert("Vec");
    skip_over.insert("Option");

    // Test wrapping simple type
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip_over);
    assert_eq!(quote::quote!(#wrapped).to_string(), "rust_sitter :: WithLeaf < String >");

    // Test wrapping with skip_over types preserved
    let ty: Type = parse_quote!(Vec<String>);
    let wrapped = wrap_leaf_type(&ty, &skip_over);
    assert_eq!(quote::quote!(#wrapped).to_string(), "Vec < rust_sitter :: WithLeaf < String > >");

    // Test wrapping nested skip_over types
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip_over);
    assert_eq!(quote::quote!(#wrapped).to_string(), "Option < Vec < rust_sitter :: WithLeaf < i32 > > >");
}