// Property tests for FirstFollowSets computation
use adze_glr_core::*;
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helper: build grammar with GrammarBuilder
// ---------------------------------------------------------------------------

fn simple_ab_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn two_rule_grammar() -> Grammar {
    GrammarBuilder::new("two_rules")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .start("start")
        .build()
}

fn sequence_grammar() -> Grammar {
    GrammarBuilder::new("sequence")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

// ---------------------------------------------------------------------------
// Basic FirstFollowSets computation
// ---------------------------------------------------------------------------

#[test]
fn compute_simple_grammar() {
    let grammar = simple_ab_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    // 'a' terminal should be in FIRST set of start
    let start_id = *grammar
        .rule_names
        .iter()
        .find(|(_, n)| *n == "start")
        .unwrap()
        .0;
    let first = ff.first(start_id);
    assert!(first.is_some());
}

#[test]
fn compute_two_rule_grammar() {
    let grammar = two_rule_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let start_id = *grammar
        .rule_names
        .iter()
        .find(|(_, n)| *n == "start")
        .unwrap()
        .0;
    let first = ff.first(start_id).unwrap();
    // Both terminals should be in FIRST(start)
    assert!(first.contains(1)); // SymbolId(1)
    assert!(first.contains(2)); // SymbolId(2)
}

#[test]
fn compute_sequence_grammar() {
    let grammar = sequence_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let start_id = *grammar
        .rule_names
        .iter()
        .find(|(_, n)| *n == "start")
        .unwrap()
        .0;
    let first = ff.first(start_id).unwrap();
    // FIRST(start) = {a}
    assert!(first.contains(1));
}

// ---------------------------------------------------------------------------
// Nullable detection
// ---------------------------------------------------------------------------

#[test]
fn terminal_not_nullable() {
    let grammar = simple_ab_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    // Terminals are never nullable
    assert!(!ff.is_nullable(SymbolId(1)));
}

#[test]
fn start_with_terminal_not_nullable() {
    let grammar = simple_ab_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let start_id = *grammar
        .rule_names
        .iter()
        .find(|(_, n)| *n == "start")
        .unwrap()
        .0;
    assert!(!ff.is_nullable(start_id));
}

// ---------------------------------------------------------------------------
// Epsilon grammar
// ---------------------------------------------------------------------------

#[test]
fn epsilon_rule_makes_nullable() {
    let mut grammar = Grammar::new("epsilon_test".to_string());
    let tok_a = SymbolId(1);
    let nt_start = SymbolId(10);

    grammar.tokens.insert(
        tok_a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(nt_start, "start".into());
    grammar.rules.insert(
        nt_start,
        vec![
            Rule {
                lhs: nt_start,
                rhs: vec![Symbol::Terminal(tok_a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: nt_start,
                rhs: vec![Symbol::Epsilon],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    assert!(ff.is_nullable(nt_start));
}

// ---------------------------------------------------------------------------
// FIRST of sequence
// ---------------------------------------------------------------------------

#[test]
fn first_of_terminal_sequence() {
    let grammar = sequence_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let seq = vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))];
    let first_set = ff.first_of_sequence(&seq).unwrap();
    assert!(first_set.contains(1));
    assert!(!first_set.contains(2)); // 'b' not in FIRST since 'a' is not nullable
}

#[test]
fn first_of_single_terminal() {
    let grammar = simple_ab_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let seq = vec![Symbol::Terminal(SymbolId(1))];
    let first_set = ff.first_of_sequence(&seq).unwrap();
    assert!(first_set.contains(1));
}

#[test]
fn first_of_empty_sequence() {
    let grammar = simple_ab_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let seq: Vec<Symbol> = vec![];
    let first_set = ff.first_of_sequence(&seq).unwrap();
    // Empty sequence FIRST set should be empty (no symbols to derive)
    assert_eq!(first_set.count_ones(..), 0);
}

// ---------------------------------------------------------------------------
// FOLLOW set tests
// ---------------------------------------------------------------------------

#[test]
fn follow_of_terminal_in_sequence() {
    let grammar = sequence_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    // In S -> a b, FOLLOW(a) would depend on what follows in the rule
    // but terminal FOLLOW is less meaningful; check nonterminal
    let start_id = *grammar
        .rule_names
        .iter()
        .find(|(_, n)| *n == "start")
        .unwrap()
        .0;
    let follow = ff.follow(start_id);
    // start is the root nonterminal - FOLLOW should contain EOF or be present
    assert!(follow.is_some());
}

// ---------------------------------------------------------------------------
// compute_normalized
// ---------------------------------------------------------------------------

#[test]
fn compute_normalized_simple() {
    let mut grammar = simple_ab_grammar();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let start_id = *grammar
        .rule_names
        .iter()
        .find(|(_, n)| *n == "start")
        .unwrap()
        .0;
    assert!(ff.first(start_id).is_some());
}

// ---------------------------------------------------------------------------
// Recursive grammar
// ---------------------------------------------------------------------------

#[test]
fn recursive_grammar_computes() {
    let mut grammar = Grammar::new("recursive".to_string());
    let tok_a = SymbolId(1);
    let tok_plus = SymbolId(2);
    let nt_expr = SymbolId(10);

    grammar.tokens.insert(
        tok_a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        tok_plus,
        Token {
            name: "plus".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(nt_expr, "expr".into());
    grammar.rules.insert(
        nt_expr,
        vec![
            Rule {
                lhs: nt_expr,
                rhs: vec![Symbol::Terminal(tok_a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: nt_expr,
                rhs: vec![
                    Symbol::NonTerminal(nt_expr),
                    Symbol::Terminal(tok_plus),
                    Symbol::NonTerminal(nt_expr),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let first = ff.first(nt_expr).unwrap();
    assert!(first.contains(tok_a.0 as usize));
    assert!(!ff.is_nullable(nt_expr));
}

// ---------------------------------------------------------------------------
// Multiple nonterminals
// ---------------------------------------------------------------------------

#[test]
fn two_nonterminal_grammar() {
    let mut grammar = Grammar::new("two_nt".to_string());
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    let nt_start = SymbolId(10);
    let nt_inner = SymbolId(11);

    grammar.tokens.insert(
        tok_a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        tok_b,
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(nt_start, "start".into());
    grammar.rule_names.insert(nt_inner, "inner".into());

    grammar.rules.insert(
        nt_inner,
        vec![Rule {
            lhs: nt_inner,
            rhs: vec![Symbol::Terminal(tok_b)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    grammar.rules.insert(
        nt_start,
        vec![Rule {
            lhs: nt_start,
            rhs: vec![Symbol::Terminal(tok_a), Symbol::NonTerminal(nt_inner)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let first_start = ff.first(nt_start).unwrap();
    assert!(first_start.contains(tok_a.0 as usize));
    // FIRST(start) should NOT contain b since a is not nullable
    assert!(!first_start.contains(tok_b.0 as usize));

    let first_inner = ff.first(nt_inner).unwrap();
    assert!(first_inner.contains(tok_b.0 as usize));
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn first_set_nonempty_for_terminal_rule(tok_id in 1u16..50) {
        let mut grammar = Grammar::new("prop_test".to_string());
        let tok = SymbolId(tok_id);
        let nt = SymbolId(100);

        grammar.tokens.insert(tok, Token {
            name: format!("t{}", tok_id),
            pattern: TokenPattern::String(format!("t{}", tok_id)),
            fragile: false,
        });
        grammar.rule_names.insert(nt, "start".into());
        grammar.rules.insert(nt, vec![
            Rule {
                lhs: nt,
                rhs: vec![Symbol::Terminal(tok)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }
        ]);

        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let first = ff.first(nt).unwrap();
        prop_assert!(first.contains(tok_id as usize));
    }

    #[test]
    fn non_nullable_terminal_rule(tok_id in 1u16..50) {
        let mut grammar = Grammar::new("prop_test".to_string());
        let tok = SymbolId(tok_id);
        let nt = SymbolId(100);

        grammar.tokens.insert(tok, Token {
            name: format!("t{}", tok_id),
            pattern: TokenPattern::String(format!("t{}", tok_id)),
            fragile: false,
        });
        grammar.rule_names.insert(nt, "start".into());
        grammar.rules.insert(nt, vec![
            Rule {
                lhs: nt,
                rhs: vec![Symbol::Terminal(tok)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            }
        ]);

        let ff = FirstFollowSets::compute(&grammar).unwrap();
        prop_assert!(!ff.is_nullable(nt));
    }

    #[test]
    fn choice_grammar_first_union(a_id in 1u16..20, b_id in 21u16..40) {
        let mut grammar = Grammar::new("choice_prop".to_string());
        let tok_a = SymbolId(a_id);
        let tok_b = SymbolId(b_id);
        let nt = SymbolId(100);

        grammar.tokens.insert(tok_a, Token {
            name: format!("a{}", a_id),
            pattern: TokenPattern::String(format!("a{}", a_id)),
            fragile: false,
        });
        grammar.tokens.insert(tok_b, Token {
            name: format!("b{}", b_id),
            pattern: TokenPattern::String(format!("b{}", b_id)),
            fragile: false,
        });
        grammar.rule_names.insert(nt, "start".into());
        grammar.rules.insert(nt, vec![
            Rule {
                lhs: nt,
                rhs: vec![Symbol::Terminal(tok_a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: nt,
                rhs: vec![Symbol::Terminal(tok_b)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ]);

        let ff = FirstFollowSets::compute(&grammar).unwrap();
        let first = ff.first(nt).unwrap();
        prop_assert!(first.contains(a_id as usize));
        prop_assert!(first.contains(b_id as usize));
    }
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn single_token_single_rule_grammar() {
    let mut grammar = Grammar::new("minimal".to_string());
    let tok = SymbolId(1);
    let nt = SymbolId(10);

    grammar.tokens.insert(
        tok,
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(nt, "start".into());
    grammar.rules.insert(
        nt,
        vec![Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(tok)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    assert!(ff.first(nt).is_some());
    assert!(ff.follow(nt).is_some());
    assert!(!ff.is_nullable(nt));
}

#[test]
fn first_set_query_unknown_symbol() {
    let grammar = simple_ab_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    // Query for a symbol not in the grammar
    assert!(ff.first(SymbolId(999)).is_none());
}

#[test]
fn follow_set_query_unknown_symbol() {
    let grammar = simple_ab_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    assert!(ff.follow(SymbolId(999)).is_none());
}

#[test]
fn is_nullable_unknown_symbol() {
    let grammar = simple_ab_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    // Unknown symbol should not be nullable
    assert!(!ff.is_nullable(SymbolId(999)));
}
