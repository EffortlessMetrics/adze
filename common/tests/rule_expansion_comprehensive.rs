#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for rule expansion logic in adze-common.
//!
//! Rule expansion transforms Rust type annotations into grammar rules.
//! These tests exercise how struct fields, enum variants, optional/repeat
//! wrappers, nested types, skip wrappers, leaf types, and precedence
//! annotations map onto the expansion pipeline provided by adze-common.

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

fn ts(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

// ===========================================================================
// 1. Struct with single field → single rule
// ===========================================================================

#[test]
fn single_field_struct_produces_single_rule() {
    // A struct with one field wraps it directly as a leaf.
    let ty: Type = parse_quote!(Identifier);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Identifier >");
}

#[test]
fn single_field_struct_with_box_wrapper_filters_to_inner() {
    let ty: Type = parse_quote!(Box<Expression>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Expression");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Expression >");
}

#[test]
fn single_field_struct_field_then_params_preserves_type() {
    let parsed: FieldThenParams = parse_quote!(Literal);
    assert_eq!(ts(&parsed.field.ty), "Literal");
    assert!(parsed.params.is_empty());
}

// ===========================================================================
// 2. Struct with multiple fields → sequence rule
// ===========================================================================

#[test]
fn multi_field_struct_each_field_wraps_independently() {
    let fields: Vec<Type> = vec![
        parse_quote!(Keyword),
        parse_quote!(Identifier),
        parse_quote!(Block),
    ];
    let wrap_skip = skip(&[]);
    let results: Vec<String> = fields
        .iter()
        .map(|ty| ts(&wrap_leaf_type(ty, &wrap_skip)))
        .collect();
    assert_eq!(results[0], "adze :: WithLeaf < Keyword >");
    assert_eq!(results[1], "adze :: WithLeaf < Identifier >");
    assert_eq!(results[2], "adze :: WithLeaf < Block >");
}

#[test]
fn multi_field_struct_mixed_leaf_and_container() {
    let fields: Vec<Type> = vec![
        parse_quote!(Token),
        parse_quote!(Vec<Statement>),
        parse_quote!(Option<ReturnExpr>),
    ];
    let wrap_skip = skip(&["Vec", "Option"]);
    let results: Vec<String> = fields
        .iter()
        .map(|ty| ts(&wrap_leaf_type(ty, &wrap_skip)))
        .collect();
    assert_eq!(results[0], "adze :: WithLeaf < Token >");
    assert_eq!(results[1], "Vec < adze :: WithLeaf < Statement > >");
    assert_eq!(results[2], "Option < adze :: WithLeaf < ReturnExpr > >");
}

#[test]
fn multi_field_sequence_order_preserved() {
    // Verifying that processing order matches field declaration order.
    let field_names = ["kw", "name", "body", "semi"];
    let field_types: Vec<Type> = vec![
        parse_quote!(Keyword),
        parse_quote!(Ident),
        parse_quote!(Vec<Stmt>),
        parse_quote!(Punct),
    ];
    let wrap_skip = skip(&["Vec"]);
    let mut sequence = Vec::new();
    for i in 0..field_names.len() {
        sequence.push((
            field_names[i],
            ts(&wrap_leaf_type(&field_types[i], &wrap_skip)),
        ));
    }
    assert_eq!(sequence[0].0, "kw");
    assert_eq!(sequence[1].0, "name");
    assert_eq!(sequence[2].0, "body");
    assert!(sequence[2].1.starts_with("Vec <"));
    assert_eq!(sequence[3].0, "semi");
}

// ===========================================================================
// 3. Enum with variants → choice rule
// ===========================================================================

#[test]
fn enum_variants_each_produce_distinct_wrapped_type() {
    // Each enum variant type is independently processed.
    let variant_types: Vec<Type> = vec![
        parse_quote!(IfExpr),
        parse_quote!(WhileExpr),
        parse_quote!(ForExpr),
        parse_quote!(MatchExpr),
    ];
    let wrap_skip = skip(&[]);
    let mut results = Vec::new();
    for ty in &variant_types {
        results.push(ts(&wrap_leaf_type(ty, &wrap_skip)));
    }
    assert_eq!(results.len(), 4);
    for r in &results {
        assert!(r.starts_with("adze :: WithLeaf <"));
    }
    // Each is distinct
    let unique: HashSet<&String> = results.iter().collect();
    assert_eq!(unique.len(), 4);
}

#[test]
fn enum_variant_with_tuple_payload_wraps_whole() {
    let ty: Type = parse_quote!((Operator, Expr));
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < (Operator , Expr) >");
}

#[test]
fn enum_variant_with_box_payload_filters_then_wraps() {
    let ty: Type = parse_quote!(Box<BinaryExpr>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < BinaryExpr >");
}

// ===========================================================================
// 4. Optional field → choice with blank
// ===========================================================================

#[test]
fn optional_field_extracts_inner_type() {
    let ty: Type = parse_quote!(Option<ElseClause>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "ElseClause");
}

#[test]
fn optional_field_through_box_skip_extracts_inner() {
    let ty: Type = parse_quote!(Box<Option<TypeAnnotation>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(ok);
    assert_eq!(ts(&inner), "TypeAnnotation");
}

#[test]
fn optional_field_absent_returns_false() {
    let ty: Type = parse_quote!(Vec<Token>);
    let (_, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!ok);
}

#[test]
fn optional_nested_option_extracts_outermost() {
    let ty: Type = parse_quote!(Option<Option<Modifier>>);
    let (inner, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Option < Modifier >");
}

// ===========================================================================
// 5. Vec field → repeat
// ===========================================================================

#[test]
fn vec_field_extracts_element_type() {
    let ty: Type = parse_quote!(Vec<Parameter>);
    let (inner, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Parameter");
}

#[test]
fn vec_field_wrap_preserves_vec_wraps_element() {
    let ty: Type = parse_quote!(Vec<Argument>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < Argument > >");
}

#[test]
fn vec_field_with_separator_param() {
    let parsed: FieldThenParams = parse_quote!(Vec<Item>, separator = ",");
    let (inner, ok) = try_extract_inner_type(&parsed.field.ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&inner), "Item");
    assert_eq!(parsed.params[0].path.to_string(), "separator");
}

// ===========================================================================
// 6. Nested types → nested rules
// ===========================================================================

#[test]
fn nested_option_vec_extracts_sequentially() {
    let ty: Type = parse_quote!(Option<Vec<Decorator>>);
    let (after_opt, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&after_opt), "Vec < Decorator >");

    let (after_vec, ok) = try_extract_inner_type(&after_opt, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&after_vec), "Decorator");
}

#[test]
fn nested_vec_option_box_full_pipeline() {
    let ty: Type = parse_quote!(Vec<Option<Box<Expr>>>);

    let (after_vec, ok) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&after_vec), "Option < Box < Expr > >");

    let (after_opt, ok) = try_extract_inner_type(&after_vec, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&after_opt), "Box < Expr >");

    let filtered = filter_inner_type(&after_opt, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Expr");
}

#[test]
fn nested_wrap_multi_layer_skip() {
    let ty: Type = parse_quote!(Option<Vec<Arc<Leaf>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec", "Arc"]));
    assert_eq!(
        ts(&wrapped),
        "Option < Vec < Arc < adze :: WithLeaf < Leaf > > > >"
    );
}

// ===========================================================================
// 7. Skip field → excluded from rule
// ===========================================================================

#[test]
fn skip_box_removes_wrapper_preserves_inner() {
    let ty: Type = parse_quote!(Box<Statement>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Statement");
}

#[test]
fn skip_multiple_wrappers_chains() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Payload>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ts(&filtered), "Payload");
}

#[test]
fn skip_stops_at_non_skip_container() {
    let ty: Type = parse_quote!(Box<Vec<Token>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Vec < Token >");
}

#[test]
fn skip_empty_set_preserves_everything() {
    let ty: Type = parse_quote!(Box<Arc<Inner>>);
    let filtered = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ts(&filtered), "Box < Arc < Inner > >");
}

// ===========================================================================
// 8. Leaf field → terminal
// ===========================================================================

#[test]
fn leaf_simple_type_wraps_with_leaf() {
    let ty: Type = parse_quote!(NumberLiteral);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < NumberLiteral >");
}

#[test]
fn leaf_primitive_type_wraps() {
    let ty: Type = parse_quote!(u64);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < u64 >");
}

#[test]
fn leaf_reference_type_wraps_entirely() {
    let ty: Type = parse_quote!(&str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < & str >");
}

// ===========================================================================
// 9. Precedence field → prec wrapper
// ===========================================================================

#[test]
fn precedence_param_parsed_from_field_then_params() {
    let parsed: FieldThenParams = parse_quote!(Expr, precedence = 5);
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "precedence");
    if let syn::Expr::Lit(lit) = &parsed.params[0].expr {
        if let syn::Lit::Int(i) = &lit.lit {
            assert_eq!(i.base10_parse::<i32>().unwrap(), 5);
        } else {
            panic!("Expected integer literal");
        }
    } else {
        panic!("Expected literal expression");
    }
}

#[test]
fn precedence_with_associativity_params() {
    let parsed: FieldThenParams = parse_quote!(BinOp, precedence = 3, associativity = "left");
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "precedence");
    assert_eq!(parsed.params[1].path.to_string(), "associativity");
}

#[test]
fn precedence_field_type_preserved_after_param_parse() {
    let parsed: FieldThenParams = parse_quote!(Box<UnaryExpr>, precedence = 10);
    let filtered = filter_inner_type(&parsed.field.ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "UnaryExpr");
    assert_eq!(parsed.params[0].path.to_string(), "precedence");
}

#[test]
fn precedence_negative_value_parsed() {
    let nv: NameValueExpr = parse_quote!(precedence = -1);
    assert_eq!(nv.path.to_string(), "precedence");
    assert!(matches!(nv.expr, syn::Expr::Unary(_)));
}

// ===========================================================================
// 10. Combined: full struct expansion simulation
// ===========================================================================

#[test]
fn full_struct_expansion_function_decl() {
    // Simulate expanding: struct FnDecl { kw: Keyword, name: Ident,
    //   params: Vec<Param>, ret: Option<RetType>, body: Box<Block> }
    let fields: Vec<(&str, Type)> = vec![
        ("kw", parse_quote!(Keyword)),
        ("name", parse_quote!(Ident)),
        ("params", parse_quote!(Vec<Param>)),
        ("ret", parse_quote!(Option<RetType>)),
        ("body", parse_quote!(Box<Block>)),
    ];

    let extract_skip = skip(&["Box"]);
    let wrap_skip = skip(&["Vec", "Option"]);

    let mut expanded = Vec::new();
    for (name, ty) in &fields {
        let (after_opt, is_optional) = try_extract_inner_type(ty, "Option", &extract_skip);
        let (after_vec, is_repeat) = if is_optional {
            try_extract_inner_type(&after_opt, "Vec", &extract_skip)
        } else {
            try_extract_inner_type(ty, "Vec", &extract_skip)
        };

        let base = if is_repeat {
            after_vec
        } else if is_optional {
            after_opt
        } else {
            ty.clone()
        };
        let filtered = filter_inner_type(&base, &skip(&["Box"]));
        expanded.push((*name, is_optional, is_repeat, ts(&filtered)));
    }

    assert_eq!(expanded[0], ("kw", false, false, "Keyword".to_string()));
    assert_eq!(expanded[1], ("name", false, false, "Ident".to_string()));
    assert_eq!(expanded[2], ("params", false, true, "Param".to_string()));
    assert_eq!(expanded[3], ("ret", true, false, "RetType".to_string()));
    assert_eq!(expanded[4], ("body", false, false, "Block".to_string()));
}

#[test]
fn full_enum_expansion_expression_variants() {
    // Simulate expanding: enum Expr { Lit(Literal), Bin(BinExpr), Un(UnExpr), Call(CallExpr) }
    let variant_types: Vec<Type> = vec![
        parse_quote!(Literal),
        parse_quote!(BinExpr),
        parse_quote!(UnExpr),
        parse_quote!(CallExpr),
    ];
    let wrap_skip = skip(&[]);
    let wrapped: Vec<String> = variant_types
        .iter()
        .map(|ty| ts(&wrap_leaf_type(ty, &wrap_skip)))
        .collect();

    // Each variant produces its own rule, all distinct
    assert_eq!(wrapped.len(), 4);
    assert!(wrapped[0].contains("Literal"));
    assert!(wrapped[1].contains("BinExpr"));
    assert!(wrapped[2].contains("UnExpr"));
    assert!(wrapped[3].contains("CallExpr"));
}
