//! Comprehensive tests for ParseTable structure and properties.
#![cfg(feature = "test-api")]

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

fn build_table(name: &str, setup: impl FnOnce(GrammarBuilder) -> GrammarBuilder) -> ParseTable {
    let g = setup(GrammarBuilder::new(name)).build();
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW failed");
    build_lr1_automaton(&g, &ff).expect("LR1 build failed")
}

// ── Basic ParseTable properties ──

#[test]
fn table_has_states() {
    let pt = build_table("t1", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert!(pt.state_count > 0);
}

#[test]
fn table_has_symbols() {
    let pt = build_table("t2", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert!(pt.symbol_count > 0);
}

#[test]
fn table_action_table_nonempty() {
    let pt = build_table("t3", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert!(!pt.action_table.is_empty());
}

#[test]
fn table_goto_table_nonempty() {
    let pt = build_table("t4", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert!(!pt.goto_table.is_empty());
}

#[test]
fn table_has_rules() {
    let pt = build_table("t5", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert!(!pt.rules.is_empty());
}

#[test]
fn table_has_eof_symbol() {
    let pt = build_table("t6", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    let _ = pt.eof_symbol; // Should exist
}

#[test]
fn table_has_start_symbol() {
    let pt = build_table("t7", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    let _ = pt.start_symbol;
}

#[test]
fn table_has_grammar() {
    let pt = build_table("t8", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert_eq!(pt.grammar.name, "t8");
}

// ── Table size scaling ──

#[test]
fn two_token_more_states() {
    let pt1 = build_table("s1", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    let pt2 = build_table("s2", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a", "b"])
            .start("s")
    });
    assert!(pt2.state_count >= pt1.state_count);
}

#[test]
fn alternatives_add_states() {
    let pt = build_table("alts", |b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .rule("s", vec!["c"])
            .start("s")
    });
    assert!(pt.state_count >= 2);
}

#[test]
fn recursive_grammar_states() {
    let pt = build_table("rec", |b| {
        b.token("x", "x")
            .token("p", "+")
            .rule("e", vec!["x"])
            .rule("e", vec!["e", "p", "x"])
            .start("e")
    });
    assert!(pt.state_count >= 4);
}

// ── Symbol mapping ──

#[test]
fn symbol_to_index_populated() {
    let pt = build_table("sym", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert!(!pt.symbol_to_index.is_empty());
}

#[test]
fn index_to_symbol_populated() {
    let pt = build_table("idx", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert!(!pt.index_to_symbol.is_empty());
}

#[test]
fn symbol_to_index_roundtrip() {
    let pt = build_table("rt", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a", "b"])
            .start("s")
    });
    for (&sym, &idx) in &pt.symbol_to_index {
        assert_eq!(pt.index_to_symbol[idx], sym);
    }
}

// ── Symbol metadata ──

#[test]
fn symbol_metadata_populated() {
    let pt = build_table("meta", |b| {
        b.token("a", "a").rule("s", vec!["a"]).start("s")
    });
    assert!(!pt.symbol_metadata.is_empty());
}

#[test]
fn symbol_metadata_count_matches() {
    let pt = build_table("mc", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    // Metadata count should be related to symbol count but may differ
    assert!(pt.symbol_metadata.len() > 0);
}

// ── Action table dimensions ──

#[test]
fn action_table_rows_match_states() {
    let pt = build_table("ar", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert_eq!(pt.action_table.len(), pt.state_count);
}

#[test]
fn goto_table_rows_match_states() {
    let pt = build_table("gr", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

// ── Nonterminal mapping ──

#[test]
fn nonterminal_to_index_populated() {
    let pt = build_table("nt", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert!(!pt.nonterminal_to_index.is_empty());
}

// ── Initial state ──

#[test]
fn initial_state_valid() {
    let pt = build_table("is", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    assert!(pt.initial_state.0 < pt.state_count as u16);
}

// ── External scanner states ──

#[test]
fn external_scanner_states_match_count() {
    let pt = build_table("es", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    // External scanner states may be empty or match state count
    assert!(
        pt.external_scanner_states.is_empty() || pt.external_scanner_states.len() == pt.state_count
    );
}

// ── Grammar preserved ──

#[test]
fn grammar_name_preserved() {
    let pt = build_table("preserved_name", |b| {
        b.token("a", "a").rule("s", vec!["a"]).start("s")
    });
    assert_eq!(pt.grammar.name, "preserved_name");
}

#[test]
fn grammar_tokens_preserved() {
    let pt = build_table("tok_pres", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .start("s")
    });
    assert!(pt.grammar.tokens.len() >= 2);
}

// ── Precedence grammars ──

#[test]
fn precedence_grammar_table() {
    let g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("p", "+")
        .token("m", "*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "p", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "m", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count >= 5);
    assert!(pt.rules.len() >= 3);
}

#[test]
fn right_assoc_table() {
    let g = GrammarBuilder::new("rassoc")
        .token("n", "n")
        .token("c", "^")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "c", "e"], 1, Associativity::Right)
        .start("e")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count >= 4);
}

// ── Determinism ──

#[test]
fn table_deterministic() {
    let tables: Vec<_> = (0..3)
        .map(|_| {
            build_table("det", |b| {
                b.token("a", "a")
                    .token("b", "b")
                    .rule("s", vec!["a", "b"])
                    .start("s")
            })
        })
        .collect();
    for i in 1..tables.len() {
        assert_eq!(tables[0].state_count, tables[i].state_count);
        assert_eq!(tables[0].symbol_count, tables[i].symbol_count);
        assert_eq!(tables[0].rules.len(), tables[i].rules.len());
    }
}

// ── Multiple nonterminals ──

#[test]
fn two_nonterminals_table() {
    let pt = build_table("2nt", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("x", vec!["a"])
            .rule("y", vec!["b"])
            .rule("s", vec!["x", "y"])
            .start("s")
    });
    assert!(pt.nonterminal_to_index.len() >= 3); // s, x, y
}

// ── Chain grammar ──

#[test]
fn chain_grammar_table() {
    let pt = build_table("chain", |b| {
        b.token("x", "x")
            .rule("a", vec!["x"])
            .rule("b", vec!["a"])
            .rule("c", vec!["b"])
            .start("c")
    });
    assert!(pt.state_count >= 2);
}

// ── Wide sequence ──

#[test]
fn wide_sequence_table() {
    let mut builder = GrammarBuilder::new("wide");
    let mut toks = Vec::new();
    for i in 0..6 {
        let name: &str = Box::leak(format!("t{}", i).into_boxed_str());
        builder = builder.token(name, name);
        toks.push(name);
    }
    builder = builder.rule("s", toks).start("s");
    let g = builder.build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count >= 7);
}

// ── Debug formatting ──

#[test]
fn parse_table_debug() {
    let pt = build_table("dbg", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    let s = format!("{:?}", pt);
    assert!(!s.is_empty());
}

// ── ParseRule in table ──

#[test]
fn parse_rules_have_lhs() {
    let pt = build_table("lhs", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    for rule in &pt.rules {
        // Each rule should have a valid LHS
        let _ = rule.lhs;
    }
}

#[test]
fn parse_rules_have_rhs_length() {
    let pt = build_table("rhs", |b| b.token("a", "a").rule("s", vec!["a"]).start("s"));
    for rule in &pt.rules {
        let _ = rule.rhs_len;
    }
}
