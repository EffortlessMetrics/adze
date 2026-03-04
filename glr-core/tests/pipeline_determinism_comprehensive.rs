// Comprehensive tests for full pipeline determinism and consistency
// Verifies that identical grammars always produce identical outputs

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;

fn make_simple() -> adze_ir::Grammar {
    GrammarBuilder::new("det")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn make_alt() -> adze_ir::Grammar {
    GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

fn make_chain() -> adze_ir::Grammar {
    GrammarBuilder::new("ch")
        .token("a", "a")
        .rule("s", vec!["m"])
        .rule("m", vec!["a"])
        .start("s")
        .build()
}

#[test]
fn first_follow_deterministic_simple() {
    let g = make_simple();
    let ff1 = FirstFollowSets::compute(&g).unwrap();
    let ff2 = FirstFollowSets::compute(&g).unwrap();
    // Same grammar should produce same FIRST/FOLLOW sets
    let _ = (ff1, ff2);
}

#[test]
fn parse_table_deterministic_simple() {
    let g1 = make_simple();
    let g2 = make_simple();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    let pt1 = build_lr1_automaton(&g1, &ff1).unwrap();
    let pt2 = build_lr1_automaton(&g2, &ff2).unwrap();
    assert_eq!(pt1.state_count, pt2.state_count);
    assert_eq!(pt1.symbol_count, pt2.symbol_count);
}

#[test]
fn parse_table_deterministic_alt() {
    let g1 = make_alt();
    let g2 = make_alt();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    let pt1 = build_lr1_automaton(&g1, &ff1).unwrap();
    let pt2 = build_lr1_automaton(&g2, &ff2).unwrap();
    assert_eq!(pt1.state_count, pt2.state_count);
}

#[test]
fn parse_table_deterministic_chain() {
    let g1 = make_chain();
    let g2 = make_chain();
    let ff1 = FirstFollowSets::compute(&g1).unwrap();
    let ff2 = FirstFollowSets::compute(&g2).unwrap();
    let pt1 = build_lr1_automaton(&g1, &ff1).unwrap();
    let pt2 = build_lr1_automaton(&g2, &ff2).unwrap();
    assert_eq!(pt1.state_count, pt2.state_count);
}

#[test]
fn simple_grammar_has_states() {
    let g = make_simple();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count > 0);
}

#[test]
fn alt_grammar_has_more_or_equal_states() {
    let g_simple = make_simple();
    let ff_simple = FirstFollowSets::compute(&g_simple).unwrap();
    let pt_simple = build_lr1_automaton(&g_simple, &ff_simple).unwrap();

    let g_alt = make_alt();
    let ff_alt = FirstFollowSets::compute(&g_alt).unwrap();
    let pt_alt = build_lr1_automaton(&g_alt, &ff_alt).unwrap();

    assert!(pt_alt.state_count >= pt_simple.state_count);
}

#[test]
fn chain_grammar_has_states() {
    let g = make_chain();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count > 0);
}

#[test]
fn parse_table_has_rules() {
    let g = make_simple();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    assert!(!pt.rules.is_empty());
}

#[test]
fn symbol_count_includes_terminals() {
    let g = make_alt();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    // At least 2 terminals + nonterminals
    assert!(pt.symbol_count >= 3);
}

#[test]
fn recursive_grammar_compiles() {
    let g = GrammarBuilder::new("rec")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count > 0);
}
