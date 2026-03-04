#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for type expansion logic in adze-common.
//!
//! Covers try_extract_inner_type, filter_inner_type, wrap_leaf_type,
//! NameValueExpr parsing, and FieldThenParams parsing with diverse
//! type shapes, skip-set configurations, and annotation combinations.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip_set<'a>(names: &[&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn to_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. try_extract_inner_type — deeply nested skip chains
// ===========================================================================

#[test]
fn extract_through_three_skip_layers() {
    // Rc<Arc<Box<Option<u8>>>> with Rc, Arc, Box in skip, target = Option
    let ty: Type = parse_quote!(Rc<Arc<Box<Option<u8>>>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&["Rc", "Arc", "Box"]));
    assert!(ok);
    assert_eq!(to_str(&inner), "u8");
}

#[test]
fn extract_stops_at_first_non_skip_without_target() {
    // Arc<HashMap<String, i32>> — Arc skipped, HashMap is not skip nor target
    let ty: Type = parse_quote!(Arc<HashMap<String, i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Arc"]));
    assert!(!ok);
    assert_eq!(to_str(&inner), "Arc < HashMap < String , i32 > >");
}

#[test]
fn extract_target_is_outermost_type() {
    // Vec<Node> where target = Vec, no skip
    let ty: Type = parse_quote!(Vec<Node>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(to_str(&inner), "Node");
}

// ===========================================================================
// 2. try_extract_inner_type — target type with complex inner
// ===========================================================================

#[test]
fn extract_option_containing_tuple() {
    let ty: Type = parse_quote!(Option<(String, i32)>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(to_str(&inner), "(String , i32)");
}

#[test]
fn extract_vec_containing_option() {
    // Extract Vec's inner even when it's Option<T>
    let ty: Type = parse_quote!(Vec<Option<bool>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&[]));
    assert!(ok);
    assert_eq!(to_str(&inner), "Option < bool >");
}

// ===========================================================================
// 3. try_extract_inner_type — same type in skip and target
// ===========================================================================

#[test]
fn extract_target_in_skip_set_matches_outermost() {
    // If "Option" is both target and in skip_set, the outermost match wins
    // because the target check happens before the skip check.
    let ty: Type = parse_quote!(Option<Option<i32>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&["Option"]));
    assert!(ok);
    assert_eq!(to_str(&inner), "Option < i32 >");
}

// ===========================================================================
// 4. try_extract_inner_type — qualified / multi-segment paths
// ===========================================================================

#[test]
fn extract_ignores_qualified_path_segments() {
    // std::option::Option<T> — last segment is "Option", should match
    let ty: Type = parse_quote!(std::option::Option<u64>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(to_str(&inner), "u64");
}

#[test]
fn extract_qualified_skip_matches_last_segment() {
    // std::boxed::Box<Vec<u8>> — last segment "Box" is in skip
    let ty: Type = parse_quote!(std::boxed::Box<Vec<u8>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set(&["Box"]));
    assert!(ok);
    assert_eq!(to_str(&inner), "u8");
}

// ===========================================================================
// 5. filter_inner_type — deeply nested and mixed
// ===========================================================================

#[test]
fn filter_four_layers_all_skipped() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Cell<Leaf>>>>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box", "Arc", "Rc", "Cell"]));
    assert_eq!(to_str(&filtered), "Leaf");
}

#[test]
fn filter_stops_at_unknown_wrapper() {
    // Mutex not in skip, so stops there
    let ty: Type = parse_quote!(Box<Mutex<Inner>>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(to_str(&filtered), "Mutex < Inner >");
}

#[test]
fn filter_qualified_path_skip_matches_last_segment() {
    let ty: Type = parse_quote!(std::sync::Arc<Data>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Arc"]));
    assert_eq!(to_str(&filtered), "Data");
}

// ===========================================================================
// 6. filter_inner_type — non-generic types in skip set
// ===========================================================================

#[test]
fn filter_plain_type_not_in_skip_is_identity() {
    let ty: Type = parse_quote!(MyStruct);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box", "Option"]));
    assert_eq!(to_str(&filtered), "MyStruct");
}

// ===========================================================================
// 7. wrap_leaf_type — deeply nested skip chain
// ===========================================================================

#[test]
fn wrap_four_level_skip_wraps_innermost() {
    let ty: Type = parse_quote!(Box<Option<Vec<Arc<Leaf>>>>);
    let skip = &skip_set(&["Box", "Option", "Vec", "Arc"]);
    let wrapped = wrap_leaf_type(&ty, skip);
    assert_eq!(
        to_str(&wrapped),
        "Box < Option < Vec < Arc < adze :: WithLeaf < Leaf > > > > >"
    );
}

#[test]
fn wrap_only_outer_in_skip_wraps_inner_generics_entirely() {
    // Vec is in skip but HashMap is not, so HashMap<K,V> gets wrapped as a whole
    let ty: Type = parse_quote!(Vec<HashMap<String, i32>>);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&["Vec"]));
    assert_eq!(
        to_str(&wrapped),
        "Vec < adze :: WithLeaf < HashMap < String , i32 > > >"
    );
}

// ===========================================================================
// 8. wrap_leaf_type — non-path types
// ===========================================================================

#[test]
fn wrap_reference_type() {
    let ty: Type = parse_quote!(&'static str);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(to_str(&wrapped), "adze :: WithLeaf < & 'static str >");
}

#[test]
fn wrap_slice_type() {
    let ty: Type = parse_quote!([u8]);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(to_str(&wrapped), "adze :: WithLeaf < [u8] >");
}

#[test]
fn wrap_fn_pointer_type() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    let wrapped = wrap_leaf_type(&ty, &skip_set(&[]));
    assert_eq!(to_str(&wrapped), "adze :: WithLeaf < fn (i32) -> bool >");
}

// ===========================================================================
// 9. Consistency: filter then extract, extract then wrap
// ===========================================================================

#[test]
fn filter_then_extract_pipeline() {
    // Box<Option<Vec<Token>>> — filter Box, then extract Vec
    let ty: Type = parse_quote!(Box<Option<Vec<Token>>>);
    let filtered = filter_inner_type(&ty, &skip_set(&["Box"]));
    assert_eq!(to_str(&filtered), "Option < Vec < Token > >");

    let (inner, ok) = try_extract_inner_type(&filtered, "Vec", &skip_set(&["Option"]));
    assert!(ok);
    assert_eq!(to_str(&inner), "Token");
}

#[test]
fn extract_then_wrap_pipeline() {
    // Option<Expr> — extract Option, then wrap leaf
    let ty: Type = parse_quote!(Option<Expr>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_set(&[]));
    assert!(ok);
    assert_eq!(to_str(&inner), "Expr");

    let wrapped = wrap_leaf_type(&inner, &skip_set(&[]));
    assert_eq!(to_str(&wrapped), "adze :: WithLeaf < Expr >");
}

// ===========================================================================
// 10. NameValueExpr — diverse value expression types
// ===========================================================================

#[test]
fn name_value_expr_negative_integer() {
    let nv: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nv.path.to_string(), "offset");
    // Negative literal is a Unary expression
    assert!(matches!(nv.expr, syn::Expr::Unary(_)));
}

#[test]
fn name_value_expr_method_call() {
    let nv: NameValueExpr = parse_quote!(default = String::new());
    assert_eq!(nv.path.to_string(), "default");
    assert!(matches!(nv.expr, syn::Expr::Call(_)));
}

#[test]
fn name_value_expr_array_value() {
    let nv: NameValueExpr = parse_quote!(values = [1, 2, 3]);
    assert_eq!(nv.path.to_string(), "values");
    assert!(matches!(nv.expr, syn::Expr::Array(_)));
}

// ===========================================================================
// 11. FieldThenParams — generic field types
// ===========================================================================

#[test]
fn field_then_params_option_field() {
    let parsed: FieldThenParams = parse_quote!(Option<Identifier>);
    assert_eq!(to_str(&parsed.field.ty), "Option < Identifier >");
    assert!(parsed.params.is_empty());
}

#[test]
fn field_then_params_nested_generic_field() {
    let parsed: FieldThenParams = parse_quote!(Vec<Option<Token>>, separator = ",");
    assert_eq!(to_str(&parsed.field.ty), "Vec < Option < Token > >");
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "separator");
}

// ===========================================================================
// 12. FieldThenParams — multiple params ordering
// ===========================================================================

#[test]
fn field_then_params_three_params_order_preserved() {
    let parsed: FieldThenParams = parse_quote!(
        Expr,
        precedence = 5,
        associativity = "left",
        pattern = "\\+"
    );
    assert_eq!(parsed.params.len(), 3);
    let names: Vec<String> = parsed.params.iter().map(|p| p.path.to_string()).collect();
    assert_eq!(names, vec!["precedence", "associativity", "pattern"]);
}

// ===========================================================================
// 13. Idempotency and symmetry properties
// ===========================================================================

#[test]
fn filter_idempotent_on_simple_type() {
    let skip = skip_set(&["Box"]);
    let ty: Type = parse_quote!(String);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(to_str(&once), to_str(&twice));
}

#[test]
fn filter_idempotent_after_unwrap() {
    let skip = skip_set(&["Box"]);
    let ty: Type = parse_quote!(Box<String>);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(to_str(&once), "String");
    assert_eq!(to_str(&once), to_str(&twice));
}

#[test]
fn extract_false_for_plain_type_regardless_of_target() {
    let targets = ["Option", "Vec", "Box", "Result", "Arc"];
    let ty: Type = parse_quote!(i64);
    for target in &targets {
        let (_, ok) = try_extract_inner_type(&ty, target, &skip_set(&[]));
        assert!(
            !ok,
            "Should not extract from plain type for target {target}"
        );
    }
}

// ===========================================================================
// 14. Batch processing — simulating field expansion over a struct
// ===========================================================================

#[test]
fn batch_wrap_multiple_field_types() {
    let skip = skip_set(&["Vec", "Option"]);
    let field_types: Vec<Type> = vec![
        parse_quote!(String),
        parse_quote!(Vec<Token>),
        parse_quote!(Option<Expr>),
        parse_quote!(i32),
    ];

    let expected = [
        "adze :: WithLeaf < String >",
        "Vec < adze :: WithLeaf < Token > >",
        "Option < adze :: WithLeaf < Expr > >",
        "adze :: WithLeaf < i32 >",
    ];

    for i in 0..field_types.len() {
        let wrapped = wrap_leaf_type(&field_types[i], &skip);
        assert_eq!(to_str(&wrapped), expected[i]);
    }
}

#[test]
fn batch_extract_option_from_mixed_fields() {
    let skip = skip_set(&[]);
    let field_types: Vec<Type> = vec![
        parse_quote!(Option<String>),
        parse_quote!(Vec<i32>),
        parse_quote!(Option<Token>),
        parse_quote!(bool),
    ];

    let results: Vec<(String, bool)> = field_types
        .iter()
        .map(|ty| {
            let (inner, ok) = try_extract_inner_type(ty, "Option", &skip);
            (to_str(&inner), ok)
        })
        .collect();

    assert_eq!(results[0], ("String".to_string(), true));
    assert!(!results[1].1);
    assert_eq!(results[2], ("Token".to_string(), true));
    assert!(!results[3].1);
}

// ===========================================================================
// 15. NameValueExpr — Debug/Clone/PartialEq traits
// ===========================================================================

#[test]
fn name_value_expr_clone_and_eq() {
    let nv: NameValueExpr = parse_quote!(key = "value");
    let cloned = nv.clone();
    assert_eq!(nv, cloned);
}

#[test]
fn field_then_params_clone_and_eq() {
    let parsed: FieldThenParams = parse_quote!(Expr, precedence = 5);
    let cloned = parsed.clone();
    assert_eq!(parsed, cloned);
}

#[test]
fn name_value_expr_debug_contains_field_name() {
    let nv: NameValueExpr = parse_quote!(pattern = "abc");
    let debug = format!("{nv:?}");
    assert!(debug.contains("NameValueExpr"));
    assert!(debug.contains("path"));
}
