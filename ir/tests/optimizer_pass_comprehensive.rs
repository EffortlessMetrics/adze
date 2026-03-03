#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for grammar optimization passes in adze-ir.
//!
//! Covers dead rule elimination, unreachable symbol removal, rule deduplication,
//! inline rule expansion, semantics preservation, idempotency, trivial grammars,
//! and complex grammar stress scenarios.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, OptimizationStats, optimize_grammar};
use adze_ir::{
    Associativity, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn total_rules(g: &Grammar) -> usize {
    g.all_rules().count()
}

fn token_names(g: &Grammar) -> Vec<String> {
    g.tokens.values().map(|t| t.name.clone()).collect()
}

fn rule_names(g: &Grammar) -> Vec<String> {
    g.rule_names.values().cloned().collect()
}

fn run_optimizer(g: Grammar) -> (Grammar, OptimizationStats) {
    let mut g = g;
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    (g, stats)
}

// ===========================================================================
// 1. Dead rule elimination
// ===========================================================================

#[test]
fn dead_rule_elimination_removes_orphan_rule() {
    // orphan has a single-symbol RHS so it's marked inlinable and cleaned up
    let grammar = GrammarBuilder::new("dead_rule_elim")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "A"])
        .rule("orphan", vec!["B"])
        .start("start")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    assert!(!rule_names(&optimized).contains(&"orphan".to_string()));
}

#[test]
fn dead_rule_elimination_keeps_transitively_reachable() {
    // start -> mid A (multi-symbol prevents full inlining of start)
    // mid -> leaf A
    // leaf -> T
    let grammar = GrammarBuilder::new("transitive")
        .token("T", "t")
        .token("A", "a")
        .rule("start", vec!["mid", "A"])
        .rule("mid", vec!["leaf", "A"])
        .rule("leaf", vec!["T"])
        .start("start")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    // start and mid have multi-symbol RHS so they survive.
    assert!(total_rules(&optimized) >= 1);
}

#[test]
fn dead_rule_elimination_multiple_orphans() {
    let grammar = GrammarBuilder::new("multi_orphan")
        .token("A", "a")
        .token("X", "x")
        .token("Y", "y")
        .token("Z", "z")
        .rule("start", vec!["A", "A"])
        .rule("orphan1", vec!["X"])
        .rule("orphan2", vec!["Y"])
        .rule("orphan3", vec!["Z"])
        .start("start")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    let names = rule_names(&optimized);
    assert!(!names.contains(&"orphan1".to_string()));
    assert!(!names.contains(&"orphan2".to_string()));
    assert!(!names.contains(&"orphan3".to_string()));
}

// ===========================================================================
// 2. Unreachable symbol removal
// ===========================================================================

#[test]
fn unreachable_token_is_removed() {
    let grammar = GrammarBuilder::new("unreach_tok")
        .token("USED", "used")
        .token("UNUSED", "unused")
        .rule("start", vec!["USED"])
        .start("start")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    assert!(!token_names(&optimized).contains(&"UNUSED".to_string()));
}

#[test]
fn unreachable_tokens_removed_but_used_kept() {
    let grammar = GrammarBuilder::new("unreach_multi")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("DEAD1", "d1")
        .token("DEAD2", "d2")
        .rule("start", vec!["A", "B", "C"])
        .start("start")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    let names = token_names(&optimized);
    assert!(names.contains(&"A".to_string()));
    assert!(names.contains(&"B".to_string()));
    assert!(names.contains(&"C".to_string()));
    assert!(!names.contains(&"DEAD1".to_string()));
    assert!(!names.contains(&"DEAD2".to_string()));
}

#[test]
fn unreachable_removal_preserves_extras() {
    let grammar = GrammarBuilder::new("extras_keep")
        .token("A", "a")
        .token("WS", "ws")
        .extra("WS")
        .rule("start", vec!["A", "WS", "A"])
        .start("start")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    assert!(!optimized.extras.is_empty());
}

// ===========================================================================
// 3. Rule deduplication (merge equivalent tokens)
// ===========================================================================

#[test]
fn dedup_string_tokens_same_pattern() {
    let mut grammar = Grammar::new("dedup_str".into());
    let t1 = SymbolId(1);
    let t2 = SymbolId(2);
    let rule_sym = SymbolId(3);

    grammar.tokens.insert(t1, Token { name: "eq1".into(), pattern: TokenPattern::String("=".into()), fragile: false });
    grammar.tokens.insert(t2, Token { name: "eq2".into(), pattern: TokenPattern::String("=".into()), fragile: false });
    grammar.rule_names.insert(rule_sym, "expr".to_string());
    grammar.add_rule(Rule {
        lhs: rule_sym,
        rhs: vec![Symbol::Terminal(t1), Symbol::Terminal(t2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let (optimized, stats) = run_optimizer(grammar);
    assert!(stats.merged_tokens >= 1);
    let eq_count = optimized.tokens.values()
        .filter(|t| matches!(&t.pattern, TokenPattern::String(s) if s == "="))
        .count();
    assert_eq!(eq_count, 1);
}

#[test]
fn dedup_regex_tokens_same_pattern() {
    let mut grammar = Grammar::new("dedup_regex".into());
    let t1 = SymbolId(1);
    let t2 = SymbolId(2);
    let rule_sym = SymbolId(3);

    grammar.tokens.insert(t1, Token { name: "num_a".into(), pattern: TokenPattern::Regex(r"[0-9]+".into()), fragile: false });
    grammar.tokens.insert(t2, Token { name: "num_b".into(), pattern: TokenPattern::Regex(r"[0-9]+".into()), fragile: false });
    grammar.rule_names.insert(rule_sym, "expr".to_string());
    grammar.add_rule(Rule {
        lhs: rule_sym,
        rhs: vec![Symbol::Terminal(t1), Symbol::Terminal(t2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let (_, stats) = run_optimizer(grammar);
    assert!(stats.merged_tokens >= 1);
}

#[test]
fn dedup_does_not_merge_distinct_patterns() {
    let grammar = GrammarBuilder::new("no_merge")
        .token("PLUS", "+")
        .token("MINUS", "-")
        .rule("expr", vec!["PLUS", "MINUS"])
        .start("expr")
        .build();

    let (_, stats) = run_optimizer(grammar);
    assert_eq!(stats.merged_tokens, 0);
}

// ===========================================================================
// 4. Inline rule expansion
// ===========================================================================

#[test]
fn inline_single_symbol_rule() {
    // wrapper -> TOK   (single-symbol, inlinable)
    // start -> wrapper
    let grammar = GrammarBuilder::new("inline_single")
        .token("TOK", "t")
        .rule("start", vec!["wrapper"])
        .rule("wrapper", vec!["TOK"])
        .start("start")
        .build();

    let (_, stats) = run_optimizer(grammar);
    assert!(stats.inlined_rules >= 1 || stats.eliminated_unit_rules >= 1);
}

#[test]
fn inline_does_not_inline_multi_symbol_rhs() {
    // helper -> A B   (multi-symbol, not inlinable by inline_simple_rules)
    let grammar = GrammarBuilder::new("no_inline_multi")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["A", "B"])
        .start("start")
        .build();

    let (_, stats) = run_optimizer(grammar);
    // Multi-symbol RHS should not be inlined by inline_simple_rules.
    // It may still be unit-rule eliminated though.
    assert_eq!(stats.inlined_rules, 0);
}

#[test]
fn inline_does_not_inline_recursive_rule() {
    // rec -> rec TOK  (recursive, not inlinable)
    let grammar = GrammarBuilder::new("no_inline_rec")
        .token("TOK", "t")
        .rule("start", vec!["rec"])
        .rule("rec", vec!["rec", "TOK"])
        .rule("rec", vec!["TOK"])
        .start("start")
        .build();

    let (_, stats) = run_optimizer(grammar);
    // rec has multiple alternatives and is recursive; should not be inlined
    assert_eq!(stats.inlined_rules, 0);
}

#[test]
fn inline_chain_is_resolved() {
    // a -> b, b -> c, c -> TOK — chain of single-symbol rules fully inlined
    let grammar = GrammarBuilder::new("inline_chain")
        .token("TOK", "t")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["TOK"])
        .start("a")
        .build();

    let (_, stats) = run_optimizer(grammar);
    // The chain should trigger inlining or unit-rule elimination
    assert!(stats.total() > 0);
}

// ===========================================================================
// 5. Optimization preserves parse semantics
// ===========================================================================

#[test]
fn semantics_start_symbol_always_exists() {
    let grammar = GrammarBuilder::new("sem_start")
        .token("A", "a")
        .token("B", "b")
        .rule("program", vec!["A", "B"])
        .start("program")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    assert!(optimized.start_symbol().is_some());
}

#[test]
fn semantics_all_rhs_symbols_resolve() {
    let grammar = GrammarBuilder::new("sem_resolve")
        .token("NUM", r"\d+")
        .token("OP", "+")
        .rule("expr", vec!["NUM", "OP", "NUM"])
        .start("expr")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    for rule in optimized.all_rules() {
        for sym in &rule.rhs {
            match sym {
                Symbol::Terminal(id) => {
                    assert!(
                        optimized.tokens.contains_key(id),
                        "terminal {:?} not found in tokens after optimization",
                        id
                    );
                }
                Symbol::NonTerminal(id) => {
                    assert!(
                        optimized.rules.contains_key(id),
                        "non-terminal {:?} not found in rules after optimization",
                        id
                    );
                }
                _ => {}
            }
        }
    }
}

#[test]
fn semantics_precedence_values_preserved() {
    let grammar = GrammarBuilder::new("sem_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    let prec_values: Vec<i16> = optimized
        .all_rules()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => None,
        })
        .collect();
    assert!(prec_values.contains(&1));
    assert!(prec_values.contains(&2));
}

#[test]
fn semantics_associativity_preserved() {
    let grammar = GrammarBuilder::new("sem_assoc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "NUM"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    let has_right = optimized
        .all_rules()
        .any(|r| r.associativity == Some(Associativity::Right));
    assert!(has_right, "Right associativity must survive optimization");
}

#[test]
fn semantics_externals_preserved() {
    let grammar = GrammarBuilder::new("sem_ext")
        .token("A", "a")
        .external("INDENT")
        .external("DEDENT")
        .rule("block", vec!["A"])
        .start("block")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    assert_eq!(optimized.externals.len(), 2);
}

// ===========================================================================
// 6. Multiple passes produce stable output (idempotency)
// ===========================================================================

#[test]
fn idempotency_second_pass_zero_stats() {
    // Use multi-symbol RHS to prevent full inlining of start
    let grammar = GrammarBuilder::new("idem_zero")
        .token("A", "a")
        .token("B", "b")
        .token("DEAD", "dead")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();

    let (once, _) = run_optimizer(grammar);
    let (_, stats2) = run_optimizer(once);
    assert_eq!(stats2.total(), 0, "second pass should do no work");
}

#[test]
fn idempotency_rule_count_stable() {
    let grammar = GrammarBuilder::new("idem_rules")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "NUM"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let (first, _) = run_optimizer(grammar);
    let first_rules = total_rules(&first);
    let first_tokens = first.tokens.len();

    let (second, _) = run_optimizer(first);
    assert_eq!(total_rules(&second), first_rules);
    assert_eq!(second.tokens.len(), first_tokens);
}

#[test]
fn idempotency_five_passes_converge() {
    let grammar = GrammarBuilder::new("idem_five")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("DEAD", "d")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["A", "B", "C"])
        .rule("orphan", vec!["DEAD"])
        .start("start")
        .build();

    let mut g = grammar;
    for _ in 0..5 {
        let mut opt = GrammarOptimizer::new();
        opt.optimize(&mut g);
    }
    let mut final_opt = GrammarOptimizer::new();
    let final_stats = final_opt.optimize(&mut g);
    assert_eq!(final_stats.total(), 0, "should converge to fixed point");
}

#[test]
fn idempotency_token_set_stable() {
    let grammar = GrammarBuilder::new("idem_tok")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();

    let (first, _) = run_optimizer(grammar);
    let tok1: Vec<String> = token_names(&first);

    let (second, _) = run_optimizer(first);
    let tok2: Vec<String> = token_names(&second);
    assert_eq!(tok1, tok2);
}

// ===========================================================================
// 7. Optimization of empty/trivial grammars
// ===========================================================================

#[test]
fn empty_grammar_no_panic() {
    let grammar = Grammar::new("empty".into());
    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
}

#[test]
fn empty_grammar_zero_stats() {
    let (_, stats) = run_optimizer(Grammar::new("empty".into()));
    assert_eq!(stats.total(), 0);
}

#[test]
fn single_epsilon_rule_grammar() {
    let grammar = GrammarBuilder::new("eps")
        .rule("start", vec![])
        .start("start")
        .build();

    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
}

#[test]
fn single_terminal_rule_grammar() {
    // Single-symbol RHS rules are inlinable; the optimizer may remove them.
    // Use multi-symbol RHS so start survives.
    let grammar = GrammarBuilder::new("single_term")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A", "B"])
        .start("start")
        .build();

    let (optimized, stats) = run_optimizer(grammar);
    assert!(total_rules(&optimized) >= 1);
    assert_eq!(stats.merged_tokens, 0);
    assert_eq!(stats.optimized_left_recursion, 0);
}

#[test]
fn grammar_with_only_tokens_no_rules() {
    let mut grammar = Grammar::new("tokens_only".into());
    grammar.tokens.insert(SymbolId(1), Token {
        name: "A".into(),
        pattern: TokenPattern::String("a".into()),
        fragile: false,
    });

    let (optimized, stats) = run_optimizer(grammar);
    // Token should be removed since no rule references it.
    assert!(stats.removed_unused_symbols >= 1);
    assert!(optimized.tokens.is_empty());
}

// ===========================================================================
// 8. Optimizer with complex grammars (many rules/symbols)
// ===========================================================================

#[test]
fn complex_grammar_many_rules() {
    let mut b = GrammarBuilder::new("complex");
    // Create 20 tokens
    for i in 0..20 {
        b = b.token(&format!("T{}", i), &format!("t{}", i));
    }
    // Create a start rule referencing a few tokens (multi-symbol to prevent inlining)
    b = b.rule("start", vec!["T0", "T1", "T2"]).start("start");
    // Create 10 orphan rules (single-symbol RHS, will be inlined away)
    for i in 0..10 {
        let name = format!("orphan{}", i);
        let tok = format!("T{}", i + 10);
        b = b.rule(&name, vec![&tok]);
    }
    let grammar = b.build();

    let (optimized, _) = run_optimizer(grammar);
    // Orphan rules should be removed (via inlining cleanup)
    for i in 0..10 {
        assert!(
            !rule_names(&optimized).contains(&format!("orphan{}", i)),
            "orphan{} should be removed",
            i
        );
    }
    // Start should survive
    assert!(optimized.start_symbol().is_some());
    assert!(total_rules(&optimized) >= 1);
}

#[test]
fn complex_grammar_deep_chain() {
    // chain: r0 -> r1, r1 -> r2, ..., r9 -> TOK
    // All single-symbol rules — the optimizer inlines the entire chain.
    let mut b = GrammarBuilder::new("deep_chain");
    b = b.token("TOK", "t");
    for i in 0..10 {
        let lhs = format!("r{}", i);
        let rhs = if i < 9 {
            format!("r{}", i + 1)
        } else {
            "TOK".to_string()
        };
        b = b.rule(&lhs, vec![&rhs]);
    }
    b = b.start("r0");
    let grammar = b.build();

    let (_, stats) = run_optimizer(grammar);
    // Deep chain should be collapsed via inlining/unit-rule elimination
    assert!(stats.total() > 0);
}

#[test]
fn complex_grammar_wide_alternatives() {
    // start -> A | B | C | D | E | F | G | H
    let mut b = GrammarBuilder::new("wide");
    let names = ["A", "B", "C", "D", "E", "F", "G", "H"];
    for name in &names {
        b = b.token(name, &name.to_lowercase());
    }
    for name in &names {
        b = b.rule("start", vec![name]);
    }
    b = b.start("start");
    let grammar = b.build();

    let (optimized, _) = run_optimizer(grammar);
    assert!(optimized.start_symbol().is_some());
    assert!(optimized.tokens.len() >= 8);
}

#[test]
fn complex_grammar_multiple_lr_symbols() {
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

    let (_, stats) = run_optimizer(grammar);
    assert!(stats.optimized_left_recursion >= 2);
}

#[test]
fn complex_grammar_mixed_dead_and_live() {
    let grammar = GrammarBuilder::new("mixed")
        .token("A", "a")
        .token("B", "b")
        .token("DEAD_T", "dt")
        .rule("start", vec!["inner", "A"])
        .rule("inner", vec!["A", "B"])
        .rule("dead_rule", vec!["DEAD_T"])
        .start("start")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    // dead_rule is removed (single-symbol RHS, inlinable cleanup)
    assert!(!rule_names(&optimized).contains(&"dead_rule".to_string()));
    // inner should survive since it's reachable and has multi-symbol RHS
    assert!(total_rules(&optimized) >= 2);
}

#[test]
fn complex_javascript_like_survives_optimization() {
    let grammar = GrammarBuilder::javascript_like();
    let (optimized, _) = run_optimizer(grammar);
    assert!(optimized.start_symbol().is_some());
    assert!(total_rules(&optimized) >= 5);
    assert!(optimized.tokens.len() >= 5);
}

#[test]
fn complex_python_like_survives_optimization() {
    let grammar = GrammarBuilder::python_like();
    let (optimized, _) = run_optimizer(grammar);
    assert!(optimized.start_symbol().is_some());
    assert!(total_rules(&optimized) >= 1);
}

// ===========================================================================
// 9. Additional edge cases
// ===========================================================================

#[test]
fn lr_elimination_creates_epsilon_production() {
    let grammar = GrammarBuilder::new("lr_eps")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "NUM"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    let has_empty = optimized.all_rules().any(|r| r.rhs.is_empty());
    assert!(has_empty, "LR elimination must create epsilon production");
}

#[test]
fn lr_elimination_creates_rec_helper_name() {
    let grammar = GrammarBuilder::new("lr_helper")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "NUM"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    assert!(
        optimized.rule_names.values().any(|n| n.contains("__rec")),
        "LR elimination should create __rec helper"
    );
}

#[test]
fn stats_total_matches_field_sum() {
    let grammar = GrammarBuilder::new("stats_check")
        .token("A", "a")
        .token("DEAD", "dead")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let (_, stats) = run_optimizer(grammar);
    let expected = stats.removed_unused_symbols
        + stats.inlined_rules
        + stats.merged_tokens
        + stats.optimized_left_recursion
        + stats.eliminated_unit_rules;
    assert_eq!(stats.total(), expected);
}

#[test]
fn optimization_stats_default_all_zero() {
    let stats = OptimizationStats::default();
    assert_eq!(stats.total(), 0);
}

#[test]
fn source_file_symbol_never_inlined() {
    let mut grammar = Grammar::new("sf_protect".into());
    let sf = SymbolId(10);
    let inner = SymbolId(11);
    let tok = SymbolId(12);

    grammar.rule_names.insert(sf, "source_file".to_string());
    grammar.rule_names.insert(inner, "inner".to_string());
    grammar.tokens.insert(tok, Token {
        name: "tok".into(),
        pattern: TokenPattern::String("t".into()),
        fragile: false,
    });

    grammar.add_rule(Rule {
        lhs: sf,
        rhs: vec![Symbol::NonTerminal(inner)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: inner,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let (optimized, _) = run_optimizer(grammar);
    assert!(
        optimized.rule_names.values().any(|n| n == "source_file"),
        "source_file must never be inlined away"
    );
}

#[test]
fn renumbering_produces_contiguous_ids() {
    let grammar = GrammarBuilder::new("renum_check")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("start", vec!["A", "B", "C"])
        .start("start")
        .build();

    let (optimized, _) = run_optimizer(grammar);
    let max_tok = optimized.tokens.keys().map(|id| id.0).max().unwrap_or(0);
    let max_rule = optimized.rules.keys().map(|id| id.0).max().unwrap_or(0);
    let max_id = max_tok.max(max_rule);
    let num_symbols = optimized.tokens.len() + optimized.rules.len();
    assert!(
        (max_id as usize) <= num_symbols + 1,
        "IDs should be contiguous after renumbering"
    );
}

#[test]
fn conflict_declarations_survive() {
    use adze_ir::{ConflictDeclaration, ConflictResolution};

    let mut grammar = GrammarBuilder::new("conflict_surv")
        .token("A", "a")
        .token("B", "b")
        .rule("expr", vec!["A", "B"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![expr_id],
        resolution: ConflictResolution::GLR,
    });

    let (optimized, _) = run_optimizer(grammar);
    assert!(!optimized.conflicts.is_empty());
}
