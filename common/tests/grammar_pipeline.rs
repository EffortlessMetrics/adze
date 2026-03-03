//! Integration tests for the common crate's grammar expansion pipeline.
//!
//! These tests verify the type-analysis and annotation-parsing utilities that
//! form the shared foundation of adze's grammar expansion pipeline (used by
//! both the macro and tool crates).  We exercise the full flow from annotated
//! Rust types through extraction, filtering, and wrapping — without touching
//! Tree-sitter integration (which lives in the tool crate).

use std::collections::{HashMap, HashSet};

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ===========================================================================
// 1. Full expansion: annotated Rust type → processed leaf type
// ===========================================================================

#[test]
fn full_pipeline_simple_leaf() {
    let parsed: FieldThenParams = parse_quote!(i32, pattern = "\\d+");

    let skip: HashSet<&str> = HashSet::new();
    let ty = &parsed.field.ty;

    // No container to strip
    let (inner, extracted) = try_extract_inner_type(ty, "Option", &skip);
    assert!(!extracted);

    let filtered = filter_inner_type(&inner, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "i32");

    // Wrap for leaf processing
    let wrapped = wrap_leaf_type(&filtered, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < i32 >"
    );

    // Pattern param preserved
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "pattern");
}

#[test]
fn full_pipeline_optional_field() {
    let ty: Type = parse_quote!(Option<String>);
    let skip_wrap: HashSet<&str> = ["Option"].into_iter().collect();

    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "String");

    let wrapped = wrap_leaf_type(&ty, &skip_wrap);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < adze :: WithLeaf < String > >"
    );
}

#[test]
fn full_pipeline_vec_field() {
    let ty: Type = parse_quote!(Vec<Expr>);
    let skip_wrap: HashSet<&str> = ["Vec"].into_iter().collect();

    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Expr");

    let wrapped = wrap_leaf_type(&ty, &skip_wrap);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < Expr > >"
    );
}

#[test]
fn full_pipeline_nested_option_vec() {
    let ty: Type = parse_quote!(Option<Vec<Item>>);
    let skip_wrap: HashSet<&str> = ["Option", "Vec"].into_iter().collect();

    // Layer 1: Option
    let (after_option, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(extracted);
    assert_eq!(after_option.to_token_stream().to_string(), "Vec < Item >");

    // Layer 2: Vec
    let (after_vec, extracted) = try_extract_inner_type(&after_option, "Vec", &HashSet::new());
    assert!(extracted);
    assert_eq!(after_vec.to_token_stream().to_string(), "Item");

    // Full wrap preserves both containers
    let wrapped = wrap_leaf_type(&ty, &skip_wrap);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < Vec < adze :: WithLeaf < Item > > >"
    );
}

#[test]
fn full_pipeline_box_spanned_mirror_real_usage() {
    // The real pipeline uses skip_over = {"Spanned", "Box"} when extracting
    let skip_over: HashSet<&str> = ["Spanned", "Box"].into_iter().collect();

    let ty: Type = parse_quote!(Box<Vec<Token>>);
    let (inner, is_vec) = try_extract_inner_type(&ty, "Vec", &skip_over);
    assert!(is_vec);
    assert_eq!(inner.to_token_stream().to_string(), "Token");

    let ty: Type = parse_quote!(Box<Option<Ident>>);
    let (inner, is_opt) = try_extract_inner_type(&ty, "Option", &skip_over);
    assert!(is_opt);
    assert_eq!(inner.to_token_stream().to_string(), "Ident");
}

// ===========================================================================
// 2. Token extraction and naming
// ===========================================================================

#[test]
fn token_pattern_extraction() {
    let parsed: FieldThenParams = parse_quote!(String, pattern = "[a-zA-Z_][a-zA-Z0-9_]*");
    assert_eq!(parsed.params[0].path.to_string(), "pattern");

    if let syn::Expr::Lit(lit) = &parsed.params[0].expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "[a-zA-Z_][a-zA-Z0-9_]*");
        return;
    }
    panic!("Expected string literal for pattern");
}

#[test]
fn token_text_extraction() {
    let expr: NameValueExpr = parse_quote!(text = "+");
    assert_eq!(expr.path.to_string(), "text");

    if let syn::Expr::Lit(lit) = &expr.expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "+");
        return;
    }
    panic!("Expected string literal for text");
}

#[test]
fn token_pattern_with_transform() {
    let parsed: FieldThenParams = parse_quote!(
        i32,
        pattern = "\\d+",
        transform = |v: String| v.parse::<i32>().unwrap()
    );
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "pattern");
    assert_eq!(parsed.params[1].path.to_string(), "transform");
    assert!(matches!(parsed.params[1].expr, syn::Expr::Closure(_)));
}

#[test]
fn token_symbol_name_from_filtered_type() {
    // After filtering containers, the single-segment path is the symbol name.
    let skip: HashSet<&str> = ["Spanned", "Box"].into_iter().collect();

    let ty: Type = parse_quote!(Box<Identifier>);
    let filtered = filter_inner_type(&ty, &skip);
    if let Type::Path(p) = &filtered {
        assert_eq!(p.path.segments.len(), 1);
        assert_eq!(p.path.segments[0].ident.to_string(), "Identifier");
    } else {
        panic!("Expected path type after filtering");
    }
}

// ===========================================================================
// 3. Rule generation for pattern types (Optional, Repeat, Choice, Seq)
// ===========================================================================

#[test]
fn rule_optional() {
    let ty: Type = parse_quote!(Option<Identifier>);
    let (inner, is_optional) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(is_optional);
    assert_eq!(inner.to_token_stream().to_string(), "Identifier");
}

#[test]
fn rule_repeat() {
    let ty: Type = parse_quote!(Vec<Statement>);
    let (inner, is_repeat) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(is_repeat);
    assert_eq!(inner.to_token_stream().to_string(), "Statement");
}

#[test]
fn rule_optional_repeat() {
    let ty: Type = parse_quote!(Option<Vec<Arg>>);

    let (after_option, is_opt) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(is_opt);

    let (inner, is_rep) = try_extract_inner_type(&after_option, "Vec", &HashSet::new());
    assert!(is_rep);
    assert_eq!(inner.to_token_stream().to_string(), "Arg");
}

#[test]
fn rule_boxed_recursive() {
    let ty: Type = parse_quote!(Box<Expression>);
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Expression");
}

#[test]
fn rule_choice_via_enum_variants() {
    // Each variant field represents a choice alternative
    let variants: Vec<FieldThenParams> = vec![
        parse_quote!(LiteralExpr),
        parse_quote!(BinaryExpr),
        parse_quote!(UnaryExpr),
    ];

    let types: Vec<String> = variants
        .iter()
        .map(|f| f.field.ty.to_token_stream().to_string())
        .collect();

    assert_eq!(types, vec!["LiteralExpr", "BinaryExpr", "UnaryExpr"]);
}

#[test]
fn rule_sequence_via_struct_fields() {
    let fields: Vec<FieldThenParams> = vec![
        parse_quote!(Keyword),
        parse_quote!(Identifier),
        parse_quote!(Option<TypeAnnotation>),
    ];

    let types: Vec<String> = fields
        .iter()
        .map(|f| f.field.ty.to_token_stream().to_string())
        .collect();

    assert_eq!(
        types,
        vec!["Keyword", "Identifier", "Option < TypeAnnotation >"]
    );

    // Third field is optional in the sequence
    let (_, is_opt) = try_extract_inner_type(&fields[2].field.ty, "Option", &HashSet::new());
    assert!(is_opt);
}

#[test]
fn rule_vec_not_confused_with_option() {
    let ty: Type = parse_quote!(Vec<Foo>);
    let (_, is_opt) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(!is_opt);
}

#[test]
fn rule_option_not_confused_with_vec() {
    let ty: Type = parse_quote!(Option<Bar>);
    let (_, is_vec) = try_extract_inner_type(&ty, "Vec", &HashSet::new());
    assert!(!is_vec);
}

// ===========================================================================
// 4. Field mapping correctness
// ===========================================================================

#[test]
fn field_mapping_preserves_type_and_params() {
    let parsed: FieldThenParams = parse_quote!(Vec<String>, separator = ",", min = 1);

    assert_eq!(
        parsed.field.ty.to_token_stream().to_string(),
        "Vec < String >"
    );

    let param_map: HashMap<String, &syn::Expr> = parsed
        .params
        .iter()
        .map(|p| (p.path.to_string(), &p.expr))
        .collect();

    assert!(param_map.contains_key("separator"));
    assert!(param_map.contains_key("min"));
}

#[test]
fn field_mapping_option_vec_with_separator() {
    let parsed: FieldThenParams = parse_quote!(Option<Vec<Token>>, separator = ";");

    let (inner, extracted) = try_extract_inner_type(&parsed.field.ty, "Option", &HashSet::new());
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Vec < Token >");

    assert_eq!(parsed.params[0].path.to_string(), "separator");
}

#[test]
fn field_mapping_multiple_fields_independent() {
    let f1: FieldThenParams = parse_quote!(String, pattern = "\\w+");
    let f2: FieldThenParams = parse_quote!(i32, pattern = "\\d+");
    let f3: FieldThenParams = parse_quote!(bool);

    assert_eq!(f1.params.len(), 1);
    assert_eq!(f2.params.len(), 1);
    assert_eq!(f3.params.len(), 0);

    assert_eq!(f1.field.ty.to_token_stream().to_string(), "String");
    assert_eq!(f2.field.ty.to_token_stream().to_string(), "i32");
    assert_eq!(f3.field.ty.to_token_stream().to_string(), "bool");
}

// ===========================================================================
// 5. Precedence annotation handling
// ===========================================================================

#[test]
fn precedence_integer_annotation() {
    let expr: NameValueExpr = parse_quote!(precedence = 5);
    assert_eq!(expr.path.to_string(), "precedence");

    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = &expr.expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 5);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn precedence_zero() {
    let expr: NameValueExpr = parse_quote!(precedence = 0);
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(i),
        ..
    }) = &expr.expr
    {
        assert_eq!(i.base10_parse::<i32>().unwrap(), 0);
    } else {
        panic!("Expected integer literal");
    }
}

#[test]
fn precedence_combined_with_associativity() {
    let parsed: FieldThenParams = parse_quote!(BinaryOp, precedence = 3, assoc = "left");
    assert_eq!(parsed.params.len(), 2);
    assert_eq!(parsed.params[0].path.to_string(), "precedence");
    assert_eq!(parsed.params[1].path.to_string(), "assoc");
}

#[test]
fn precedence_ordering_across_levels() {
    let levels: Vec<NameValueExpr> = vec![
        parse_quote!(precedence = 1),
        parse_quote!(precedence = 5),
        parse_quote!(precedence = 10),
    ];

    let values: Vec<i32> = levels
        .iter()
        .map(|nv| {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(i),
                ..
            }) = &nv.expr
            {
                i.base10_parse().unwrap()
            } else {
                panic!("Expected int literal")
            }
        })
        .collect();

    assert!(values.windows(2).all(|w| w[0] < w[1]));
}

// ===========================================================================
// 6. Multiple grammar fragments merged correctly
// ===========================================================================

#[test]
fn multiple_fragments_consistent_wrapping() {
    let skip_wrap: HashSet<&str> = ["Option", "Vec"].into_iter().collect();

    let fields: Vec<FieldThenParams> = vec![
        parse_quote!(String, pattern = "\\w+"),
        parse_quote!(Vec<Expr>),
        parse_quote!(Option<String>),
        parse_quote!(i32, pattern = "\\d+"),
    ];

    let wrapped: Vec<String> = fields
        .iter()
        .map(|f| {
            wrap_leaf_type(&f.field.ty, &skip_wrap)
                .to_token_stream()
                .to_string()
        })
        .collect();

    assert_eq!(wrapped[0], "adze :: WithLeaf < String >");
    assert_eq!(wrapped[1], "Vec < adze :: WithLeaf < Expr > >");
    assert_eq!(wrapped[2], "Option < adze :: WithLeaf < String > >");
    assert_eq!(wrapped[3], "adze :: WithLeaf < i32 >");
}

#[test]
fn multiple_fragments_extract_all_optionals() {
    let types: Vec<Type> = vec![
        parse_quote!(Option<A>),
        parse_quote!(B),
        parse_quote!(Option<C>),
        parse_quote!(Vec<D>),
    ];

    let optionals: Vec<bool> = types
        .iter()
        .map(|ty| try_extract_inner_type(ty, "Option", &HashSet::new()).1)
        .collect();

    assert_eq!(optionals, vec![true, false, true, false]);
}

#[test]
fn multiple_fragments_filter_preserves_order() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();

    let types: Vec<Type> = vec![
        parse_quote!(Box<Alpha>),
        parse_quote!(Arc<Beta>),
        parse_quote!(Box<Arc<Gamma>>),
        parse_quote!(Delta),
    ];

    let filtered: Vec<String> = types
        .iter()
        .map(|ty| filter_inner_type(ty, &skip).to_token_stream().to_string())
        .collect();

    assert_eq!(filtered, vec!["Alpha", "Beta", "Gamma", "Delta"]);
}

#[test]
fn multiple_fragments_mixed_annotations() {
    let f1: FieldThenParams = parse_quote!(String, pattern = "\\w+");
    let f2: FieldThenParams = parse_quote!(BinOp, precedence = 2);
    let f3: FieldThenParams = parse_quote!(Vec<Stmt>, separator = ";");

    // Each fragment retains its own annotation set
    assert_eq!(f1.params[0].path.to_string(), "pattern");
    assert_eq!(f2.params[0].path.to_string(), "precedence");
    assert_eq!(f3.params[0].path.to_string(), "separator");
}

// ===========================================================================
// 7. Inline rule expansion
// ===========================================================================

#[test]
fn inline_strips_containers_then_wraps() {
    let ty: Type = parse_quote!(Box<Option<Vec<Token>>>);
    let strip: HashSet<&str> = ["Box"].into_iter().collect();

    // Strip Box
    let after_box = filter_inner_type(&ty, &strip);
    assert_eq!(
        after_box.to_token_stream().to_string(),
        "Option < Vec < Token > >"
    );

    // Detect Optional
    let (after_option, is_opt) = try_extract_inner_type(&after_box, "Option", &HashSet::new());
    assert!(is_opt);

    // Detect Repeat
    let (inner, is_rep) = try_extract_inner_type(&after_option, "Vec", &HashSet::new());
    assert!(is_rep);
    assert_eq!(inner.to_token_stream().to_string(), "Token");

    // Wrap leaf
    let wrapped = wrap_leaf_type(&inner, &HashSet::new());
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < Token >"
    );
}

#[test]
fn inline_spanned_wrapper_transparent() {
    let ty: Type = parse_quote!(Spanned<Identifier>);
    let strip: HashSet<&str> = ["Spanned"].into_iter().collect();

    let filtered = filter_inner_type(&ty, &strip);
    assert_eq!(filtered.to_token_stream().to_string(), "Identifier");
}

#[test]
fn inline_non_container_passthrough() {
    let ty: Type = parse_quote!(MyCustomRule);
    let strip: HashSet<&str> = ["Box", "Spanned"].into_iter().collect();

    let filtered = filter_inner_type(&ty, &strip);
    assert_eq!(filtered.to_token_stream().to_string(), "MyCustomRule");

    let wrapped = wrap_leaf_type(&filtered, &HashSet::new());
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < MyCustomRule >"
    );
}

#[test]
fn inline_deeply_nested_containers() {
    let ty: Type = parse_quote!(Box<Spanned<Arc<Expr>>>);
    let strip: HashSet<&str> = ["Box", "Spanned", "Arc"].into_iter().collect();

    let filtered = filter_inner_type(&ty, &strip);
    assert_eq!(filtered.to_token_stream().to_string(), "Expr");
}

#[test]
fn inline_wrap_preserves_grammar_containers() {
    // Vec and Option are grammar-meaningful; Box and Spanned are not
    let ty: Type = parse_quote!(Box<Option<Vec<Spanned<Leaf>>>>);

    // Step 1: strip transparent wrappers
    let strip: HashSet<&str> = ["Box", "Spanned"].into_iter().collect();
    let stripped = filter_inner_type(&ty, &strip);
    assert_eq!(
        stripped.to_token_stream().to_string(),
        "Option < Vec < Spanned < Leaf > > >"
    );

    // Step 2: wrap preserving grammar containers
    // For wrapping, we also skip through Spanned to reach the leaf
    let wrap_skip: HashSet<&str> = ["Option", "Vec", "Spanned"].into_iter().collect();
    let wrapped = wrap_leaf_type(&stripped, &wrap_skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Option < Vec < Spanned < adze :: WithLeaf < Leaf > > > >"
    );
}

// ===========================================================================
// 8. Error reporting for invalid grammar definitions
// ===========================================================================

#[test]
fn error_empty_field_fails() {
    let result = syn::parse_str::<FieldThenParams>("");
    assert!(result.is_err());
}

#[test]
fn error_name_value_missing_equals() {
    let result = syn::parse_str::<NameValueExpr>("key value");
    assert!(result.is_err());
}

#[test]
fn error_name_value_missing_value() {
    let result = syn::parse_str::<NameValueExpr>("key =");
    assert!(result.is_err());
}

#[test]
fn error_name_value_missing_name() {
    let result = syn::parse_str::<NameValueExpr>("= 42");
    assert!(result.is_err());
}

#[test]
fn error_bare_comma_as_field() {
    let result = syn::parse_str::<FieldThenParams>(",");
    assert!(result.is_err());
}

#[test]
fn error_double_comma_in_params() {
    let result = syn::parse_str::<FieldThenParams>("String, , key = 1");
    assert!(result.is_err());
}

#[test]
fn error_param_without_value() {
    let result = syn::parse_str::<FieldThenParams>("String, key");
    assert!(result.is_err());
}

#[test]
fn error_completely_invalid_syntax() {
    let result = syn::parse_str::<NameValueExpr>("!@#$%");
    assert!(result.is_err());
}
