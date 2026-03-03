#![allow(clippy::needless_range_loop)]

//! Property-based tests for `StaticLanguageGenerator` in adze-tablegen.
//!
//! Properties verified:
//!  1.  Generated code is valid Rust tokens (non-empty TokenStream)
//!  2.  Generated code contains LANGUAGE constant
//!  3.  Generated code is deterministic (same input → same output)
//!  4.  Symbol metadata in generated code (SYMBOL_METADATA present)
//!  5.  Parse table in generated code (PARSE_TABLE present)
//!  6.  Different grammar sizes produce non-empty output
//!  7.  Generated code roundtrip (parse as syn::File)
//!  8.  Node types JSON is valid
//!  9.  Node types JSON is deterministic
//! 10.  Node types entries have required fields
//! 11.  Node types count scales with grammar
//! 12.  Hidden rules excluded from node types
//! 13.  External tokens appear in node types
//! 14.  Hidden externals excluded from node types
//! 15.  Grammar name embedded in generated code
//! 16.  SYMBOL_NAMES present in generated code
//! 17.  FIELD_NAMES present when grammar has fields
//! 18.  LEX_MODES present in generated code
//! 19.  PARSE_ACTIONS present in generated code
//! 20.  set_start_can_be_empty does not affect code structure
//! 21.  Compressed tables field is None by default
//! 22.  Grammar with extras generates valid code
//! 23.  Grammar with many tokens generates valid code
//! 24.  Grammar with many rules generates valid code
//! 25.  Generator preserves grammar name
//! 26.  Generator preserves parse table state count
//! 27.  Node types JSON array length ≥ 1 for non-trivial grammar
//! 28.  Generated code contains version constant
//! 29.  Generated code contains language function
//! 30.  Different grammar names produce different generated code

use adze_glr_core::ParseTable;
use adze_ir::builder::GrammarBuilder;
use adze_ir::{ExternalToken, FieldId, Grammar, SymbolId};
use adze_tablegen::StaticLanguageGenerator;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a valid grammar name (ASCII lowercase, non-empty).
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_filter("non-empty", |s| !s.is_empty())
}

/// Generate a valid token name (ASCII lowercase, non-empty).
fn token_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_filter("non-empty", |s| !s.is_empty())
}

/// Generate a hidden token name (starts with underscore).
fn hidden_name_strategy() -> impl Strategy<Value = String> {
    "_[a-z][a-z0-9_]{0,10}".prop_filter("must start with _", |s| s.starts_with('_'))
}

/// Generate a field name.
fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z_]{0,10}"
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a grammar with `n` visible tokens and a single rule + start.
fn grammar_with_n_tokens(name: &str, n: usize) -> Grammar {
    let count = n.max(1);
    let mut builder = GrammarBuilder::new(name);
    for i in 0..count {
        builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
    }
    builder = builder.rule("root", vec!["tok0"]).start("root");
    builder.build()
}

/// Build a grammar with multiple rules referencing different tokens.
fn grammar_with_n_rules(n: usize) -> Grammar {
    let count = n.max(1);
    let mut builder = GrammarBuilder::new("multi_rule");
    // Need at least count tokens so each rule can reference a unique token
    for i in 0..count {
        builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
    }
    for i in 0..count {
        let tok = format!("tok{i}");
        builder = builder.rule("root", vec![Box::leak(tok.into_boxed_str())]);
    }
    builder = builder.start("root");
    builder.build()
}

/// Build a grammar with external tokens appended.
fn grammar_with_externals(base_tokens: usize, external_names: Vec<String>) -> Grammar {
    let mut grammar = grammar_with_n_tokens("ext_grammar", base_tokens);
    for (i, name) in external_names.into_iter().enumerate() {
        grammar.externals.push(ExternalToken {
            name,
            symbol_id: SymbolId(200 + i as u16),
        });
    }
    grammar
}

/// Build a grammar with field entries injected.
fn grammar_with_fields(base_tokens: usize, field_names: Vec<String>) -> Grammar {
    let mut grammar = grammar_with_n_tokens("field_grammar", base_tokens);
    for (i, name) in field_names.into_iter().enumerate() {
        grammar.fields.insert(FieldId(i as u16), name);
    }
    grammar
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // 1. Generated code is valid Rust tokens (non-empty TokenStream)
    #[test]
    fn generated_code_is_nonempty(n in 1usize..8) {
        let grammar = grammar_with_n_tokens("nonempty", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code = generator.generate_language_code();
        prop_assert!(!code.is_empty(), "generated TokenStream must not be empty");
    }

    // 2. Generated code contains LANGUAGE constant
    #[test]
    fn generated_code_contains_language_constant(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("lang_const", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code_str = generator.generate_language_code().to_string();
        prop_assert!(
            code_str.contains("LANGUAGE"),
            "generated code must contain LANGUAGE: {}", code_str
        );
    }

    // 3. Generated code is deterministic
    #[test]
    fn generated_code_is_deterministic(n in 1usize..8) {
        let g1 = grammar_with_n_tokens("det", n);
        let g2 = grammar_with_n_tokens("det", n);
        let t1 = ParseTable::default();
        let t2 = ParseTable::default();

        let gen1 = StaticLanguageGenerator::new(g1, t1);
        let gen2 = StaticLanguageGenerator::new(g2, t2);

        let code1 = gen1.generate_language_code().to_string();
        let code2 = gen2.generate_language_code().to_string();
        prop_assert_eq!(&code1, &code2, "same inputs must produce identical code");
    }

    // 4. Symbol metadata in generated code
    #[test]
    fn generated_code_contains_symbol_metadata(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("meta", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code_str = generator.generate_language_code().to_string();
        prop_assert!(
            code_str.contains("SYMBOL_METADATA"),
            "code must contain SYMBOL_METADATA"
        );
    }

    // 5. Parse table in generated code
    #[test]
    fn generated_code_contains_parse_table(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("ptable", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code_str = generator.generate_language_code().to_string();
        prop_assert!(
            code_str.contains("PARSE_TABLE"),
            "code must contain PARSE_TABLE"
        );
    }

    // 6. Different grammar sizes produce non-empty output
    #[test]
    fn different_grammar_sizes_produce_output(n in 1usize..20) {
        let grammar = grammar_with_n_tokens("sized", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code = generator.generate_language_code();
        prop_assert!(!code.is_empty());
    }

    // 7. Generated code roundtrip (parse as syn::File)
    #[test]
    fn generated_code_parses_as_valid_rust(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("syncheck", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code = generator.generate_language_code();
        let result: Result<syn::File, _> = syn::parse2(code);
        prop_assert!(result.is_ok(), "syn parse failed: {:?}", result.err());
    }

    // 8. Node types JSON is valid
    #[test]
    fn node_types_is_valid_json(n in 1usize..8) {
        let grammar = grammar_with_n_tokens("ntjson", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let json_str = generator.generate_node_types();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "node types must be valid JSON");
        prop_assert!(parsed.unwrap().is_array(), "node types must be a JSON array");
    }

    // 9. Node types JSON is deterministic
    #[test]
    fn node_types_is_deterministic(n in 1usize..6) {
        let g1 = grammar_with_n_tokens("ntdet", n);
        let g2 = grammar_with_n_tokens("ntdet", n);

        let gen1 = StaticLanguageGenerator::new(g1, ParseTable::default());
        let gen2 = StaticLanguageGenerator::new(g2, ParseTable::default());

        let json1 = gen1.generate_node_types();
        let json2 = gen2.generate_node_types();
        prop_assert_eq!(&json1, &json2);
    }

    // 10. Node types entries have required fields
    #[test]
    fn node_types_entries_have_type_and_named(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("ntfields", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let json_str = generator.generate_node_types();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        for entry in parsed.as_array().unwrap() {
            prop_assert!(entry.get("type").is_some(), "entry missing 'type'");
            prop_assert!(entry.get("named").is_some(), "entry missing 'named'");
        }
    }

    // 11. Node types count scales with grammar (more tokens → more or equal entries)
    #[test]
    fn node_types_count_scales(n in 1usize..6) {
        let small = grammar_with_n_tokens("scale_s", 1);
        let big = grammar_with_n_tokens("scale_b", n + 1);

        let gen_s = StaticLanguageGenerator::new(small, ParseTable::default());
        let gen_b = StaticLanguageGenerator::new(big, ParseTable::default());

        let arr_s: serde_json::Value = serde_json::from_str(&gen_s.generate_node_types()).unwrap();
        let arr_b: serde_json::Value = serde_json::from_str(&gen_b.generate_node_types()).unwrap();

        prop_assert!(
            arr_b.as_array().unwrap().len() >= arr_s.as_array().unwrap().len(),
            "bigger grammar should produce >= node type entries"
        );
    }

    // 12. Hidden rules excluded from node types
    #[test]
    fn hidden_rules_excluded_from_node_types(
        hidden in hidden_name_strategy(),
    ) {
        let grammar = GrammarBuilder::new("hidden_test")
            .token("a", "a")
            .rule(&hidden, vec!["a"])
            .rule("visible", vec![Box::leak(hidden.clone().into_boxed_str())])
            .start("visible")
            .build();
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let json_str = generator.generate_node_types();
        // The hidden rule name should not appear as a quoted type value
        let quoted = format!("\"{}\"", hidden);
        prop_assert!(
            !json_str.contains(&quoted),
            "hidden rule '{}' should not appear in node types", hidden
        );
    }

    // 13. External tokens appear in node types
    #[test]
    fn external_tokens_in_node_types(
        ext_names in prop::collection::vec(token_name_strategy(), 1..4),
    ) {
        let grammar = grammar_with_externals(1, ext_names.clone());
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let json_str = generator.generate_node_types();
        for name in &ext_names {
            prop_assert!(
                json_str.contains(name),
                "external token '{}' should appear in node types", name
            );
        }
    }

    // 14. Hidden externals excluded from node types
    #[test]
    fn hidden_externals_excluded(
        hidden in hidden_name_strategy(),
    ) {
        let grammar = grammar_with_externals(1, vec![hidden.clone()]);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let json_str = generator.generate_node_types();
        let quoted = format!("\"{}\"", hidden);
        prop_assert!(
            !json_str.contains(&quoted),
            "hidden external '{}' should not appear in node types", hidden
        );
    }

    // 15. Grammar name embedded in generated code
    #[test]
    fn grammar_name_embedded_in_code(name in grammar_name_strategy()) {
        let grammar = grammar_with_n_tokens(&name, 1);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code_str = generator.generate_language_code().to_string();
        prop_assert!(
            code_str.contains(&name),
            "grammar name '{}' should appear in generated code", name
        );
    }

    // 16. SYMBOL_NAMES present in generated code
    #[test]
    fn symbol_names_in_generated_code(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("symnames", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code_str = generator.generate_language_code().to_string();
        prop_assert!(code_str.contains("SYMBOL_NAMES"));
    }

    // 17. FIELD_NAMES present when grammar has fields
    #[test]
    fn field_names_in_code_when_fields_exist(
        field_names in prop::collection::vec(field_name_strategy(), 1..4),
    ) {
        let grammar = grammar_with_fields(1, field_names);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code_str = generator.generate_language_code().to_string();
        prop_assert!(
            code_str.contains("FIELD_NAMES"),
            "FIELD_NAMES must appear when grammar has fields"
        );
    }

    // 18. LEX_MODES present in generated code
    #[test]
    fn lex_modes_in_generated_code(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("lexmodes", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code_str = generator.generate_language_code().to_string();
        prop_assert!(code_str.contains("LEX_MODES"));
    }

    // 19. PARSE_ACTIONS present in generated code
    #[test]
    fn parse_actions_in_generated_code(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("pactions", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code_str = generator.generate_language_code().to_string();
        prop_assert!(code_str.contains("PARSE_ACTIONS"));
    }

    // 20. set_start_can_be_empty does not affect code structure
    #[test]
    fn start_can_be_empty_does_not_change_code(n in 1usize..6) {
        let g1 = grammar_with_n_tokens("empty_flag", n);
        let g2 = grammar_with_n_tokens("empty_flag", n);

        let mut gen1 = StaticLanguageGenerator::new(g1, ParseTable::default());
        let gen2 = StaticLanguageGenerator::new(g2, ParseTable::default());

        gen1.set_start_can_be_empty(true);

        let code1 = gen1.generate_language_code().to_string();
        let code2 = gen2.generate_language_code().to_string();
        // generate_language_code delegates to LanguageGenerator which ignores start_can_be_empty
        prop_assert_eq!(&code1, &code2);
    }

    // 21. Compressed tables field is None by default
    #[test]
    fn compressed_tables_none_by_default(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("no_compress", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        prop_assert!(generator.compressed_tables.is_none());
    }

    // 22. Grammar with extras generates valid code
    #[test]
    fn grammar_with_extras_generates_valid_code(n in 1usize..4) {
        let mut builder = GrammarBuilder::new("extras_test");
        for i in 0..n {
            builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
        }
        builder = builder.token("ws", r"[ \t]+");
        builder = builder.extra("ws");
        builder = builder.rule("root", vec!["tok0"]).start("root");
        let grammar = builder.build();

        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code = generator.generate_language_code();
        prop_assert!(!code.is_empty());
        let result: Result<syn::File, _> = syn::parse2(code);
        prop_assert!(result.is_ok(), "extras grammar code must parse: {:?}", result.err());
    }

    // 23. Grammar with many tokens generates valid code
    #[test]
    fn many_tokens_generates_valid_code(n in 5usize..15) {
        let grammar = grammar_with_n_tokens("many_tok", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code = generator.generate_language_code();
        prop_assert!(!code.is_empty());
        let result: Result<syn::File, _> = syn::parse2(code);
        prop_assert!(result.is_ok(), "many-token grammar must parse: {:?}", result.err());
    }

    // 24. Grammar with many rules generates valid code
    #[test]
    fn many_rules_generates_valid_code(n in 2usize..10) {
        let grammar = grammar_with_n_rules(n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code = generator.generate_language_code();
        prop_assert!(!code.is_empty());
        let result: Result<syn::File, _> = syn::parse2(code);
        prop_assert!(result.is_ok(), "many-rule grammar must parse: {:?}", result.err());
    }

    // 25. Generator preserves grammar name
    #[test]
    fn generator_preserves_grammar_name(name in grammar_name_strategy()) {
        let grammar = grammar_with_n_tokens(&name, 1);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        prop_assert_eq!(&generator.grammar.name, &name);
    }

    // 26. Generator preserves parse table state count
    #[test]
    fn generator_preserves_state_count(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("preserve_sc", n);
        let table = ParseTable::default();
        let expected = table.state_count;
        let generator = StaticLanguageGenerator::new(grammar, table);
        prop_assert_eq!(generator.parse_table.state_count, expected);
    }

    // 27. Node types JSON array length ≥ 1 for non-trivial grammar
    #[test]
    fn node_types_array_nonempty(n in 1usize..8) {
        let grammar = grammar_with_n_tokens("ntne", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let json_str = generator.generate_node_types();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let arr = parsed.as_array().unwrap();
        prop_assert!(!arr.is_empty(), "node types array should not be empty for n={}", n);
    }

    // 28. Generated code contains version constant
    #[test]
    fn generated_code_contains_version(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("verchk", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code_str = generator.generate_language_code().to_string();
        prop_assert!(
            code_str.contains("LANGUAGE_VERSION"),
            "code must reference LANGUAGE_VERSION"
        );
    }

    // 29. Generated code contains language function
    #[test]
    fn generated_code_contains_language_fn(n in 1usize..6) {
        let grammar = grammar_with_n_tokens("langfn", n);
        let generator = StaticLanguageGenerator::new(grammar, ParseTable::default());
        let code_str = generator.generate_language_code().to_string();
        prop_assert!(
            code_str.contains("language"),
            "code must contain a `language` function"
        );
    }

    // 30. Different grammar names produce different generated code
    #[test]
    fn different_names_produce_different_code(
        name1 in "[a-z]{3,6}",
        name2 in "[a-z]{3,6}",
    ) {
        prop_assume!(name1 != name2);
        let g1 = grammar_with_n_tokens(&name1, 1);
        let g2 = grammar_with_n_tokens(&name2, 1);

        let gen1 = StaticLanguageGenerator::new(g1, ParseTable::default());
        let gen2 = StaticLanguageGenerator::new(g2, ParseTable::default());

        let code1 = gen1.generate_language_code().to_string();
        let code2 = gen2.generate_language_code().to_string();
        prop_assert_ne!(&code1, &code2, "different names should yield different code");
    }
}
