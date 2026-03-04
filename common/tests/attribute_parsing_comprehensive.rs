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

// ===========================================================================
// 7. NameValueExpr — additional value expression kinds
// ===========================================================================

#[test]
fn nve_float_literal() {
    let nv: NameValueExpr = parse_quote!(weight = 3.14);
    assert_eq!(nv.path.to_string(), "weight");
    if let syn::Expr::Lit(lit) = &nv.expr {
        assert!(matches!(lit.lit, syn::Lit::Float(_)));
    } else {
        panic!("expected literal expression");
    }
}

#[test]
fn nve_char_literal() {
    let nv: NameValueExpr = parse_quote!(delimiter = 'x');
    assert_eq!(nv.path.to_string(), "delimiter");
    if let syn::Expr::Lit(lit) = &nv.expr {
        assert!(matches!(lit.lit, syn::Lit::Char(_)));
    } else {
        panic!("expected literal expression");
    }
}

#[test]
fn nve_byte_string_literal() {
    let nv: NameValueExpr = parse_quote!(data = b"abc");
    assert_eq!(nv.path.to_string(), "data");
    assert!(matches!(nv.expr, syn::Expr::Lit(_)));
}

#[test]
fn nve_empty_string() {
    let nv: NameValueExpr = parse_quote!(pattern = "");
    assert_eq!(nv.path.to_string(), "pattern");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), "");
    } else {
        panic!("expected empty string literal");
    }
}

#[test]
fn nve_qualified_path_value() {
    let nv: NameValueExpr = parse_quote!(assoc = std::cmp::Ordering::Less);
    assert_eq!(nv.path.to_string(), "assoc");
}

#[test]
fn nve_tuple_expression() {
    let nv: NameValueExpr = parse_quote!(range = (1, 10));
    assert_eq!(nv.path.to_string(), "range");
    assert!(matches!(nv.expr, syn::Expr::Tuple(_)));
}

#[test]
fn nve_closure_expression() {
    let nv: NameValueExpr = parse_quote!(transform = |x| x + 1);
    assert_eq!(nv.path.to_string(), "transform");
    assert!(matches!(nv.expr, syn::Expr::Closure(_)));
}

#[test]
fn nve_array_expression() {
    let nv: NameValueExpr = parse_quote!(items = [1, 2, 3]);
    assert_eq!(nv.path.to_string(), "items");
    assert!(matches!(nv.expr, syn::Expr::Array(_)));
}

#[test]
fn nve_zero_integer() {
    let nv: NameValueExpr = parse_quote!(precedence = 0);
    assert_eq!(nv.path.to_string(), "precedence");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = &nv.expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 0);
    } else {
        panic!("expected int literal");
    }
}

#[test]
fn nve_large_integer() {
    let nv: NameValueExpr = parse_quote!(max = 999999);
    assert_eq!(nv.path.to_string(), "max");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = &nv.expr
    {
        assert_eq!(i.base10_parse::<i64>().unwrap(), 999_999);
    } else {
        panic!("expected int literal");
    }
}

#[test]
fn nve_underscore_in_name() {
    let nv: NameValueExpr = parse_quote!(my_param = 7);
    assert_eq!(nv.path.to_string(), "my_param");
}

#[test]
fn nve_unicode_string() {
    let nv: NameValueExpr = parse_quote!(label = "こんにちは");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), "こんにちは");
    } else {
        panic!("expected string literal");
    }
}

#[test]
fn nve_escaped_string() {
    let nv: NameValueExpr = parse_quote!(pattern = "a\\.b");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &nv.expr
    {
        assert_eq!(s.value(), "a\\.b");
    } else {
        panic!("expected string literal");
    }
}

// ===========================================================================
// 8. FieldThenParams — more structural variations
// ===========================================================================

#[test]
fn ftp_unit_type() {
    let parsed: FieldThenParams = parse_quote!(());
    assert_eq!(ty_str(&parsed.field.ty), "()");
    assert!(parsed.params.is_empty());
}

#[test]
fn ftp_reference_type() {
    let parsed: FieldThenParams = parse_quote!(&str);
    assert_eq!(ty_str(&parsed.field.ty), "& str");
    assert!(parsed.params.is_empty());
}

#[test]
fn ftp_slice_type() {
    let parsed: FieldThenParams = parse_quote!(&[u8]);
    assert_eq!(ty_str(&parsed.field.ty), "& [u8]");
    assert!(parsed.params.is_empty());
}

#[test]
fn ftp_array_type() {
    let parsed: FieldThenParams = parse_quote!([u8; 4]);
    assert_eq!(ty_str(&parsed.field.ty), "[u8 ; 4]");
    assert!(parsed.params.is_empty());
}

#[test]
fn ftp_tuple_type() {
    let parsed: FieldThenParams = parse_quote!((i32, String));
    assert_eq!(ty_str(&parsed.field.ty), "(i32 , String)");
    assert!(parsed.params.is_empty());
}

#[test]
fn ftp_double_nested_generic() {
    let parsed: FieldThenParams = parse_quote!(HashMap<String, Vec<i32>>);
    assert_eq!(ty_str(&parsed.field.ty), "HashMap < String , Vec < i32 > >");
    assert!(parsed.params.is_empty());
}

#[test]
fn ftp_two_params() {
    let parsed: FieldThenParams = parse_quote!(Expr, precedence = 5, assoc = "left");
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "precedence");
    assert_eq!(parsed.params[1].path.to_string(), "assoc");
}

#[test]
fn ftp_param_with_bool_value() {
    let parsed: FieldThenParams = parse_quote!(Node, optional = true);
    assert_eq!(parsed.params.len(), 1);
    assert!(matches!(parsed.params[0].expr, syn::Expr::Lit(_)));
}

#[test]
fn ftp_param_with_path_value() {
    let parsed: FieldThenParams = parse_quote!(Token, kind = Left);
    assert_eq!(parsed.params.len(), 1);
    assert!(matches!(parsed.params[0].expr, syn::Expr::Path(_)));
}

#[test]
fn ftp_path_qualified_type() {
    let parsed: FieldThenParams = parse_quote!(std::collections::HashMap<String, i32>);
    let s = ty_str(&parsed.field.ty);
    assert!(s.contains("HashMap"));
    assert!(parsed.params.is_empty());
}

#[test]
fn ftp_five_params() {
    let parsed: FieldThenParams = parse_quote!(Node, a = 1, b = 2, c = 3, d = 4, e = 5);
    assert_eq!(parsed.params.len(), 5);
    let names: Vec<String> = parsed.params.iter().map(|p| p.path.to_string()).collect();
    assert_eq!(names, vec!["a", "b", "c", "d", "e"]);
}

// ===========================================================================
// 9. try_extract_inner_type — deeper nesting and edge cases
// ===========================================================================

#[test]
fn extract_through_three_skips() {
    let ty: Type = parse_quote!(Rc<Arc<Box<Option<u64>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Rc", "Arc", "Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u64");
}

#[test]
fn extract_target_is_outermost() {
    let ty: Type = parse_quote!(Vec<String>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_plain_type_no_generics() {
    let ty: Type = parse_quote!(i32);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_empty_skip_set_target_mismatch() {
    let ty: Type = parse_quote!(Box<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "Box < i32 >");
}

#[test]
fn extract_tuple_inner_type() {
    let ty: Type = parse_quote!(Vec<(u8, u16, u32)>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "(u8 , u16 , u32)");
}

#[test]
fn extract_array_inner_type() {
    let ty: Type = parse_quote!(Option<[u8; 32]>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "[u8 ; 32]");
}

#[test]
fn extract_reference_inner_type() {
    let ty: Type = parse_quote!(Box<&str>);
    let (inner, ok) = try_extract_inner_type(&ty, "Box", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "& str");
}

#[test]
fn extract_nested_same_container() {
    let ty: Type = parse_quote!(Option<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Option < bool >");
}

#[test]
fn extract_skip_same_as_target_extracts_first() {
    // When skip set contains the target, the target is matched first (not skipped)
    let ty: Type = parse_quote!(Option<i32>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Option"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "i32");
}

#[test]
fn extract_fn_pointer_type_returns_unchanged() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "fn (i32) -> bool");
}

#[test]
fn extract_raw_pointer_type_returns_unchanged() {
    let ty: Type = parse_quote!(*const u8);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "* const u8");
}

// ===========================================================================
// 10. filter_inner_type — more edge cases
// ===========================================================================

#[test]
fn filter_four_layers() {
    let ty: Type = parse_quote!(A<B<C<D<Leaf>>>>);
    let filtered = filter_inner_type(&ty, &skip(&["A", "B", "C", "D"]));
    assert_eq!(ty_str(&filtered), "Leaf");
}

#[test]
fn filter_plain_type_no_generics() {
    let ty: Type = parse_quote!(i32);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "i32");
}

#[test]
fn filter_skip_type_with_complex_inner() {
    let ty: Type = parse_quote!(Box<(String, Vec<u8>)>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(String , Vec < u8 >)");
}

#[test]
fn filter_reference_type_unchanged() {
    let ty: Type = parse_quote!(&mut Vec<u8>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "& mut Vec < u8 >");
}

#[test]
fn filter_fn_pointer_unchanged() {
    let ty: Type = parse_quote!(fn() -> i32);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "fn () -> i32");
}

#[test]
fn filter_raw_pointer_unchanged() {
    let ty: Type = parse_quote!(*const u8);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "* const u8");
}

#[test]
fn filter_array_type_unchanged() {
    let ty: Type = parse_quote!([u8; 16]);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "[u8 ; 16]");
}

#[test]
fn filter_non_skip_generic_preserved() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "HashMap < String , i32 >");
}

// ===========================================================================
// 11. wrap_leaf_type — more edge cases
// ===========================================================================

#[test]
fn wrap_tuple_type() {
    let ty: Type = parse_quote!((i32, String));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < (i32 , String) >");
}

#[test]
fn wrap_array_type() {
    let ty: Type = parse_quote!([u8; 4]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 4] >");
}

#[test]
fn wrap_fn_pointer_type() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < fn (i32) -> bool >");
}

#[test]
fn wrap_raw_pointer_type() {
    let ty: Type = parse_quote!(*const u8);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < * const u8 >");
}

#[test]
fn wrap_three_skip_layers() {
    let ty: Type = parse_quote!(A<B<C<Leaf>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["A", "B", "C"]));
    assert_eq!(
        ty_str(&wrapped),
        "A < B < C < adze :: WithLeaf < Leaf > > > >"
    );
}

#[test]
fn wrap_skip_type_multiple_args() {
    let ty: Type = parse_quote!(HashMap<Key, Value>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["HashMap"]));
    assert_eq!(
        ty_str(&wrapped),
        "HashMap < adze :: WithLeaf < Key > , adze :: WithLeaf < Value > >"
    );
}

#[test]
fn wrap_mixed_skip_and_non_skip() {
    // Vec is skip, but inner HashMap is not
    let ty: Type = parse_quote!(Vec<HashMap<K, V>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < adze :: WithLeaf < HashMap < K , V > > >"
    );
}

#[test]
fn wrap_plain_ident_type() {
    let ty: Type = parse_quote!(MyCustomType);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < MyCustomType >");
}

#[test]
fn wrap_qualified_path_type() {
    let ty: Type = parse_quote!(std::string::String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < std :: string :: String >"
    );
}

#[test]
fn wrap_mut_reference_type() {
    let ty: Type = parse_quote!(&mut i32);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & mut i32 >");
}

// ===========================================================================
// 12. Combined workflows / integration scenarios
// ===========================================================================

#[test]
fn roundtrip_extract_wrap_option() {
    let ty: Type = parse_quote!(Option<Expr>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Expr");
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Expr >");
}

#[test]
fn roundtrip_filter_then_extract() {
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < String >");
    let (inner, ok) = try_extract_inner_type(&filtered, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn roundtrip_filter_wrap() {
    let ty: Type = parse_quote!(Arc<Box<Token>>);
    let filtered = filter_inner_type(&ty, &skip(&["Arc", "Box"]));
    assert_eq!(ty_str(&filtered), "Token");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Token >");
}

#[test]
fn extract_wrap_with_skip_in_wrap() {
    let ty: Type = parse_quote!(Option<Vec<Leaf>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Vec < Leaf >");
    let wrapped = wrap_leaf_type(&inner, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < Leaf > >");
}

#[test]
fn double_extract() {
    let ty: Type = parse_quote!(Option<Vec<i32>>);
    let (inner1, ok1) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok1);
    assert_eq!(ty_str(&inner1), "Vec < i32 >");
    let (inner2, ok2) = try_extract_inner_type(&inner1, "Vec", &skip(&[]));
    assert!(ok2);
    assert_eq!(ty_str(&inner2), "i32");
}

#[test]
fn filter_and_extract_equivalence_nested() {
    let ty: Type = parse_quote!(Box<Arc<Vec<u8>>>);
    // Filter strips Box and Arc
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
    // Extract through Box and Arc finds Vec
    let (extracted, ok) = try_extract_inner_type(&ty, "Vec", &skip(&["Box", "Arc"]));
    assert!(ok);
    // Extract gives the inner of Vec
    assert_eq!(ty_str(&extracted), "u8");
}

// ===========================================================================
// 13. NameValueExpr — structural equality and cloning
// ===========================================================================

#[test]
fn nve_clone_preserves_equality() {
    let nv: NameValueExpr = parse_quote!(key = 42);
    let cloned = nv.clone();
    assert_eq!(nv, cloned);
}

#[test]
fn nve_different_values_not_equal() {
    let nv1: NameValueExpr = parse_quote!(key = 1);
    let nv2: NameValueExpr = parse_quote!(key = 2);
    assert_ne!(nv1, nv2);
}

#[test]
fn nve_different_names_not_equal() {
    let nv1: NameValueExpr = parse_quote!(alpha = 1);
    let nv2: NameValueExpr = parse_quote!(beta = 1);
    assert_ne!(nv1, nv2);
}

// ===========================================================================
// 14. FieldThenParams — structural equality and cloning
// ===========================================================================

#[test]
fn ftp_clone_preserves_equality() {
    let parsed: FieldThenParams = parse_quote!(u32, min = 0);
    let cloned = parsed.clone();
    assert_eq!(parsed, cloned);
}

#[test]
fn ftp_different_types_not_equal() {
    let ftp1: FieldThenParams = parse_quote!(u32);
    let ftp2: FieldThenParams = parse_quote!(i32);
    assert_ne!(ftp1, ftp2);
}

#[test]
fn ftp_with_and_without_params_not_equal() {
    let ftp1: FieldThenParams = parse_quote!(u32);
    let ftp2: FieldThenParams = parse_quote!(u32, x = 1);
    assert_ne!(ftp1, ftp2);
}

// ===========================================================================
// 15. Idempotency and identity properties
// ===========================================================================

#[test]
fn filter_idempotent_after_full_unwrap() {
    let ty: Type = parse_quote!(Box<String>);
    let filtered_once = filter_inner_type(&ty, &skip(&["Box"]));
    let filtered_twice = filter_inner_type(&filtered_once, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered_once), ty_str(&filtered_twice));
}

#[test]
fn extract_failure_preserves_type_identity() {
    let ty: Type = parse_quote!(String);
    let (result, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&result), ty_str(&ty));
}

#[test]
fn filter_empty_skip_is_identity() {
    let ty: Type = parse_quote!(Vec<Option<Box<i32>>>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&filtered), ty_str(&ty));
}

#[test]
fn wrap_with_empty_skip_wraps_everything() {
    let ty: Type = parse_quote!(Option<i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Option < i32 > >");
}

// ===========================================================================
// 16. Type-specific edge cases
// ===========================================================================

#[test]
fn extract_from_result_type() {
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, ok) = try_extract_inner_type(&ty, "Result", &skip(&[]));
    assert!(ok);
    // Result's first generic arg is String
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn filter_never_type_unchanged() {
    let ty: Type = parse_quote!(!);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "!");
}

#[test]
fn wrap_never_type() {
    let ty: Type = parse_quote!(!);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < ! >");
}

#[test]
fn extract_from_deeply_qualified_path() {
    let ty: Type = parse_quote!(std::option::Option<u32>);
    // Only matches last segment, so "Option" should match
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn filter_qualified_skip_type() {
    let ty: Type = parse_quote!(std::boxed::Box<i64>);
    // filter_inner_type matches on last segment
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "i64");
}

#[test]
fn wrap_preserves_lifetime_in_reference() {
    let ty: Type = parse_quote!(&'a str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & 'a str >");
}
