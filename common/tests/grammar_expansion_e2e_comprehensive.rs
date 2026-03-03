#![allow(clippy::needless_range_loop)]

//! End-to-end tests for the full grammar expansion pipeline in adze-common.
//!
//! Each test simulates the real expansion path: annotated Rust types flow
//! through extraction → filtering → wrapping, exactly as the macro and tool
//! crates would process them when expanding grammar definitions.

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

/// Simulate the real pipeline for a single grammar field:
/// 1. try to extract Option  (marks field optional)
/// 2. try to extract Vec     (marks field as repetition)
/// 3. filter transparent wrappers (Box, Spanned)
/// 4. wrap leaf with `adze::WithLeaf`
struct FieldResult {
    is_optional: bool,
    is_repeat: bool,
    leaf: String,
    wrapped: String,
}

fn process_field(ty: &Type) -> FieldResult {
    let extract_skip = skip(&["Box", "Spanned"]);
    let wrap_skip = skip(&["Option", "Vec"]);
    let filter_skip = skip(&["Box", "Spanned"]);

    let (after_opt, is_optional) = try_extract_inner_type(ty, "Option", &extract_skip);
    let cur = if is_optional { after_opt } else { ty.clone() };

    let (after_vec, is_repeat) = try_extract_inner_type(&cur, "Vec", &extract_skip);
    let cur = if is_repeat { after_vec } else { cur };

    let filtered = filter_inner_type(&cur, &filter_skip);
    let wrapped = wrap_leaf_type(&filtered, &wrap_skip);

    FieldResult {
        is_optional,
        is_repeat,
        leaf: ts(&filtered),
        wrapped: ts(&wrapped),
    }
}

// ===========================================================================
// 1. Struct with fields → grammar rules (sequence)
// ===========================================================================

#[test]
fn struct_simple_two_fields_sequence() {
    let fields: Vec<Type> = vec![parse_quote!(Keyword), parse_quote!(Identifier)];
    let results: Vec<FieldResult> = fields.iter().map(process_field).collect();

    for i in 0..results.len() {
        assert!(!results[i].is_optional);
        assert!(!results[i].is_repeat);
    }
    assert_eq!(results[0].leaf, "Keyword");
    assert_eq!(results[1].leaf, "Identifier");
}

#[test]
fn struct_mixed_required_and_optional_fields() {
    let fields: Vec<Type> = vec![
        parse_quote!(Keyword),
        parse_quote!(Option<Modifier>),
        parse_quote!(Identifier),
        parse_quote!(Option<TypeAnnotation>),
    ];
    let results: Vec<FieldResult> = fields.iter().map(process_field).collect();

    assert!(!results[0].is_optional);
    assert!(results[1].is_optional);
    assert!(!results[2].is_optional);
    assert!(results[3].is_optional);

    assert_eq!(results[1].leaf, "Modifier");
    assert_eq!(results[3].leaf, "TypeAnnotation");
}

#[test]
fn struct_with_repetition_field() {
    let fields: Vec<Type> = vec![
        parse_quote!(Keyword),
        parse_quote!(Vec<Statement>),
        parse_quote!(EndKeyword),
    ];
    let results: Vec<FieldResult> = fields.iter().map(process_field).collect();

    assert!(!results[0].is_repeat);
    assert!(results[1].is_repeat);
    assert!(!results[2].is_repeat);
    assert_eq!(results[1].leaf, "Statement");
}

// ===========================================================================
// 2. Enum with variants → choice rules
// ===========================================================================

#[test]
fn enum_variants_as_choice_alternatives() {
    let variant_types: Vec<Type> = vec![
        parse_quote!(LiteralExpr),
        parse_quote!(BinaryExpr),
        parse_quote!(UnaryExpr),
        parse_quote!(CallExpr),
    ];

    let leaves: Vec<String> = variant_types.iter().map(|ty| process_field(ty).leaf).collect();
    assert_eq!(leaves, ["LiteralExpr", "BinaryExpr", "UnaryExpr", "CallExpr"]);
}

#[test]
fn enum_variant_with_optional_payload() {
    let variant_types: Vec<Type> = vec![
        parse_quote!(IntLit),
        parse_quote!(Option<FloatLit>),
        parse_quote!(StringLit),
    ];
    let results: Vec<FieldResult> = variant_types.iter().map(process_field).collect();

    assert!(!results[0].is_optional);
    assert!(results[1].is_optional);
    assert!(!results[2].is_optional);
    assert_eq!(results[1].leaf, "FloatLit");
}

#[test]
fn enum_variant_with_boxed_recursive_payload() {
    let variant_types: Vec<Type> = vec![
        parse_quote!(Literal),
        parse_quote!(Box<BinaryExpr>),
        parse_quote!(Box<UnaryExpr>),
    ];
    let results: Vec<FieldResult> = variant_types.iter().map(process_field).collect();

    // Box is transparent; all variants resolve to their inner leaf
    assert_eq!(results[0].leaf, "Literal");
    assert_eq!(results[1].leaf, "BinaryExpr");
    assert_eq!(results[2].leaf, "UnaryExpr");
}

// ===========================================================================
// 3. Nested struct/enum → nested rules
// ===========================================================================

#[test]
fn nested_struct_fields_referencing_other_rules() {
    // Outer struct: FnDecl { name: Ident, params: Vec<Param>, body: Block }
    // Inner struct: Param { ty: TypeExpr, name: Ident }
    let outer_fields: Vec<Type> = vec![
        parse_quote!(Ident),
        parse_quote!(Vec<Param>),
        parse_quote!(Block),
    ];
    let inner_fields: Vec<Type> = vec![parse_quote!(TypeExpr), parse_quote!(Ident)];

    let outer: Vec<FieldResult> = outer_fields.iter().map(process_field).collect();
    let inner: Vec<FieldResult> = inner_fields.iter().map(process_field).collect();

    assert!(outer[1].is_repeat);
    assert_eq!(outer[1].leaf, "Param");
    assert_eq!(inner[0].leaf, "TypeExpr");
    assert_eq!(inner[1].leaf, "Ident");
}

#[test]
fn nested_enum_inside_struct_field() {
    // Struct field references an enum type
    let struct_fields: Vec<Type> = vec![
        parse_quote!(Operator),
        parse_quote!(Box<Expr>),
        parse_quote!(Box<Expr>),
    ];
    // The Expr enum has these variants
    let enum_variants: Vec<Type> = vec![
        parse_quote!(Literal),
        parse_quote!(Box<BinaryExpr>),
    ];

    let sresults: Vec<FieldResult> = struct_fields.iter().map(process_field).collect();
    let eresults: Vec<FieldResult> = enum_variants.iter().map(process_field).collect();

    assert_eq!(sresults[1].leaf, "Expr");
    assert_eq!(sresults[2].leaf, "Expr");
    assert_eq!(eresults[0].leaf, "Literal");
    assert_eq!(eresults[1].leaf, "BinaryExpr");
}

#[test]
fn deeply_nested_three_levels() {
    // Module → FnDecl → Param → TypeAnnotation
    let mod_fields: Vec<Type> = vec![parse_quote!(Vec<FnDecl>)];
    let fn_fields: Vec<Type> = vec![parse_quote!(Ident), parse_quote!(Vec<Param>)];
    let param_fields: Vec<Type> = vec![parse_quote!(Ident), parse_quote!(Option<TypeAnnotation>)];

    let mr: Vec<FieldResult> = mod_fields.iter().map(process_field).collect();
    let fr: Vec<FieldResult> = fn_fields.iter().map(process_field).collect();
    let pr: Vec<FieldResult> = param_fields.iter().map(process_field).collect();

    assert!(mr[0].is_repeat);
    assert_eq!(mr[0].leaf, "FnDecl");
    assert!(fr[1].is_repeat);
    assert_eq!(fr[1].leaf, "Param");
    assert!(pr[1].is_optional);
    assert_eq!(pr[1].leaf, "TypeAnnotation");
}

// ===========================================================================
// 4. Optional + Vec + Box combinations
// ===========================================================================

#[test]
fn option_vec_combined() {
    let ty: Type = parse_quote!(Option<Vec<Arg>>);
    let r = process_field(&ty);
    assert!(r.is_optional);
    assert!(r.is_repeat);
    assert_eq!(r.leaf, "Arg");
}

#[test]
fn box_option_combined() {
    let ty: Type = parse_quote!(Box<Option<ReturnType>>);
    let r = process_field(&ty);
    // Box is skipped during extraction, so Option is found
    assert!(r.is_optional);
    assert!(!r.is_repeat);
    assert_eq!(r.leaf, "ReturnType");
}

#[test]
fn box_vec_combined() {
    let ty: Type = parse_quote!(Box<Vec<Field>>);
    let r = process_field(&ty);
    assert!(!r.is_optional);
    assert!(r.is_repeat);
    assert_eq!(r.leaf, "Field");
}

#[test]
fn box_option_vec_triple_nesting() {
    let ty: Type = parse_quote!(Box<Option<Vec<Decorator>>>);
    let r = process_field(&ty);
    assert!(r.is_optional);
    assert!(r.is_repeat);
    assert_eq!(r.leaf, "Decorator");
}

#[test]
fn spanned_box_option_transparent_wrappers() {
    let ty: Type = parse_quote!(Spanned<Box<Option<Label>>>);
    let r = process_field(&ty);
    assert!(r.is_optional);
    assert_eq!(r.leaf, "Label");
}

#[test]
fn plain_type_no_wrappers() {
    let ty: Type = parse_quote!(Identifier);
    let r = process_field(&ty);
    assert!(!r.is_optional);
    assert!(!r.is_repeat);
    assert_eq!(r.leaf, "Identifier");
    assert_eq!(r.wrapped, "adze :: WithLeaf < Identifier >");
}

// ===========================================================================
// 5. Skip + leaf + prec combinations
// ===========================================================================

#[test]
fn skip_annotation_on_leaf_field() {
    // A field with a skip pattern (e.g., whitespace) is parsed but not in the AST
    let parsed: FieldThenParams = parse_quote!(String, pattern = "\\s+");
    let r = process_field(&parsed.field.ty);
    assert_eq!(r.leaf, "String");
    assert_eq!(parsed.params[0].path.to_string(), "pattern");
}

#[test]
fn leaf_with_precedence_and_transform() {
    let parsed: FieldThenParams = parse_quote!(
        i64,
        pattern = "\\d+",
        precedence = 3,
        transform = |s: String| s.parse::<i64>().unwrap()
    );
    assert_eq!(parsed.params.len(), 3);
    assert_eq!(parsed.params[0].path.to_string(), "pattern");
    assert_eq!(parsed.params[1].path.to_string(), "precedence");
    assert_eq!(parsed.params[2].path.to_string(), "transform");

    let r = process_field(&parsed.field.ty);
    assert_eq!(r.leaf, "i64");
}

#[test]
fn prec_with_associativity_left_right() {
    let left: FieldThenParams = parse_quote!(AddExpr, precedence = 1, assoc = "left");
    let right: FieldThenParams = parse_quote!(PowExpr, precedence = 3, assoc = "right");

    assert_eq!(left.params[1].path.to_string(), "assoc");
    assert_eq!(right.params[1].path.to_string(), "assoc");

    // Extract assoc string values
    let extract_str = |nv: &NameValueExpr| -> String {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &nv.expr
        {
            s.value()
        } else {
            panic!("expected string literal")
        }
    };
    assert_eq!(extract_str(&left.params[1]), "left");
    assert_eq!(extract_str(&right.params[1]), "right");
}

// ===========================================================================
// 6. Extra whitespace handling
// ===========================================================================

#[test]
fn whitespace_pattern_as_extra_rule() {
    let ws: FieldThenParams = parse_quote!(String, pattern = "\\s+");
    let r = process_field(&ws.field.ty);
    assert_eq!(r.leaf, "String");

    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(s),
        ..
    }) = &ws.params[0].expr
    {
        assert_eq!(s.value(), "\\s+");
    } else {
        panic!("expected string literal");
    }
}

#[test]
fn comment_patterns_as_extras() {
    let line_comment: NameValueExpr = parse_quote!(pattern = "//[^\\n]*");
    let block_comment: NameValueExpr = parse_quote!(pattern = "/\\*[^*]*\\*+(?:[^/*][^*]*\\*+)*/");

    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &line_comment.expr {
        assert!(s.value().starts_with("//"));
    } else {
        panic!("expected string literal");
    }
    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &block_comment.expr {
        assert!(s.value().starts_with("/\\*"));
    } else {
        panic!("expected string literal");
    }
}

// ===========================================================================
// 7. Word annotation processing
// ===========================================================================

#[test]
fn word_pattern_identifier_rule() {
    // Simulates a word-boundary leaf for identifiers
    let parsed: FieldThenParams = parse_quote!(String, pattern = "[a-zA-Z_][a-zA-Z0-9_]*");
    let r = process_field(&parsed.field.ty);
    assert_eq!(r.leaf, "String");
    assert_eq!(r.wrapped, "adze :: WithLeaf < String >");

    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &parsed.params[0].expr {
        assert!(s.value().contains("[a-zA-Z_]"));
    } else {
        panic!("expected string literal");
    }
}

#[test]
fn word_keyword_as_fixed_text() {
    let kw: NameValueExpr = parse_quote!(text = "fn");
    assert_eq!(kw.path.to_string(), "text");

    if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &kw.expr {
        assert_eq!(s.value(), "fn");
    } else {
        panic!("expected string literal");
    }
}

#[test]
fn word_multiple_keywords_distinct() {
    let keywords: Vec<NameValueExpr> = vec![
        parse_quote!(text = "if"),
        parse_quote!(text = "else"),
        parse_quote!(text = "while"),
        parse_quote!(text = "return"),
    ];

    let values: Vec<String> = keywords
        .iter()
        .map(|kw| {
            if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &kw.expr {
                s.value()
            } else {
                panic!("expected string literal")
            }
        })
        .collect();

    assert_eq!(values, ["if", "else", "while", "return"]);
    // All distinct
    let unique: HashSet<&str> = values.iter().map(|s| s.as_str()).collect();
    assert_eq!(unique.len(), values.len());
}

// ===========================================================================
// 8. Grammar module → full grammar output
// ===========================================================================

#[test]
fn full_grammar_module_expression_language() {
    // Simulate an expression language grammar:
    //   Expr = Literal | BinaryExpr | UnaryExpr
    //   BinaryExpr { lhs: Box<Expr>, op: Operator, rhs: Box<Expr> }
    //   UnaryExpr { op: UnaryOp, operand: Box<Expr> }

    // Enum variants (choice)
    let expr_variants: Vec<Type> = vec![
        parse_quote!(Literal),
        parse_quote!(Box<BinaryExpr>),
        parse_quote!(Box<UnaryExpr>),
    ];
    let choice: Vec<String> = expr_variants
        .iter()
        .map(|ty| process_field(ty).leaf)
        .collect();
    assert_eq!(choice, ["Literal", "BinaryExpr", "UnaryExpr"]);

    // Struct fields (sequence) for BinaryExpr
    let bin_fields: Vec<Type> = vec![
        parse_quote!(Box<Expr>),
        parse_quote!(Operator),
        parse_quote!(Box<Expr>),
    ];
    let bin_seq: Vec<FieldResult> = bin_fields.iter().map(process_field).collect();
    assert_eq!(bin_seq[0].leaf, "Expr");
    assert_eq!(bin_seq[1].leaf, "Operator");
    assert_eq!(bin_seq[2].leaf, "Expr");

    // Struct fields (sequence) for UnaryExpr
    let un_fields: Vec<Type> = vec![parse_quote!(UnaryOp), parse_quote!(Box<Expr>)];
    let un_seq: Vec<FieldResult> = un_fields.iter().map(process_field).collect();
    assert_eq!(un_seq[0].leaf, "UnaryOp");
    assert_eq!(un_seq[1].leaf, "Expr");
}

#[test]
fn full_grammar_module_statement_language() {
    // Simulate: Program { stmts: Vec<Stmt> }
    //   Stmt = LetStmt | ReturnStmt | ExprStmt
    //   LetStmt { kw: LetKw, name: Ident, init: Option<Expr> }

    let program_fields: Vec<Type> = vec![parse_quote!(Vec<Stmt>)];
    let pr = process_field(&program_fields[0]);
    assert!(pr.is_repeat);
    assert_eq!(pr.leaf, "Stmt");

    let stmt_variants: Vec<Type> = vec![
        parse_quote!(LetStmt),
        parse_quote!(ReturnStmt),
        parse_quote!(ExprStmt),
    ];
    let stmt_choice: Vec<String> = stmt_variants
        .iter()
        .map(|ty| process_field(ty).leaf)
        .collect();
    assert_eq!(stmt_choice, ["LetStmt", "ReturnStmt", "ExprStmt"]);

    let let_fields: Vec<Type> = vec![
        parse_quote!(LetKw),
        parse_quote!(Ident),
        parse_quote!(Option<Expr>),
    ];
    let lr: Vec<FieldResult> = let_fields.iter().map(process_field).collect();
    assert!(!lr[0].is_optional);
    assert!(!lr[1].is_optional);
    assert!(lr[2].is_optional);
    assert_eq!(lr[2].leaf, "Expr");
}

#[test]
fn full_grammar_module_wrapping_consistency() {
    // Verify that every leaf across the grammar gets consistently wrapped
    let all_types: Vec<Type> = vec![
        parse_quote!(Ident),
        parse_quote!(Option<Ident>),
        parse_quote!(Vec<Ident>),
        parse_quote!(Box<Ident>),
        parse_quote!(Option<Vec<Ident>>),
        parse_quote!(Box<Option<Ident>>),
    ];

    let results: Vec<FieldResult> = all_types.iter().map(process_field).collect();

    // All should resolve to "Ident" as the leaf
    for i in 0..results.len() {
        assert_eq!(results[i].leaf, "Ident", "type index {i} should have leaf Ident");
    }
}

#[test]
fn full_grammar_module_with_extras_and_prec() {
    // Grammar with whitespace extra, keyword text, and precedence annotations

    // Whitespace extra
    let ws: FieldThenParams = parse_quote!(String, pattern = "\\s+");
    assert_eq!(ws.params[0].path.to_string(), "pattern");

    // Keyword leaf
    let kw_if: NameValueExpr = parse_quote!(text = "if");
    let kw_else: NameValueExpr = parse_quote!(text = "else");

    // Precedence on operators
    let add: FieldThenParams = parse_quote!(AddOp, precedence = 1);
    let mul: FieldThenParams = parse_quote!(MulOp, precedence = 2);

    let extract_int = |nv: &NameValueExpr| -> i32 {
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Int(i), .. }) = &nv.expr {
            i.base10_parse().unwrap()
        } else {
            panic!("expected int literal")
        }
    };

    let extract_str = |nv: &NameValueExpr| -> String {
        if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.expr {
            s.value()
        } else {
            panic!("expected string literal")
        }
    };

    assert_eq!(extract_str(&kw_if), "if");
    assert_eq!(extract_str(&kw_else), "else");
    assert!(extract_int(&add.params[0]) < extract_int(&mul.params[0]));
}

// ===========================================================================
// 9. Additional pipeline edge cases
// ===========================================================================

#[test]
fn pipeline_idempotent_filter() {
    let ty: Type = parse_quote!(Box<Spanned<Leaf>>);
    let s = skip(&["Box", "Spanned"]);
    let once = filter_inner_type(&ty, &s);
    let twice = filter_inner_type(&once, &s);
    assert_eq!(ts(&once), ts(&twice));
    assert_eq!(ts(&once), "Leaf");
}

#[test]
fn pipeline_wrap_then_extract_does_not_unwrap() {
    // Once a type is wrapped in WithLeaf, extraction should not see through it
    let ty: Type = parse_quote!(Token);
    let wrapped = wrap_leaf_type(&ty, &skip(&[]));
    assert_eq!(ts(&wrapped), "adze :: WithLeaf < Token >");

    let (inner, extracted) = try_extract_inner_type(&wrapped, "Option", &skip(&[]));
    assert!(!extracted);
    assert_eq!(ts(&inner), ts(&wrapped));
}

#[test]
fn pipeline_qualified_path_types_work() {
    let ty: Type = parse_quote!(std::option::Option<std::vec::Vec<MyNode>>);
    let (after_opt, ok) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&after_opt), "std :: vec :: Vec < MyNode >");

    let (after_vec, ok) = try_extract_inner_type(&after_opt, "Vec", &skip(&[]));
    assert!(ok);
    assert_eq!(ts(&after_vec), "MyNode");
}

#[test]
fn pipeline_batch_enum_all_variants_unique_leaves() {
    let variants: Vec<Type> = vec![
        parse_quote!(Alpha),
        parse_quote!(Box<Beta>),
        parse_quote!(Spanned<Gamma>),
        parse_quote!(Box<Spanned<Delta>>),
        parse_quote!(Option<Epsilon>),
    ];

    let leaves: Vec<String> = variants.iter().map(|ty| process_field(ty).leaf).collect();
    let unique: HashSet<&str> = leaves.iter().map(|s| s.as_str()).collect();
    assert_eq!(unique.len(), leaves.len(), "all variant leaves must be unique");
}
