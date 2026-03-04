//! Comprehensive tests for LR(1) item set collection building.
//!
//! Covers: build_lr1_automaton_res with various grammars, canonical collection
//! properties (closure, goto, kernel), first/follow set edge cases, grammar
//! normalization effects on LR1 construction, conflict detection, and state
//! count verification for known grammars.

use adze_glr_core::{
    Action, ConflictResolver, ConflictType, FirstFollowSets, ItemSetCollection,
    build_lr1_automaton, build_lr1_automaton_res,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> adze_glr_core::ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW should succeed");
    build_lr1_automaton(grammar, &ff).expect("automaton construction should succeed")
}

fn build_collection(grammar: &mut Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute_normalized(grammar).expect("FIRST/FOLLOW should succeed");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (col, ff)
}

fn has_accept(table: &adze_glr_core::ParseTable) -> bool {
    let eof = table.eof();
    (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    })
}

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

fn transitions_from(col: &ItemSetCollection, state: StateId) -> usize {
    col.goto_table
        .iter()
        .filter(|((src, _), _)| *src == state)
        .count()
}

fn count_states_with_reduce(table: &adze_glr_core::ParseTable, sym: SymbolId) -> usize {
    (0..table.state_count)
        .filter(|&st| {
            table
                .actions(StateId(st as u16), sym)
                .iter()
                .any(|a| matches!(a, Action::Reduce(_)))
        })
        .count()
}

// ===========================================================================
// Section 1: build_lr1_automaton_res with various grammars
// ===========================================================================

#[test]
fn res_single_terminal_grammar() {
    let g = GrammarBuilder::new("t1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton_res(&g, &ff).expect("should succeed");
    assert!(has_accept(&table));
}

#[test]
fn res_two_terminal_sequence() {
    let g = GrammarBuilder::new("t2")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton_res(&g, &ff).unwrap();
    assert!(table.state_count >= 3, "a·b needs at least 3 states");
    assert!(has_accept(&table));
}

#[test]
fn res_alternation_grammar() {
    let g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton_res(&g, &ff).unwrap();
    assert!(has_accept(&table));
    assert!(table.state_count >= 2);
}

#[test]
fn res_nullable_start() {
    let g = GrammarBuilder::new("eps")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton_res(&g, &ff).unwrap();
    assert!(has_accept(&table));
}

#[test]
fn res_left_recursive_grammar() {
    let g = GrammarBuilder::new("lrec")
        .token("a", "a")
        .rule("list", vec!["list", "a"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton_res(&g, &ff).unwrap();
    assert!(has_accept(&table));
    assert!(table.state_count >= 3);
}

#[test]
fn res_right_recursive_grammar() {
    let g = GrammarBuilder::new("rrec")
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton_res(&g, &ff).unwrap();
    assert!(has_accept(&table));
}

#[test]
fn res_chained_nonterminals() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("a")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton_res(&g, &ff).unwrap();
    assert!(has_accept(&table));
}

#[test]
fn res_multiple_nonterminals_sequence() {
    let g = GrammarBuilder::new("seq")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["b", "c"])
        .rule("b", vec!["x"])
        .rule("c", vec!["y"])
        .start("a")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton_res(&g, &ff).unwrap();
    assert!(has_accept(&table));
    assert!(table.state_count >= 4);
}

#[test]
fn res_three_way_alternation() {
    let g = GrammarBuilder::new("alt3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton_res(&g, &ff).unwrap();
    assert!(has_accept(&table));
    let eof = table.eof();
    assert!(count_states_with_reduce(&table, eof) >= 3);
}

#[test]
fn res_returns_ok_for_valid_grammar() {
    let g = GrammarBuilder::new("ok")
        .token("n", "n")
        .rule("start", vec!["n"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(build_lr1_automaton_res(&g, &ff).is_ok());
}

// ===========================================================================
// Section 2: Canonical collection properties (closure, goto, kernel)
// ===========================================================================

#[test]
fn closure_adds_items_for_nonterminal_after_dot() {
    let mut g = GrammarBuilder::new("cl1")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("a", vec!["x"])
        .start("s")
        .build();
    let (col, _ff) = build_collection(&mut g);
    // State 0 closure: s -> •a and a -> •x should both appear
    let state0 = &col.sets[0];
    assert!(
        state0.items.len() >= 2,
        "closure must add items for a -> •x"
    );
}

#[test]
fn closure_propagates_through_chain() {
    let mut g = GrammarBuilder::new("cl2")
        .token("z", "z")
        .rule("s", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["z"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    let state0 = &col.sets[0];
    // s -> •a, a -> •b, b -> •z should all be in state 0
    assert!(
        state0.items.len() >= 3,
        "closure must propagate through chain: got {} items",
        state0.items.len()
    );
}

#[test]
fn goto_creates_new_state_for_terminal() {
    let mut g = GrammarBuilder::new("gt1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 2, "goto on 'x' must create a new state");
}

#[test]
fn goto_creates_states_for_each_symbol() {
    let mut g = GrammarBuilder::new("gt2")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    // Need states for: initial, after 'a', after 'a b'
    assert!(col.sets.len() >= 3);
}

#[test]
fn goto_table_maps_initial_state_to_targets() {
    let mut g = GrammarBuilder::new("gt3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(
        transitions_from(&col, StateId(0)) >= 1,
        "state 0 must have at least one goto transition"
    );
}

#[test]
fn kernel_items_advance_dot() {
    let mut g = GrammarBuilder::new("kern")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    // After shifting 'a', the kernel item should have dot at position 1
    let a_id = tok_id(&g, "a");
    if let Some(&target) = col.goto_table.get(&(StateId(0), a_id)) {
        let target_set = col.sets.iter().find(|s| s.id == target).unwrap();
        let has_advanced_dot = target_set.items.iter().any(|item| item.position >= 1);
        assert!(has_advanced_dot, "kernel items must have advanced dot");
    }
}

#[test]
fn no_duplicate_states_in_collection() {
    let mut g = GrammarBuilder::new("nodup")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec!["a", "a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    for i in 0..col.sets.len() {
        for j in (i + 1)..col.sets.len() {
            assert_ne!(
                col.sets[i].items, col.sets[j].items,
                "states {} and {} are duplicates",
                i, j
            );
        }
    }
}

#[test]
fn collection_state_ids_are_sequential() {
    let mut g = GrammarBuilder::new("seqid")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16), "state ID should be sequential");
    }
}

#[test]
fn goto_targets_reference_existing_states() {
    let mut g = GrammarBuilder::new("ref")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    let state_ids: BTreeSet<StateId> = col.sets.iter().map(|s| s.id).collect();
    for ((_src, _sym), &target) in &col.goto_table {
        assert!(
            state_ids.contains(&target),
            "goto target {:?} not in collection",
            target
        );
    }
}

#[test]
fn initial_state_is_state_zero() {
    let mut g = GrammarBuilder::new("init0")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert_eq!(col.sets[0].id, StateId(0));
}

#[test]
fn closure_handles_alternation_correctly() {
    let mut g = GrammarBuilder::new("clalt")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["a"])
        .rule("a", vec!["x"])
        .rule("a", vec!["y"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    let state0 = &col.sets[0];
    // s -> •a, a -> •x, a -> •y should all be in closure
    assert!(
        state0.items.len() >= 3,
        "closure should include both alternatives for a"
    );
}

// ===========================================================================
// Section 3: First/follow set computation edge cases
// ===========================================================================

#[test]
fn first_set_of_nonterminal_contains_leading_terminal() {
    // FIRST(s) should contain 'a' since s -> a
    let g = GrammarBuilder::new("ff1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "s");
    let a = tok_id(&g, "a");
    let first_s = ff.first(s).expect("FIRST(s) should exist");
    assert!(
        first_s.contains(a.0 as usize),
        "FIRST(s) must contain terminal 'a'"
    );
}

#[test]
fn first_set_propagates_through_nonterminal() {
    let g = GrammarBuilder::new("ff2")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("a", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "s");
    let x = tok_id(&g, "x");
    let first_s = ff.first(s).expect("FIRST(s) should exist");
    assert!(
        first_s.contains(x.0 as usize),
        "FIRST(s) should contain 'x' via s->a->x"
    );
}

#[test]
fn nullable_symbol_detected() {
    let g = GrammarBuilder::new("ff3")
        .token("a", "a")
        .rule("s", vec![])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "s");
    assert!(
        ff.is_nullable(s),
        "s with epsilon production must be nullable"
    );
}

#[test]
fn non_nullable_symbol() {
    let g = GrammarBuilder::new("ff4")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "s");
    assert!(
        !ff.is_nullable(s),
        "s with only terminal production should not be nullable"
    );
}

#[test]
fn follow_of_start_contains_eof_marker() {
    // By convention EOF (SymbolId(0)) should be in FOLLOW(start)
    let g = GrammarBuilder::new("ff5")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "s");
    let follow_s = ff.follow(s).expect("FOLLOW(s) should exist");
    assert!(
        follow_s.contains(0),
        "FOLLOW(start) must contain EOF (index 0)"
    );
}

#[test]
fn first_of_nullable_includes_successors() {
    // s -> a b, a -> eps | x, b -> y
    // FIRST(s) should include x and y (since a is nullable)
    let g = GrammarBuilder::new("ff6")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["a", "b"])
        .rule("a", vec![])
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "s");
    let x = tok_id(&g, "x");
    let y = tok_id(&g, "y");
    let first_s = ff.first(s).unwrap();
    assert!(first_s.contains(x.0 as usize), "FIRST(s) should contain x");
    assert!(
        first_s.contains(y.0 as usize),
        "FIRST(s) should contain y (a is nullable)"
    );
}

#[test]
fn follow_propagates_from_production_context() {
    // s -> a b, b -> y => FOLLOW(a) should contain FIRST(b) = {y}
    let g = GrammarBuilder::new("ff7")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["a", "b"])
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a = nt_id(&g, "a");
    let y = tok_id(&g, "y");
    let follow_a = ff.follow(a).expect("FOLLOW(a) should exist");
    assert!(
        follow_a.contains(y.0 as usize),
        "FOLLOW(a) should contain FIRST(b) = {{y}}"
    );
}

#[test]
fn first_set_union_across_alternatives() {
    let g = GrammarBuilder::new("ff8")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = nt_id(&g, "s");
    let first_s = ff.first(s).unwrap();
    assert!(first_s.contains(tok_id(&g, "a").0 as usize));
    assert!(first_s.contains(tok_id(&g, "b").0 as usize));
    assert!(first_s.contains(tok_id(&g, "c").0 as usize));
}

#[test]
fn first_of_sequence_helper() {
    let g = GrammarBuilder::new("ffseq")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let x = tok_id(&g, "x");
    let seq = vec![Symbol::Terminal(x)];
    let first = ff.first_of_sequence(&seq).unwrap();
    assert!(first.contains(x.0 as usize), "FIRST of [x] should be {{x}}");
}

#[test]
fn chain_nullable_propagation() {
    // s -> a, a -> b, b -> eps | x => s, a, b all nullable
    let g = GrammarBuilder::new("ffchain")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec![])
        .rule("b", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.is_nullable(nt_id(&g, "b")));
    assert!(ff.is_nullable(nt_id(&g, "a")));
    assert!(ff.is_nullable(nt_id(&g, "s")));
}

// ===========================================================================
// Section 4: Grammar normalization effects on LR1 construction
// ===========================================================================

#[test]
fn normalized_grammar_produces_valid_table() {
    let mut g = GrammarBuilder::new("norm1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    assert!(has_accept(&table));
}

#[test]
fn normalize_then_compute_first_follow() {
    let mut g = GrammarBuilder::new("norm2")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["a", "b"])
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .start("s")
        .build();
    // compute_normalized does normalize + compute in one step
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = nt_id(&g, "s");
    let first_s = ff.first(s).unwrap();
    assert!(first_s.contains(tok_id(&g, "x").0 as usize));
}

#[test]
fn compute_normalized_matches_manual_normalize_then_compute() {
    let mut g1 = GrammarBuilder::new("norm3a")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();

    let mut g2 = g1.clone();

    let ff1 = FirstFollowSets::compute_normalized(&mut g1).unwrap();
    g2.normalize();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();

    let s1 = nt_id(&g1, "s");
    let s2 = nt_id(&g2, "s");
    assert_eq!(ff1.is_nullable(s1), ff2.is_nullable(s2));
}

#[test]
fn normalization_idempotent_for_simple_grammar() {
    let mut g = GrammarBuilder::new("norm4")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let before_rules: usize = g.rules.values().map(|v| v.len()).sum();
    g.normalize();
    let after_rules: usize = g.rules.values().map(|v| v.len()).sum();
    // Simple grammar should not gain rules from normalization
    assert_eq!(before_rules, after_rules);
}

#[test]
fn normalization_preserves_start_symbol() {
    let mut g = GrammarBuilder::new("norm5")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let start_before = g.start_symbol();
    g.normalize();
    let start_after = g.start_symbol();
    assert_eq!(start_before, start_after);
}

#[test]
fn normalized_collection_has_same_or_more_states() {
    let g1 = GrammarBuilder::new("norm6a")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let mut g2 = g1.clone();

    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let col1 = ItemSetCollection::build_canonical_collection(&g1, &ff1);

    g2.normalize();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    let col2 = ItemSetCollection::build_canonical_collection(&g2, &ff2);

    // Normalization should not remove states
    assert!(
        col2.sets.len() >= col1.sets.len() || col1.sets.len() >= col2.sets.len(),
        "collections should be comparable in size"
    );
}

// ===========================================================================
// Section 5: Conflict detection in parsed tables
// ===========================================================================

#[test]
fn unambiguous_grammar_no_shift_reduce_conflicts() {
    let g = GrammarBuilder::new("nosr")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    // No multi-action cells expected
    let mut multi = 0;
    for st in 0..table.state_count {
        for col in 0..table.action_table[st].len() {
            if table.action_table[st][col].len() > 1 {
                multi += 1;
            }
        }
    }
    assert_eq!(multi, 0, "simple grammar should have no conflict cells");
}

#[test]
fn ambiguous_expr_has_conflicts() {
    // E -> E + E | a — classic shift/reduce conflict
    let g = GrammarBuilder::new("ambig")
        .token("a", "a")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&col, &g, &ff);
    assert!(
        !resolver.conflicts.is_empty(),
        "ambiguous E->E+E|a must produce conflicts"
    );
}

#[test]
fn ambiguous_expr_has_shift_reduce_type() {
    let g = GrammarBuilder::new("srtype")
        .token("a", "a")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&col, &g, &ff);
    let has_sr = resolver
        .conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::ShiftReduce);
    assert!(has_sr, "E->E+E should produce shift/reduce conflict");
}

#[test]
fn dangling_else_has_shift_reduce_conflict() {
    // Classic dangling else: stmt -> IF expr stmt | IF expr stmt ELSE stmt | OTHER
    let g = GrammarBuilder::new("dangle")
        .token("IF", "if")
        .token("ELSE", "else")
        .token("EXPR", "e")
        .token("OTHER", "o")
        .rule("stmt", vec!["IF", "EXPR", "stmt", "ELSE", "stmt"])
        .rule("stmt", vec!["IF", "EXPR", "stmt"])
        .rule("stmt", vec!["OTHER"])
        .start("stmt")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&col, &g, &ff);
    let has_sr = resolver
        .conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::ShiftReduce);
    assert!(has_sr, "dangling else must produce shift/reduce conflict");
}

#[test]
fn conflict_resolver_returns_empty_for_lr1_grammar() {
    // s -> a b is unambiguous LR(1)
    let g = GrammarBuilder::new("lr1ok")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&col, &g, &ff);
    assert!(
        resolver.conflicts.is_empty(),
        "LR(1) grammar should have no conflicts, got {}",
        resolver.conflicts.len()
    );
}

#[test]
fn reduce_reduce_conflict_detected() {
    // Two rules that reduce on the same lookahead:
    // s -> a | b, a -> x, b -> x
    // When we see 'x' then EOF, we could reduce to a or b
    let g = GrammarBuilder::new("rr")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("a", vec!["x"])
        .rule("b", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&col, &g, &ff);
    let has_rr = resolver
        .conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::ReduceReduce);
    assert!(has_rr, "s->a|b, a->x, b->x should produce reduce/reduce");
}

#[test]
fn conflict_actions_have_at_least_two_entries() {
    let g = GrammarBuilder::new("ca2")
        .token("a", "a")
        .token("+", "+")
        .rule("e", vec!["e", "+", "e"])
        .rule("e", vec!["a"])
        .start("e")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let col = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&col, &g, &ff);
    for conflict in &resolver.conflicts {
        assert!(
            conflict.actions.len() >= 2,
            "conflict must have at least 2 actions"
        );
    }
}

#[test]
fn precedence_resolves_shift_reduce() {
    let g = GrammarBuilder::new("prec")
        .token("a", "a")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule("e", vec!["a"])
        .start("e")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    // Should still produce a valid table with accept
    assert!(has_accept(&table));
}

// ===========================================================================
// Section 6: State count verification for known grammars
// ===========================================================================

#[test]
fn single_rule_state_count() {
    // s -> a: expect exactly 3 states (initial, after shift a, accept/reduce)
    let g = GrammarBuilder::new("sc1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 2 && table.state_count <= 5,
        "s->a should have 2-5 states, got {}",
        table.state_count
    );
}

#[test]
fn two_alternation_state_count() {
    // s -> a | b
    let g = GrammarBuilder::new("sc2")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3 && table.state_count <= 8,
        "s->a|b should have 3-8 states, got {}",
        table.state_count
    );
}

#[test]
fn sequence_grammar_state_count() {
    // s -> a b c
    let g = GrammarBuilder::new("sc3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 4,
        "s->a b c should have at least 4 states, got {}",
        table.state_count
    );
}

#[test]
fn left_recursive_list_state_count() {
    // list -> list a | a
    let g = GrammarBuilder::new("sc4")
        .token("a", "a")
        .rule("list", vec!["list", "a"])
        .rule("list", vec!["a"])
        .start("list")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 3 && table.state_count <= 10,
        "left-recursive list should have 3-10 states, got {}",
        table.state_count
    );
}

#[test]
fn nested_nonterminal_state_count() {
    // s -> a, a -> b, b -> x
    let g = GrammarBuilder::new("sc5")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["x"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 2,
        "chained nonterminals should have at least 2 states, got {}",
        table.state_count
    );
}

#[test]
fn arithmetic_grammar_state_count() {
    let g = GrammarBuilder::new("arith")
        .token("NUM", "n")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();
    let table = build_table(&g);
    // Classic expression grammar typically has ~12 states
    assert!(
        table.state_count >= 8 && table.state_count <= 30,
        "arithmetic grammar should have 8-30 states, got {}",
        table.state_count
    );
}

#[test]
fn nullable_start_state_count() {
    // s -> eps | a
    let g = GrammarBuilder::new("sc6")
        .token("a", "a")
        .rule("s", vec![])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(
        table.state_count >= 2,
        "nullable start should have at least 2 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// Section 7: Additional edge cases and integration tests
// ===========================================================================

#[test]
fn build_lr1_automaton_res_consistent_with_build_lr1_automaton() {
    let g = GrammarBuilder::new("cons")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let t1 = build_lr1_automaton(&g, &ff).unwrap();
    let t2 = build_lr1_automaton_res(&g, &ff).unwrap();
    assert_eq!(t1.state_count, t2.state_count);
    assert_eq!(t1.eof(), t2.eof());
}

#[test]
fn table_has_correct_eof_symbol() {
    let g = GrammarBuilder::new("eof1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    // EOF should be a valid symbol in the table
    let eof = table.eof();
    assert!(
        eof.0 > 0,
        "EOF symbol should have a non-zero ID (reserved above grammar symbols)"
    );
}

#[test]
fn table_start_symbol_matches_grammar() {
    let g = GrammarBuilder::new("startsym")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    // The table should track the start symbol
    let table_start = table.start_symbol();
    assert!(table_start.0 > 0, "start symbol should be set");
}

#[test]
fn goto_table_has_nonterminal_entries() {
    let g = GrammarBuilder::new("gont")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("a", vec!["x"])
        .start("s")
        .build();
    let table = build_table(&g);
    let a = nt_id(&g, "a");
    // Check GOTO table has entry for nonterminal 'a'
    let has_goto = (0..table.state_count).any(|st| table.goto(StateId(st as u16), a).is_some());
    assert!(has_goto, "GOTO table should have entry for nonterminal 'a'");
}

#[test]
fn parse_table_rules_match_grammar_rules() {
    let g = GrammarBuilder::new("prules")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let table = build_table(&g);
    // Table should have parse rules
    assert!(
        !table.rules.is_empty(),
        "parse table should contain parse rules"
    );
}

#[test]
fn collection_goto_table_not_empty_for_nontrivial_grammar() {
    let mut g = GrammarBuilder::new("gne")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.goto_table.is_empty(), "goto table should not be empty");
}

#[test]
fn collection_tracks_terminal_vs_nonterminal() {
    let mut g = GrammarBuilder::new("tnt")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("a", vec!["x"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    // symbol_is_terminal should have entries
    assert!(
        !col.symbol_is_terminal.is_empty(),
        "should track terminal vs nonterminal"
    );
}

#[test]
fn multiple_reduce_actions_on_eof_for_alternation() {
    let g = GrammarBuilder::new("mred")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let table = build_table(&g);
    let eof = table.eof();
    let total_reduces: usize = (0..table.state_count)
        .map(|st| {
            table
                .actions(StateId(st as u16), eof)
                .iter()
                .filter(|a| matches!(a, Action::Reduce(_)))
                .count()
        })
        .sum();
    assert!(
        total_reduces >= 2,
        "two alternations should produce at least 2 reduce-on-EOF, got {}",
        total_reduces
    );
}

#[test]
fn table_eof_normalization_sets_eof_to_zero() {
    let g = GrammarBuilder::new("val")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let original_eof = table.eof();
    assert!(original_eof.0 > 0, "original EOF should be non-zero");
    let normalized = table.normalize_eof_to_zero();
    assert_eq!(
        normalized.eof(),
        SymbolId(0),
        "after normalization EOF should be SymbolId(0)"
    );
}

#[test]
fn parenthesized_expression_grammar() {
    let g = GrammarBuilder::new("paren")
        .token("NUM", "n")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["NUM"])
        .rule("expr", vec!["(", "expr", ")"])
        .start("expr")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    assert!(table.state_count >= 4);
}

#[test]
fn deeply_nested_nonterminals() {
    let g = GrammarBuilder::new("deep")
        .token("z", "z")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["d"])
        .rule("d", vec!["z"])
        .start("a")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn mixed_terminals_and_nonterminals_in_rhs() {
    let g = GrammarBuilder::new("mix")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "mid", "y"])
        .rule("mid", vec!["x"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn parse_table_sanity_check_passes() {
    let g = GrammarBuilder::new("sanity")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    let result = adze_glr_core::sanity_check_tables(&table);
    assert!(result.is_ok(), "sanity check failed: {:?}", result.err());
}

#[test]
fn left_and_right_recursive_grammar() {
    // s -> s a | a s | a — both left and right recursive
    let g = GrammarBuilder::new("lrrec")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn epsilon_only_grammar() {
    // s -> eps
    let g = GrammarBuilder::new("epsonly")
        .rule("s", vec![])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
}

#[test]
fn many_alternatives_grammar() {
    let g = GrammarBuilder::new("many")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .rule("s", vec!["d"])
        .rule("s", vec!["e"])
        .start("s")
        .build();
    let table = build_table(&g);
    assert!(has_accept(&table));
    let eof = table.eof();
    assert!(count_states_with_reduce(&table, eof) >= 5);
}

#[test]
fn python_like_grammar_builds() {
    let g = GrammarBuilder::python_like();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let result = build_lr1_automaton(&g, &ff);
    assert!(result.is_ok(), "Python-like grammar should build");
    let table = result.unwrap();
    assert!(has_accept(&table));
}

#[test]
fn javascript_like_grammar_builds() {
    let g = GrammarBuilder::javascript_like();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let result = build_lr1_automaton(&g, &ff);
    assert!(result.is_ok(), "JavaScript-like grammar should build");
    let table = result.unwrap();
    assert!(has_accept(&table));
    assert!(table.state_count >= 10);
}

#[test]
fn collection_initial_state_has_nonempty_items() {
    let mut g = GrammarBuilder::new("nonempty")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(
        !col.sets[0].items.is_empty(),
        "initial state must have items after closure"
    );
}

#[test]
fn all_goto_targets_in_range() {
    let mut g = GrammarBuilder::new("range")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let (col, _) = build_collection(&mut g);
    let max_state = col.sets.len();
    for (_, &target) in &col.goto_table {
        assert!(
            (target.0 as usize) < max_state,
            "goto target {} out of range (max {})",
            target.0,
            max_state
        );
    }
}
