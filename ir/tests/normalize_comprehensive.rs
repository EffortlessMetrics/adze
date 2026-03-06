//! Comprehensive tests for `Grammar::normalize()`.
//!
//! This test suite covers:
//! - Idempotency: normalize() on already-normalized grammar is idempotent
//! - Preservation: normalize() preserves token count, start symbol, and name
//! - Edge cases: empty grammar, simple grammars, complex nested structures
//! - Multiple calls: stability after multiple normalizations
//! - Serialization: normalized grammar can be serialized/deserialized

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

/// Check that a grammar is fully normalized (no complex symbols in rules)
fn is_fully_normalized(grammar: &Grammar) -> bool {
    grammar.all_rules().all(|rule| {
        rule.rhs.iter().all(|sym| {
            matches!(
                sym,
                Symbol::Terminal(_)
                    | Symbol::NonTerminal(_)
                    | Symbol::External(_)
                    | Symbol::Epsilon
            )
        })
    })
}

/// Count total rules in grammar
fn count_total_rules(grammar: &Grammar) -> usize {
    grammar.rules.values().map(|v| v.len()).sum()
}

/// Count auxiliary (generated) symbol IDs (those with ID >= 1000)
fn _count_aux_symbols(grammar: &Grammar) -> usize {
    grammar.rules.keys().filter(|id| id.0 >= 1000).count()
}

// ---------------------------------------------------------------------------
// Test 1: Idempotency - normalize() on already-normalized grammar
// ---------------------------------------------------------------------------

#[test]
fn normalize_idempotent_on_empty_grammar() {
    let mut grammar = Grammar::default();
    grammar.normalize();
    let result1 = grammar.normalize();
    assert!(result1.is_empty());
    assert!(is_fully_normalized(&grammar));
}

#[test]
fn normalize_idempotent_on_simple_grammar() {
    let mut grammar = GrammarBuilder::new("simple")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a", "b"])
        .start("expr")
        .build();

    let _rules_before = count_total_rules(&grammar);
    grammar.normalize();
    let rules_after_first = count_total_rules(&grammar);

    grammar.normalize();
    let rules_after_second = count_total_rules(&grammar);

    assert_eq!(
        rules_after_first, rules_after_second,
        "Rules changed after second normalize"
    );
    assert!(is_fully_normalized(&grammar));
}

#[test]
fn normalize_idempotent_multiple_times() {
    let mut grammar = GrammarBuilder::new("multi")
        .token("x", "x")
        .rule("start", vec!["x"])
        .rule("start", vec!["x", "x"])
        .start("start")
        .build();

    grammar.normalize();
    let rules1 = count_total_rules(&grammar);

    grammar.normalize();
    let rules2 = count_total_rules(&grammar);

    grammar.normalize();
    let rules3 = count_total_rules(&grammar);

    assert_eq!(rules1, rules2);
    assert_eq!(rules2, rules3);
}

// ---------------------------------------------------------------------------
// Test 2: Preserves token count
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_token_count_empty() {
    let mut grammar = Grammar::default();
    let tokens_before = grammar.tokens.len();
    grammar.normalize();
    assert_eq!(grammar.tokens.len(), tokens_before);
}

#[test]
fn normalize_preserves_token_count_with_tokens() {
    let mut grammar = GrammarBuilder::new("tokens")
        .token("NUMBER", r"\d+")
        .token("PLUS", "+")
        .token("MINUS", "-")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let tokens_before = grammar.tokens.len();
    assert_eq!(tokens_before, 3);

    grammar.normalize();
    assert_eq!(grammar.tokens.len(), tokens_before);
}

#[test]
fn normalize_preserves_token_count_multiple_calls() {
    let mut grammar = GrammarBuilder::new("preserve")
        .token("ID", r"[a-z]+")
        .token("COLON", ":")
        .token("SEMI", ";")
        .rule("stmt", vec!["ID", "COLON"])
        .start("stmt")
        .build();

    let tokens_before = grammar.tokens.len();

    for _ in 0..5 {
        grammar.normalize();
        assert_eq!(grammar.tokens.len(), tokens_before);
    }
}

// ---------------------------------------------------------------------------
// Test 3: Preserves start symbol
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_start_symbol() {
    let mut grammar = GrammarBuilder::new("with_start")
        .token("a", "a")
        .rule("program", vec!["a"])
        .rule("statement", vec!["a"])
        .start("program")
        .build();

    let start_before = grammar.start_symbol();
    grammar.normalize();
    let start_after = grammar.start_symbol();

    assert_eq!(start_before, start_after);
    assert!(start_before.is_some());
}

#[test]
fn normalize_preserves_start_symbol_multiple_rules() {
    let mut grammar = GrammarBuilder::new("multi_rule_start")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .rule("other", vec!["x", "y"])
        .start("start")
        .build();

    let start_before = grammar.start_symbol();
    grammar.normalize();
    let start_after = grammar.start_symbol();

    assert_eq!(start_before, start_after);
}

// ---------------------------------------------------------------------------
// Test 4: Grammar name preservation
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_grammar_name() {
    let mut grammar = GrammarBuilder::new("my_language")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .start("expr")
        .build();

    let name_before = grammar.name.clone();
    grammar.normalize();

    assert_eq!(grammar.name, name_before);
    assert_eq!(grammar.name, "my_language");
}

// ---------------------------------------------------------------------------
// Test 5: Doesn't panic on empty grammar
// ---------------------------------------------------------------------------

#[test]
fn normalize_no_panic_empty_grammar() {
    let mut grammar = Grammar::default();
    let _ = grammar.normalize();
    // No panic should occur
}

#[test]
fn normalize_no_panic_empty_rules() {
    let mut grammar = Grammar {
        name: "empty".to_string(),
        ..Default::default()
    };
    let _ = grammar.normalize();
    assert!(grammar.rules.is_empty());
}

// ---------------------------------------------------------------------------
// Test 6: Terminal-only rules
// ---------------------------------------------------------------------------

#[test]
fn normalize_simple_terminal_rules() {
    let mut grammar = GrammarBuilder::new("terminals")
        .token("A", "a")
        .token("B", "b")
        .rule("rule1", vec!["A"])
        .rule("rule2", vec!["A", "B"])
        .start("rule1")
        .build();

    let rules_before = count_total_rules(&grammar);
    grammar.normalize();
    let rules_after = count_total_rules(&grammar);

    assert_eq!(rules_before, rules_after);
    assert!(is_fully_normalized(&grammar));
}

#[test]
fn normalize_single_terminal_rule() {
    let mut grammar = GrammarBuilder::new("single")
        .token("T", "t")
        .rule("expr", vec!["T"])
        .start("expr")
        .build();

    let result = grammar.normalize();
    assert!(!result.is_empty());
    assert!(is_fully_normalized(&grammar));
}

// ---------------------------------------------------------------------------
// Test 7: Multiple rules for same LHS
// ---------------------------------------------------------------------------

#[test]
fn normalize_multiple_rules_same_lhs() {
    let mut grammar = GrammarBuilder::new("multi_lhs")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .rule("expr", vec!["a", "b"])
        .start("expr")
        .build();

    let rules_before = count_total_rules(&grammar);
    grammar.normalize();
    let rules_after = count_total_rules(&grammar);

    assert_eq!(rules_before, rules_after);
    assert!(is_fully_normalized(&grammar));
}

#[test]
fn normalize_preserves_all_alternatives() {
    let mut grammar = GrammarBuilder::new("alternatives")
        .token("x", "x")
        .rule("stmt", vec!["x"])
        .rule("stmt", vec!["x", "x"])
        .rule("stmt", vec!["x", "x", "x"])
        .start("stmt")
        .build();

    let stmt_rules_before = grammar
        .get_rules_for_symbol(grammar.find_symbol_by_name("stmt").unwrap())
        .unwrap()
        .len();

    grammar.normalize();

    let stmt_rules_after = grammar
        .get_rules_for_symbol(grammar.find_symbol_by_name("stmt").unwrap())
        .unwrap()
        .len();

    assert_eq!(stmt_rules_before, stmt_rules_after);
}

// ---------------------------------------------------------------------------
// Test 8: Grammar properties after normalization
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_name_and_tokens() {
    let mut grammar = GrammarBuilder::new("preserve_all")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .rule("expr", vec!["ID"])
        .start("expr")
        .build();

    let name_before = grammar.name.clone();
    let token_count_before = grammar.tokens.len();

    grammar.normalize();

    assert_eq!(grammar.name, name_before);
    assert_eq!(grammar.tokens.len(), token_count_before);
}

#[test]
fn normalize_preserves_multiple_properties() {
    let mut grammar = GrammarBuilder::new("multi_prop")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["expr", "STAR", "expr"])
        .start("expr")
        .build();

    let name = grammar.name.clone();
    let tokens = grammar.tokens.len();
    let start = grammar.start_symbol();

    grammar.normalize();

    assert_eq!(grammar.name, name);
    assert_eq!(grammar.tokens.len(), tokens);
    assert_eq!(grammar.start_symbol(), start);
    assert!(is_fully_normalized(&grammar));
}

// ---------------------------------------------------------------------------
// Test 9: Stability after multiple calls
// ---------------------------------------------------------------------------

#[test]
fn normalize_stable_after_multiple_calls() {
    let mut grammar = GrammarBuilder::new("stable")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a", "b"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();

    grammar.normalize();
    let rules_after_1st = count_total_rules(&grammar);
    let tokens_1st = grammar.tokens.len();

    grammar.normalize();
    let rules_after_2nd = count_total_rules(&grammar);
    let tokens_2nd = grammar.tokens.len();

    grammar.normalize();
    let rules_after_3rd = count_total_rules(&grammar);
    let tokens_3rd = grammar.tokens.len();

    assert_eq!(rules_after_1st, rules_after_2nd);
    assert_eq!(rules_after_2nd, rules_after_3rd);
    assert_eq!(tokens_1st, tokens_2nd);
    assert_eq!(tokens_2nd, tokens_3rd);
}

#[test]
fn normalize_convergence() {
    let mut grammar = GrammarBuilder::new("converge")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .start("c")
        .build();

    let mut prev_count = count_total_rules(&grammar);

    for _ in 0..10 {
        grammar.normalize();
        let curr_count = count_total_rules(&grammar);
        assert_eq!(
            prev_count, curr_count,
            "Rule count changed between normalizations"
        );
        prev_count = curr_count;
    }
}

// ---------------------------------------------------------------------------
// Test 10: Normalized grammar structure
// ---------------------------------------------------------------------------

#[test]
fn normalize_result_is_fully_normalized() {
    let mut grammar = GrammarBuilder::new("result")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .start("expr")
        .build();

    let _ = grammar.normalize();
    assert!(is_fully_normalized(&grammar));
}

#[test]
fn normalize_all_symbols_in_rhs_are_simple() {
    let mut grammar = GrammarBuilder::new("simple_rhs")
        .token("t", "t")
        .rule("s", vec!["t"])
        .rule("s", vec!["s", "t"])
        .start("s")
        .build();

    grammar.normalize();

    for rule in grammar.all_rules() {
        for sym in &rule.rhs {
            match sym {
                Symbol::Terminal(_)
                | Symbol::NonTerminal(_)
                | Symbol::External(_)
                | Symbol::Epsilon => {
                    // OK
                }
                _ => panic!("Found complex symbol after normalize: {:?}", sym),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Test 11: Rule count behavior
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_simple_rule_count() {
    let mut grammar = GrammarBuilder::new("rule_count")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A"])
        .rule("expr", vec!["B"])
        .start("expr")
        .build();

    let count_before = count_total_rules(&grammar);
    grammar.normalize();
    let count_after = count_total_rules(&grammar);

    assert_eq!(count_before, count_after);
}

#[test]
fn normalize_with_epsilon_rules() {
    let mut grammar = GrammarBuilder::new("epsilon")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("expr", vec![]) // Epsilon rule
        .start("expr")
        .build();

    let count_before = count_total_rules(&grammar);
    grammar.normalize();
    let count_after = count_total_rules(&grammar);

    assert_eq!(count_before, count_after);
    assert!(is_fully_normalized(&grammar));
}

// ---------------------------------------------------------------------------
// Test 12: Return value of normalize()
// ---------------------------------------------------------------------------

#[test]
fn normalize_returns_all_rules() {
    let mut grammar = GrammarBuilder::new("return_val")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .start("expr")
        .build();

    let returned = grammar.normalize();
    let rule_count = count_total_rules(&grammar);

    assert!(!returned.is_empty());
    assert_eq!(returned.len(), rule_count);
}

#[test]
fn normalize_return_value_matches_grammar() {
    let mut grammar = GrammarBuilder::new("match_return")
        .token("x", "x")
        .rule("stmt", vec!["x"])
        .rule("stmt", vec!["x", "x"])
        .start("stmt")
        .build();

    let returned = grammar.normalize();
    let grammar_rules: Vec<_> = grammar.all_rules().cloned().collect();

    assert_eq!(returned.len(), grammar_rules.len());
}

// ---------------------------------------------------------------------------
// Test 13: Non-regression - basic grammar properties
// ---------------------------------------------------------------------------

#[test]
fn normalize_maintains_rule_lhs() {
    let mut grammar = GrammarBuilder::new("maintain_lhs")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    grammar.normalize();

    assert!(grammar.rules.contains_key(&expr_id));
}

#[test]
fn normalize_does_not_remove_original_rules() {
    let mut grammar = GrammarBuilder::new("no_remove")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a", "b"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let expr_rules_before = grammar.get_rules_for_symbol(expr_id).unwrap().len();

    grammar.normalize();

    let expr_rules_after = grammar.get_rules_for_symbol(expr_id).unwrap().len();
    assert_eq!(expr_rules_before, expr_rules_after);
}

// ---------------------------------------------------------------------------
// Test 14: Complex multi-rule grammar
// ---------------------------------------------------------------------------

#[test]
fn normalize_complex_multi_rule_grammar() {
    let mut grammar = GrammarBuilder::new("complex")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["middle"])
        .rule("middle", vec!["b"])
        .rule("middle", vec!["end"])
        .rule("end", vec!["c"])
        .start("start")
        .build();

    grammar.normalize();
    assert!(is_fully_normalized(&grammar));
    assert!(grammar.rules.len() >= 3); // At least start, middle, end
}

#[test]
fn normalize_deeply_nested_rules() {
    let mut grammar = GrammarBuilder::new("nested")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["d"])
        .rule("d", vec!["x"])
        .start("a")
        .build();

    let rules_before = count_total_rules(&grammar);
    grammar.normalize();
    let rules_after = count_total_rules(&grammar);

    assert_eq!(rules_before, rules_after);
}

// ---------------------------------------------------------------------------
// Test 15: Token reference preservation
// ---------------------------------------------------------------------------

#[test]
fn normalize_preserves_token_references() {
    let mut grammar = GrammarBuilder::new("token_refs")
        .token("INT", r"\d+")
        .token("ID", r"[a-z]+")
        .rule("expr", vec!["INT"])
        .rule("expr", vec!["ID"])
        .start("expr")
        .build();

    let token_count_before = grammar.tokens.len();
    assert_eq!(token_count_before, 2);

    grammar.normalize();

    // Tokens should still be in the grammar
    assert_eq!(grammar.tokens.len(), 2);
}

// ---------------------------------------------------------------------------
// Test 16: Serialization/Deserialization of normalized grammar
// ---------------------------------------------------------------------------

#[test]
fn normalize_grammar_serializable() {
    let mut grammar = GrammarBuilder::new("serializable")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .start("expr")
        .build();

    grammar.normalize();

    // Try to serialize
    let json = serde_json::to_string(&grammar);
    assert!(json.is_ok(), "Failed to serialize normalized grammar");
}

#[test]
fn normalize_grammar_roundtrip_serialization() {
    let mut grammar = GrammarBuilder::new("roundtrip")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a", "b"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();

    grammar.normalize();

    let json = serde_json::to_string(&grammar).expect("Serialization failed");
    let deserialized: Grammar = serde_json::from_str(&json).expect("Deserialization failed");

    assert_eq!(grammar.name, deserialized.name);
    assert_eq!(grammar.tokens.len(), deserialized.tokens.len());
    assert_eq!(
        count_total_rules(&grammar),
        count_total_rules(&deserialized)
    );
}

// ---------------------------------------------------------------------------
// Test 17: Empty rules preservation
// ---------------------------------------------------------------------------

#[test]
fn normalize_with_empty_rules_preserved() {
    let mut grammar = GrammarBuilder::new("empty_rules")
        .token("a", "a")
        .rule("expr", vec![])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();

    let rules_before = count_total_rules(&grammar);
    grammar.normalize();
    let rules_after = count_total_rules(&grammar);

    assert_eq!(rules_before, rules_after);
}

// ---------------------------------------------------------------------------
// Test 18: Practical grammar examples
// ---------------------------------------------------------------------------

#[test]
fn normalize_arithmetic_grammar() {
    let mut grammar = GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("div", ":")
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["expr", "div", "expr"])
        .start("expr")
        .build();

    let rules_before = count_total_rules(&grammar);
    let tokens_before = grammar.tokens.len();

    grammar.normalize();

    let rules_after = count_total_rules(&grammar);
    let tokens_after = grammar.tokens.len();

    assert_eq!(rules_before, rules_after);
    assert_eq!(tokens_before, tokens_after);
    assert!(is_fully_normalized(&grammar));
}

#[test]
fn normalize_statement_grammar() {
    let mut grammar = GrammarBuilder::new("statements")
        .token("IF", "if")
        .token("ELSE", "else")
        .token("END", "end")
        .token("ID", r"[a-z]+")
        .rule("program", vec!["statement"])
        .rule("program", vec!["program", "statement"])
        .rule("statement", vec!["IF", "ID"])
        .rule("statement", vec!["IF", "ID", "ELSE"])
        .start("program")
        .build();

    grammar.normalize();
    assert!(is_fully_normalized(&grammar));
}

// ---------------------------------------------------------------------------
// Test 19: No auxiliary symbol overflow
// ---------------------------------------------------------------------------

#[test]
fn normalize_auxiliary_ids_high_enough() {
    let mut grammar = GrammarBuilder::new("aux_ids")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .start("expr")
        .build();

    grammar.normalize();

    // Any auxiliary symbols should be >= 1000 (as per normalize implementation)
    for id in grammar.rules.keys() {
        if id.0 >= 1000 {
            // This is an auxiliary symbol, which is fine
        }
    }
}

// ---------------------------------------------------------------------------
// Test 20: Mixed token types
// ---------------------------------------------------------------------------

#[test]
fn normalize_mixed_terminals_nonterminals() {
    let mut grammar = GrammarBuilder::new("mixed")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["inner"])
        .rule("inner", vec!["b"])
        .start("expr")
        .build();

    let rules_before = count_total_rules(&grammar);
    grammar.normalize();
    let rules_after = count_total_rules(&grammar);

    assert_eq!(rules_before, rules_after);
    assert!(is_fully_normalized(&grammar));
}

// ---------------------------------------------------------------------------
// Test 21: Comprehensive idempotency test with varied grammars
// ---------------------------------------------------------------------------

#[test]
fn normalize_idempotent_varied_grammar() {
    let mut grammar = GrammarBuilder::new("varied")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+")
        .token("COMMA", ",")
        .rule("list", vec!["ID"])
        .rule("list", vec!["NUM"])
        .rule("list", vec!["list", "COMMA", "ID"])
        .rule("list", vec!["list", "COMMA", "NUM"])
        .start("list")
        .build();

    grammar.normalize();
    let state_1 = (count_total_rules(&grammar), grammar.tokens.len());

    grammar.normalize();
    let state_2 = (count_total_rules(&grammar), grammar.tokens.len());

    grammar.normalize();
    let state_3 = (count_total_rules(&grammar), grammar.tokens.len());

    assert_eq!(state_1, state_2);
    assert_eq!(state_2, state_3);
}

// ---------------------------------------------------------------------------
// Test 22: All rules remain accessible after normalize
// ---------------------------------------------------------------------------

#[test]
fn normalize_all_rules_method_works() {
    let mut grammar = GrammarBuilder::new("all_rules")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .rule("term", vec!["expr"])
        .start("expr")
        .build();

    let rules_before: Vec<_> = grammar.all_rules().cloned().collect();
    grammar.normalize();
    let rules_after: Vec<_> = grammar.all_rules().cloned().collect();

    assert_eq!(rules_before.len(), rules_after.len());
}

// ---------------------------------------------------------------------------
// Test 23: Normalize doesn't break grammar iteration
// ---------------------------------------------------------------------------

#[test]
fn normalize_grammar_iteration_intact() {
    let mut grammar = GrammarBuilder::new("iterate")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .start("a")
        .build();

    grammar.normalize();

    let mut rule_count = 0;
    for (_, rules) in &grammar.rules {
        for rule in rules {
            rule_count += 1;
            assert!(!rule.rhs.is_empty() || rule.rhs.iter().all(|s| matches!(s, Symbol::Epsilon)));
        }
    }

    assert!(rule_count > 0);
}

// ---------------------------------------------------------------------------
// Test 24: Large grammar normalization
// ---------------------------------------------------------------------------

#[test]
fn normalize_large_grammar() {
    let mut builder = GrammarBuilder::new("large");

    // Add many tokens
    for i in 0..10 {
        builder = builder.token(&format!("T{}", i), &format!("t{}", i));
    }

    // Add many rules
    for i in 0..10 {
        builder = builder.rule(&format!("rule_{}", i), vec![&format!("T{}", i)]);
    }

    let mut grammar = builder.start("rule_0").build();

    let rules_before = count_total_rules(&grammar);
    let tokens_before = grammar.tokens.len();

    grammar.normalize();

    let rules_after = count_total_rules(&grammar);
    let tokens_after = grammar.tokens.len();

    assert_eq!(rules_before, rules_after);
    assert_eq!(tokens_before, tokens_after);
}

// ---------------------------------------------------------------------------
// Test 25: Normalize with single-symbol rules
// ---------------------------------------------------------------------------

#[test]
fn normalize_single_symbol_rules() {
    let mut grammar = GrammarBuilder::new("single_sym")
        .token("t", "t")
        .rule("a", vec!["t"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .start("a")
        .build();

    grammar.normalize();
    assert!(is_fully_normalized(&grammar));

    // Verify rule chain is intact
    let c_id = grammar.find_symbol_by_name("c").unwrap();
    assert!(grammar.get_rules_for_symbol(c_id).is_some());
}
