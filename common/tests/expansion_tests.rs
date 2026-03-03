//! Tests for grammar expansion logic in adze-common.
//!
//! Covers annotation patterns, error handling, edge cases,
//! and shared utility functions used by both macro and tool crates.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Expr, Type, parse_quote};

// ---------------------------------------------------------------------------
// NameValueExpr parsing – annotation parameter patterns
// ---------------------------------------------------------------------------

#[test]
fn name_value_expr_with_integer_literal() {
    let expr: NameValueExpr = parse_quote!(precedence = 5);
    assert_eq!(expr.path.to_string(), "precedence");
    assert!(matches!(expr.expr, Expr::Lit(_)));
}

#[test]
fn name_value_expr_with_string_literal() {
    let expr: NameValueExpr = parse_quote!(pattern = "\\d+");
    assert_eq!(expr.path.to_string(), "pattern");
    assert!(matches!(expr.expr, Expr::Lit(_)));
}

#[test]
fn name_value_expr_with_bool_literal() {
    let expr: NameValueExpr = parse_quote!(inline = true);
    assert_eq!(expr.path.to_string(), "inline");
}

#[test]
fn name_value_expr_with_closure() {
    let expr: NameValueExpr = parse_quote!(transform = |v: String| v.parse::<i32>().unwrap());
    assert_eq!(expr.path.to_string(), "transform");
    assert!(matches!(expr.expr, Expr::Closure(_)));
}

#[test]
fn name_value_expr_with_path_value() {
    let expr: NameValueExpr = parse_quote!(kind = SomeEnum::Variant);
    assert_eq!(expr.path.to_string(), "kind");
    assert!(matches!(expr.expr, Expr::Path(_)));
}

#[test]
fn name_value_expr_with_negative_literal() {
    let expr: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(expr.path.to_string(), "offset");
    assert!(matches!(expr.expr, Expr::Unary(_)));
}

#[test]
fn name_value_expr_with_block() {
    let expr: NameValueExpr = parse_quote!(init = { Vec::new() });
    assert_eq!(expr.path.to_string(), "init");
    assert!(matches!(expr.expr, Expr::Block(_)));
}

#[test]
fn name_value_expr_preserves_eq_token() {
    let expr: NameValueExpr = parse_quote!(key = 42);
    // The eq_token is present (structurally required by the parser).
    let _ = expr.eq_token;
}

// ---------------------------------------------------------------------------
// FieldThenParams parsing – field declarations with optional parameters
// ---------------------------------------------------------------------------

#[test]
fn field_then_params_bare_type() {
    let parsed: FieldThenParams = parse_quote!(u32);
    assert!(parsed.comma.is_none());
    assert!(parsed.params.is_empty());
}

#[test]
fn field_then_params_generic_type_no_params() {
    let parsed: FieldThenParams = parse_quote!(Vec<String>);
    assert!(parsed.comma.is_none());
    assert!(parsed.params.is_empty());
}

#[test]
fn field_then_params_complex_generic() {
    let parsed: FieldThenParams = parse_quote!(HashMap<String, Vec<i32>>);
    assert!(parsed.comma.is_none());
    assert!(parsed.params.is_empty());
}

#[test]
fn field_then_params_single_param() {
    let parsed: FieldThenParams = parse_quote!(String, pattern = r"\w+");
    assert!(parsed.comma.is_some());
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "pattern");
}

#[test]
fn field_then_params_three_params() {
    let parsed: FieldThenParams = parse_quote!(Vec<u8>, min = 0, max = 255, separator = ",");
    assert_eq!(parsed.params.len(), 3);
    assert_eq!(parsed.params[0].path.to_string(), "min");
    assert_eq!(parsed.params[1].path.to_string(), "max");
    assert_eq!(parsed.params[2].path.to_string(), "separator");
}

#[test]
fn field_then_params_option_type_with_param() {
    let parsed: FieldThenParams = parse_quote!(Option<String>, default = "none");
    assert!(parsed.comma.is_some());
    assert_eq!(parsed.params.len(), 1);
}

#[test]
fn field_then_params_nested_generic_with_params() {
    let parsed: FieldThenParams = parse_quote!(Box<Option<Vec<i32>>>, flatten = true);
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "flatten");
}

// ---------------------------------------------------------------------------
// try_extract_inner_type – type extraction through wrapper layers
// ---------------------------------------------------------------------------

#[test]
fn extract_inner_direct_match() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_inner_no_match_returns_original() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<i32>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Vec < i32 >");
}

#[test]
fn extract_inner_skip_over_single_wrapper() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Option<u8>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "u8");
}

#[test]
fn extract_inner_skip_over_multiple_wrappers() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Vec<bool>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "bool");
}

#[test]
fn extract_inner_target_not_present_through_skips() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    // When extraction fails through a skip_over wrapper, the *original* type is returned.
    assert_eq!(inner.to_token_stream().to_string(), "Box < String >");
}

#[test]
fn extract_inner_plain_type_no_generics() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(String);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn extract_inner_non_path_type_reference() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "& str");
}

#[test]
fn extract_inner_non_path_type_tuple() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!((i32, String));
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "(i32 , String)");
}

#[test]
fn extract_inner_non_path_type_array() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!([u8; 4]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "[u8 ; 4]");
}

#[test]
fn extract_inner_deeply_nested_skip() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<f64>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "f64");
}

// ---------------------------------------------------------------------------
// filter_inner_type – stripping container wrappers
// ---------------------------------------------------------------------------

#[test]
fn filter_single_wrapper() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(
        filter_inner_type(&ty, &skip).to_token_stream().to_string(),
        "String"
    );
}

#[test]
fn filter_nested_wrappers() {
    let skip: HashSet<&str> = ["Box", "Arc", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Option<u32>>>);
    assert_eq!(
        filter_inner_type(&ty, &skip).to_token_stream().to_string(),
        "u32"
    );
}

#[test]
fn filter_stops_at_non_skip_type() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<i32>>);
    assert_eq!(
        filter_inner_type(&ty, &skip).to_token_stream().to_string(),
        "Vec < i32 >"
    );
}

#[test]
fn filter_no_wrappers_returns_original() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(String);
    assert_eq!(
        filter_inner_type(&ty, &skip).to_token_stream().to_string(),
        "String"
    );
}

#[test]
fn filter_empty_skip_set() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Box<String>);
    assert_eq!(
        filter_inner_type(&ty, &skip).to_token_stream().to_string(),
        "Box < String >"
    );
}

#[test]
fn filter_non_path_type_passthrough() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        filter_inner_type(&ty, &skip).to_token_stream().to_string(),
        "& str"
    );
}

#[test]
fn filter_is_idempotent() {
    let skip: HashSet<&str> = ["Box", "Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<Option<String>>>);
    let once = filter_inner_type(&ty, &skip);
    let twice = filter_inner_type(&once, &skip);
    assert_eq!(
        once.to_token_stream().to_string(),
        twice.to_token_stream().to_string()
    );
}

#[test]
fn filter_qualified_path() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    // Only the last segment is checked, so std::option::Option should work.
    let ty: Type = parse_quote!(std::option::Option<bool>);
    assert_eq!(
        filter_inner_type(&ty, &skip).to_token_stream().to_string(),
        "bool"
    );
}

// ---------------------------------------------------------------------------
// wrap_leaf_type – wrapping inner types with adze::WithLeaf
// ---------------------------------------------------------------------------

#[test]
fn wrap_simple_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(i32);
    assert_eq!(
        wrap_leaf_type(&ty, &skip).to_token_stream().to_string(),
        "adze :: WithLeaf < i32 >"
    );
}

#[test]
fn wrap_preserves_vec_wrapper() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<String>);
    assert_eq!(
        wrap_leaf_type(&ty, &skip).to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < String > >"
    );
}

#[test]
fn wrap_preserves_option_wrapper() {
    let skip: HashSet<&str> = ["Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<u64>);
    assert_eq!(
        wrap_leaf_type(&ty, &skip).to_token_stream().to_string(),
        "Option < adze :: WithLeaf < u64 > >"
    );
}

#[test]
fn wrap_preserves_nested_skip_types() {
    let skip: HashSet<&str> = ["Vec", "Option"].into_iter().collect();
    let ty: Type = parse_quote!(Option<Vec<bool>>);
    assert_eq!(
        wrap_leaf_type(&ty, &skip).to_token_stream().to_string(),
        "Option < Vec < adze :: WithLeaf < bool > > >"
    );
}

#[test]
fn wrap_type_not_in_skip_set() {
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    // HashMap is not in the skip set, so the entire type gets wrapped.
    let ty: Type = parse_quote!(HashMap<String, i32>);
    assert_eq!(
        wrap_leaf_type(&ty, &skip).to_token_stream().to_string(),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

#[test]
fn wrap_reference_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(&str);
    assert_eq!(
        wrap_leaf_type(&ty, &skip).to_token_stream().to_string(),
        "adze :: WithLeaf < & str >"
    );
}

#[test]
fn wrap_tuple_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!((i32, String));
    assert_eq!(
        wrap_leaf_type(&ty, &skip).to_token_stream().to_string(),
        "adze :: WithLeaf < (i32 , String) >"
    );
}

#[test]
fn wrap_empty_skip_set_wraps_everything() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Vec<String>);
    // Vec is NOT in skip set, so the entire Vec<String> is wrapped.
    assert_eq!(
        wrap_leaf_type(&ty, &skip).to_token_stream().to_string(),
        "adze :: WithLeaf < Vec < String > >"
    );
}

#[test]
fn wrap_deeply_nested_skip_types() {
    let skip: HashSet<&str> = ["Vec", "Option", "Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Option<Vec<f32>>>);
    assert_eq!(
        wrap_leaf_type(&ty, &skip).to_token_stream().to_string(),
        "Box < Option < Vec < adze :: WithLeaf < f32 > > > >"
    );
}

// ---------------------------------------------------------------------------
// Combined / interaction tests – simulating grammar expansion patterns
// ---------------------------------------------------------------------------

#[test]
fn extract_then_wrap_roundtrip() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let wrap_skip: HashSet<&str> = ["Vec"].into_iter().collect();

    // Simulate: grammar field is Box<Option<String>>
    // 1. Extract Option to get String
    let ty: Type = parse_quote!(Box<Option<String>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");

    // 2. Wrap the extracted inner type
    let wrapped = wrap_leaf_type(&inner, &wrap_skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < String >"
    );
}

#[test]
fn filter_then_wrap_roundtrip() {
    let filter_skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();
    let wrap_skip: HashSet<&str> = ["Vec"].into_iter().collect();

    // Strip Box<Arc<...>> then wrap the leaf
    let ty: Type = parse_quote!(Box<Arc<Vec<u16>>>);
    let filtered = filter_inner_type(&ty, &filter_skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Vec < u16 >");

    let wrapped = wrap_leaf_type(&filtered, &wrap_skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < u16 > >"
    );
}

#[test]
fn grammar_field_with_all_wrappers() {
    // Simulate processing a grammar field: Option<Vec<Box<MyNode>>>
    let extract_skip: HashSet<&str> = ["Box"].into_iter().collect();
    let wrap_skip: HashSet<&str> = ["Vec"].into_iter().collect();

    let ty: Type = parse_quote!(Option<Vec<Box<MyNode>>>);

    // Extract Option
    let (after_option, did_extract) = try_extract_inner_type(&ty, "Option", &extract_skip);
    assert!(did_extract);
    assert_eq!(
        after_option.to_token_stream().to_string(),
        "Vec < Box < MyNode > >"
    );

    // Filter Box
    let filter_skip: HashSet<&str> = ["Vec", "Box"].into_iter().collect();
    let filtered = filter_inner_type(&after_option, &filter_skip);
    assert_eq!(filtered.to_token_stream().to_string(), "MyNode");

    // Wrap for leaf
    let wrapped = wrap_leaf_type(&filtered, &wrap_skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < MyNode >"
    );
}

// ---------------------------------------------------------------------------
// Edge cases – empty / degenerate inputs
// ---------------------------------------------------------------------------

#[test]
fn filter_with_unit_type() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(());
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "()");
}

#[test]
fn wrap_unit_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < () >"
    );
}

#[test]
fn extract_from_unit_type() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(());
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "()");
}

#[test]
fn extract_inner_self_referential_type_name() {
    // When the target name matches itself as a wrapper (not really recursive,
    // but makes sure we don't infinite loop).
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(Foo<Bar>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Foo", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Bar");
}

#[test]
fn field_then_params_preserves_field_type_fidelity() {
    let parsed: FieldThenParams = parse_quote!(Option<Vec<String>>, separator = ",");
    let field_ty = &parsed.field.ty;
    assert_eq!(
        field_ty.to_token_stream().to_string(),
        "Option < Vec < String > >"
    );
}

#[test]
fn name_value_expr_clone_and_eq() {
    let expr: NameValueExpr = parse_quote!(key = "value");
    let cloned = expr.clone();
    assert_eq!(expr.path.to_string(), cloned.path.to_string());
}

#[test]
fn name_value_expr_debug_impl() {
    let expr: NameValueExpr = parse_quote!(key = 42);
    let debug_str = format!("{:?}", expr);
    assert!(debug_str.contains("NameValueExpr"));
}

#[test]
fn field_then_params_debug_impl() {
    let parsed: FieldThenParams = parse_quote!(String, key = 1);
    let debug_str = format!("{:?}", parsed);
    assert!(debug_str.contains("FieldThenParams"));
}

// ---------------------------------------------------------------------------
// Qualified / multi-segment paths
// ---------------------------------------------------------------------------

#[test]
fn extract_inner_qualified_path() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(std::option::Option<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}

#[test]
fn wrap_qualified_skip_type() {
    // Only the last segment is checked against the skip set.
    let skip: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    assert_eq!(
        wrap_leaf_type(&ty, &skip).to_token_stream().to_string(),
        "std :: vec :: Vec < adze :: WithLeaf < u8 > >"
    );
}

// ---------------------------------------------------------------------------
// Multiple generic arguments (only first is used by implementation)
// ---------------------------------------------------------------------------

#[test]
fn extract_inner_from_result_like_type() {
    let skip: HashSet<&str> = HashSet::new();
    // Result<T, E> has two generic args; try_extract_inner_type uses the first.
    let ty: Type = parse_quote!(Result<String, Error>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Result", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");
}
