//! Comprehensive tests for `StaticLanguageGenerator` output validation.
//!
//! Covers: single/multi-token grammars, precedence, inline, extras,
//! determinism, scaling, structural content checks, edge cases.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::StaticLanguageGenerator;

// ===========================================================================
// Helper
// ===========================================================================

fn generate_code(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> String {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string()
}

/// Helper with precedence rules.
fn generate_code_with_prec(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    prec_rules: &[(&str, Vec<&str>, i16, Associativity)],
    start: &str,
) -> String {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    for (lhs, rhs, prec, assoc) in prec_rules {
        b = b.rule_with_precedence(lhs, rhs.clone(), *prec, *assoc);
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string()
}

/// Helper with inline rules.
fn generate_code_with_inline(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    inlines: &[&str],
    start: &str,
) -> String {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    for &inline in inlines {
        b = b.inline(inline);
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string()
}

/// Helper with extras.
fn generate_code_with_extras(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    extras: &[&str],
    start: &str,
) -> String {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    for &extra in extras {
        b = b.extra(extra);
    }
    let g = b.start(start).build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string()
}

// ===========================================================================
// 1–4. Single-token grammar basics
// ===========================================================================

#[test]
fn lg_v9_single_token_nonempty_output() {
    let code = generate_code(
        "lg_v9_single",
        &[("num", r"\d+")],
        &[("expr", vec!["num"])],
        "expr",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_output_contains_fn_or_const_or_static() {
    let code = generate_code(
        "lg_v9_fncheck",
        &[("num", r"\d+")],
        &[("expr", vec!["num"])],
        "expr",
    );
    assert!(code.contains("fn") || code.contains("const") || code.contains("static"));
}

#[test]
fn lg_v9_output_contains_grammar_name() {
    let code = generate_code(
        "lg_v9_namecheck",
        &[("num", r"\d+")],
        &[("expr", vec!["num"])],
        "expr",
    );
    assert!(code.contains("lg_v9_namecheck"));
}

#[test]
fn lg_v9_output_is_valid_utf8() {
    let code = generate_code(
        "lg_v9_utf8",
        &[("num", r"\d+")],
        &[("expr", vec!["num"])],
        "expr",
    );
    // If we got a String, it is already valid UTF-8.
    // Double-check by round-tripping through bytes.
    let bytes = code.as_bytes();
    assert!(std::str::from_utf8(bytes).is_ok());
}

// ===========================================================================
// 5. Symbol table references
// ===========================================================================

#[test]
fn lg_v9_output_contains_symbol_references() {
    let code = generate_code(
        "lg_v9_symref",
        &[("num", r"\d+")],
        &[("expr", vec!["num"])],
        "expr",
    );
    // Generated code should reference symbol names
    assert!(code.contains("num") || code.contains("expr") || code.contains("SYMBOL"));
}

// ===========================================================================
// 6–7. Multi-token grammars
// ===========================================================================

#[test]
fn lg_v9_two_token_grammar() {
    let code = generate_code(
        "lg_v9_two_tok",
        &[("num", r"\d+"), ("id", r"[a-z]+")],
        &[("expr", vec!["num"]), ("expr", vec!["id"])],
        "expr",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_five_token_grammar() {
    let code = generate_code(
        "lg_v9_five_tok",
        &[
            ("num", r"\d+"),
            ("plus", r"\+"),
            ("minus", r"-"),
            ("star", r"\*"),
            ("slash", r"\/"),
        ],
        &[
            ("expr", vec!["num"]),
            ("expr", vec!["expr", "plus", "expr"]),
            ("expr", vec!["expr", "minus", "expr"]),
            ("expr", vec!["expr", "star", "expr"]),
            ("expr", vec!["expr", "slash", "expr"]),
        ],
        "expr",
    );
    assert!(!code.is_empty());
}

// ===========================================================================
// 8. Precedence
// ===========================================================================

#[test]
fn lg_v9_grammar_with_precedence_produces_output() {
    let code = generate_code_with_prec(
        "lg_v9_prec",
        &[("num", r"\d+"), ("plus", r"\+"), ("star", r"\*")],
        &[("expr", vec!["num"])],
        &[
            ("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left),
            ("expr", vec!["expr", "star", "expr"], 2, Associativity::Left),
        ],
        "expr",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_right_associativity_produces_output() {
    let code = generate_code_with_prec(
        "lg_v9_rassoc",
        &[("num", r"\d+"), ("caret", r"\^")],
        &[("expr", vec!["num"])],
        &[(
            "expr",
            vec!["expr", "caret", "expr"],
            3,
            Associativity::Right,
        )],
        "expr",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_none_associativity_produces_output() {
    let code = generate_code_with_prec(
        "lg_v9_nassoc",
        &[("num", r"\d+"), ("eq", "=")],
        &[("expr", vec!["num"])],
        &[("expr", vec!["expr", "eq", "expr"], 1, Associativity::None)],
        "expr",
    );
    assert!(!code.is_empty());
}

// ===========================================================================
// 9. Inline rules
// ===========================================================================

#[test]
fn lg_v9_grammar_with_inline_produces_output() {
    let code = generate_code_with_inline(
        "lg_v9_inline",
        &[("num", r"\d+"), ("id", r"[a-z]+")],
        &[
            ("expr", vec!["atom"]),
            ("atom", vec!["num"]),
            ("atom", vec!["id"]),
        ],
        &["atom"],
        "expr",
    );
    assert!(!code.is_empty());
}

// ===========================================================================
// 10. Extras
// ===========================================================================

#[test]
fn lg_v9_grammar_with_extras_produces_output() {
    let code = generate_code_with_extras(
        "lg_v9_extras",
        &[("num", r"\d+"), ("ws", r"\s+")],
        &[("expr", vec!["num"])],
        &["ws"],
        "expr",
    );
    assert!(!code.is_empty());
}

// ===========================================================================
// 11. Determinism
// ===========================================================================

#[test]
fn lg_v9_determinism_same_grammar_same_output() {
    let code1 = generate_code(
        "lg_v9_determ",
        &[("num", r"\d+")],
        &[("expr", vec!["num"])],
        "expr",
    );
    let code2 = generate_code(
        "lg_v9_determ",
        &[("num", r"\d+")],
        &[("expr", vec!["num"])],
        "expr",
    );
    assert_eq!(code1, code2);
}

#[test]
fn lg_v9_determinism_repeated_calls_stable() {
    let results: Vec<String> = (0..5)
        .map(|_| {
            generate_code(
                "lg_v9_detloop",
                &[("x", "x")],
                &[("start", vec!["x"])],
                "start",
            )
        })
        .collect();
    for r in &results[1..] {
        assert_eq!(&results[0], r);
    }
}

// ===========================================================================
// 12. Different grammars → different output
// ===========================================================================

#[test]
fn lg_v9_different_grammars_different_output() {
    let code_a = generate_code(
        "lg_v9_diff_a",
        &[("num", r"\d+")],
        &[("expr", vec!["num"])],
        "expr",
    );
    let code_b = generate_code(
        "lg_v9_diff_b",
        &[("id", r"[a-z]+")],
        &[("stmt", vec!["id"])],
        "stmt",
    );
    assert_ne!(code_a, code_b);
}

#[test]
fn lg_v9_different_token_patterns_different_output() {
    let code_a = generate_code(
        "lg_v9_diffpat_a",
        &[("tok", "a")],
        &[("start", vec!["tok"])],
        "start",
    );
    let code_b = generate_code(
        "lg_v9_diffpat_b",
        &[("tok", "b")],
        &[("start", vec!["tok"])],
        "start",
    );
    assert_ne!(code_a, code_b);
}

// ===========================================================================
// 13. Output length scales with grammar complexity
// ===========================================================================

#[test]
fn lg_v9_output_length_scales_with_tokens() {
    let small = generate_code(
        "lg_v9_scale_s",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    let large = generate_code(
        "lg_v9_scale_l",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[
            ("start", vec!["a"]),
            ("start", vec!["b"]),
            ("start", vec!["c"]),
            ("start", vec!["d"]),
            ("start", vec!["e"]),
        ],
        "start",
    );
    assert!(large.len() > small.len());
}

#[test]
fn lg_v9_output_length_scales_with_rules() {
    let one_rule = generate_code(
        "lg_v9_scale_r1",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a"])],
        "start",
    );
    let many_rules = generate_code(
        "lg_v9_scale_rm",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["a"]),
            ("start", vec!["b"]),
            ("start", vec!["a", "b"]),
            ("start", vec!["b", "a"]),
        ],
        "start",
    );
    assert!(many_rules.len() > one_rule.len());
}

// ===========================================================================
// 14. State count info
// ===========================================================================

#[test]
fn lg_v9_output_contains_state_count_info() {
    let code = generate_code(
        "lg_v9_stcnt",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    // The generated code should embed numeric state count data
    // Either as a constant or within a struct field
    assert!(
        code.contains("state_count")
            || code.contains("STATE_COUNT")
            || code.contains("state")
            || code.contains("STATE")
    );
}

// ===========================================================================
// 15. Complex arithmetic grammar
// ===========================================================================

#[test]
fn lg_v9_complex_arithmetic_grammar() {
    let code = generate_code_with_prec(
        "lg_v9_arith",
        &[
            ("num", r"\d+"),
            ("plus", r"\+"),
            ("minus", r"-"),
            ("star", r"\*"),
            ("slash", r"\/"),
            ("lparen", r"\("),
            ("rparen", r"\)"),
        ],
        &[
            ("expr", vec!["num"]),
            ("expr", vec!["lparen", "expr", "rparen"]),
        ],
        &[
            ("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left),
            (
                "expr",
                vec!["expr", "minus", "expr"],
                1,
                Associativity::Left,
            ),
            ("expr", vec!["expr", "star", "expr"], 2, Associativity::Left),
            (
                "expr",
                vec!["expr", "slash", "expr"],
                2,
                Associativity::Left,
            ),
        ],
        "expr",
    );
    assert!(!code.is_empty());
    assert!(code.len() > 100);
}

#[test]
fn lg_v9_complex_arithmetic_contains_grammar_name() {
    let code = generate_code_with_prec(
        "lg_v9_arith_name",
        &[("num", r"\d+"), ("plus", r"\+"), ("star", r"\*")],
        &[("expr", vec!["num"])],
        &[
            ("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left),
            ("expr", vec!["expr", "star", "expr"], 2, Associativity::Left),
        ],
        "expr",
    );
    assert!(code.contains("lg_v9_arith_name"));
}

// ===========================================================================
// 16. Parse table data
// ===========================================================================

#[test]
fn lg_v9_output_contains_parse_table_data() {
    let code = generate_code(
        "lg_v9_ptdata",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    // Should have some array/table data (numbers, brackets)
    assert!(code.contains('[') || code.contains("table") || code.contains("TABLE"));
}

#[test]
fn lg_v9_output_contains_numeric_data() {
    let code = generate_code(
        "lg_v9_numdata",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    // Generated code should contain numeric constants (symbol IDs, state counts, etc.)
    assert!(code.contains('0') || code.contains('1') || code.contains('2'));
}

// ===========================================================================
// 17. Action table references
// ===========================================================================

#[test]
fn lg_v9_output_contains_action_references() {
    let code = generate_code(
        "lg_v9_actref",
        &[("x", "x"), ("y", "y")],
        &[("start", vec!["x"]), ("start", vec!["y"])],
        "start",
    );
    assert!(
        code.contains("action")
            || code.contains("ACTION")
            || code.contains("parse")
            || code.contains("PARSE")
            || code.contains("shift")
            || code.contains("reduce")
    );
}

// ===========================================================================
// 18. Goto table references
// ===========================================================================

#[test]
fn lg_v9_output_contains_goto_references() {
    let code = generate_code(
        "lg_v9_gotoref",
        &[("x", "x"), ("y", "y")],
        &[
            ("start", vec!["inner"]),
            ("inner", vec!["x"]),
            ("inner", vec!["y"]),
        ],
        "start",
    );
    // Goto tables or state transitions should appear
    assert!(
        code.contains("goto")
            || code.contains("GOTO")
            || code.contains("state")
            || code.contains("STATE")
    );
}

// ===========================================================================
// 19. Various grammar complexities
// ===========================================================================

#[test]
fn lg_v9_single_rule_single_token() {
    let code = generate_code(
        "lg_v9_1r1t",
        &[("a", "a")],
        &[("start", vec!["a"])],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_two_rules_one_nonterminal() {
    let code = generate_code(
        "lg_v9_2r1nt",
        &[("a", "a"), ("b", "b")],
        &[("start", vec!["a"]), ("start", vec!["b"])],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_nested_nonterminals() {
    let code = generate_code(
        "lg_v9_nested",
        &[("x", "x")],
        &[("start", vec!["mid"]), ("mid", vec!["x"])],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_deeply_nested_chain() {
    let code = generate_code(
        "lg_v9_deep",
        &[("x", "x")],
        &[
            ("start", vec!["a"]),
            ("a", vec!["b"]),
            ("b", vec!["c"]),
            ("c", vec!["x"]),
        ],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_multiple_nonterminals_multiple_rules() {
    let code = generate_code(
        "lg_v9_multi_nt",
        &[("num", r"\d+"), ("id", r"[a-z]+"), ("op", r"\+")],
        &[
            ("start", vec!["expr"]),
            ("expr", vec!["num"]),
            ("expr", vec!["id"]),
            ("expr", vec!["expr", "op", "expr"]),
        ],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_three_level_grammar() {
    let code = generate_code(
        "lg_v9_3level",
        &[("num", r"\d+"), ("plus", r"\+"), ("star", r"\*")],
        &[
            ("start", vec!["sum"]),
            ("sum", vec!["product"]),
            ("sum", vec!["sum", "plus", "product"]),
            ("product", vec!["num"]),
            ("product", vec!["product", "star", "num"]),
        ],
        "start",
    );
    assert!(!code.is_empty());
    assert!(code.len() > 100);
}

// ===========================================================================
// 20. Edge case: minimal grammar
// ===========================================================================

#[test]
fn lg_v9_minimal_single_char_token() {
    let code = generate_code("lg_v9_min", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(!code.is_empty());
}

// ===========================================================================
// Additional structural content checks (21–40)
// ===========================================================================

#[test]
fn lg_v9_output_contains_tree_sitter_function() {
    let code = generate_code(
        "lg_v9_tsfn",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    assert!(code.contains("tree_sitter"));
}

#[test]
fn lg_v9_output_contains_language_version() {
    let code = generate_code(
        "lg_v9_langver",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    assert!(code.contains("version") || code.contains("VERSION"));
}

#[test]
fn lg_v9_output_contains_symbol_count() {
    let code = generate_code(
        "lg_v9_symcnt",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    assert!(
        code.contains("symbol_count") || code.contains("SYMBOL_COUNT") || code.contains("symbol")
    );
}

#[test]
fn lg_v9_output_contains_token_count() {
    let code = generate_code(
        "lg_v9_tokcnt",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    assert!(code.contains("token_count") || code.contains("TOKEN_COUNT") || code.contains("token"));
}

#[test]
fn lg_v9_output_contains_field_count() {
    let code = generate_code(
        "lg_v9_fldcnt",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    assert!(code.contains("field_count") || code.contains("FIELD_COUNT") || code.contains("field"));
}

#[test]
fn lg_v9_output_contains_production_id_count() {
    let code = generate_code(
        "lg_v9_prodid",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    assert!(
        code.contains("production_id_count")
            || code.contains("PRODUCTION")
            || code.contains("production")
    );
}

#[test]
fn lg_v9_output_contains_lex_reference() {
    let code = generate_code("lg_v9_lex", &[("x", "x")], &[("start", vec!["x"])], "start");
    assert!(code.contains("lex") || code.contains("LEX"));
}

#[test]
fn lg_v9_output_contains_symbol_names_array() {
    let code = generate_code(
        "lg_v9_snames",
        &[("foo", "foo")],
        &[("start", vec!["foo"])],
        "start",
    );
    // The generator produces symbol name strings
    assert!(code.contains("foo") || code.contains("SYMBOL_NAMES") || code.contains("symbol_names"));
}

#[test]
fn lg_v9_output_contains_symbol_metadata_reference() {
    let code = generate_code(
        "lg_v9_smeta",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    assert!(
        code.contains("metadata")
            || code.contains("METADATA")
            || code.contains("visible")
            || code.contains("named")
    );
}

#[test]
fn lg_v9_output_contains_external_token_count() {
    let code = generate_code(
        "lg_v9_extcnt",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    assert!(
        code.contains("external_token_count")
            || code.contains("EXTERNAL")
            || code.contains("external")
    );
}

// ===========================================================================
// Precedence-specific content checks (41–50)
// ===========================================================================

#[test]
fn lg_v9_prec_left_output_nonempty() {
    let code = generate_code_with_prec(
        "lg_v9_prec_l",
        &[("a", "a"), ("op", "+")],
        &[("expr", vec!["a"])],
        &[("expr", vec!["expr", "op", "expr"], 1, Associativity::Left)],
        "expr",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_prec_right_output_nonempty() {
    let code = generate_code_with_prec(
        "lg_v9_prec_r",
        &[("a", "a"), ("op", "^")],
        &[("expr", vec!["a"])],
        &[("expr", vec!["expr", "op", "expr"], 2, Associativity::Right)],
        "expr",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_prec_multiple_levels() {
    let code = generate_code_with_prec(
        "lg_v9_prec_ml",
        &[("n", r"\d+"), ("p", "+"), ("m", "*"), ("e", "^")],
        &[("expr", vec!["n"])],
        &[
            ("expr", vec!["expr", "p", "expr"], 1, Associativity::Left),
            ("expr", vec!["expr", "m", "expr"], 2, Associativity::Left),
            ("expr", vec!["expr", "e", "expr"], 3, Associativity::Right),
        ],
        "expr",
    );
    assert!(!code.is_empty());
    assert!(code.len() > 100);
}

#[test]
fn lg_v9_prec_same_level_different_ops() {
    let code = generate_code_with_prec(
        "lg_v9_prec_samelv",
        &[("n", r"\d+"), ("p", "+"), ("m", "-")],
        &[("expr", vec!["n"])],
        &[
            ("expr", vec!["expr", "p", "expr"], 1, Associativity::Left),
            ("expr", vec!["expr", "m", "expr"], 1, Associativity::Left),
        ],
        "expr",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_prec_output_differs_from_no_prec() {
    let no_prec = generate_code(
        "lg_v9_prec_cmp_np",
        &[("n", r"\d+"), ("p", "+")],
        &[("expr", vec!["n"]), ("expr", vec!["expr", "p", "expr"])],
        "expr",
    );
    let with_prec = generate_code_with_prec(
        "lg_v9_prec_cmp_wp",
        &[("n", r"\d+"), ("p", "+")],
        &[("expr", vec!["n"])],
        &[("expr", vec!["expr", "p", "expr"], 1, Associativity::Left)],
        "expr",
    );
    // They have different grammar names so must differ
    assert_ne!(no_prec, with_prec);
}

#[test]
fn lg_v9_prec_negative_level() {
    let code = generate_code_with_prec(
        "lg_v9_prec_neg",
        &[("a", "a"), ("op", "+")],
        &[("expr", vec!["a"])],
        &[("expr", vec!["expr", "op", "expr"], -1, Associativity::Left)],
        "expr",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_prec_zero_level() {
    let code = generate_code_with_prec(
        "lg_v9_prec_zero",
        &[("a", "a"), ("op", "+")],
        &[("expr", vec!["a"])],
        &[("expr", vec!["expr", "op", "expr"], 0, Associativity::Left)],
        "expr",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_prec_high_level() {
    let code = generate_code_with_prec(
        "lg_v9_prec_hi",
        &[("a", "a"), ("op", "+")],
        &[("expr", vec!["a"])],
        &[("expr", vec!["expr", "op", "expr"], 100, Associativity::Left)],
        "expr",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_prec_determinism() {
    let code1 = generate_code_with_prec(
        "lg_v9_prec_det",
        &[("n", r"\d+"), ("p", "+")],
        &[("expr", vec!["n"])],
        &[("expr", vec!["expr", "p", "expr"], 1, Associativity::Left)],
        "expr",
    );
    let code2 = generate_code_with_prec(
        "lg_v9_prec_det",
        &[("n", r"\d+"), ("p", "+")],
        &[("expr", vec!["n"])],
        &[("expr", vec!["expr", "p", "expr"], 1, Associativity::Left)],
        "expr",
    );
    assert_eq!(code1, code2);
}

#[test]
fn lg_v9_prec_with_parens() {
    let code = generate_code_with_prec(
        "lg_v9_prec_par",
        &[("n", r"\d+"), ("p", "+"), ("lp", "("), ("rp", ")")],
        &[("expr", vec!["n"]), ("expr", vec!["lp", "expr", "rp"])],
        &[("expr", vec!["expr", "p", "expr"], 1, Associativity::Left)],
        "expr",
    );
    assert!(!code.is_empty());
}

// ===========================================================================
// Inline-specific tests (51–55)
// ===========================================================================

#[test]
fn lg_v9_inline_output_nonempty() {
    let code = generate_code_with_inline(
        "lg_v9_inl_ne",
        &[("a", "a"), ("b", "b")],
        &[
            ("start", vec!["helper"]),
            ("helper", vec!["a"]),
            ("helper", vec!["b"]),
        ],
        &["helper"],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_inline_determinism() {
    let code1 = generate_code_with_inline(
        "lg_v9_inl_det",
        &[("a", "a")],
        &[("start", vec!["h"]), ("h", vec!["a"])],
        &["h"],
        "start",
    );
    let code2 = generate_code_with_inline(
        "lg_v9_inl_det",
        &[("a", "a")],
        &[("start", vec!["h"]), ("h", vec!["a"])],
        &["h"],
        "start",
    );
    assert_eq!(code1, code2);
}

#[test]
fn lg_v9_inline_multiple_rules() {
    let code = generate_code_with_inline(
        "lg_v9_inl_multi",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["h1"]),
            ("start", vec!["h2"]),
            ("h1", vec!["a"]),
            ("h1", vec!["b"]),
            ("h2", vec!["c"]),
        ],
        &["h1", "h2"],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_inline_differs_from_no_inline() {
    let no_inline = generate_code(
        "lg_v9_inl_cmp_ni",
        &[("a", "a")],
        &[("start", vec!["h"]), ("h", vec!["a"])],
        "start",
    );
    let with_inline = generate_code_with_inline(
        "lg_v9_inl_cmp_wi",
        &[("a", "a")],
        &[("start", vec!["h"]), ("h", vec!["a"])],
        &["h"],
        "start",
    );
    // Different names → different output
    assert_ne!(no_inline, with_inline);
}

#[test]
fn lg_v9_inline_contains_grammar_name() {
    let code = generate_code_with_inline(
        "lg_v9_inl_gn",
        &[("a", "a")],
        &[("start", vec!["h"]), ("h", vec!["a"])],
        &["h"],
        "start",
    );
    assert!(code.contains("lg_v9_inl_gn"));
}

// ===========================================================================
// Extras-specific tests (56–60)
// ===========================================================================

#[test]
fn lg_v9_extras_output_nonempty() {
    let code = generate_code_with_extras(
        "lg_v9_ext_ne",
        &[("x", "x"), ("ws", r"\s+")],
        &[("start", vec!["x"])],
        &["ws"],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_extras_determinism() {
    let code1 = generate_code_with_extras(
        "lg_v9_ext_det",
        &[("x", "x"), ("ws", r"\s+")],
        &[("start", vec!["x"])],
        &["ws"],
        "start",
    );
    let code2 = generate_code_with_extras(
        "lg_v9_ext_det",
        &[("x", "x"), ("ws", r"\s+")],
        &[("start", vec!["x"])],
        &["ws"],
        "start",
    );
    assert_eq!(code1, code2);
}

#[test]
fn lg_v9_extras_multiple() {
    let code = generate_code_with_extras(
        "lg_v9_ext_mul",
        &[("x", "x"), ("ws", r"\s+"), ("comment", r"//[^\n]*")],
        &[("start", vec!["x"])],
        &["ws", "comment"],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_extras_contains_grammar_name() {
    let code = generate_code_with_extras(
        "lg_v9_ext_gn",
        &[("x", "x"), ("ws", r"\s+")],
        &[("start", vec!["x"])],
        &["ws"],
        "start",
    );
    assert!(code.contains("lg_v9_ext_gn"));
}

#[test]
fn lg_v9_extras_differs_from_no_extras() {
    let no_ext = generate_code(
        "lg_v9_ext_cmp_ne",
        &[("x", "x"), ("ws", r"\s+")],
        &[("start", vec!["x"])],
        "start",
    );
    let with_ext = generate_code_with_extras(
        "lg_v9_ext_cmp_we",
        &[("x", "x"), ("ws", r"\s+")],
        &[("start", vec!["x"])],
        &["ws"],
        "start",
    );
    // Different names → different output
    assert_ne!(no_ext, with_ext);
}

// ===========================================================================
// Generator construction and mutation tests (61–70)
// ===========================================================================

#[test]
fn lg_v9_set_start_can_be_empty() {
    let g = GrammarBuilder::new("lg_v9_empty_start")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    let mut slg = StaticLanguageGenerator::new(g, pt);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_default_start_not_empty() {
    let g = GrammarBuilder::new("lg_v9_notempty")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn lg_v9_grammar_field_accessible() {
    let g = GrammarBuilder::new("lg_v9_field")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    let slg = StaticLanguageGenerator::new(g, pt);
    assert_eq!(slg.grammar.name, "lg_v9_field");
}

#[test]
fn lg_v9_parse_table_field_accessible() {
    let g = GrammarBuilder::new("lg_v9_ptfield")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(slg.parse_table.state_count > 0);
}

#[test]
fn lg_v9_compressed_tables_initially_none() {
    let g = GrammarBuilder::new("lg_v9_compnone")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    let slg = StaticLanguageGenerator::new(g, pt);
    assert!(slg.compressed_tables.is_none());
}

#[test]
fn lg_v9_node_types_json_nonempty() {
    let g = GrammarBuilder::new("lg_v9_nodetypes")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    let slg = StaticLanguageGenerator::new(g, pt);
    let json = slg.generate_node_types();
    assert!(!json.is_empty());
}

#[test]
fn lg_v9_node_types_is_valid_json() {
    let g = GrammarBuilder::new("lg_v9_validjson")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    let slg = StaticLanguageGenerator::new(g, pt);
    let json = slg.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn lg_v9_node_types_determinism() {
    let make = || {
        let g = GrammarBuilder::new("lg_v9_ntdet")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let ff = FirstFollowSets::compute(&g).expect("ff");
        let pt = build_lr1_automaton(&g, &ff).expect("table");
        StaticLanguageGenerator::new(g, pt).generate_node_types()
    };
    assert_eq!(make(), make());
}

#[test]
fn lg_v9_start_can_be_empty_toggle() {
    let g = GrammarBuilder::new("lg_v9_toggle")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    let pt = build_lr1_automaton(&g, &ff).expect("table");
    let mut slg = StaticLanguageGenerator::new(g, pt);
    assert!(!slg.start_can_be_empty);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
    slg.set_start_can_be_empty(false);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn lg_v9_grammar_name_in_function_ident() {
    let code = generate_code(
        "lg_v9_fnident",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    assert!(code.contains("tree_sitter_lg_v9_fnident"));
}

// ===========================================================================
// Grammar variation tests (71–80)
// ===========================================================================

#[test]
fn lg_v9_long_token_name() {
    let code = generate_code(
        "lg_v9_longname",
        &[("very_long_token_name_for_testing", "x")],
        &[("start", vec!["very_long_token_name_for_testing"])],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_multiple_alternatives_same_nonterminal() {
    let code = generate_code(
        "lg_v9_alts",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("start", vec!["a"]),
            ("start", vec!["b"]),
            ("start", vec!["c"]),
        ],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_sequence_rule() {
    let code = generate_code(
        "lg_v9_seq",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("start", vec!["a", "b", "c"])],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_mixed_sequence_and_alternatives() {
    let code = generate_code(
        "lg_v9_mixed",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("start", vec!["a", "b"]), ("start", vec!["c"])],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_regex_token_pattern() {
    let code = generate_code(
        "lg_v9_regex",
        &[("ident", r"[a-zA-Z_][a-zA-Z0-9_]*")],
        &[("start", vec!["ident"])],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_numeric_regex_token() {
    let code = generate_code(
        "lg_v9_numregex",
        &[("number", r"[0-9]+(\.[0-9]+)?")],
        &[("start", vec!["number"])],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_string_literal_token() {
    let code = generate_code(
        "lg_v9_strlit",
        &[("kw", "if")],
        &[("start", vec!["kw"])],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_multiple_keyword_tokens() {
    let code = generate_code(
        "lg_v9_kwmulti",
        &[("kw_if", "if"), ("kw_else", "else"), ("kw_while", "while")],
        &[
            ("start", vec!["kw_if"]),
            ("start", vec!["kw_else"]),
            ("start", vec!["kw_while"]),
        ],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_output_has_minimum_length() {
    let code = generate_code(
        "lg_v9_minlen",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    // Even a minimal grammar should generate substantial code
    assert!(code.len() >= 50);
}

#[test]
fn lg_v9_large_grammar_produces_output() {
    let tokens: Vec<(&str, &str)> = vec![
        ("t0", "a"),
        ("t1", "b"),
        ("t2", "c"),
        ("t3", "d"),
        ("t4", "e"),
        ("t5", "f"),
        ("t6", "g"),
        ("t7", "h"),
    ];
    let rules: Vec<(&str, Vec<&str>)> = vec![
        ("start", vec!["t0"]),
        ("start", vec!["t1"]),
        ("start", vec!["t2"]),
        ("start", vec!["t3"]),
        ("start", vec!["t4"]),
        ("start", vec!["t5"]),
        ("start", vec!["t6"]),
        ("start", vec!["t7"]),
    ];
    let code = generate_code("lg_v9_large", &tokens, &rules, "start");
    assert!(!code.is_empty());
    assert!(code.len() > 200);
}

// ===========================================================================
// Cross-feature combination tests (81–85)
// ===========================================================================

#[test]
fn lg_v9_extras_with_multiple_tokens() {
    let code = generate_code_with_extras(
        "lg_v9_ext_mtok",
        &[("num", r"\d+"), ("id", r"[a-z]+"), ("ws", r"\s+")],
        &[("start", vec!["num"]), ("start", vec!["id"])],
        &["ws"],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_inline_with_chain() {
    let code = generate_code_with_inline(
        "lg_v9_inl_chain",
        &[("x", "x")],
        &[
            ("start", vec!["mid"]),
            ("mid", vec!["leaf"]),
            ("leaf", vec!["x"]),
        ],
        &["mid"],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_prec_with_multiple_nonterminals() {
    let code = generate_code_with_prec(
        "lg_v9_prec_mnt",
        &[("n", r"\d+"), ("p", "+"), ("lp", "("), ("rp", ")")],
        &[
            ("start", vec!["expr"]),
            ("expr", vec!["atom"]),
            ("atom", vec!["n"]),
            ("atom", vec!["lp", "expr", "rp"]),
        ],
        &[("expr", vec!["expr", "p", "expr"], 1, Associativity::Left)],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_two_nonterminal_with_extras() {
    let code = generate_code_with_extras(
        "lg_v9_2nt_ext",
        &[("a", "a"), ("b", "b"), ("ws", r"\s+")],
        &[
            ("start", vec!["inner"]),
            ("inner", vec!["a"]),
            ("inner", vec!["b"]),
        ],
        &["ws"],
        "start",
    );
    assert!(!code.is_empty());
}

#[test]
fn lg_v9_three_alt_determinism() {
    let make = || {
        generate_code(
            "lg_v9_3alt_det",
            &[("a", "a"), ("b", "b"), ("c", "c")],
            &[
                ("start", vec!["a"]),
                ("start", vec!["b"]),
                ("start", vec!["c"]),
            ],
            "start",
        )
    };
    assert_eq!(make(), make());
}

// ===========================================================================
// Output format assertions (86–90)
// ===========================================================================

#[test]
fn lg_v9_output_no_null_bytes() {
    let code = generate_code(
        "lg_v9_nonull",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    assert!(!code.contains('\0'));
}

#[test]
fn lg_v9_output_contains_braces() {
    let code = generate_code(
        "lg_v9_braces",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    // Rust code must contain braces
    assert!(code.contains('{') && code.contains('}'));
}

#[test]
fn lg_v9_output_contains_semicolons() {
    let code = generate_code(
        "lg_v9_semi",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    assert!(code.contains(';'));
}

#[test]
fn lg_v9_output_balanced_braces() {
    let code = generate_code(
        "lg_v9_balanced",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    let opens = code.chars().filter(|&c| c == '{').count();
    let closes = code.chars().filter(|&c| c == '}').count();
    assert_eq!(opens, closes);
}

#[test]
fn lg_v9_output_balanced_brackets() {
    let code = generate_code(
        "lg_v9_bal_brk",
        &[("x", "x")],
        &[("start", vec!["x"])],
        "start",
    );
    let opens = code.chars().filter(|&c| c == '[').count();
    let closes = code.chars().filter(|&c| c == ']').count();
    assert_eq!(opens, closes);
}
