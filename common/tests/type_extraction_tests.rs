//! Comprehensive tests for type extraction and manipulation utilities
//! in adze-common (filter_inner_type, try_extract_inner_type, wrap_leaf_type,
//! FieldThenParams).

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. filter_inner_type — simple types (String, i32, bool)
// ===========================================================================

#[test]
fn filter_simple_string_no_skip() {
    let ty: Type = parse_quote!(String);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&[]))), "String");
}

#[test]
fn filter_simple_i32_no_skip() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&[]))), "i32");
}

#[test]
fn filter_simple_bool_with_irrelevant_skip() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "bool");
}

// ===========================================================================
// 2. filter_inner_type — generic types (Vec<T>, Option<T>)
// ===========================================================================

#[test]
fn filter_vec_when_vec_in_skip() {
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Vec"]))), "String");
}

#[test]
fn filter_option_when_option_in_skip() {
    let ty: Type = parse_quote!(Option<i32>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Option"]))), "i32");
}

#[test]
fn filter_vec_when_vec_not_in_skip() {
    let ty: Type = parse_quote!(Vec<u8>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < u8 >"
    );
}

// ===========================================================================
// 3. filter_inner_type — nested generics (Option<Vec<T>>)
// ===========================================================================

#[test]
fn filter_option_vec_both_skipped() {
    let ty: Type = parse_quote!(Option<Vec<f64>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Option", "Vec"]))),
        "f64"
    );
}

#[test]
fn filter_option_vec_only_option_skipped() {
    let ty: Type = parse_quote!(Option<Vec<f64>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Option"]))),
        "Vec < f64 >"
    );
}

#[test]
fn filter_vec_option_result_all_skipped() {
    let ty: Type = parse_quote!(Vec<Option<Result<Token>>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Vec", "Option", "Result"]))),
        "Token"
    );
}

// ===========================================================================
// 4. filter_inner_type — Box types
// ===========================================================================

#[test]
fn filter_box_strips_to_inner() {
    let ty: Type = parse_quote!(Box<Expr>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "Expr");
}

#[test]
fn filter_box_box_strips_both() {
    let ty: Type = parse_quote!(Box<Box<Leaf>>);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "Leaf");
}

// ===========================================================================
// 5. filter_inner_type — skip set matching
// ===========================================================================

#[test]
fn filter_empty_skip_set_preserves_generic() {
    let ty: Type = parse_quote!(Option<String>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&[]))),
        "Option < String >"
    );
}

#[test]
fn filter_skip_set_partial_match_stops_at_first_non_skip() {
    let ty: Type = parse_quote!(Box<Vec<Arc<Node>>>);
    // Only Box is in skip, so stops at Vec
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "Vec < Arc < Node > >"
    );
}

#[test]
fn filter_arc_and_rc_in_skip() {
    let ty: Type = parse_quote!(Arc<Rc<Inner>>);
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Arc", "Rc"]))),
        "Inner"
    );
}

// ===========================================================================
// 6. try_extract_inner_type — Option<T>
// ===========================================================================

#[test]
fn extract_option_string() {
    let ty: Type = parse_quote!(Option<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_option_through_box() {
    let ty: Type = parse_quote!(Box<Option<u32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn extract_option_nested_under_two_skips() {
    let ty: Type = parse_quote!(Arc<Box<Option<Token>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Token");
}

// ===========================================================================
// 7. try_extract_inner_type — Vec<T>
// ===========================================================================

#[test]
fn extract_vec_i32() {
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_vec_through_option_skip() {
    let ty: Type = parse_quote!(Option<Vec<Stmt>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Option"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Stmt");
}

#[test]
fn extract_vec_returns_generic_inner_intact() {
    let ty: Type = parse_quote!(Vec<HashMap<String, i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

// ===========================================================================
// 8. try_extract_inner_type — non-matching types
// ===========================================================================

#[test]
fn extract_no_match_simple() {
    let ty: Type = parse_quote!(String);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_no_match_different_generic() {
    let ty: Type = parse_quote!(Result<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Result < String >");
}

#[test]
fn extract_no_match_skip_but_target_absent() {
    // Box is in skip set, but inner is String — no Option anywhere
    let ty: Type = parse_quote!(Box<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < String >");
}

// ===========================================================================
// 9. wrap_leaf_type — simple types
// ===========================================================================

#[test]
fn wrap_simple_string() {
    let ty: Type = parse_quote!(String);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn wrap_simple_i32() {
    let ty: Type = parse_quote!(i32);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_simple_bool() {
    let ty: Type = parse_quote!(bool);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < bool >"
    );
}

// ===========================================================================
// 10. wrap_leaf_type — complex types
// ===========================================================================

#[test]
fn wrap_option_vec_both_in_skip() {
    let ty: Type = parse_quote!(Option<Vec<Node>>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Option", "Vec"]))),
        "Option < Vec < adze :: WithLeaf < Node > > >"
    );
}

#[test]
fn wrap_result_in_skip_wraps_both_args() {
    let ty: Type = parse_quote!(Result<Ok, Err>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Result"]))),
        "Result < adze :: WithLeaf < Ok > , adze :: WithLeaf < Err > >"
    );
}

#[test]
fn wrap_hashmap_not_in_skip_wraps_entire_type() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

// ===========================================================================
// 11. wrap_leaf_type — skip set behavior
// ===========================================================================

#[test]
fn wrap_skip_preserves_outer_wraps_inner() {
    let ty: Type = parse_quote!(Vec<Token>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "Vec < adze :: WithLeaf < Token > >"
    );
}

#[test]
fn wrap_skip_nested_three_levels() {
    let ty: Type = parse_quote!(Box<Option<Vec<Leaf>>>);
    let s = &skip(&["Box", "Option", "Vec"]);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, s)),
        "Box < Option < Vec < adze :: WithLeaf < Leaf > > > >"
    );
}

#[test]
fn wrap_skip_stops_at_non_skip_outer() {
    // HashMap is NOT in skip, so the whole thing gets wrapped
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&["Vec"]))),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

// ===========================================================================
// 12. FieldThenParams — extraction from attributes
// ===========================================================================

#[test]
fn field_then_params_bare_type() {
    let parsed: FieldThenParams = parse_quote!(Identifier);
    assert_eq!(ty_str(&parsed.field.ty), "Identifier");
    assert!(parsed.comma.is_none());
    assert!(parsed.params.is_empty());
}

#[test]
fn field_then_params_with_pattern() {
    let parsed: FieldThenParams = parse_quote!(String, pattern = "[a-z]+");
    assert_eq!(ty_str(&parsed.field.ty), "String");
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "pattern");
}

#[test]
fn field_then_params_generic_field_with_params() {
    let parsed: FieldThenParams = parse_quote!(Vec<Statement>, min = 0, max = 100);
    assert_eq!(ty_str(&parsed.field.ty), "Vec < Statement >");
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "min");
    assert_eq!(parsed.params[1].path.to_string(), "max");
}

// ===========================================================================
// 13. FieldThenParams — precedence
// ===========================================================================

#[test]
fn field_then_params_precedence_integer() {
    let parsed: FieldThenParams = parse_quote!(Expr, precedence = 5);
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "precedence");
    if let syn::Expr::Lit(lit) = &parsed.params[0].expr
        && let syn::Lit::Int(i) = &lit.lit
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 5);
    } else {
        panic!("Expected integer literal for precedence");
    }
}

#[test]
fn field_then_params_precedence_zero() {
    let parsed: FieldThenParams = parse_quote!(Term, precedence = 0);
    if let syn::Expr::Lit(lit) = &parsed.params[0].expr
        && let syn::Lit::Int(i) = &lit.lit
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 0);
    } else {
        panic!("Expected integer literal for precedence");
    }
}

#[test]
fn field_then_params_precedence_with_other_params() {
    let parsed: FieldThenParams = parse_quote!(BinOp, precedence = 3, pattern = "\\+");
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "precedence");
    assert_eq!(parsed.params[1].path.to_string(), "pattern");
}

// ===========================================================================
// 14. FieldThenParams — associativity
// ===========================================================================

#[test]
fn field_then_params_associativity_left() {
    let parsed: FieldThenParams = parse_quote!(AddExpr, associativity = "left");
    assert_eq!(parsed.params[0].path.to_string(), "associativity");
    if let syn::Expr::Lit(lit) = &parsed.params[0].expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "left");
    } else {
        panic!("Expected string literal for associativity");
    }
}

#[test]
fn field_then_params_associativity_right() {
    let parsed: FieldThenParams = parse_quote!(PowExpr, associativity = "right");
    if let syn::Expr::Lit(lit) = &parsed.params[0].expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "right");
    } else {
        panic!("Expected string literal for associativity");
    }
}

#[test]
fn field_then_params_precedence_and_associativity() {
    let parsed: FieldThenParams = parse_quote!(MulExpr, precedence = 10, associativity = "left");
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "precedence");
    assert_eq!(parsed.params[1].path.to_string(), "associativity");
}

// ===========================================================================
// 15. Edge cases — unit types, never type, reference types
// ===========================================================================

#[test]
fn edge_unit_type_filter() {
    let ty: Type = parse_quote!(());
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "()");
}

#[test]
fn edge_unit_type_extract_no_match() {
    let ty: Type = parse_quote!(());
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "()");
}

#[test]
fn edge_unit_type_wrap() {
    let ty: Type = parse_quote!(());
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < () >"
    );
}

#[test]
fn edge_never_type_wrap() {
    let ty: Type = parse_quote!(!);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < ! >"
    );
}

#[test]
fn edge_never_type_extract() {
    let ty: Type = parse_quote!(!);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "!");
}

#[test]
fn edge_reference_type_filter() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(ty_str(&filter_inner_type(&ty, &skip(&["Box"]))), "& str");
}

#[test]
fn edge_reference_type_extract_no_match() {
    let ty: Type = parse_quote!(&str);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn edge_reference_type_wrap() {
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn edge_tuple_type_filter() {
    let ty: Type = parse_quote!((i32, u64));
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "(i32 , u64)"
    );
}

#[test]
fn edge_array_type_wrap() {
    let ty: Type = parse_quote!([u8; 4]);
    assert_eq!(
        ty_str(&wrap_leaf_type(&ty, &skip(&[]))),
        "adze :: WithLeaf < [u8 ; 4] >"
    );
}

#[test]
fn edge_qualified_path_type() {
    let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
    // Not in skip set, so filter returns unchanged
    assert_eq!(
        ty_str(&filter_inner_type(&ty, &skip(&["Box"]))),
        "std :: collections :: HashMap < String , i32 >"
    );
}

// ===========================================================================
// Additional coverage — NameValueExpr parsing
// ===========================================================================

#[test]
fn name_value_expr_closure_value() {
    let nv: NameValueExpr = parse_quote!(transform = |x: String| x.len());
    assert_eq!(nv.path.to_string(), "transform");
    assert!(matches!(nv.expr, syn::Expr::Closure(_)));
}
