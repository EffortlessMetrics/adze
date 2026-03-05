//! Advanced v9 tests for `StaticLanguageGenerator` in `adze-tablegen`.
//!
//! 80+ tests covering: generation output, determinism, grammar variants
//! (precedence, extras, externals, inline, alternatives, conflicts),
//! scaling, real parse tables, and edge cases.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, ConflictDeclaration, ConflictResolution, Grammar};
use adze_tablegen::StaticLanguageGenerator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build grammar + real LR(1) parse table from a GrammarBuilder.
fn build_pair(builder: GrammarBuilder) -> (Grammar, ParseTable) {
    let mut g = builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("compute ff");
    let t = build_lr1_automaton(&g, &ff).expect("build automaton");
    (g, t)
}

fn gen_code(grammar: Grammar, table: ParseTable) -> String {
    StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string()
}

fn gen_node_types(grammar: Grammar, table: ParseTable) -> String {
    StaticLanguageGenerator::new(grammar, table).generate_node_types()
}

fn minimal_builder(name: &str) -> GrammarBuilder {
    GrammarBuilder::new(name)
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
}

fn minimal_pair(name: &str) -> (Grammar, ParseTable) {
    build_pair(minimal_builder(name))
}

// ===========================================================================
// 1. Generate → non-empty string
// ===========================================================================

#[test]
fn test_generate_nonempty_minimal() {
    let (g, t) = minimal_pair("sga_v9_nonempty");
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_generate_nonempty_default_table() {
    let g = minimal_builder("sga_v9_nonempty_def").build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

// ===========================================================================
// 2. Output contains "const" or "static"
// ===========================================================================

#[test]
fn test_output_contains_const_or_static() {
    let (g, t) = minimal_pair("sga_v9_conststat");
    let code = gen_code(g, t);
    assert!(
        code.contains("static") || code.contains("const"),
        "output must contain static or const declarations"
    );
}

#[test]
fn test_output_contains_const_or_static_default_table() {
    let g = minimal_builder("sga_v9_conststat2").build();
    let code = gen_code(g, ParseTable::default());
    assert!(code.contains("static") || code.contains("const"));
}

// ===========================================================================
// 3. Output is valid UTF-8
// ===========================================================================

#[test]
fn test_output_valid_utf8() {
    let (g, t) = minimal_pair("sga_v9_utf8");
    let code = gen_code(g, t);
    // gen_code returns String, which is always valid UTF-8.
    // Verify it round-trips through bytes.
    let bytes = code.as_bytes();
    let roundtripped = std::str::from_utf8(bytes).expect("must be valid UTF-8");
    assert_eq!(code, roundtripped);
}

// ===========================================================================
// 4. Deterministic: same grammar → same output
// ===========================================================================

#[test]
fn test_deterministic_code_generation() {
    let make = || {
        let (g, t) = minimal_pair("sga_v9_det");
        gen_code(g, t)
    };
    assert_eq!(make(), make());
}

#[test]
fn test_deterministic_node_types() {
    let make = || {
        let (g, t) = minimal_pair("sga_v9_det_nt");
        gen_node_types(g, t)
    };
    assert_eq!(make(), make());
}

#[test]
fn test_deterministic_multi_token() {
    let make = || {
        let (g, t) = build_pair(
            GrammarBuilder::new("sga_v9_det_mt")
                .token("a", "a")
                .token("b", "b")
                .rule("start", vec!["a", "b"])
                .start("start"),
        );
        gen_code(g, t)
    };
    assert_eq!(make(), make());
}

// ===========================================================================
// 5. Different grammars → different output
// ===========================================================================

#[test]
fn test_different_grammars_different_output() {
    let (g1, t1) = minimal_pair("sga_v9_diff_a");
    let (g2, t2) = build_pair(
        GrammarBuilder::new("sga_v9_diff_b")
            .token("y", "y")
            .token("z", "z")
            .rule("start", vec!["y", "z"])
            .start("start"),
    );
    let code1 = gen_code(g1, t1);
    let code2 = gen_code(g2, t2);
    assert_ne!(code1, code2);
}

#[test]
fn test_different_names_different_output() {
    let (g1, t1) = minimal_pair("sga_v9_name_alpha");
    let (g2, t2) = minimal_pair("sga_v9_name_beta");
    let code1 = gen_code(g1, t1);
    let code2 = gen_code(g2, t2);
    assert_ne!(code1, code2);
}

// ===========================================================================
// 6. Output contains grammar name
// ===========================================================================

#[test]
fn test_output_contains_grammar_name() {
    let (g, t) = minimal_pair("sga_v9_gname");
    let code = gen_code(g, t);
    assert!(code.contains("sga_v9_gname"));
}

#[test]
fn test_output_contains_custom_name() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_custom_parser")
            .token("tok", "t")
            .rule("start", vec!["tok"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(code.contains("sga_v9_custom_parser"));
}

#[test]
fn test_tree_sitter_function_contains_name() {
    let (g, t) = minimal_pair("sga_v9_fnname");
    let code = gen_code(g, t);
    assert!(code.contains("tree_sitter_sga_v9_fnname"));
}

// ===========================================================================
// 7. Output contains state data
// ===========================================================================

#[test]
fn test_output_contains_parse_table_reference() {
    let (g, t) = minimal_pair("sga_v9_state");
    let code = gen_code(g, t);
    assert!(
        code.contains("PARSE_TABLE") || code.contains("parse_table"),
        "output must reference parse table data"
    );
}

#[test]
fn test_output_contains_symbol_names() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_symnames")
            .token("num", r"\d+")
            .token("plus", r"\+")
            .rule("expr", vec!["num"])
            .rule("expr", vec!["expr", "plus", "num"])
            .start("expr"),
    );
    let code = gen_code(g, t);
    assert!(code.contains("SYMBOL_NAMES"));
}

#[test]
fn test_output_contains_symbol_metadata() {
    let (g, t) = minimal_pair("sga_v9_meta");
    let code = gen_code(g, t);
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn test_output_contains_lex_modes() {
    let (g, t) = minimal_pair("sga_v9_lex");
    let code = gen_code(g, t);
    assert!(code.contains("LEX_MODES"));
}

// ===========================================================================
// 8. Output length scales with grammar size
// ===========================================================================

#[test]
fn test_larger_grammar_produces_longer_output() {
    let small = gen_code(
        minimal_builder("sga_v9_small").build(),
        ParseTable::default(),
    );
    let big = gen_code(
        GrammarBuilder::new("sga_v9_big")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .token("e", "e")
            .rule("r1", vec!["a"])
            .rule("r2", vec!["b"])
            .rule("r3", vec!["c"])
            .rule("r4", vec!["d"])
            .rule("r5", vec!["e"])
            .rule("top", vec!["r1"])
            .rule("top", vec!["r2"])
            .rule("top", vec!["r3"])
            .rule("top", vec!["r4"])
            .rule("top", vec!["r5"])
            .start("top")
            .build(),
        ParseTable::default(),
    );
    assert!(
        big.len() > small.len(),
        "big grammar output ({}) must exceed small ({})",
        big.len(),
        small.len()
    );
}

// ===========================================================================
// 9. Minimal grammar → minimal output
// ===========================================================================

#[test]
fn test_minimal_grammar_produces_output() {
    let (g, t) = minimal_pair("sga_v9_min");
    let code = gen_code(g, t);
    assert!(!code.is_empty());
    // Minimal grammar should produce reasonably short output
    assert!(
        code.len() < 100_000,
        "minimal grammar output should be bounded"
    );
}

// ===========================================================================
// 10. Complex grammar → larger output
// ===========================================================================

#[test]
fn test_complex_grammar_larger_than_minimal() {
    let (gmin, tmin) = minimal_pair("sga_v9_complex_min");
    let min_code = gen_code(gmin, tmin);

    let (gbig, tbig) = build_pair(
        GrammarBuilder::new("sga_v9_complex_big")
            .token("id", r"[a-z]+")
            .token("num", r"\d+")
            .token("plus", r"\+")
            .token("star", r"\*")
            .token("lparen", r"\(")
            .token("rparen", r"\)")
            .token("eq", "=")
            .token("semi", ";")
            .rule("expr", vec!["num"])
            .rule("expr", vec!["id"])
            .rule("expr", vec!["expr", "plus", "expr"])
            .rule("expr", vec!["expr", "star", "expr"])
            .rule("expr", vec!["lparen", "expr", "rparen"])
            .rule("stmt", vec!["id", "eq", "expr", "semi"])
            .rule("program", vec!["stmt"])
            .rule("program", vec!["program", "stmt"])
            .start("program"),
    );
    let big_code = gen_code(gbig, tbig);
    assert!(big_code.len() > min_code.len());
}

// ===========================================================================
// 11. Grammar with precedence → output
// ===========================================================================

#[test]
fn test_precedence_left_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_prec_l")
            .token("num", r"\d+")
            .token("plus", r"\+")
            .rule("expr", vec!["num"])
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .start("expr"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_precedence_right_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_prec_r")
            .token("num", r"\d+")
            .token("caret", r"\^")
            .rule("expr", vec!["num"])
            .rule_with_precedence(
                "expr",
                vec!["expr", "caret", "expr"],
                1,
                Associativity::Right,
            )
            .start("expr"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_precedence_mixed_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_prec_mix")
            .token("num", r"\d+")
            .token("plus", r"\+")
            .token("star", r"\*")
            .rule("expr", vec!["num"])
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
            .start("expr"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
    assert!(code.contains("sga_v9_prec_mix"));
}

#[test]
fn test_precedence_deterministic() {
    let make = || {
        let (g, t) = build_pair(
            GrammarBuilder::new("sga_v9_prec_det")
                .token("num", r"\d+")
                .token("plus", r"\+")
                .rule("expr", vec!["num"])
                .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
                .start("expr"),
        );
        gen_code(g, t)
    };
    assert_eq!(make(), make());
}

// ===========================================================================
// 12. Grammar with extras → output
// ===========================================================================

#[test]
fn test_extras_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_extras")
            .token("id", r"[a-z]+")
            .token("ws", r"[ \t]+")
            .extra("ws")
            .rule("start", vec!["id"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_extras_code_contains_name() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_extras_name")
            .token("id", r"[a-z]+")
            .token("ws", r"[ \t]+")
            .extra("ws")
            .rule("start", vec!["id"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(code.contains("sga_v9_extras_name"));
}

#[test]
fn test_extras_node_types_valid() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_extras_nt")
            .token("id", r"[a-z]+")
            .token("ws", r"[ \t]+")
            .extra("ws")
            .rule("start", vec!["id"])
            .start("start"),
    );
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 13. Grammar with externals → output
// ===========================================================================

#[test]
fn test_externals_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_ext")
            .token("id", r"[a-z]+")
            .external("indent")
            .external("dedent")
            .rule("start", vec!["id"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_externals_node_types_contain_names() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_ext_nt")
            .token("id", r"[a-z]+")
            .external("indent")
            .external("dedent")
            .rule("start", vec!["id"])
            .start("start"),
    );
    let json = gen_node_types(g, t);
    assert!(json.contains("\"indent\""));
    assert!(json.contains("\"dedent\""));
}

#[test]
fn test_three_externals_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_ext3")
            .token("a", "a")
            .external("ext_one")
            .external("ext_two")
            .external("ext_three")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

// ===========================================================================
// 14. Grammar with inline → output
// ===========================================================================

#[test]
fn test_inline_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_inline")
            .token("num", r"\d+")
            .token("plus", r"\+")
            .inline("term")
            .rule("term", vec!["num"])
            .rule("expr", vec!["term"])
            .rule("expr", vec!["expr", "plus", "term"])
            .start("expr"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_inline_code_contains_name() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_inline_name")
            .token("num", r"\d+")
            .inline("atom")
            .rule("atom", vec!["num"])
            .rule("start", vec!["atom"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(code.contains("sga_v9_inline_name"));
}

#[test]
fn test_inline_deterministic() {
    let make = || {
        let (g, t) = build_pair(
            GrammarBuilder::new("sga_v9_inline_det")
                .token("num", r"\d+")
                .inline("atom")
                .rule("atom", vec!["num"])
                .rule("start", vec!["atom"])
                .start("start"),
        );
        gen_code(g, t)
    };
    assert_eq!(make(), make());
}

// ===========================================================================
// 15. Grammar with alternatives → output
// ===========================================================================

#[test]
fn test_alternatives_two() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_alt2")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_alternatives_five() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_alt5")
            .token("t1", "1")
            .token("t2", "2")
            .token("t3", "3")
            .token("t4", "4")
            .token("t5", "5")
            .rule("start", vec!["t1"])
            .rule("start", vec!["t2"])
            .rule("start", vec!["t3"])
            .rule("start", vec!["t4"])
            .rule("start", vec!["t5"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
    assert!(code.contains("sga_v9_alt5"));
}

#[test]
fn test_alternatives_node_types_valid() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_alt_nt")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .rule("start", vec!["c"])
            .start("start"),
    );
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 16. Grammar with conflicts declared → output
// ===========================================================================

#[test]
fn test_conflict_declared_generates() {
    let mut g = GrammarBuilder::new("sga_v9_conflict")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .rule("start", vec!["id"])
        .rule("start", vec!["num"])
        .start("start")
        .build();
    let id_sym = *g
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "start")
        .unwrap()
        .0;
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![id_sym],
        resolution: ConflictResolution::GLR,
    });
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn test_conflict_precedence_resolution_generates() {
    let mut g = GrammarBuilder::new("sga_v9_conflict_prec")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .start("expr")
        .build();
    let expr_sym = *g
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "expr")
        .unwrap()
        .0;
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![expr_sym],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    });
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

// ===========================================================================
// 17. Output contains token references
// ===========================================================================

#[test]
fn test_output_contains_token_info() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_tok_ref")
            .token("num", r"\d+")
            .token("id", r"[a-z]+")
            .rule("start", vec!["num"])
            .rule("start", vec!["id"])
            .start("start"),
    );
    let code = gen_code(g, t);
    // The generated code should reference symbol names, which include token names
    assert!(code.contains("SYMBOL_NAMES"));
}

#[test]
fn test_output_with_many_tokens_has_symbol_names() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_manytok")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(code.contains("SYMBOL_NAMES"));
}

// ===========================================================================
// 18. Output contains symbol count data
// ===========================================================================

#[test]
fn test_output_contains_symbol_count() {
    let (g, t) = minimal_pair("sga_v9_symcount");
    let code = gen_code(g, t);
    assert!(
        code.contains("symbol_count") || code.contains("SYMBOL_COUNT"),
        "output must reference symbol count"
    );
}

#[test]
fn test_real_table_has_nonzero_symbol_count() {
    let (_, t) = minimal_pair("sga_v9_symcount2");
    assert!(t.symbol_count > 0);
}

// ===========================================================================
// 19. Grammar with 1 rule
// ===========================================================================

#[test]
fn test_one_rule_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_one_rule")
            .token("tok", "t")
            .rule("start", vec!["tok"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_one_rule_has_tslanguage() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_one_rule_ts")
            .token("tok", "t")
            .rule("start", vec!["tok"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(code.contains("TSLanguage"));
}

#[test]
fn test_one_rule_node_types_valid() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_one_rule_nt")
            .token("tok", "t")
            .rule("start", vec!["tok"])
            .start("start"),
    );
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 20. Grammar with 10 rules
// ===========================================================================

#[test]
fn test_ten_rules_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_ten_rules")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .token("e", "e")
            .rule("r1", vec!["a"])
            .rule("r2", vec!["b"])
            .rule("r3", vec!["c"])
            .rule("r4", vec!["d"])
            .rule("r5", vec!["e"])
            .rule("r6", vec!["a", "b"])
            .rule("r7", vec!["c", "d"])
            .rule("r8", vec!["r1"])
            .rule("r9", vec!["r2"])
            .rule("top", vec!["r8"])
            .rule("top", vec!["r9"])
            .start("top"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_ten_rules_has_name() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_ten_name")
            .token("a", "a")
            .token("b", "b")
            .rule("r1", vec!["a"])
            .rule("r2", vec!["b"])
            .rule("r3", vec!["r1"])
            .rule("r4", vec!["r2"])
            .rule("r5", vec!["r3"])
            .rule("r6", vec!["r4"])
            .rule("r7", vec!["r5"])
            .rule("r8", vec!["r6"])
            .rule("r9", vec!["r7"])
            .rule("top", vec!["r8"])
            .start("top"),
    );
    let code = gen_code(g, t);
    assert!(code.contains("sga_v9_ten_name"));
}

#[test]
fn test_ten_rules_node_types_nonempty() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_ten_nt")
            .token("a", "a")
            .token("b", "b")
            .rule("r1", vec!["a"])
            .rule("r2", vec!["b"])
            .rule("r3", vec!["r1"])
            .rule("r4", vec!["r2"])
            .rule("r5", vec!["r3"])
            .rule("r6", vec!["r4"])
            .rule("r7", vec!["r5"])
            .rule("r8", vec!["r6"])
            .rule("r9", vec!["r7"])
            .rule("top", vec!["r8"])
            .start("top"),
    );
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(!v.as_array().unwrap().is_empty());
}

// ===========================================================================
// 21-30. Additional coverage: construction, accessors, edge cases
// ===========================================================================

#[test]
fn test_construction_preserves_grammar_name() {
    let (g, t) = minimal_pair("sga_v9_ctor");
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.grammar.name, "sga_v9_ctor");
}

#[test]
fn test_construction_defaults_start_can_be_empty() {
    let (g, t) = minimal_pair("sga_v9_empty_def");
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn test_construction_defaults_compressed_none() {
    let (g, t) = minimal_pair("sga_v9_comp_none");
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(slg.compressed_tables.is_none());
}

#[test]
fn test_set_start_can_be_empty_roundtrip() {
    let (g, t) = minimal_pair("sga_v9_empty_rt");
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
    slg.set_start_can_be_empty(false);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn test_set_start_can_be_empty_still_generates() {
    let (g, t) = minimal_pair("sga_v9_empty_gen");
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_construction_preserves_state_count() {
    let (g, t) = minimal_pair("sga_v9_statecnt");
    let expected = t.state_count;
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.parse_table.state_count, expected);
}

#[test]
fn test_real_table_has_nonzero_state_count() {
    let (_, t) = minimal_pair("sga_v9_statecnt2");
    assert!(t.state_count > 0);
}

#[test]
fn test_real_table_has_nonzero_token_count() {
    let (_, t) = build_pair(
        GrammarBuilder::new("sga_v9_tokcnt")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(t.token_count > 0);
}

#[test]
fn test_empty_grammar_generates() {
    let g = Grammar::new("sga_v9_empty".to_string());
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn test_empty_grammar_node_types_valid_json() {
    let g = Grammar::new("sga_v9_empty_nt".to_string());
    let json = gen_node_types(g, ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 31-40. Node types JSON structure
// ===========================================================================

#[test]
fn test_node_types_entries_have_type_field() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_type_field")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(entry.get("type").is_some());
    }
}

#[test]
fn test_node_types_entries_have_named_field() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_named_field")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn test_node_types_contains_rule_entries() {
    let (g, t) = minimal_pair("sga_v9_rule_ent");
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn test_external_tokens_in_node_types() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_ext_in_nt")
            .token("a", "a")
            .external("my_ext_tok")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let json = gen_node_types(g, t);
    assert!(json.contains("\"my_ext_tok\""));
}

#[test]
fn test_hidden_externals_excluded() {
    let g = GrammarBuilder::new("sga_v9_hid_ext")
        .token("a", "a")
        .external("_hidden_scan")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let json = gen_node_types(g, ParseTable::default());
    assert!(!json.contains("\"_hidden_scan\""));
}

// ===========================================================================
// 41-50. Recursive and chain grammars
// ===========================================================================

#[test]
fn test_recursive_list_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_rec_list")
            .token("item", r"[a-z]+")
            .token("comma", ",")
            .rule("list", vec!["item"])
            .rule("list", vec!["list", "comma", "item"])
            .start("list"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_recursive_list_deterministic() {
    let make = || {
        let (g, t) = build_pair(
            GrammarBuilder::new("sga_v9_rec_det")
                .token("item", r"[a-z]+")
                .token("comma", ",")
                .rule("list", vec!["item"])
                .rule("list", vec!["list", "comma", "item"])
                .start("list"),
        );
        gen_code(g, t)
    };
    assert_eq!(make(), make());
}

#[test]
fn test_chain_grammar_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_chain")
            .token("leaf", "l")
            .rule("a", vec!["leaf"])
            .rule("b", vec!["a"])
            .rule("c", vec!["b"])
            .rule("top", vec!["c"])
            .start("top"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_long_chain_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_longchain")
            .token("leaf", "l")
            .rule("a", vec!["leaf"])
            .rule("b", vec!["a"])
            .rule("c", vec!["b"])
            .rule("d", vec!["c"])
            .rule("e", vec!["d"])
            .rule("f", vec!["e"])
            .rule("top", vec!["f"])
            .start("top"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_nested_parens_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_nested")
            .token("lp", "(")
            .token("rp", ")")
            .token("x", "x")
            .rule("atom", vec!["x"])
            .rule("atom", vec!["lp", "expr", "rp"])
            .rule("expr", vec!["atom"])
            .start("expr"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

// ===========================================================================
// 51-60. Output structure validation
// ===========================================================================

#[test]
fn test_output_contains_tslanguage() {
    let (g, t) = minimal_pair("sga_v9_tslang");
    let code = gen_code(g, t);
    assert!(code.contains("TSLanguage"));
}

#[test]
fn test_output_contains_fn_keyword() {
    let (g, t) = minimal_pair("sga_v9_fnkw");
    let code = gen_code(g, t);
    assert!(code.contains("fn"));
}

#[test]
fn test_output_contains_extern_c() {
    let (g, t) = minimal_pair("sga_v9_externc");
    let code = gen_code(g, t);
    assert!(code.contains("extern \"C\"") || code.contains("extern\"C\""));
}

#[test]
fn test_output_contains_language_version() {
    let (g, t) = minimal_pair("sga_v9_langver");
    let code = gen_code(g, t);
    assert!(code.contains("LANGUAGE_VERSION") || code.contains("TREE_SITTER_LANGUAGE_VERSION"));
}

#[test]
fn test_output_contains_tree_sitter_prefix() {
    let (g, t) = minimal_pair("sga_v9_ts_prefix");
    let code = gen_code(g, t);
    assert!(code.contains("tree_sitter_"));
}

#[test]
fn test_output_contains_language_word() {
    let (g, t) = minimal_pair("sga_v9_langword");
    let code = gen_code(g, t);
    assert!(code.contains("language"));
}

#[test]
fn test_multi_token_output_larger_than_single() {
    let small = gen_code(
        minimal_builder("sga_v9_scale_s").build(),
        ParseTable::default(),
    );
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_scale_l")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start"),
    );
    let big = gen_code(g, t);
    assert!(big.len() > small.len());
}

#[test]
fn test_different_structures_different_output() {
    let (g1, t1) = build_pair(
        GrammarBuilder::new("sga_v9_struct_a")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let (g2, t2) = build_pair(
        GrammarBuilder::new("sga_v9_struct_b")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    assert_ne!(gen_code(g1, t1), gen_code(g2, t2));
}

#[test]
fn test_node_types_deterministic_recursive() {
    let make = || {
        let (g, t) = build_pair(
            GrammarBuilder::new("sga_v9_nt_det_rec")
                .token("item", r"[a-z]+")
                .token("comma", ",")
                .rule("list", vec!["item"])
                .rule("list", vec!["list", "comma", "item"])
                .start("list"),
        );
        gen_node_types(g, t)
    };
    assert_eq!(make(), make());
}

#[test]
fn test_node_types_different_grammars_differ() {
    let (g1, t1) = minimal_pair("sga_v9_ntdiff_a");
    let (g2, t2) = build_pair(
        GrammarBuilder::new("sga_v9_ntdiff_b")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    assert_ne!(gen_node_types(g1, t1), gen_node_types(g2, t2));
}

// ===========================================================================
// 61-70. Supertype, keyword, diamond topologies
// ===========================================================================

#[test]
fn test_supertype_generates() {
    let mut g = GrammarBuilder::new("sga_v9_super")
        .token("num", r"\d+")
        .token("id", r"[a-z]+")
        .rule("literal", vec!["num"])
        .rule("literal", vec!["id"])
        .rule("program", vec!["literal"])
        .start("program")
        .build();
    let lit_id = *g
        .rules
        .keys()
        .find(|id| g.rule_names.get(*id).map(|n| n.as_str()) == Some("literal"))
        .unwrap();
    g.supertypes.push(lit_id);
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn test_supertype_node_types_has_subtypes() {
    let mut g = GrammarBuilder::new("sga_v9_super_nt")
        .token("num", r"\d+")
        .token("id", r"[a-z]+")
        .rule("literal", vec!["num"])
        .rule("literal", vec!["id"])
        .rule("program", vec!["literal"])
        .start("program")
        .build();
    let lit_id = *g
        .rules
        .keys()
        .find(|id| g.rule_names.get(*id).map(|n| n.as_str()) == Some("literal"))
        .unwrap();
    g.supertypes.push(lit_id);
    let json = gen_node_types(g, ParseTable::default());
    assert!(json.contains("subtypes"));
}

#[test]
fn test_keyword_rich_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_kw")
            .token("if_kw", "if")
            .token("else_kw", "else")
            .token("id", r"[a-z]+")
            .rule("start", vec!["if_kw", "id"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_diamond_grammar_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_diamond")
            .token("x", "x")
            .rule("bottom", vec!["x"])
            .rule("left_node", vec!["bottom"])
            .rule("right_node", vec!["bottom"])
            .rule("top", vec!["left_node"])
            .rule("top", vec!["right_node"])
            .start("top"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_diamond_node_types_valid() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_diamond_nt")
            .token("x", "x")
            .rule("bottom", vec!["x"])
            .rule("left_node", vec!["bottom"])
            .rule("right_node", vec!["bottom"])
            .rule("top", vec!["left_node"])
            .rule("top", vec!["right_node"])
            .start("top"),
    );
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

#[test]
fn test_wide_alt_code_nonempty() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_wide")
            .token("t1", "1")
            .token("t2", "2")
            .token("t3", "3")
            .token("t4", "4")
            .token("t5", "5")
            .token("t6", "6")
            .rule("start", vec!["t1"])
            .rule("start", vec!["t2"])
            .rule("start", vec!["t3"])
            .rule("start", vec!["t4"])
            .rule("start", vec!["t5"])
            .rule("start", vec!["t6"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

// ===========================================================================
// 71-80. Misc edge cases & combined features
// ===========================================================================

#[test]
fn test_extras_plus_externals_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_ext_extras")
            .token("id", r"[a-z]+")
            .token("ws", r"[ \t]+")
            .extra("ws")
            .external("newline")
            .rule("start", vec!["id"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_extras_plus_externals_node_types() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_ext_extras_nt")
            .token("id", r"[a-z]+")
            .token("ws", r"[ \t]+")
            .extra("ws")
            .external("newline")
            .rule("start", vec!["id"])
            .start("start"),
    );
    let json = gen_node_types(g, t);
    assert!(json.contains("\"newline\""));
}

#[test]
fn test_inline_plus_precedence_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_inl_prec")
            .token("num", r"\d+")
            .token("plus", r"\+")
            .inline("atom")
            .rule("atom", vec!["num"])
            .rule("expr", vec!["atom"])
            .rule_with_precedence("expr", vec!["expr", "plus", "atom"], 1, Associativity::Left)
            .start("expr"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_large_grammar_with_default_table() {
    let mut builder = GrammarBuilder::new("sga_v9_large");
    for i in 0..20 {
        builder = builder.token(&format!("tok_{i}"), &format!("t{i}"));
    }
    for i in 0..10 {
        let tok = format!("tok_{i}");
        let rule_name = format!("rule_{i}");
        builder = builder.rule(&rule_name, vec![Box::leak(tok.into_boxed_str())]);
    }
    builder = builder
        .rule("top", vec!["rule_0"])
        .rule("top", vec!["rule_1"])
        .start("top");
    let g = builder.build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn test_large_grammar_node_types_has_entries() {
    let mut builder = GrammarBuilder::new("sga_v9_large_nt");
    for i in 0..15 {
        builder = builder.token(&format!("tok_{i}"), &format!("t{i}"));
    }
    for i in 0..10 {
        let tok = format!("tok_{i}");
        let rule_name = format!("rule_{i}");
        builder = builder.rule(&rule_name, vec![Box::leak(tok.into_boxed_str())]);
    }
    builder = builder
        .rule("top", vec!["rule_0"])
        .rule("top", vec!["rule_1"])
        .start("top");
    let json = gen_node_types(builder.build(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.as_array().unwrap().len() >= 5);
}

#[test]
fn test_seq_grammar_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_seq")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_seq_grammar_deterministic() {
    let make = || {
        let (g, t) = build_pair(
            GrammarBuilder::new("sga_v9_seq_det")
                .token("a", "a")
                .token("b", "b")
                .rule("start", vec!["a", "b"])
                .start("start"),
        );
        gen_code(g, t)
    };
    assert_eq!(make(), make());
}

#[test]
fn test_name_with_underscores_preserved() {
    let (g, t) = minimal_pair("sga_v9_a_b_c_d");
    let code = gen_code(g, t);
    assert!(code.contains("sga_v9_a_b_c_d"));
}

#[test]
fn test_code_from_all_features_combined() {
    let mut g = GrammarBuilder::new("sga_v9_all_feat")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("ws", r"[ \t]+")
        .extra("ws")
        .external("indent")
        .inline("atom")
        .rule("atom", vec!["num"])
        .rule("atom", vec!["id"])
        .rule("expr", vec!["atom"])
        .rule_with_precedence("expr", vec!["expr", "plus", "atom"], 1, Associativity::Left)
        .rule("program", vec!["expr"])
        .start("program")
        .build();
    let prog_id = *g
        .rules
        .keys()
        .find(|id| g.rule_names.get(*id).map(|n| n.as_str()) == Some("expr"))
        .unwrap();
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![prog_id],
        resolution: ConflictResolution::GLR,
    });
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
    assert!(code.contains("sga_v9_all_feat"));
}

// ===========================================================================
// 81-85. Bonus coverage
// ===========================================================================

#[test]
fn test_multiple_determinism_runs() {
    let results: Vec<String> = (0..5)
        .map(|_| {
            let (g, t) = minimal_pair("sga_v9_multi_det");
            gen_code(g, t)
        })
        .collect();
    for r in &results[1..] {
        assert_eq!(&results[0], r);
    }
}

#[test]
fn test_precedence_declaration_generates() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_prec_decl")
            .token("num", r"\d+")
            .token("plus", r"\+")
            .token("star", r"\*")
            .precedence(1, Associativity::Left, vec!["plus"])
            .precedence(2, Associativity::Left, vec!["star"])
            .rule("expr", vec!["num"])
            .rule("expr", vec!["expr", "plus", "expr"])
            .rule("expr", vec!["expr", "star", "expr"])
            .start("expr"),
    );
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_node_types_all_named_true() {
    let (g, t) = build_pair(
        GrammarBuilder::new("sga_v9_all_named")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        let named = entry.get("named").and_then(|n| n.as_bool());
        assert_eq!(named, Some(true));
    }
}

#[test]
fn test_hidden_rule_excluded_from_node_types() {
    let g = GrammarBuilder::new("sga_v9_hidden")
        .token("a", "a")
        .rule("_hidden", vec!["a"])
        .rule("visible", vec!["_hidden"])
        .start("visible")
        .build();
    let json = gen_node_types(g, ParseTable::default());
    assert!(!json.contains("\"_hidden\""));
}

#[test]
fn test_generate_language_code_returns_tokenstream() {
    let (g, t) = minimal_pair("sga_v9_tokstream");
    let slg = StaticLanguageGenerator::new(g, t);
    let ts = slg.generate_language_code();
    // TokenStream.to_string() produces valid Rust-ish text
    let text = ts.to_string();
    assert!(!text.is_empty());
}
