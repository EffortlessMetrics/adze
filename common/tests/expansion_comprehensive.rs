//! Comprehensive tests for the adze-common crate's public API.
//!
//! Covers the full expansion pipeline: parsing (`NameValueExpr`, `FieldThenParams`),
//! type extraction (`try_extract_inner_type`), wrapper removal (`filter_inner_type`),
//! and leaf wrapping (`wrap_leaf_type`), with emphasis on compositional properties
//! and real-world grammar expansion patterns.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. Full pipeline: extract → filter → wrap mirrors real grammar expansion
// ===========================================================================

#[test]
fn pipeline_optional_vec_box_node() {
    // Simulates processing a grammar field: Option<Vec<Box<Node>>>
    let ty: Type = parse_quote!(Option<Vec<Box<Node>>>);

    // Step 1: extract Option (skip over nothing)
    let (after_opt, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&after_opt), "Vec < Box < Node > >");

    // Step 2: extract Vec (skip over nothing)
    let (after_vec, found) = try_extract_inner_type(&after_opt, "Vec", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&after_vec), "Box < Node >");

    // Step 3: filter Box away
    let filtered = filter_inner_type(&after_vec, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Node");

    // Step 4: wrap the leaf
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Node >");
}

#[test]
fn pipeline_wrap_entire_optional_vec_with_skip_set() {
    // wrap_leaf_type with Vec+Option in skip preserves containers, wraps leaf
    let ty: Type = parse_quote!(Option<Vec<Token>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < adze :: WithLeaf < Token > > >"
    );
}

// ===========================================================================
// 2. filter_inner_type is truly idempotent after convergence
// ===========================================================================

#[test]
fn filter_idempotent_after_convergence() {
    let ty: Type = parse_quote!(Box<Arc<Vec<Leaf>>>);
    let s = skip(&["Box", "Arc"]);

    let first = filter_inner_type(&ty, &s);
    let second = filter_inner_type(&first, &s);
    let third = filter_inner_type(&second, &s);
    assert_eq!(ty_str(&first), "Vec < Leaf >");
    assert_eq!(ty_str(&second), ty_str(&first));
    assert_eq!(ty_str(&third), ty_str(&first));
}

// ===========================================================================
// 3. extract + filter composition: extract through skip, then filter residual
// ===========================================================================

#[test]
fn extract_through_skip_then_filter_residual_wrapper() {
    // Arc<Box<Option<Vec<Leaf>>>>
    // extract Option skipping Arc+Box → Vec<Leaf>
    // filter Vec → Leaf (if Vec in filter skip)
    let ty: Type = parse_quote!(Arc<Box<Option<Vec<Leaf>>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Arc", "Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Vec < Leaf >");

    let filtered = filter_inner_type(&inner, &skip(&["Vec"]));
    assert_eq!(ty_str(&filtered), "Leaf");
}

// ===========================================================================
// 4. NameValueExpr: various expression forms and trait impls
// ===========================================================================

#[test]
fn name_value_expr_method_call() {
    let nv: NameValueExpr = parse_quote!(init = Vec::new());
    assert_eq!(nv.path.to_string(), "init");
    // Vec::new() is a path-call expression
    assert!(matches!(nv.expr, syn::Expr::Call(_)));
}

#[test]
fn name_value_expr_array_literal() {
    let nv: NameValueExpr = parse_quote!(tokens = [1, 2, 3]);
    assert_eq!(nv.path.to_string(), "tokens");
    assert!(matches!(nv.expr, syn::Expr::Array(_)));
}

#[test]
fn name_value_expr_eq_and_clone_consistency() {
    let a: NameValueExpr = parse_quote!(key = "val");
    let b = a.clone();
    // PartialEq is derived; cloned value should be equal
    assert_eq!(a, b);
}

// ===========================================================================
// 5. FieldThenParams: complex field types with transform closures
// ===========================================================================

#[test]
fn field_then_params_with_transform_closure() {
    let parsed: FieldThenParams = parse_quote!(
        String,
        pattern = r"\d+",
        transform = |s: String| s.parse::<u64>().unwrap()
    );
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "pattern");
    assert_eq!(parsed.params[1].path.to_string(), "transform");
    assert!(matches!(parsed.params[1].expr, syn::Expr::Closure(_)));
}

#[test]
fn field_then_params_eq_and_clone() {
    let a: FieldThenParams = parse_quote!(i32, min = 0);
    let b = a.clone();
    assert_eq!(a, b);
}

// ===========================================================================
// 6. wrap_leaf_type: wrapping preserves all inner generic args in skip types
// ===========================================================================

#[test]
fn wrap_result_in_skip_wraps_all_type_args() {
    // Result<A, B> with Result in skip → both A and B are wrapped
    let ty: Type = parse_quote!(Result<Success, Failure>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < Success > , adze :: WithLeaf < Failure > >"
    );
}

// ===========================================================================
// 7. Qualified paths: last-segment matching for all three functions
// ===========================================================================

#[test]
fn qualified_path_extract_matches_last_segment() {
    let ty: Type = parse_quote!(std::option::Option<Leaf>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "Leaf");
}

#[test]
fn qualified_path_filter_matches_last_segment() {
    let ty: Type = parse_quote!(std::boxed::Box<std::sync::Arc<Inner>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    assert_eq!(ty_str(&filtered), "Inner");
}

#[test]
fn qualified_path_wrap_matches_last_segment() {
    let ty: Type = parse_quote!(std::vec::Vec<Item>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "std :: vec :: Vec < adze :: WithLeaf < Item > >"
    );
}

// ===========================================================================
// 8. extract returns original type (not inner) when target is absent
// ===========================================================================

#[test]
fn extract_absent_target_through_deep_skips_returns_outermost() {
    // Box<Arc<Rc<String>>> — looking for Option which isn't there
    let ty: Type = parse_quote!(Box<Arc<Rc<String>>>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc", "Rc"]));
    assert!(!found);
    // Should return the *original* outer type, not the innermost
    assert_eq!(ty_str(&result), "Box < Arc < Rc < String > > >");
}

// ===========================================================================
// 9. Batch processing: simulate processing many fields in a grammar struct
// ===========================================================================

#[test]
fn batch_process_grammar_fields() {
    let fields: Vec<(&str, Type)> = vec![
        ("name", parse_quote!(String)),
        ("body", parse_quote!(Vec<Statement>)),
        ("ret", parse_quote!(Option<Box<Expr>>)),
        ("decorators", parse_quote!(Vec<Option<Decorator>>)),
    ];

    let extract_skip = skip(&["Box"]);
    let wrap_skip = skip(&["Vec", "Option"]);

    let mut results = Vec::new();
    for (name, ty) in &fields {
        // Check if optional
        let (after_opt, is_optional) = try_extract_inner_type(ty, "Option", &extract_skip);
        let wrapped = wrap_leaf_type(if is_optional { &after_opt } else { ty }, &wrap_skip);
        results.push((*name, is_optional, ty_str(&wrapped)));
    }

    assert_eq!(
        results[0],
        ("name", false, "adze :: WithLeaf < String >".to_string())
    );
    assert_eq!(
        results[1],
        (
            "body",
            false,
            "Vec < adze :: WithLeaf < Statement > >".to_string()
        )
    );
    assert!(results[2].1); // ret is optional
    // After extracting Option (skipping Box), inner is Box<Expr>; wrap wraps it whole
    assert_eq!(results[2].2, "adze :: WithLeaf < Box < Expr > >");
    assert!(!results[3].1); // decorators: outer Vec, not Option at top
}

// ===========================================================================
// 10. fn pointer and slice types as non-path forms
// ===========================================================================

#[test]
fn fn_pointer_type_wrap() {
    let ty: Type = parse_quote!(fn(i32) -> bool);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert!(ty_str(&wrapped).contains("adze :: WithLeaf"));
}

#[test]
fn slice_type_wrap() {
    let ty: Type = parse_quote!([u8]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8] >");
}

// ===========================================================================
// 11. NameValueExpr Debug includes struct name
// ===========================================================================

#[test]
fn name_value_expr_debug_format() {
    let nv: NameValueExpr = parse_quote!(key = 42);
    let dbg = format!("{nv:?}");
    assert!(dbg.contains("NameValueExpr"));
    assert!(dbg.contains("path"));
}

// ===========================================================================
// 12. FieldThenParams with only a trailing comma (no params)
// ===========================================================================

#[test]
fn field_then_params_trailing_comma_no_params() {
    // A trailing comma with no params should parse with comma but empty params
    let parsed: FieldThenParams = parse_quote!(Token,);
    assert!(parsed.comma.is_some());
    assert!(parsed.params.is_empty());
}
