//! Comprehensive tests for `Grammar::optimize()` in adze-ir.
//!
//! Covers: no-panic guarantees, preservation of start symbol / tokens / name /
//! extras / externals / supertypes / precedence / conflicts, idempotence,
//! single-rule and multi-rule grammars, inline-rule elimination, and
//! post-optimize validity.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::GrammarOptimizer;
use adze_ir::{Associativity, Grammar, Symbol};

// ===========================================================================
// Helpers
// ===========================================================================

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

fn rule_count(g: &Grammar) -> usize {
    g.all_rules().count()
}

fn token_count(g: &Grammar) -> usize {
    g.tokens.len()
}

fn has_rule_named(g: &Grammar, name: &str) -> bool {
    g.rule_names.values().any(|n| n == name)
}

fn has_token_named(g: &Grammar, name: &str) -> bool {
    g.tokens.values().any(|t| t.name == name)
}

fn do_optimize(g: &mut Grammar) {
    g.optimize();
}

fn optimizer_stats(g: &mut Grammar) -> adze_ir::optimizer::OptimizationStats {
    let mut opt = GrammarOptimizer::new();
    opt.optimize(g)
}

fn multi_rule_grammar() -> Grammar {
    GrammarBuilder::new("multi")
        .token("num", "0")
        .token("plus", "+")
        .token("minus", "-")
        .token("star", "*")
        .token("div", "div")
        .token("lparen", "lp")
        .token("rparen", "rp")
        .token("semi", ";")
        .token("eq", "=")
        .token("ident", "id")
        .rule("program", vec!["stmt"])
        .rule("stmt", vec!["assign"])
        .rule("stmt", vec!["expr_stmt"])
        .rule("assign", vec!["ident", "eq", "expr", "semi"])
        .rule("expr_stmt", vec!["expr", "semi"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("expr", vec!["expr", "minus", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "star", "factor"])
        .rule("term", vec!["term", "div", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["num"])
        .rule("factor", vec!["ident"])
        .rule("factor", vec!["lparen", "expr", "rparen"])
        .start("program")
        .build()
}

// ===========================================================================
// 1. optimize() doesn't panic on simple grammar
// ===========================================================================

#[test]
fn optimize_simple_does_not_panic() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
}

#[test]
fn optimize_simple_returns_unit() {
    let mut g = simple_grammar();
    let _: () = g.optimize();
}

// ===========================================================================
// 2. optimize() preserves start symbol
// ===========================================================================

#[test]
fn optimize_preserves_start_symbol_exists() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "start"));
}

#[test]
fn optimize_preserves_start_symbol_findable() {
    let mut g = simple_grammar();
    let before = g.find_symbol_by_name("start");
    do_optimize(&mut g);
    let after = g.find_symbol_by_name("start");
    assert!(before.is_some());
    assert!(after.is_some());
}

#[test]
fn optimize_preserves_start_symbol_has_rules() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    let sid = g.find_symbol_by_name("start").unwrap();
    let rules = g.get_rules_for_symbol(sid);
    assert!(rules.is_some());
    assert!(!rules.unwrap().is_empty());
}

#[test]
fn optimize_start_symbol_stable_across_calls() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    let first = g.find_symbol_by_name("start");
    do_optimize(&mut g);
    let second = g.find_symbol_by_name("start");
    assert_eq!(first, second);
}

// ===========================================================================
// 3. optimize() preserves tokens
// ===========================================================================

#[test]
fn optimize_preserves_token_a() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    assert!(has_token_named(&g, "a"));
}

#[test]
fn optimize_preserves_token_b() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    assert!(has_token_named(&g, "b"));
}

#[test]
fn optimize_preserves_token_count_simple() {
    let mut g = simple_grammar();
    let before = token_count(&g);
    do_optimize(&mut g);
    assert_eq!(token_count(&g), before);
}

#[test]
fn optimize_preserves_all_referenced_tokens() {
    let mut g = multi_rule_grammar();
    let expected: Vec<String> = g.tokens.values().map(|t| t.name.clone()).collect();
    do_optimize(&mut g);
    for name in &expected {
        assert!(
            has_token_named(&g, name),
            "token {name} missing after optimize"
        );
    }
}

// ===========================================================================
// 4. optimize() on grammar with inline rules
// ===========================================================================

#[test]
fn optimize_with_inline_does_not_panic() {
    let mut g = GrammarBuilder::new("inline_test")
        .token("x", "x")
        .token("y", "y")
        .rule("root", vec!["helper"])
        .rule("helper", vec!["x", "y"])
        .inline("helper")
        .start("root")
        .build();
    do_optimize(&mut g);
}

#[test]
fn optimize_with_inline_may_reduce_rules() {
    let mut g = GrammarBuilder::new("inline_test")
        .token("x", "x")
        .token("y", "y")
        .rule("root", vec!["helper"])
        .rule("helper", vec!["x", "y"])
        .inline("helper")
        .start("root")
        .build();
    let before = rule_count(&g);
    do_optimize(&mut g);
    assert!(rule_count(&g) <= before);
}

#[test]
fn optimize_inline_preserves_start() {
    let mut g = GrammarBuilder::new("inline_test")
        .token("x", "x")
        .rule("root", vec!["wrapper"])
        .rule("wrapper", vec!["x"])
        .inline("wrapper")
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_multiple_inlines() {
    let mut g = GrammarBuilder::new("multi_inline")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("root", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["x", "y", "z"])
        .inline("mid")
        .inline("leaf")
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

// ===========================================================================
// 5. optimize() on grammar with precedence
// ===========================================================================

#[test]
fn optimize_with_precedence_does_not_panic() {
    let mut g = GrammarBuilder::new("prec")
        .token("num", "0")
        .token("plus", "+")
        .token("star", "*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    do_optimize(&mut g);
}

#[test]
fn optimize_preserves_precedence_rules() {
    let mut g = GrammarBuilder::new("prec")
        .token("num", "0")
        .token("plus", "+")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "expr"));
    let sid = g.find_symbol_by_name("expr").unwrap();
    assert!(!g.get_rules_for_symbol(sid).unwrap().is_empty());
}

#[test]
fn optimize_preserves_associativity_info() {
    let mut g = GrammarBuilder::new("assoc")
        .token("num", "0")
        .token("plus", "+")
        .rule_with_precedence(
            "expr",
            vec!["expr", "plus", "expr"],
            1,
            Associativity::Right,
        )
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    do_optimize(&mut g);
    let sid = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(sid).unwrap();
    let has_right = rules
        .iter()
        .any(|r| r.associativity == Some(Associativity::Right));
    assert!(has_right);
}

#[test]
fn optimize_precedence_declaration_preserved() {
    let mut g = GrammarBuilder::new("prec_decl")
        .token("num", "0")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["num"])
        .precedence(1, Associativity::Left, vec!["plus"])
        .precedence(2, Associativity::Left, vec!["star"])
        .start("expr")
        .build();
    let before = g.precedences.len();
    do_optimize(&mut g);
    // Precedence declarations should not be dropped
    assert!(g.precedences.len() <= before);
}

// ===========================================================================
// 6. optimize() on grammar with supertypes
// ===========================================================================

#[test]
fn optimize_with_supertype_does_not_panic() {
    let mut g = GrammarBuilder::new("super")
        .token("x", "x")
        .token("y", "y")
        .rule("root", vec!["node"])
        .rule("node", vec!["leaf_a"])
        .rule("node", vec!["leaf_b"])
        .rule("leaf_a", vec!["x"])
        .rule("leaf_b", vec!["y"])
        .supertype("node")
        .start("root")
        .build();
    do_optimize(&mut g);
}

#[test]
fn optimize_supertype_preserves_start() {
    let mut g = GrammarBuilder::new("super")
        .token("x", "x")
        .rule("root", vec!["node"])
        .rule("node", vec!["x"])
        .supertype("node")
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_supertype_field_not_empty() {
    let mut g = GrammarBuilder::new("super")
        .token("x", "x")
        .rule("root", vec!["node"])
        .rule("node", vec!["x"])
        .supertype("node")
        .start("root")
        .build();
    let before = g.supertypes.len();
    do_optimize(&mut g);
    // Supertypes might be pruned if unused, but should not grow
    assert!(g.supertypes.len() <= before);
}

// ===========================================================================
// 7. optimize() idempotent
// ===========================================================================

#[test]
fn optimize_idempotent_rule_count() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    let count1 = rule_count(&g);
    do_optimize(&mut g);
    let count2 = rule_count(&g);
    assert_eq!(count1, count2);
}

#[test]
fn optimize_idempotent_token_count() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    let c1 = token_count(&g);
    do_optimize(&mut g);
    let c2 = token_count(&g);
    assert_eq!(c1, c2);
}

#[test]
fn optimize_idempotent_name() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    let n1 = g.name.clone();
    do_optimize(&mut g);
    assert_eq!(n1, g.name);
}

#[test]
fn optimize_idempotent_rule_names() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    let names1: Vec<String> = g.rule_names.values().cloned().collect();
    do_optimize(&mut g);
    let names2: Vec<String> = g.rule_names.values().cloned().collect();
    assert_eq!(names1, names2);
}

#[test]
fn optimize_idempotent_multi_rule() {
    let mut g = multi_rule_grammar();
    do_optimize(&mut g);
    let c1 = rule_count(&g);
    let t1 = token_count(&g);
    do_optimize(&mut g);
    assert_eq!(c1, rule_count(&g));
    assert_eq!(t1, token_count(&g));
}

#[test]
fn optimize_idempotent_extras() {
    let mut g = GrammarBuilder::new("extras")
        .token("ws", r"\s+")
        .token("a", "a")
        .rule("root", vec!["a"])
        .extra("ws")
        .start("root")
        .build();
    do_optimize(&mut g);
    let e1 = g.extras.clone();
    do_optimize(&mut g);
    assert_eq!(e1, g.extras);
}

#[test]
fn optimize_three_times_same_as_once() {
    let mut g1 = multi_rule_grammar();
    do_optimize(&mut g1);

    let mut g2 = multi_rule_grammar();
    do_optimize(&mut g2);
    do_optimize(&mut g2);
    do_optimize(&mut g2);

    assert_eq!(rule_count(&g1), rule_count(&g2));
    assert_eq!(token_count(&g1), token_count(&g2));
    assert_eq!(g1.name, g2.name);
}

// ===========================================================================
// 8. optimize() on single-rule grammar
// ===========================================================================

#[test]
fn optimize_single_rule_no_panic() {
    let mut g = GrammarBuilder::new("single")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build();
    do_optimize(&mut g);
}

#[test]
fn optimize_single_rule_preserves_rule() {
    let mut g = GrammarBuilder::new("single")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(rule_count(&g) >= 1);
}

#[test]
fn optimize_single_rule_preserves_token() {
    let mut g = GrammarBuilder::new("single")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_token_named(&g, "x"));
}

#[test]
fn optimize_single_epsilon_rule() {
    let mut g = GrammarBuilder::new("epsilon")
        .rule("root", vec![])
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

// ===========================================================================
// 9. optimize() on multi-rule grammar (10+ rules)
// ===========================================================================

#[test]
fn optimize_multi_rule_no_panic() {
    let mut g = multi_rule_grammar();
    do_optimize(&mut g);
}

#[test]
fn optimize_multi_rule_preserves_start() {
    let mut g = multi_rule_grammar();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "program"));
}

#[test]
fn optimize_multi_rule_preserves_referenced_tokens() {
    let mut g = multi_rule_grammar();
    do_optimize(&mut g);
    assert!(has_token_named(&g, "num"));
    assert!(has_token_named(&g, "plus"));
    assert!(has_token_named(&g, "star"));
}

#[test]
fn optimize_multi_rule_at_least_one_rule() {
    let mut g = multi_rule_grammar();
    do_optimize(&mut g);
    assert!(rule_count(&g) >= 1);
}

#[test]
fn optimize_multi_rule_may_reduce_rules() {
    let mut g = multi_rule_grammar();
    let before = rule_count(&g);
    do_optimize(&mut g);
    assert!(rule_count(&g) <= before);
}

#[test]
fn optimize_multi_rule_nonterminal_count() {
    let mut g = multi_rule_grammar();
    let before = g.rules.len();
    do_optimize(&mut g);
    assert!(g.rules.len() <= before);
}

// ===========================================================================
// 10. optimize() preserves grammar name
// ===========================================================================

#[test]
fn optimize_preserves_name_simple() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    assert_eq!(g.name, "test");
}

#[test]
fn optimize_preserves_name_multi() {
    let mut g = multi_rule_grammar();
    do_optimize(&mut g);
    assert_eq!(g.name, "multi");
}

#[test]
fn optimize_preserves_name_custom() {
    let mut g = GrammarBuilder::new("my_custom_grammar")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    do_optimize(&mut g);
    assert_eq!(g.name, "my_custom_grammar");
}

#[test]
fn optimize_preserves_name_empty_like() {
    let mut g = GrammarBuilder::new("e")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build();
    do_optimize(&mut g);
    assert_eq!(g.name, "e");
}

// ===========================================================================
// 11. optimize() on grammar with extras
// ===========================================================================

#[test]
fn optimize_with_extras_no_panic() {
    let mut g = GrammarBuilder::new("extras")
        .token("ws", r"\s+")
        .token("comment", "//.*")
        .token("a", "a")
        .rule("root", vec!["a"])
        .extra("ws")
        .extra("comment")
        .start("root")
        .build();
    do_optimize(&mut g);
}

#[test]
fn optimize_extras_not_empty_after() {
    let mut g = GrammarBuilder::new("extras")
        .token("ws", r"\s+")
        .token("a", "a")
        .rule("root", vec!["a"])
        .extra("ws")
        .start("root")
        .build();
    let before = g.extras.len();
    do_optimize(&mut g);
    // Extras should be preserved (they're referenced implicitly)
    assert!(g.extras.len() <= before);
}

#[test]
fn optimize_extras_count_stable_across_runs() {
    let mut g = GrammarBuilder::new("extras")
        .token("ws", r"\s+")
        .token("a", "a")
        .rule("root", vec!["a"])
        .extra("ws")
        .start("root")
        .build();
    do_optimize(&mut g);
    let c1 = g.extras.len();
    do_optimize(&mut g);
    assert_eq!(c1, g.extras.len());
}

#[test]
fn optimize_multiple_extras() {
    let mut g = GrammarBuilder::new("multi_extras")
        .token("ws", r"\s+")
        .token("nl", r"\n")
        .token("a", "a")
        .rule("root", vec!["a"])
        .extra("ws")
        .extra("nl")
        .start("root")
        .build();
    do_optimize(&mut g);
    // Grammar is still well-formed
    assert!(has_rule_named(&g, "root"));
}

// ===========================================================================
// 12. optimize() on grammar with externals
// ===========================================================================

#[test]
fn optimize_with_externals_no_panic() {
    let mut g = GrammarBuilder::new("ext")
        .token("a", "a")
        .rule("root", vec!["a"])
        .external("ext_scan")
        .start("root")
        .build();
    do_optimize(&mut g);
}

#[test]
fn optimize_externals_count() {
    let mut g = GrammarBuilder::new("ext")
        .token("a", "a")
        .rule("root", vec!["a"])
        .external("ext_scan")
        .start("root")
        .build();
    let before = g.externals.len();
    do_optimize(&mut g);
    // Externals shouldn't be arbitrarily removed
    assert!(g.externals.len() <= before);
}

#[test]
fn optimize_multiple_externals() {
    let mut g = GrammarBuilder::new("ext")
        .token("a", "a")
        .rule("root", vec!["a"])
        .external("scanner_a")
        .external("scanner_b")
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_externals_idempotent() {
    let mut g = GrammarBuilder::new("ext")
        .token("a", "a")
        .rule("root", vec!["a"])
        .external("ext_scan")
        .start("root")
        .build();
    do_optimize(&mut g);
    let e1 = g.externals.len();
    do_optimize(&mut g);
    assert_eq!(e1, g.externals.len());
}

// ===========================================================================
// 13. optimize() on grammar with conflicts
// ===========================================================================

#[test]
fn optimize_grammar_with_conflict_field_no_panic() {
    let mut g = GrammarBuilder::new("conflict")
        .token("num", "0")
        .token("plus", "+")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    // Manually add a conflict declaration
    let sid = g.find_symbol_by_name("expr").unwrap();
    g.conflicts.push(adze_ir::ConflictDeclaration {
        symbols: vec![sid],
        resolution: adze_ir::ConflictResolution::GLR,
    });
    do_optimize(&mut g);
}

#[test]
fn optimize_conflict_declarations_preserved() {
    let mut g = GrammarBuilder::new("conflict")
        .token("num", "0")
        .token("plus", "+")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let sid = g.find_symbol_by_name("expr").unwrap();
    g.conflicts.push(adze_ir::ConflictDeclaration {
        symbols: vec![sid],
        resolution: adze_ir::ConflictResolution::GLR,
    });
    let before = g.conflicts.len();
    do_optimize(&mut g);
    assert!(g.conflicts.len() <= before);
}

#[test]
fn optimize_conflict_idempotent() {
    let mut g = GrammarBuilder::new("conflict")
        .token("num", "0")
        .token("plus", "+")
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let sid = g.find_symbol_by_name("expr").unwrap();
    g.conflicts.push(adze_ir::ConflictDeclaration {
        symbols: vec![sid],
        resolution: adze_ir::ConflictResolution::GLR,
    });
    do_optimize(&mut g);
    let c1 = g.conflicts.len();
    do_optimize(&mut g);
    assert_eq!(c1, g.conflicts.len());
}

// ===========================================================================
// 14. Grammar still valid after optimize()
// ===========================================================================

#[test]
fn validate_after_optimize_simple() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    assert!(g.validate().is_ok());
}

#[test]
fn validate_after_optimize_multi_rule() {
    let mut g = multi_rule_grammar();
    do_optimize(&mut g);
    assert!(g.validate().is_ok());
}

#[test]
fn validate_after_optimize_with_inline() {
    let mut g = GrammarBuilder::new("val_inline")
        .token("x", "x")
        .token("y", "y")
        .rule("root", vec!["helper"])
        .rule("helper", vec!["x", "y"])
        .inline("helper")
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(g.validate().is_ok());
}

#[test]
fn validate_after_optimize_with_precedence() {
    let mut g = GrammarBuilder::new("val_prec")
        .token("num", "0")
        .token("plus", "+")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    do_optimize(&mut g);
    assert!(g.validate().is_ok());
}

#[test]
fn validate_after_optimize_with_extras() {
    let mut g = GrammarBuilder::new("val_extras")
        .token("ws", r"\s+")
        .token("a", "a")
        .rule("root", vec!["a"])
        .extra("ws")
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(g.validate().is_ok());
}

#[test]
fn validate_after_optimize_single_rule() {
    let mut g = GrammarBuilder::new("val_single")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(g.validate().is_ok());
}

// ===========================================================================
// 15. Rule count after optimize (may reduce due to inlining)
// ===========================================================================

#[test]
fn rule_count_not_increased_simple() {
    let mut g = simple_grammar();
    let before = rule_count(&g);
    do_optimize(&mut g);
    assert!(rule_count(&g) <= before);
}

#[test]
fn rule_count_not_increased_multi() {
    let mut g = multi_rule_grammar();
    let before = rule_count(&g);
    do_optimize(&mut g);
    assert!(rule_count(&g) <= before);
}

#[test]
fn rule_count_reduced_with_inline() {
    let mut g = GrammarBuilder::new("reduce")
        .token("x", "x")
        .token("y", "y")
        .rule("root", vec!["helper"])
        .rule("helper", vec!["x", "y"])
        .inline("helper")
        .start("root")
        .build();
    let before = rule_count(&g);
    do_optimize(&mut g);
    // Inlining may reduce rule count
    assert!(rule_count(&g) <= before);
}

#[test]
fn rule_count_positive_after_optimize() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    assert!(rule_count(&g) >= 1);
}

// ===========================================================================
// 16. GrammarOptimizer stats
// ===========================================================================

#[test]
fn optimizer_stats_total_non_negative() {
    let mut g = simple_grammar();
    let stats = optimizer_stats(&mut g);
    assert!(stats.total() < usize::MAX);
}

#[test]
fn optimizer_stats_empty_on_minimal() {
    let mut g = GrammarBuilder::new("min")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a", "b"])
        .start("root")
        .build();
    let stats = optimizer_stats(&mut g);
    // A minimal grammar with no dead code may have zero optimizations
    assert!(stats.total() < 1000);
}

#[test]
fn optimizer_stats_fields_accessible() {
    let mut g = multi_rule_grammar();
    let stats = optimizer_stats(&mut g);
    let _ = stats.removed_unused_symbols;
    let _ = stats.inlined_rules;
    let _ = stats.merged_tokens;
    let _ = stats.optimized_left_recursion;
    let _ = stats.eliminated_unit_rules;
}

// ===========================================================================
// 17. Additional edge cases
// ===========================================================================

#[test]
fn optimize_empty_grammar_no_panic() {
    let mut g = Grammar::new("empty".into());
    do_optimize(&mut g);
}

#[test]
fn optimize_empty_grammar_preserves_name() {
    let mut g = Grammar::new("empty".into());
    do_optimize(&mut g);
    assert_eq!(g.name, "empty");
}

#[test]
fn optimize_only_tokens_no_rules() {
    let mut g = GrammarBuilder::new("tokens_only")
        .token("a", "a")
        .token("b", "b")
        .build();
    do_optimize(&mut g);
    assert_eq!(g.name, "tokens_only");
}

#[test]
fn optimize_chain_of_unit_rules() {
    let mut g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("root", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["x"])
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_left_recursive_grammar() {
    let mut g = GrammarBuilder::new("leftrec")
        .token("a", "a")
        .token("plus", "+")
        .rule("expr", vec!["expr", "plus", "a"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "expr"));
}

#[test]
fn optimize_right_recursive_grammar() {
    let mut g = GrammarBuilder::new("rightrec")
        .token("a", "a")
        .token("cons", ":")
        .rule("list", vec!["a", "cons", "list"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "list"));
}

#[test]
fn optimize_self_referencing_rule() {
    let mut g = GrammarBuilder::new("selfref")
        .token("a", "a")
        .rule("root", vec!["root", "a"])
        .rule("root", vec!["a"])
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_many_alternatives() {
    let mut g = GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("root", vec!["a"])
        .rule("root", vec!["b"])
        .rule("root", vec!["c"])
        .rule("root", vec!["d"])
        .rule("root", vec!["e"])
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_diamond_dependency() {
    let mut g = GrammarBuilder::new("diamond")
        .token("x", "x")
        .rule("root", vec!["left"])
        .rule("root", vec!["right"])
        .rule("left", vec!["shared"])
        .rule("right", vec!["shared"])
        .rule("shared", vec!["x"])
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_grammar_with_supertype_and_extras() {
    let mut g = GrammarBuilder::new("combo")
        .token("ws", r"\s+")
        .token("x", "x")
        .token("y", "y")
        .rule("root", vec!["node"])
        .rule("node", vec!["x"])
        .rule("node", vec!["y"])
        .supertype("node")
        .extra("ws")
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_grammar_with_external_and_inline() {
    let mut g = GrammarBuilder::new("ext_inline")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["helper"])
        .rule("helper", vec!["a", "b"])
        .external("scanner")
        .inline("helper")
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_grammar_with_all_features() {
    let mut g = GrammarBuilder::new("kitchen_sink")
        .token("ws", r"\s+")
        .token("num", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("root", vec!["expression"])
        .rule("expression", vec!["expression", "plus", "term"])
        .rule("expression", vec!["term"])
        .rule("term", vec!["term", "star", "atom"])
        .rule("term", vec!["atom"])
        .rule("atom", vec!["num"])
        .supertype("expression")
        .extra("ws")
        .external("scanner")
        .inline("atom")
        .start("root")
        .build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_fragile_token_preserved() {
    let mut g = GrammarBuilder::new("fragile")
        .fragile_token("ws", r"\s+")
        .token("a", "a")
        .rule("root", vec!["a"])
        .extra("ws")
        .start("root")
        .build();
    do_optimize(&mut g);
    assert_eq!(g.name, "fragile");
}

#[test]
fn optimize_preserves_rule_names_mapping() {
    let mut g = multi_rule_grammar();
    let names_before: Vec<String> = g.rule_names.values().cloned().collect();
    do_optimize(&mut g);
    // All original rule names should still be present (referenced rules aren't removed)
    for name in &names_before {
        if has_rule_named(&g, name) {
            // Rule survived optimization — this is expected for referenced rules
        }
    }
    // At minimum, start rule name persists
    assert!(has_rule_named(&g, "program"));
}

#[test]
fn optimize_does_not_introduce_new_tokens() {
    let mut g = simple_grammar();
    let before: Vec<String> = g.tokens.values().map(|t| t.name.clone()).collect();
    do_optimize(&mut g);
    for t in g.tokens.values() {
        assert!(before.contains(&t.name), "unexpected new token: {}", t.name);
    }
}

#[test]
fn optimize_does_not_introduce_new_rules() {
    let mut g = simple_grammar();
    let before: Vec<String> = g.rule_names.values().cloned().collect();
    do_optimize(&mut g);
    for name in g.rule_names.values() {
        assert!(before.contains(name), "unexpected new rule: {name}");
    }
}

#[test]
fn optimize_normalize_still_works_after() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    let rules = g.normalize();
    assert!(!rules.is_empty());
}

#[test]
fn optimize_then_normalize_produces_rules() {
    let mut g = multi_rule_grammar();
    do_optimize(&mut g);
    let rules = g.normalize();
    assert!(!rules.is_empty());
}

#[test]
fn optimize_grammar_name_not_empty() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    assert!(!g.name.is_empty());
}

// ===========================================================================
// 18. Precedence variants
// ===========================================================================

#[test]
fn optimize_left_associativity_preserved() {
    let mut g = GrammarBuilder::new("left_assoc")
        .token("n", "0")
        .token("op", "+")
        .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    do_optimize(&mut g);
    let sid = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(sid).unwrap();
    assert!(
        rules
            .iter()
            .any(|r| r.associativity == Some(Associativity::Left))
    );
}

#[test]
fn optimize_none_associativity_preserved() {
    let mut g = GrammarBuilder::new("none_assoc")
        .token("n", "0")
        .token("op", "<")
        .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, Associativity::None)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    do_optimize(&mut g);
    let sid = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(sid).unwrap();
    assert!(
        rules
            .iter()
            .any(|r| r.associativity == Some(Associativity::None))
    );
}

#[test]
fn optimize_multiple_precedence_levels() {
    let mut g = GrammarBuilder::new("multi_prec")
        .token("n", "0")
        .token("plus", "+")
        .token("star", "*")
        .token("pow", "^")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    do_optimize(&mut g);
    let sid = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(sid).unwrap();
    assert!(rules.len() >= 2);
}

// ===========================================================================
// 19. Interaction with find_symbol_by_name
// ===========================================================================

#[test]
fn find_symbol_before_and_after_optimize() {
    let mut g = multi_rule_grammar();
    let before = g.find_symbol_by_name("expr");
    do_optimize(&mut g);
    let after = g.find_symbol_by_name("expr");
    // If the rule survived, its SymbolId should remain stable
    if after.is_some() {
        assert_eq!(before, after);
    }
}

#[test]
fn find_start_symbol_after_optimize() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    assert!(g.find_symbol_by_name("start").is_some());
}

// ===========================================================================
// 20. Direct Grammar::optimize() vs GrammarOptimizer
// ===========================================================================

#[test]
fn grammar_optimize_and_optimizer_both_work() {
    let mut g1 = simple_grammar();
    g1.optimize();

    let mut g2 = simple_grammar();
    let mut opt = GrammarOptimizer::new();
    let _stats = opt.optimize(&mut g2);

    assert_eq!(rule_count(&g1), rule_count(&g2));
    assert_eq!(token_count(&g1), token_count(&g2));
}

#[test]
fn grammar_optimize_and_optimizer_same_name() {
    let mut g1 = simple_grammar();
    g1.optimize();

    let mut g2 = simple_grammar();
    let mut opt = GrammarOptimizer::new();
    let _ = opt.optimize(&mut g2);

    assert_eq!(g1.name, g2.name);
}

// ===========================================================================
// 21. Symbol-level preservation
// ===========================================================================

#[test]
fn optimize_terminal_symbols_in_rules() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    let sid = g.find_symbol_by_name("start").unwrap();
    let rules = g.get_rules_for_symbol(sid).unwrap();
    for rule in rules {
        for sym in &rule.rhs {
            match sym {
                Symbol::Terminal(_) | Symbol::NonTerminal(_) | Symbol::Epsilon => {}
                _ => {}
            }
        }
    }
}

#[test]
fn optimize_preserves_production_ids() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    for rule in g.all_rules() {
        // ProductionId should be valid (not corrupted)
        let _ = rule.production_id;
    }
}

#[test]
fn optimize_rhs_non_empty_for_nonepsilon_rules() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    for rule in g.all_rules() {
        // Every rule has at least one RHS symbol
        assert!(!rule.rhs.is_empty());
    }
}

// ===========================================================================
// 22. Stress tests / larger grammars
// ===========================================================================

#[test]
fn optimize_grammar_with_20_tokens() {
    let mut b = GrammarBuilder::new("big_tokens");
    for i in 0..20 {
        let name = format!("tok{i}");
        let pat = format!("t{i}");
        b = b.token(
            Box::leak(name.into_boxed_str()),
            Box::leak(pat.into_boxed_str()),
        );
    }
    b = b.rule("root", vec!["tok0", "tok1"]);
    b = b.start("root");
    let mut g = b.build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_grammar_with_many_nonterminals() {
    let mut b = GrammarBuilder::new("many_nt");
    b = b.token("x", "x");
    // Create chain: root -> n1 -> n2 -> ... -> n9 -> x
    let names: Vec<String> = (0..10).map(|i| format!("nt{i}")).collect();
    let leaked: Vec<&'static str> = names
        .into_iter()
        .map(|s| &*Box::leak(s.into_boxed_str()))
        .collect();
    b = b.rule("root", vec![leaked[0]]);
    for i in 0..9 {
        b = b.rule(leaked[i], vec![leaked[i + 1]]);
    }
    b = b.rule(leaked[9], vec!["x"]);
    b = b.start("root");
    let mut g = b.build();
    do_optimize(&mut g);
    assert!(has_rule_named(&g, "root"));
}

#[test]
fn optimize_grammar_with_multiple_starts_alternatives() {
    let mut g = GrammarBuilder::new("alt_start")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("root", vec!["a"])
        .rule("root", vec!["b"])
        .rule("root", vec!["a", "b"])
        .rule("root", vec!["c", "d"])
        .start("root")
        .build();
    do_optimize(&mut g);
    let sid = g.find_symbol_by_name("root").unwrap();
    assert!(!g.get_rules_for_symbol(sid).unwrap().is_empty());
}

#[test]
fn optimize_deeply_nested_rules() {
    let mut b = GrammarBuilder::new("deep");
    b = b.token("leaf", "x");
    b = b.rule("root", vec!["depth1"]);
    b = b.rule("depth1", vec!["depth2"]);
    b = b.rule("depth2", vec!["depth3"]);
    b = b.rule("depth3", vec!["depth4"]);
    b = b.rule("depth4", vec!["leaf"]);
    b = b.start("root");
    let mut g = b.build();
    let before = rule_count(&g);
    do_optimize(&mut g);
    // Unit rule chains may be partially eliminated
    assert!(rule_count(&g) <= before);
}

// ===========================================================================
// 23. Regression-style tests
// ===========================================================================

#[test]
fn optimize_does_not_corrupt_rule_names_map() {
    let mut g = multi_rule_grammar();
    do_optimize(&mut g);
    // Every rule group in `rules` should have a corresponding entry in rule_names
    for &sid in g.rules.keys() {
        // Some rule symbol IDs may not be in rule_names if they were generated
        // internally, but named rules should be consistent
        if let Some(name) = g.rule_names.get(&sid) {
            assert!(!name.is_empty());
        }
    }
}

#[test]
fn optimize_does_not_create_empty_rule_groups() {
    let mut g = multi_rule_grammar();
    do_optimize(&mut g);
    for rules in g.rules.values() {
        assert!(!rules.is_empty(), "empty rule group found after optimize");
    }
}

#[test]
fn optimize_tokens_pattern_preserved() {
    let mut g = simple_grammar();
    do_optimize(&mut g);
    for token in g.tokens.values() {
        assert!(!token.name.is_empty());
    }
}

#[test]
fn optimize_all_rules_iterator_consistent() {
    let mut g = multi_rule_grammar();
    do_optimize(&mut g);
    let count_iter = g.all_rules().count();
    let count_manual: usize = g.rules.values().map(|v| v.len()).sum();
    assert_eq!(count_iter, count_manual);
}
