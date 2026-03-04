//! Comprehensive v3 tests for adze-common grammar expansion logic.
//!
//! Tests struct/enum expansion patterns, field handling, error cases,
//! NameValueExpr–grammar interactions, and attribute extraction patterns
//! used by both the macro and tool crates.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Expr, Field, Type, parse_quote};

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
// 1. Struct expansion patterns – simulating field-by-field grammar processing
// ===========================================================================

/// Helper: simulate the expansion pipeline applied to a single grammar field.
/// Returns (is_optional, is_repeated, leaf_type_str, wrapped_str).
fn expand_grammar_field(ty: &Type) -> (bool, bool, String, String) {
    let wrapper_skip = skip(&["Box", "Arc", "Rc"]);
    let container_skip = skip(&["Vec", "Option"]);

    // Step 1: check optionality
    let (after_opt, is_optional) = try_extract_inner_type(ty, "Option", &wrapper_skip);
    let source = if is_optional { &after_opt } else { ty };

    // Step 2: check repetition
    let (after_vec, is_repeated) = try_extract_inner_type(source, "Vec", &wrapper_skip);
    let source2 = if is_repeated { &after_vec } else { source };

    // Step 3: strip wrappers
    let leaf = filter_inner_type(source2, &wrapper_skip);
    let leaf_str = ty_str(&leaf);

    // Step 4: wrap
    let wrapped = wrap_leaf_type(source2, &container_skip);
    let wrapped_str = ty_str(&wrapped);

    (is_optional, is_repeated, leaf_str, wrapped_str)
}

#[test]
fn struct_field_plain_type() {
    let ty: Type = parse_quote!(Identifier);
    let (opt, rep, leaf, _wrapped) = expand_grammar_field(&ty);
    assert!(!opt);
    assert!(!rep);
    assert_eq!(leaf, "Identifier");
}

#[test]
fn struct_field_optional() {
    let ty: Type = parse_quote!(Option<ReturnType>);
    let (opt, rep, leaf, _) = expand_grammar_field(&ty);
    assert!(opt);
    assert!(!rep);
    assert_eq!(leaf, "ReturnType");
}

#[test]
fn struct_field_repeated() {
    let ty: Type = parse_quote!(Vec<Statement>);
    let (opt, rep, leaf, _) = expand_grammar_field(&ty);
    assert!(!opt);
    assert!(rep);
    assert_eq!(leaf, "Statement");
}

#[test]
fn struct_field_optional_repeated() {
    let ty: Type = parse_quote!(Option<Vec<Decorator>>);
    let (opt, rep, leaf, _) = expand_grammar_field(&ty);
    assert!(opt);
    assert!(rep);
    assert_eq!(leaf, "Decorator");
}

#[test]
fn struct_field_boxed_leaf() {
    let ty: Type = parse_quote!(Box<Expression>);
    let (opt, rep, leaf, _) = expand_grammar_field(&ty);
    assert!(!opt);
    assert!(!rep);
    assert_eq!(leaf, "Expression");
}

#[test]
fn struct_field_optional_boxed() {
    let ty: Type = parse_quote!(Option<Box<Expression>>);
    let (opt, rep, leaf, _) = expand_grammar_field(&ty);
    assert!(opt);
    assert!(!rep);
    assert_eq!(leaf, "Expression");
}

#[test]
fn struct_field_vec_of_boxed() {
    let ty: Type = parse_quote!(Vec<Box<Node>>);
    let (opt, rep, leaf, _) = expand_grammar_field(&ty);
    assert!(!opt);
    assert!(rep);
    assert_eq!(leaf, "Node");
}

#[test]
fn struct_field_arc_wrapped_optional() {
    let ty: Type = parse_quote!(Arc<Option<Value>>);
    let (opt, rep, leaf, _) = expand_grammar_field(&ty);
    assert!(opt);
    assert!(!rep);
    assert_eq!(leaf, "Value");
}

// ===========================================================================
// 2. Enum expansion patterns – variant payloads processed identically
// ===========================================================================

#[test]
fn enum_variant_unit_like_type() {
    // A unit-like enum variant has no inner type — just a plain type name.
    let ty: Type = parse_quote!(Keyword);
    let leaf = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&leaf), "Keyword");
}

#[test]
fn enum_variant_with_box_payload() {
    let ty: Type = parse_quote!(Box<BinaryExpr>);
    let leaf = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&leaf), "BinaryExpr");
}

#[test]
fn enum_variant_with_option_payload() {
    let ty: Type = parse_quote!(Option<LiteralValue>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);
    assert_eq!(ty_str(&inner), "LiteralValue");
}

#[test]
fn enum_multiple_variants_expansion() {
    // Simulate processing all variants of an enum
    let variants: Vec<Type> = vec![
        parse_quote!(Box<IfExpr>),
        parse_quote!(Box<WhileExpr>),
        parse_quote!(LiteralValue),
        parse_quote!(Vec<Statement>),
    ];
    let wrapper_skip = skip(&["Box"]);

    let leaves: Vec<String> = variants
        .iter()
        .map(|ty| ty_str(&filter_inner_type(ty, &wrapper_skip)))
        .collect();

    assert_eq!(
        leaves,
        vec!["IfExpr", "WhileExpr", "LiteralValue", "Vec < Statement >"]
    );
}

// ===========================================================================
// 3. Field handling – FieldThenParams with grammar-relevant parameters
// ===========================================================================

#[test]
fn field_with_precedence_param() {
    let parsed: FieldThenParams = parse_quote!(Expression, precedence = 5);
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "precedence");
    if let Expr::Lit(lit) = &parsed.params[0].expr {
        if let syn::Lit::Int(i) = &lit.lit {
            assert_eq!(i.base10_parse::<i32>().unwrap(), 5);
        } else {
            panic!("expected int literal");
        }
    } else {
        panic!("expected literal expression");
    }
}

#[test]
fn field_with_associativity_param() {
    let parsed: FieldThenParams = parse_quote!(BinOp, associativity = "left");
    assert_eq!(parsed.params[0].path.to_string(), "associativity");
}

#[test]
fn field_with_rename_param() {
    let parsed: FieldThenParams = parse_quote!(Token, rename = "identifier");
    assert_eq!(parsed.params[0].path.to_string(), "rename");
}

#[test]
fn field_type_preserved_through_params() {
    let parsed: FieldThenParams = parse_quote!(Vec<Option<Box<Expr>>>, min = 1, max = 10);
    assert_eq!(ty_str(&parsed.field.ty), "Vec < Option < Box < Expr > > >");
    assert_eq!(parsed.params.len(), 2);
}

#[test]
fn field_param_iteration_order() {
    let parsed: FieldThenParams = parse_quote!(i32, alpha = 1, beta = 2, gamma = 3, delta = 4);
    let names: Vec<String> = parsed.params.iter().map(|p| p.path.to_string()).collect();
    assert_eq!(names, vec!["alpha", "beta", "gamma", "delta"]);
}

#[test]
fn field_no_comma_means_no_params() {
    let parsed: FieldThenParams = parse_quote!(ComplexType<A, B>);
    assert!(parsed.comma.is_none());
    assert!(parsed.params.is_empty());
}

// ===========================================================================
// 4. Error cases – panics on invalid type structures
// ===========================================================================

#[test]
#[should_panic(expected = "Expected angle bracketed path")]
fn extract_panics_on_skip_type_without_angle_brackets() {
    // Construct a type path segment with no angle brackets but matching skip name.
    // "Box" is in skip set but has no generic args.
    let ty: Type = parse_quote!(Box);
    let _ = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
}

#[test]
#[should_panic(expected = "Expected angle bracketed path")]
fn extract_panics_on_target_without_angle_brackets() {
    let ty: Type = parse_quote!(Option);
    let _ = try_extract_inner_type(&ty, "Option", &skip(&[]));
}

#[test]
#[should_panic(expected = "Expected angle bracketed path")]
fn filter_panics_on_skip_type_without_angle_brackets() {
    let ty: Type = parse_quote!(Box);
    let _ = filter_inner_type(&ty, &skip(&["Box"]));
}

#[test]
#[should_panic(expected = "Expected angle bracketed path")]
fn wrap_panics_on_skip_type_without_angle_brackets() {
    let ty: Type = parse_quote!(Vec);
    let _ = wrap_leaf_type(&ty, &skip(&["Vec"]));
}

// ===========================================================================
// 5. NameValueExpr and grammar building interactions
// ===========================================================================

#[test]
fn name_value_expr_numeric_precedence_extraction() {
    let nv: NameValueExpr = parse_quote!(precedence = 10);
    assert_eq!(nv.path.to_string(), "precedence");
    if let Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(ref i),
        ..
    }) = nv.expr
    {
        assert_eq!(i.base10_parse::<u32>().unwrap(), 10);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn name_value_expr_string_pattern_extraction() {
    let nv: NameValueExpr = parse_quote!(pattern = r"[a-zA-Z_]\w*");
    assert_eq!(nv.path.to_string(), "pattern");
    if let Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(ref s),
        ..
    }) = nv.expr
    {
        assert_eq!(s.value(), r"[a-zA-Z_]\w*");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn name_value_expr_bool_inline_flag() {
    let nv: NameValueExpr = parse_quote!(inline = false);
    assert_eq!(nv.path.to_string(), "inline");
    if let Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Bool(ref b),
        ..
    }) = nv.expr
    {
        assert!(!b.value());
    } else {
        panic!("Expected bool literal");
    }
}

#[test]
fn name_value_expr_tuple_value() {
    let nv: NameValueExpr = parse_quote!(range = (1, 100));
    assert_eq!(nv.path.to_string(), "range");
    assert!(matches!(nv.expr, Expr::Tuple(_)));
}

#[test]
fn name_value_expr_binary_expression() {
    let nv: NameValueExpr = parse_quote!(limit = 2 + 3);
    assert_eq!(nv.path.to_string(), "limit");
    assert!(matches!(nv.expr, Expr::Binary(_)));
}

#[test]
fn name_value_expr_if_expression() {
    let nv: NameValueExpr = parse_quote!(val = if true { 1 } else { 2 });
    assert_eq!(nv.path.to_string(), "val");
    assert!(matches!(nv.expr, Expr::If(_)));
}

#[test]
fn name_value_multiple_in_punctuated() {
    // Simulate multiple params parsed together (as in FieldThenParams)
    let parsed: FieldThenParams = parse_quote!(
        Token,
        pattern = r"\d+",
        precedence = 5,
        associativity = "left",
        inline = true
    );
    assert_eq!(parsed.params.len(), 4);

    let lookup: std::collections::HashMap<String, &NameValueExpr> = parsed
        .params
        .iter()
        .map(|p| (p.path.to_string(), p))
        .collect();

    assert!(lookup.contains_key("pattern"));
    assert!(lookup.contains_key("precedence"));
    assert!(lookup.contains_key("associativity"));
    assert!(lookup.contains_key("inline"));
}

// ===========================================================================
// 6. Attribute extraction patterns – how types are analyzed during expansion
// ===========================================================================

#[test]
fn extract_option_then_vec_two_stage() {
    let ty: Type = parse_quote!(Option<Vec<Item>>);
    let empty = skip(&[]);

    let (after_opt, found_opt) = try_extract_inner_type(&ty, "Option", &empty);
    assert!(found_opt);
    assert_eq!(ty_str(&after_opt), "Vec < Item >");

    let (after_vec, found_vec) = try_extract_inner_type(&after_opt, "Vec", &empty);
    assert!(found_vec);
    assert_eq!(ty_str(&after_vec), "Item");
}

#[test]
fn extract_vec_then_option_order_matters() {
    let ty: Type = parse_quote!(Vec<Option<Item>>);
    let empty = skip(&[]);

    // Extracting Vec first succeeds
    let (after_vec, found_vec) = try_extract_inner_type(&ty, "Vec", &empty);
    assert!(found_vec);
    assert_eq!(ty_str(&after_vec), "Option < Item >");

    // Then extracting Option from the result
    let (after_opt, found_opt) = try_extract_inner_type(&after_vec, "Option", &empty);
    assert!(found_opt);
    assert_eq!(ty_str(&after_opt), "Item");
}

#[test]
fn extract_option_fails_on_vec_outermost() {
    let ty: Type = parse_quote!(Vec<Option<Item>>);
    let empty = skip(&[]);

    // Option is not outermost
    let (result, found) = try_extract_inner_type(&ty, "Option", &empty);
    assert!(!found);
    assert_eq!(ty_str(&result), "Vec < Option < Item > >");
}

#[test]
fn extract_with_skip_finds_target_inside_wrapper() {
    // Box<Vec<String>> — skip Box, find Vec
    let ty: Type = parse_quote!(Box<Vec<String>>);
    let (inner, found) = try_extract_inner_type(&ty, "Vec", &skip(&["Box"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "String");
}

#[test]
fn extract_skip_chain_three_deep() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<u8>>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Box", "Arc", "Rc"]));
    assert!(found);
    assert_eq!(ty_str(&inner), "u8");
}

// ===========================================================================
// 7. wrap_leaf_type – deeper patterns
// ===========================================================================

#[test]
fn wrap_nested_three_skip_levels() {
    let ty: Type = parse_quote!(Option<Vec<Box<Leaf>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option", "Vec", "Box"]));
    assert_eq!(
        ty_str(&wrapped),
        "Option < Vec < Box < adze :: WithLeaf < Leaf > > > >"
    );
}

#[test]
fn wrap_with_result_both_args_wrapped() {
    let ty: Type = parse_quote!(Result<OkType, ErrType>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ty_str(&wrapped),
        "Result < adze :: WithLeaf < OkType > , adze :: WithLeaf < ErrType > >"
    );
}

#[test]
fn wrap_hashmap_not_in_skip_wraps_entire() {
    let ty: Type = parse_quote!(HashMap<String, Vec<u8>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < HashMap < String , Vec < u8 > > >"
    );
}

#[test]
fn wrap_preserves_lifetime_in_reference() {
    let ty: Type = parse_quote!(&'a str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < & 'a str >");
}

#[test]
fn wrap_array_type_wrapped_as_leaf() {
    let ty: Type = parse_quote!([u8; 16]);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < [u8 ; 16] >");
}

// ===========================================================================
// 8. filter_inner_type – deeper patterns
// ===========================================================================

#[test]
fn filter_three_deep_all_in_skip() {
    let ty: Type = parse_quote!(Box<Arc<Rc<Leaf>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ty_str(&filtered), "Leaf");
}

#[test]
fn filter_stops_at_first_non_skip() {
    let ty: Type = parse_quote!(Box<Vec<Arc<Leaf>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc"]));
    // Vec is not in skip set, so it stops there
    assert_eq!(ty_str(&filtered), "Vec < Arc < Leaf > >");
}

#[test]
fn filter_qualified_path_deep() {
    let ty: Type = parse_quote!(std::boxed::Box<std::sync::Arc<std::rc::Rc<Inner>>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box", "Arc", "Rc"]));
    assert_eq!(ty_str(&filtered), "Inner");
}

#[test]
fn filter_returns_clone_when_no_match() {
    let ty: Type = parse_quote!(MyCustomType<A>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "MyCustomType < A >");
}

// ===========================================================================
// 9. Compositional properties
// ===========================================================================

#[test]
fn filter_then_filter_is_same_as_single_filter() {
    let ty: Type = parse_quote!(Box<Arc<Inner>>);
    let s = skip(&["Box", "Arc"]);
    let once = filter_inner_type(&ty, &s);
    let twice = filter_inner_type(&once, &s);
    assert_eq!(ty_str(&once), ty_str(&twice));
    assert_eq!(ty_str(&once), "Inner");
}

#[test]
fn wrap_unwrapped_leaf_roundtrip_shape() {
    // filter strips, wrap re-adds adze::WithLeaf at leaf
    let ty: Type = parse_quote!(Box<Token>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Token");

    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Token >");
}

#[test]
fn extract_and_wrap_preserves_vec_container() {
    let ty: Type = parse_quote!(Option<Vec<Node>>);
    let (after_opt, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(found);

    let wrapped = wrap_leaf_type(&after_opt, &skip(&["Vec"]));
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < Node > >");
}

// ===========================================================================
// 10. Full pipeline simulation – multi-field struct
// ===========================================================================

#[test]
fn full_struct_expansion_simulation() {
    // Simulates expanding a grammar struct:
    //   struct FunctionDef {
    //       name: Identifier,
    //       params: Vec<Parameter>,
    //       body: Box<Block>,
    //       return_type: Option<TypeExpr>,
    //       decorators: Option<Vec<Decorator>>,
    //   }
    let fields: Vec<(&str, Type)> = vec![
        ("name", parse_quote!(Identifier)),
        ("params", parse_quote!(Vec<Parameter>)),
        ("body", parse_quote!(Box<Block>)),
        ("return_type", parse_quote!(Option<TypeExpr>)),
        ("decorators", parse_quote!(Option<Vec<Decorator>>)),
    ];

    let results: Vec<_> = fields
        .iter()
        .map(|(name, ty)| (*name, expand_grammar_field(ty)))
        .collect();

    // name: plain, not optional, not repeated
    assert_eq!(results[0].0, "name");
    assert!(!results[0].1.0 && !results[0].1.1);
    assert_eq!(results[0].1.2, "Identifier");

    // params: repeated
    assert_eq!(results[1].0, "params");
    assert!(!results[1].1.0 && results[1].1.1);
    assert_eq!(results[1].1.2, "Parameter");

    // body: Box stripped
    assert_eq!(results[2].0, "body");
    assert!(!results[2].1.0 && !results[2].1.1);
    assert_eq!(results[2].1.2, "Block");

    // return_type: optional
    assert_eq!(results[3].0, "return_type");
    assert!(results[3].1.0 && !results[3].1.1);
    assert_eq!(results[3].1.2, "TypeExpr");

    // decorators: optional + repeated
    assert_eq!(results[4].0, "decorators");
    assert!(results[4].1.0 && results[4].1.1);
    assert_eq!(results[4].1.2, "Decorator");
}

// ===========================================================================
// 11. FieldThenParams – field visibility and attributes
// ===========================================================================

#[test]
fn field_then_params_field_is_unnamed() {
    let parsed: FieldThenParams = parse_quote!(String);
    // Field::parse_unnamed produces a field with no ident
    assert!(parsed.field.ident.is_none());
}

#[test]
fn field_then_params_field_visibility_default() {
    let parsed: FieldThenParams = parse_quote!(u64, limit = 100);
    // Unnamed fields parsed via Field::parse_unnamed have inherited visibility
    assert!(matches!(parsed.field.vis, syn::Visibility::Inherited));
}

#[test]
fn field_then_params_equality_with_same_input() {
    let a: FieldThenParams = parse_quote!(String, key = "val");
    let b: FieldThenParams = parse_quote!(String, key = "val");
    assert_eq!(a, b);
}

#[test]
fn field_then_params_inequality_different_params() {
    let a: FieldThenParams = parse_quote!(String, key = "val1");
    let b: FieldThenParams = parse_quote!(String, key = "val2");
    // Different expression values should not be equal
    assert_ne!(a, b);
}

// ===========================================================================
// 12. NameValueExpr – trait implementations
// ===========================================================================

#[test]
fn name_value_expr_eq_reflexive() {
    let nv: NameValueExpr = parse_quote!(key = 42);
    assert_eq!(nv, nv.clone());
}

#[test]
fn name_value_expr_ne_different_key() {
    let a: NameValueExpr = parse_quote!(alpha = 1);
    let b: NameValueExpr = parse_quote!(beta = 1);
    assert_ne!(a, b);
}

#[test]
fn name_value_expr_ne_different_value() {
    let a: NameValueExpr = parse_quote!(key = 1);
    let b: NameValueExpr = parse_quote!(key = 2);
    assert_ne!(a, b);
}

#[test]
fn name_value_expr_debug_contains_fields() {
    let nv: NameValueExpr = parse_quote!(precedence = 5);
    let dbg = format!("{nv:?}");
    assert!(dbg.contains("path"));
    assert!(dbg.contains("eq_token"));
    assert!(dbg.contains("expr"));
}

// ===========================================================================
// 13. Non-path types – comprehensive coverage
// ===========================================================================

#[test]
fn extract_from_reference_type() {
    let ty: Type = parse_quote!(&'static str);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "& 'static str");
}

#[test]
fn extract_from_raw_pointer() {
    let ty: Type = parse_quote!(*const u8);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!found);
    assert_eq!(ty_str(&result), "* const u8");
}

#[test]
fn filter_tuple_type_passthrough() {
    let ty: Type = parse_quote!((A, B, C));
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "(A , B , C)");
}

#[test]
fn wrap_fn_pointer() {
    let ty: Type = parse_quote!(fn(i32, i32) -> bool);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert!(ty_str(&wrapped).starts_with("adze :: WithLeaf"));
}

#[test]
fn wrap_never_type() {
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < () >");
}

// ===========================================================================
// 14. Edge cases in skip set construction
// ===========================================================================

#[test]
fn empty_skip_set_extracts_only_direct_match() {
    let ty: Type = parse_quote!(Box<Option<i32>>);
    let (result, found) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    // Box is not in skip, and outer type is Box not Option
    assert!(!found);
    assert_eq!(ty_str(&result), "Box < Option < i32 > >");
}

#[test]
fn skip_set_with_target_name_still_extracts() {
    // Even if "Option" is in the skip set, it should still match as target
    let ty: Type = parse_quote!(Option<u32>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &skip(&["Option"]));
    // Target check happens before skip check
    assert!(found);
    assert_eq!(ty_str(&inner), "u32");
}

#[test]
fn large_skip_set_performance() {
    let many_skips = skip(&[
        "Box", "Arc", "Rc", "Cell", "RefCell", "Mutex", "RwLock", "Pin",
    ]);
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<Inner>>>>);
    let (inner, found) = try_extract_inner_type(&ty, "Option", &many_skips);
    assert!(found);
    assert_eq!(ty_str(&inner), "Inner");
}

// ===========================================================================
// 15. Realistic grammar expansion end-to-end scenarios
// ===========================================================================

#[test]
fn arithmetic_grammar_field_expansion() {
    // Simulates fields from an arithmetic expression grammar
    let fields: Vec<Type> = vec![
        parse_quote!(Box<Expr>),         // left operand
        parse_quote!(Operator),          // operator token
        parse_quote!(Box<Expr>),         // right operand
        parse_quote!(Option<Box<Expr>>), // optional else branch
        parse_quote!(Vec<Box<Expr>>),    // argument list
    ];

    let wrapper_skip = skip(&["Box"]);
    let wrap_skip = skip(&["Vec", "Option"]);

    let leaves: Vec<String> = fields
        .iter()
        .map(|ty| {
            let filtered = filter_inner_type(ty, &wrapper_skip);
            ty_str(&wrap_leaf_type(&filtered, &wrap_skip))
        })
        .collect();

    assert_eq!(leaves[0], "adze :: WithLeaf < Expr >");
    assert_eq!(leaves[1], "adze :: WithLeaf < Operator >");
    assert_eq!(leaves[2], "adze :: WithLeaf < Expr >");
    // Option<Box<Expr>> -> filter Box from inside stops at Option (not in filter skip)
    // so filter returns Option<Box<Expr>> as-is since Option not in wrapper_skip
    assert_eq!(leaves[3], "Option < adze :: WithLeaf < Box < Expr > > >");
    // Vec<Box<Expr>> -> filter Box stops at Vec, returns Vec<Box<Expr>>
    assert_eq!(leaves[4], "Vec < adze :: WithLeaf < Box < Expr > > >");
}

#[test]
fn python_like_grammar_field_expansion() {
    // Simulates fields from a Python-like grammar struct
    let ty_stmts: Type = parse_quote!(Vec<Statement>);
    let ty_decorators: Type = parse_quote!(Option<Vec<Decorator>>);
    let ty_bases: Type = parse_quote!(Vec<Box<Expression>>);
    let ty_name: Type = parse_quote!(Identifier);

    let (_, stmts_rep, stmts_leaf, _) = expand_grammar_field(&ty_stmts);
    assert!(stmts_rep);
    assert_eq!(stmts_leaf, "Statement");

    let (dec_opt, dec_rep, dec_leaf, _) = expand_grammar_field(&ty_decorators);
    assert!(dec_opt && dec_rep);
    assert_eq!(dec_leaf, "Decorator");

    let (_, bases_rep, bases_leaf, _) = expand_grammar_field(&ty_bases);
    assert!(bases_rep);
    assert_eq!(bases_leaf, "Expression");

    let (name_opt, name_rep, name_leaf, _) = expand_grammar_field(&ty_name);
    assert!(!name_opt && !name_rep);
    assert_eq!(name_leaf, "Identifier");
}

#[test]
fn field_then_params_simulates_leaf_attribute() {
    // Simulates: #[adze::leaf(pattern = r"\d+", transform = |s: String| s.parse::<u64>().unwrap())]
    let parsed: FieldThenParams = parse_quote!(
        u64,
        pattern = r"\d+",
        transform = |s: String| s.parse::<u64>().unwrap()
    );

    assert_eq!(parsed.params.len(), 2);

    // Extract pattern value
    let pattern_nv = &parsed.params[0];
    assert_eq!(pattern_nv.path.to_string(), "pattern");
    if let Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(ref s),
        ..
    }) = pattern_nv.expr
    {
        assert_eq!(s.value(), r"\d+");
    } else {
        panic!("Expected string literal for pattern");
    }

    // Transform is a closure
    let transform_nv = &parsed.params[1];
    assert_eq!(transform_nv.path.to_string(), "transform");
    assert!(matches!(transform_nv.expr, Expr::Closure(_)));
}

#[test]
fn wrap_leaf_type_vec_option_interleaved() {
    // Vec<Option<Vec<Item>>> with all three in skip
    let ty: Type = parse_quote!(Vec<Option<Vec<Item>>>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec", "Option"]));
    assert_eq!(
        ty_str(&wrapped),
        "Vec < Option < Vec < adze :: WithLeaf < Item > > > >"
    );
}
