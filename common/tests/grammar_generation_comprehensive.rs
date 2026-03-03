#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for grammar generation utilities in adze-common.
//!
//! Covers type extraction, filtering, wrapping, and attribute parsing
//! with grammar-like patterns including Vec<T>, Option<T>, Box<T>,
//! nested generics, unusual types, and integration between functions.

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
// 1. Grammar container extraction — Vec<T>
// ===========================================================================

#[test]
fn extract_vec_of_statement_nodes() {
    let ty: Type = parse_quote!(Vec<Statement>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Statement");
}

#[test]
fn extract_vec_through_box_skip() {
    let ty: Type = parse_quote!(Box<Vec<Declaration>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Declaration");
}

#[test]
fn extract_vec_preserves_complex_inner_type() {
    let ty: Type = parse_quote!(Vec<Result<Token, Error>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Result < Token , Error >");
}

// ===========================================================================
// 2. Grammar container extraction — Option<T>
// ===========================================================================

#[test]
fn extract_option_of_expression() {
    let ty: Type = parse_quote!(Option<Expression>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Expression");
}

#[test]
fn extract_option_through_arc_box_chain() {
    let ty: Type = parse_quote!(Arc<Box<Option<Literal>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Literal");
}

#[test]
fn extract_option_not_found_returns_original() {
    let ty: Type = parse_quote!(Vec<Token>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Vec < Token >");
}

// ===========================================================================
// 3. Nested generic extraction depth
// ===========================================================================

#[test]
fn extract_through_three_skip_layers() {
    let ty: Type = parse_quote!(Rc<Arc<Box<Option<Ident>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Rc", "Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Ident");
}

#[test]
fn extract_skip_chain_fails_when_target_absent() {
    let ty: Type = parse_quote!(Box<Arc<Rc<String>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc", "Rc"]));
    assert!(!ok);
    // When inner-most non-skip doesn't match, returns original
    assert_eq!(ty_str(&inner), "Box < Arc < Rc < String > > >");
}

// ===========================================================================
// 4. Unusual type inputs for extraction
// ===========================================================================

#[test]
fn extract_from_reference_type_returns_unchanged() {
    let ty: Type = parse_quote!(&[u8]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& [u8]");
}

#[test]
fn extract_from_tuple_type_returns_unchanged() {
    let ty: Type = parse_quote!((Expr, Stmt));
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "(Expr , Stmt)");
}

#[test]
fn extract_from_array_type_returns_unchanged() {
    let ty: Type = parse_quote!([Token; 8]);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "[Token ; 8]");
}

// ===========================================================================
// 5. filter_inner_type — grammar container unwrapping
// ===========================================================================

#[test]
fn filter_box_option_vec_all_skipped() {
    let ty: Type = parse_quote!(Box<Option<Vec<Node>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Option", "Vec"]));
    assert_eq!(ty_str(&filtered), "Node");
}

#[test]
fn filter_stops_at_first_non_skip_layer() {
    let ty: Type = parse_quote!(Box<HashMap<String, Value>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "HashMap < String , Value >");
}

#[test]
fn filter_no_op_on_plain_type() {
    let ty: Type = parse_quote!(Identifier);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Option", "Vec"]));
    assert_eq!(ty_str(&filtered), "Identifier");
}

#[test]
fn filter_reference_type_passthrough() {
    let ty: Type = parse_quote!(&'a str);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "& 'a str");
}

// ===========================================================================
// 6. wrap_leaf_type — grammar container preservation
// ===========================================================================

#[test]
fn wrap_vec_option_chain_wraps_innermost() {
    let ty: Type = parse_quote!(Vec<Option<Literal>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < Literal > > >"
    );
}

#[test]
fn wrap_plain_type_gets_withleaf() {
    let ty: Type = parse_quote!(NumberLiteral);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < NumberLiteral >");
}

#[test]
fn wrap_box_not_in_skip_wraps_entirely() {
    let ty: Type = parse_quote!(Box<Expr>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Box < Expr > >");
}

#[test]
fn wrap_tuple_type_wraps_entirely() {
    let ty: Type = parse_quote!((Expr, Operator, Expr));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < (Expr , Operator , Expr) >"
    );
}

#[test]
fn wrap_array_type_wraps_entirely() {
    let ty: Type = parse_quote!([Digit; 10]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [Digit ; 10] >");
}

#[test]
fn wrap_reference_type_wraps_entirely() {
    let ty: Type = parse_quote!(&Token);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & Token >");
}

// ===========================================================================
// 7. Integration: extract then filter then wrap
// ===========================================================================

#[test]
fn integration_extract_then_wrap_vec_inner() {
    let ty: Type = parse_quote!(Vec<Statement>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Statement >");
}

#[test]
fn integration_filter_then_wrap() {
    let ty: Type = parse_quote!(Box<Arc<Expression>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "Expression");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Expression >");
}

#[test]
fn integration_extract_through_skip_then_wrap_with_container() {
    let ty: Type = parse_quote!(Box<Option<Vec<Token>>>);
    // Extract Option, skipping Box
    let (after_option, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&after_option), "Vec < Token >");
    // Now wrap preserving Vec
    let wrapped = wrap_leaf_type(&after_option, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < Token > >");
}

#[test]
fn integration_filter_and_extract_compose() {
    let ty: Type = parse_quote!(Arc<Box<Vec<Stmt>>>);
    // First filter off Arc and Box
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(ty_str(&filtered), "Vec < Stmt >");
    // Then extract Vec
    let (inner, ok) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Stmt");
}

// ===========================================================================
// 8. NameValueExpr — grammar attribute patterns
// ===========================================================================

#[test]
fn name_value_expr_string_pattern() {
    let nv: NameValueExpr = parse_quote!(pattern = "[0-9]+");
    assert_eq!(nv.path.to_string(), "pattern");
    if let syn::Expr::Lit(lit) = &nv.expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "[0-9]+");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn name_value_expr_integer_precedence() {
    let nv: NameValueExpr = parse_quote!(precedence = 42);
    assert_eq!(nv.path.to_string(), "precedence");
    if let syn::Expr::Lit(lit) = &nv.expr
        && let syn::Lit::Int(i) = &lit.lit
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 42);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn name_value_expr_bool_value() {
    let nv: NameValueExpr = parse_quote!(extra = true);
    assert_eq!(nv.path.to_string(), "extra");
    if let syn::Expr::Lit(lit) = &nv.expr
        && let syn::Lit::Bool(b) = &lit.lit
    {
        assert!(b.value);
    } else {
        panic!("Expected bool literal");
    }
}

// ===========================================================================
// 9. FieldThenParams — grammar field declarations
// ===========================================================================

#[test]
fn field_then_params_generic_vec_with_pattern() {
    let parsed: FieldThenParams = parse_quote!(Vec<Digit>, pattern = "[0-9]");
    assert_eq!(ty_str(&parsed.field.ty), "Vec < Digit >");
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "pattern");
}

#[test]
fn field_then_params_option_with_precedence_and_assoc() {
    let parsed: FieldThenParams =
        parse_quote!(Option<BinOp>, precedence = 5, associativity = "left");
    assert_eq!(ty_str(&parsed.field.ty), "Option < BinOp >");
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "precedence");
    assert_eq!(parsed.params[1].path.to_string(), "associativity");
}

#[test]
fn field_then_params_no_params_preserves_complex_type() {
    let parsed: FieldThenParams = parse_quote!(Box<Vec<Option<Token>>>);
    assert_eq!(ty_str(&parsed.field.ty), "Box < Vec < Option < Token > > >");
    assert!(parsed.comma.is_none());
    assert!(parsed.params.is_empty());
}

// ===========================================================================
// 10. Edge cases — qualified paths and deeply nested types
// ===========================================================================

#[test]
fn extract_qualified_path_type_no_match() {
    let ty: Type = parse_quote!(std::vec::Vec<Node>);
    // The last segment is "Vec" so it should match
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Node");
}

#[test]
fn filter_qualified_path_in_skip_set() {
    // filter_inner_type checks last segment identity
    let ty: Type = parse_quote!(std::boxed::Box<Inner>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Inner");
}

#[test]
fn wrap_deeply_nested_four_skip_layers() {
    let ty: Type = parse_quote!(Vec<Option<Vec<Option<Leaf>>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < Vec < Option < adze :: WithLeaf < Leaf > > > > >"
    );
}
