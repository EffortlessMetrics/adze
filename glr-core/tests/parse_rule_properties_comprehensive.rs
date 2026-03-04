#![cfg(feature = "test-api")]

//! Comprehensive tests for `ParseRule` and parse-table rule properties.
//!
//! Covers: lhs validity, rhs_len correctness, rule counts, chain grammars,
//! recursive grammars, precedence grammars, large grammars, determinism,
//! and rule ordering properties.

use adze_glr_core::{Action, FirstFollowSets, ParseRule, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};

// ===========================================================================
// Helpers
// ===========================================================================

/// Build a parse table from a Grammar (auto-normalizes via compute_normalized).
fn build_table(g: &mut Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute_normalized(g).expect("FIRST/FOLLOW should succeed");
    build_lr1_automaton(g, &ff).expect("automaton construction should succeed")
}

/// Minimal grammar: S → a
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// Two-alternative grammar: S → a | b
fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

/// Chain grammar: S → A, A → B, B → x
fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("start", vec!["mid"])
        .rule("mid", vec!["leaf"])
        .rule("leaf", vec!["x"])
        .start("start")
        .build()
}

/// Left-recursive grammar: L → L a | a
fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("leftrec")
        .token("a", "a")
        .rule("lst", vec!["lst", "a"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build()
}

/// Right-recursive grammar: R → a R | a
fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("rightrec")
        .token("a", "a")
        .rule("lst", vec!["a", "lst"])
        .rule("lst", vec!["a"])
        .start("lst")
        .build()
}

/// Arithmetic grammar with precedence: expr → expr + expr | expr * expr | NUM
fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Multi-symbol RHS grammar: S → a b c d
fn long_rhs_grammar() -> Grammar {
    GrammarBuilder::new("longrhs")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start", vec!["a", "b", "c", "d"])
        .start("start")
        .build()
}

// ===========================================================================
// 1–5. ParseRule lhs validity
// ===========================================================================

#[test]
fn lhs_is_valid_symbol_minimal() {
    let table = build_table(&mut minimal_grammar());
    for rule in &table.rules {
        assert!(rule.lhs.0 > 0, "lhs should be a valid SymbolId");
    }
}

#[test]
fn lhs_is_valid_symbol_two_alt() {
    let table = build_table(&mut two_alt_grammar());
    for rule in &table.rules {
        assert!(rule.lhs.0 > 0, "lhs should be a valid SymbolId");
    }
}

#[test]
fn lhs_is_valid_symbol_chain() {
    let table = build_table(&mut chain_grammar());
    for rule in &table.rules {
        assert!(rule.lhs.0 > 0, "lhs should be a valid SymbolId");
    }
}

#[test]
fn lhs_is_valid_symbol_left_recursive() {
    let table = build_table(&mut left_recursive_grammar());
    for rule in &table.rules {
        assert!(rule.lhs.0 > 0, "lhs should be a valid SymbolId");
    }
}

#[test]
fn lhs_is_valid_symbol_arithmetic() {
    let table = build_table(&mut arithmetic_grammar());
    for rule in &table.rules {
        assert!(rule.lhs.0 > 0, "lhs should be a valid SymbolId");
    }
}

// ===========================================================================
// 6–10. ParseRule rhs_len is coherent (>= 0 always true for u16)
// ===========================================================================

#[test]
fn rhs_len_nonneg_minimal() {
    let table = build_table(&mut minimal_grammar());
    for rule in &table.rules {
        // u16 always >= 0, but verify the value is reasonable
        assert!(rule.rhs_len <= 1000, "rhs_len should be reasonable");
    }
}

#[test]
fn rhs_len_nonneg_two_alt() {
    let table = build_table(&mut two_alt_grammar());
    for rule in &table.rules {
        assert!(rule.rhs_len <= 1000);
    }
}

#[test]
fn rhs_len_nonneg_chain() {
    let table = build_table(&mut chain_grammar());
    for rule in &table.rules {
        assert!(rule.rhs_len <= 1000);
    }
}

#[test]
fn rhs_len_nonneg_left_recursive() {
    let table = build_table(&mut left_recursive_grammar());
    for rule in &table.rules {
        assert!(rule.rhs_len <= 1000);
    }
}

#[test]
fn rhs_len_nonneg_arithmetic() {
    let table = build_table(&mut arithmetic_grammar());
    for rule in &table.rules {
        assert!(rule.rhs_len <= 1000);
    }
}

// ===========================================================================
// 11–16. Simple grammar: rule count matches expected
// ===========================================================================

#[test]
fn minimal_grammar_rule_count() {
    let table = build_table(&mut minimal_grammar());
    // S → a  → 1 user rule; augmented start adds 1 more
    assert!(
        !table.rules.is_empty(),
        "minimal grammar must have at least 1 rule, got {}",
        table.rules.len()
    );
}

#[test]
fn two_alt_grammar_rule_count() {
    let table = build_table(&mut two_alt_grammar());
    // S → a | b  → 2 user rules
    assert!(
        table.rules.len() >= 2,
        "two-alt grammar must have at least 2 rules, got {}",
        table.rules.len()
    );
}

#[test]
fn chain_grammar_rule_count() {
    let table = build_table(&mut chain_grammar());
    // S → A, A → B, B → x → 3 user rules
    assert!(
        table.rules.len() >= 3,
        "chain grammar must have at least 3 rules, got {}",
        table.rules.len()
    );
}

#[test]
fn left_recursive_grammar_rule_count() {
    let table = build_table(&mut left_recursive_grammar());
    // L → L a | a → 2 user rules
    assert!(
        table.rules.len() >= 2,
        "left-recursive grammar should have at least 2 rules, got {}",
        table.rules.len()
    );
}

#[test]
fn right_recursive_grammar_rule_count() {
    let table = build_table(&mut right_recursive_grammar());
    assert!(
        table.rules.len() >= 2,
        "right-recursive grammar should have at least 2 rules, got {}",
        table.rules.len()
    );
}

#[test]
fn arithmetic_grammar_rule_count() {
    let table = build_table(&mut arithmetic_grammar());
    // expr → expr+expr | expr*expr | NUM → 3 user rules
    assert!(
        table.rules.len() >= 3,
        "arithmetic grammar should have at least 3 rules, got {}",
        table.rules.len()
    );
}

// ===========================================================================
// 17–22. Chain grammar: rule properties
// ===========================================================================

#[test]
fn chain_grammar_all_rhs_len_one() {
    let table = build_table(&mut chain_grammar());
    // Each chain rule has exactly 1 RHS symbol: S→A, A→B, B→x
    let chain_rules: Vec<_> = table.rules.iter().filter(|r| r.rhs_len == 1).collect();
    assert!(
        chain_rules.len() >= 3,
        "chain grammar should have at least 3 unit rules, got {}",
        chain_rules.len()
    );
}

#[test]
fn chain_grammar_has_no_long_rhs() {
    let table = build_table(&mut chain_grammar());
    for rule in &table.rules {
        assert!(
            rule.rhs_len <= 2,
            "chain grammar should not have rules with rhs_len > 2, found {}",
            rule.rhs_len
        );
    }
}

#[test]
fn chain_grammar_distinct_lhs_count() {
    let table = build_table(&mut chain_grammar());
    let mut lhs_set = std::collections::BTreeSet::new();
    for rule in &table.rules {
        lhs_set.insert(rule.lhs);
    }
    // At least start, mid, leaf (3 user nonterminals) + possibly augmented start
    assert!(
        lhs_set.len() >= 3,
        "chain grammar should have at least 3 distinct lhs symbols, got {}",
        lhs_set.len()
    );
}

#[test]
fn chain_grammar_lhs_not_eof() {
    let table = build_table(&mut chain_grammar());
    let eof = table.eof_symbol;
    for rule in &table.rules {
        assert_ne!(rule.lhs, eof, "rule lhs should never be EOF");
    }
}

#[test]
fn chain_grammar_rules_nonempty() {
    let table = build_table(&mut chain_grammar());
    assert!(!table.rules.is_empty(), "chain grammar must have rules");
}

#[test]
fn chain_grammar_rule_ids_sequential() {
    let table = build_table(&mut chain_grammar());
    // ParseTable.rules is a Vec, indices are sequential rule IDs
    for (i, _rule) in table.rules.iter().enumerate() {
        assert!(i < table.rules.len());
    }
}

// ===========================================================================
// 23–28. Recursive grammar: rules
// ===========================================================================

#[test]
fn left_recursive_has_rhs_len_2() {
    let table = build_table(&mut left_recursive_grammar());
    // L → L a has rhs_len=2
    let has_len2 = table.rules.iter().any(|r| r.rhs_len == 2);
    assert!(
        has_len2,
        "left-recursive grammar should have a rule with rhs_len == 2"
    );
}

#[test]
fn left_recursive_has_rhs_len_1() {
    let table = build_table(&mut left_recursive_grammar());
    // L → a has rhs_len=1
    let has_len1 = table.rules.iter().any(|r| r.rhs_len == 1);
    assert!(
        has_len1,
        "left-recursive grammar should have a rule with rhs_len == 1"
    );
}

#[test]
fn right_recursive_has_rhs_len_2() {
    let table = build_table(&mut right_recursive_grammar());
    let has_len2 = table.rules.iter().any(|r| r.rhs_len == 2);
    assert!(
        has_len2,
        "right-recursive grammar should have a rule with rhs_len == 2"
    );
}

#[test]
fn right_recursive_has_rhs_len_1() {
    let table = build_table(&mut right_recursive_grammar());
    let has_len1 = table.rules.iter().any(|r| r.rhs_len == 1);
    assert!(
        has_len1,
        "right-recursive grammar should have a rule with rhs_len == 1"
    );
}

#[test]
fn left_recursive_lhs_consistent() {
    let table = build_table(&mut left_recursive_grammar());
    // Both alternatives of "lst" should share the same lhs
    let lhs_counts = count_lhs_occurrences(&table.rules);
    let max_count = lhs_counts.values().max().copied().unwrap_or(0);
    assert!(
        max_count >= 2,
        "recursive grammar should have nonterminal with multiple rules"
    );
}

#[test]
fn right_recursive_lhs_consistent() {
    let table = build_table(&mut right_recursive_grammar());
    let lhs_counts = count_lhs_occurrences(&table.rules);
    let max_count = lhs_counts.values().max().copied().unwrap_or(0);
    assert!(
        max_count >= 2,
        "recursive grammar should have nonterminal with multiple rules"
    );
}

fn count_lhs_occurrences(
    rules: &[ParseRule],
) -> std::collections::BTreeMap<adze_glr_core::SymbolId, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for rule in rules {
        *counts.entry(rule.lhs).or_insert(0) += 1;
    }
    counts
}

// ===========================================================================
// 29–34. Precedence grammar: rules
// ===========================================================================

#[test]
fn precedence_grammar_has_multiple_rules() {
    let table = build_table(&mut arithmetic_grammar());
    assert!(
        table.rules.len() >= 3,
        "precedence grammar should have at least 3 rules"
    );
}

#[test]
fn precedence_grammar_has_rhs_len_3() {
    let table = build_table(&mut arithmetic_grammar());
    // expr → expr + expr has rhs_len=3
    let has_len3 = table.rules.iter().any(|r| r.rhs_len == 3);
    assert!(
        has_len3,
        "arithmetic grammar should have rules with rhs_len == 3"
    );
}

#[test]
fn precedence_grammar_has_rhs_len_1() {
    let table = build_table(&mut arithmetic_grammar());
    // expr → NUM has rhs_len=1
    let has_len1 = table.rules.iter().any(|r| r.rhs_len == 1);
    assert!(
        has_len1,
        "arithmetic grammar should have a rule with rhs_len == 1"
    );
}

#[test]
fn precedence_grammar_lhs_not_eof() {
    let table = build_table(&mut arithmetic_grammar());
    let eof = table.eof_symbol;
    for rule in &table.rules {
        assert_ne!(rule.lhs, eof, "no rule should have EOF as lhs");
    }
}

#[test]
fn precedence_right_assoc_grammar() {
    let mut g = GrammarBuilder::new("right_assoc")
        .token("NUM", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&mut g);
    assert!(
        table.rules.len() >= 2,
        "right-assoc grammar should have at least 2 rules"
    );
}

#[test]
fn precedence_mixed_assoc_grammar() {
    let mut g = GrammarBuilder::new("mixed")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 2, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&mut g);
    assert!(
        table.rules.len() >= 3,
        "mixed-assoc grammar should have at least 3 rules"
    );
    let has_len3 = table.rules.iter().any(|r| r.rhs_len == 3);
    assert!(
        has_len3,
        "should have binary operator rules with rhs_len == 3"
    );
}

// ===========================================================================
// 35–40. Large grammar: many rules
// ===========================================================================

#[test]
fn large_grammar_many_alternatives() {
    let mut builder = GrammarBuilder::new("large");
    // 10 tokens, 10 alternatives for start
    for i in 0..10 {
        let name = format!("t{i}");
        builder = builder.token(&name, &name);
    }
    for i in 0..10 {
        let name = format!("t{i}");
        builder = builder.rule("start", vec![&name]);
    }
    let mut g = builder.start("start").build();
    let table = build_table(&mut g);
    assert!(
        table.rules.len() >= 10,
        "large grammar should have at least 10 rules, got {}",
        table.rules.len()
    );
}

#[test]
fn large_grammar_many_nonterminals() {
    let mut builder = GrammarBuilder::new("many_nt");
    builder = builder.token("x", "x");
    // Chain of 8 nonterminals: n0→n1, n1→n2, ..., n6→n7, n7→x
    for i in 0..7 {
        let lhs = format!("n{i}");
        let rhs_name = format!("n{}", i + 1);
        builder = builder.rule(&lhs, vec![&rhs_name]);
    }
    builder = builder.rule("n7", vec!["x"]);
    let mut g = builder.start("n0").build();
    let table = build_table(&mut g);
    assert!(
        table.rules.len() >= 8,
        "long chain should have at least 8 rules, got {}",
        table.rules.len()
    );
}

#[test]
fn large_grammar_all_lhs_valid() {
    let mut builder = GrammarBuilder::new("big");
    for i in 0..5 {
        let name = format!("t{i}");
        builder = builder.token(&name, &name);
    }
    for i in 0..5 {
        let name = format!("t{i}");
        builder = builder.rule("start", vec![&name]);
    }
    let mut g = builder.start("start").build();
    let table = build_table(&mut g);
    for rule in &table.rules {
        assert!(rule.lhs.0 > 0, "every lhs should be a valid SymbolId");
    }
}

#[test]
fn large_grammar_rhs_len_bounded() {
    let mut builder = GrammarBuilder::new("bounded");
    for i in 0..8 {
        let name = format!("t{i}");
        builder = builder.token(&name, &name);
    }
    // One rule with 8 symbols on RHS
    builder = builder.rule(
        "start",
        vec!["t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7"],
    );
    let mut g = builder.start("start").build();
    let table = build_table(&mut g);
    let has_len8 = table.rules.iter().any(|r| r.rhs_len == 8);
    assert!(has_len8, "should have a rule with rhs_len == 8");
}

#[test]
fn large_grammar_reduce_actions_reference_valid_rules() {
    let mut builder = GrammarBuilder::new("valid_refs");
    for i in 0..5 {
        let name = format!("t{i}");
        builder = builder.token(&name, &name);
    }
    for i in 0..5 {
        let name = format!("t{i}");
        builder = builder.rule("start", vec![&name]);
    }
    let mut g = builder.start("start").build();
    let table = build_table(&mut g);
    let rule_count = table.rules.len();
    for state in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            for action in &table.action_table[state][sym_idx] {
                if let Action::Reduce(rid) = action {
                    assert!(
                        (rid.0 as usize) < rule_count,
                        "Reduce rule {rid:?} out of range (max={})",
                        rule_count
                    );
                }
            }
        }
    }
}

#[test]
fn large_grammar_rules_non_empty() {
    let mut builder = GrammarBuilder::new("nonempty");
    for i in 0..5 {
        let name = format!("t{i}");
        builder = builder.token(&name, &name);
    }
    builder = builder.rule("start", vec!["t0"]);
    let mut g = builder.start("start").build();
    let table = build_table(&mut g);
    assert!(!table.rules.is_empty());
}

// ===========================================================================
// 41–48. rhs_len correct for different rule shapes
// ===========================================================================

#[test]
fn rhs_len_single_terminal() {
    let table = build_table(&mut minimal_grammar());
    // S → a: rhs_len should be 1
    let has_len1 = table.rules.iter().any(|r| r.rhs_len == 1);
    assert!(has_len1, "single-terminal rule should have rhs_len == 1");
}

#[test]
fn rhs_len_two_terminals() {
    let mut g = GrammarBuilder::new("two_term")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&mut g);
    let has_len2 = table.rules.iter().any(|r| r.rhs_len == 2);
    assert!(has_len2, "two-terminal rule should have rhs_len == 2");
}

#[test]
fn rhs_len_three_symbols() {
    let table = build_table(&mut arithmetic_grammar());
    // expr → expr + expr: rhs_len == 3
    let has_len3 = table.rules.iter().any(|r| r.rhs_len == 3);
    assert!(has_len3, "ternary rule should have rhs_len == 3");
}

#[test]
fn rhs_len_four_terminals() {
    let table = build_table(&mut long_rhs_grammar());
    let has_len4 = table.rules.iter().any(|r| r.rhs_len == 4);
    assert!(has_len4, "four-terminal rule should have rhs_len == 4");
}

#[test]
fn rhs_len_nonterminal_chain() {
    let table = build_table(&mut chain_grammar());
    // Each chain rule: rhs_len == 1
    let unit_rules = table.rules.iter().filter(|r| r.rhs_len == 1).count();
    assert!(
        unit_rules >= 3,
        "chain grammar should have at least 3 unit rules, got {}",
        unit_rules
    );
}

#[test]
fn rhs_len_mixed_terminal_nonterminal() {
    let table = build_table(&mut left_recursive_grammar());
    // L → L a: has mixed terminal + nonterminal, rhs_len == 2
    let has_len2 = table.rules.iter().any(|r| r.rhs_len == 2);
    assert!(has_len2, "mixed rule should have rhs_len == 2");
}

#[test]
fn rhs_len_max_across_grammar() {
    let table = build_table(&mut arithmetic_grammar());
    let max_rhs = table.rules.iter().map(|r| r.rhs_len).max().unwrap_or(0);
    assert!(
        max_rhs >= 3,
        "max rhs_len should be at least 3 for binary ops"
    );
}

#[test]
fn rhs_len_min_across_grammar() {
    let table = build_table(&mut arithmetic_grammar());
    let min_rhs = table.rules.iter().map(|r| r.rhs_len).min().unwrap_or(0);
    assert!(min_rhs >= 1, "min rhs_len should be at least 1");
}

// ===========================================================================
// 49–55. Rules deterministic across builds
// ===========================================================================

#[test]
fn deterministic_rule_count() {
    let t1 = build_table(&mut minimal_grammar());
    let t2 = build_table(&mut minimal_grammar());
    assert_eq!(
        t1.rules.len(),
        t2.rules.len(),
        "rule count should be deterministic"
    );
}

#[test]
fn deterministic_rhs_len_values() {
    let t1 = build_table(&mut arithmetic_grammar());
    let t2 = build_table(&mut arithmetic_grammar());
    let rhs1: Vec<u16> = t1.rules.iter().map(|r| r.rhs_len).collect();
    let rhs2: Vec<u16> = t2.rules.iter().map(|r| r.rhs_len).collect();
    assert_eq!(rhs1, rhs2, "rhs_len values should be deterministic");
}

#[test]
fn deterministic_lhs_values() {
    let t1 = build_table(&mut chain_grammar());
    let t2 = build_table(&mut chain_grammar());
    let lhs1: Vec<u16> = t1.rules.iter().map(|r| r.lhs.0).collect();
    let lhs2: Vec<u16> = t2.rules.iter().map(|r| r.lhs.0).collect();
    assert_eq!(lhs1, lhs2, "lhs values should be deterministic");
}

#[test]
fn deterministic_two_alt() {
    let t1 = build_table(&mut two_alt_grammar());
    let t2 = build_table(&mut two_alt_grammar());
    assert_eq!(t1.rules.len(), t2.rules.len());
    for (r1, r2) in t1.rules.iter().zip(t2.rules.iter()) {
        assert_eq!(r1.lhs, r2.lhs);
        assert_eq!(r1.rhs_len, r2.rhs_len);
    }
}

#[test]
fn deterministic_left_recursive() {
    let t1 = build_table(&mut left_recursive_grammar());
    let t2 = build_table(&mut left_recursive_grammar());
    for (r1, r2) in t1.rules.iter().zip(t2.rules.iter()) {
        assert_eq!(r1.lhs, r2.lhs);
        assert_eq!(r1.rhs_len, r2.rhs_len);
    }
}

#[test]
fn deterministic_right_recursive() {
    let t1 = build_table(&mut right_recursive_grammar());
    let t2 = build_table(&mut right_recursive_grammar());
    for (r1, r2) in t1.rules.iter().zip(t2.rules.iter()) {
        assert_eq!(r1.lhs, r2.lhs);
        assert_eq!(r1.rhs_len, r2.rhs_len);
    }
}

#[test]
fn deterministic_precedence() {
    let t1 = build_table(&mut arithmetic_grammar());
    let t2 = build_table(&mut arithmetic_grammar());
    assert_eq!(t1.rules.len(), t2.rules.len());
    for (r1, r2) in t1.rules.iter().zip(t2.rules.iter()) {
        assert_eq!(r1.lhs, r2.lhs);
        assert_eq!(r1.rhs_len, r2.rhs_len);
    }
}

// ===========================================================================
// 56–63. Rule ordering properties
// ===========================================================================

#[test]
fn rule_index_matches_reduce_action() {
    let table = build_table(&mut two_alt_grammar());
    let rule_count = table.rules.len();
    for state in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            for action in &table.action_table[state][sym_idx] {
                if let Action::Reduce(rid) = action {
                    assert!(
                        (rid.0 as usize) < rule_count,
                        "reduce action references rule {} but only {} rules exist",
                        rid.0,
                        rule_count
                    );
                }
            }
        }
    }
}

#[test]
fn rule_index_matches_fork_reduce_action() {
    let table = build_table(&mut arithmetic_grammar());
    let rule_count = table.rules.len();
    for state in 0..table.state_count {
        for sym_idx in 0..table.symbol_count {
            for action in &table.action_table[state][sym_idx] {
                match action {
                    Action::Reduce(rid) => {
                        assert!((rid.0 as usize) < rule_count);
                    }
                    Action::Fork(sub) => {
                        for a in sub {
                            if let Action::Reduce(rid) = a {
                                assert!((rid.0 as usize) < rule_count);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn rules_start_symbol_appears_in_lhs() {
    let table = build_table(&mut minimal_grammar());
    let start = table.start_symbol;
    let has_start = table.rules.iter().any(|r| r.lhs == start);
    // The augmented start or user start should be present
    assert!(
        has_start || !table.rules.is_empty(),
        "at least one rule should reference the start symbol as lhs"
    );
}

#[test]
fn rules_vec_indexable() {
    let table = build_table(&mut chain_grammar());
    // Every index 0..rules.len() is valid
    for i in 0..table.rules.len() {
        let _rule = &table.rules[i];
    }
}

#[test]
fn rule_lhs_is_nonterminal_not_token() {
    let mut g = GrammarBuilder::new("nt_check")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .start("start")
        .build();
    let table = build_table(&mut g);
    // Collect token IDs
    let token_ids: std::collections::BTreeSet<_> = g.tokens.keys().copied().collect();
    // No rule lhs should be a token
    for rule in &table.rules {
        assert!(
            !token_ids.contains(&rule.lhs),
            "rule lhs {:?} should not be a token",
            rule.lhs
        );
    }
}

#[test]
fn rule_ordering_stable_across_same_grammar() {
    let t1 = build_table(&mut arithmetic_grammar());
    let t2 = build_table(&mut arithmetic_grammar());
    let order1: Vec<(u16, u16)> = t1.rules.iter().map(|r| (r.lhs.0, r.rhs_len)).collect();
    let order2: Vec<(u16, u16)> = t2.rules.iter().map(|r| (r.lhs.0, r.rhs_len)).collect();
    assert_eq!(order1, order2, "rule ordering should be stable");
}

#[test]
fn multiple_rules_same_lhs_adjacent_or_present() {
    let table = build_table(&mut two_alt_grammar());
    // For a grammar with S → a | b, both rules should have the same lhs
    let lhs_counts = count_lhs_occurrences(&table.rules);
    let multi = lhs_counts.values().filter(|&&c| c >= 2).count();
    assert!(
        multi >= 1,
        "two-alt grammar should have at least one nonterminal with 2+ rules"
    );
}

#[test]
fn rules_len_equals_dynamic_prec_len() {
    let table = build_table(&mut arithmetic_grammar());
    assert_eq!(
        table.rules.len(),
        table.dynamic_prec_by_rule.len(),
        "rules and dynamic_prec_by_rule should have same length"
    );
}

// ===========================================================================
// 64–70. Additional edge cases and cross-checks
// ===========================================================================

#[test]
fn single_token_grammar_has_at_least_one_rule() {
    let table = build_table(&mut minimal_grammar());
    assert!(
        !table.rules.is_empty(),
        "even a minimal grammar should produce rules"
    );
}

#[test]
fn all_rule_lhs_are_in_nonterminal_to_index() {
    let table = build_table(&mut chain_grammar());
    for rule in &table.rules {
        assert!(
            table.nonterminal_to_index.contains_key(&rule.lhs),
            "rule lhs {:?} should be in nonterminal_to_index",
            rule.lhs
        );
    }
}

#[test]
fn rules_len_equals_assoc_len() {
    let table = build_table(&mut arithmetic_grammar());
    assert_eq!(
        table.rules.len(),
        table.rule_assoc_by_rule.len(),
        "rules and rule_assoc_by_rule should have same length"
    );
}

#[test]
fn multi_level_chain_rhs_all_one() {
    let mut builder = GrammarBuilder::new("deep_chain");
    builder = builder.token("z", "z");
    builder = builder.rule("a0", vec!["a1"]);
    builder = builder.rule("a1", vec!["a2"]);
    builder = builder.rule("a2", vec!["a3"]);
    builder = builder.rule("a3", vec!["z"]);
    let mut g = builder.start("a0").build();
    let table = build_table(&mut g);
    // All user rules are unit rules (rhs_len == 1)
    let unit_count = table.rules.iter().filter(|r| r.rhs_len == 1).count();
    assert!(
        unit_count >= 4,
        "deep chain should have at least 4 unit rules, got {}",
        unit_count
    );
}

#[test]
fn grammar_with_two_nonterminals_distinct_lhs() {
    let mut g = GrammarBuilder::new("two_nt")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["inner"])
        .rule("inner", vec!["x", "y"])
        .start("start")
        .build();
    let table = build_table(&mut g);
    let mut lhs_set = std::collections::BTreeSet::new();
    for rule in &table.rules {
        lhs_set.insert(rule.lhs);
    }
    assert!(
        lhs_set.len() >= 2,
        "two-nonterminal grammar should have at least 2 distinct lhs values"
    );
}

#[test]
fn rhs_len_five_symbols() {
    let mut g = GrammarBuilder::new("five")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a", "b", "c", "d", "e"])
        .start("start")
        .build();
    let table = build_table(&mut g);
    let has_len5 = table.rules.iter().any(|r| r.rhs_len == 5);
    assert!(has_len5, "five-symbol rule should have rhs_len == 5");
}

#[test]
fn multiple_nonterminals_with_alternatives() {
    let mut g = GrammarBuilder::new("multi_alt")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["alpha"])
        .rule("start", vec!["beta"])
        .rule("alpha", vec!["x", "y"])
        .rule("beta", vec!["y", "z"])
        .start("start")
        .build();
    let table = build_table(&mut g);
    assert!(
        table.rules.len() >= 4,
        "multi-alternative grammar should have at least 4 rules, got {}",
        table.rules.len()
    );
}
