//! Edge-case tests for grammar expansion in adze-common.
//!
//! Covers degenerate, extreme, and unusual inputs that the shared
//! type-analysis and annotation-parsing utilities must handle correctly.

use std::collections::HashSet;

use adze_common::{
    FieldThenParams, NameValueExpr, filter_inner_type, try_extract_inner_type, wrap_leaf_type,
};
use quote::ToTokens;
use syn::{Type, parse_quote};

// ===========================================================================
// 1. Empty struct grammar — unit / zero-field types
// ===========================================================================

#[test]
fn empty_struct_unit_type_wraps() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(());
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < () >"
    );
}

#[test]
fn empty_struct_unit_type_filter_passthrough() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(());
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "()");
}

#[test]
fn empty_struct_unit_type_extract_no_match() {
    let skip: HashSet<&str> = HashSet::new();
    let ty: Type = parse_quote!(());
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(!extracted);
    assert_eq!(inner.to_token_stream().to_string(), "()");
}

#[test]
fn empty_struct_zero_fields_vec() {
    // A grammar struct with zero fields is just an empty Vec of FieldThenParams.
    let fields: Vec<FieldThenParams> = vec![];
    assert!(fields.is_empty());
}

// ===========================================================================
// 2. Deeply nested types — Box<Option<Vec<Box<T>>>>
// ===========================================================================

#[test]
fn deeply_nested_filter_strips_all_skippable() {
    let skip: HashSet<&str> = ["Box", "Option", "Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Option<Vec<Box<Leaf>>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Leaf");
}

#[test]
fn deeply_nested_extract_through_many_layers() {
    let skip: HashSet<&str> = ["Box", "Arc", "Rc"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Arc<Rc<Option<Vec<Inner>>>>>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &skip);
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "Vec < Inner >");
}

#[test]
fn deeply_nested_wrap_preserves_all_grammar_containers() {
    let skip: HashSet<&str> = ["Option", "Vec", "Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Option<Vec<Box<Token>>>>);
    let wrapped = wrap_leaf_type(&ty, &skip);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Box < Option < Vec < Box < adze :: WithLeaf < Token > > > > >"
    );
}

#[test]
fn deeply_nested_four_levels_of_box() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Box<Box<Box<Core>>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Core");
}

#[test]
fn deeply_nested_alternating_skip_and_non_skip() {
    // Only Box is skippable; Vec stops the stripping
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Vec<Box<Inner>>>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(
        filtered.to_token_stream().to_string(),
        "Vec < Box < Inner > >"
    );
}

// ===========================================================================
// 3. Grammar with only one rule — single field / single variant
// ===========================================================================

#[test]
fn single_rule_struct_one_field() {
    let fields: Vec<FieldThenParams> = vec![parse_quote!(Token)];
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].field.ty.to_token_stream().to_string(), "Token");
}

#[test]
fn single_rule_enum_one_variant() {
    let variants: Vec<FieldThenParams> = vec![parse_quote!(OnlyVariant)];
    let types: Vec<String> = variants
        .iter()
        .map(|f| f.field.ty.to_token_stream().to_string())
        .collect();
    assert_eq!(types, vec!["OnlyVariant"]);
}

#[test]
fn single_rule_with_params() {
    let parsed: FieldThenParams = parse_quote!(String, pattern = ".*");
    assert_eq!(parsed.params.len(), 1);
    let wrapped = wrap_leaf_type(&parsed.field.ty, &HashSet::new());
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < String >"
    );
}

// ===========================================================================
// 4. Grammar with many alternatives — Choice with 50+ variants
// ===========================================================================

#[test]
fn many_alternatives_50_variants() {
    let variants: Vec<FieldThenParams> = (0..50)
        .map(|i| {
            let name = syn::Ident::new(&format!("Variant{i}"), proc_macro2::Span::call_site());
            parse_quote!(#name)
        })
        .collect();

    assert_eq!(variants.len(), 50);

    // Each variant is independently parseable
    for (i, v) in variants.iter().enumerate() {
        assert_eq!(
            v.field.ty.to_token_stream().to_string(),
            format!("Variant{i}")
        );
    }
}

#[test]
fn many_alternatives_100_variants_wrapping() {
    let skip: HashSet<&str> = HashSet::new();
    let wrapped: Vec<String> = (0..100)
        .map(|i| {
            let name = syn::Ident::new(&format!("Alt{i}"), proc_macro2::Span::call_site());
            let ty: Type = parse_quote!(#name);
            wrap_leaf_type(&ty, &skip).to_token_stream().to_string()
        })
        .collect();

    assert_eq!(wrapped.len(), 100);
    assert_eq!(wrapped[0], "adze :: WithLeaf < Alt0 >");
    assert_eq!(wrapped[99], "adze :: WithLeaf < Alt99 >");
}

#[test]
fn many_alternatives_extract_optional_subset() {
    // Mix of Option<T> and bare T in 60 variants
    let types: Vec<Type> = (0..60)
        .map(|i| {
            let name = syn::Ident::new(&format!("V{i}"), proc_macro2::Span::call_site());
            if i % 3 == 0 {
                parse_quote!(Option<#name>)
            } else {
                parse_quote!(#name)
            }
        })
        .collect();

    let optional_count = types
        .iter()
        .filter(|ty| try_extract_inner_type(ty, "Option", &HashSet::new()).1)
        .count();

    assert_eq!(optional_count, 20); // 0, 3, 6, ..., 57 → 20 multiples of 3
}

// ===========================================================================
// 5. Recursive type definitions — Box<Self>-style
// ===========================================================================

#[test]
fn recursive_type_box_strips_to_self() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Expr>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Expr");
}

#[test]
fn recursive_type_option_box_extract_then_filter() {
    let extract_skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filter_skip: HashSet<&str> = ["Box"].into_iter().collect();

    let ty: Type = parse_quote!(Option<Box<Expr>>);
    let (after_option, extracted) = try_extract_inner_type(&ty, "Option", &extract_skip);
    assert!(extracted);
    // Box<Expr> extracted from Option
    assert_eq!(after_option.to_token_stream().to_string(), "Box < Expr >");

    let filtered = filter_inner_type(&after_option, &filter_skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Expr");
}

#[test]
fn recursive_type_vec_of_self() {
    // Vec<Statement> where Statement can contain Vec<Statement>
    let skip_wrap: HashSet<&str> = ["Vec"].into_iter().collect();
    let ty: Type = parse_quote!(Vec<Statement>);
    let wrapped = wrap_leaf_type(&ty, &skip_wrap);
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "Vec < adze :: WithLeaf < Statement > >"
    );
}

#[test]
fn recursive_type_mutual_references() {
    // Simulate mutual recursion: Expr contains Box<Term>, Term contains Box<Expr>
    let skip: HashSet<&str> = ["Box"].into_iter().collect();

    let expr_field: Type = parse_quote!(Box<Term>);
    let term_field: Type = parse_quote!(Box<Expr>);

    assert_eq!(
        filter_inner_type(&expr_field, &skip)
            .to_token_stream()
            .to_string(),
        "Term"
    );
    assert_eq!(
        filter_inner_type(&term_field, &skip)
            .to_token_stream()
            .to_string(),
        "Expr"
    );
}

// ===========================================================================
// 6. Grammar with inline regex patterns
// ===========================================================================

#[test]
fn regex_complex_character_class() {
    let parsed: FieldThenParams = parse_quote!(String, pattern = "[a-zA-Z_][a-zA-Z0-9_]*");
    if let syn::Expr::Lit(lit) = &parsed.params[0].expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "[a-zA-Z_][a-zA-Z0-9_]*");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn regex_alternation_and_grouping() {
    let parsed: FieldThenParams = parse_quote!(String, pattern = "(true|false|null|undefined)");
    if let syn::Expr::Lit(lit) = &parsed.params[0].expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "(true|false|null|undefined)");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn regex_quantifiers() {
    let parsed: FieldThenParams = parse_quote!(String, pattern = "\\d{1,3}(\\.\\d{1,3}){3}");
    if let syn::Expr::Lit(lit) = &parsed.params[0].expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "\\d{1,3}(\\.\\d{1,3}){3}");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn regex_empty_pattern() {
    let parsed: FieldThenParams = parse_quote!(String, pattern = "");
    if let syn::Expr::Lit(lit) = &parsed.params[0].expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn regex_unicode_category() {
    let parsed: FieldThenParams = parse_quote!(String, pattern = "\\p{L}[\\p{L}\\p{N}_]*");
    if let syn::Expr::Lit(lit) = &parsed.params[0].expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "\\p{L}[\\p{L}\\p{N}_]*");
    } else {
        panic!("Expected string literal");
    }
}

// ===========================================================================
// 7. Grammar with escape sequences in string literals
// ===========================================================================

#[test]
fn escape_newline_tab() {
    let expr: NameValueExpr = parse_quote!(text = "\n\t\r");
    if let syn::Expr::Lit(lit) = &expr.expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "\n\t\r");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn escape_null_byte() {
    let expr: NameValueExpr = parse_quote!(text = "\0");
    if let syn::Expr::Lit(lit) = &expr.expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "\0");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn escape_unicode_escape_sequence() {
    let expr: NameValueExpr = parse_quote!(text = "\u{1F600}");
    if let syn::Expr::Lit(lit) = &expr.expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "😀");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn escape_backslash_literal() {
    let expr: NameValueExpr = parse_quote!(text = "\\\\");
    if let syn::Expr::Lit(lit) = &expr.expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "\\\\");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn escape_double_quote_inside_string() {
    let expr: NameValueExpr = parse_quote!(text = "say \"hello\"");
    if let syn::Expr::Lit(lit) = &expr.expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), "say \"hello\"");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn escape_raw_string_no_escaping() {
    let expr: NameValueExpr = parse_quote!(pattern = r"\d+\.\d+");
    if let syn::Expr::Lit(lit) = &expr.expr
        && let syn::Lit::Str(s) = &lit.lit
    {
        assert_eq!(s.value(), r"\d+\.\d+");
    } else {
        panic!("Expected string literal");
    }
}

// ===========================================================================
// 8. Very long rule names
// ===========================================================================

#[test]
fn long_rule_name_128_chars() {
    let long_name = "A".repeat(128);
    let ident = syn::Ident::new(&long_name, proc_macro2::Span::call_site());
    let ty: Type = parse_quote!(#ident);

    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    let s = wrapped.to_token_stream().to_string();
    assert!(s.contains(&long_name));
    assert!(s.starts_with("adze :: WithLeaf"));
}

#[test]
fn long_rule_name_in_generic() {
    let long_name = "VeryLongTypeName".repeat(10);
    let ident = syn::Ident::new(&long_name, proc_macro2::Span::call_site());
    let ty: Type = parse_quote!(Option<#ident>);

    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), long_name);
}

#[test]
fn long_rule_name_filter() {
    let long_name = "X".repeat(256);
    let ident = syn::Ident::new(&long_name, proc_macro2::Span::call_site());
    let ty: Type = parse_quote!(Box<#ident>);

    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), long_name);
}

#[test]
fn long_rule_name_as_field_param() {
    let long_name = "SuperLongRuleName".repeat(8);
    let ident = syn::Ident::new(&long_name, proc_macro2::Span::call_site());
    let parsed: FieldThenParams = parse_quote!(#ident, precedence = 1);
    assert_eq!(parsed.field.ty.to_token_stream().to_string(), long_name);
    assert_eq!(parsed.params.len(), 1);
}

// ===========================================================================
// 9. Grammar with unicode identifiers
// ===========================================================================

#[test]
fn unicode_ident_chinese() {
    let ty: Type = parse_quote!(表达式);
    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert_eq!(
        wrapped.to_token_stream().to_string(),
        "adze :: WithLeaf < 表达式 >"
    );
}

#[test]
fn unicode_ident_japanese() {
    let ty: Type = parse_quote!(Option<文字列>);
    let (inner, extracted) = try_extract_inner_type(&ty, "Option", &HashSet::new());
    assert!(extracted);
    assert_eq!(inner.to_token_stream().to_string(), "文字列");
}

#[test]
fn unicode_ident_cyrillic() {
    let skip: HashSet<&str> = ["Box"].into_iter().collect();
    let ty: Type = parse_quote!(Box<Выражение>);
    let filtered = filter_inner_type(&ty, &skip);
    assert_eq!(filtered.to_token_stream().to_string(), "Выражение");
}

#[test]
fn unicode_ident_mixed_script() {
    // Latin + CJK in same grammar
    let skip_wrap: HashSet<&str> = ["Vec"].into_iter().collect();

    let ty_latin: Type = parse_quote!(Vec<Expression>);
    let ty_cjk: Type = parse_quote!(Vec<式>);

    let w1 = wrap_leaf_type(&ty_latin, &skip_wrap)
        .to_token_stream()
        .to_string();
    let w2 = wrap_leaf_type(&ty_cjk, &skip_wrap)
        .to_token_stream()
        .to_string();

    assert_eq!(w1, "Vec < adze :: WithLeaf < Expression > >");
    assert_eq!(w2, "Vec < adze :: WithLeaf < 式 > >");
}

#[test]
fn unicode_ident_emoji_like_name() {
    // Rust identifiers can't be emoji, but accented Latin is fine
    let ty: Type = parse_quote!(Résumé);
    let wrapped = wrap_leaf_type(&ty, &HashSet::new());
    assert!(wrapped.to_token_stream().to_string().contains("Résumé"));
}

// ===========================================================================
// 10. Multiple grammars in same module — independent processing
// ===========================================================================

#[test]
fn multiple_grammars_independent_skip_sets() {
    // Grammar A uses Box as transparent, Grammar B does not
    let skip_a: HashSet<&str> = ["Box"].into_iter().collect();
    let skip_b: HashSet<&str> = HashSet::new();

    let ty: Type = parse_quote!(Box<Node>);

    let filtered_a = filter_inner_type(&ty, &skip_a);
    let filtered_b = filter_inner_type(&ty, &skip_b);

    assert_eq!(filtered_a.to_token_stream().to_string(), "Node");
    assert_eq!(filtered_b.to_token_stream().to_string(), "Box < Node >");
}

#[test]
fn multiple_grammars_independent_wrap_sets() {
    // Grammar A preserves Vec, Grammar B preserves Option
    let wrap_a: HashSet<&str> = ["Vec"].into_iter().collect();
    let wrap_b: HashSet<&str> = ["Option"].into_iter().collect();

    let ty_vec: Type = parse_quote!(Vec<Item>);
    let ty_opt: Type = parse_quote!(Option<Item>);

    // Grammar A: Vec preserved, Option wrapped entirely
    assert_eq!(
        wrap_leaf_type(&ty_vec, &wrap_a)
            .to_token_stream()
            .to_string(),
        "Vec < adze :: WithLeaf < Item > >"
    );
    assert_eq!(
        wrap_leaf_type(&ty_opt, &wrap_a)
            .to_token_stream()
            .to_string(),
        "adze :: WithLeaf < Option < Item > >"
    );

    // Grammar B: Option preserved, Vec wrapped entirely
    assert_eq!(
        wrap_leaf_type(&ty_vec, &wrap_b)
            .to_token_stream()
            .to_string(),
        "adze :: WithLeaf < Vec < Item > >"
    );
    assert_eq!(
        wrap_leaf_type(&ty_opt, &wrap_b)
            .to_token_stream()
            .to_string(),
        "Option < adze :: WithLeaf < Item > >"
    );
}

#[test]
fn multiple_grammars_same_type_different_annotations() {
    // Two grammars parse the same field type but with different annotations
    let f_grammar_a: FieldThenParams = parse_quote!(String, pattern = "[a-z]+");
    let f_grammar_b: FieldThenParams = parse_quote!(
        String,
        pattern = "\\d+",
        transform = |v: String| v.parse::<i32>().unwrap()
    );

    assert_eq!(f_grammar_a.params.len(), 1);
    assert_eq!(f_grammar_b.params.len(), 2);

    assert_eq!(f_grammar_a.params[0].path.to_string(), "pattern");
    assert_eq!(f_grammar_b.params[0].path.to_string(), "pattern");
    assert_eq!(f_grammar_b.params[1].path.to_string(), "transform");
}

#[test]
fn multiple_grammars_no_cross_contamination() {
    // Simulate processing two grammars sequentially
    let grammar_a_fields: Vec<FieldThenParams> =
        vec![parse_quote!(Keyword), parse_quote!(Option<Identifier>)];
    let grammar_b_fields: Vec<FieldThenParams> =
        vec![parse_quote!(Vec<Statement>), parse_quote!(Box<Expression>)];

    let skip_a: HashSet<&str> = HashSet::new();
    let skip_b: HashSet<&str> = ["Box"].into_iter().collect();

    // Process grammar A
    let a_types: Vec<String> = grammar_a_fields
        .iter()
        .map(|f| {
            filter_inner_type(&f.field.ty, &skip_a)
                .to_token_stream()
                .to_string()
        })
        .collect();

    // Process grammar B
    let b_types: Vec<String> = grammar_b_fields
        .iter()
        .map(|f| {
            filter_inner_type(&f.field.ty, &skip_b)
                .to_token_stream()
                .to_string()
        })
        .collect();

    // Grammar A: no Box skipping
    assert_eq!(a_types, vec!["Keyword", "Option < Identifier >"]);
    // Grammar B: Box stripped
    assert_eq!(b_types, vec!["Vec < Statement >", "Expression"]);
}

#[test]
fn multiple_grammars_shared_type_name_different_nesting() {
    let skip: HashSet<&str> = ["Box", "Arc"].into_iter().collect();

    // Grammar A has Node at one nesting level
    let ty_a: Type = parse_quote!(Box<Node>);
    // Grammar B has Node at another nesting level
    let ty_b: Type = parse_quote!(Arc<Box<Node>>);

    let fa = filter_inner_type(&ty_a, &skip);
    let fb = filter_inner_type(&ty_b, &skip);

    // Both resolve to the same leaf type
    assert_eq!(fa.to_token_stream().to_string(), "Node");
    assert_eq!(fb.to_token_stream().to_string(), "Node");
}
