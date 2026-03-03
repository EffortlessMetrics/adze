#![allow(clippy::needless_range_loop)]

//! Integration tests for adze-common exercising multi-function workflows.
//!
//! These tests simulate realistic grammar-like scenarios that chain
//! `try_extract_inner_type`, `filter_inner_type`, and `wrap_leaf_type`
//! in patterns typical of adze grammar processing pipelines.

use std::collections::HashSet;

use adze_common::{FieldThenParams, filter_inner_type, try_extract_inner_type, wrap_leaf_type};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skip<'a>(names: &'a [&'a str]) -> HashSet<&'a str> {
    names.iter().copied().collect()
}

fn ty_str(ty: &Type) -> String {
    ty.to_token_stream().to_string()
}

/// Simulates the adze grammar field processing pipeline:
/// 1. Try to extract Option (marks field as optional)
/// 2. Try to extract Vec (marks field as repeated)
/// 3. Filter away smart pointers (Box, Arc)
/// 4. Wrap remaining leaf type
fn grammar_field_pipeline(ty: &Type) -> (Type, bool, bool) {
    let ptr_skip = skip(&["Box", "Arc"]);
    let (after_opt, is_optional) = try_extract_inner_type(ty, "Option", &ptr_skip);
    let (after_vec, is_repeated) = try_extract_inner_type(&after_opt, "Vec", &ptr_skip);
    let filtered = filter_inner_type(&after_vec, &ptr_skip);
    let wrapped = wrap_leaf_type(&filtered, &skip(&["Vec", "Option"]));
    (wrapped, is_optional, is_repeated)
}

// ===========================================================================
// 1. Simulated grammar extraction workflow
// ===========================================================================

#[test]
fn grammar_extraction_simple_leaf_field() {
    // A grammar field like `name: String` — plain leaf, not optional, not repeated
    let ty: Type = parse_quote!(String);
    let (result, is_opt, is_rep) = grammar_field_pipeline(&ty);
    assert!(!is_opt);
    assert!(!is_rep);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < String >");
}

#[test]
fn grammar_extraction_optional_field() {
    // A grammar field like `semicolon: Option<Token>` — optional leaf
    let ty: Type = parse_quote!(Option<Token>);
    let (result, is_opt, is_rep) = grammar_field_pipeline(&ty);
    assert!(is_opt);
    assert!(!is_rep);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Token >");
}

#[test]
fn grammar_extraction_repeated_field() {
    // A grammar field like `statements: Vec<Statement>` — repeated
    let ty: Type = parse_quote!(Vec<Statement>);
    let (result, is_opt, is_rep) = grammar_field_pipeline(&ty);
    assert!(!is_opt);
    assert!(is_rep);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Statement >");
}

#[test]
fn grammar_extraction_optional_repeated_field() {
    // A grammar field like `args: Option<Vec<Expr>>` — optional + repeated
    let ty: Type = parse_quote!(Option<Vec<Expr>>);
    let (result, is_opt, is_rep) = grammar_field_pipeline(&ty);
    assert!(is_opt);
    assert!(is_rep);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Expr >");
}

#[test]
fn grammar_extraction_boxed_optional_field() {
    // A grammar field like `body: Box<Option<Block>>` — Box skipped, then optional extracted
    let ty: Type = parse_quote!(Box<Option<Block>>);
    let (result, is_opt, is_rep) = grammar_field_pipeline(&ty);
    assert!(is_opt);
    assert!(!is_rep);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Block >");
}

// ===========================================================================
// 2. Type processing pipeline (extract → filter → wrap)
// ===========================================================================

#[test]
fn pipeline_extract_filter_wrap_nested_pointers() {
    // Arc<Box<Vec<Identifier>>> → extract Vec → filter Arc/Box → wrap
    let ty: Type = parse_quote!(Arc<Box<Vec<Identifier>>>);
    let ptr_skip = skip(&["Arc", "Box"]);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &ptr_skip);
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Identifier");
    let filtered = filter_inner_type(&inner, &ptr_skip);
    assert_eq!(ty_str(&filtered), "Identifier");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Identifier >");
}

#[test]
fn pipeline_no_extraction_still_filters_and_wraps() {
    // Box<Literal> — no Option/Vec to extract, but Box is filtered
    let ty: Type = parse_quote!(Box<Literal>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(!extracted);
    let filtered = filter_inner_type(&inner, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "Literal");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Literal >");
}

#[test]
fn pipeline_extract_option_from_deeply_wrapped() {
    // Box<Arc<Option<Vec<Token>>>> — skip Box+Arc, extract Option, then extract Vec
    let ty: Type = parse_quote!(Box<Arc<Option<Vec<Token>>>>);
    let ptr_skip = skip(&["Box", "Arc"]);
    let (after_opt, opt_found) = try_extract_inner_type(&ty, "Option", &ptr_skip);
    assert!(opt_found);
    assert_eq!(ty_str(&after_opt), "Vec < Token >");
    let (after_vec, vec_found) = try_extract_inner_type(&after_opt, "Vec", &skip(&[]));
    assert!(vec_found);
    assert_eq!(ty_str(&after_vec), "Token");
    let wrapped = wrap_leaf_type(&after_vec, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Token >");
}

// ===========================================================================
// 3. Multiple field types processed in sequence
// ===========================================================================

#[test]
fn batch_process_arithmetic_grammar_fields() {
    // Simulates processing fields of an arithmetic expression grammar struct
    let field_types: Vec<Type> = vec![
        parse_quote!(Expr),           // left operand
        parse_quote!(String),         // operator token
        parse_quote!(Expr),           // right operand
        parse_quote!(Option<String>), // optional trailing comma
    ];
    let expected_optional = [false, false, false, true];
    let expected_results = [
        "adze :: WithLeaf < Expr >",
        "adze :: WithLeaf < String >",
        "adze :: WithLeaf < Expr >",
        "adze :: WithLeaf < String >",
    ];

    for i in 0..field_types.len() {
        let (result, is_opt, _) = grammar_field_pipeline(&field_types[i]);
        assert_eq!(is_opt, expected_optional[i], "field {i} optional mismatch");
        assert_eq!(ty_str(&result), expected_results[i], "field {i} type mismatch");
    }
}

#[test]
fn batch_process_function_def_fields() {
    // Simulates: fn name(params) -> return_type { body }
    let field_types: Vec<Type> = vec![
        parse_quote!(String),             // fn keyword
        parse_quote!(Identifier),         // function name
        parse_quote!(Vec<Parameter>),     // parameter list
        parse_quote!(Option<ReturnType>), // optional return type
        parse_quote!(Vec<Statement>),     // function body
    ];
    let expected_opt = [false, false, false, true, false];
    let expected_rep = [false, false, true, false, true];

    for i in 0..field_types.len() {
        let (_, is_opt, is_rep) = grammar_field_pipeline(&field_types[i]);
        assert_eq!(is_opt, expected_opt[i], "field {i} optional");
        assert_eq!(is_rep, expected_rep[i], "field {i} repeated");
    }
}

#[test]
fn batch_process_mixed_wrapper_fields() {
    // Fields with various wrapper combinations
    let field_types: Vec<Type> = vec![
        parse_quote!(Box<Node>),
        parse_quote!(Arc<Box<Node>>),
        parse_quote!(Box<Option<Node>>),
        parse_quote!(Arc<Vec<Node>>),
    ];
    let expected_opt = [false, false, true, false];
    let expected_rep = [false, false, false, true];

    for i in 0..field_types.len() {
        let (result, is_opt, is_rep) = grammar_field_pipeline(&field_types[i]);
        assert_eq!(is_opt, expected_opt[i], "field {i} optional");
        assert_eq!(is_rep, expected_rep[i], "field {i} repeated");
        assert_eq!(ty_str(&result), "adze :: WithLeaf < Node >", "field {i} result");
    }
}

// ===========================================================================
// 4. Grammar struct field analysis
// ===========================================================================

#[test]
fn grammar_struct_if_statement_analysis() {
    // if (condition) { consequent } else { alternate }
    struct FieldInfo {
        ty: Type,
        expect_opt: bool,
        expect_rep: bool,
        expect_leaf: &'static str,
    }
    let fields = vec![
        FieldInfo { ty: parse_quote!(Expr), expect_opt: false, expect_rep: false, expect_leaf: "Expr" },
        FieldInfo { ty: parse_quote!(Vec<Statement>), expect_opt: false, expect_rep: true, expect_leaf: "Statement" },
        FieldInfo { ty: parse_quote!(Option<Vec<Statement>>), expect_opt: true, expect_rep: true, expect_leaf: "Statement" },
    ];

    for (i, f) in fields.iter().enumerate() {
        let (result, is_opt, is_rep) = grammar_field_pipeline(&f.ty);
        assert_eq!(is_opt, f.expect_opt, "field {i} optional");
        assert_eq!(is_rep, f.expect_rep, "field {i} repeated");
        assert_eq!(
            ty_str(&result),
            format!("adze :: WithLeaf < {} >", f.expect_leaf),
            "field {i} leaf"
        );
    }
}

#[test]
fn grammar_struct_field_with_params_and_type_processing() {
    // Parse a FieldThenParams then process its type through the pipeline
    let parsed: FieldThenParams = parse_quote!(Vec<Statement>, precedence = 1);
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].path.to_string(), "precedence");

    let (result, is_opt, is_rep) = grammar_field_pipeline(&parsed.field.ty);
    assert!(!is_opt);
    assert!(is_rep);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Statement >");
}

#[test]
fn grammar_struct_optional_field_with_pattern_param() {
    let parsed: FieldThenParams = parse_quote!(Option<String>, pattern = "[0-9]+");
    assert_eq!(parsed.params[0].path.to_string(), "pattern");

    let (result, is_opt, _) = grammar_field_pipeline(&parsed.field.ty);
    assert!(is_opt);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < String >");
}

// ===========================================================================
// 5. Enum variant type processing
// ===========================================================================

#[test]
fn enum_variant_types_expression_grammar() {
    // Simulates processing an Expr enum: Literal(i64), Binary(Box<Expr>, Op, Box<Expr>), Call(Ident, Vec<Expr>)
    // Each variant's contained types go through filter+wrap
    let ptr_skip = skip(&["Box"]);
    let wrap_skip = skip(&["Vec", "Option"]);

    // Literal variant — plain leaf
    let ty: Type = parse_quote!(i64);
    let filtered = filter_inner_type(&ty, &ptr_skip);
    let wrapped = wrap_leaf_type(&filtered, &wrap_skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < i64 >");

    // Binary variant — Box<Expr> fields
    let ty: Type = parse_quote!(Box<Expr>);
    let filtered = filter_inner_type(&ty, &ptr_skip);
    assert_eq!(ty_str(&filtered), "Expr");
    let wrapped = wrap_leaf_type(&filtered, &wrap_skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Expr >");

    // Call variant — Vec<Expr> arguments
    let ty: Type = parse_quote!(Vec<Expr>);
    let filtered = filter_inner_type(&ty, &ptr_skip);
    assert_eq!(ty_str(&filtered), "Vec < Expr >");
    let wrapped = wrap_leaf_type(&filtered, &wrap_skip);
    assert_eq!(ty_str(&wrapped), "Vec < adze :: WithLeaf < Expr > >");
}

#[test]
fn enum_variant_optional_payload() {
    // Variant like Return(Option<Expr>) — extract Option, wrap leaf
    let ty: Type = parse_quote!(Option<Expr>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&[]));
    assert!(extracted);
    let wrapped = wrap_leaf_type(&inner, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Expr >");
}

#[test]
fn enum_variant_boxed_recursive() {
    // Variant like Unary(Op, Box<Expr>) — filter Box, wrap
    let variant_fields: Vec<Type> = vec![parse_quote!(Op), parse_quote!(Box<Expr>)];
    let ptr_skip = skip(&["Box"]);

    let results: Vec<String> = variant_fields
        .iter()
        .map(|ty| {
            let filtered = filter_inner_type(ty, &ptr_skip);
            ty_str(&wrap_leaf_type(&filtered, &skip(&[])))
        })
        .collect();

    assert_eq!(results[0], "adze :: WithLeaf < Op >");
    assert_eq!(results[1], "adze :: WithLeaf < Expr >");
}

#[test]
fn enum_variant_all_patterns() {
    // Process multiple variant shapes through the full pipeline
    let variants: Vec<Type> = vec![
        parse_quote!(i64),                  // Literal
        parse_quote!(String),               // StringLit
        parse_quote!(bool),                 // BoolLit
        parse_quote!(Box<Expr>),            // Grouped
        parse_quote!(Vec<Expr>),            // Array
        parse_quote!(Option<Expr>),         // Maybe
        parse_quote!(Option<Vec<Expr>>),    // OptionalList
    ];

    let (_, opt, rep) = grammar_field_pipeline(&variants[0]);
    assert!(!opt && !rep);

    let (_, opt, rep) = grammar_field_pipeline(&variants[3]);
    assert!(!opt && !rep); // Box is just filtered away

    let (_, opt, rep) = grammar_field_pipeline(&variants[4]);
    assert!(!opt && rep);

    let (_, opt, rep) = grammar_field_pipeline(&variants[5]);
    assert!(opt && !rep);

    let (_, opt, rep) = grammar_field_pipeline(&variants[6]);
    assert!(opt && rep);
}

// ===========================================================================
// 6. Real-world type patterns from adze grammars
// ===========================================================================

#[test]
fn real_world_python_function_def() {
    // Python grammar: def name(params) -> annotation: body
    let decorators: Type = parse_quote!(Vec<Decorator>);
    let name: Type = parse_quote!(Identifier);
    let params: Type = parse_quote!(Vec<Parameter>);
    let return_annotation: Type = parse_quote!(Option<Expression>);
    let body: Type = parse_quote!(Vec<Statement>);

    let (_, _, rep) = grammar_field_pipeline(&decorators);
    assert!(rep);

    let (result, opt, rep) = grammar_field_pipeline(&name);
    assert!(!opt && !rep);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Identifier >");

    let (_, _, rep) = grammar_field_pipeline(&params);
    assert!(rep);

    let (_, opt, _) = grammar_field_pipeline(&return_annotation);
    assert!(opt);

    let (_, _, rep) = grammar_field_pipeline(&body);
    assert!(rep);
}

#[test]
fn real_world_javascript_arrow_function() {
    // JS grammar: (params) => body | expression
    let params: Type = parse_quote!(Vec<Pattern>);
    let body: Type = parse_quote!(Box<Statement>);
    let expression: Type = parse_quote!(Box<Expression>);

    let ptr_skip = skip(&["Box", "Arc"]);
    let wrap_skip = skip(&["Vec", "Option"]);

    // params: repeated
    let (inner, _) = try_extract_inner_type(&params, "Vec", &ptr_skip);
    let wrapped = wrap_leaf_type(&inner, &wrap_skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Pattern >");

    // body: Box filtered away
    let filtered = filter_inner_type(&body, &ptr_skip);
    let wrapped = wrap_leaf_type(&filtered, &wrap_skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Statement >");

    // expression: same treatment
    let filtered = filter_inner_type(&expression, &ptr_skip);
    let wrapped = wrap_leaf_type(&filtered, &wrap_skip);
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Expression >");
}

#[test]
fn real_world_rust_match_arm() {
    // Rust match arm: pattern => body, with optional guard
    let pattern: Type = parse_quote!(Pattern);
    let guard: Type = parse_quote!(Option<Box<Expr>>);
    let body: Type = parse_quote!(Box<Expr>);

    let ptr_skip = skip(&["Box", "Arc"]);

    // pattern — plain leaf
    let (result, _, _) = grammar_field_pipeline(&pattern);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Pattern >");

    // guard — Option extracted, Box filtered
    let (after_opt, opt) = try_extract_inner_type(&guard, "Option", &ptr_skip);
    assert!(opt);
    let filtered = filter_inner_type(&after_opt, &ptr_skip);
    assert_eq!(ty_str(&filtered), "Expr");

    // body — Box filtered
    let filtered = filter_inner_type(&body, &ptr_skip);
    assert_eq!(ty_str(&filtered), "Expr");
}

#[test]
fn real_world_import_statement() {
    // import { specifiers } from "source"
    let specifiers: Type = parse_quote!(Vec<ImportSpecifier>);
    let source: Type = parse_quote!(StringLiteral);
    let default_import: Type = parse_quote!(Option<Identifier>);

    let (_, _, rep) = grammar_field_pipeline(&specifiers);
    assert!(rep);
    let (result, _, _) = grammar_field_pipeline(&source);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < StringLiteral >");
    let (_, opt, _) = grammar_field_pipeline(&default_import);
    assert!(opt);
}

#[test]
fn real_world_class_definition_fields() {
    // class Name extends Base { members }
    let name: Type = parse_quote!(Identifier);
    let superclass: Type = parse_quote!(Option<Identifier>);
    let members: Type = parse_quote!(Vec<ClassMember>);
    let decorators: Type = parse_quote!(Option<Vec<Decorator>>);

    let results: Vec<(bool, bool)> = [&name, &superclass, &members, &decorators]
        .iter()
        .map(|ty| {
            let (_, opt, rep) = grammar_field_pipeline(ty);
            (opt, rep)
        })
        .collect();

    assert_eq!(results[0], (false, false)); // name
    assert_eq!(results[1], (true, false));  // superclass
    assert_eq!(results[2], (false, true));  // members
    assert_eq!(results[3], (true, true));   // decorators
}

#[test]
fn real_world_binary_expression_recursive() {
    // BinaryExpr { left: Box<Expr>, op: BinOp, right: Box<Expr> }
    let fields: Vec<Type> = vec![
        parse_quote!(Box<Expr>),
        parse_quote!(BinOp),
        parse_quote!(Box<Expr>),
    ];

    let ptr_skip = skip(&["Box"]);
    for (i, ty) in fields.iter().enumerate() {
        let filtered = filter_inner_type(ty, &ptr_skip);
        let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
        match i {
            0 | 2 => assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Expr >"),
            1 => assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < BinOp >"),
            _ => unreachable!(),
        }
    }
}

#[test]
fn real_world_try_catch_statement() {
    // try { body } catch(param) { handler } finally { finalizer }
    let body: Type = parse_quote!(Vec<Statement>);
    let catch_param: Type = parse_quote!(Option<Pattern>);
    let handler: Type = parse_quote!(Option<Vec<Statement>>);
    let finalizer: Type = parse_quote!(Option<Vec<Statement>>);

    let (_, opt, rep) = grammar_field_pipeline(&body);
    assert!(!opt && rep);

    let (_, opt, rep) = grammar_field_pipeline(&catch_param);
    assert!(opt && !rep);

    let (_, opt, rep) = grammar_field_pipeline(&handler);
    assert!(opt && rep);

    let (_, opt, rep) = grammar_field_pipeline(&finalizer);
    assert!(opt && rep);
}

// ===========================================================================
// 7. Pipeline composition edge cases
// ===========================================================================

#[test]
fn pipeline_idempotent_wrap_after_full_processing() {
    // Wrapping an already-wrapped type produces double wrapping
    let ty: Type = parse_quote!(String);
    let (first, _, _) = grammar_field_pipeline(&ty);
    // first = adze::WithLeaf<String>
    // Running wrap again on this (not in skip) wraps again
    let double = wrap_leaf_type(&first, &skip(&[]));
    assert_eq!(
        ty_str(&double),
        "adze :: WithLeaf < adze :: WithLeaf < String > >"
    );
}

#[test]
fn pipeline_filter_preserves_non_skip_generics() {
    // HashMap<K, V> is not in skip set — filter is identity, wrap wraps whole thing
    let ty: Type = parse_quote!(HashMap<String, Vec<i32>>);
    let filtered = filter_inner_type(&ty, &skip(&["Box"]));
    assert_eq!(ty_str(&filtered), "HashMap < String , Vec < i32 > >");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(
        ty_str(&wrapped),
        "adze :: WithLeaf < HashMap < String , Vec < i32 > > >"
    );
}

#[test]
fn pipeline_extract_then_filter_different_skip_sets() {
    // Extract Option with Box skip, then filter with Arc skip
    let ty: Type = parse_quote!(Box<Option<Arc<Leaf>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip(&["Box"]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Arc < Leaf >");
    let filtered = filter_inner_type(&inner, &skip(&["Arc"]));
    assert_eq!(ty_str(&filtered), "Leaf");
    let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Leaf >");
}

#[test]
fn pipeline_process_vec_of_options() {
    // Vec<Option<Token>> — extract Vec, then check inner is Option<Token>
    let ty: Type = parse_quote!(Vec<Option<Token>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Vec", &skip(&[]));
    assert!(extracted);
    assert_eq!(ty_str(&inner), "Option < Token >");
    // Now extract Option from the inner
    let (leaf, opt_extracted) = try_extract_inner_type(&inner, "Option", &skip(&[]));
    assert!(opt_extracted);
    assert_eq!(ty_str(&leaf), "Token");
    let wrapped = wrap_leaf_type(&leaf, &skip(&[]));
    assert_eq!(ty_str(&wrapped), "adze :: WithLeaf < Token >");
}

#[test]
fn pipeline_fieldthenparams_full_workflow() {
    // Parse attribute, extract type info, process through pipeline
    let parsed: FieldThenParams =
        parse_quote!(Option<Vec<Expr>>, precedence = 3, associativity = "left");

    assert_eq!(parsed.params.len(), 2);

    let (result, is_opt, is_rep) = grammar_field_pipeline(&parsed.field.ty);
    assert!(is_opt);
    assert!(is_rep);
    assert_eq!(ty_str(&result), "adze :: WithLeaf < Expr >");

    // Verify param values
    assert_eq!(parsed.params[0].path.to_string(), "precedence");
    assert_eq!(parsed.params[1].path.to_string(), "associativity");
}

#[test]
fn pipeline_sequential_fields_consistent_output() {
    // All leaf types regardless of wrapper should produce the same wrapped output
    let variations: Vec<Type> = vec![
        parse_quote!(Token),
        parse_quote!(Box<Token>),
        parse_quote!(Arc<Token>),
        parse_quote!(Box<Arc<Token>>),
    ];

    let ptr_skip = skip(&["Box", "Arc"]);
    for (i, ty) in variations.iter().enumerate() {
        let filtered = filter_inner_type(ty, &ptr_skip);
        let wrapped = wrap_leaf_type(&filtered, &skip(&[]));
        assert_eq!(
            ty_str(&wrapped),
            "adze :: WithLeaf < Token >",
            "variation {i} should produce same leaf"
        );
    }
}
