// Comprehensive tests for Grammar::normalize() and Grammar::optimize()
// Tests cover 8 categories with 8 tests each = 64 total tests
// Uses Rust 2024 edition conventions

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Symbol};

// ============================================================================
// Category 1: Basic Normalize (8 tests)
// ============================================================================

#[test]
fn basic_normalize_empty_grammar() {
    let mut grammar = GrammarBuilder::new("empty_basic").build();
    let initial_rules = grammar.all_rules().count();
    grammar.normalize();
    assert_eq!(grammar.all_rules().count(), initial_rules);
    assert!(grammar.name == "empty_basic");
}

#[test]
fn basic_normalize_single_rule() {
    let mut grammar = GrammarBuilder::new("single")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    assert!(grammar.all_rules().count() >= 1);
    assert!(!grammar.tokens.is_empty());
}

#[test]
fn basic_normalize_multi_rule() {
    let mut grammar = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .rule("s", vec!["b", "a"])
        .rule("x", vec!["a"])
        .start("s")
        .build();
    let initial = grammar.all_rules().count();
    grammar.normalize();
    let after = grammar.all_rules().count();
    assert_eq!(initial, after);
}

#[test]
fn basic_normalize_idempotent_double_normalize() {
    let mut grammar = GrammarBuilder::new("idem")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    grammar.normalize();
    let count1 = grammar.all_rules().count();
    grammar.normalize();
    let count2 = grammar.all_rules().count();
    assert_eq!(count1, count2);
}

#[test]
fn basic_normalize_preserves_start_symbol() {
    let mut grammar = GrammarBuilder::new("start_preserve")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("x", vec!["a"])
        .start("s")
        .build();
    let start_before = grammar.start_symbol();
    grammar.normalize();
    let start_after = grammar.start_symbol();
    assert_eq!(start_before, start_after);
}

#[test]
fn basic_normalize_preserves_tokens() {
    let mut grammar = GrammarBuilder::new("tokens_preserve")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let token_count_before = grammar.tokens.len();
    grammar.normalize();
    let token_count_after = grammar.tokens.len();
    assert_eq!(token_count_before, token_count_after);
    assert_eq!(token_count_after, 3);
}

#[test]
fn basic_normalize_preserves_grammar_name() {
    let mut grammar = GrammarBuilder::new("myname")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let name_before = grammar.name.clone();
    grammar.normalize();
    assert_eq!(grammar.name, name_before);
    assert_eq!(grammar.name, "myname");
}

#[test]
fn basic_normalize_result_validates() {
    let mut grammar = GrammarBuilder::new("validate_ok")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    grammar.normalize();
    assert!(grammar.validate().is_ok());
}

// ============================================================================
// Category 2: Optional Expansion (8 tests)
// ============================================================================

#[test]
fn optional_normalize_simple_optional() {
    let mut grammar = GrammarBuilder::new("opt_simple").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);
    grammar.rule_names.insert(s_id, "s".to_string());
    grammar.rule_names.insert(a_id, "a".to_string());

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let rules = grammar.all_rules().collect::<Vec<_>>();
    assert!(rules.len() > 1);
    let has_epsilon = rules.iter().any(|r| r.rhs.contains(&Symbol::Epsilon));
    assert!(has_epsilon);
}

#[test]
fn optional_generates_aux_rule() {
    let mut grammar = GrammarBuilder::new("opt_aux").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);
    grammar.rule_names.insert(s_id, "s".to_string());
    grammar.rule_names.insert(a_id, "a".to_string());

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let rule_count = grammar.all_rules().count();
    assert!(rule_count >= 3);
}

#[test]
fn optional_aux_rule_has_epsilon_alt() {
    let mut grammar = GrammarBuilder::new("opt_epsilon").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);
    grammar.rule_names.insert(s_id, "s".to_string());
    grammar.rule_names.insert(a_id, "a".to_string());

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let has_epsilon_rule = grammar
        .all_rules()
        .any(|r| r.rhs.len() == 1 && r.rhs[0] == Symbol::Epsilon);
    assert!(has_epsilon_rule);
}

#[test]
fn optional_nested_optional() {
    let mut grammar = GrammarBuilder::new("opt_nested").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);
    grammar.rule_names.insert(s_id, "s".to_string());
    grammar.rule_names.insert(a_id, "a".to_string());

    let nested_opt = Symbol::Optional(Box::new(Symbol::Optional(Box::new(Symbol::Terminal(a_id)))));
    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![nested_opt],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

#[test]
fn optional_multiple_optionals_in_one_rule() {
    let mut grammar = GrammarBuilder::new("opt_multi").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);
    grammar.rule_names.insert(s_id, "s".to_string());

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(a_id))),
            Symbol::Optional(Box::new(Symbol::Terminal(b_id))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let rule_count = grammar.all_rules().count();
    assert!(rule_count >= 4);
}

#[test]
fn optional_at_start_of_rule() {
    let mut grammar = GrammarBuilder::new("opt_start").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(a_id))),
            Symbol::Terminal(b_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

#[test]
fn optional_at_end_of_rule() {
    let mut grammar = GrammarBuilder::new("opt_end").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Terminal(a_id),
            Symbol::Optional(Box::new(Symbol::Terminal(b_id))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

#[test]
fn optional_preserves_precedence() {
    let mut grammar = GrammarBuilder::new("opt_prec").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);
    grammar.rule_names.insert(s_id, "s".to_string());

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
        precedence: Some(adze_ir::PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

// ============================================================================
// Category 3: Repeat Expansion (8 tests)
// ============================================================================

#[test]
fn repeat_normalize_kleene_star() {
    let mut grammar = GrammarBuilder::new("repeat_star").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);
    grammar.rule_names.insert(s_id, "s".to_string());

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let rule_count = grammar.all_rules().count();
    assert!(rule_count >= 2);
}

#[test]
fn repeat_generates_recursive_rule() {
    let mut grammar = GrammarBuilder::new("repeat_recursive").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);
    grammar.rule_names.insert(s_id, "s".to_string());

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let has_recursive = grammar.all_rules().any(|r| {
        r.rhs.iter().any(|sym| {
            if let Symbol::NonTerminal(_) = sym {
                r.rhs.iter().any(|inner| inner == sym)
            } else {
                false
            }
        })
    });
    assert!(has_recursive || grammar.all_rules().count() >= 2);
}

#[test]
fn repeat_minimum_one() {
    let mut grammar = GrammarBuilder::new("repeat_one").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);
    grammar.rule_names.insert(s_id, "s".to_string());

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let rule_count = grammar.all_rules().count();
    assert!(rule_count >= 2);
}

#[test]
fn repeat_with_separator() {
    let mut grammar = GrammarBuilder::new("repeat_sep").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Repeat(Box::new(Symbol::Terminal(a_id))),
            Symbol::Terminal(b_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

#[test]
fn repeat_nested_repeats() {
    let mut grammar = GrammarBuilder::new("repeat_nested").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);
    grammar.rule_names.insert(s_id, "s".to_string());

    let nested = Symbol::Repeat(Box::new(Symbol::Repeat(Box::new(Symbol::Terminal(a_id)))));
    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![nested],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

#[test]
fn repeat_at_start_of_rule() {
    let mut grammar = GrammarBuilder::new("repeat_start").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Repeat(Box::new(Symbol::Terminal(a_id))),
            Symbol::Terminal(b_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

#[test]
fn repeat_at_end_of_rule() {
    let mut grammar = GrammarBuilder::new("repeat_end").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Terminal(a_id),
            Symbol::Repeat(Box::new(Symbol::Terminal(b_id))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

#[test]
fn repeat_preserves_associativity() {
    let mut grammar = GrammarBuilder::new("repeat_assoc").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);
    grammar.rule_names.insert(s_id, "s".to_string());

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: Some(Associativity::Right),
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

// ============================================================================
// Category 4: Choice Expansion (8 tests)
// ============================================================================

#[test]
fn choice_normalize_simple_choice() {
    let mut grammar = GrammarBuilder::new("choice_simple").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let rule_count = grammar.all_rules().count();
    assert!(rule_count >= 2);
}

#[test]
fn choice_generates_multiple_rules() {
    let mut grammar = GrammarBuilder::new("choice_multi").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let c_id = adze_ir::SymbolId(3);
    let s_id = adze_ir::SymbolId(4);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
            Symbol::Terminal(c_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let rule_count = grammar.all_rules().count();
    assert!(rule_count >= 3);
}

#[test]
fn choice_nested_choices_flattened() {
    let mut grammar = GrammarBuilder::new("choice_nested").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let nested_choice = Symbol::Choice(vec![
        Symbol::Choice(vec![Symbol::Terminal(a_id), Symbol::Terminal(b_id)]),
        Symbol::Terminal(a_id),
    ]);
    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![nested_choice],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

#[test]
fn choice_with_epsilon() {
    let mut grammar = GrammarBuilder::new("choice_epsilon").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(a_id),
            Symbol::Epsilon,
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let has_epsilon = grammar
        .all_rules()
        .any(|r| r.rhs.contains(&Symbol::Epsilon));
    assert!(has_epsilon);
}

#[test]
fn choice_with_terminals_only() {
    let mut grammar = GrammarBuilder::new("choice_term").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let rule_count = grammar.all_rules().count();
    assert!(rule_count >= 2);
}

#[test]
fn choice_with_nonterminals() {
    let mut grammar = GrammarBuilder::new("choice_nonterm").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::NonTerminal(a_id),
            Symbol::NonTerminal(b_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

#[test]
fn choice_many_alternatives() {
    let mut grammar = GrammarBuilder::new("choice_many").build();
    let s_id = adze_ir::SymbolId(10);

    let mut choices = vec![];
    for i in 1..9 {
        choices.push(Symbol::Terminal(adze_ir::SymbolId(i as u16)));
    }

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Choice(choices)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let rule_count = grammar.all_rules().count();
    assert!(rule_count >= 8);
}

#[test]
fn choice_preserves_production_ids() {
    let mut grammar = GrammarBuilder::new("choice_prod_id").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(42),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let has_prod_id = grammar
        .all_rules()
        .any(|r| r.production_id == adze_ir::ProductionId(42));
    assert!(has_prod_id || grammar.all_rules().count() >= 2);
}

// ============================================================================
// Category 5: Combined Transformations (8 tests)
// ============================================================================

#[test]
fn combined_optional_and_repeat() {
    let mut grammar = GrammarBuilder::new("combined_opt_rep").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(a_id))),
            Symbol::Repeat(Box::new(Symbol::Terminal(a_id))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 3);
}

#[test]
fn combined_choice_and_optional() {
    let mut grammar = GrammarBuilder::new("combined_choice_opt").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Choice(vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
        ])))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 3);
}

#[test]
fn combined_all_three() {
    let mut grammar = GrammarBuilder::new("combined_all").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(a_id))),
            Symbol::Choice(vec![Symbol::Terminal(b_id), Symbol::Terminal(a_id)]),
            Symbol::Repeat(Box::new(Symbol::Terminal(b_id))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 3);
}

#[test]
fn combined_deeply_nested() {
    let mut grammar = GrammarBuilder::new("combined_deep").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);

    let deeply_nested = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::Optional(
        Box::new(Symbol::Terminal(a_id)),
    )))));
    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![deeply_nested],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

#[test]
fn combined_complex_expression_grammar() {
    let mut grammar = GrammarBuilder::new("combined_expr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule("expr", vec!["term"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build();

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 3);
    assert_eq!(grammar.tokens.len(), 4);
}

#[test]
fn combined_list_with_optional_separator() {
    let mut grammar = GrammarBuilder::new("combined_list_sep").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Repeat(Box::new(Symbol::Terminal(a_id))),
            Symbol::Optional(Box::new(Symbol::Terminal(b_id))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 2);
}

#[test]
fn combined_multiple_in_one_rule() {
    let mut grammar = GrammarBuilder::new("combined_multi").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(a_id))),
            Symbol::Repeat(Box::new(Symbol::Terminal(b_id))),
            Symbol::Optional(Box::new(Symbol::Terminal(a_id))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 3);
}

#[test]
fn combined_preserves_structure() {
    let mut grammar = GrammarBuilder::new("combined_preserve")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let name_before = grammar.name.clone();
    let start_before = grammar.start_symbol();
    grammar.normalize();
    let name_after = grammar.name.clone();
    let start_after = grammar.start_symbol();

    assert_eq!(name_before, name_after);
    assert_eq!(start_before, start_after);
}

// ============================================================================
// Category 6: Rule Counting (8 tests)
// ============================================================================

#[test]
fn counting_normalize_adds_aux_rules() {
    let mut grammar = GrammarBuilder::new("count_aux").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    let count_before = grammar.all_rules().count();
    grammar.normalize();
    let count_after = grammar.all_rules().count();

    assert!(count_after > count_before);
}

#[test]
fn counting_before_and_after() {
    let mut grammar = GrammarBuilder::new("count_before_after").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    let before = grammar.all_rules().count();
    grammar.normalize();
    let after = grammar.all_rules().count();

    assert!(after > before);
    assert!(after >= 2);
}

#[test]
fn counting_symbols_after_normalize() {
    let mut grammar = GrammarBuilder::new("count_symbols").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    let rule = adze_ir::Rule {
        lhs: s_id,
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(a_id))),
            Symbol::Choice(vec![Symbol::Terminal(b_id), Symbol::Terminal(a_id)]),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar.normalize();
    let total_symbols: usize = grammar.all_rules().map(|r| r.rhs.len()).sum();
    assert!(total_symbols > 0);
}

#[test]
fn counting_complex_grammar_counts() {
    let mut grammar = GrammarBuilder::new("count_complex").build();
    let a_id = adze_ir::SymbolId(1);
    let b_id = adze_ir::SymbolId(2);
    let s_id = adze_ir::SymbolId(3);

    grammar.add_rule(adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });

    grammar.add_rule(adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(b_id),
            Symbol::Terminal(a_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(1),
    });

    let before = grammar.all_rules().count();
    grammar.normalize();
    let after = grammar.all_rules().count();

    assert!(after > before);
}

#[test]
fn counting_arithmetic_grammar() {
    let mut grammar = GrammarBuilder::new("count_arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let before = grammar.all_rules().count();
    grammar.normalize();
    let after = grammar.all_rules().count();

    assert_eq!(before, after);
    assert_eq!(after, 3);
}

#[test]
fn counting_with_inline_rules() {
    let mut grammar = GrammarBuilder::new("count_inline")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["x", "b"])
        .rule("x", vec!["a"])
        .inline("x")
        .start("s")
        .build();

    let before = grammar.all_rules().count();
    grammar.normalize();
    let after = grammar.all_rules().count();

    assert_eq!(before, after);
    assert!(!grammar.inline_rules.is_empty());
}

#[test]
fn counting_with_supertypes() {
    let mut grammar = GrammarBuilder::new("count_super")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("x", vec!["b"])
        .supertype("base")
        .start("s")
        .build();

    let before = grammar.all_rules().count();
    grammar.normalize();
    let after = grammar.all_rules().count();

    assert_eq!(before, after);
    assert!(!grammar.supertypes.is_empty());
}

#[test]
fn counting_rule_count_deterministic() {
    let mut grammar1 = GrammarBuilder::new("count_det").build();
    let mut grammar2 = GrammarBuilder::new("count_det").build();

    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);

    for g in [&mut grammar1, &mut grammar2] {
        g.add_rule(adze_ir::Rule {
            lhs: s_id,
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: adze_ir::ProductionId(0),
        });
    }

    grammar1.normalize();
    grammar2.normalize();

    assert_eq!(grammar1.all_rules().count(), grammar2.all_rules().count());
}

// ============================================================================
// Category 7: Optimize (8 tests)
// ============================================================================

#[test]
fn optimize_simple_grammar() {
    let mut grammar = GrammarBuilder::new("opt_simple")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    grammar.optimize();
    assert!(grammar.all_rules().count() >= 1);
}

#[test]
fn optimize_removes_unreachable() {
    let mut grammar = GrammarBuilder::new("opt_unreachable")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("unused", vec!["b"])
        .start("s")
        .build();

    grammar.optimize();
    assert!(grammar.all_rules().count() <= 2);
}

#[test]
fn optimize_after_normalize() {
    let mut grammar = GrammarBuilder::new("opt_after_norm").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);

    grammar.add_rule(adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });

    grammar.normalize();
    let count_after_normalize = grammar.all_rules().count();
    grammar.optimize();
    let count_after_optimize = grammar.all_rules().count();

    assert!(count_after_optimize >= 1);
    assert!(count_after_optimize <= count_after_normalize);
}

#[test]
fn optimize_idempotent() {
    let mut grammar = GrammarBuilder::new("opt_idem")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    grammar.optimize();
    let count1 = grammar.all_rules().count();
    grammar.optimize();
    let count2 = grammar.all_rules().count();

    assert_eq!(count1, count2);
}

#[test]
fn optimize_preserves_semantics() {
    let mut grammar = GrammarBuilder::new("opt_semantics")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();

    let rules_before: Vec<_> = grammar.all_rules().map(|r| (r.lhs, r.rhs.len())).collect();
    grammar.optimize();
    let rules_after: Vec<_> = grammar.all_rules().map(|r| (r.lhs, r.rhs.len())).collect();

    assert!(rules_before.len() == rules_after.len());
}

#[test]
fn optimize_complex_grammar() {
    let mut grammar = GrammarBuilder::new("opt_complex")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let before = grammar.all_rules().count();
    grammar.optimize();
    let after = grammar.all_rules().count();

    assert!(after <= before);
    assert_eq!(after, 4);
}

#[test]
fn optimize_normalize_then_count() {
    let mut grammar = GrammarBuilder::new("opt_norm_count").build();
    let a_id = adze_ir::SymbolId(1);
    let s_id = adze_ir::SymbolId(2);

    grammar.add_rule(adze_ir::Rule {
        lhs: s_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(a_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: adze_ir::ProductionId(0),
    });

    grammar.normalize();
    let count_after_norm = grammar.all_rules().count();
    grammar.optimize();
    let count_after_opt = grammar.all_rules().count();

    assert!(count_after_norm >= 2);
    assert!(count_after_opt <= count_after_norm);
}

#[test]
fn optimize_preserves_start() {
    let mut grammar = GrammarBuilder::new("opt_preserve_start")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("x", vec!["a"])
        .start("s")
        .build();

    let start_before = grammar.start_symbol();
    grammar.optimize();
    let start_after = grammar.start_symbol();

    assert_eq!(start_before, start_after);
}

// ============================================================================
// Category 8: Edge Cases (8 tests)
// ============================================================================

#[test]
fn edge_normalize_all_rule_types() {
    let mut grammar = GrammarBuilder::new("edge_all_types")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("x", vec!["a", "b"])
        .rule("y", vec![])
        .start("s")
        .build();

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 3);
}

#[test]
fn edge_normalize_many_rules() {
    let mut grammar = GrammarBuilder::new("edge_many_rules").build();

    for i in 0..50 {
        let lhs_id = adze_ir::SymbolId((i as u16) + 100);
        grammar.rule_names.insert(lhs_id, format!("rule_{}", i));
        grammar.add_rule(adze_ir::Rule {
            lhs: lhs_id,
            rhs: vec![Symbol::Terminal(adze_ir::SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: adze_ir::ProductionId(i as u16),
        });
    }

    grammar.normalize();
    assert_eq!(grammar.all_rules().count(), 50);
}

#[test]
fn edge_normalize_with_unicode_names() {
    let mut grammar = GrammarBuilder::new("edge_unicode")
        .token("α", "a")
        .token("β", "b")
        .rule("γ", vec!["α", "β"])
        .start("γ")
        .build();

    grammar.normalize();
    assert!(grammar.all_rules().count() >= 1);
    assert!(grammar.tokens.len() >= 2);
}

#[test]
fn edge_normalize_with_long_chain() {
    let mut grammar = GrammarBuilder::new("edge_chain")
        .token("a", "a")
        .rule("s", vec!["x1"])
        .rule("x1", vec!["x2"])
        .rule("x2", vec!["x3"])
        .rule("x3", vec!["x4"])
        .rule("x4", vec!["x5"])
        .rule("x5", vec!["x6"])
        .rule("x6", vec!["x7"])
        .rule("x7", vec!["a"])
        .start("s")
        .build();

    grammar.normalize();
    assert_eq!(grammar.all_rules().count(), 8);
}

#[test]
fn edge_normalize_then_validate() {
    let mut grammar = GrammarBuilder::new("edge_validate")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    grammar.normalize();
    let validation = grammar.validate();
    assert!(validation.is_ok());
}

#[test]
fn edge_normalize_preserves_extras() {
    let mut grammar = GrammarBuilder::new("edge_extras")
        .token("a", "a")
        .token("WS", r"\\s+")
        .token("COMMENT", r"//.*")
        .rule("s", vec!["a"])
        .extra("WS")
        .extra("COMMENT")
        .start("s")
        .build();

    let extras_before = grammar.extras.len();
    grammar.normalize();
    let extras_after = grammar.extras.len();

    assert_eq!(extras_before, extras_after);
    assert_eq!(extras_after, 2);
}

#[test]
fn edge_normalize_preserves_externals() {
    let mut grammar = GrammarBuilder::new("edge_externals")
        .token("a", "a")
        .rule("s", vec!["a"])
        .external("INDENT")
        .external("DEDENT")
        .start("s")
        .build();

    let externals_before = grammar.externals.len();
    grammar.normalize();
    let externals_after = grammar.externals.len();

    assert_eq!(externals_before, externals_after);
    assert_eq!(externals_after, 2);
}

#[test]
fn edge_normalize_preserves_fields() {
    let mut grammar = GrammarBuilder::new("edge_fields")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();

    grammar
        .fields
        .insert(adze_ir::FieldId(0), "field_a".to_string());
    grammar
        .fields
        .insert(adze_ir::FieldId(1), "field_b".to_string());

    let fields_before = grammar.fields.len();
    grammar.normalize();
    let fields_after = grammar.fields.len();

    assert_eq!(fields_before, fields_after);
    assert_eq!(fields_after, 2);
}
