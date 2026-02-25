use adze_bdd_grammar_analysis_core::{
    analyze_conflicts, count_multi_action_cells, resolve_shift_reduce_actions,
};
use adze_glr_core::{Action, ParseTable, StateId};
use adze_ir::{Grammar, ProductionId, Rule, RuleId, Symbol, SymbolId, Token, TokenPattern};

fn minimal_reduce_reduce_grammar() -> Grammar {
    let mut grammar = Grammar::new("fixture-minimal".to_string());

    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    grammar.rule_names.insert(SymbolId(10), "S".to_string());
    grammar.rules.insert(
        SymbolId(10),
        vec![Rule {
            lhs: SymbolId(10),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );

    grammar
}

#[test]
fn given_singleton_cells_then_conflict_analysis_reports_no_conflicts() {
    // Given
    let table = ParseTable {
        state_count: 1,
        symbol_count: 2,
        action_table: vec![vec![vec![Action::Shift(StateId(1))], vec![Action::Accept]]],
        ..Default::default()
    };

    // When
    let analysis = analyze_conflicts(&table);

    // Then
    assert_eq!(count_multi_action_cells(&table), 0);
    assert_eq!(analysis.total_conflicts, 0);
    assert_eq!(analysis.shift_reduce_conflicts, 0);
    assert_eq!(analysis.reduce_reduce_conflicts, 0);
}

#[test]
fn given_shift_reduce_cell_then_analysis_classifies_the_cell() {
    // Given
    let table = ParseTable {
        state_count: 1,
        symbol_count: 2,
        action_table: vec![vec![
            vec![
                Action::Shift(StateId(1)),
                Action::Reduce(RuleId(0)),
                Action::Accept,
            ],
            vec![],
        ]],
        ..Default::default()
    };

    // When
    let analysis = analyze_conflicts(&table);

    // Then
    assert_eq!(analysis.total_conflicts, 1);
    assert_eq!(analysis.shift_reduce_conflicts, 1);
    assert_eq!(analysis.reduce_reduce_conflicts, 0);
    assert_eq!(analysis.conflict_details[0].0, 0);
    assert_eq!(analysis.conflict_details[0].1, 0);
}

#[test]
fn given_reduce_reduce_cell_then_analysis_marks_reduce_reduce() {
    // Given
    let table = ParseTable {
        state_count: 1,
        symbol_count: 1,
        action_table: vec![vec![vec![
            Action::Reduce(RuleId(0)),
            Action::Reduce(RuleId(1)),
        ]]],
        ..Default::default()
    };

    // When
    let analysis = analyze_conflicts(&table);

    // Then
    assert_eq!(analysis.total_conflicts, 1);
    assert_eq!(analysis.shift_reduce_conflicts, 0);
    assert_eq!(analysis.reduce_reduce_conflicts, 1);
}

#[test]
fn given_generic_conflict_then_resolve_shift_reduce_returns_fork_shape() {
    // Given
    let grammar = minimal_reduce_reduce_grammar();

    // When
    let actions = resolve_shift_reduce_actions(&grammar, SymbolId(1), RuleId(0));

    // Then
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        Action::Fork(inner) => {
            assert_eq!(inner.len(), 2);
            assert!(
                inner
                    .iter()
                    .any(|action| matches!(action, Action::Shift(_)))
            );
            assert!(
                inner
                    .iter()
                    .any(|action| matches!(action, Action::Reduce(_)))
            );
        }
        other => panic!("expected fork after conflict resolution, got {other:?}"),
    }
}
