#![cfg(feature = "test-api")]

//! Comprehensive tests for LR(1) automaton building and FIRST/FOLLOW sets.
//!
//! Covers: single-production grammars, multiple alternatives, left/right
//! recursion, nullable rules, chained non-terminals, epsilon productions,
//! state counts, action correctness (Shift/Reduce/Accept), GOTO transitions,
//! and FIRST/FOLLOW set contents.

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a parse table from a grammar using the standard pipeline.
fn build_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW computation should succeed");
    build_lr1_automaton(grammar, &ff).expect("automaton construction should succeed")
}

/// Returns true if the parse table has an Accept action somewhere on EOF.
fn has_accept(table: &adze_glr_core::ParseTable) -> bool {
    let eof = table.eof();
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

/// Returns true if the initial state has a Shift action on the given terminal.
fn initial_shifts(table: &adze_glr_core::ParseTable, terminal: SymbolId) -> bool {
    table
        .actions(table.initial_state, terminal)
        .iter()
        .any(|a| matches!(a, Action::Shift(_)))
}

/// Collect all Reduce actions across all states for a given lookahead symbol.
fn collect_reduces(
    table: &adze_glr_core::ParseTable,
    lookahead: SymbolId,
) -> Vec<(StateId, RuleId)> {
    let mut out = Vec::new();
    for st in 0..table.state_count {
        let state = StateId(st as u16);
        for action in table.actions(state, lookahead) {
            if let Action::Reduce(rid) = action {
                out.push((state, *rid));
            }
        }
    }
    out
}

/// Find a token's SymbolId by name.
fn tok_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found in grammar"))
}

/// Find a rule nonterminal's SymbolId by name.
fn nt_id(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("nonterminal '{name}' not found in grammar"))
}

// ===========================================================================
// 1. Single production: start → a
// ===========================================================================

#[test]
fn single_production_has_accept() {
    let g = GrammarBuilder::new("tiny")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table), "table must contain an Accept action");
}

#[test]
fn single_production_state_count() {
    let g = GrammarBuilder::new("tiny")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3,
        "expected >= 3 states, got {}",
        table.state_count
    );
    assert!(
        table.state_count <= 8,
        "table should not explode; got {} states",
        table.state_count
    );
}

#[test]
fn single_production_shift_on_terminal() {
    let g = GrammarBuilder::new("tiny")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    assert!(
        initial_shifts(&table, a),
        "initial state must shift on terminal 'a'"
    );
}

// ===========================================================================
// 2. Two alternatives: start → a | b
// ===========================================================================

#[test]
fn two_alternatives_both_shift() {
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(initial_shifts(&table, tok_id(&g, "a")), "must shift on 'a'");
    assert!(initial_shifts(&table, tok_id(&g, "b")), "must shift on 'b'");
    assert!(has_accept(&table));
}

#[test]
fn two_alternatives_reduce_on_eof() {
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let reduces = collect_reduces(&table, table.eof());
    assert!(
        !reduces.is_empty(),
        "there must be at least one Reduce action on EOF"
    );
}

// ===========================================================================
// 3. Sequence: start → a b
// ===========================================================================

#[test]
fn sequence_two_terminals() {
    let g = GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");

    assert!(initial_shifts(&table, a), "must shift 'a' first");
    assert!(has_accept(&table));
    let shifts_b = (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), b)
            .iter()
            .any(|act| matches!(act, Action::Shift(_)))
    });
    assert!(shifts_b, "some state must shift on 'b'");
}

// ===========================================================================
// 4. Chained non-terminals: start → inner, inner → a
// ===========================================================================

#[test]
fn chained_nonterminal() {
    let g = GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(initial_shifts(&table, tok_id(&g, "a")));
    assert!(has_accept(&table));
}

// ===========================================================================
// 5. Left recursion: list → list a | a
// ===========================================================================

#[test]
fn left_recursion_builds_successfully() {
    let g = GrammarBuilder::new("lrec")
        .token("a", "a")
        .rule("list", vec!["list", "a"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(
        table.state_count >= 3,
        "left-recursive grammar needs >= 3 states, got {}",
        table.state_count
    );
}

#[test]
fn left_recursion_shift_on_initial() {
    let g = GrammarBuilder::new("lrec")
        .token("a", "a")
        .rule("list", vec!["list", "a"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(
        initial_shifts(&table, tok_id(&g, "a")),
        "left-recursive grammar must shift 'a' from initial state"
    );
}

// ===========================================================================
// 6. Right recursion: seq → a seq | a
// ===========================================================================

#[test]
fn right_recursion_builds_successfully() {
    let g = GrammarBuilder::new("rrec")
        .token("a", "a")
        .rule("seq", vec!["a", "seq"])
        .rule("seq", vec!["a"])
        .start("seq")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

// ===========================================================================
// 7. Nullable production: start → ε | a
// ===========================================================================

#[test]
fn nullable_production() {
    let g = GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

// ===========================================================================
// 8. Multiple non-terminals: start → lhs rhs, lhs → a, rhs → b
// ===========================================================================

#[test]
fn multiple_nonterminals() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("lhs", vec!["a"])
        .rule("rhs", vec!["b"])
        .rule("start", vec!["lhs", "rhs"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(
        initial_shifts(&table, tok_id(&g, "a")),
        "must shift 'a' first"
    );
}

// ===========================================================================
// 9. GOTO table: start → inner, inner → a — verify goto for inner exists
// ===========================================================================

#[test]
fn goto_exists_for_nonterminal() {
    let g = GrammarBuilder::new("goto")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let table = build_table(&g);
    let inner = nt_id(&g, "inner");
    let goto_exists =
        (0..table.state_count).any(|st| table.goto(StateId(st as u16), inner).is_some());
    assert!(
        goto_exists,
        "GOTO for nonterminal 'inner' must exist somewhere"
    );
}

// ===========================================================================
// 10. Simple arithmetic: expr → expr '+' term | term,  term → n
// ===========================================================================

#[test]
fn arithmetic_grammar() {
    let g = GrammarBuilder::new("arith")
        .token("n", "n")
        .token("+", "+")
        .rule("term", vec!["n"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(
        table.state_count >= 4,
        "arithmetic grammar needs >= 4 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// 11. FIRST set: terminal in FIRST(start) for start → a
// ===========================================================================

#[test]
fn first_set_contains_terminal() {
    let g = GrammarBuilder::new("first")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    let a = tok_id(&g, "a");
    let first = ff.first(s).expect("FIRST(start) must exist");
    assert!(
        first.contains(a.0 as usize),
        "FIRST(start) must contain terminal 'a'"
    );
}

// ===========================================================================
// 12. FIRST set propagation: start → inner, inner → a  ⇒  a ∈ FIRST(start)
// ===========================================================================

#[test]
fn first_set_propagation_through_nonterminal() {
    let g = GrammarBuilder::new("fprop")
        .token("a", "a")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    let a = tok_id(&g, "a");
    let first_s = ff.first(s).expect("FIRST(start) must exist");
    assert!(
        first_s.contains(a.0 as usize),
        "FIRST(start) must contain 'a' via inner"
    );
}

// ===========================================================================
// 13. FOLLOW set contains EOF for start symbol
// ===========================================================================

#[test]
fn follow_set_start_contains_eof() {
    let g = GrammarBuilder::new("feof")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    let follow_s = ff.follow(s).expect("FOLLOW(start) must exist");
    assert!(
        follow_s.contains(0),
        "FOLLOW(start symbol) must contain EOF sentinel"
    );
}

// ===========================================================================
// 14. Nullable detection
// ===========================================================================

#[test]
fn nullable_detected() {
    let g = GrammarBuilder::new("eps")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    assert!(
        ff.is_nullable(s),
        "start with ε production must be nullable"
    );
}

#[test]
fn non_nullable_detected() {
    let g = GrammarBuilder::new("noneps")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    assert!(
        !ff.is_nullable(s),
        "start without ε production must not be nullable"
    );
}

// ===========================================================================
// 15. FIRST set with alternatives: start → a | b  ⇒  {a, b} ⊆ FIRST(start)
// ===========================================================================

#[test]
fn first_set_union_of_alternatives() {
    let g = GrammarBuilder::new("funion")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");
    let first_s = ff.first(s).unwrap();
    assert!(
        first_s.contains(a.0 as usize),
        "'a' must be in FIRST(start)"
    );
    assert!(
        first_s.contains(b.0 as usize),
        "'b' must be in FIRST(start)"
    );
}

// ===========================================================================
// 16. FOLLOW set propagation: start → lhs rhs, lhs → a, rhs → b  ⇒  b ∈ FOLLOW(lhs)
// ===========================================================================

#[test]
fn follow_set_from_sequence() {
    let g = GrammarBuilder::new("fseq")
        .token("a", "a")
        .token("b", "b")
        .rule("lhs", vec!["a"])
        .rule("rhs", vec!["b"])
        .rule("start", vec!["lhs", "rhs"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let lhs = nt_id(&g, "lhs");
    let b = tok_id(&g, "b");
    let follow_lhs = ff.follow(lhs).expect("FOLLOW(lhs) must exist");
    assert!(
        follow_lhs.contains(b.0 as usize),
        "FOLLOW(lhs) must contain 'b' because start → lhs rhs"
    );
}

// ===========================================================================
// 17. Parse table rule metadata
// ===========================================================================

#[test]
fn parse_table_rules_populated() {
    let g = GrammarBuilder::new("rules")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    // The table must have at least the original rule start → a
    assert!(
        !table.rules.is_empty(),
        "parse table must have at least one rule, got {}",
        table.rules.len()
    );
}

// ===========================================================================
// 18. Three-level chain: start → mid, mid → leaf, leaf → x
// ===========================================================================

#[test]
fn three_level_chain() {
    let g = GrammarBuilder::new("chain3")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("start", vec!["mid"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));

    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "start");
    let x = tok_id(&g, "x");
    let first_s = ff.first(s).unwrap();
    assert!(
        first_s.contains(x.0 as usize),
        "FIRST(start) must contain 'x' via mid → leaf → x"
    );
}

// ===========================================================================
// 19. Ambiguous grammar: expr → expr '+' expr | n  (GLR handles it)
// ===========================================================================

#[test]
fn ambiguous_grammar_builds() {
    let g = GrammarBuilder::new("ambig")
        .token("n", "n")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 4);
}

// ===========================================================================
// 20. EOF symbol is distinct from all grammar symbols
// ===========================================================================

#[test]
fn eof_symbol_distinct() {
    let g = GrammarBuilder::new("eof")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    for (tok, _) in &g.tokens {
        assert_ne!(eof, *tok, "EOF must differ from token {:?}", tok);
    }
    for (rule, _) in &g.rules {
        assert_ne!(eof, *rule, "EOF must differ from rule {:?}", rule);
    }
}

// ===========================================================================
// 21. Start symbol preservation
// ===========================================================================

#[test]
fn start_symbol_preserved() {
    let g = GrammarBuilder::new("startp")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    assert_eq!(
        table.start_symbol(),
        s,
        "parse table start_symbol must match grammar start"
    );
}

// ===========================================================================
// 22. remap_goto_to_direct_symbol_id round-trip
// ===========================================================================

#[test]
fn remap_goto_preserves_accept() {
    let g = GrammarBuilder::new("remap")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g).remap_goto_to_direct_symbol_id();
    assert!(has_accept(&table), "Accept must survive GOTO remapping");
}

// ===========================================================================
// 23. FIRST set for left-recursive nonterminal
// ===========================================================================

#[test]
fn first_set_left_recursive() {
    let g = GrammarBuilder::new("lrfirst")
        .token("a", "a")
        .token("b", "b")
        .rule("list", vec!["list", "b"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let list = nt_id(&g, "list");
    let a = tok_id(&g, "a");
    let b = tok_id(&g, "b");

    let first_list = ff.first(list).unwrap();
    assert!(
        first_list.contains(a.0 as usize),
        "'a' must be in FIRST(list)"
    );
    // 'b' should NOT be in FIRST(list) since list always starts with 'a'
    assert!(
        !first_list.contains(b.0 as usize),
        "'b' must NOT be in FIRST(list)"
    );
}

// ===========================================================================
// 24. FOLLOW set for right-recursive nonterminal
// ===========================================================================

#[test]
fn follow_set_right_recursive() {
    // seq → a seq | a   ⇒  FOLLOW(seq) should contain EOF
    let g = GrammarBuilder::new("rrfollow")
        .token("a", "a")
        .rule("seq", vec!["a", "seq"])
        .rule("seq", vec!["a"])
        .start("seq")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let seq = nt_id(&g, "seq");
    let follow_seq = ff.follow(seq).expect("FOLLOW(seq) must exist");
    assert!(
        follow_seq.contains(0),
        "FOLLOW(seq) must contain EOF for start symbol"
    );
}

// ===========================================================================
// 25. Multiple tokens in sequence produce correct state count
// ===========================================================================

#[test]
fn long_sequence_state_count() {
    // start → a b c  requires states for each shift position
    let g = GrammarBuilder::new("longseq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    // Need at least 4 states: initial, after-a, after-b, after-c/goto-start
    assert!(
        table.state_count >= 4,
        "3-token sequence needs >= 4 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// 26. No spurious shift for absent terminals
// ===========================================================================

#[test]
fn no_shift_on_absent_terminal() {
    // Grammar only uses 'a'; should not shift on 'b' from initial state
    let g = GrammarBuilder::new("absent")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let b = tok_id(&g, "b");
    assert!(
        !initial_shifts(&table, b),
        "initial state must NOT shift on unused terminal 'b'"
    );
}

// ===========================================================================
// 27. GOTO for start symbol exists in initial state
// ===========================================================================

#[test]
fn goto_for_start_symbol() {
    let g = GrammarBuilder::new("gstart")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let s = nt_id(&g, "start");
    let goto = table.goto(table.initial_state, s);
    assert!(
        goto.is_some(),
        "goto(initial, start) must exist for augmented grammar"
    );
}
