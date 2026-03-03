//! Comprehensive macro expansion pipeline tests for adze-common.
//!
//! Simulates the complete macro expansion pipeline: from annotated Rust types
//! (structs, enums, generics, lifetimes, where-clauses) through extraction,
//! filtering, and wrapping — exercising only the public API surface.

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

fn ts(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

/// Simulate the pipeline for a single struct field:
/// parse → extract Option/Vec → filter containers → wrap leaf.
fn pipeline_field(
    ty: &Type,
    filter_skip: &HashSet<&str>,
    wrap_skip: &HashSet<&str>,
) -> (bool, bool, String, String, String) {
    let (_, is_opt) = try_extract_inner_type(ty, "Option", filter_skip);
    let (_, is_rep) = try_extract_inner_type(ty, "Vec", filter_skip);
    let filtered = filter_inner_type(ty, filter_skip);
    let wrapped = wrap_leaf_type(ty, wrap_skip);
    (is_opt, is_rep, ts(&filtered), ts(&wrapped), ts(ty))
}

// ===========================================================================
// 1. Simple struct grammar rule extraction
// ===========================================================================

#[test]
fn simple_struct_extracts_plain_fields() {
    let fields: Vec<Type> = vec![parse_quote!(String), parse_quote!(i32), parse_quote!(bool)];
    let fs = skip(&[]);
    let ws = skip(&[]);

    for ty in &fields {
        let (is_opt, is_rep, filtered, _, _) = pipeline_field(ty, &fs, &ws);
        assert!(!is_opt);
        assert!(!is_rep);
        assert_eq!(filtered, ts(ty));
    }
}

#[test]
fn simple_struct_wraps_all_plain_fields() {
    let fields: Vec<Type> = vec![
        parse_quote!(Ident),
        parse_quote!(Token),
        parse_quote!(Keyword),
    ];
    let ws = skip(&[]);

    for ty in &fields {
        let wrapped = wrap_leaf_type(ty, &ws);
        assert_eq!(ts(&wrapped), format!("adze :: WithLeaf < {} >", ts(ty)));
    }
}

// ===========================================================================
// 2. Enum grammar rule extraction
// ===========================================================================

#[test]
fn enum_variants_extract_as_choice_alternatives() {
    let variants: Vec<FieldThenParams> = vec![
        parse_quote!(LitExpr),
        parse_quote!(BinExpr),
        parse_quote!(UnaryExpr),
        parse_quote!(CallExpr),
    ];

    let names: Vec<String> = variants.iter().map(|v| ts(&v.field.ty)).collect();

    assert_eq!(names, vec!["LitExpr", "BinExpr", "UnaryExpr", "CallExpr"]);
}

#[test]
fn enum_variants_wrap_independently() {
    let variants: Vec<Type> = vec![parse_quote!(LitExpr), parse_quote!(Box<BinExpr>)];
    let fs = skip(&["Box"]);
    let ws = skip(&[]);

    let filtered: Vec<String> = variants
        .iter()
        .map(|ty| ts(&filter_inner_type(ty, &fs)))
        .collect();
    assert_eq!(filtered, vec!["LitExpr", "BinExpr"]);

    let wrapped: Vec<String> = variants
        .iter()
        .map(|ty| {
            let f = filter_inner_type(ty, &fs);
            ts(&wrap_leaf_type(&f, &ws))
        })
        .collect();
    assert_eq!(
        wrapped,
        vec![
            "adze :: WithLeaf < LitExpr >",
            "adze :: WithLeaf < BinExpr >",
        ]
    );
}

// ===========================================================================
// 3. Struct with Option fields (optional rules)
// ===========================================================================

#[test]
fn option_field_detected_as_optional() {
    let ty: Type = parse_quote!(Option<Label>);
    let (inner, is_opt) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(is_opt);
    assert_eq!(ts(&inner), "Label");
}

#[test]
fn option_field_wrapping_preserves_container() {
    let ty: Type = parse_quote!(Option<TypeAnnotation>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Option"]));
    assert_eq!(
        ts(&wrapped),
        "Option < adze :: WithLeaf < TypeAnnotation > >"
    );
}

#[test]
fn option_box_field_extracts_through_box() {
    let ty: Type = parse_quote!(Box<Option<ReturnType>>);
    let (inner, is_opt) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(is_opt);
    assert_eq!(ts(&inner), "ReturnType");
}

// ===========================================================================
// 4. Struct with Vec fields (repeat rules)
// ===========================================================================

#[test]
fn vec_field_detected_as_repeat() {
    let ty: Type = parse_quote!(Vec<Statement>);
    let (inner, is_rep) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(is_rep);
    assert_eq!(ts(&inner), "Statement");
}

#[test]
fn vec_field_wrapping_preserves_container() {
    let ty: Type = parse_quote!(Vec<Param>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < Param > >");
}

#[test]
fn vec_with_separator_annotation() {
    let parsed: FieldThenParams = parse_quote!(Vec<Argument>, separator = ",");
    let (_, is_rep) = try_extract_inner_type(&parsed.field.ty, "Vec", &skip(&[]));
    assert!(is_rep);
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "separator");
}

// ===========================================================================
// 5. Struct with Box fields (recursive types)
// ===========================================================================

#[test]
fn box_field_stripped_for_recursive_reference() {
    let ty: Type = parse_quote!(Box<Expr>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Expr");
}

#[test]
fn box_option_recursive_pipeline() {
    let ty: Type = parse_quote!(Option<Box<Expr>>);
    let fs = skip(&["Box"]);

    // Extract Option (skipping through Box if needed)
    let (opt_inner, is_opt) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(is_opt);
    assert_eq!(ts(&opt_inner), "Box < Expr >");

    // Filter Box from the inner
    let filtered = filter_inner_type(&opt_inner, &fs);
    assert_eq!(ts(&filtered), "Expr");
}

#[test]
fn double_box_stripped_completely() {
    let ty: Type = parse_quote!(Box<Box<Leaf>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "Leaf");
}

// ===========================================================================
// 6. Enum with named variants
// ===========================================================================

#[test]
fn named_variant_fields_process_independently() {
    // Simulate: enum Stmt { Let { name: Ident, value: Option<Expr> }, Return { value: Expr } }
    // Each named field is processed through the pipeline independently.
    let let_fields: Vec<(&str, Type)> = vec![
        ("name", parse_quote!(Ident)),
        ("value", parse_quote!(Option<Expr>)),
    ];
    let return_fields: Vec<(&str, Type)> = vec![("value", parse_quote!(Expr))];

    let ws = skip(&["Option"]);

    for (name, ty) in &let_fields {
        let wrapped = wrap_leaf_type(ty, &ws);
        match *name {
            "name" => assert_eq!(ts(&wrapped), "adze :: WithLeaf < Ident >"),
            "value" => assert_eq!(ts(&wrapped), "Option < adze :: WithLeaf < Expr > >"),
            _ => unreachable!(),
        }
    }

    for (name, ty) in &return_fields {
        let wrapped = wrap_leaf_type(ty, &ws);
        assert_eq!(*name, "value");
        assert_eq!(ts(&wrapped), "adze :: WithLeaf < Expr >");
    }
}

#[test]
fn named_variant_with_annotations() {
    let field: FieldThenParams = parse_quote!(Expr, precedence = 2, assoc = "left");
    assert_eq!(field.params.len(), 2);
    assert_eq!(field.params[0].path.to_string(), "precedence");
    assert_eq!(field.params[1].path.to_string(), "assoc");
}

// ===========================================================================
// 7. Enum with unnamed variants
// ===========================================================================

#[test]
fn unnamed_variant_single_field_extraction() {
    // enum Token { Number(i32), Ident(String) } — each variant has one unnamed field
    let variant_types: Vec<Type> = vec![parse_quote!(i32), parse_quote!(String)];
    let ws = skip(&[]);

    let wrapped: Vec<String> = variant_types
        .iter()
        .map(|ty| ts(&wrap_leaf_type(ty, &ws)))
        .collect();
    assert_eq!(
        wrapped,
        vec!["adze :: WithLeaf < i32 >", "adze :: WithLeaf < String >"]
    );
}

#[test]
fn unnamed_variant_with_box_filter() {
    // enum Expr { Lit(i32), Binary(Box<BinExpr>), Unary(Box<UnaryExpr>) }
    let variant_types: Vec<Type> = vec![
        parse_quote!(i32),
        parse_quote!(Box<BinExpr>),
        parse_quote!(Box<UnaryExpr>),
    ];
    let fs = skip(&["Box"]);

    let filtered: Vec<String> = variant_types
        .iter()
        .map(|ty| ts(&filter_inner_type(ty, &fs)))
        .collect();
    assert_eq!(filtered, vec!["i32", "BinExpr", "UnaryExpr"]);
}

// ===========================================================================
// 8. Grammar with precedence annotations
// ===========================================================================

#[test]
fn precedence_integer_parsed_from_field_param() {
    let parsed: FieldThenParams = parse_quote!(AddExpr, precedence = 1);
    let prec_param = &parsed.params[0];
    assert_eq!(prec_param.path.to_string(), "precedence");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = &prec_param.expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 1);
    } else {
        panic!("Expected integer literal for precedence");
    }
}

#[test]
fn precedence_ordering_across_multiple_variants() {
    let variants: Vec<FieldThenParams> = vec![
        parse_quote!(AddExpr, precedence = 1),
        parse_quote!(MulExpr, precedence = 2),
        parse_quote!(PowExpr, precedence = 3),
    ];

    let prec_values: Vec<i32> = variants
        .iter()
        .map(|v| {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(i),
                ..
            }) = &v.params[0].expr
            {
                i.base10_parse().unwrap()
            } else {
                panic!("Expected int literal")
            }
        })
        .collect();

    assert!(prec_values.windows(2).all(|w| w[0] < w[1]));
}

// ===========================================================================
// 9. Grammar with associativity annotations
// ===========================================================================

#[test]
fn associativity_left_parsed() {
    let parsed: FieldThenParams = parse_quote!(SubExpr, associativity = "left");
    assert_eq!(parsed.params[0].path.to_string(), "associativity");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &parsed.params[0].expr
    {
        assert_eq!(s.value(), "left");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn associativity_right_parsed() {
    let parsed: FieldThenParams = parse_quote!(PowExpr, associativity = "right");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &parsed.params[0].expr
    {
        assert_eq!(s.value(), "right");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn precedence_and_associativity_combined() {
    let parsed: FieldThenParams = parse_quote!(DivExpr, precedence = 5, associativity = "left");
    assert_eq!(parsed.params.len(), 2);

    let prec = &parsed.params[0];
    let assoc = &parsed.params[1];
    assert_eq!(prec.path.to_string(), "precedence");
    assert_eq!(assoc.path.to_string(), "associativity");

    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = &prec.expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 5);
    } else {
        panic!("Expected integer");
    }
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &assoc.expr
    {
        assert_eq!(s.value(), "left");
    } else {
        panic!("Expected string");
    }
}

// ===========================================================================
// 10. Grammar with leaf annotations
// ===========================================================================

#[test]
fn leaf_pattern_annotation() {
    let parsed: FieldThenParams = parse_quote!(String, pattern = "[a-zA-Z_][a-zA-Z0-9_]*");
    assert_eq!(parsed.params[0].path.to_string(), "pattern");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &parsed.params[0].expr
    {
        assert_eq!(s.value(), "[a-zA-Z_][a-zA-Z0-9_]*");
    } else {
        panic!("Expected string literal for pattern");
    }
}

#[test]
fn leaf_text_annotation() {
    let expr: NameValueExpr = parse_quote!(text = "fn");
    assert_eq!(expr.path.to_string(), "text");
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &expr.expr
    {
        assert_eq!(s.value(), "fn");
    } else {
        panic!("Expected string literal for text");
    }
}

#[test]
fn leaf_with_transform_closure() {
    let parsed: FieldThenParams = parse_quote!(
        i64,
        pattern = "-?\\d+",
        transform = |v: String| v.parse::<i64>().unwrap()
    );
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "pattern");
    assert_eq!(parsed.params[1].path.to_string(), "transform");
    assert!(matches!(parsed.params[1].expr, syn::Expr::Closure(_)));
}

#[test]
fn leaf_wrapped_in_with_leaf() {
    let ty: Type = parse_quote!(String);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < String >");
}

// ===========================================================================
// 11. Multiple types in same module
// ===========================================================================

#[test]
fn multiple_types_processed_independently() {
    // Simulate two types in one module: struct Decl { ... } and enum Expr { ... }
    let decl_fields: Vec<Type> = vec![
        parse_quote!(Keyword),
        parse_quote!(Ident),
        parse_quote!(Option<TypeAnnotation>),
    ];
    let expr_variants: Vec<Type> = vec![parse_quote!(LitExpr), parse_quote!(Box<BinExpr>)];

    let ws = skip(&["Option", "Vec"]);
    let fs = skip(&["Box"]);

    // Process Decl fields
    let decl_wrapped: Vec<String> = decl_fields
        .iter()
        .map(|ty| ts(&wrap_leaf_type(ty, &ws)))
        .collect();
    assert_eq!(decl_wrapped[0], "adze :: WithLeaf < Keyword >");
    assert_eq!(decl_wrapped[1], "adze :: WithLeaf < Ident >");
    assert_eq!(
        decl_wrapped[2],
        "Option < adze :: WithLeaf < TypeAnnotation > >"
    );

    // Process Expr variants (filter then wrap)
    let expr_filtered: Vec<String> = expr_variants
        .iter()
        .map(|ty| ts(&filter_inner_type(ty, &fs)))
        .collect();
    assert_eq!(expr_filtered, vec!["LitExpr", "BinExpr"]);
}

#[test]
fn multiple_types_no_cross_contamination_in_skip_sets() {
    let ty: Type = parse_quote!(Box<Node>);

    // Type A treats Box as transparent
    let fa = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&fa), "Node");

    // Type B does NOT treat Box as transparent
    let fb = filter_inner_type(&ty, &skip(&[]));
    assert_eq!(ts(&fb), "Box < Node >");
}

// ===========================================================================
// 12. Nested module grammar extraction
// ===========================================================================

#[test]
fn nested_module_qualified_type_filter() {
    // Types from a nested module: sub::Expr, sub::Stmt
    let ty: Type = parse_quote!(std::option::Option<sub::Expr>);
    let (inner, is_opt) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(is_opt);
    assert_eq!(ts(&inner), "sub :: Expr");
}

#[test]
fn nested_module_qualified_skip_type() {
    // std::boxed::Box should match "Box" in skip set (last segment)
    let ty: Type = parse_quote!(std::boxed::Box<sub::Node>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "sub :: Node");
}

#[test]
fn nested_module_qualified_wrap() {
    let ty: Type = parse_quote!(std::vec::Vec<module::Item>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(
        ts(&wrapped),
        "std :: vec :: Vec < adze :: WithLeaf < module :: Item > >"
    );
}

// ===========================================================================
// 13. Type with lifetime parameters
// ===========================================================================

#[test]
fn lifetime_reference_type_wrap() {
    let ty: Type = parse_quote!(&'a str);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < & 'a str >");
}

#[test]
fn lifetime_reference_type_filter_passthrough() {
    let ty: Type = parse_quote!(&'a str);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ts(&filtered), "& 'a str");
}

#[test]
fn lifetime_reference_type_extract_no_match() {
    let ty: Type = parse_quote!(&'static str);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ts(&inner), "& 'static str");
}

// ===========================================================================
// 14. Type with generic parameters
// ===========================================================================

#[test]
fn generic_type_not_in_skip_wraps_entirely() {
    let ty: Type = parse_quote!(HashMap<String, i32>);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(
        ts(&wrapped),
        "adze :: WithLeaf < HashMap < String , i32 > >"
    );
}

#[test]
fn generic_type_in_skip_wraps_each_arg() {
    let ty: Type = parse_quote!(Result<OkType, ErrType>);
    let wrapped = wrap_leaf_type(&ty, &skip(&["Result"]));
    assert_eq!(
        ts(&wrapped),
        "Result < adze :: WithLeaf < OkType > , adze :: WithLeaf < ErrType > >"
    );
}

#[test]
fn generic_nested_in_option() {
    let ty: Type = parse_quote!(Option<HashMap<K, V>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    assert_eq!(ts(&inner), "HashMap < K , V >");
}

#[test]
fn generic_vec_of_generic() {
    let ty: Type = parse_quote!(Vec<Pair<A, B>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ts(&inner), "Pair < A , B >");
}

// ===========================================================================
// 15. Type with where clauses (simulated via complex type patterns)
// ===========================================================================

#[test]
fn where_clause_constrained_type_wraps() {
    // Types that would appear in `where T: Display` contexts still process as types.
    let ty: Type = parse_quote!(T);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < T >");
}

#[test]
fn where_clause_bounded_generic_in_container() {
    // Vec<T> where T: Clone — the pipeline sees Vec<T> regardless of bounds.
    let ty: Type = parse_quote!(Vec<T>);
    let (inner, is_rep) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(is_rep);
    assert_eq!(ts(&inner), "T");

    let wrapped = wrap_leaf_type(&ty, &skip(&["Vec"]));
    assert_eq!(ts(&wrapped), "Vec < adze :: WithLeaf < T > >");
}

#[test]
fn where_clause_option_bounded_generic() {
    // Option<T> where T: Default
    let ty: Type = parse_quote!(Option<T>);
    let (inner, is_opt) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(is_opt);
    assert_eq!(ts(&inner), "T");
}

// ===========================================================================
// Full end-to-end pipeline simulations
// ===========================================================================

#[test]
fn e2e_arithmetic_grammar_pipeline() {
    // Simulate a minimal arithmetic grammar:
    //   struct Num { value: i32 }
    //   enum Expr { Num(Num), Add { lhs: Box<Expr>, rhs: Box<Expr> } }
    let fs = skip(&["Box"]);
    let ws = skip(&["Option", "Vec"]);

    // Struct Num: single field
    let num_ty: Type = parse_quote!(i32);
    let (is_opt, is_rep, filtered, wrapped, _) = pipeline_field(&num_ty, &fs, &ws);
    assert!(!is_opt);
    assert!(!is_rep);
    assert_eq!(filtered, "i32");
    assert_eq!(wrapped, "adze :: WithLeaf < i32 >");

    // Enum Expr variant Num: plain type
    let num_variant: Type = parse_quote!(Num);
    let filtered_variant = filter_inner_type(&num_variant, &fs);
    assert_eq!(ts(&filtered_variant), "Num");

    // Enum Expr variant Add: Box<Expr> fields
    let add_lhs: Type = parse_quote!(Box<Expr>);
    let add_rhs: Type = parse_quote!(Box<Expr>);
    assert_eq!(ts(&filter_inner_type(&add_lhs, &fs)), "Expr");
    assert_eq!(ts(&filter_inner_type(&add_rhs, &fs)), "Expr");
}

#[test]
fn e2e_function_definition_grammar() {
    // struct FnDef { name: Ident, params: Vec<Param>, ret: Option<Type>, body: Vec<Stmt> }
    let fs = skip(&["Box"]);
    let ws = skip(&["Option", "Vec"]);

    let fields: Vec<(&str, Type)> = vec![
        ("name", parse_quote!(Ident)),
        ("params", parse_quote!(Vec<Param>)),
        ("ret", parse_quote!(Option<ReturnType>)),
        ("body", parse_quote!(Vec<Stmt>)),
    ];

    for (name, ty) in &fields {
        let (is_opt, is_rep, _, wrapped, _) = pipeline_field(ty, &fs, &ws);
        match *name {
            "name" => {
                assert!(!is_opt);
                assert!(!is_rep);
                assert_eq!(wrapped, "adze :: WithLeaf < Ident >");
            }
            "params" => {
                assert!(!is_opt);
                assert!(is_rep);
                assert_eq!(wrapped, "Vec < adze :: WithLeaf < Param > >");
            }
            "ret" => {
                assert!(is_opt);
                assert!(!is_rep);
                assert_eq!(wrapped, "Option < adze :: WithLeaf < ReturnType > >");
            }
            "body" => {
                assert!(!is_opt);
                assert!(is_rep);
                assert_eq!(wrapped, "Vec < adze :: WithLeaf < Stmt > >");
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn e2e_expression_with_precedence_and_associativity() {
    // Simulate enum Expr with precedence/associativity on each variant
    let variants: Vec<FieldThenParams> = vec![
        parse_quote!(AddExpr, precedence = 1, associativity = "left"),
        parse_quote!(MulExpr, precedence = 2, associativity = "left"),
        parse_quote!(PowExpr, precedence = 3, associativity = "right"),
        parse_quote!(NegExpr, precedence = 4),
    ];

    for (i, v) in variants.iter().enumerate() {
        let ty = &v.field.ty;
        let wrapped = wrap_leaf_type(ty, &skip(&[]));
        let type_name = ts(ty);
        assert_eq!(ts(&wrapped), format!("adze :: WithLeaf < {} >", type_name));

        // Check precedence
        let prec_param = &v.params[0];
        assert_eq!(prec_param.path.to_string(), "precedence");
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit_int),
            ..
        }) = &prec_param.expr
        {
            assert_eq!(lit_int.base10_parse::<i32>().unwrap(), (i as i32) + 1);
        } else {
            panic!("Expected int literal for precedence");
        }

        // Check associativity if present
        if v.params.len() > 1 {
            let assoc_param = &v.params[1];
            assert_eq!(assoc_param.path.to_string(), "associativity");
        }
    }
}
