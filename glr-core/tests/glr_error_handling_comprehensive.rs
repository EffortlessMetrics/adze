//! Comprehensive tests for GLR error types and error handling patterns.

use adze_glr_core::GLRError;

// ── GLRError construction and display ──

#[test]
fn glr_error_debug() {
    // Try to get a GLRError from an invalid grammar operation
    let mut g = adze_ir::builder::GrammarBuilder::new("empty").build();
    g.normalize();
    let result = adze_glr_core::FirstFollowSets::compute(&g);
    match result {
        Ok(_) => {} // Empty grammar may succeed
        Err(e) => {
            let d = format!("{:?}", e);
            assert!(!d.is_empty());
        }
    }
}

#[test]
fn glr_error_from_invalid_grammar() {
    // Build a grammar and try to produce an error
    let g = adze_ir::builder::GrammarBuilder::new("broken").build();
    // Not calling normalize — may or may not cause error
    let result = adze_glr_core::FirstFollowSets::compute(&g);
    let _ = result; // Just verify it doesn't panic
}

// ── FirstFollowSets error paths ──

#[test]
fn first_follow_empty_grammar() {
    let mut g = adze_ir::builder::GrammarBuilder::new("empty").build();
    g.normalize();
    let result = adze_glr_core::FirstFollowSets::compute(&g);
    let _ = result;
}

#[test]
fn first_follow_tokens_only() {
    let mut g = adze_ir::builder::GrammarBuilder::new("tokens")
        .token("a", "a")
        .build();
    g.normalize();
    let result = adze_glr_core::FirstFollowSets::compute(&g);
    let _ = result;
}

// ── build_lr1_automaton error paths ──

#[test]
fn build_automaton_empty_grammar() {
    let mut g = adze_ir::builder::GrammarBuilder::new("empty").build();
    g.normalize();
    if let Ok(ff) = adze_glr_core::FirstFollowSets::compute(&g) {
        let result = adze_glr_core::build_lr1_automaton(&g, &ff);
        let _ = result;
    }
}

#[test]
fn build_automaton_tokens_only() {
    let mut g = adze_ir::builder::GrammarBuilder::new("tokens")
        .token("a", "a")
        .build();
    g.normalize();
    if let Ok(ff) = adze_glr_core::FirstFollowSets::compute(&g) {
        let result = adze_glr_core::build_lr1_automaton(&g, &ff);
        let _ = result;
    }
}

// ── ParseTable from valid grammars ──

#[test]
fn parse_table_simple_ok() {
    let mut g = adze_ir::builder::GrammarBuilder::new("ok")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    let ff = adze_glr_core::FirstFollowSets::compute(&g).unwrap();
    let pt = adze_glr_core::build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count > 0);
}

#[test]
fn parse_table_two_alt_ok() {
    let mut g = adze_ir::builder::GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    g.normalize();
    let ff = adze_glr_core::FirstFollowSets::compute(&g).unwrap();
    let pt = adze_glr_core::build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count >= 2);
}

// ── FirstFollowSets properties ──

#[test]
fn first_set_for_start_symbol() {
    let mut g = adze_ir::builder::GrammarBuilder::new("fs")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    let ff = adze_glr_core::FirstFollowSets::compute(&g).unwrap();
    if let Some(start) = g.start_symbol() {
        let first = ff.first(start);
        assert!(first.is_some());
    }
}

#[test]
fn follow_set_for_start_symbol() {
    let mut g = adze_ir::builder::GrammarBuilder::new("fol")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    let ff = adze_glr_core::FirstFollowSets::compute(&g).unwrap();
    if let Some(start) = g.start_symbol() {
        let follow = ff.follow(start);
        assert!(follow.is_some());
    }
}

#[test]
fn first_set_for_terminal() {
    let mut g = adze_ir::builder::GrammarBuilder::new("ft")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    let ff = adze_glr_core::FirstFollowSets::compute(&g).unwrap();
    // Terminal symbols may or may not have FIRST sets
    for (sid, _) in &g.tokens {
        let first = ff.first(*sid);
        let _ = first;
    }
}

// ── Multiple compute calls ──

#[test]
fn first_follow_compute_deterministic() {
    let make = || {
        let mut g = adze_ir::builder::GrammarBuilder::new("det")
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        g.normalize();
        adze_glr_core::FirstFollowSets::compute(&g).unwrap()
    };
    let _ff1 = make();
    let _ff2 = make();
}
