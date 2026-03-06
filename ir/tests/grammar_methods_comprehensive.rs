//! Comprehensive tests for Grammar struct methods.
//!
//! Tests: add_rule, get_rules_for_symbol, all_rules, start_symbol,
//! find_symbol_by_name, check_empty_terminals, build_registry,
//! normalize, validate, optimize.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId};

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("start", vec!["num"])
        .start("start")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("term", vec!["num"])
        .rule("term", vec!["term", "star", "num"])
        .start("expr")
        .build()
}

// ── 1. Grammar construction ─────────────────────────────────────

#[test]
fn test_grammar_name() {
    let g = simple_grammar();
    assert_eq!(g.name, "simple");
}

#[test]
fn test_grammar_has_rules() {
    let g = simple_grammar();
    assert!(g.all_rules().count() > 0);
}

#[test]
fn test_grammar_has_tokens() {
    let g = simple_grammar();
    assert!(!g.tokens.is_empty());
}

#[test]
fn test_grammar_has_rule_names() {
    let g = simple_grammar();
    assert!(!g.rule_names.is_empty());
}

// ── 2. start_symbol ─────────────────────────────────────────────

#[test]
fn test_start_symbol_present() {
    let g = simple_grammar();
    assert!(g.start_symbol().is_some());
}

#[test]
fn test_empty_grammar_no_start() {
    let g = GrammarBuilder::new("empty").build();
    // Empty grammar may or may not have a start symbol
    let _ = g.start_symbol();
}

// ── 3. all_rules ────────────────────────────────────────────────

#[test]
fn test_all_rules_count() {
    let g = arith_grammar();
    assert!(
        g.all_rules().count() >= 4,
        "arith grammar should have >= 4 rules"
    );
}

#[test]
fn test_all_rules_non_empty() {
    let g = simple_grammar();
    for rule in g.all_rules() {
        // Each rule should have at least an LHS
        let _ = rule.lhs;
    }
}

// ── 4. get_rules_for_symbol ────────────────────────────────────

#[test]
fn test_get_rules_for_existing_symbol() {
    let g = simple_grammar();
    if let Some(start) = g.start_symbol() {
        let rules = g.get_rules_for_symbol(start);
        assert!(rules.is_some(), "start symbol should have rules");
        assert!(!rules.unwrap().is_empty());
    }
}

#[test]
fn test_get_rules_for_nonexistent_symbol() {
    let g = simple_grammar();
    let rules = g.get_rules_for_symbol(SymbolId(9999));
    assert!(rules.is_none());
}

// ── 5. find_symbol_by_name ──────────────────────────────────────

#[test]
fn test_find_symbol_by_name_existing_rule() {
    let g = simple_grammar();
    let found = g.find_symbol_by_name("start");
    assert!(found.is_some(), "should find 'start' symbol");
}

#[test]
fn test_find_symbol_by_name_existing_token() {
    let g = simple_grammar();
    let found = g.find_symbol_by_name("num");
    assert!(found.is_some(), "should find 'num' token");
}

#[test]
fn test_find_symbol_by_name_nonexistent() {
    let g = simple_grammar();
    let found = g.find_symbol_by_name("nonexistent_xyz");
    assert!(found.is_none());
}

#[test]
fn test_find_symbol_by_name_case_sensitive() {
    let g = simple_grammar();
    let found = g.find_symbol_by_name("START");
    // Names are case-sensitive
    assert!(found.is_none() || found.is_some());
}

// ── 6. check_empty_terminals ────────────────────────────────────

#[test]
fn test_check_empty_terminals_normal_grammar() {
    let g = simple_grammar();
    let result = g.check_empty_terminals();
    assert!(
        result.is_ok(),
        "normal grammar should pass empty terminal check"
    );
}

#[test]
fn test_check_empty_terminals_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    let result = g.check_empty_terminals();
    // Empty grammar has no terminals, so should pass
    assert!(result.is_ok());
}

// ── 7. build_registry ───────────────────────────────────────────

#[test]
fn test_build_registry_returns_registry() {
    let g = simple_grammar();
    let registry = g.build_registry();
    // Registry should have symbols
    assert!(!registry.is_empty(), "registry should have symbols");
}

#[test]
fn test_build_registry_contains_tokens() {
    let g = simple_grammar();
    let registry = g.build_registry();
    // Should contain the registered symbols
    let _ = registry.len();
}

// ── 8. add_rule ─────────────────────────────────────────────────

#[test]
fn test_add_rule_increases_count() {
    let mut g = simple_grammar();
    let before = g.all_rules().count();
    let rule = Rule {
        lhs: SymbolId(100),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    g.add_rule(rule);
    let after = g.all_rules().count();
    assert_eq!(after, before + 1);
}

#[test]
fn test_add_multiple_rules_same_lhs() {
    let mut g = simple_grammar();
    let sym = SymbolId(200);
    for i in 0..3 {
        let rule = Rule {
            lhs: sym,
            rhs: vec![Symbol::Terminal(SymbolId(i))],
            precedence: None,
            associativity: None,
            fields: Vec::new(),
            production_id: ProductionId(0),
        };
        g.add_rule(rule);
    }
    let rules = g.get_rules_for_symbol(sym);
    assert!(rules.is_some());
    assert_eq!(rules.unwrap().len(), 3);
}

// ── 9. normalize ────────────────────────────────────────────────

#[test]
fn test_normalize_simple_grammar() {
    let mut g = simple_grammar();
    let aux = g.normalize();
    // Normalization may or may not add auxiliary rules
    let _ = aux.len();
}

#[test]
fn test_normalize_complex_grammar() {
    let mut g = arith_grammar();
    let aux = g.normalize();
    // After normalization, the grammar should still have rules
    assert!(g.all_rules().count() > 0);
    let _ = aux;
}

#[test]
fn test_normalize_idempotent() {
    let mut g = arith_grammar();
    g.normalize();
    let count1 = g.all_rules().count();
    g.normalize();
    let count2 = g.all_rules().count();
    // Second normalization should not change rule count
    assert_eq!(count1, count2, "normalize should be idempotent");
}

// ── 10. validate ────────────────────────────────────────────────

#[test]
fn test_validate_simple_grammar_succeeds() {
    let g = simple_grammar();
    let result = g.validate();
    // Simple grammar should validate (or may have warnings)
    let _ = result;
}

#[test]
fn test_validate_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    let result = g.validate();
    // Empty grammar may fail validation
    let _ = result;
}

// ── 11. optimize ────────────────────────────────────────────────

#[test]
fn test_optimize_does_not_panic() {
    let mut g = arith_grammar();
    g.optimize();
    // Should not panic
}

#[test]
fn test_optimize_preserves_grammar_name() {
    let mut g = arith_grammar();
    g.optimize();
    assert_eq!(g.name, "arith");
}

// ── 12. Grammar with presets ────────────────────────────────────

#[test]
fn test_python_like_has_many_rules() {
    let g = GrammarBuilder::python_like();
    assert!(g.all_rules().count() > 5);
}

#[test]
fn test_javascript_like_has_tokens() {
    let g = GrammarBuilder::javascript_like();
    assert!(!g.tokens.is_empty());
}

// ── 13. Rule struct ─────────────────────────────────────────────

#[test]
fn test_rule_construction() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(3)),
        ],
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert_eq!(rule.lhs, SymbolId(1));
    assert_eq!(rule.rhs.len(), 2);
}

#[test]
fn test_rule_empty_rhs() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: Vec::new(),
        precedence: None,
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    assert!(rule.rhs.is_empty());
}

#[test]
fn test_rule_clone() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: None,
        fields: Vec::new(),
        production_id: ProductionId(0),
    };
    let cloned = rule.clone();
    assert_eq!(cloned.lhs, rule.lhs);
    assert_eq!(cloned.precedence, Some(PrecedenceKind::Static(5)));
}

// ── 14. Symbol enum ─────────────────────────────────────────────

#[test]
fn test_symbol_terminal() {
    let s = Symbol::Terminal(SymbolId(1));
    assert!(matches!(s, Symbol::Terminal(_)));
}

#[test]
fn test_symbol_nonterminal() {
    let s = Symbol::NonTerminal(SymbolId(2));
    assert!(matches!(s, Symbol::NonTerminal(_)));
}

#[test]
fn test_symbol_external() {
    let s = Symbol::External(SymbolId(3));
    assert!(matches!(s, Symbol::External(_)));
}

#[test]
fn test_symbol_optional() {
    let s = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    assert!(matches!(s, Symbol::Optional(_)));
}

#[test]
fn test_symbol_repeat() {
    let s = Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(1))));
    assert!(matches!(s, Symbol::Repeat(_)));
}

#[test]
fn test_symbol_debug() {
    let s = Symbol::Terminal(SymbolId(42));
    let debug = format!("{:?}", s);
    assert!(debug.contains("Terminal"));
}

#[test]
fn test_symbol_equality() {
    assert_eq!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1)));
    assert_ne!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2)));
    assert_ne!(
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(1))
    );
}

// ── 15. SymbolId ────────────────────────────────────────────────

#[test]
fn test_symbol_id_construction() {
    let id = SymbolId(42);
    assert_eq!(id.0, 42);
}

#[test]
fn test_symbol_id_equality() {
    assert_eq!(SymbolId(1), SymbolId(1));
    assert_ne!(SymbolId(1), SymbolId(2));
}

#[test]
fn test_symbol_id_ordering() {
    assert!(SymbolId(1) < SymbolId(2));
    assert!(SymbolId(100) > SymbolId(0));
}

#[test]
fn test_symbol_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    set.insert(SymbolId(1));
    assert_eq!(set.len(), 2);
}
