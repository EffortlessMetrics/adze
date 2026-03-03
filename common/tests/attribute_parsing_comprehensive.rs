#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for attribute parsing logic in adze-common.
//!
//! Covers NameValueExpr parsing, FieldThenParams parsing, try_extract_inner_type,
//! filter_inner_type, and wrap_leaf_type across a wide range of type shapes.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
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
// 1. NameValueExpr — various value expression kinds
// ===========================================================================

#[test]
fn nve_integer_literal() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path.to_string(), "precedence");
    if let syn::Expr::Lit(lit) = &nv.expr {
        if let syn::Lit::Int(i) = &lit.lit {
            assert_eq!(i.base10_parse::<i32>().unwrap(), 42);
        } else {
            panic!("expected int literal");
        }
    } else {
        panic!("expected literal expression");
    }
}

#[test]
fn nve_string_literal() {
    let nv: NameValueExpr = parse_quote!(pattern = "hello");
    assert_eq!(nv.path.to_string(), "pattern");
    if let syn::Expr::Lit(lit) = &nv.expr {
        if let syn::Lit::Str(s) = &lit.lit {
            assert_eq!(s.value(), "hello");
        } else {
            panic!("expected string literal");
        }
    } else {
        panic!("expected literal expression");
    }
}

#[test]
fn nve_bool_literal() {
    let nv: NameValueExpr = parse_quote!(optional = true);
    assert_eq!(nv.path.to_string(), "optional");
    assert!(matches!(nv.expr, syn::Expr::Lit(_)));
}

#[test]
fn nve_negative_integer() {
    let nv: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nv.path.to_string(), "offset");
    // Negative literal is parsed as a unary negation expression
    assert!(matches!(nv.expr, syn::Expr::Unary(_)));
}

#[test]
fn nve_path_value() {
    let nv: NameValueExpr = parse_quote!(kind = Left);
    assert_eq!(nv.path.to_string(), "kind");
    assert!(matches!(nv.expr, syn::Expr::Path(_)));
}

// ===========================================================================
// 2. FieldThenParams — structural variations
// ===========================================================================

#[test]
fn ftp_simple_type_no_params() {
    let parsed: FieldThenParams = parse_quote!(u64);
    assert_eq!(ty_str(&parsed.field.ty), "u64");
    assert!(parsed.comma.is_none());
    assert!(parsed.params.is_empty());
}

#[test]
fn ftp_generic_type_no_params() {
    let parsed: FieldThenParams = parse_quote!(Option<String>);
    assert_eq!(ty_str(&parsed.field.ty), "Option < String >");
    assert!(parsed.comma.is_none());
    assert!(parsed.params.is_empty());
}

#[test]
fn ftp_single_param() {
    let parsed: FieldThenParams = parse_quote!(i32, min = 0);
    assert_eq!(ty_str(&parsed.field.ty), "i32");
    assert!(parsed.comma.is_some());
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "min");
}

#[test]
fn ftp_three_params() {
    let parsed: FieldThenParams = parse_quote!(
        Token,
        precedence = 5,
        associativity = "left",
        pattern = "\\+"
    );
    assert_eq!(ty_str(&parsed.field.ty), "Token");
    assert_eq!(parsed.params.len(), 3);
    let names: Vec<String> = parsed.params.iter().map(|p| p.path.to_string()).collect();
    assert_eq!(names, vec!["precedence", "associativity", "pattern"]);
}

#[test]
fn ftp_nested_generic_with_param() {
    let parsed: FieldThenParams = parse_quote!(Vec<Option<Expr>>, min = 1);
    assert_eq!(ty_str(&parsed.field.ty), "Vec < Option < Expr > >");
    assert_eq!(parsed.params.len(), 1);
}

#[test]
fn ftp_param_ordering_preserved() {
    let parsed: FieldThenParams = parse_quote!(Stmt, beta = 2, alpha = 1);
    assert_eq!(parsed.params[0].path.to_string(), "beta");
    assert_eq!(parsed.params[1].path.to_string(), "alpha");
}

// ===========================================================================
// 3. try_extract_inner_type — extraction through multiple skip layers
// ===========================================================================

#[test]
fn extract_direct_match_returns_inner() {
    let ty: Type = parse_quote!(Option<bool>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "bool");
}

#[test]
fn extract_through_single_skip() {
    let ty: Type = parse_quote!(Box<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

#[test]
fn extract_through_two_skips() {
    let ty: Type = parse_quote!(Arc<Box<Option<f32>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "f32");
}

#[test]
fn extract_mismatch_returns_original_with_false() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < String >");
}

#[test]
fn extract_skip_present_but_target_missing() {
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    // Returns the original type when target is not found
    assert_eq!(ty_str(&inner), "Box < String >");
}

#[test]
fn extract_non_path_type_returns_unchanged() {
    let ty: Type = parse_quote!(&[u8]);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& [u8]");
}

#[test]
fn extract_preserves_complex_inner_type() {
    let ty: Type = parse_quote!(Option<(i32, String)>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "(i32 , String)");
}

// ===========================================================================
// 4. filter_inner_type — unwrapping container layers
// ===========================================================================

#[test]
fn filter_no_skip_preserves_original() {
    let ty: Type = parse_quote!(Box<i32>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), "Box < i32 >");
}

#[test]
fn filter_single_layer() {
    let ty: Type = parse_quote!(Arc<MyType>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc"]));
    assert_eq!(ty_str(&filtered), "MyType");
}

#[test]
fn filter_triple_nesting() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Leaf>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ty_str(&filtered), "Leaf");
}

#[test]
fn filter_stops_at_non_skip_type() {
    let ty: Type = parse_quote!(Box<HashMap<String, i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "HashMap < String , i32 >");
}

#[test]
fn filter_non_path_type_unchanged() {
    let ty: Type = parse_quote!((u8, u16));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(u8 , u16)");
}

// ===========================================================================
// 5. wrap_leaf_type — WithLeaf wrapping behavior
// ===========================================================================

#[test]
fn wrap_plain_type() {
    let ty: Type = parse_quote!(Token);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Token >");
}

#[test]
fn wrap_skip_preserves_outer_wraps_inner() {
    let ty: Type = parse_quote!(Option<Node>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < Node > >");
}

#[test]
fn wrap_nested_skip_types() {
    let ty: Type = parse_quote!(Vec<Option<Ident>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < Ident > > >"
    );
}

#[test]
fn wrap_non_skip_generic_wraps_whole() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < Result < String , Error > >"
    );
}

#[test]
fn wrap_reference_type_wraps_whole() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & str >");
}

// ===========================================================================
// 6. Interactions between functions
// ===========================================================================

#[test]
fn extract_then_wrap() {
    // Extract inner type from Option, then wrap the result
    let ty: Type = parse_quote!(Option<Expr>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Expr >");
}

#[test]
fn filter_then_wrap() {
    // Filter away Box, then wrap the result
    let ty: Type = parse_quote!(Box<Stmt>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Stmt");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Stmt >");
}

#[test]
fn extract_and_filter_same_result() {
    // For a single-layer container, extracting the inner type should match
    // filtering with the same skip set
    let ty: Type = parse_quote!(Box<Token>);
    let (extracted, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&extracted), ty_str(&filtered));
}
