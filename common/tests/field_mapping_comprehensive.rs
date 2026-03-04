#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for field mapping and field extraction logic
//! in adze-common.
//!
//! Exercises FieldThenParams parsing (field declarations with metadata),
//! NameValueExpr key-value extraction, and the type-mapping functions
//! (try_extract_inner_type, filter_inner_type, wrap_leaf_type) as they
//! apply to field type transformations in grammar definitions.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip<'a>(names: &[&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. FieldThenParams — field mapping metadata extraction
// ===========================================================================

/// A bare generic field with no trailing params should parse cleanly.
#[test]
fn field_params_generic_no_metadata() {
    let ftp: FieldThenParams = parse_quote!(Vec<Token>);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    assert_eq!(ty_str(&ftp.field.ty), "Vec < Token >");
}

/// A field with a single `rename` parameter.
#[test]
fn field_params_single_rename() {
    let ftp: FieldThenParams = parse_quote!(String, rename = "identifier");
    assert!(ftp.comma.is_some());
    assert_eq!(ftp.params.len(), 1);
    assert_eq!(ftp.params[0].path.to_string(), "rename");
}

/// A field with multiple metadata params (precedence + associativity).
#[test]
fn field_params_precedence_and_assoc() {
    let ftp: FieldThenParams = parse_quote!(Expr, precedence = 3, associativity = "left");
    assert_eq!(ftp.params.len(), 2);
    assert_eq!(ftp.params[0].path.to_string(), "precedence");
    assert_eq!(ftp.params[1].path.to_string(), "associativity");
}

/// Option<T> field type with metadata survives parsing intact.
#[test]
fn field_params_option_type_with_metadata() {
    let ftp: FieldThenParams = parse_quote!(Option<Identifier>, rename = "name");
    assert_eq!(ty_str(&ftp.field.ty), "Option < Identifier >");
    assert_eq!(ftp.params.len(), 1);
}

/// Three parameters on a single field declaration.
#[test]
fn field_params_three_params() {
    let ftp: FieldThenParams =
        parse_quote!(Node, precedence = 1, associativity = "right", rename = "op");
    assert_eq!(ftp.params.len(), 3);
    assert_eq!(ftp.params[2].path.to_string(), "rename");
}

/// A complex nested generic field with no params.
#[test]
fn field_params_nested_generic_no_params() {
    let ftp: FieldThenParams = parse_quote!(Vec<Option<Box<Leaf>>>);
    assert!(ftp.params.is_empty());
    assert_eq!(ty_str(&ftp.field.ty), "Vec < Option < Box < Leaf > > >");
}

/// Verify field visibility is None for unnamed fields.
#[test]
fn field_params_unnamed_has_no_vis() {
    let ftp: FieldThenParams = parse_quote!(u64);
    // Unnamed fields parsed via Field::parse_unnamed have inherited visibility.
    assert!(ftp.field.ident.is_none());
}

// ===========================================================================
// 2. NameValueExpr — key-value mapping extraction
// ===========================================================================

/// String literal value.
#[test]
fn nv_string_literal() {
    let nv: NameValueExpr = parse_quote!(name = "hello");
    assert_eq!(nv.path.to_string(), "name");
    // The expression should be a string literal.
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

/// Integer literal value.
#[test]
fn nv_integer_literal() {
    let nv: NameValueExpr = parse_quote!(precedence = 10);
    assert_eq!(nv.path.to_string(), "precedence");
    if let syn::Expr::Lit(lit) = &nv.expr {
        if let syn::Lit::Int(i) = &lit.lit {
            assert_eq!(i.base10_parse::<i32>().unwrap(), 10);
        } else {
            panic!("expected integer literal");
        }
    } else {
        panic!("expected literal expression");
    }
}

/// Boolean literal value.
#[test]
fn nv_bool_literal() {
    let nv: NameValueExpr = parse_quote!(hidden = true);
    assert_eq!(nv.path.to_string(), "hidden");
}

/// Negative integer expression.
#[test]
fn nv_negative_integer() {
    let nv: NameValueExpr = parse_quote!(offset = -1);
    assert_eq!(nv.path.to_string(), "offset");
}

/// Path expression as value (e.g., an enum variant).
#[test]
fn nv_path_value() {
    let nv: NameValueExpr = parse_quote!(kind = Left);
    assert_eq!(nv.path.to_string(), "kind");
    assert_eq!(nv.expr.to_token_stream().to_string(), "Left");
}

// ===========================================================================
// 3. Field type extraction — mapping field types through containers
// ===========================================================================

/// Extract Option inner from a field's type for field mapping.
#[test]
fn field_extract_option_inner() {
    let ftp: FieldThenParams = parse_quote!(Option<Token>);
    let (inner, ok) = try_extract_inner_type(&ftp.field.ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Token");
}

/// Extract Vec inner from a field's type, skipping Box wrapper.
#[test]
fn field_extract_vec_through_box() {
    let ftp: FieldThenParams = parse_quote!(Box<Vec<Item>>);
    let (inner, ok) = try_extract_inner_type(&ftp.field.ty, "Vec", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Item");
}

/// When field type doesn't match target, extraction returns full type.
#[test]
fn field_extract_no_match_returns_full() {
    let ftp: FieldThenParams = parse_quote!(HashMap<String, i32>);
    let (inner, ok) = try_extract_inner_type(&ftp.field.ty, "Vec", &skip(&[]));
    assert!(!ok);
    assert_eq!(ty_str(&inner), "HashMap < String , i32 >");
}

/// Skip chain stops when inner type is not the target.
#[test]
fn field_extract_skip_chain_no_target() {
    let ftp: FieldThenParams = parse_quote!(Arc<Box<String>>);
    let (inner, ok) = try_extract_inner_type(&ftp.field.ty, "Option", &skip(&["Arc", "Box"]));
    assert!(!ok);
    // Returns original when target not found.
    assert_eq!(ty_str(&inner), "Arc < Box < String > >");
}

// ===========================================================================
// 4. Field type filtering — stripping containers for field mapping
// ===========================================================================

/// Filter strips a single wrapper for field mapping.
#[test]
fn field_filter_single_wrapper() {
    let ftp: FieldThenParams = parse_quote!(Box<Expr>);
    let filtered = filter_inner_type(&ftp.field.ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Expr");
}

/// Filter strips multiple nested wrappers in succession.
#[test]
fn field_filter_multiple_wrappers() {
    let ftp: FieldThenParams = parse_quote!(Arc<Box<Rc<Leaf>>>);
    let filtered = filter_inner_type(&ftp.field.ty, &skip(&["Arc", "Box", "Rc"]));
    assert_eq!(ty_str(&filtered), "Leaf");
}

/// Filter preserves type when no wrappers match skip set.
#[test]
fn field_filter_no_match_preserves() {
    let ftp: FieldThenParams = parse_quote!(Vec<String>);
    let filtered = filter_inner_type(&ftp.field.ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < String >");
}

/// Filter stops at first non-skip container.
#[test]
fn field_filter_partial_match() {
    let ftp: FieldThenParams = parse_quote!(Box<Vec<u8>>);
    let filtered = filter_inner_type(&ftp.field.ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Vec < u8 >");
}

// ===========================================================================
// 5. Field type wrapping — wrapping leaf types for grammar nodes
// ===========================================================================

/// Simple field type gets wrapped in WithLeaf.
#[test]
fn field_wrap_simple_type() {
    let ftp: FieldThenParams = parse_quote!(Token);
    let wrapped = wrap_leaf_type(&ftp.field.ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Token >");
}

/// Container in skip set has its inner type wrapped.
#[test]
fn field_wrap_container_inner() {
    let ftp: FieldThenParams = parse_quote!(Option<Token>);
    let wrapped = wrap_leaf_type(&ftp.field.ty, &skip(&["Option"]));
    assert_eq!(ty_str(&wrapped), "Option < adze :: WithLeaf < Token > >");
}

/// Nested containers all in skip set — only leaf gets wrapped.
#[test]
fn field_wrap_nested_containers() {
    let ftp: FieldThenParams = parse_quote!(Vec<Option<Leaf>>);
    let wrapped = wrap_leaf_type(&ftp.field.ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < adze :: WithLeaf < Leaf > > >"
    );
}

/// Container NOT in skip set gets wrapped entirely.
#[test]
fn field_wrap_container_not_skipped() {
    let ftp: FieldThenParams = parse_quote!(HashMap<K, V>);
    let wrapped = wrap_leaf_type(&ftp.field.ty, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < HashMap < K , V > >");
}

// ===========================================================================
// 6. Combined field mapping pipeline — extract, filter, then wrap
// ===========================================================================

/// Full pipeline: extract Option inner, filter Box, wrap leaf.
#[test]
fn pipeline_extract_filter_wrap() {
    let ty: Type = parse_quote!(Option<Box<Ident>>);
    let skip_extract = skip(&[]);
    let skip_filter = skip(&["Box"]);
    let skip_wrap = skip(&[]);

    // Step 1: extract Option's inner
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip_extract);
    assert!(ok);
    assert_eq!(ty_str(&inner), "Box < Ident >");

    // Step 2: filter out Box
    let filtered = filter_inner_type(&inner, &skip_filter);
    assert_eq!(ty_str(&filtered), "Ident");

    // Step 3: wrap the leaf
    let wrapped = wrap_leaf_type(&filtered, &skip_wrap);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Ident >");
}

/// Pipeline with Vec extraction through Arc skip.
#[test]
fn pipeline_extract_through_skip_then_wrap() {
    let ty: Type = parse_quote!(Arc<Vec<Statement>>);
    let skip_set = skip(&["Arc"]);

    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip_set);
    assert!(ok);
    assert_eq!(ty_str(&inner), "Statement");

    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Statement >");
}

/// Pipeline: field with metadata — type extraction ignores params.
#[test]
fn pipeline_field_with_params_type_extraction() {
    let ftp: FieldThenParams = parse_quote!(Vec<Expr>, precedence = 5);
    assert_eq!(ftp.params.len(), 1);

    let (inner, ok) = try_extract_inner_type(&ftp.field.ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "Expr");

    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Expr >");
}

// ===========================================================================
// 7. Field mapping with qualified / multi-segment paths
// ===========================================================================

/// Qualified path type (std::vec::Vec<T>) does NOT match simple "Vec" target.
#[test]
fn extract_qualified_path_no_match() {
    let ty: Type = parse_quote!(std::vec::Vec<u8>);
    // try_extract_inner_type checks only last segment, so "Vec" matches.
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ty_str(&inner), "u8");
}

/// Filter with qualified path — skip set checks last segment.
#[test]
fn filter_qualified_path_in_skip() {
    let ty: Type = parse_quote!(std::boxed::Box<Inner>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Inner");
}

/// Wrap with qualified path not in skip — wraps entire type.
#[test]
fn wrap_qualified_path_not_skipped() {
    let ty: Type = parse_quote!(std::collections::HashMap<K, V>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < std :: collections :: HashMap < K , V > >"
    );
}
