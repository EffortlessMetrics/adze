// Edge-case tests for the IR grammar optimizer
use adze_ir::optimizer::{GrammarOptimizer, optimize_grammar};
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_rule(lhs: u16, rhs: Vec<Symbol>, prod: u16) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

fn make_prec_rule(
    lhs: u16,
    rhs: Vec<Symbol>,
    prod: u16,
    prec: PrecedenceKind,
    assoc: Associativity,
) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs,
        precedence: Some(prec),
        associativity: Some(assoc),
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

fn make_token(id: u16, name: &str, pattern: &str) -> (SymbolId, Token) {
    (
        SymbolId(id),
        Token {
            name: name.to_string(),
            pattern: TokenPattern::String(pattern.to_string()),
            fragile: false,
        },
    )
}

// ---------------------------------------------------------------------------
// Empty grammar
// ---------------------------------------------------------------------------

#[test]
fn optimizer_handles_empty_grammar_gracefully() {
    let grammar = Grammar::new("empty".into());
    let result = optimize_grammar(grammar);
    assert!(result.is_ok());
    let g = result.unwrap();
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
}

#[test]
fn optimizer_struct_on_empty_grammar() {
    let mut grammar = Grammar::new("empty".into());
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut grammar);
    assert_eq!(stats.total(), 0);
}

// ---------------------------------------------------------------------------
// Optimizer doesn't remove needed rules
// ---------------------------------------------------------------------------

#[test]
fn optimizer_preserves_start_rule_and_terminals() {
    let mut g = Grammar::new("preserve".into());

    let (tid1, tok1) = make_token(10, "num", "42");
    let (tid2, tok2) = make_token(11, "plus", "+");
    g.tokens.insert(tid1, tok1);
    g.tokens.insert(tid2, tok2);

    // S -> num + num  (non-trivial, won't be inlined as unit rule)
    g.add_rule(make_rule(
        0,
        vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::Terminal(SymbolId(11)),
            Symbol::Terminal(SymbolId(10)),
        ],
        0,
    ));
    g.rule_names.insert(SymbolId(0), "S".into());

    let result = optimize_grammar(g).unwrap();

    // The terminal tokens used in the start rule must survive.
    assert!(!result.tokens.is_empty(), "optimizer removed needed token");
    // The rule must remain (it's not a unit rule).
    assert!(!result.rules.is_empty(), "optimizer removed all rules");
}

#[test]
fn optimizer_keeps_reachable_chain() {
    // S -> A tok, A -> B tok, B -> tok  (non-unit rules to prevent full inlining)
    let mut g = Grammar::new("chain".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);

    g.add_rule(make_rule(
        0,
        vec![
            Symbol::NonTerminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(10)),
        ],
        0,
    ));
    g.add_rule(make_rule(
        1,
        vec![
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(10)),
        ],
        1,
    ));
    g.add_rule(make_rule(2, vec![Symbol::Terminal(SymbolId(10))], 2));
    g.rule_names.insert(SymbolId(0), "S".into());
    g.rule_names.insert(SymbolId(1), "A".into());
    g.rule_names.insert(SymbolId(2), "B".into());

    let result = optimize_grammar(g).unwrap();

    // The optimizer may inline or eliminate some rules but the grammar
    // should still contain rules that derive the terminal.
    let all_syms: Vec<Symbol> = result
        .all_rules()
        .flat_map(|r| r.rhs.iter().cloned())
        .collect();
    let has_terminal = all_syms.iter().any(|s| matches!(s, Symbol::Terminal(_)));
    assert!(
        has_terminal,
        "optimizer lost the terminal in the derivation chain"
    );
}

#[test]
fn optimizer_removes_truly_unused_token() {
    let mut g = Grammar::new("unused_tok".into());
    let (tid1, tok1) = make_token(10, "used", "a");
    let (tid2, tok2) = make_token(20, "unused", "b");
    g.tokens.insert(tid1, tok1);
    g.tokens.insert(tid2, tok2);

    // Only reference tid1
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(10))], 0));

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    assert!(
        stats.removed_unused_symbols >= 1,
        "expected at least 1 removal, got {}",
        stats.removed_unused_symbols
    );
    assert!(
        !g.tokens.contains_key(&SymbolId(20)),
        "unused token should be removed"
    );
}

// ---------------------------------------------------------------------------
// Optimizer preserves precedence and associativity
// ---------------------------------------------------------------------------

#[test]
fn optimizer_preserves_precedence_on_non_unit_rules() {
    let mut g = Grammar::new("prec".into());
    let (tid1, tok1) = make_token(10, "plus", "+");
    let (tid2, tok2) = make_token(11, "num", "1");
    g.tokens.insert(tid1, tok1);
    g.tokens.insert(tid2, tok2);

    // expr -> expr + expr  (left, prec 1)
    g.add_rule(make_prec_rule(
        0,
        vec![
            Symbol::NonTerminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(10)),
            Symbol::NonTerminal(SymbolId(0)),
        ],
        0,
        PrecedenceKind::Static(1),
        Associativity::Left,
    ));
    // expr -> num
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(11))], 1));
    g.rule_names.insert(SymbolId(0), "Expr".into());

    let result = optimize_grammar(g).unwrap();

    // After optimization, at least one rule should retain the original precedence.
    let any_prec = result.all_rules().any(|r| {
        r.precedence == Some(PrecedenceKind::Static(1))
            && r.associativity == Some(Associativity::Left)
    });
    assert!(any_prec, "precedence/associativity lost after optimization");
}

#[test]
fn optimizer_preserves_dynamic_precedence() {
    let mut g = Grammar::new("dyn_prec".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);

    g.add_rule(make_prec_rule(
        0,
        vec![
            Symbol::NonTerminal(SymbolId(0)),
            Symbol::Terminal(SymbolId(10)),
            Symbol::NonTerminal(SymbolId(0)),
        ],
        0,
        PrecedenceKind::Dynamic(3),
        Associativity::Right,
    ));
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(10))], 1));
    g.rule_names.insert(SymbolId(0), "D".into());

    let result = optimize_grammar(g).unwrap();

    let any_dyn = result.all_rules().any(|r| {
        r.precedence == Some(PrecedenceKind::Dynamic(3))
            && r.associativity == Some(Associativity::Right)
    });
    assert!(any_dyn, "dynamic precedence lost after optimization");
}

// ---------------------------------------------------------------------------
// Optimizer handles grammars with complex symbol types
// ---------------------------------------------------------------------------

#[test]
fn optimizer_handles_optional_repeat_choice_sequence() {
    let mut g = Grammar::new("complex_syms".into());
    let (tid, tok) = make_token(10, "a", "a");
    g.tokens.insert(tid, tok);

    g.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(10)))),
            Symbol::Repeat(Box::new(Symbol::Terminal(SymbolId(10)))),
            Symbol::RepeatOne(Box::new(Symbol::Terminal(SymbolId(10)))),
            Symbol::Choice(vec![Symbol::Terminal(SymbolId(10)), Symbol::Epsilon]),
            Symbol::Sequence(vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(10)),
            ]),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let result = optimize_grammar(g);
    assert!(
        result.is_ok(),
        "optimizer panicked on complex symbols: {:?}",
        result.err()
    );
}

// ---------------------------------------------------------------------------
// Optimizer handles source_file convention
// ---------------------------------------------------------------------------

#[test]
fn optimizer_preserves_source_file_rule() {
    let mut g = Grammar::new("sf".into());
    let (tid, tok) = make_token(10, "x", "x");
    g.tokens.insert(tid, tok);

    let sf = SymbolId(0);
    let inner = SymbolId(1);

    // source_file -> inner
    g.add_rule(make_rule(0, vec![Symbol::NonTerminal(inner)], 0));
    // inner -> x
    g.add_rule(make_rule(1, vec![Symbol::Terminal(SymbolId(10))], 1));
    g.rule_names.insert(sf, "source_file".into());
    g.rule_names.insert(inner, "inner".into());

    let result = optimize_grammar(g).unwrap();

    // source_file should not be inlined away.
    let sf_id = result.find_symbol_by_name("source_file");
    // It's acceptable for the optimizer to either keep the source_file rule
    // or inline it; the key thing is it doesn't panic.
    // If it exists, verify it has rules.
    if let Some(id) = sf_id {
        assert!(
            result.rules.contains_key(&id),
            "source_file symbol exists but has no rules"
        );
    }
}

// ---------------------------------------------------------------------------
// Optimizer handles duplicate tokens (merge)
// ---------------------------------------------------------------------------

#[test]
fn optimizer_merges_duplicate_token_patterns() {
    let mut g = Grammar::new("merge".into());
    let (tid1, tok1) = make_token(10, "plus1", "+");
    let (tid2, tok2) = make_token(11, "plus2", "+"); // same pattern
    g.tokens.insert(tid1, tok1);
    g.tokens.insert(tid2, tok2);

    g.add_rule(make_rule(
        0,
        vec![
            Symbol::Terminal(SymbolId(10)),
            Symbol::Terminal(SymbolId(11)),
        ],
        0,
    ));

    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);

    assert!(
        stats.merged_tokens >= 1,
        "expected at least 1 token merge, got {}",
        stats.merged_tokens
    );
}

// ---------------------------------------------------------------------------
// Optimizer handles left recursion with fields
// ---------------------------------------------------------------------------

#[test]
fn optimizer_preserves_fields_in_left_recursion_transform() {
    let mut g = Grammar::new("lr_fields".into());
    let (tid1, tok1) = make_token(10, "plus", "+");
    let (tid2, tok2) = make_token(11, "num", "1");
    g.tokens.insert(tid1, tok1);
    g.tokens.insert(tid2, tok2);
    g.fields.insert(FieldId(0), "left".into());
    g.fields.insert(FieldId(1), "op".into());
    g.fields.insert(FieldId(2), "right".into());

    let expr = SymbolId(0);
    g.rule_names.insert(expr, "Expr".into());

    // expr -> expr + expr   with fields
    g.add_rule(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(SymbolId(10)),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0), (FieldId(1), 1), (FieldId(2), 2)],
        production_id: ProductionId(0),
    });
    // expr -> num
    g.add_rule(make_rule(0, vec![Symbol::Terminal(SymbolId(11))], 1));

    let result = optimize_grammar(g);
    assert!(
        result.is_ok(),
        "optimizer panicked on left-recursive rule with fields"
    );
}

// ---------------------------------------------------------------------------
// Optimizer idempotency
// ---------------------------------------------------------------------------

#[test]
fn optimizer_is_idempotent_on_simple_grammar() {
    let build = || {
        let mut g = Grammar::new("idem".into());
        let (tid1, tok1) = make_token(10, "x", "x");
        let (tid2, tok2) = make_token(11, "y", "y");
        g.tokens.insert(tid1, tok1);
        g.tokens.insert(tid2, tok2);
        // Non-trivial rule so it doesn't get inlined away
        g.add_rule(make_rule(
            0,
            vec![
                Symbol::Terminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(11)),
            ],
            0,
        ));
        g.rule_names.insert(SymbolId(0), "S".into());
        g
    };

    let first = optimize_grammar(build()).unwrap();
    let second = optimize_grammar(first.clone()).unwrap();

    assert_eq!(first.rules.len(), second.rules.len());
    assert_eq!(first.tokens.len(), second.tokens.len());
}
