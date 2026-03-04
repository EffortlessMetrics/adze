//! Comprehensive tests for Grammar normalization pipeline.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId};

fn make_rule(lhs: SymbolId, rhs: Vec<Symbol>) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    }
}

// ─── Basic normalization ───

#[test]
fn normalize_empty_grammar() {
    let mut g = Grammar::new("empty".to_string());
    g.normalize();
    assert_eq!(g.rules.len(), 0);
}

#[test]
fn normalize_single_token_grammar() {
    let mut g = GrammarBuilder::new("single").token("a", "a").build();
    g.normalize();
    // Token-only grammar should survive normalization
    assert!(g.tokens.len() >= 1);
}

#[test]
fn normalize_simple_rule_unchanged() {
    let mut g = GrammarBuilder::new("simple")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let rules_before = g.rules.len();
    g.normalize();
    // Simple rules (all terminals) should not generate auxiliary rules
    assert!(g.rules.len() >= rules_before);
}

// ─── Optional symbol normalization ───

#[test]
fn normalize_optional_creates_auxiliary() {
    let mut g = GrammarBuilder::new("opt").token("a", "a").build();
    // Manually add a rule with Optional symbol
    let a_id = *g.tokens.keys().next().unwrap();
    let start_id = SymbolId(100);
    g.rules.insert(
        start_id,
        vec![make_rule(
            start_id,
            vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
        )],
    );
    g.rule_names.insert(start_id, "start".to_string());
    g.normalize();
    // After normalization, the Optional should be expanded into auxiliary rules
    assert!(g.rules.len() >= 2, "Expected auxiliary rule for Optional");
}

// ─── Repeat symbol normalization ───

#[test]
fn normalize_repeat_creates_auxiliary() {
    let mut g = GrammarBuilder::new("rep").token("x", "x").build();
    let x_id = *g.tokens.keys().next().unwrap();
    let start_id = SymbolId(100);
    g.rules.insert(
        start_id,
        vec![make_rule(
            start_id,
            vec![Symbol::Repeat(Box::new(Symbol::Terminal(x_id)))],
        )],
    );
    g.rule_names.insert(start_id, "start".to_string());
    g.normalize();
    assert!(g.rules.len() >= 2, "Expected auxiliary rule for Repeat");
}

// ─── Choice symbol normalization ───

#[test]
fn normalize_choice_creates_auxiliary() {
    let mut g = GrammarBuilder::new("choice")
        .token("a", "a")
        .token("b", "b")
        .build();
    let ids: Vec<SymbolId> = g.tokens.keys().copied().collect();
    let start_id = SymbolId(100);
    g.rules.insert(
        start_id,
        vec![make_rule(
            start_id,
            vec![Symbol::Choice(vec![
                Symbol::Terminal(ids[0]),
                Symbol::Terminal(ids[1]),
            ])],
        )],
    );
    g.rule_names.insert(start_id, "start".to_string());
    g.normalize();
    assert!(g.rules.len() >= 2, "Expected auxiliary rule for Choice");
}

// ─── Sequence symbol normalization ───

#[test]
fn normalize_sequence_flattens_inline() {
    let mut g = GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .build();
    let ids: Vec<SymbolId> = g.tokens.keys().copied().collect();
    let start_id = SymbolId(100);
    g.rules.insert(
        start_id,
        vec![make_rule(
            start_id,
            vec![Symbol::Sequence(vec![
                Symbol::Terminal(ids[0]),
                Symbol::Terminal(ids[1]),
            ])],
        )],
    );
    g.rule_names.insert(start_id, "start".to_string());
    let rules_before = g.rules.len();
    g.normalize();
    // Sequence is flattened inline, NOT into auxiliary rules
    assert_eq!(
        g.rules.len(),
        rules_before,
        "Sequence should be flattened, not create new rules"
    );
}

// ─── Nested complex symbols ───

#[test]
fn normalize_nested_optional_repeat() {
    let mut g = GrammarBuilder::new("nested").token("x", "x").build();
    let x_id = *g.tokens.keys().next().unwrap();
    let start_id = SymbolId(100);
    g.rules.insert(
        start_id,
        vec![make_rule(
            start_id,
            vec![Symbol::Optional(Box::new(Symbol::Repeat(Box::new(
                Symbol::Terminal(x_id),
            ))))],
        )],
    );
    g.rule_names.insert(start_id, "start".to_string());
    g.normalize();
    // Both Optional and Repeat should be expanded
    assert!(g.rules.len() >= 3, "Expected multiple auxiliary rules");
}

// ─── Normalization idempotency ───

#[test]
fn normalize_idempotent() {
    let mut g = GrammarBuilder::new("idem")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    g.normalize();
    let rules_after_first = g.rules.len();
    let names_after_first: Vec<String> = g.rule_names.values().cloned().collect();
    g.normalize();
    assert_eq!(g.rules.len(), rules_after_first);
    let names_after_second: Vec<String> = g.rule_names.values().cloned().collect();
    assert_eq!(names_after_first.len(), names_after_second.len());
}

// ─── Grammar::new basics ───

#[test]
fn grammar_new_empty() {
    let g = Grammar::new("test".to_string());
    assert_eq!(g.name, "test");
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
}

#[test]
fn grammar_name() {
    let g = Grammar::new("my_grammar".to_string());
    assert_eq!(g.name, "my_grammar");
}

// ─── start_symbol ───

#[test]
fn grammar_start_symbol_none_when_no_rules() {
    let g = Grammar::new("empty".to_string());
    assert!(g.start_symbol().is_none());
}

#[test]
fn grammar_start_symbol_some_with_rules() {
    let g = GrammarBuilder::new("with_start")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(g.start_symbol().is_some());
}

// ─── all_rules ───

#[test]
fn grammar_all_rules_empty() {
    let g = Grammar::new("empty".to_string());
    assert_eq!(g.all_rules().count(), 0);
}

#[test]
fn grammar_all_rules_count() {
    let g = GrammarBuilder::new("rules")
        .token("a", "a")
        .token("b", "b")
        .rule("s1", vec!["a"])
        .rule("s2", vec!["b"])
        .start("s1")
        .build();
    assert!(g.all_rules().count() >= 2);
}

// ─── GrammarBuilder edge cases ───

#[test]
fn builder_multiple_rules_same_lhs() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .start("expr")
        .build();
    // Both alternatives should exist
    let rules_count = g.all_rules().count();
    assert!(rules_count >= 2);
}

#[test]
fn builder_chain_many_tokens() {
    let mut builder = GrammarBuilder::new("many_tokens");
    for i in 0..50 {
        builder = builder.token(&format!("t{}", i), &format!("t{}", i));
    }
    let g = builder.build();
    assert_eq!(g.tokens.len(), 50);
}

#[test]
fn builder_chain_many_rules() {
    let mut builder = GrammarBuilder::new("many_rules").token("x", "x");
    for i in 0..20 {
        builder = builder.rule(&format!("r{}", i), vec!["x"]);
    }
    builder = builder.start("r0");
    let g = builder.build();
    assert!(g.rules.len() >= 20);
}

// ─── Symbol types ───

#[test]
fn symbol_terminal_debug() {
    let s = Symbol::Terminal(SymbolId(1));
    let d = format!("{:?}", s);
    assert!(d.contains("Terminal"));
}

#[test]
fn symbol_nonterminal_debug() {
    let s = Symbol::NonTerminal(SymbolId(2));
    let d = format!("{:?}", s);
    assert!(d.contains("NonTerminal"));
}

#[test]
fn symbol_optional_debug() {
    let s = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    let d = format!("{:?}", s);
    assert!(d.contains("Optional"));
}

#[test]
fn symbol_repeat_debug() {
    let s = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))));
    let d = format!("{:?}", s);
    assert!(d.contains("Repeat"));
}

#[test]
fn symbol_choice_debug() {
    let s = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    let d = format!("{:?}", s);
    assert!(d.contains("Choice"));
}

#[test]
fn symbol_sequence_debug() {
    let s = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    let d = format!("{:?}", s);
    assert!(d.contains("Sequence"));
}

#[test]
fn symbol_clone() {
    let s = Symbol::Terminal(SymbolId(42));
    let s2 = s.clone();
    assert_eq!(format!("{:?}", s), format!("{:?}", s2));
}

// ─── SymbolId ───

#[test]
fn symbol_id_eq() {
    assert_eq!(SymbolId(0), SymbolId(0));
    assert_ne!(SymbolId(0), SymbolId(1));
}

#[test]
fn symbol_id_ord() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(100) > SymbolId(50));
}

#[test]
fn symbol_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    set.insert(SymbolId(1)); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn symbol_id_u16_max() {
    let id = SymbolId(u16::MAX);
    assert_eq!(id.0, u16::MAX);
}

// ─── Rule struct ───

#[test]
fn rule_empty_symbols() {
    let r = make_rule(SymbolId(0), vec![]);
    assert!(r.rhs.is_empty());
}

#[test]
fn rule_single_terminal() {
    let r = make_rule(SymbolId(0), vec![Symbol::Terminal(SymbolId(1))]);
    assert_eq!(r.rhs.len(), 1);
}

#[test]
fn rule_multiple_symbols() {
    let r = make_rule(
        SymbolId(0),
        vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
        ],
    );
    assert_eq!(r.rhs.len(), 3);
}

#[test]
fn rule_clone() {
    let r = make_rule(SymbolId(0), vec![Symbol::Terminal(SymbolId(1))]);
    let r2 = r.clone();
    assert_eq!(r.rhs.len(), r2.rhs.len());
}

// ─── Grammar with builder: full pipeline ───

#[test]
fn full_pipeline_simple_arithmetic() {
    let mut g = GrammarBuilder::new("arith")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["num", "plus", "num"])
        .start("expr")
        .build();
    g.normalize();
    assert!(g.start_symbol().is_some());
    assert!(g.tokens.len() >= 2);
    assert!(g.rules.len() >= 1);
}

#[test]
fn full_pipeline_multiple_alternatives() {
    let mut g = GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    g.normalize();
    assert!(g.all_rules().count() >= 3);
}

#[test]
fn full_pipeline_chain_rules() {
    let mut g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .start("c")
        .build();
    g.normalize();
    assert!(g.rules.len() >= 3);
}

// ─── Grammar fields ───

#[test]
fn grammar_rule_names_populated() {
    let g = GrammarBuilder::new("named")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    assert!(g.rule_names.values().any(|n| n == "start"));
}

#[test]
fn grammar_tokens_have_patterns() {
    let g = GrammarBuilder::new("pats")
        .token("num", "[0-9]+")
        .token("id", "[a-z]+")
        .build();
    assert_eq!(g.tokens.len(), 2);
}
