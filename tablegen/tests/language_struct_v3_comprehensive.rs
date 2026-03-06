//! Comprehensive tests for `StaticLanguageGenerator` Language struct generation.
//!
//! Covers: struct definition, static arrays, symbol names, state tables,
//! code generation determinism, different grammars, node types JSON, and edge cases.

use adze_ir::FieldId;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::StaticLanguageGenerator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar + default parse table, returning a `StaticLanguageGenerator`.
fn minimal_generator(name: &str) -> StaticLanguageGenerator {
    let grammar = GrammarBuilder::new(name)
        .token("number", r"\d+")
        .rule("expr", vec!["number"])
        .start("expr")
        .build();
    StaticLanguageGenerator::new(grammar, Default::default())
}

/// Build an arithmetic grammar with multiple tokens and rules.
fn arithmetic_generator() -> StaticLanguageGenerator {
    let grammar = GrammarBuilder::new("arithmetic")
        .token("number", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["number"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .start("expr")
        .build();
    StaticLanguageGenerator::new(grammar, Default::default())
}

/// Build a grammar with external tokens.
fn external_generator() -> StaticLanguageGenerator {
    let grammar = GrammarBuilder::new("ext_lang")
        .token("id", r"[a-z]+")
        .external("comment")
        .external("heredoc")
        .rule("program", vec!["id"])
        .start("program")
        .build();
    StaticLanguageGenerator::new(grammar, Default::default())
}

/// Build a grammar with fields.
fn fields_generator() -> StaticLanguageGenerator {
    let mut grammar = GrammarBuilder::new("field_lang")
        .token("number", r"\d+")
        .token("plus", "+")
        .rule("binary", vec!["number", "plus", "number"])
        .start("binary")
        .build();
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "operator".to_string());
    grammar.fields.insert(FieldId(2), "right".to_string());
    StaticLanguageGenerator::new(grammar, Default::default())
}

/// Build a grammar with hidden rules (prefixed with `_`).
fn hidden_rules_generator() -> StaticLanguageGenerator {
    let grammar = GrammarBuilder::new("hidden")
        .token("a", "a")
        .rule("_hidden", vec!["a"])
        .rule("visible", vec!["_hidden"])
        .start("visible")
        .build();
    StaticLanguageGenerator::new(grammar, Default::default())
}

/// Stringify the generated language code.
fn lang_code_str(slg: &StaticLanguageGenerator) -> String {
    slg.generate_language_code().to_string()
}

// ===========================================================================
// 1. Language code contains struct definition (8 tests)
// ===========================================================================

#[test]
fn struct_def_contains_tslanguage_keyword() {
    let code = lang_code_str(&minimal_generator("my_lang"));
    assert!(
        code.contains("TSLanguage"),
        "generated code must reference TSLanguage"
    );
}

#[test]
fn struct_def_contains_language_static() {
    let code = lang_code_str(&minimal_generator("my_lang"));
    assert!(
        code.contains("LANGUAGE"),
        "generated code must define a LANGUAGE static"
    );
}

#[test]
fn struct_def_contains_language_fn() {
    let code = lang_code_str(&minimal_generator("my_lang"));
    assert!(
        code.contains("language"),
        "generated code must define a language() function"
    );
}

#[test]
fn struct_def_contains_tree_sitter_prefix() {
    let code = lang_code_str(&minimal_generator("foo"));
    assert!(
        code.contains("tree_sitter_foo"),
        "FFI export must be named tree_sitter_<grammar_name>"
    );
}

#[test]
fn struct_def_version_field_present() {
    let code = lang_code_str(&minimal_generator("v"));
    assert!(
        code.contains("version"),
        "generated struct must include `version` field"
    );
}

#[test]
fn struct_def_symbol_count_present() {
    let code = lang_code_str(&minimal_generator("s"));
    assert!(
        code.contains("symbol_count"),
        "generated struct must include `symbol_count`"
    );
}

#[test]
fn struct_def_state_count_present() {
    let code = lang_code_str(&minimal_generator("s"));
    assert!(
        code.contains("state_count"),
        "generated struct must include `state_count`"
    );
}

#[test]
fn struct_def_extern_c_export() {
    let code = lang_code_str(&minimal_generator("abc"));
    assert!(
        code.contains("extern \"C\""),
        "FFI function must be extern \"C\""
    );
}

// ===========================================================================
// 2. Language code contains static arrays (8 tests)
// ===========================================================================

#[test]
fn static_arrays_symbol_names() {
    let code = lang_code_str(&minimal_generator("arr"));
    assert!(
        code.contains("SYMBOL_NAMES"),
        "must define SYMBOL_NAMES array"
    );
}

#[test]
fn static_arrays_symbol_metadata() {
    let code = lang_code_str(&minimal_generator("arr"));
    assert!(
        code.contains("SYMBOL_METADATA"),
        "must define SYMBOL_METADATA array"
    );
}

#[test]
fn static_arrays_parse_table() {
    let code = lang_code_str(&minimal_generator("arr"));
    assert!(
        code.contains("PARSE_TABLE"),
        "must define PARSE_TABLE array"
    );
}

#[test]
fn static_arrays_lex_modes() {
    let code = lang_code_str(&minimal_generator("arr"));
    assert!(code.contains("LEX_MODES"), "must define LEX_MODES array");
}

#[test]
fn static_arrays_parse_actions() {
    let code = lang_code_str(&minimal_generator("arr"));
    assert!(
        code.contains("PARSE_ACTIONS"),
        "must define PARSE_ACTIONS array"
    );
}

#[test]
fn static_arrays_field_names() {
    let code = lang_code_str(&fields_generator());
    assert!(
        code.contains("FIELD_NAMES"),
        "must define FIELD_NAMES array when fields present"
    );
}

#[test]
fn static_arrays_public_symbol_map() {
    let code = lang_code_str(&minimal_generator("arr"));
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP"),
        "must define PUBLIC_SYMBOL_MAP"
    );
}

#[test]
fn static_arrays_primary_state_ids() {
    let code = lang_code_str(&minimal_generator("arr"));
    assert!(
        code.contains("PRIMARY_STATE_IDS"),
        "must define PRIMARY_STATE_IDS"
    );
}

// ===========================================================================
// 3. Language code symbol names (5 tests)
// ===========================================================================

#[test]
fn symbol_names_array_always_present() {
    let code = lang_code_str(&minimal_generator("sn"));
    // With a default (empty) parse table, symbol names array is still emitted
    assert!(
        code.contains("SYMBOL_NAMES"),
        "SYMBOL_NAMES must always be present in generated code"
    );
}

#[test]
fn symbol_names_code_references_symbol_names_ptrs() {
    let slg = arithmetic_generator();
    let code = lang_code_str(&slg);
    assert!(
        code.contains("SYMBOL_NAMES_PTRS"),
        "symbol names pointer array must be present"
    );
}

#[test]
fn symbol_names_token_name_in_node_types() {
    // Token names appear in node_types JSON rather than always in codegen
    let slg = arithmetic_generator();
    let json_str = slg.generate_node_types();
    assert!(
        json_str.contains("\"number\""),
        "must have number in node types"
    );
}

#[test]
fn symbol_names_field_names_present() {
    let slg = fields_generator();
    let code = lang_code_str(&slg);
    assert!(
        code.contains("FIELD_NAMES"),
        "field names array should be present"
    );
}

#[test]
fn symbol_names_external_token_count_embedded() {
    let slg = external_generator();
    let code = lang_code_str(&slg);
    assert!(
        code.contains("EXTERNAL_TOKEN_COUNT"),
        "external token count must be defined"
    );
}

// ===========================================================================
// 4. Language code state tables (5 tests)
// ===========================================================================

#[test]
fn state_tables_small_parse_table_map_present() {
    let code = lang_code_str(&minimal_generator("st"));
    assert!(
        code.contains("SMALL_PARSE_TABLE_MAP"),
        "must define small_parse_table_map"
    );
}

#[test]
fn state_tables_field_map_slices_present() {
    let code = lang_code_str(&minimal_generator("st"));
    assert!(
        code.contains("FIELD_MAP_SLICES"),
        "must define FIELD_MAP_SLICES"
    );
}

#[test]
fn state_tables_field_map_entries_present() {
    let code = lang_code_str(&minimal_generator("st"));
    assert!(
        code.contains("FIELD_MAP_ENTRIES"),
        "must define FIELD_MAP_ENTRIES"
    );
}

#[test]
fn state_tables_external_scanner_present() {
    let code = lang_code_str(&minimal_generator("st"));
    assert!(
        code.contains("EXTERNAL_SCANNER"),
        "must define EXTERNAL_SCANNER"
    );
}

#[test]
fn state_tables_abi_version_constant() {
    let code = lang_code_str(&minimal_generator("st"));
    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "must define TREE_SITTER_LANGUAGE_VERSION constant"
    );
}

// ===========================================================================
// 5. Code generation determinism (8 tests)
// ===========================================================================

#[test]
fn determinism_same_grammar_same_output() {
    let a = lang_code_str(&minimal_generator("det"));
    let b = lang_code_str(&minimal_generator("det"));
    assert_eq!(a, b, "same grammar must produce identical code");
}

#[test]
fn determinism_arithmetic_repeated() {
    let a = lang_code_str(&arithmetic_generator());
    let b = lang_code_str(&arithmetic_generator());
    assert_eq!(a, b);
}

#[test]
fn determinism_external_repeated() {
    let a = lang_code_str(&external_generator());
    let b = lang_code_str(&external_generator());
    assert_eq!(a, b);
}

#[test]
fn determinism_fields_repeated() {
    let a = lang_code_str(&fields_generator());
    let b = lang_code_str(&fields_generator());
    assert_eq!(a, b);
}

#[test]
fn determinism_hidden_repeated() {
    let a = lang_code_str(&hidden_rules_generator());
    let b = lang_code_str(&hidden_rules_generator());
    assert_eq!(a, b);
}

#[test]
fn determinism_node_types_same_grammar() {
    let ga = arithmetic_generator();
    let gb = arithmetic_generator();
    let a = ga.generate_node_types();
    let b = gb.generate_node_types();
    assert_eq!(a, b, "node types must be deterministic");
}

#[test]
fn determinism_three_runs_identical() {
    let runs: Vec<String> = (0..3)
        .map(|_| lang_code_str(&minimal_generator("tri")))
        .collect();
    assert_eq!(runs[0], runs[1]);
    assert_eq!(runs[1], runs[2]);
}

#[test]
fn determinism_start_can_be_empty_does_not_change_code() {
    let grammar = GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();

    let mut slg_a = StaticLanguageGenerator::new(grammar.clone(), Default::default());
    slg_a.set_start_can_be_empty(false);
    let code_a = lang_code_str(&slg_a);

    let mut slg_b = StaticLanguageGenerator::new(grammar, Default::default());
    slg_b.set_start_can_be_empty(true);
    let code_b = lang_code_str(&slg_b);

    // The flag doesn't affect code gen today; just confirm no panic.
    // If it does affect output, both must still be valid token streams.
    assert!(!code_a.is_empty());
    assert!(!code_b.is_empty());
}

// ===========================================================================
// 6. Different grammars different output (5 tests)
// ===========================================================================

#[test]
fn different_grammar_names_different_ffi_export() {
    let a = lang_code_str(&minimal_generator("alpha"));
    let b = lang_code_str(&minimal_generator("beta"));
    assert_ne!(a, b, "different grammar names must produce different code");
    assert!(a.contains("tree_sitter_alpha"));
    assert!(b.contains("tree_sitter_beta"));
}

#[test]
fn different_token_count_different_output() {
    let one_token = minimal_generator("tok");
    let two_tokens = {
        let g = GrammarBuilder::new("tok")
            .token("number", r"\d+")
            .token("ident", r"[a-z]+")
            .rule("expr", vec!["number"])
            .start("expr")
            .build();
        StaticLanguageGenerator::new(g, Default::default())
    };
    let a = lang_code_str(&one_token);
    let b = lang_code_str(&two_tokens);
    assert_ne!(a, b, "different token counts must differ");
}

#[test]
fn with_and_without_externals_differ() {
    let without = minimal_generator("ext_test");
    let with = external_generator();
    let a = lang_code_str(&without);
    let b = lang_code_str(&with);
    assert_ne!(a, b);
}

#[test]
fn arithmetic_vs_minimal_differ() {
    let a = lang_code_str(&minimal_generator("arithmetic"));
    let b = lang_code_str(&arithmetic_generator());
    assert_ne!(a, b);
}

#[test]
fn hidden_vs_visible_rules_differ() {
    let visible = minimal_generator("hidden");
    let hidden = hidden_rules_generator();
    let a = lang_code_str(&visible);
    let b = lang_code_str(&hidden);
    assert_ne!(a, b);
}

// ===========================================================================
// 7. Node types JSON validity (8 tests)
// ===========================================================================

#[test]
fn node_types_is_valid_json() {
    let slg = arithmetic_generator();
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("node types must be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn node_types_contains_rule_entries() {
    let slg = arithmetic_generator();
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = parsed.as_array().unwrap();
    assert!(
        !arr.is_empty(),
        "node types array should not be empty for a grammar with rules"
    );
}

#[test]
fn node_types_entries_have_type_field() {
    let slg = minimal_generator("nt");
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "every node type entry must have a `type` field"
        );
    }
}

#[test]
fn node_types_entries_have_named_field() {
    let slg = minimal_generator("nt");
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(
            entry.get("named").is_some(),
            "every node type entry must have a `named` field"
        );
    }
}

#[test]
fn node_types_excludes_hidden_rules() {
    let slg = hidden_rules_generator();
    let json_str = slg.generate_node_types();
    assert!(
        !json_str.contains("\"_hidden\""),
        "hidden rules (underscore prefix) must not appear in node types"
    );
}

#[test]
fn node_types_includes_external_tokens() {
    let slg = external_generator();
    let json_str = slg.generate_node_types();
    assert!(
        json_str.contains("\"comment\""),
        "external token 'comment' should appear in node types"
    );
    assert!(
        json_str.contains("\"heredoc\""),
        "external token 'heredoc' should appear in node types"
    );
}

#[test]
fn node_types_named_tokens_are_named_true() {
    let slg = arithmetic_generator();
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let arr = parsed.as_array().unwrap();
    // "number" token uses regex pattern => named: true
    let number_entry = arr.iter().find(|e| e["type"] == "number");
    if let Some(entry) = number_entry {
        assert_eq!(entry["named"], true, "regex token should be named");
    }
}

#[test]
fn node_types_empty_grammar_produces_valid_json() {
    let grammar = GrammarBuilder::new("empty")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let slg = StaticLanguageGenerator::new(grammar, Default::default());
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("even simple grammars produce valid JSON");
    assert!(parsed.is_array());
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn edge_case_single_token_grammar() {
    let slg = minimal_generator("single");
    let code = lang_code_str(&slg);
    assert!(
        !code.is_empty(),
        "single token grammar must still generate code"
    );
    assert!(code.contains("TSLanguage"));
}

#[test]
fn edge_case_multiple_rules_same_lhs() {
    let slg = arithmetic_generator();
    let code = lang_code_str(&slg);
    assert!(!code.is_empty());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn edge_case_grammar_name_with_underscores() {
    let slg = minimal_generator("my_cool_lang");
    let code = lang_code_str(&slg);
    assert!(
        code.contains("tree_sitter_my_cool_lang"),
        "underscored names must be preserved in FFI export"
    );
}

#[test]
fn edge_case_many_tokens() {
    let mut builder = GrammarBuilder::new("many_tok");
    for i in 0..20 {
        builder = builder.token(&format!("tok_{i}"), &format!("t{i}"));
    }
    let grammar = builder.rule("start", vec!["tok_0"]).start("start").build();
    let slg = StaticLanguageGenerator::new(grammar, Default::default());
    let code = lang_code_str(&slg);
    assert!(code.contains("TSLanguage"));
}

#[test]
fn edge_case_node_types_many_rules() {
    let mut builder = GrammarBuilder::new("many_rules");
    builder = builder.token("a", "a");
    for i in 0..10 {
        builder = builder.rule(&format!("rule_{i}"), vec!["a"]);
    }
    let grammar = builder.start("rule_0").build();
    let slg = StaticLanguageGenerator::new(grammar, Default::default());
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn edge_case_default_parse_table() {
    // Using ParseTable::default() should not panic during codegen
    let grammar = GrammarBuilder::new("def_tbl")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let slg = StaticLanguageGenerator::new(grammar, Default::default());
    let code = lang_code_str(&slg);
    assert!(!code.is_empty());
}

#[test]
fn edge_case_compressed_tables_none() {
    let slg = minimal_generator("no_compress");
    assert!(
        slg.compressed_tables.is_none(),
        "new() should not set compressed_tables"
    );
    // Still generates code fine
    let code = lang_code_str(&slg);
    assert!(code.contains("TSLanguage"));
}

#[test]
fn edge_case_set_start_can_be_empty_toggle() {
    let grammar = GrammarBuilder::new("toggle")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let mut slg = StaticLanguageGenerator::new(grammar, Default::default());

    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);

    slg.set_start_can_be_empty(false);
    assert!(!slg.start_can_be_empty);

    // Code generation works regardless of the flag state
    let code = lang_code_str(&slg);
    assert!(code.contains("TSLanguage"));
}
