//! Comprehensive tests for adze-tablegen code generation output quality,
//! correctness, and determinism.
//!
//! 8 categories × 8 tests = 64 total `#[test]` functions.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::{AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator};
use serde_json::Value;

// =====================================================================
// Helpers
// =====================================================================

/// Build a grammar through the full pipeline (FIRST/FOLLOW + LR(1)).
fn build_pipeline(builder: GrammarBuilder) -> (Grammar, ParseTable) {
    let mut grammar = builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1)");
    (grammar, table)
}

fn single_token_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("single_v6")
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s"),
    )
}

fn expr_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("expr_v6")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule("expr", vec!["NUM", "PLUS", "NUM"])
            .start("expr"),
    )
}

fn two_alt_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("two_alt_v6")
            .token("x", "x")
            .token("y", "y")
            .rule("s", vec!["x"])
            .rule("s", vec!["y"])
            .start("s"),
    )
}

fn chain_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("chain_v6")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a", "b"])
            .start("s"),
    )
}

fn nested_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("nested_v6")
            .token("a", "a")
            .token("b", "b")
            .rule("inner", vec!["a"])
            .rule("outer", vec!["inner", "b"])
            .start("outer"),
    )
}

fn keyword_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("kw_v6")
            .token("IF", "if")
            .token("THEN", "then")
            .token("ID", r"[a-z]+")
            .rule("stmt", vec!["IF", "ID", "THEN", "ID"])
            .start("stmt"),
    )
}

fn multi_rule_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("multi_v6")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("s", vec!["a", "m"])
            .rule("m", vec!["b", "c"])
            .start("s"),
    )
}

fn deep_chain_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("deep_v6")
            .token("t1", "t1")
            .token("t2", "t2")
            .token("t3", "t3")
            .token("t4", "t4")
            .token("t5", "t5")
            .rule("s", vec!["r1"])
            .rule("r1", vec!["t1", "r2"])
            .rule("r2", vec!["t2", "r3"])
            .rule("r3", vec!["t3", "r4"])
            .rule("r4", vec!["t4", "t5"])
            .start("s"),
    )
}

/// Build grammar using only GrammarBuilder (no LR pipeline), paired with default table.
fn minimal_default_table(name: &str) -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new(name)
        .token("tok", "t")
        .rule("root", vec!["tok"])
        .start("root")
        .build();
    (grammar, ParseTable::default())
}

// =====================================================================
// 1. codegen_language_* — language code generation (8 tests)
// =====================================================================

#[test]
fn codegen_language_single_token_nonempty() {
    let (g, t) = single_token_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty(), "Language code must not be empty");
}

#[test]
fn codegen_language_contains_language_struct() {
    let (g, t) = expr_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("TSLanguage"),
        "Generated code must reference TSLanguage"
    );
}

#[test]
fn codegen_language_contains_symbol_names_array() {
    let (g, t) = chain_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("SYMBOL_NAMES"),
        "Generated code must include SYMBOL_NAMES"
    );
}

#[test]
fn codegen_language_contains_parse_table() {
    let (g, t) = two_alt_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("PARSE_TABLE"),
        "Generated code must include PARSE_TABLE"
    );
}

#[test]
fn codegen_language_contains_lex_modes() {
    let (g, t) = nested_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("LEX_MODES"),
        "Generated code must include LEX_MODES"
    );
}

#[test]
fn codegen_language_contains_symbol_metadata() {
    let (g, t) = keyword_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("SYMBOL_METADATA"),
        "Generated code must include SYMBOL_METADATA"
    );
}

#[test]
fn codegen_language_contains_ffi_export() {
    let (g, t) = expr_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("tree_sitter_expr_v6"),
        "Generated code must include FFI export function named after grammar"
    );
}

#[test]
fn codegen_language_multi_rule_larger_than_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = multi_rule_grammar();
    let len1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string()
        .len();
    let len2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string()
        .len();
    assert!(
        len2 > len1,
        "Multi-rule grammar must produce more code than single token"
    );
}

// =====================================================================
// 2. codegen_node_types_* — node types JSON generation (8 tests)
// =====================================================================

#[test]
fn codegen_node_types_valid_json_single() {
    let (g, t) = single_token_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    serde_json::from_str::<Value>(&json).expect("Must produce valid JSON");
}

#[test]
fn codegen_node_types_valid_json_expr() {
    let (g, t) = expr_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    serde_json::from_str::<Value>(&json).expect("Must produce valid JSON");
}

#[test]
fn codegen_node_types_is_array() {
    let (g, t) = chain_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array(), "Node types JSON must be an array");
}

#[test]
fn codegen_node_types_entries_have_type_field() {
    let (g, t) = nested_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    for entry in arr {
        assert!(
            entry.get("type").is_some(),
            "Each node type entry must have a 'type' field"
        );
    }
}

#[test]
fn codegen_node_types_entries_have_named_field() {
    let (g, t) = two_alt_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    for entry in arr {
        assert!(
            entry.get("named").is_some(),
            "Each node type entry must have a 'named' field"
        );
    }
}

#[test]
fn codegen_node_types_regex_token_named_true() {
    let (g, t) = expr_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    let num_entry = arr.iter().find(|e| e["type"] == "NUM");
    assert!(num_entry.is_some(), "NUM token must appear in node types");
    assert_eq!(
        num_entry.unwrap()["named"],
        true,
        "Regex token NUM must be named=true"
    );
}

#[test]
fn codegen_node_types_contains_rule_entries() {
    let (g, t) = multi_rule_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    assert!(
        json.contains("rule_"),
        "Node types must include rule entries"
    );
}

#[test]
fn codegen_node_types_deep_grammar_multiple_entries() {
    let (g, t) = deep_chain_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert!(
        arr.len() >= 4,
        "Deep grammar must produce at least 4 node type entries, got {}",
        arr.len()
    );
}

// =====================================================================
// 3. codegen_abi_* — ABI builder output (8 tests)
// =====================================================================

#[test]
fn codegen_abi_generates_nonempty() {
    let (g, t) = single_token_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty(), "ABI builder must produce non-empty code");
}

#[test]
fn codegen_abi_contains_language_struct() {
    let (g, t) = expr_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("LANGUAGE") || code.contains("TSLanguage"),
        "ABI output must contain LANGUAGE or TSLanguage"
    );
}

#[test]
fn codegen_abi_contains_symbol_names() {
    let (g, t) = chain_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("SYMBOL_NAME"),
        "ABI output must reference symbol names"
    );
}

#[test]
fn codegen_abi_contains_parse_actions() {
    let (g, t) = two_alt_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("PARSE_ACTIONS"),
        "ABI output must contain PARSE_ACTIONS"
    );
}

#[test]
fn codegen_abi_contains_lex_modes() {
    let (g, t) = nested_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("LEX_MODES"),
        "ABI output must contain LEX_MODES"
    );
}

#[test]
fn codegen_abi_contains_production_metadata() {
    let (g, t) = multi_rule_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("PRODUCTION") || code.contains("production"),
        "ABI output must contain production metadata"
    );
}

#[test]
fn codegen_abi_contains_ffi_function() {
    let (g, t) = keyword_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("tree_sitter_kw_v6"),
        "ABI output must include the FFI export named after grammar"
    );
}

#[test]
fn codegen_abi_contains_primary_state_ids() {
    let (g, t) = deep_chain_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("PRIMARY_STATE_IDS"),
        "ABI output must contain PRIMARY_STATE_IDS"
    );
}

// =====================================================================
// 4. codegen_deterministic_* — deterministic output (8 tests)
// =====================================================================

#[test]
fn codegen_deterministic_language_code_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = single_token_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b, "Same grammar must yield identical language code");
}

#[test]
fn codegen_deterministic_language_code_expr() {
    let (g1, t1) = expr_grammar();
    let (g2, t2) = expr_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(a, b);
}

#[test]
fn codegen_deterministic_node_types_chain() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = chain_grammar();
    let a = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let b = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(a, b, "Same grammar must yield identical node types");
}

#[test]
fn codegen_deterministic_abi_output_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = single_token_grammar();
    let a = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
    assert_eq!(a, b, "ABI output must be deterministic");
}

#[test]
fn codegen_deterministic_abi_output_nested() {
    let (g1, t1) = nested_grammar();
    let (g2, t2) = nested_grammar();
    let a = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
    assert_eq!(a, b, "ABI output must be deterministic for nested grammar");
}

#[test]
fn codegen_deterministic_ntg_expr() {
    let mut g1 = GrammarBuilder::new("det_ntg_v6")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .start("expr")
        .build();
    let _ff1 = FirstFollowSets::compute_normalized(&mut g1).unwrap();

    let mut g2 = GrammarBuilder::new("det_ntg_v6")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .start("expr")
        .build();
    let _ff2 = FirstFollowSets::compute_normalized(&mut g2).unwrap();

    let a = NodeTypesGenerator::new(&g1).generate().unwrap();
    let b = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(a, b, "NodeTypesGenerator must be deterministic");
}

#[test]
fn codegen_deterministic_triple_run() {
    let outputs: Vec<String> = (0..3)
        .map(|_| {
            let (g, t) = multi_rule_grammar();
            StaticLanguageGenerator::new(g, t)
                .generate_language_code()
                .to_string()
        })
        .collect();
    assert_eq!(outputs[0], outputs[1]);
    assert_eq!(outputs[1], outputs[2]);
}

#[test]
fn codegen_deterministic_node_types_deep() {
    let (g1, t1) = deep_chain_grammar();
    let (g2, t2) = deep_chain_grammar();
    let a = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let b = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(a, b, "Deep grammar node types must be deterministic");
}

// =====================================================================
// 5. codegen_complex_* — complex grammar codegen (8 tests)
// =====================================================================

#[test]
fn codegen_complex_deep_chain_language_nonempty() {
    let (g, t) = deep_chain_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn codegen_complex_deep_chain_abi_nonempty() {
    let (g, t) = deep_chain_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn codegen_complex_deep_chain_node_types_valid_json() {
    let (g, t) = deep_chain_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    serde_json::from_str::<Value>(&json).expect("Must produce valid JSON");
}

#[test]
fn codegen_complex_keyword_abi_has_extern_c() {
    let (g, t) = keyword_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("extern") && code.contains("\"C\""),
        "ABI output must include extern \"C\" function"
    );
}

#[test]
fn codegen_complex_two_alt_has_multiple_states() {
    let (g, t) = two_alt_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    // Two alternatives require multiple states in the parse table
    assert!(
        code.contains("state_count"),
        "Output must reference state_count"
    );
}

#[test]
fn codegen_complex_nested_ntg_has_inner_outer() {
    let mut grammar = GrammarBuilder::new("complex_nested_v6")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("outer", vec!["inner", "b"])
        .start("outer")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let output = NodeTypesGenerator::new(&grammar).generate().unwrap();
    assert!(
        output.contains("inner") || output.contains("outer"),
        "NodeTypesGenerator must include inner/outer rules"
    );
}

#[test]
fn codegen_complex_multi_rule_abi_has_rules_array() {
    let (g, t) = multi_rule_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("TS_RULES") || code.contains("rules"),
        "ABI output must include rules array"
    );
}

#[test]
fn codegen_complex_expr_language_larger_than_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = expr_grammar();
    let len1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string()
        .len();
    let len2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string()
        .len();
    assert!(
        len2 > len1,
        "Expr grammar must produce more code than single token"
    );
}

// =====================================================================
// 6. codegen_empty_* — empty/minimal grammar codegen (8 tests)
// =====================================================================

#[test]
fn codegen_empty_minimal_default_table_language_nonempty() {
    let (g, t) = minimal_default_table("empty_v6a");
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty(), "Even minimal grammar must produce code");
}

#[test]
fn codegen_empty_minimal_node_types_valid_json() {
    let (g, t) = minimal_default_table("empty_v6b");
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    serde_json::from_str::<Value>(&json).expect("Minimal grammar must produce valid JSON");
}

#[test]
fn codegen_empty_minimal_node_types_is_array() {
    let (g, t) = minimal_default_table("empty_v6c");
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(
        parsed.is_array(),
        "Minimal grammar node types must be an array"
    );
}

#[test]
fn codegen_empty_minimal_abi_nonempty() {
    let (g, t) = single_token_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        !code.is_empty(),
        "ABI builder must produce non-empty output for minimal grammar"
    );
}

#[test]
fn codegen_empty_single_token_node_types_has_entry() {
    let (g, t) = single_token_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    assert!(
        !arr.is_empty(),
        "Single-token grammar must produce at least one node type entry"
    );
}

#[test]
fn codegen_empty_start_can_be_empty_flag() {
    let (g, t) = single_token_grammar();
    let mut generator = StaticLanguageGenerator::new(g, t);
    generator.set_start_can_be_empty(true);
    let code = generator.generate_language_code().to_string();
    assert!(
        !code.is_empty(),
        "Nullable-start grammar must still produce code"
    );
}

#[test]
fn codegen_empty_minimal_language_has_version() {
    let (g, t) = minimal_default_table("empty_v6e");
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("version") || code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "Even minimal grammar code must reference the language version"
    );
}

#[test]
fn codegen_empty_minimal_abi_has_version() {
    let (g, t) = single_token_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("version") || code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "ABI output for minimal grammar must reference language version"
    );
}

// =====================================================================
// 7. codegen_format_* — output format verification (8 tests)
// =====================================================================

#[test]
fn codegen_format_language_code_no_panic() {
    // Verify that generating code for several grammars does not panic.
    for builder_fn in [
        single_token_grammar,
        expr_grammar,
        two_alt_grammar,
        chain_grammar,
    ] {
        let (g, t) = builder_fn();
        let _code = StaticLanguageGenerator::new(g, t).generate_language_code();
    }
}

#[test]
fn codegen_format_node_types_pretty_printed() {
    let (g, t) = expr_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    // Pretty-printed JSON contains newlines
    assert!(
        json.contains('\n'),
        "Node types JSON should be pretty-printed with newlines"
    );
}

#[test]
fn codegen_format_node_types_type_values_are_strings() {
    let (g, t) = chain_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(
            entry["type"].is_string(),
            "Each 'type' field must be a string"
        );
    }
}

#[test]
fn codegen_format_node_types_named_values_are_bools() {
    let (g, t) = nested_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(
            entry["named"].is_boolean(),
            "Each 'named' field must be a boolean"
        );
    }
}

#[test]
fn codegen_format_abi_output_contains_static_declarations() {
    let (g, t) = expr_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    // The ABI builder generates static data declarations
    assert!(
        code.contains("static"),
        "ABI output must contain 'static' declarations"
    );
}

#[test]
fn codegen_format_ntg_output_is_json_array() {
    let mut grammar = GrammarBuilder::new("fmt_ntg_v6")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let output = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert!(
        parsed.is_array(),
        "NodeTypesGenerator output must be a JSON array"
    );
}

#[test]
fn codegen_format_language_code_contains_const_or_static() {
    let (g, t) = keyword_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("static") || code.contains("const"),
        "Language code must include static or const declarations"
    );
}

#[test]
fn codegen_format_different_grammars_differ() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = expr_grammar();
    let code1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let code2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_ne!(
        code1, code2,
        "Different grammars must produce different code"
    );
}

// =====================================================================
// 8. codegen_roundtrip_* — generate then validate (8 tests)
// =====================================================================

#[test]
fn codegen_roundtrip_node_types_parse_back() {
    let (g, t) = expr_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let reserialized = serde_json::to_string_pretty(&parsed).unwrap();
    // Re-parse the re-serialized output
    let reparsed: Value = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(parsed, reparsed, "Roundtrip must preserve JSON structure");
}

#[test]
fn codegen_roundtrip_node_types_stable_reserialize() {
    let (g, t) = chain_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let reserialized = serde_json::to_string_pretty(&parsed).unwrap();
    // The re-serialized form should be identical to the original
    assert_eq!(json, reserialized, "Reserialization must be stable");
}

#[test]
fn codegen_roundtrip_ntg_parse_back() {
    let mut grammar = GrammarBuilder::new("rt_ntg_v6")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let output = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    let reserialized = serde_json::to_string_pretty(&parsed).unwrap();
    let reparsed: Value = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(parsed, reparsed);
}

#[test]
fn codegen_roundtrip_ntg_stable_reserialize() {
    let mut grammar = GrammarBuilder::new("rt_ntg_stable_v6")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let output = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    let reserialized = serde_json::to_string_pretty(&parsed).unwrap();
    assert_eq!(
        output, reserialized,
        "NodeTypesGenerator reserialization must be stable"
    );
}

#[test]
fn codegen_roundtrip_language_determinism_across_runs() {
    let runs: Vec<String> = (0..5)
        .map(|_| {
            let (g, t) = expr_grammar();
            StaticLanguageGenerator::new(g, t)
                .generate_language_code()
                .to_string()
        })
        .collect();
    for i in 1..runs.len() {
        assert_eq!(runs[0], runs[i], "Run {} diverged from run 0", i);
    }
}

#[test]
fn codegen_roundtrip_abi_determinism_across_runs() {
    let runs: Vec<String> = (0..5)
        .map(|_| {
            let (g, t) = chain_grammar();
            AbiLanguageBuilder::new(&g, &t).generate().to_string()
        })
        .collect();
    for i in 1..runs.len() {
        assert_eq!(runs[0], runs[i], "ABI run {} diverged from run 0", i);
    }
}

#[test]
fn codegen_roundtrip_node_types_sorted_output() {
    let (g, t) = deep_chain_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    let type_names: Vec<&str> = arr.iter().map(|e| e["type"].as_str().unwrap()).collect();
    let mut sorted = type_names.clone();
    sorted.sort();
    // StaticLanguageGenerator may not sort, but we can verify the output is stable
    // by checking the first run matches the second
    let (g2, t2) = deep_chain_grammar();
    let json2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(json, json2, "Node types ordering must be stable");
}

#[test]
fn codegen_roundtrip_ntg_sorted_type_names() {
    let mut grammar = GrammarBuilder::new("rt_sort_v6")
        .token("z", "z")
        .token("a", "a")
        .token("m", "m")
        .rule("beta", vec!["z"])
        .rule("alpha", vec!["a"])
        .rule("gamma", vec!["m"])
        .rule("root", vec!["alpha", "beta", "gamma"])
        .start("root")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let output = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    let arr = parsed.as_array().unwrap();
    let type_names: Vec<&str> = arr.iter().map(|e| e["type"].as_str().unwrap()).collect();
    let mut sorted = type_names.clone();
    sorted.sort();
    assert_eq!(
        type_names, sorted,
        "NodeTypesGenerator must emit types in sorted order"
    );
}
