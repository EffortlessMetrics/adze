#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for production handling in glr-core.
//!
//! Covers `ParseRule` (lhs, rhs_len), `ParseTable::rules`, `ParseTable::rule()`,
//! production indexing, and production properties derived from grammar construction
//! via `build_lr1_automaton`.
//!
//! Note: `build_lr1_automaton` iterates the *original* grammar's `all_rules()` to
//! populate `ParseTable::rules`.  The augmented start rule (S' → S) is included
//! because it is inserted into the augmented grammar before iteration.

use adze_glr_core::{
    Action, FirstFollowSets, ParseRule, ParseTable, RuleId, StateId, SymbolId, build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

#[allow(dead_code)]
fn tok_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found"))
}

fn nt_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("nonterminal '{name}' not found"))
}

/// Build a minimal grammar: start → a
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// Build a two-alternative grammar: start → a | b
fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("two_alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build()
}

/// Build a multi-symbol grammar: start → a b c
fn multi_symbol_grammar() -> Grammar {
    GrammarBuilder::new("multi_sym")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

/// Build a recursive grammar: expr → a | expr plus expr
fn recursive_grammar() -> Grammar {
    GrammarBuilder::new("recursive")
        .token("a", "a")
        .token("plus", "+")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .start("expr")
        .build()
}

/// Build a multi-nonterminal grammar: start → lhs rhs, lhs → a, rhs → b
fn multi_nt_grammar() -> Grammar {
    GrammarBuilder::new("multi_nt")
        .token("a", "a")
        .token("b", "b")
        .rule("lhs", vec!["a"])
        .rule("rhs", vec!["b"])
        .rule("start", vec!["lhs", "rhs"])
        .start("start")
        .build()
}

// ===========================================================================
// 1. ParseRule struct basics
// ===========================================================================

#[test]
fn parse_rule_stores_lhs_and_rhs_len() {
    let rule = ParseRule {
        lhs: SymbolId(10),
        rhs_len: 3,
    };
    assert_eq!(rule.lhs, SymbolId(10));
    assert_eq!(rule.rhs_len, 3);
}

#[test]
fn parse_rule_zero_length_rhs() {
    let rule = ParseRule {
        lhs: SymbolId(5),
        rhs_len: 0,
    };
    assert_eq!(rule.rhs_len, 0);
}

#[test]
fn parse_rule_clone() {
    let rule = ParseRule {
        lhs: SymbolId(7),
        rhs_len: 2,
    };
    let cloned = rule.clone();
    assert_eq!(cloned.lhs, rule.lhs);
    assert_eq!(cloned.rhs_len, rule.rhs_len);
}

#[test]
fn parse_rule_debug_format() {
    let rule = ParseRule {
        lhs: SymbolId(3),
        rhs_len: 1,
    };
    let dbg = format!("{:?}", rule);
    assert!(dbg.contains("ParseRule"), "debug should contain type name");
}

// ===========================================================================
// 2. ParseTable::rules populated from grammar
// ===========================================================================

#[test]
fn rules_non_empty_for_any_grammar() {
    let g = minimal_grammar();
    let table = build_table(&g);
    assert!(!table.rules.is_empty(), "rules must not be empty");
}

#[test]
fn rules_count_at_least_user_rules() {
    let g = minimal_grammar();
    let table = build_table(&g);
    // Original grammar has 1 rule (start → a)
    assert!(
        !table.rules.is_empty(),
        "should have at least the user rule, got {}",
        table.rules.len()
    );
}

#[test]
fn rules_count_for_two_alternatives() {
    let g = two_alt_grammar();
    let table = build_table(&g);
    // 2 user rules (start → a, start → b)
    assert!(
        table.rules.len() >= 2,
        "should have at least 2 rules, got {}",
        table.rules.len()
    );
}

#[test]
fn rules_lhs_for_minimal_grammar() {
    let g = minimal_grammar();
    let s = nt_id(&g, "start");
    let table = build_table(&g);

    let has_s = table.rules.iter().any(|r| r.lhs == s);
    assert!(has_s, "at least one rule must have 'start' as LHS");
}

#[test]
fn rules_rhs_len_for_single_terminal() {
    let g = minimal_grammar();
    let s = nt_id(&g, "start");
    let table = build_table(&g);

    let s_rules: Vec<_> = table.rules.iter().filter(|r| r.lhs == s).collect();
    assert!(
        s_rules.iter().any(|r| r.rhs_len == 1),
        "start → a should have rhs_len == 1"
    );
}

#[test]
fn rules_rhs_len_for_multi_symbol_rule() {
    let g = multi_symbol_grammar();
    let s = nt_id(&g, "start");
    let table = build_table(&g);

    let s_rules: Vec<_> = table.rules.iter().filter(|r| r.lhs == s).collect();
    assert!(
        s_rules.iter().any(|r| r.rhs_len == 3),
        "start → a b c should have rhs_len == 3"
    );
}

#[test]
fn rules_rhs_len_for_recursive_grammar() {
    let g = recursive_grammar();
    let e = nt_id(&g, "expr");
    let table = build_table(&g);

    let e_rules: Vec<_> = table.rules.iter().filter(|r| r.lhs == e).collect();
    let lengths: Vec<u16> = e_rules.iter().map(|r| r.rhs_len).collect();
    assert!(lengths.contains(&1), "expr → a should have rhs_len == 1");
    assert!(
        lengths.contains(&3),
        "expr → expr plus expr should have rhs_len == 3"
    );
}

// ===========================================================================
// 3. ParseTable::rule() accessor
// ===========================================================================

#[test]
fn rule_accessor_returns_correct_lhs_and_len() {
    let g = minimal_grammar();
    let s = nt_id(&g, "start");
    let table = build_table(&g);

    let idx = table
        .rules
        .iter()
        .position(|r| r.lhs == s && r.rhs_len == 1)
        .expect("start → a rule must exist");

    let (lhs, len) = table.rule(RuleId(idx as u16));
    assert_eq!(lhs, s);
    assert_eq!(len, 1);
}

#[test]
fn rule_accessor_consistency_with_rules_vec() {
    let g = recursive_grammar();
    let table = build_table(&g);

    for i in 0..table.rules.len() {
        let (lhs, rhs_len) = table.rule(RuleId(i as u16));
        assert_eq!(lhs, table.rules[i].lhs, "LHS mismatch at rule {}", i);
        assert_eq!(
            rhs_len, table.rules[i].rhs_len,
            "rhs_len mismatch at rule {}",
            i
        );
    }
}

#[test]
fn rule_accessor_for_all_rules_in_multi_nt() {
    let g = multi_nt_grammar();
    let table = build_table(&g);

    // Every rule should be queryable via rule()
    for i in 0..table.rules.len() {
        let (lhs, rhs_len) = table.rule(RuleId(i as u16));
        assert_eq!(lhs, table.rules[i].lhs);
        assert_eq!(rhs_len, table.rules[i].rhs_len);
    }
}

// ===========================================================================
// 4. Production indexing: Reduce actions reference valid rule IDs
// ===========================================================================

#[test]
fn reduce_actions_reference_valid_rule_ids() {
    let g = minimal_grammar();
    let table = build_table(&g);

    for state_idx in 0..table.state_count {
        let state = StateId(state_idx as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if let Action::Reduce(rule_id) = action {
                    assert!(
                        (rule_id.0 as usize) < table.rules.len(),
                        "Reduce({}) out of bounds (rules.len() = {})",
                        rule_id.0,
                        table.rules.len()
                    );
                }
            }
        }
    }
}

#[test]
fn reduce_actions_valid_in_recursive_grammar() {
    let g = recursive_grammar();
    let table = build_table(&g);

    for state_idx in 0..table.state_count {
        let state = StateId(state_idx as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if let Action::Reduce(rule_id) = action {
                    assert!(
                        (rule_id.0 as usize) < table.rules.len(),
                        "Reduce({}) out of bounds in recursive grammar",
                        rule_id.0,
                    );
                }
            }
        }
    }
}

#[test]
fn reduce_actions_valid_in_multi_nt_grammar() {
    let g = multi_nt_grammar();
    let table = build_table(&g);

    for state_idx in 0..table.state_count {
        let state = StateId(state_idx as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if let Action::Reduce(rule_id) = action {
                    assert!(
                        (rule_id.0 as usize) < table.rules.len(),
                        "Reduce({}) out of bounds in multi-nt grammar",
                        rule_id.0,
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 5. Production LHS matches expected nonterminals
// ===========================================================================

#[test]
fn production_lhs_values_are_known_symbols() {
    let g = multi_nt_grammar();
    let table = build_table(&g);
    let user_nts: Vec<SymbolId> = g.rule_names.keys().copied().collect();

    for rule in &table.rules {
        let is_user_nt = user_nts.contains(&rule.lhs);
        // Non-user LHS is the augmented start symbol
        let is_augmented =
            !g.rule_names.contains_key(&rule.lhs) && !g.tokens.contains_key(&rule.lhs);
        assert!(
            is_user_nt || is_augmented,
            "rule LHS {:?} should be a user nonterminal or augmented start",
            rule.lhs
        );
    }
}

#[test]
fn two_alt_productions_share_lhs() {
    let g = two_alt_grammar();
    let s = nt_id(&g, "start");
    let table = build_table(&g);

    let s_rule_count = table.rules.iter().filter(|r| r.lhs == s).count();
    assert_eq!(
        s_rule_count, 2,
        "start → a | b should yield exactly 2 rules with LHS = start"
    );
}

// ===========================================================================
// 6. Dynamic precedence and associativity parallel arrays
// ===========================================================================

#[test]
fn dynamic_prec_array_length_matches_rules() {
    let g = minimal_grammar();
    let table = build_table(&g);
    assert_eq!(
        table.dynamic_prec_by_rule.len(),
        table.rules.len(),
        "dynamic_prec_by_rule length must match rules count"
    );
}

#[test]
fn rule_assoc_array_length_matches_rules() {
    let g = minimal_grammar();
    let table = build_table(&g);
    assert_eq!(
        table.rule_assoc_by_rule.len(),
        table.rules.len(),
        "rule_assoc_by_rule length must match rules count"
    );
}

#[test]
fn precedence_stored_in_parallel_array() {
    let g = GrammarBuilder::new("prec")
        .token("a", "a")
        .token("plus", "+")
        .rule("expr", vec!["a"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 5, Associativity::Left)
        .start("expr")
        .build();
    let e = nt_id(&g, "expr");
    let table = build_table(&g);

    let idx = table
        .rules
        .iter()
        .position(|r| r.lhs == e && r.rhs_len == 3)
        .expect("expr → expr plus expr rule must exist");

    assert_eq!(table.dynamic_prec_by_rule[idx], 5, "precedence should be 5");
    assert_eq!(
        table.rule_assoc_by_rule[idx], 1,
        "Left associativity should be encoded as 1"
    );
}

#[test]
fn no_precedence_yields_zero() {
    let g = minimal_grammar();
    let s = nt_id(&g, "start");
    let table = build_table(&g);

    for i in 0..table.rules.len() {
        if table.rules[i].lhs == s {
            assert_eq!(
                table.dynamic_prec_by_rule[i], 0,
                "rule {} without precedence should have prec 0",
                i
            );
        }
    }
}

// ===========================================================================
// 7. Default ParseTable has empty rules
// ===========================================================================

#[test]
fn default_parse_table_has_no_rules() {
    let table = ParseTable::default();
    assert!(table.rules.is_empty(), "default table should have no rules");
}

#[test]
fn default_parse_table_has_empty_parallel_arrays() {
    let table = ParseTable::default();
    assert!(table.dynamic_prec_by_rule.is_empty());
    assert!(table.rule_assoc_by_rule.is_empty());
}

// ===========================================================================
// 8. Multi-nonterminal production coverage
// ===========================================================================

#[test]
fn multi_nt_grammar_covers_all_user_nonterminals() {
    let g = multi_nt_grammar();
    let table = build_table(&g);

    let lhs_nt = nt_id(&g, "lhs");
    let rhs_nt = nt_id(&g, "rhs");
    let start_nt = nt_id(&g, "start");

    assert!(
        table.rules.iter().any(|r| r.lhs == lhs_nt),
        "'lhs' should appear as production LHS"
    );
    assert!(
        table.rules.iter().any(|r| r.lhs == rhs_nt),
        "'rhs' should appear as production LHS"
    );
    assert!(
        table.rules.iter().any(|r| r.lhs == start_nt),
        "'start' should appear as production LHS"
    );
}

#[test]
fn multi_nt_rule_start_has_rhs_len_two() {
    let g = multi_nt_grammar();
    let s = nt_id(&g, "start");
    let table = build_table(&g);

    assert!(
        table.rules.iter().any(|r| r.lhs == s && r.rhs_len == 2),
        "start → lhs rhs should have rhs_len == 2"
    );
}

// ===========================================================================
// 9. Reduce action LHS matches goto nonterminal
// ===========================================================================

#[test]
fn reduce_lhs_has_goto_entry() {
    let g = minimal_grammar();
    let table = build_table(&g);

    for state_idx in 0..table.state_count {
        let state = StateId(state_idx as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if let Action::Reduce(rule_id) = action {
                    let (lhs, _) = table.rule(*rule_id);
                    if table.nonterminal_to_index.contains_key(&lhs) {
                        let has_goto = (0..table.state_count)
                            .any(|s| table.goto(StateId(s as u16), lhs).is_some());
                        assert!(
                            has_goto,
                            "LHS {:?} from Reduce({}) should have at least one goto entry",
                            lhs, rule_id.0,
                        );
                    }
                }
            }
        }
    }
}

// ===========================================================================
// 10. Edge cases: larger grammars
// ===========================================================================

#[test]
fn chain_grammar_production_lengths() {
    // start → item, item → inner, inner → leaf, leaf → x
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("inner", vec!["leaf"])
        .rule("item", vec!["inner"])
        .rule("start", vec!["item"])
        .start("start")
        .build();
    let table = build_table(&g);

    let user_nts: std::collections::BTreeSet<_> = g.rule_names.keys().copied().collect();
    for rule in &table.rules {
        if user_nts.contains(&rule.lhs) {
            assert_eq!(
                rule.rhs_len, 1,
                "chain grammar: every user rule should have rhs_len == 1"
            );
        }
    }
}

#[test]
fn wide_alternative_grammar() {
    // start → a | b | c | d | e  (5 alternatives)
    let g = GrammarBuilder::new("wide")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .rule("start", vec!["d"])
        .rule("start", vec!["e"])
        .start("start")
        .build();
    let s = nt_id(&g, "start");
    let table = build_table(&g);

    let s_count = table.rules.iter().filter(|r| r.lhs == s).count();
    assert_eq!(s_count, 5, "start should have 5 alternative productions");
}

#[test]
fn right_associative_precedence_stored() {
    let g = GrammarBuilder::new("right_assoc")
        .token("a", "a")
        .token("pow", "^")
        .rule("expr", vec!["a"])
        .rule_with_precedence(
            "expr",
            vec!["expr", "pow", "expr"],
            10,
            Associativity::Right,
        )
        .start("expr")
        .build();
    let e = nt_id(&g, "expr");
    let table = build_table(&g);

    let idx = table
        .rules
        .iter()
        .position(|r| r.lhs == e && r.rhs_len == 3)
        .expect("expr → expr pow expr rule must exist");

    assert_eq!(
        table.rule_assoc_by_rule[idx], -1,
        "Right assoc should be -1"
    );
    assert_eq!(table.dynamic_prec_by_rule[idx], 10);
}

// ===========================================================================
// 11. Production rule ordering and distinctness
// ===========================================================================

#[test]
fn reduce_rule_id_maps_back_to_correct_production() {
    let g = multi_nt_grammar();
    let table = build_table(&g);

    for state_idx in 0..table.state_count {
        let state = StateId(state_idx as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if let Action::Reduce(rule_id) = action {
                    let (lhs, rhs_len) = table.rule(*rule_id);
                    // Verify round-trip: the rule at this index has this LHS and len
                    let r = &table.rules[rule_id.0 as usize];
                    assert_eq!(r.lhs, lhs);
                    assert_eq!(r.rhs_len, rhs_len);
                }
            }
        }
    }
}

#[test]
fn parallel_arrays_synchronized_across_all_grammars() {
    for g in &[
        minimal_grammar(),
        two_alt_grammar(),
        multi_symbol_grammar(),
        recursive_grammar(),
        multi_nt_grammar(),
    ] {
        let table = build_table(g);
        assert_eq!(table.rules.len(), table.dynamic_prec_by_rule.len());
        assert_eq!(table.rules.len(), table.rule_assoc_by_rule.len());
    }
}

#[test]
fn no_negative_assoc_without_precedence_decl() {
    let g = multi_symbol_grammar();
    let table = build_table(&g);

    for i in 0..table.rules.len() {
        // Without any precedence declaration, assoc should be 0 (None)
        if g.rule_names.contains_key(&table.rules[i].lhs) {
            assert_eq!(
                table.rule_assoc_by_rule[i], 0,
                "assoc should be 0 when no precedence is declared"
            );
        }
    }
}

#[test]
fn fork_actions_contain_valid_reduce_ids() {
    let g = recursive_grammar();
    let table = build_table(&g);

    for state_idx in 0..table.state_count {
        let state = StateId(state_idx as u16);
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(state, sym) {
                if let Action::Fork(inner) = action {
                    for a in inner {
                        if let Action::Reduce(rule_id) = a {
                            assert!(
                                (rule_id.0 as usize) < table.rules.len(),
                                "Fork inner Reduce({}) out of bounds",
                                rule_id.0,
                            );
                        }
                    }
                }
            }
        }
    }
}
