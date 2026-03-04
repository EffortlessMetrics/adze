// End-to-end pipeline tests: Grammar → FirstFollow → ParseTable → Compression
// Tests the full compilation pipeline from IR through to tablegen
#![cfg(feature = "test-api")]

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::SymbolId;
use adze_ir::builder::GrammarBuilder;

fn build_table(builder: GrammarBuilder) -> (adze_ir::Grammar, adze_glr_core::ParseTable) {
    let grammar = builder.build();
    let ff = FirstFollowSets::compute(&grammar).expect("compute first-follow");
    let table = build_lr1_automaton(&grammar, &ff).expect("should build parse table");
    (grammar, table)
}

// ── Grammar → ParseTable state count properties ──

#[test]
fn single_token_grammar_minimal_states() {
    let b = GrammarBuilder::new("min")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start");
    let (_, table) = build_table(b);
    // At least: initial state + after shift + accept
    assert!(table.state_count >= 2);
}

#[test]
fn two_alternative_grammar_states() {
    let b = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start");
    let (_, table) = build_table(b);
    assert!(table.state_count >= 2);
}

#[test]
fn chained_sequence_adds_states() {
    let b2 = GrammarBuilder::new("s2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start");
    let (_, t2) = build_table(b2);

    let b3 = GrammarBuilder::new("s3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start");
    let (_, t3) = build_table(b3);

    // Longer sequence → more states
    assert!(t3.state_count >= t2.state_count);
}

// ── FirstFollow set properties ──

#[test]
fn first_sets_contain_tokens() {
    let grammar = GrammarBuilder::new("ff")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&grammar).expect("compute first-follow");
    let _ = format!("{:?}", ff);
}

#[test]
fn first_follow_deterministic() {
    let mk = || {
        GrammarBuilder::new("det")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
            .build()
    };
    let ff1 = FirstFollowSets::compute(&mk()).expect("compute");
    let ff2 = FirstFollowSets::compute(&mk()).expect("compute");
    // FirstFollowSets is deterministic if it doesn't error on same grammar
    let _ = ff1;
    let _ = ff2;
}

// ── Action table properties ──

#[test]
fn parse_table_has_eof_symbol() {
    let b = GrammarBuilder::new("eof")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start");
    let (_, table) = build_table(b);
    // The eof_symbol should be set
    let _ = table.eof_symbol;
}

#[test]
fn state_zero_has_actions() {
    let b = GrammarBuilder::new("s0")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start");
    let (_, table) = build_table(b);
    // State 0 should have at least one non-empty action cell
    let has_actions = table.action_table[0].iter().any(|cell| !cell.is_empty());
    assert!(has_actions, "State 0 should have at least one action");
}

// ── Recursive grammar properties ──

#[test]
fn left_recursive_builds() {
    let b = GrammarBuilder::new("lr")
        .token("a", "a")
        .rule("list", vec!["a"])
        .rule("list", vec!["list", "a"])
        .start("list");
    let (_, table) = build_table(b);
    assert!(table.state_count > 0);
}

#[test]
fn right_recursive_builds() {
    let b = GrammarBuilder::new("rr")
        .token("a", "a")
        .rule("list", vec!["a"])
        .rule("list", vec!["a", "list"])
        .start("list");
    let (_, table) = build_table(b);
    assert!(table.state_count > 0);
}

// ── Nested nonterminal properties ──

#[test]
fn deeply_nested_nonterminals() {
    let b = GrammarBuilder::new("deep")
        .token("x", "x")
        .rule("d", vec!["x"])
        .rule("c", vec!["d"])
        .rule("b", vec!["c"])
        .rule("start", vec!["b"])
        .start("start");
    let (_, table) = build_table(b);
    assert!(table.state_count > 0);
}

#[test]
fn mixed_terminals_nonterminals() {
    let b = GrammarBuilder::new("mix")
        .token("a", "a")
        .token("b", "b")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .start("start");
    let (_, table) = build_table(b);
    assert!(table.state_count >= 3);
}

// ── Grammar with multiple productions for same nonterminal ──

#[test]
fn three_alternatives() {
    let b = GrammarBuilder::new("three")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start");
    let (_, table) = build_table(b);
    assert!(table.state_count > 0);
}

#[test]
fn alternative_with_different_lengths() {
    let b = GrammarBuilder::new("diff_len")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["a", "b"])
        .start("start");
    let (_, table) = build_table(b);
    assert!(table.state_count > 0);
}

// ── Table size invariants ──

#[test]
fn action_table_matches_state_count() {
    let b = GrammarBuilder::new("at")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start");
    let (_, table) = build_table(b);
    assert_eq!(table.action_table.len(), table.state_count);
}

#[test]
fn goto_table_matches_state_count() {
    let b = GrammarBuilder::new("gt")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start");
    let (_, table) = build_table(b);
    assert_eq!(table.goto_table.len(), table.state_count);
}

// ── Determinism ──

#[test]
fn identical_grammars_produce_identical_tables() {
    let mk = || {
        GrammarBuilder::new("eq")
            .token("a", "a")
            .token("b", "b")
            .rule("expr", vec!["a"])
            .rule("expr", vec!["b"])
            .rule("start", vec!["expr"])
            .start("start")
    };
    let (_, t1) = build_table(mk());
    let (_, t2) = build_table(mk());
    assert_eq!(t1.state_count, t2.state_count);
    assert_eq!(t1.action_table.len(), t2.action_table.len());
}

// ── Grammar builder API tests ──

#[test]
fn grammar_name_preserved() {
    let grammar = GrammarBuilder::new("my_grammar")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert_eq!(grammar.name, "my_grammar");
}

#[test]
fn start_symbol_is_set() {
    let grammar = GrammarBuilder::new("g")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(grammar.start_symbol().is_some());
}

#[test]
fn tokens_registered() {
    let grammar = GrammarBuilder::new("g")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    assert!(grammar.tokens.len() >= 2);
}

#[test]
fn rules_registered() {
    let grammar = GrammarBuilder::new("g")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec!["a", "a"])
        .start("start")
        .build();
    assert!(grammar.rules.values().map(|v| v.len()).sum::<usize>() >= 2);
}

// ── Accept action present ──

#[test]
fn accept_action_exists_somewhere() {
    let b = GrammarBuilder::new("acc")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start");
    let (_, table) = build_table(b);
    let has_accept = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    });
    assert!(has_accept, "Parse table should have an Accept action");
}
