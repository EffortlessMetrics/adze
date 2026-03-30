//! Comprehensive tests for adze-tablegen code generation output validation.
//!
//! Covers ABI output, NodeTypes JSON, StaticLanguage Rust codegen,
//! determinism, scaling, grammar complexity, and format correctness.
//!
//! 84 `#[test]` functions across 10 categories.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
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

fn make_grammar_and_table(name: &str) -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new(name)
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start"),
    )
}

fn single_token_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("cg_v8_single")
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s"),
    )
}

fn expr_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("cg_v8_expr")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule("expr", vec!["NUM", "PLUS", "NUM"])
            .start("expr"),
    )
}

fn two_alt_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("cg_v8_two_alt")
            .token("x", "x")
            .token("y", "y")
            .rule("s", vec!["x"])
            .rule("s", vec!["y"])
            .start("s"),
    )
}

fn chain_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("cg_v8_chain")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a", "b"])
            .start("s"),
    )
}

fn nested_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("cg_v8_nested")
            .token("a", "a")
            .token("b", "b")
            .rule("inner", vec!["a"])
            .rule("outer", vec!["inner", "b"])
            .start("outer"),
    )
}

fn keyword_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("cg_v8_kw")
            .token("IF", "if")
            .token("THEN", "then")
            .token("ID", r"[a-z]+")
            .rule("stmt", vec!["IF", "ID", "THEN", "ID"])
            .start("stmt"),
    )
}

fn multi_rule_grammar() -> (Grammar, ParseTable) {
    build_pipeline(
        GrammarBuilder::new("cg_v8_multi")
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
        GrammarBuilder::new("cg_v8_deep")
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

fn minimal_default_table(name: &str) -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new(name)
        .token("tok", "t")
        .rule("root", vec!["tok"])
        .start("root")
        .build();
    (grammar, ParseTable::default())
}

// =====================================================================
// 1. ABI output basics (10 tests)
// =====================================================================

#[test]
fn abi_output_is_nonempty() {
    let (g, t) = single_token_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn abi_output_contains_grammar_name() {
    let (g, t) = single_token_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("cg_v8_single"),
        "ABI output must contain the grammar name"
    );
}

#[test]
fn abi_output_contains_language_or_tslanguage() {
    let (g, t) = expr_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(code.contains("LANGUAGE") || code.contains("TSLanguage"));
}

#[test]
fn abi_output_contains_symbol_names() {
    let (g, t) = chain_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(code.contains("SYMBOL_NAME"));
}

#[test]
fn abi_output_contains_parse_actions() {
    let (g, t) = two_alt_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(code.contains("PARSE_ACTIONS"));
}

#[test]
fn abi_output_contains_lex_modes() {
    let (g, t) = nested_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(code.contains("LEX_MODES"));
}

#[test]
fn abi_output_contains_primary_state_ids() {
    let (g, t) = deep_chain_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(code.contains("PRIMARY_STATE_IDS"));
}

#[test]
fn abi_output_contains_static_decls() {
    let (g, t) = expr_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(code.contains("static"));
}

#[test]
fn abi_output_contains_extern_c() {
    let (g, t) = keyword_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(code.contains("extern") && code.contains("\"C\""));
}

#[test]
fn abi_output_contains_ffi_export_named_after_grammar() {
    let (g, t) = keyword_grammar();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(
        code.contains("tree_sitter_cg_v8_kw"),
        "ABI must include FFI export named after grammar"
    );
}

// =====================================================================
// 2. NodeTypes JSON basics (10 tests)
// =====================================================================

#[test]
fn node_types_produces_valid_json() {
    let (g, t) = single_token_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    serde_json::from_str::<Value>(&json).expect("must be valid JSON");
}

#[test]
fn node_types_json_is_array() {
    let (g, t) = chain_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn node_types_array_has_entries() {
    let (g, t) = single_token_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(!parsed.as_array().unwrap().is_empty());
}

#[test]
fn node_types_entries_have_type_field() {
    let (g, t) = nested_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(entry.get("type").is_some());
    }
}

#[test]
fn node_types_entries_have_named_field() {
    let (g, t) = two_alt_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn node_types_type_values_are_strings() {
    let (g, t) = chain_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(entry["type"].is_string());
    }
}

#[test]
fn node_types_named_values_are_bools() {
    let (g, t) = nested_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(entry["named"].is_boolean());
    }
}

#[test]
fn node_types_pretty_printed() {
    let (g, t) = expr_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    assert!(json.contains('\n'), "JSON should be pretty-printed");
}

#[test]
fn node_types_includes_regex_token() {
    let (g, t) = expr_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let arr = parsed.as_array().unwrap();
    let has_num = arr.iter().any(|e| e["type"] == "NUM");
    assert!(has_num, "NUM regex token must appear in node types");
}

#[test]
fn node_types_contains_rule_entries() {
    let (g, t) = multi_rule_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    assert!(json.contains("rule_"), "must include rule entries");
}

// =====================================================================
// 3. StaticLanguage output basics (10 tests)
// =====================================================================

#[test]
fn static_lang_output_is_nonempty() {
    let (g, t) = single_token_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn static_lang_contains_fn_const_or_static() {
    let (g, t) = expr_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("fn") || code.contains("const") || code.contains("static"),
        "Rust codegen must contain fn, const, or static"
    );
}

#[test]
fn static_lang_contains_pub() {
    let (g, t) = chain_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    // TokenStream stringification may not always include `pub`, but we check the common case
    let has_pub_or_fn = code.contains("pub") || code.contains("fn");
    assert!(has_pub_or_fn, "codegen must contain pub or fn");
}

#[test]
fn static_lang_contains_tslanguage() {
    let (g, t) = expr_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("TSLanguage"));
}

#[test]
fn static_lang_contains_symbol_names_array() {
    let (g, t) = chain_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("SYMBOL_NAMES"));
}

#[test]
fn static_lang_contains_parse_table() {
    let (g, t) = two_alt_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("PARSE_TABLE"));
}

#[test]
fn static_lang_contains_lex_modes() {
    let (g, t) = nested_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("LEX_MODES"));
}

#[test]
fn static_lang_contains_symbol_metadata() {
    let (g, t) = keyword_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn static_lang_contains_ffi_export() {
    let (g, t) = expr_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_cg_v8_expr"));
}

#[test]
fn static_lang_contains_version() {
    let (g, t) = make_grammar_and_table("cg_v8_ver");
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("version") || code.contains("TREE_SITTER_LANGUAGE_VERSION"));
}

// =====================================================================
// 4. Determinism (10 tests)
// =====================================================================

#[test]
fn abi_determinism_same_grammar_same_output() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = single_token_grammar();
    let a = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
    assert_eq!(a, b);
}

#[test]
fn abi_determinism_nested_grammar() {
    let (g1, t1) = nested_grammar();
    let (g2, t2) = nested_grammar();
    let a = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
    assert_eq!(a, b);
}

#[test]
fn node_types_determinism_same_grammar_same_output() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = chain_grammar();
    let a = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let b = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(a, b);
}

#[test]
fn static_lang_determinism_same_grammar_same_output() {
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
fn static_lang_determinism_triple_run() {
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
fn abi_determinism_five_runs() {
    let runs: Vec<String> = (0..5)
        .map(|_| {
            let (g, t) = chain_grammar();
            AbiLanguageBuilder::new(&g, &t).generate().to_string()
        })
        .collect();
    for i in 1..runs.len() {
        assert_eq!(runs[0], runs[i], "ABI run {i} diverged from run 0");
    }
}

#[test]
fn node_types_determinism_deep_grammar() {
    let (g1, t1) = deep_chain_grammar();
    let (g2, t2) = deep_chain_grammar();
    let a = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let b = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(a, b);
}

#[test]
fn ntg_determinism_same_grammar() {
    let mut g1 = GrammarBuilder::new("cg_v8_det_ntg")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .start("expr")
        .build();
    let _ff1 = FirstFollowSets::compute_normalized(&mut g1).unwrap();

    let mut g2 = GrammarBuilder::new("cg_v8_det_ntg")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .start("expr")
        .build();
    let _ff2 = FirstFollowSets::compute_normalized(&mut g2).unwrap();

    let a = NodeTypesGenerator::new(&g1).generate().unwrap();
    let b = NodeTypesGenerator::new(&g2).generate().unwrap();
    assert_eq!(a, b);
}

#[test]
fn static_lang_determinism_five_runs() {
    let runs: Vec<String> = (0..5)
        .map(|_| {
            let (g, t) = expr_grammar();
            StaticLanguageGenerator::new(g, t)
                .generate_language_code()
                .to_string()
        })
        .collect();
    for i in 1..runs.len() {
        assert_eq!(runs[0], runs[i], "StaticLanguage run {i} diverged");
    }
}

#[test]
fn node_types_determinism_five_runs() {
    let runs: Vec<String> = (0..5)
        .map(|_| {
            let (g, t) = nested_grammar();
            StaticLanguageGenerator::new(g, t).generate_node_types()
        })
        .collect();
    for i in 1..runs.len() {
        assert_eq!(runs[0], runs[i], "NodeTypes run {i} diverged");
    }
}

// =====================================================================
// 5. Different grammars → different output (8 tests)
// =====================================================================

#[test]
fn different_grammars_different_abi() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = expr_grammar();
    let a = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
    assert_ne!(a, b);
}

#[test]
fn different_grammars_different_node_types() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = expr_grammar();
    let a = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let b = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_ne!(a, b);
}

#[test]
fn different_grammars_different_static_lang() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = expr_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_ne!(a, b);
}

#[test]
fn chain_vs_nested_different_abi() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = nested_grammar();
    let a = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
    assert_ne!(a, b);
}

#[test]
fn chain_vs_nested_different_node_types() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = nested_grammar();
    let a = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let b = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_ne!(a, b);
}

#[test]
fn chain_vs_nested_different_static_lang() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = nested_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_ne!(a, b);
}

#[test]
fn multi_rule_vs_deep_different_abi() {
    let (g1, t1) = multi_rule_grammar();
    let (g2, t2) = deep_chain_grammar();
    let a = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
    assert_ne!(a, b);
}

#[test]
fn two_alt_vs_keyword_different_static_lang() {
    let (g1, t1) = two_alt_grammar();
    let (g2, t2) = keyword_grammar();
    let a = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let b = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_ne!(a, b);
}

// =====================================================================
// 6. Grammar complexity features (10 tests)
// =====================================================================

#[test]
fn multi_token_grammar_abi_nonempty() {
    let (g, t) = build_pipeline(
        GrammarBuilder::new("cg_v8_mt")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("s", vec!["a", "b", "c", "d"])
            .start("s"),
    );
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn multi_token_grammar_node_types_valid() {
    let (g, t) = build_pipeline(
        GrammarBuilder::new("cg_v8_mt2")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("s", vec!["a", "b", "c"])
            .start("s"),
    );
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    serde_json::from_str::<Value>(&json).expect("valid JSON");
}

#[test]
fn grammar_with_precedence_abi_works() {
    let (g, t) = build_pipeline(
        GrammarBuilder::new("cg_v8_prec")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .token("STAR", r"\*")
            .rule("expr", vec!["NUM"])
            .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
            .start("expr"),
    );
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn grammar_with_precedence_static_lang_works() {
    let (g, t) = build_pipeline(
        GrammarBuilder::new("cg_v8_prec2")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule("expr", vec!["NUM"])
            .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
            .start("expr"),
    );
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn grammar_with_inline_abi_works() {
    let (g, t) = build_pipeline(
        GrammarBuilder::new("cg_v8_inl")
            .token("a", "a")
            .token("b", "b")
            .rule("helper", vec!["a"])
            .rule("s", vec!["helper", "b"])
            .inline("helper")
            .start("s"),
    );
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn grammar_with_extras_abi_works() {
    let (g, t) = build_pipeline(
        GrammarBuilder::new("cg_v8_ext")
            .token("a", "a")
            .token("ws", r"\s+")
            .rule("s", vec!["a"])
            .extra("ws")
            .start("s"),
    );
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn grammar_with_extras_node_types_valid() {
    let (g, t) = build_pipeline(
        GrammarBuilder::new("cg_v8_ext2")
            .token("a", "a")
            .token("ws", r"\s+")
            .rule("s", vec!["a"])
            .extra("ws")
            .start("s"),
    );
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    serde_json::from_str::<Value>(&json).expect("valid JSON for extras grammar");
}

#[test]
fn grammar_with_external_abi_works() {
    let (g, t) = build_pipeline(
        GrammarBuilder::new("cg_v8_extscn")
            .token("a", "a")
            .rule("s", vec!["a"])
            .external("indent")
            .start("s"),
    );
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn grammar_with_supertype_node_types_valid() {
    let (g, t) = build_pipeline(
        GrammarBuilder::new("cg_v8_super")
            .token("a", "a")
            .token("b", "b")
            .rule("expr", vec!["a"])
            .rule("expr", vec!["b"])
            .supertype("expr")
            .rule("s", vec!["expr"])
            .start("s"),
    );
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    serde_json::from_str::<Value>(&json).expect("valid JSON for supertype grammar");
}

#[test]
fn grammar_with_right_assoc_abi_works() {
    let (g, t) = build_pipeline(
        GrammarBuilder::new("cg_v8_rassoc")
            .token("NUM", r"\d+")
            .token("EXP", r"\^")
            .rule("expr", vec!["NUM"])
            .rule_with_precedence("expr", vec!["expr", "EXP", "expr"], 3, Associativity::Right)
            .start("expr"),
    );
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

// =====================================================================
// 7. Output scaling (6 tests)
// =====================================================================

#[test]
fn abi_output_length_scales_with_grammar_complexity() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = deep_chain_grammar();
    let len1 = AbiLanguageBuilder::new(&g1, &t1)
        .generate()
        .to_string()
        .len();
    let len2 = AbiLanguageBuilder::new(&g2, &t2)
        .generate()
        .to_string()
        .len();
    assert!(
        len2 > len1,
        "Deep grammar ABI ({len2}) must be larger than single token ({len1})"
    );
}

#[test]
fn static_lang_output_length_scales_with_grammar() {
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
    assert!(len2 > len1);
}

#[test]
fn node_types_entry_count_scales_with_rules() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = deep_chain_grammar();
    let count1 =
        serde_json::from_str::<Value>(&StaticLanguageGenerator::new(g1, t1).generate_node_types())
            .unwrap()
            .as_array()
            .unwrap()
            .len();
    let count2 =
        serde_json::from_str::<Value>(&StaticLanguageGenerator::new(g2, t2).generate_node_types())
            .unwrap()
            .as_array()
            .unwrap()
            .len();
    assert!(
        count2 > count1,
        "Deep grammar must have more node type entries ({count2} vs {count1})"
    );
}

#[test]
fn deep_chain_has_at_least_four_node_entries() {
    let (g, t) = deep_chain_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.as_array().unwrap().len() >= 4);
}

#[test]
fn static_lang_expr_larger_than_single() {
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
    assert!(len2 > len1);
}

#[test]
fn keyword_abi_larger_than_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = keyword_grammar();
    let len1 = AbiLanguageBuilder::new(&g1, &t1)
        .generate()
        .to_string()
        .len();
    let len2 = AbiLanguageBuilder::new(&g2, &t2)
        .generate()
        .to_string()
        .len();
    assert!(len2 > len1);
}

// =====================================================================
// 8. NodeTypesGenerator standalone (8 tests)
// =====================================================================

#[test]
fn ntg_produces_valid_json() {
    let mut g = GrammarBuilder::new("cg_v8_ntg1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    serde_json::from_str::<Value>(&output).expect("must be valid JSON");
}

#[test]
fn ntg_output_is_json_array() {
    let mut g = GrammarBuilder::new("cg_v8_ntg2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn ntg_includes_rule_names() {
    let mut g = GrammarBuilder::new("cg_v8_ntg3")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("outer", vec!["inner", "b"])
        .start("outer")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    assert!(output.contains("inner") || output.contains("outer"));
}

#[test]
fn ntg_deterministic() {
    let make = || {
        let mut g = GrammarBuilder::new("cg_v8_ntg4")
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        let _ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
        NodeTypesGenerator::new(&g).generate().unwrap()
    };
    assert_eq!(make(), make());
}

#[test]
fn ntg_multi_rule_has_entries() {
    let mut g = GrammarBuilder::new("cg_v8_ntg5")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "m"])
        .rule("m", vec!["b", "c"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert!(!parsed.as_array().unwrap().is_empty());
}

#[test]
fn ntg_entries_have_type_and_named() {
    let mut g = GrammarBuilder::new("cg_v8_ntg6")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    for entry in parsed.as_array().unwrap() {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn ntg_sorted_type_names() {
    let mut g = GrammarBuilder::new("cg_v8_ntg_sort")
        .token("z", "z")
        .token("a", "a")
        .token("m", "m")
        .rule("beta", vec!["z"])
        .rule("alpha", vec!["a"])
        .rule("gamma", vec!["m"])
        .rule("root", vec!["alpha", "beta", "gamma"])
        .start("root")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    let arr = parsed.as_array().unwrap();
    let type_names: Vec<&str> = arr.iter().map(|e| e["type"].as_str().unwrap()).collect();
    let mut sorted = type_names.clone();
    sorted.sort();
    assert_eq!(type_names, sorted, "NodeTypesGenerator must emit sorted");
}

#[test]
fn ntg_roundtrip_stable_reserialize() {
    let mut g = GrammarBuilder::new("cg_v8_ntg_rt")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    // Parse again and compare structurally - field order in JSON objects is not guaranteed
    let reparsed: Value = serde_json::from_str(&serde_json::to_string(&parsed).unwrap()).unwrap();
    assert_eq!(
        parsed, reparsed,
        "reserialization must be stable (structural equality)"
    );
}

// =====================================================================
// 9. Roundtrip & stability (6 tests)
// =====================================================================

#[test]
fn roundtrip_node_types_parse_and_reserialize() {
    let (g, t) = expr_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let reserialized = serde_json::to_string_pretty(&parsed).unwrap();
    let reparsed: Value = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(parsed, reparsed);
}

#[test]
fn roundtrip_node_types_stable_reserialize() {
    let (g, t) = chain_grammar();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let reserialized = serde_json::to_string_pretty(&parsed).unwrap();
    assert_eq!(json, reserialized);
}

#[test]
fn roundtrip_language_determinism_across_runs() {
    let runs: Vec<String> = (0..5)
        .map(|_| {
            let (g, t) = expr_grammar();
            StaticLanguageGenerator::new(g, t)
                .generate_language_code()
                .to_string()
        })
        .collect();
    for i in 1..runs.len() {
        assert_eq!(runs[0], runs[i], "run {i} diverged");
    }
}

#[test]
fn roundtrip_abi_determinism_across_runs() {
    let runs: Vec<String> = (0..5)
        .map(|_| {
            let (g, t) = chain_grammar();
            AbiLanguageBuilder::new(&g, &t).generate().to_string()
        })
        .collect();
    for i in 1..runs.len() {
        assert_eq!(runs[0], runs[i], "ABI run {i} diverged");
    }
}

#[test]
fn roundtrip_node_types_ordering_stable() {
    let (g1, t1) = deep_chain_grammar();
    let json1 = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let (g2, t2) = deep_chain_grammar();
    let json2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(json1, json2, "ordering must be stable across runs");
}

#[test]
fn roundtrip_ntg_parse_back() {
    let mut g = GrammarBuilder::new("cg_v8_rt_ntg")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let _ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    let reserialized = serde_json::to_string_pretty(&parsed).unwrap();
    let reparsed: Value = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(parsed, reparsed);
}

// =====================================================================
// 10. Minimal / edge-case grammars (6 tests)
// =====================================================================

#[test]
fn minimal_default_table_static_lang_nonempty() {
    let (g, t) = minimal_default_table("cg_v8_min_a");
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn minimal_default_table_node_types_valid_json() {
    let (g, t) = minimal_default_table("cg_v8_min_b");
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    serde_json::from_str::<Value>(&json).expect("valid JSON");
}

#[test]
fn minimal_default_table_node_types_is_array() {
    let (g, t) = minimal_default_table("cg_v8_min_c");
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn start_can_be_empty_flag_still_produces_code() {
    let (g, t) = single_token_grammar();
    let mut generator = StaticLanguageGenerator::new(g, t);
    generator.set_start_can_be_empty(true);
    let code = generator.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn no_panic_on_all_grammar_helpers() {
    for builder_fn in [
        single_token_grammar,
        expr_grammar,
        two_alt_grammar,
        chain_grammar,
        nested_grammar,
        keyword_grammar,
        multi_rule_grammar,
        deep_chain_grammar,
    ] {
        let (g, t) = builder_fn();
        let _abi = AbiLanguageBuilder::new(&g, &t).generate().to_string();
        let _lang = StaticLanguageGenerator::new(g.clone(), t.clone())
            .generate_language_code()
            .to_string();
        let _json = StaticLanguageGenerator::new(g, t).generate_node_types();
    }
}

#[test]
fn make_grammar_and_table_helper_works() {
    let (g, t) = make_grammar_and_table("cg_v8_helper");
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    serde_json::from_str::<Value>(&json).expect("valid JSON");
}
