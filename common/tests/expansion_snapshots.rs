//! Snapshot tests for the common crate's grammar expansion output.
//!
//! Each test simulates the grammar expansion pipeline for a particular
//! type pattern and snapshots the resulting type transformations as JSON.

use std::collections::HashSet;

use adze_common::{FieldThenParams, filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Expand a list of named fields into a JSON object representing a struct rule.
fn expand_struct_rule(
    name: &str,
    fields: &[(&str, Type)],
    filter_skip: &HashSet<&str>,
    wrap_skip: &HashSet<&str>,
) -> serde_json::Value {
    let field_expansions: Vec<serde_json::Value> = fields
        .iter()
        .map(|(field_name, ty)| {
            let (opt_inner, is_optional) = try_extract_inner_type(ty, "Option", filter_skip);
            let (vec_inner, is_repeat) = try_extract_inner_type(ty, "Vec", filter_skip);
            let filtered = filter_inner_type(ty, filter_skip);
            let wrapped = wrap_leaf_type(ty, wrap_skip);

            serde_json::json!({
                "name": field_name,
                "input_type": ty.to_token_stream().to_string(),
                "is_optional": is_optional,
                "optional_inner": if is_optional {
                    opt_inner.to_token_stream().to_string()
                } else {
                    "N/A".to_string()
                },
                "is_repeat": is_repeat,
                "repeat_inner": if is_repeat {
                    vec_inner.to_token_stream().to_string()
                } else {
                    "N/A".to_string()
                },
                "filtered": filtered.to_token_stream().to_string(),
                "wrapped": wrapped.to_token_stream().to_string(),
            })
        })
        .collect();

    serde_json::json!({
        "rule_name": name,
        "kind": "struct",
        "fields": field_expansions,
    })
}

/// Expand enum variants into a JSON object representing a choice rule.
fn expand_enum_rule(
    name: &str,
    variants: &[(&str, Type)],
    filter_skip: &HashSet<&str>,
    wrap_skip: &HashSet<&str>,
) -> serde_json::Value {
    let variant_expansions: Vec<serde_json::Value> = variants
        .iter()
        .map(|(variant_name, ty)| {
            let filtered = filter_inner_type(ty, filter_skip);
            let wrapped = wrap_leaf_type(ty, wrap_skip);

            serde_json::json!({
                "variant": variant_name,
                "input_type": ty.to_token_stream().to_string(),
                "filtered": filtered.to_token_stream().to_string(),
                "wrapped": wrapped.to_token_stream().to_string(),
            })
        })
        .collect();

    serde_json::json!({
        "rule_name": name,
        "kind": "enum",
        "variants": variant_expansions,
    })
}

/// Expand a field with annotation params into JSON.
fn expand_annotated_field(
    parsed: &FieldThenParams,
    wrap_skip: &HashSet<&str>,
) -> serde_json::Value {
    let ty = &parsed.field.ty;
    let wrapped = wrap_leaf_type(ty, wrap_skip);

    let params: Vec<serde_json::Value> = parsed
        .params
        .iter()
        .map(|p| {
            serde_json::json!({
                "name": p.path.to_string(),
                "value": p.expr.to_token_stream().to_string(),
            })
        })
        .collect();

    serde_json::json!({
        "input_type": ty.to_token_stream().to_string(),
        "wrapped": wrapped.to_token_stream().to_string(),
        "params": params,
    })
}

// ===========================================================================
// 1. Simple struct
// ===========================================================================

#[test]
fn snapshot_simple_struct() {
    let filter_skip: HashSet<&str> = HashSet::new();
    let wrap_skip: HashSet<&str> = HashSet::new();

    let fields: Vec<(&str, Type)> = vec![
        ("name", parse_quote!(String)),
        ("value", parse_quote!(i32)),
        ("flag", parse_quote!(bool)),
    ];

    let result = expand_struct_rule("SimpleStruct", &fields, &filter_skip, &wrap_skip);
    insta::assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}

// ===========================================================================
// 2. Struct with Optional fields
// ===========================================================================

#[test]
fn snapshot_struct_with_optional_fields() {
    let filter_skip: HashSet<&str> = HashSet::new();
    let wrap_skip: HashSet<&str> = ["Option"].into_iter().collect();

    let fields: Vec<(&str, Type)> = vec![
        ("required_name", parse_quote!(String)),
        ("optional_value", parse_quote!(Option<i32>)),
        ("optional_label", parse_quote!(Option<String>)),
    ];

    let result = expand_struct_rule("OptionalStruct", &fields, &filter_skip, &wrap_skip);
    insta::assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}

// ===========================================================================
// 3. Struct with Vec fields
// ===========================================================================

#[test]
fn snapshot_struct_with_vec_fields() {
    let filter_skip: HashSet<&str> = HashSet::new();
    let wrap_skip: HashSet<&str> = ["Vec"].into_iter().collect();

    let fields: Vec<(&str, Type)> = vec![
        ("items", parse_quote!(Vec<Expr>)),
        ("tags", parse_quote!(Vec<String>)),
        ("single", parse_quote!(Identifier)),
    ];

    let result = expand_struct_rule("VecStruct", &fields, &filter_skip, &wrap_skip);
    insta::assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}

// ===========================================================================
// 4. Enum with multiple variants
// ===========================================================================

#[test]
fn snapshot_enum_multiple_variants() {
    let filter_skip: HashSet<&str> = ["Box"].into_iter().collect();
    let wrap_skip: HashSet<&str> = HashSet::new();

    let variants: Vec<(&str, Type)> = vec![
        ("Literal", parse_quote!(LiteralExpr)),
        ("Binary", parse_quote!(Box<BinaryExpr>)),
        ("Unary", parse_quote!(Box<UnaryExpr>)),
        ("Group", parse_quote!(Box<GroupExpr>)),
    ];

    let result = expand_enum_rule("Expression", &variants, &filter_skip, &wrap_skip);
    insta::assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}

// ===========================================================================
// 5. Nested types
// ===========================================================================

#[test]
fn snapshot_nested_types() {
    let filter_skip: HashSet<&str> = ["Box", "Spanned"].into_iter().collect();
    let wrap_skip: HashSet<&str> = ["Option", "Vec"].into_iter().collect();

    let fields: Vec<(&str, Type)> = vec![
        ("items", parse_quote!(Option<Vec<Item>>)),
        ("boxed_child", parse_quote!(Box<Spanned<Node>>)),
        ("deep", parse_quote!(Box<Option<Vec<Token>>>)),
    ];

    let result = expand_struct_rule("NestedStruct", &fields, &filter_skip, &wrap_skip);
    insta::assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}

// ===========================================================================
// 6. Types with precedence annotations
// ===========================================================================

#[test]
fn snapshot_precedence_annotations() {
    let wrap_skip: HashSet<&str> = HashSet::new();

    let add: FieldThenParams = parse_quote!(BinaryExpr, precedence = 1, assoc = "left");
    let mul: FieldThenParams = parse_quote!(BinaryExpr, precedence = 2, assoc = "left");
    let unary: FieldThenParams = parse_quote!(UnaryExpr, precedence = 3);

    let result = serde_json::json!({
        "rule_name": "PrecedenceExpr",
        "kind": "enum_with_precedence",
        "variants": [
            expand_annotated_field(&add, &wrap_skip),
            expand_annotated_field(&mul, &wrap_skip),
            expand_annotated_field(&unary, &wrap_skip),
        ],
    });

    insta::assert_snapshot!(serde_json::to_string_pretty(&result).unwrap());
}
