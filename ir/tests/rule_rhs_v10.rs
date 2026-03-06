//! Comprehensive tests for Rule RHS (right-hand side) structures in adze-ir.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, PrecedenceKind, Symbol, SymbolId};

/// Find a token's SymbolId by name from the Grammar.
fn find_token_id(g: &Grammar, name: &str) -> Option<SymbolId> {
    g.tokens
        .iter()
        .find(|(_, tok)| tok.name == name)
        .map(|(id, _)| *id)
}

// ── 1. Single rule, single RHS element ──────────────────────────────────────

#[test]
fn test_single_rule_single_rhs_token() {
    let g = GrammarBuilder::new("rr_v10_single_tok")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 1);
}

#[test]
fn test_single_rule_single_rhs_is_terminal() {
    let g = GrammarBuilder::new("rr_v10_single_term")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(matches!(rule.rhs[0], Symbol::Terminal(_)));
}

#[test]
fn test_single_rule_single_rhs_nonterminal() {
    let g = GrammarBuilder::new("rr_v10_single_nt")
        .token("X", "x")
        .rule("inner", vec!["X"])
        .rule("s", vec!["inner"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert_eq!(s_rules[0].rhs.len(), 1);
    assert!(matches!(s_rules[0].rhs[0], Symbol::NonTerminal(_)));
}

// ── 2. Single rule, two RHS elements ────────────────────────────────────────

#[test]
fn test_single_rule_two_rhs_elements() {
    let g = GrammarBuilder::new("rr_v10_two_rhs")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 2);
}

#[test]
fn test_two_rhs_both_terminals() {
    let g = GrammarBuilder::new("rr_v10_two_terms")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(matches!(rule.rhs[0], Symbol::Terminal(_)));
    assert!(matches!(rule.rhs[1], Symbol::Terminal(_)));
}

#[test]
fn test_two_rhs_order_preserved() {
    let g = GrammarBuilder::new("rr_v10_two_order")
        .token("FIRST", "1")
        .token("SECOND", "2")
        .rule("s", vec!["FIRST", "SECOND"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    // Both should be terminals and in order (first != second)
    assert!(matches!(rule.rhs[0], Symbol::Terminal(_)));
    assert!(matches!(rule.rhs[1], Symbol::Terminal(_)));
    assert_ne!(rule.rhs[0], rule.rhs[1]);
}

// ── 3. Single rule, three RHS elements ──────────────────────────────────────

#[test]
fn test_single_rule_three_rhs_elements() {
    let g = GrammarBuilder::new("rr_v10_three_rhs")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A", "B", "C"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 3);
}

#[test]
fn test_three_rhs_all_terminals() {
    let g = GrammarBuilder::new("rr_v10_three_all_t")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A", "B", "C"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    for sym in &rule.rhs {
        assert!(matches!(sym, Symbol::Terminal(_)));
    }
}

// ── 4. Rule with zero RHS elements (epsilon/empty) ─────────────────────────

#[test]
fn test_rule_empty_rhs_becomes_epsilon() {
    let g = GrammarBuilder::new("rr_v10_empty_rhs")
        .token("X", "x")
        .rule("s", vec![])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 1);
    assert!(matches!(rule.rhs[0], Symbol::Epsilon));
}

#[test]
fn test_rule_empty_rhs_is_epsilon_symbol() {
    let g = GrammarBuilder::new("rr_v10_empty_len")
        .token("X", "x")
        .rule("s", vec![])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs, vec![Symbol::Epsilon]);
}

#[test]
fn test_rule_empty_rhs_lhs_valid() {
    let g = GrammarBuilder::new("rr_v10_empty_lhs")
        .token("X", "x")
        .rule("s", vec![])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.lhs, s_id);
}

// ── 5. Multiple rules, same LHS ────────────────────────────────────────────

#[test]
fn test_multiple_rules_same_lhs_count() {
    let g = GrammarBuilder::new("rr_v10_same_lhs_ct")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert_eq!(s_rules.len(), 2);
}

#[test]
fn test_multiple_rules_same_lhs_different_rhs() {
    let g = GrammarBuilder::new("rr_v10_same_lhs_dr")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert_ne!(s_rules[0].rhs, s_rules[1].rhs);
}

#[test]
fn test_three_rules_same_lhs() {
    let g = GrammarBuilder::new("rr_v10_three_same")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .rule("s", vec!["C"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert_eq!(s_rules.len(), 3);
}

// ── 6. Multiple rules, different LHS ───────────────────────────────────────

#[test]
fn test_multiple_rules_different_lhs() {
    let g = GrammarBuilder::new("rr_v10_diff_lhs")
        .token("X", "x")
        .token("Y", "y")
        .rule("a", vec!["X"])
        .rule("s", vec!["a", "Y"])
        .start("s")
        .build();
    let a_id = g.find_symbol_by_name("a").unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    assert_ne!(a_id, s_id);
    assert!(g.get_rules_for_symbol(a_id).is_some());
    assert!(g.get_rules_for_symbol(s_id).is_some());
}

#[test]
fn test_rules_for_each_lhs_independent() {
    let g = GrammarBuilder::new("rr_v10_indep_lhs")
        .token("X", "x")
        .token("Y", "y")
        .rule("a", vec!["X"])
        .rule("b", vec!["Y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let a_id = g.find_symbol_by_name("a").unwrap();
    let b_id = g.find_symbol_by_name("b").unwrap();
    let a_rules = g.get_rules_for_symbol(a_id).unwrap();
    let b_rules = g.get_rules_for_symbol(b_id).unwrap();
    assert_eq!(a_rules.len(), 1);
    assert_eq!(b_rules.len(), 1);
}

// ── 7. RHS contains token symbols ──────────────────────────────────────────

#[test]
fn test_rhs_all_tokens() {
    let g = GrammarBuilder::new("rr_v10_all_tokens")
        .token("PLUS", "+")
        .token("NUM", "[0-9]+")
        .rule("s", vec!["NUM", "PLUS", "NUM"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    for sym in &rule.rhs {
        assert!(matches!(sym, Symbol::Terminal(_)));
    }
}

#[test]
fn test_rhs_single_token_id_matches() {
    let g = GrammarBuilder::new("rr_v10_tok_id")
        .token("TOK", "t")
        .rule("s", vec!["TOK"])
        .start("s")
        .build();
    let tok_id = find_token_id(&g, "TOK").unwrap();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs[0], Symbol::Terminal(tok_id));
}

// ── 8. RHS contains non-terminal symbols ───────────────────────────────────

#[test]
fn test_rhs_nonterminal_only() {
    let g = GrammarBuilder::new("rr_v10_nt_only")
        .token("X", "x")
        .rule("leaf", vec!["X"])
        .rule("s", vec!["leaf"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert!(matches!(s_rules[0].rhs[0], Symbol::NonTerminal(_)));
}

#[test]
fn test_rhs_nonterminal_id_matches() {
    let g = GrammarBuilder::new("rr_v10_nt_id")
        .token("X", "x")
        .rule("inner", vec!["X"])
        .rule("s", vec!["inner"])
        .start("s")
        .build();
    let inner_id = g.find_symbol_by_name("inner").unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert_eq!(s_rules[0].rhs[0], Symbol::NonTerminal(inner_id));
}

#[test]
fn test_rhs_two_nonterminals() {
    let g = GrammarBuilder::new("rr_v10_two_nt")
        .token("X", "x")
        .token("Y", "y")
        .rule("a", vec!["X"])
        .rule("b", vec!["Y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    for sym in &s_rules[0].rhs {
        assert!(matches!(sym, Symbol::NonTerminal(_)));
    }
}

// ── 9. RHS contains mix of tokens and non-terminals ────────────────────────

#[test]
fn test_rhs_mixed_token_and_nonterminal() {
    let g = GrammarBuilder::new("rr_v10_mixed")
        .token("PLUS", "+")
        .token("NUM", "[0-9]+")
        .rule("atom", vec!["NUM"])
        .rule("s", vec!["atom", "PLUS", "atom"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert!(matches!(s_rules[0].rhs[0], Symbol::NonTerminal(_)));
    assert!(matches!(s_rules[0].rhs[1], Symbol::Terminal(_)));
    assert!(matches!(s_rules[0].rhs[2], Symbol::NonTerminal(_)));
}

#[test]
fn test_rhs_mixed_order_nt_then_t() {
    let g = GrammarBuilder::new("rr_v10_nt_t")
        .token("SEP", ";")
        .token("X", "x")
        .rule("item", vec!["X"])
        .rule("s", vec!["item", "SEP"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert!(matches!(s_rules[0].rhs[0], Symbol::NonTerminal(_)));
    assert!(matches!(s_rules[0].rhs[1], Symbol::Terminal(_)));
}

#[test]
fn test_rhs_mixed_order_t_then_nt() {
    let g = GrammarBuilder::new("rr_v10_t_nt")
        .token("OPEN", "(")
        .token("X", "x")
        .rule("body", vec!["X"])
        .rule("s", vec!["OPEN", "body"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert!(matches!(s_rules[0].rhs[0], Symbol::Terminal(_)));
    assert!(matches!(s_rules[0].rhs[1], Symbol::NonTerminal(_)));
}

// ── 10. Rule count matches builder call count ──────────────────────────────

#[test]
fn test_rule_count_one() {
    let g = GrammarBuilder::new("rr_v10_count1")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 1);
}

#[test]
fn test_rule_count_two_same_lhs() {
    let g = GrammarBuilder::new("rr_v10_count2_same")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 2);
}

#[test]
fn test_rule_count_two_diff_lhs() {
    let g = GrammarBuilder::new("rr_v10_count2_diff")
        .token("X", "x")
        .token("Y", "y")
        .rule("a", vec!["X"])
        .rule("s", vec!["a", "Y"])
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 2);
}

#[test]
fn test_rule_count_five() {
    let g = GrammarBuilder::new("rr_v10_count5")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .rule("s", vec!["C"])
        .rule("s", vec!["A", "B"])
        .rule("s", vec!["B", "C"])
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 5);
}

// ── 11. RHS order preserved from builder ───────────────────────────────────

#[test]
fn test_rhs_order_three_tokens() {
    let g = GrammarBuilder::new("rr_v10_order3")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A", "B", "C"])
        .start("s")
        .build();
    let a_id = find_token_id(&g, "A").unwrap();
    let b_id = find_token_id(&g, "B").unwrap();
    let c_id = find_token_id(&g, "C").unwrap();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs[0], Symbol::Terminal(a_id));
    assert_eq!(rule.rhs[1], Symbol::Terminal(b_id));
    assert_eq!(rule.rhs[2], Symbol::Terminal(c_id));
}

#[test]
fn test_rhs_order_reversed() {
    let g = GrammarBuilder::new("rr_v10_order_rev")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["C", "B", "A"])
        .start("s")
        .build();
    let a_id = find_token_id(&g, "A").unwrap();
    let b_id = find_token_id(&g, "B").unwrap();
    let c_id = find_token_id(&g, "C").unwrap();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs[0], Symbol::Terminal(c_id));
    assert_eq!(rule.rhs[1], Symbol::Terminal(b_id));
    assert_eq!(rule.rhs[2], Symbol::Terminal(a_id));
}

#[test]
fn test_rhs_order_with_nonterminals() {
    let g = GrammarBuilder::new("rr_v10_order_nt")
        .token("X", "x")
        .token("Y", "y")
        .rule("a", vec!["X"])
        .rule("b", vec!["Y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let a_id = g.find_symbol_by_name("a").unwrap();
    let b_id = g.find_symbol_by_name("b").unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert_eq!(s_rules[0].rhs[0], Symbol::NonTerminal(a_id));
    assert_eq!(s_rules[0].rhs[1], Symbol::NonTerminal(b_id));
}

// ── 12. Precedence on rule ─────────────────────────────────────────────────

#[test]
fn test_rule_with_precedence_static() {
    let g = GrammarBuilder::new("rr_v10_prec_stat")
        .token("PLUS", "+")
        .token("NUM", "[0-9]+")
        .rule("atom", vec!["NUM"])
        .rule_with_precedence("s", vec!["s", "PLUS", "s"], 1, Associativity::Left)
        .rule("s", vec!["atom"])
        .start("s")
        .build();
    let has_prec = g
        .all_rules()
        .any(|r| r.precedence == Some(PrecedenceKind::Static(1)));
    assert!(has_prec);
}

#[test]
fn test_rule_without_precedence_is_none() {
    let g = GrammarBuilder::new("rr_v10_no_prec")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(rule.precedence.is_none());
}

#[test]
fn test_rule_precedence_value_preserved() {
    let g = GrammarBuilder::new("rr_v10_prec_val")
        .token("STAR", "*")
        .token("PLUS", "+")
        .token("NUM", "[0-9]+")
        .rule("atom", vec!["NUM"])
        .rule_with_precedence("s", vec!["s", "STAR", "s"], 2, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "PLUS", "s"], 1, Associativity::Left)
        .rule("s", vec!["atom"])
        .start("s")
        .build();
    let prec_values: Vec<_> = g.all_rules().filter_map(|r| r.precedence).collect();
    assert!(prec_values.contains(&PrecedenceKind::Static(2)));
    assert!(prec_values.contains(&PrecedenceKind::Static(1)));
}

#[test]
fn test_rule_negative_precedence() {
    let g = GrammarBuilder::new("rr_v10_neg_prec")
        .token("X", "x")
        .token("Y", "y")
        .rule("atom", vec!["X"])
        .rule_with_precedence("s", vec!["s", "Y", "s"], -5, Associativity::None)
        .rule("s", vec!["atom"])
        .start("s")
        .build();
    let has_neg = g
        .all_rules()
        .any(|r| r.precedence == Some(PrecedenceKind::Static(-5)));
    assert!(has_neg);
}

// ── 13. Associativity on rule ──────────────────────────────────────────────

#[test]
fn test_rule_assoc_left() {
    let g = GrammarBuilder::new("rr_v10_assoc_l")
        .token("PLUS", "+")
        .token("NUM", "[0-9]+")
        .rule("atom", vec!["NUM"])
        .rule_with_precedence("s", vec!["s", "PLUS", "s"], 1, Associativity::Left)
        .rule("s", vec!["atom"])
        .start("s")
        .build();
    let has_left = g
        .all_rules()
        .any(|r| r.associativity == Some(Associativity::Left));
    assert!(has_left);
}

#[test]
fn test_rule_assoc_right() {
    let g = GrammarBuilder::new("rr_v10_assoc_r")
        .token("EQ", "=")
        .token("ID", "[a-z]+")
        .rule("atom", vec!["ID"])
        .rule_with_precedence("s", vec!["s", "EQ", "s"], 1, Associativity::Right)
        .rule("s", vec!["atom"])
        .start("s")
        .build();
    let has_right = g
        .all_rules()
        .any(|r| r.associativity == Some(Associativity::Right));
    assert!(has_right);
}

#[test]
fn test_rule_assoc_none() {
    let g = GrammarBuilder::new("rr_v10_assoc_n")
        .token("CMP", "==")
        .token("NUM", "[0-9]+")
        .rule("atom", vec!["NUM"])
        .rule_with_precedence("s", vec!["s", "CMP", "s"], 1, Associativity::None)
        .rule("s", vec!["atom"])
        .start("s")
        .build();
    let has_none_assoc = g
        .all_rules()
        .any(|r| r.associativity == Some(Associativity::None));
    assert!(has_none_assoc);
}

#[test]
fn test_rule_without_assoc_is_none() {
    let g = GrammarBuilder::new("rr_v10_no_assoc")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(rule.associativity.is_none());
}

#[test]
fn test_mixed_assoc_rules() {
    let g = GrammarBuilder::new("rr_v10_mix_assoc")
        .token("PLUS", "+")
        .token("EQ", "=")
        .token("NUM", "[0-9]+")
        .rule("atom", vec!["NUM"])
        .rule_with_precedence("s", vec!["s", "PLUS", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "EQ", "s"], 2, Associativity::Right)
        .rule("s", vec!["atom"])
        .start("s")
        .build();
    let assocs: Vec<_> = g.all_rules().filter_map(|r| r.associativity).collect();
    assert!(assocs.contains(&Associativity::Left));
    assert!(assocs.contains(&Associativity::Right));
}

// ── 14. Rule iteration via all_rules ───────────────────────────────────────

#[test]
fn test_all_rules_returns_all() {
    let g = GrammarBuilder::new("rr_v10_iter_all")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"])
        .rule("y", vec!["B"])
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 3);
}

#[test]
fn test_all_rules_includes_all_lhs() {
    let g = GrammarBuilder::new("rr_v10_iter_lhs")
        .token("A", "a")
        .token("B", "b")
        .rule("x", vec!["A"])
        .rule("y", vec!["B"])
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let x_id = g.find_symbol_by_name("x").unwrap();
    let y_id = g.find_symbol_by_name("y").unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let lhs_set: Vec<_> = g.all_rules().map(|r| r.lhs).collect();
    assert!(lhs_set.contains(&x_id));
    assert!(lhs_set.contains(&y_id));
    assert!(lhs_set.contains(&s_id));
}

#[test]
fn test_all_rules_collect_to_vec() {
    let g = GrammarBuilder::new("rr_v10_iter_vec")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rules: Vec<_> = g.all_rules().collect();
    assert_eq!(rules.len(), 1);
}

#[test]
fn test_all_rules_multiple_same_lhs() {
    let g = GrammarBuilder::new("rr_v10_iter_same")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .rule("s", vec!["C"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_count = g.all_rules().filter(|r| r.lhs == s_id).count();
    assert_eq!(s_count, 3);
}

#[test]
fn test_all_rules_filter_by_rhs_len() {
    let g = GrammarBuilder::new("rr_v10_iter_filt")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let single_rhs = g.all_rules().filter(|r| r.rhs.len() == 1).count();
    let double_rhs = g.all_rules().filter(|r| r.rhs.len() == 2).count();
    assert_eq!(single_rhs, 1);
    assert_eq!(double_rhs, 1);
}

// ── 15. Rule clone preserves RHS ───────────────────────────────────────────

#[test]
fn test_rule_clone_preserves_rhs() {
    let g = GrammarBuilder::new("rr_v10_clone_rhs")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let cloned = rule.clone();
    assert_eq!(rule.rhs, cloned.rhs);
}

#[test]
fn test_rule_clone_preserves_lhs() {
    let g = GrammarBuilder::new("rr_v10_clone_lhs")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let cloned = rule.clone();
    assert_eq!(rule.lhs, cloned.lhs);
}

#[test]
fn test_rule_clone_preserves_precedence() {
    let g = GrammarBuilder::new("rr_v10_clone_prec")
        .token("PLUS", "+")
        .token("NUM", "[0-9]+")
        .rule("atom", vec!["NUM"])
        .rule_with_precedence("s", vec!["s", "PLUS", "s"], 3, Associativity::Left)
        .rule("s", vec!["atom"])
        .start("s")
        .build();
    let prec_rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
    let cloned = prec_rule.clone();
    assert_eq!(prec_rule.precedence, cloned.precedence);
    assert_eq!(prec_rule.associativity, cloned.associativity);
}

#[test]
fn test_rule_clone_eq() {
    let g = GrammarBuilder::new("rr_v10_clone_eq")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let cloned = rule.clone();
    assert_eq!(rule, &cloned);
}

// ── 16. Rule Debug format ──────────────────────────────────────────────────

#[test]
fn test_rule_debug_not_empty() {
    let g = GrammarBuilder::new("rr_v10_debug_ne")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let dbg = format!("{:?}", rule);
    assert!(!dbg.is_empty());
}

#[test]
fn test_rule_debug_contains_rule() {
    let g = GrammarBuilder::new("rr_v10_debug_rule")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let dbg = format!("{:?}", rule);
    assert!(dbg.contains("Rule"));
}

#[test]
fn test_rule_debug_contains_lhs() {
    let g = GrammarBuilder::new("rr_v10_debug_lhs")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let dbg = format!("{:?}", rule);
    assert!(dbg.contains("lhs"));
}

#[test]
fn test_rule_debug_contains_rhs() {
    let g = GrammarBuilder::new("rr_v10_debug_rhs")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let dbg = format!("{:?}", rule);
    assert!(dbg.contains("rhs"));
}

#[test]
fn test_rule_debug_contains_precedence() {
    let g = GrammarBuilder::new("rr_v10_dbg_prec")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let dbg = format!("{:?}", rule);
    assert!(dbg.contains("precedence"));
}

// ── 17. Normalize affects rules ────────────────────────────────────────────

#[test]
fn test_normalize_returns_rules() {
    let mut g = GrammarBuilder::new("rr_v10_norm_ret")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let normalized = g.normalize();
    assert!(!normalized.is_empty());
}

#[test]
fn test_normalize_preserves_simple_rule() {
    let mut g = GrammarBuilder::new("rr_v10_norm_simple")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let before_count = g.all_rules().count();
    let _normalized = g.normalize();
    let after_count = g.all_rules().count();
    // Simple rules should still exist after normalization
    assert!(after_count >= before_count);
}

#[test]
fn test_normalize_rule_lhs_still_valid() {
    let mut g = GrammarBuilder::new("rr_v10_norm_lhs")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let _normalized = g.normalize();
    let s_rules = g.get_rules_for_symbol(s_id);
    assert!(s_rules.is_some());
}

#[test]
fn test_normalize_multiple_rules() {
    let mut g = GrammarBuilder::new("rr_v10_norm_multi")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    let normalized = g.normalize();
    // Should produce at least the original 2 rules
    assert!(normalized.len() >= 2);
}

// ── 18. Optimize may change rules ──────────────────────────────────────────

#[test]
fn test_optimize_does_not_panic() {
    let mut g = GrammarBuilder::new("rr_v10_opt_np")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    g.optimize();
    // Should not panic; grammar still usable
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_optimize_preserves_start_rule() {
    let mut g = GrammarBuilder::new("rr_v10_opt_start")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    g.optimize();
    assert!(g.start_symbol().is_some());
}

#[test]
fn test_optimize_grammar_still_has_rules() {
    let mut g = GrammarBuilder::new("rr_v10_opt_rules")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    g.optimize();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn test_optimize_then_iterate() {
    let mut g = GrammarBuilder::new("rr_v10_opt_iter")
        .token("X", "x")
        .token("Y", "y")
        .rule("a", vec!["X"])
        .rule("s", vec!["a", "Y"])
        .start("s")
        .build();
    g.optimize();
    for rule in g.all_rules() {
        let _ = rule.lhs;
        let _ = rule.rhs.len();
    }
}

// ── 19. Large RHS (10 elements) ────────────────────────────────────────────

#[test]
fn test_large_rhs_ten_elements() {
    let g = GrammarBuilder::new("rr_v10_large10")
        .token("T0", "0")
        .token("T1", "1")
        .token("T2", "2")
        .token("T3", "3")
        .token("T4", "4")
        .token("T5", "5")
        .token("T6", "6")
        .token("T7", "7")
        .token("T8", "8")
        .token("T9", "9")
        .rule(
            "s",
            vec!["T0", "T1", "T2", "T3", "T4", "T5", "T6", "T7", "T8", "T9"],
        )
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 10);
}

#[test]
fn test_large_rhs_all_terminals() {
    let g = GrammarBuilder::new("rr_v10_large_term")
        .token("T0", "0")
        .token("T1", "1")
        .token("T2", "2")
        .token("T3", "3")
        .token("T4", "4")
        .token("T5", "5")
        .token("T6", "6")
        .token("T7", "7")
        .token("T8", "8")
        .token("T9", "9")
        .rule(
            "s",
            vec!["T0", "T1", "T2", "T3", "T4", "T5", "T6", "T7", "T8", "T9"],
        )
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    for sym in &rule.rhs {
        assert!(matches!(sym, Symbol::Terminal(_)));
    }
}

#[test]
fn test_large_rhs_order_preserved() {
    let g = GrammarBuilder::new("rr_v10_large_ord")
        .token("T0", "0")
        .token("T1", "1")
        .token("T2", "2")
        .token("T3", "3")
        .token("T4", "4")
        .token("T5", "5")
        .token("T6", "6")
        .token("T7", "7")
        .token("T8", "8")
        .token("T9", "9")
        .rule(
            "s",
            vec!["T0", "T1", "T2", "T3", "T4", "T5", "T6", "T7", "T8", "T9"],
        )
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let names = ["T0", "T1", "T2", "T3", "T4", "T5", "T6", "T7", "T8", "T9"];
    for (i, name) in names.iter().enumerate() {
        let expected_id = find_token_id(&g, name).unwrap();
        assert_eq!(rule.rhs[i], Symbol::Terminal(expected_id));
    }
}

// ── 20. Many rules (20+) ──────────────────────────────────────────────────

#[test]
fn test_twenty_rules_count() {
    let mut builder = GrammarBuilder::new("rr_v10_twenty")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d");
    for i in 0..20 {
        let lhs = format!("r{}", i);
        builder = builder.rule(&lhs, vec!["A"]);
    }
    builder = builder.rule("s", vec!["r0"]).start("s");
    let g = builder.build();
    assert!(g.all_rules().count() >= 20);
}

#[test]
fn test_many_rules_each_has_rhs() {
    let mut builder = GrammarBuilder::new("rr_v10_many_rhs").token("X", "x");
    for i in 0..15 {
        let lhs = format!("n{}", i);
        builder = builder.rule(&lhs, vec!["X"]);
    }
    builder = builder.rule("s", vec!["n0"]).start("s");
    let g = builder.build();
    for rule in g.all_rules() {
        assert!(!rule.rhs.is_empty());
    }
}

#[test]
fn test_many_rules_different_rhs_lengths() {
    let g = GrammarBuilder::new("rr_v10_many_len")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .rule("s", vec!["A"])
        .rule("s", vec!["A", "B"])
        .rule("s", vec!["A", "B", "C"])
        .rule("s", vec!["A", "B", "C", "D"])
        .start("s")
        .build();
    let lengths: Vec<_> = g.all_rules().map(|r| r.rhs.len()).collect();
    assert!(lengths.contains(&1));
    assert!(lengths.contains(&2));
    assert!(lengths.contains(&3));
    assert!(lengths.contains(&4));
}

// ── Additional coverage: RHS with repeated tokens ──────────────────────────

#[test]
fn test_rhs_repeated_token() {
    let g = GrammarBuilder::new("rr_v10_repeat_tok")
        .token("X", "x")
        .rule("s", vec!["X", "X", "X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 3);
    let first = &rule.rhs[0];
    assert_eq!(first, &rule.rhs[1]);
    assert_eq!(first, &rule.rhs[2]);
}

#[test]
fn test_rhs_repeated_nonterminal() {
    let g = GrammarBuilder::new("rr_v10_repeat_nt")
        .token("X", "x")
        .rule("item", vec!["X"])
        .rule("s", vec!["item", "item"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert_eq!(s_rules[0].rhs[0], s_rules[0].rhs[1]);
}

// ── Additional coverage: production_id field ───────────────────────────────

#[test]
fn test_rule_has_production_id() {
    let g = GrammarBuilder::new("rr_v10_prod_id")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let _pid = rule.production_id;
}

#[test]
fn test_rule_fields_default_empty() {
    let g = GrammarBuilder::new("rr_v10_fields_e")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(rule.fields.is_empty());
}

// ── Additional coverage: get_rules_for_symbol returns None for tokens ──────

#[test]
fn test_no_rules_for_token_symbol() {
    let g = GrammarBuilder::new("rr_v10_no_tok_rul")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let x_id = find_token_id(&g, "X").unwrap();
    assert!(g.get_rules_for_symbol(x_id).is_none());
}

// ── Additional coverage: lhs matches find_symbol_by_name ───────────────────

#[test]
fn test_lhs_matches_symbol_lookup() {
    let g = GrammarBuilder::new("rr_v10_lhs_match")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.lhs, s_id);
}

// ── Additional coverage: recursive rule (self-reference in RHS) ────────────

#[test]
fn test_recursive_rule_rhs_contains_lhs() {
    let g = GrammarBuilder::new("rr_v10_recursive")
        .token("X", "x")
        .token("COMMA", ",")
        .rule("s", vec!["X"])
        .rule("s", vec!["s", "COMMA", "X"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    let recursive = s_rules.iter().any(|r| {
        r.rhs
            .iter()
            .any(|sym| matches!(sym, Symbol::NonTerminal(id) if *id == s_id))
    });
    assert!(recursive);
}

#[test]
fn test_recursive_rule_base_case_exists() {
    let g = GrammarBuilder::new("rr_v10_recur_base")
        .token("X", "x")
        .token("COMMA", ",")
        .rule("s", vec!["X"])
        .rule("s", vec!["s", "COMMA", "X"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    let base_case = s_rules
        .iter()
        .any(|r| r.rhs.iter().all(|sym| matches!(sym, Symbol::Terminal(_))));
    assert!(base_case);
}

// ── Additional coverage: mutual recursion ──────────────────────────────────

#[test]
fn test_mutual_recursion_rhs() {
    let g = GrammarBuilder::new("rr_v10_mutual")
        .token("X", "x")
        .token("Y", "y")
        .rule("a", vec!["X"])
        .rule("a", vec!["b", "X"])
        .rule("b", vec!["Y"])
        .rule("b", vec!["a", "Y"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let a_id = g.find_symbol_by_name("a").unwrap();
    let b_id = g.find_symbol_by_name("b").unwrap();
    let a_rules = g.get_rules_for_symbol(a_id).unwrap();
    let b_rules = g.get_rules_for_symbol(b_id).unwrap();
    let a_refs_b = a_rules.iter().any(|r| {
        r.rhs
            .iter()
            .any(|sym| matches!(sym, Symbol::NonTerminal(id) if *id == b_id))
    });
    let b_refs_a = b_rules.iter().any(|r| {
        r.rhs
            .iter()
            .any(|sym| matches!(sym, Symbol::NonTerminal(id) if *id == a_id))
    });
    assert!(a_refs_b);
    assert!(b_refs_a);
}

// ── Additional coverage: grammar name preserved ────────────────────────────

#[test]
fn test_grammar_name_preserved() {
    let g = GrammarBuilder::new("rr_v10_name_test")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    assert_eq!(g.name, "rr_v10_name_test");
}

// ── Additional coverage: start_symbol correct ──────────────────────────────

#[test]
fn test_start_symbol_matches() {
    let g = GrammarBuilder::new("rr_v10_start_sym")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let s_id = g.find_symbol_by_name("s").unwrap();
    assert_eq!(g.start_symbol(), Some(s_id));
}

// ── Additional coverage: rules indexmap has expected keys ───────────────────

#[test]
fn test_rules_map_keys_match_lhs() {
    let g = GrammarBuilder::new("rr_v10_map_keys")
        .token("X", "x")
        .token("Y", "y")
        .rule("a", vec!["X"])
        .rule("b", vec!["Y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let a_id = g.find_symbol_by_name("a").unwrap();
    let b_id = g.find_symbol_by_name("b").unwrap();
    let s_id = g.find_symbol_by_name("s").unwrap();
    assert!(g.rules.contains_key(&a_id));
    assert!(g.rules.contains_key(&b_id));
    assert!(g.rules.contains_key(&s_id));
}

#[test]
fn test_rules_map_len() {
    let g = GrammarBuilder::new("rr_v10_map_len")
        .token("X", "x")
        .token("Y", "y")
        .rule("a", vec!["X"])
        .rule("b", vec!["Y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    // 3 distinct LHS non-terminals
    assert_eq!(g.rules.len(), 3);
}

// ── Additional coverage: precedence + rhs combined ─────────────────────────

#[test]
fn test_prec_rule_rhs_len_preserved() {
    let g = GrammarBuilder::new("rr_v10_prec_rhs_l")
        .token("PLUS", "+")
        .token("NUM", "[0-9]+")
        .rule("atom", vec!["NUM"])
        .rule_with_precedence("s", vec!["s", "PLUS", "s"], 1, Associativity::Left)
        .rule("s", vec!["atom"])
        .start("s")
        .build();
    let prec_rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.rhs.len(), 3);
}

#[test]
fn test_prec_rule_rhs_contains_terminal() {
    let g = GrammarBuilder::new("rr_v10_prec_rhs_t")
        .token("STAR", "*")
        .token("NUM", "[0-9]+")
        .rule("atom", vec!["NUM"])
        .rule_with_precedence("s", vec!["s", "STAR", "s"], 2, Associativity::Left)
        .rule("s", vec!["atom"])
        .start("s")
        .build();
    let prec_rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
    assert!(matches!(prec_rule.rhs[1], Symbol::Terminal(_)));
}

// ── Additional coverage: clone of grammar preserves rules ──────────────────

#[test]
fn test_grammar_clone_preserves_rule_count() {
    let g = GrammarBuilder::new("rr_v10_gclone")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn test_grammar_clone_preserves_rhs() {
    let g = GrammarBuilder::new("rr_v10_gclone_rhs")
        .token("X", "x")
        .token("Y", "y")
        .rule("s", vec!["X", "Y"])
        .start("s")
        .build();
    let g2 = g.clone();
    let r1 = g.all_rules().next().unwrap();
    let r2 = g2.all_rules().next().unwrap();
    assert_eq!(r1.rhs, r2.rhs);
}

// ── Additional coverage: Symbol equality in RHS ────────────────────────────

#[test]
fn test_symbol_terminal_eq() {
    let g = GrammarBuilder::new("rr_v10_sym_eq")
        .token("X", "x")
        .rule("s", vec!["X", "X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs[0], rule.rhs[1]);
}

#[test]
fn test_symbol_terminal_ne_different_tokens() {
    let g = GrammarBuilder::new("rr_v10_sym_ne")
        .token("X", "x")
        .token("Y", "y")
        .rule("s", vec!["X", "Y"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_ne!(rule.rhs[0], rule.rhs[1]);
}

// ── Additional coverage: normalize then iterate ────────────────────────────

#[test]
fn test_normalize_then_all_rules_works() {
    let mut g = GrammarBuilder::new("rr_v10_norm_iter")
        .token("X", "x")
        .token("Y", "y")
        .rule("a", vec!["X"])
        .rule("s", vec!["a", "Y"])
        .start("s")
        .build();
    let _normalized = g.normalize();
    let count = g.all_rules().count();
    assert!(count >= 2);
}

// ── Additional coverage: optimize then normalize ───────────────────────────

#[test]
fn test_optimize_then_normalize() {
    let mut g = GrammarBuilder::new("rr_v10_opt_norm")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    g.optimize();
    let normalized = g.normalize();
    assert!(!normalized.is_empty());
}

// ── Additional coverage: rule with five different tokens ───────────────────

#[test]
fn test_five_token_rhs() {
    let g = GrammarBuilder::new("rr_v10_five_tok")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule("s", vec!["A", "B", "C", "D", "E"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 5);
    for sym in &rule.rhs {
        assert!(matches!(sym, Symbol::Terminal(_)));
    }
}

// ── Additional coverage: chain of nonterminals ─────────────────────────────

#[test]
fn test_chain_of_nonterminals() {
    let g = GrammarBuilder::new("rr_v10_chain_nt")
        .token("X", "x")
        .rule("d", vec!["X"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 4);
    let s_id = g.find_symbol_by_name("s").unwrap();
    let b_id = g.find_symbol_by_name("b").unwrap();
    let s_rules = g.get_rules_for_symbol(s_id).unwrap();
    assert_eq!(s_rules[0].rhs[0], Symbol::NonTerminal(b_id));
}

// ── Additional coverage: SymbolId is Copy ──────────────────────────────────

#[test]
fn test_symbol_id_is_copy() {
    let g = GrammarBuilder::new("rr_v10_sid_copy")
        .token("X", "x")
        .rule("s", vec!["X"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    let id = rule.lhs;
    let id2 = id; // Copy, not move
    assert_eq!(id, id2);
}

// ── Additional coverage: multiple precedence levels ────────────────────────

#[test]
fn test_three_precedence_levels() {
    let g = GrammarBuilder::new("rr_v10_three_prec")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("HAT", "^")
        .token("NUM", "[0-9]+")
        .rule("atom", vec!["NUM"])
        .rule_with_precedence("s", vec!["s", "PLUS", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "STAR", "s"], 2, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "HAT", "s"], 3, Associativity::Right)
        .rule("s", vec!["atom"])
        .start("s")
        .build();
    let precs: Vec<_> = g.all_rules().filter_map(|r| r.precedence).collect();
    assert_eq!(precs.len(), 3);
    assert!(precs.contains(&PrecedenceKind::Static(1)));
    assert!(precs.contains(&PrecedenceKind::Static(2)));
    assert!(precs.contains(&PrecedenceKind::Static(3)));
}

// ── Additional coverage: Associativity is Copy ─────────────────────────────

#[test]
fn test_associativity_copy() {
    let a = Associativity::Left;
    let b = a; // Copy
    assert_eq!(a, b);
}

// ── Additional coverage: PrecedenceKind is Copy ────────────────────────────

#[test]
fn test_precedence_kind_copy() {
    let p = PrecedenceKind::Static(5);
    let q = p; // Copy
    assert_eq!(p, q);
}
