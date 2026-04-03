//! Contract lock test - verifies that public API remains stable.

use adze_common_syntax_core::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use std::collections::HashSet;
use syn::parse_quote;

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify NameValueExpr struct exists with expected fields
    let nve: NameValueExpr = parse_quote!(key = "value");
    assert_eq!(nve.path.to_string(), "key");
    // Verify eq_token and expr fields exist
    let _eq = &nve.eq_token;
    let _expr = &nve.expr;

    // Verify FieldThenParams struct exists with expected fields
    let ftp: FieldThenParams = parse_quote!(Type);
    assert!(ftp.comma.is_none());
    assert!(ftp.params.is_empty());
    // Verify field exists
    let _field = &ftp.field;

    // Verify Debug trait is implemented for NameValueExpr
    let _debug_nve = format!("{nve:?}");

    // Verify Clone trait is implemented for NameValueExpr
    let _cloned_nve = nve.clone();

    // Verify PartialEq trait is implemented for NameValueExpr
    let nve2: NameValueExpr = parse_quote!(key = "value");
    assert_eq!(nve2, nve2.clone());

    // Verify Debug trait is implemented for FieldThenParams
    let _debug_ftp = format!("{ftp:?}");
}

/// Verify all public functions exist with expected signatures.
#[test]
fn test_contract_lock_functions() {
    let skip_over: HashSet<&str> = HashSet::from(["Box", "Arc"]);

    // Verify try_extract_inner_type function exists
    let ty: syn::Type = parse_quote!(Vec<String>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip_over);
    assert!(extracted);
    assert!(!inner.to_token_stream().to_string().is_empty());

    // Verify filter_inner_type function exists
    let ty: syn::Type = parse_quote!(Box<String>);
    let filtered = filter_inner_type(&ty, &skip_over);
    assert!(!filtered.to_token_stream().to_string().is_empty());

    // Verify wrap_leaf_type function exists
    let ty: syn::Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip_over);
    assert!(!wrapped.to_token_stream().to_string().is_empty());
}

use quote::ToTokens;
