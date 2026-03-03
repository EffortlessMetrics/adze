#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for the IR grammar optimizer.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, optimize_grammar};
use adze_ir::{Associativity, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find a symbol ID by name in rule_names or tokens.
#[allow(dead_code)]
fn find_symbol(grammar: &Grammar, name: &str) -> Option<SymbolId> {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .or_else(|| {
            grammar
                .tokens
                .iter()
                .find(|(_, t)| t.name == name)
                .map(|(id, _)| *id)
        })
}

/// Count total rules across all symbols.
fn total_rule_count(grammar: &Grammar) -> usize {
    grammar.all_rules().count()
}

/// Collect all token names.
fn token_names(grammar: &Grammar) -> Vec<String> {
    grammar.tokens.values().map(|t| t.name.clone()).collect()
}

/// Collect all rule-name entries.
fn rule_name_values(grammar: &Grammar) -> Vec<String> {
    grammar.rule_names.values().cloned().collect()
}

// ===========================================================================
// 1. Empty grammar
// ===========================================================================

#[test]
fn optimize_empty_grammar_returns_ok() {
    let grammar = Grammar::new("empty".into());
    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
}

#[test]
fn optimize_empty_grammar_has_zero_stats() {
    let mut grammar = Grammar::new("empty".into());
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert_eq!(stats.total(), 0);
    assert_eq!(stats.removed_unused_symbols, 0);
    assert_eq!(stats.inlined_rules, 0);
    assert_eq!(stats.merged_tokens, 0);
    assert_eq!(stats.optimized_left_recursion, 0);
    assert_eq!(stats.eliminated_unit_rules, 0);
}

// ===========================================================================
// 2. Dead rule elimination
// ===========================================================================

#[test]
fn dead_token_is_removed() {
    // "DEAD" token is never referenced by any rule.
    let grammar = GrammarBuilder::new("dead_tok")
        .token("A", "a")
        .token("DEAD", "dead")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    let names = token_names(&optimized);
    assert!(names.contains(&"A".to_string()));
    assert!(!names.contains(&"DEAD".to_string()));
}

#[test]
fn dead_rule_is_removed() {
    // "orphan" is never referenced from the start symbol's reachable set.
    let grammar = GrammarBuilder::new("dead_rule")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A"])
        .rule("orphan", vec!["B"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    let names = rule_name_values(&optimized);
    // orphan should have been removed as it is unreachable.
    assert!(!names.contains(&"orphan".to_string()));
}

#[test]
fn reachable_rule_is_kept() {
    let grammar = GrammarBuilder::new("keep")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["inner"])
        .rule("inner", vec!["A", "B"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    // "inner" is reachable from "expr" so it must survive (possibly inlined).
    // The grammar should still be able to produce the same language.
    assert!(total_rule_count(&optimized) >= 1);
}

#[test]
fn multiple_dead_tokens_removed() {
    let grammar = GrammarBuilder::new("multi_dead")
        .token("A", "a")
        .token("UNUSED1", "x")
        .token("UNUSED2", "y")
        .token("UNUSED3", "z")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    let names = token_names(&optimized);
    assert!(!names.contains(&"UNUSED1".to_string()));
    assert!(!names.contains(&"UNUSED2".to_string()));
    assert!(!names.contains(&"UNUSED3".to_string()));
}

// ===========================================================================
// 3. Symbol deduplication (merge equivalent tokens)
// ===========================================================================

#[test]
fn duplicate_tokens_are_merged() {
    // Two tokens with the same pattern should be deduplicated.
    let mut grammar = Grammar::new("dedup".into());
    let id1 = SymbolId(1);
    let id2 = SymbolId(2);
    let rule_sym = SymbolId(3);

    grammar.tokens.insert(
        id1,
        Token {
            name: "plus_a".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        id2,
        Token {
            name: "plus_b".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(rule_sym, "expr".to_string());
    grammar.add_rule(Rule {
        lhs: rule_sym,
        rhs: vec![Symbol::Terminal(id1), Symbol::Terminal(id2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);

    assert!(stats.merged_tokens >= 1);
    // After merging, only one token with pattern "+" should remain.
    let plus_count = grammar
        .tokens
        .values()
        .filter(|t| matches!(&t.pattern, TokenPattern::String(s) if s == "+"))
        .count();
    assert_eq!(plus_count, 1);
}

// ===========================================================================
// 4. Left recursion optimisation
// ===========================================================================

#[test]
fn left_recursion_is_detected_and_transformed() {
    let grammar = GrammarBuilder::new("lr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "NUM"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let mut g = grammar;
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    assert!(stats.optimized_left_recursion >= 1);
}

#[test]
fn left_recursion_produces_epsilon_rule() {
    let grammar = GrammarBuilder::new("lr_eps")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "NUM"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();

    // Transformation creates A' -> ... | ε.  At least one rule should have an
    // empty RHS (the epsilon production).
    let has_epsilon = optimized.all_rules().any(|r| r.rhs.is_empty());
    assert!(
        has_epsilon,
        "left-recursion elimination should create an epsilon rule"
    );
}

// ===========================================================================
// 5. Unit rule elimination
// ===========================================================================

#[test]
fn unit_rule_is_eliminated() {
    // The optimizer skips unit-rule elimination when the target has terminals
    // and the LHS is the start symbol.  Place the unit rule below the start.
    //   program -> wrapper  (non-unit, two symbols)
    //   wrapper -> inner    (unit rule)
    //   inner   -> A B
    let grammar = GrammarBuilder::new("unit")
        .token("A", "a")
        .token("B", "b")
        .rule("program", vec!["wrapper", "A"])
        .rule("wrapper", vec!["inner"])
        .rule("inner", vec!["A", "B"])
        .start("program")
        .build();

    let mut g = grammar;
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    // wrapper -> inner is a unit rule; inner -> A B has only terminals
    // but wrapper is NOT the start symbol, so the skip-guard does not apply.
    assert!(
        stats.eliminated_unit_rules >= 1 || stats.inlined_rules >= 1,
        "unit rule should be eliminated or inlined",
    );
}

// ===========================================================================
// 6. Optimization preserves semantics
// ===========================================================================

#[test]
fn optimization_preserves_start_symbol_reachability() {
    let grammar = GrammarBuilder::new("sem")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("expr", vec!["A", "B", "C"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();

    // The optimized grammar should still have rules reachable from a start symbol.
    assert!(optimized.start_symbol().is_some());
    assert!(total_rule_count(&optimized) >= 1);
}

#[test]
fn optimization_preserves_token_patterns() {
    let grammar = GrammarBuilder::new("pat")
        .token("NUM", r"\d+")
        .token("IDENT", r"[a-z]+")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["IDENT"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();

    // All referenced tokens must keep their original patterns.
    for tok in optimized.tokens.values() {
        match tok.name.as_str() {
            "NUM" => assert!(matches!(&tok.pattern, TokenPattern::Regex(r) if r.contains("\\d"))),
            "IDENT" => {
                assert!(matches!(&tok.pattern, TokenPattern::Regex(r) if r.contains("[a-z]")))
            }
            _ => {}
        }
    }
}

#[test]
fn optimization_preserves_precedence_info() {
    let grammar = GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();

    // After optimisation the grammar should still have rules carrying precedence.
    let has_prec = optimized.all_rules().any(|r| r.precedence.is_some());
    assert!(has_prec, "precedence information must survive optimisation");
}

#[test]
fn optimization_preserves_extras_when_referenced() {
    // WS is referenced both as an extra and in a rule, so it should survive.
    let grammar = GrammarBuilder::new("extras")
        .token("A", "a")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("expr", vec!["A", "WS", "A"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    assert!(!optimized.extras.is_empty(), "extras should be preserved");
}

// ===========================================================================
// 7. Already-optimized grammar
// ===========================================================================

#[test]
fn already_optimal_grammar_is_unchanged() {
    // A multi-symbol RHS avoids inlining. No dead symbols, no unit rules.
    let grammar = GrammarBuilder::new("optimal")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A", "B"])
        .start("expr")
        .build();

    let before_rules = total_rule_count(&grammar);
    let before_tokens = grammar.tokens.len();

    let optimized = optimize_grammar(grammar).unwrap();

    assert_eq!(total_rule_count(&optimized), before_rules);
    assert_eq!(optimized.tokens.len(), before_tokens);
}

#[test]
fn double_optimize_is_idempotent_on_rule_count() {
    let grammar = GrammarBuilder::new("idem")
        .token("A", "a")
        .token("B", "b")
        .token("DEAD", "dead")
        .rule("expr", vec!["A", "B"])
        .start("expr")
        .build();

    let once = optimize_grammar(grammar).unwrap();
    let once_rules = total_rule_count(&once);
    let once_tokens = once.tokens.len();

    let twice = optimize_grammar(once).unwrap();
    assert_eq!(total_rule_count(&twice), once_rules);
    assert_eq!(twice.tokens.len(), once_tokens);
}

// ===========================================================================
// 8. Multiple optimization passes
// ===========================================================================

#[test]
fn multiple_manual_passes_converge() {
    let grammar = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .token("DEAD", "dead")
        .rule("expr", vec!["inner"])
        .rule("inner", vec!["A", "B"])
        .rule("orphan", vec!["DEAD"])
        .start("expr")
        .build();

    let mut g = grammar;
    // After enough passes the optimizer should reach a fixed point.
    for _ in 0..5 {
        let mut opt = GrammarOptimizer::new();
        opt.optimize(&mut g);
    }
    // One final pass should do zero work.
    let mut opt = GrammarOptimizer::new();
    let final_stats = opt.optimize(&mut g);
    assert_eq!(
        final_stats.total(),
        0,
        "optimizer should converge to a fixed point"
    );
}

// ===========================================================================
// 9. Optimization statistics
// ===========================================================================

#[test]
fn stats_total_is_sum_of_fields() {
    let grammar = GrammarBuilder::new("stats")
        .token("A", "a")
        .token("DEAD", "dead")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();

    let mut g = grammar;
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    let expected = stats.removed_unused_symbols
        + stats.inlined_rules
        + stats.merged_tokens
        + stats.optimized_left_recursion
        + stats.eliminated_unit_rules;
    assert_eq!(stats.total(), expected);
}

#[test]
fn stats_default_is_all_zero() {
    let stats = adze_ir::optimizer::OptimizationStats::default();
    assert_eq!(stats.total(), 0);
}

#[test]
fn stats_reports_dead_symbol_removal() {
    let grammar = GrammarBuilder::new("stat_dead")
        .token("A", "a")
        .token("DEAD1", "d1")
        .token("DEAD2", "d2")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();

    let mut g = grammar;
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    // At least the two dead tokens should be counted.
    assert!(stats.removed_unused_symbols >= 2);
}

// ===========================================================================
// 10. Rule reordering / renumbering
// ===========================================================================

#[test]
fn renumbering_keeps_symbols_contiguous() {
    let grammar = GrammarBuilder::new("renum")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A", "B"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();

    // All symbol IDs in the optimized grammar should be small and contiguous.
    let max_token_id = optimized.tokens.keys().map(|id| id.0).max().unwrap_or(0);
    let max_rule_id = optimized.rules.keys().map(|id| id.0).max().unwrap_or(0);
    let max_id = max_token_id.max(max_rule_id);
    let num_symbols = optimized.tokens.len() + optimized.rules.len();

    // The max ID should be at most num_symbols (plus 1 for the reserved 0 / EOF).
    assert!(
        (max_id as usize) <= num_symbols + 1,
        "max_id={max_id} should be <= num_symbols+1={}",
        num_symbols + 1,
    );
}

// ===========================================================================
// 11. Complex / composite grammars
// ===========================================================================

#[test]
fn optimize_grammar_with_multiple_alternatives() {
    let grammar = GrammarBuilder::new("alts")
        .token("NUM", r"\d+")
        .token("IDENT", r"[a-z]+")
        .token("STR", r#""[^"]*""#)
        .rule("value", vec!["NUM"])
        .rule("value", vec!["IDENT"])
        .rule("value", vec!["STR"])
        .start("value")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    assert!(total_rule_count(&optimized) >= 1);
    assert!(optimized.tokens.len() >= 3);
}

#[test]
fn optimize_grammar_with_chain_of_unit_rules() {
    // Unit-rule elimination skips creating terminal productions for the
    // start symbol, so place the chain below the start.
    //   start -> A C  (multi-symbol, not a unit rule)
    //   A -> B        (unit rule — non-start)
    //   B -> C        (unit rule — non-start)
    //   C -> TOK TOK  (terminal production)
    let grammar = GrammarBuilder::new("chain")
        .token("TOK", "t")
        .rule("start", vec!["A", "TOK"])
        .rule("A", vec!["B"])
        .rule("B", vec!["C"])
        .rule("C", vec!["TOK", "TOK"])
        .start("start")
        .build();

    let mut g = grammar;
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    assert!(
        stats.eliminated_unit_rules >= 1 || stats.inlined_rules >= 1,
        "chain of unit rules should trigger elimination or inlining",
    );
}

#[test]
fn optimize_grammar_with_externals() {
    let grammar = GrammarBuilder::new("ext")
        .token("A", "a")
        .external("INDENT")
        .external("DEDENT")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    assert!(
        optimized.externals.len() >= 2,
        "external tokens should be preserved",
    );
}

#[test]
fn optimize_convenience_function_returns_valid_grammar() {
    let grammar = GrammarBuilder::new("conv")
        .token("A", "a")
        .token("B", "b")
        .token("DEAD", "x")
        .rule("expr", vec!["A", "B"])
        .start("expr")
        .build();

    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
    let g = result.unwrap();
    assert!(!g.tokens.is_empty());
    assert!(!g.rules.is_empty());
}

// ===========================================================================
// 12. Inline simple rules
// ===========================================================================

#[test]
fn single_use_non_terminal_can_be_inlined() {
    // wrapper -> A  (used once in start)
    // start -> wrapper
    let grammar = GrammarBuilder::new("inl")
        .token("A", "a")
        .rule("start", vec!["wrapper"])
        .rule("wrapper", vec!["A"])
        .start("start")
        .build();

    let mut g = grammar;
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    // Either inlined or unit-eliminated — total should be > 0.
    assert!(stats.total() > 0);
}

// ===========================================================================
// 13. Fragile tokens
// ===========================================================================

#[test]
fn fragile_tokens_survive_optimization() {
    let grammar = GrammarBuilder::new("fragile")
        .token("A", "a")
        .fragile_token("ERR", "error")
        .rule("expr", vec!["A"])
        .rule("expr", vec!["ERR"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();
    let has_fragile = optimized.tokens.values().any(|t| t.fragile);
    assert!(
        has_fragile,
        "fragile tokens referenced by rules should survive"
    );
}

// ===========================================================================
// 14. Large grammar stress
// ===========================================================================

#[test]
fn optimize_javascript_like_grammar() {
    let grammar = GrammarBuilder::javascript_like();
    let optimized = optimize_grammar(grammar).unwrap();

    // The grammar should still have a start symbol and be non-empty.
    assert!(optimized.start_symbol().is_some());
    assert!(total_rule_count(&optimized) >= 1);
}

#[test]
fn optimize_python_like_grammar() {
    let grammar = GrammarBuilder::python_like();
    let optimized = optimize_grammar(grammar).unwrap();

    assert!(optimized.start_symbol().is_some());
    assert!(total_rule_count(&optimized) >= 1);
}

// ===========================================================================
// 15. Regex tokens survive deduplication
// ===========================================================================

#[test]
fn regex_tokens_with_same_pattern_are_merged() {
    let mut grammar = Grammar::new("regex_dedup".into());
    let id1 = SymbolId(1);
    let id2 = SymbolId(2);
    let rule_sym = SymbolId(3);

    grammar.tokens.insert(
        id1,
        Token {
            name: "digits_a".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        id2,
        Token {
            name: "digits_b".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(rule_sym, "expr".to_string());
    grammar.add_rule(Rule {
        lhs: rule_sym,
        rhs: vec![Symbol::Terminal(id1), Symbol::Terminal(id2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert!(stats.merged_tokens >= 1);

    let regex_count = grammar
        .tokens
        .values()
        .filter(|t| matches!(&t.pattern, TokenPattern::Regex(r) if r == r"\d+"))
        .count();
    assert_eq!(regex_count, 1);
}

// ===========================================================================
// 16. Distinct tokens are not merged
// ===========================================================================

#[test]
fn distinct_tokens_are_not_merged() {
    let grammar = GrammarBuilder::new("distinct")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A", "B"])
        .start("expr")
        .build();

    let mut g = grammar;
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert_eq!(stats.merged_tokens, 0);
}

// ===========================================================================
// 17. Non-left-recursive rules are left untouched
// ===========================================================================

#[test]
fn right_recursion_is_not_transformed() {
    // A -> B A  is right-recursive, NOT left-recursive.
    let grammar = GrammarBuilder::new("rr")
        .token("B", "b")
        .rule("expr", vec!["B", "expr"])
        .rule("expr", vec!["B"])
        .start("expr")
        .build();

    let mut g = grammar;
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    assert_eq!(
        stats.optimized_left_recursion, 0,
        "right recursion should not trigger left-recursion elimination"
    );
}

// ===========================================================================
// 18. Associativity preserved through left-recursion elimination
// ===========================================================================

#[test]
fn associativity_preserved_after_lr_elimination() {
    let grammar = GrammarBuilder::new("assoc_lr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "NUM"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();

    let has_right_assoc = optimized
        .all_rules()
        .any(|r| r.associativity == Some(Associativity::Right));
    assert!(
        has_right_assoc,
        "right associativity should survive left-recursion elimination"
    );
}

// ===========================================================================
// 19. Optimizer creates helper symbol name for LR transformation
// ===========================================================================

#[test]
fn lr_transform_creates_rec_helper_name() {
    let grammar = GrammarBuilder::new("lr_name")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "NUM"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let optimized = optimize_grammar(grammar).unwrap();

    let has_rec_name = optimized.rule_names.values().any(|n| n.contains("__rec"));
    assert!(
        has_rec_name,
        "left-recursion elimination should create an __rec helper symbol"
    );
}

// ===========================================================================
// 20. source_file symbol is never inlined
// ===========================================================================

#[test]
fn source_file_is_not_inlined() {
    let mut grammar = Grammar::new("sf_test".into());
    let sf = SymbolId(10);
    let inner = SymbolId(11);
    let tok = SymbolId(12);

    grammar.rule_names.insert(sf, "source_file".to_string());
    grammar.rule_names.insert(inner, "inner".to_string());
    grammar.tokens.insert(
        tok,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );

    // source_file -> inner (unit rule, single RHS)
    grammar.add_rule(Rule {
        lhs: sf,
        rhs: vec![Symbol::NonTerminal(inner)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    // inner -> a
    grammar.add_rule(Rule {
        lhs: inner,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let mut opt = GrammarOptimizer::new();
    opt.optimize(&mut grammar);

    // source_file should still exist as a named rule
    let has_sf = grammar.rule_names.values().any(|n| n == "source_file");
    assert!(has_sf, "source_file should never be inlined away");
}

// ===========================================================================
// 21. Epsilon-only grammar
// ===========================================================================

#[test]
fn epsilon_only_grammar_optimizes_without_panic() {
    let grammar = GrammarBuilder::new("eps_only")
        .rule("start", vec![])
        .start("start")
        .build();

    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
}

// ===========================================================================
// 22. Conflict declarations survive optimization
// ===========================================================================

#[test]
fn conflict_declarations_survive_optimization() {
    use adze_ir::{ConflictDeclaration, ConflictResolution};

    let mut grammar = GrammarBuilder::new("conflict")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A", "B"])
        .start("expr")
        .build();

    let a_id = grammar.find_symbol_by_name("expr").unwrap();
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![a_id],
        resolution: ConflictResolution::GLR,
    });

    let optimized = optimize_grammar(grammar).unwrap();
    assert!(
        !optimized.conflicts.is_empty(),
        "conflict declarations should survive optimization"
    );
}

// ===========================================================================
// 23. Supertypes survive optimization
// ===========================================================================

#[test]
fn supertypes_are_renumbered_during_optimization() {
    // Use a multi-symbol RHS to avoid inlining, so the rule survives and
    // the supertype ID remains in the renumbering map.
    let mut grammar = GrammarBuilder::new("super")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A", "B"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    grammar.supertypes.push(expr_id);

    let optimized = optimize_grammar(grammar).unwrap();
    assert!(
        !optimized.supertypes.is_empty(),
        "supertypes referencing known symbols should be renumbered"
    );
}

// ===========================================================================
// 24. Multiple left-recursive symbols
// ===========================================================================

#[test]
fn multiple_left_recursive_symbols_are_all_transformed() {
    let grammar = GrammarBuilder::new("multi_lr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "term"], 1, Associativity::Left)
        .rule("expr", vec!["term"])
        .rule_with_precedence("term", vec!["term", "*", "NUM"], 2, Associativity::Left)
        .rule("term", vec!["NUM"])
        .start("expr")
        .build();

    let mut g = grammar;
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    // Both expr and term are left-recursive
    assert!(
        stats.optimized_left_recursion >= 2,
        "both left-recursive symbols should be transformed, got {}",
        stats.optimized_left_recursion
    );
}
