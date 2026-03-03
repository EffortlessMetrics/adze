#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for `StaticLanguageGenerator` public API in adze-tablegen.
//!
//! Covers: construction, language code generation, node types generation,
//! generated code structure, grammar sizes, feature combinations, and output determinism.

use adze_glr_core::ParseTable;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar};
use adze_tablegen::StaticLanguageGenerator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_grammar_and_table() -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new("minimal")
        .token("number", r"\d+")
        .rule("expr", vec!["number"])
        .start("expr")
        .build();
    (grammar, ParseTable::default())
}

fn arithmetic_grammar_and_table() -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new("arithmetic")
        .token("number", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["number"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .start("expr")
        .build();
    (grammar, ParseTable::default())
}

fn grammar_with_externals() -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new("ext_lang")
        .token("id", r"[a-z]+")
        .token("colon", ":")
        .external("indent")
        .external("dedent")
        .external("newline")
        .rule("block", vec!["id", "colon", "indent", "id", "dedent"])
        .start("block")
        .build();
    (grammar, ParseTable::default())
}

fn grammar_with_fields() -> (Grammar, ParseTable) {
    let mut grammar = GrammarBuilder::new("field_lang")
        .token("number", r"\d+")
        .token("plus", "+")
        .rule("binary", vec!["number", "plus", "number"])
        .start("binary")
        .build();
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "operator".to_string());
    grammar.fields.insert(FieldId(2), "right".to_string());
    (grammar, ParseTable::default())
}

fn grammar_with_extras() -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new("extras_lang")
        .token("id", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .extra("ws")
        .rule("start", vec!["id"])
        .start("start")
        .build();
    (grammar, ParseTable::default())
}

fn medium_grammar_and_table() -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new("medium")
        .token("number", r"\d+")
        .token("id", r"[a-z]+")
        .token("plus", "+")
        .token("minus", "-")
        .token("star", "*")
        .token("slash", "SLASH")
        .token("lparen", "(")
        .token("rparen", ")")
        .token("eq", "=")
        .token("semi", ";")
        .rule("program", vec!["statement"])
        .rule("program", vec!["program", "statement"])
        .rule("statement", vec!["id", "eq", "expr", "semi"])
        .rule("expr", vec!["number"])
        .rule("expr", vec!["id"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "minus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .rule("expr", vec!["expr", "slash", "expr"])
        .rule("expr", vec!["lparen", "expr", "rparen"])
        .start("program")
        .build();
    (grammar, ParseTable::default())
}

fn large_grammar_and_table() -> (Grammar, ParseTable) {
    let mut builder = GrammarBuilder::new("large");
    // 20 tokens
    for i in 0..20 {
        builder = builder.token(&format!("tok_{i}"), &format!("t{i}"));
    }
    // Chain of rules referencing tokens
    for i in 0..15 {
        let tok_name = format!("tok_{i}");
        let rule_name = format!("rule_{i}");
        builder = builder.rule(&rule_name, vec![Box::leak(tok_name.into_boxed_str())]);
    }
    // A top-level start rule referencing several sub-rules
    builder = builder.rule("top", vec!["rule_0"]);
    builder = builder.rule("top", vec!["rule_1"]);
    builder = builder.rule("top", vec!["rule_2"]);
    builder = builder.start("top");
    let grammar = builder.build();
    (grammar, ParseTable::default())
}

// ===========================================================================
// 1. Generator construction tests
// ===========================================================================

#[test]
fn construction_preserves_grammar_name() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    assert_eq!(generator.grammar.name, "minimal");
}

#[test]
fn construction_defaults_start_can_be_empty_false() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    assert!(!generator.start_can_be_empty);
}

#[test]
fn construction_defaults_compressed_tables_none() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    assert!(generator.compressed_tables.is_none());
}

#[test]
fn set_start_can_be_empty_toggles() {
    let (grammar, table) = minimal_grammar_and_table();
    let mut generator = StaticLanguageGenerator::new(grammar, table);

    generator.set_start_can_be_empty(true);
    assert!(generator.start_can_be_empty);

    generator.set_start_can_be_empty(false);
    assert!(!generator.start_can_be_empty);
}

#[test]
fn construction_preserves_parse_table_state_count() {
    let (grammar, table) = minimal_grammar_and_table();
    let expected = table.state_count;
    let generator = StaticLanguageGenerator::new(grammar, table);
    assert_eq!(generator.parse_table.state_count, expected);
}

// ===========================================================================
// 2. Language code generation tests
// ===========================================================================

#[test]
fn generate_language_code_returns_nonempty_token_stream() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    assert!(!code.is_empty(), "generated token stream must not be empty");
}

#[test]
fn generate_language_code_contains_language_function() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code_str = generator.generate_language_code().to_string();
    assert!(
        code_str.contains("language"),
        "code should declare a `language` function"
    );
}

#[test]
fn generate_language_code_embeds_grammar_name() {
    let grammar = GrammarBuilder::new("my_cool_lang")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
    let code_str = generator.generate_language_code().to_string();
    assert!(
        code_str.contains("my_cool_lang"),
        "generated code should embed grammar name"
    );
}

#[test]
fn generate_language_code_contains_version_constant() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code_str = generator.generate_language_code().to_string();
    assert!(
        code_str.contains("LANGUAGE_VERSION"),
        "code should reference LANGUAGE_VERSION"
    );
}

#[test]
fn generate_language_code_contains_symbol_names() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code_str = generator.generate_language_code().to_string();
    assert!(
        code_str.contains("SYMBOL_NAMES"),
        "code should include SYMBOL_NAMES"
    );
}

// ===========================================================================
// 3. Node types generation tests
// ===========================================================================

#[test]
fn generate_node_types_produces_valid_json() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let json_str = generator.generate_node_types();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("node types must be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn generate_node_types_array_has_entries() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let json_str = generator.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = parsed.as_array().unwrap();
    assert!(!arr.is_empty(), "node types should have at least one entry");
}

#[test]
fn generate_node_types_entries_have_type_field() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let json_str = generator.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "every node type entry must have a \"type\" field"
        );
    }
}

#[test]
fn generate_node_types_entries_have_named_field() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let json_str = generator.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(
            entry.get("named").is_some(),
            "every node type entry must have a \"named\" field"
        );
    }
}

#[test]
fn generate_node_types_excludes_hidden_tokens() {
    let grammar = GrammarBuilder::new("hidden")
        .token("a", "a")
        .rule("_internal", vec!["a"])
        .rule("visible", vec!["_internal"])
        .start("visible")
        .build();
    let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
    let json_str = generator.generate_node_types();
    assert!(
        !json_str.contains("\"_internal\""),
        "hidden rules (prefixed _) must not appear in node types"
    );
}

#[test]
fn generate_node_types_includes_external_tokens() {
    let (grammar, table) = grammar_with_externals();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let json_str = generator.generate_node_types();
    assert!(json_str.contains("\"indent\""));
    assert!(json_str.contains("\"dedent\""));
    assert!(json_str.contains("\"newline\""));
}

#[test]
fn generate_node_types_excludes_hidden_externals() {
    let grammar = GrammarBuilder::new("hid_ext")
        .token("a", "a")
        .external("_hidden_ext")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
    let json_str = generator.generate_node_types();
    assert!(
        !json_str.contains("\"_hidden_ext\""),
        "hidden externals should not appear in node types"
    );
}

// ===========================================================================
// 4. Generated code structure tests
// ===========================================================================

#[test]
fn generated_code_is_valid_rust_syntax() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    // syn::parse2 will fail on invalid syntax
    let result: Result<syn::File, _> = syn::parse2(code);
    assert!(
        result.is_ok(),
        "generated code must be syntactically valid Rust: {:?}",
        result.err()
    );
}

#[test]
fn generated_code_contains_static_arrays() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code_str = generator.generate_language_code().to_string();
    // Language generators typically produce static arrays for symbol names, metadata, etc.
    assert!(
        code_str.contains("static") || code_str.contains("const"),
        "generated code should contain static/const declarations"
    );
}

#[test]
fn different_grammar_names_produce_different_function_names() {
    let g1 = GrammarBuilder::new("alpha")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("beta")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();

    let c1 = StaticLanguageGenerator::new(g1, ParseTable::default())
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, ParseTable::default())
        .generate_language_code()
        .to_string();

    assert!(c1.contains("alpha"));
    assert!(c2.contains("beta"));
    assert_ne!(c1, c2, "different grammar names must yield different code");
}

// ===========================================================================
// 5. Different grammar sizes
// ===========================================================================

#[test]
fn small_grammar_generates_successfully() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    assert!(!code.is_empty());
    let json = generator.generate_node_types();
    assert!(!json.is_empty());
}

#[test]
fn medium_grammar_generates_successfully() {
    let (grammar, table) = medium_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code_str = generator.generate_language_code().to_string();
    assert!(code_str.contains("medium"));
    let json_str = generator.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.as_array().unwrap().len() > 1);
}

#[test]
fn large_grammar_generates_successfully() {
    let (grammar, table) = large_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    assert!(!code.is_empty());
    let json_str = generator.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = parsed.as_array().unwrap();
    // 20 tokens + 15 rules + start rule => many entries
    assert!(arr.len() >= 10, "large grammar should produce many node types");
}

#[test]
fn large_grammar_code_size_exceeds_small() {
    let (sg, st) = minimal_grammar_and_table();
    let (lg, lt) = large_grammar_and_table();

    let small_len = StaticLanguageGenerator::new(sg, st)
        .generate_language_code()
        .to_string()
        .len();
    let large_len = StaticLanguageGenerator::new(lg, lt)
        .generate_language_code()
        .to_string()
        .len();

    assert!(
        large_len > small_len,
        "large grammar ({large_len}) should produce more code than small ({small_len})"
    );
}

// ===========================================================================
// 6. Grammars with various feature combinations
// ===========================================================================

#[test]
fn grammar_with_externals_generates_code() {
    let (grammar, table) = grammar_with_externals();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn grammar_with_fields_generates_node_types() {
    let (grammar, table) = grammar_with_fields();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let json_str = generator.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn grammar_with_extras_generates_code() {
    let (grammar, table) = grammar_with_extras();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn grammar_with_supertypes_marks_subtypes_in_node_types() {
    let mut grammar = GrammarBuilder::new("super_test")
        .token("number", r"\d+")
        .token("id", r"[a-z]+")
        .rule("expr", vec!["number"])
        .rule("expr", vec!["id"])
        .rule("program", vec!["expr"])
        .start("program")
        .build();
    // Mark expr as a supertype
    let expr_id = *grammar.rules.keys().find(|id| {
        grammar.rule_names.get(*id).map(|n| n.as_str()) == Some("expr")
    }).unwrap();
    grammar.supertypes.push(expr_id);

    let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
    let json_str = generator.generate_node_types();
    assert!(
        json_str.contains("subtypes"),
        "supertype symbols should produce subtypes in node types"
    );
}

#[test]
fn grammar_with_multiple_rules_per_symbol() {
    let grammar = GrammarBuilder::new("multi_rule")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
    let code = generator.generate_language_code();
    assert!(!code.is_empty());
    let json = generator.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn grammar_with_no_tokens_still_generates() {
    // Edge case: grammar with only rules and no explicit tokens
    let grammar = Grammar::new("empty_tokens".to_string());
    let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
    let code = generator.generate_language_code();
    assert!(!code.is_empty());
    let json = generator.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());
}

// ===========================================================================
// 7. Output determinism tests
// ===========================================================================

#[test]
fn language_code_is_deterministic() {
    let make = || {
        let (g, t) = arithmetic_grammar_and_table();
        StaticLanguageGenerator::new(g, t)
            .generate_language_code()
            .to_string()
    };
    let a = make();
    let b = make();
    assert_eq!(a, b, "language code generation must be deterministic");
}

#[test]
fn node_types_is_deterministic() {
    let make = || {
        let (g, t) = arithmetic_grammar_and_table();
        StaticLanguageGenerator::new(g, t).generate_node_types()
    };
    let a = make();
    let b = make();
    assert_eq!(a, b, "node types generation must be deterministic");
}

#[test]
fn determinism_across_medium_grammars() {
    let make = || {
        let (g, t) = medium_grammar_and_table();
        (
            StaticLanguageGenerator::new(g, t)
                .generate_language_code()
                .to_string(),
        )
    };
    let (a,) = make();
    let (b,) = make();
    assert_eq!(a, b);
}

#[test]
fn determinism_for_node_types_with_externals() {
    let make = || {
        let (g, t) = grammar_with_externals();
        StaticLanguageGenerator::new(g, t).generate_node_types()
    };
    let a = make();
    let b = make();
    assert_eq!(a, b);
}

#[test]
fn determinism_for_large_grammars() {
    let make = || {
        let (g, t) = large_grammar_and_table();
        StaticLanguageGenerator::new(g, t)
            .generate_language_code()
            .to_string()
    };
    let a = make();
    let b = make();
    assert_eq!(a, b);
}
