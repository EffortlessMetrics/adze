//! Comprehensive tests for GLR error types and display formatting.
//!
//! Covers: GLRError variants, TableError variants, Conflict/ConflictType
//! Debug/Display, error string content, grammar-driven error paths,
//! success vs error results, multiple errors, and edge cases.

use adze_glr_core::{
    Action, Conflict, ConflictResolver, ConflictType, FirstFollowSets, GLRError, GlrError,
    GlrResult, ItemSetCollection, TableError, build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use std::error::Error;

// ===========================================================================
// Helpers
// ===========================================================================

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn ambiguous_grammar() -> Grammar {
    // E → a | E E — inherently ambiguous
    let mut grammar = Grammar::new("ambig".into());
    let a = SymbolId(1);
    let e = SymbolId(10);
    grammar.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(e, "E".into());
    grammar.rules.insert(
        e,
        vec![
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::NonTerminal(e), Symbol::NonTerminal(e)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    grammar
}

// ===========================================================================
// 1. Build error messages for invalid grammars (8 tests)
// ===========================================================================

#[test]
fn build_error_empty_grammar_no_start() {
    let grammar = Grammar::new("empty".into());
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let result = build_lr1_automaton(&grammar, &ff);
    assert!(result.is_err(), "empty grammar must fail");
}

#[test]
fn build_error_grammar_error_display_contains_grammar() {
    let err = GLRError::GrammarError(GrammarError::InvalidFieldOrdering);
    let msg = err.to_string();
    assert!(msg.contains("Grammar error"), "got: {msg}");
}

#[test]
fn build_error_unresolved_symbol_shows_id() {
    let err = GLRError::GrammarError(GrammarError::UnresolvedSymbol(SymbolId(77)));
    let msg = err.to_string();
    assert!(msg.contains("77"), "got: {msg}");
}

#[test]
fn build_error_unresolved_external_symbol_shows_id() {
    let err = GLRError::GrammarError(GrammarError::UnresolvedExternalSymbol(SymbolId(33)));
    let msg = err.to_string();
    assert!(msg.contains("33"), "got: {msg}");
}

#[test]
fn build_error_conflict_error_in_grammar_error() {
    let err = GLRError::GrammarError(GrammarError::ConflictError("rule clash".into()));
    let msg = err.to_string();
    assert!(msg.contains("rule clash"), "got: {msg}");
}

#[test]
fn build_error_invalid_precedence_in_grammar_error() {
    let err = GLRError::GrammarError(GrammarError::InvalidPrecedence("bad prec".into()));
    let msg = err.to_string();
    assert!(msg.contains("bad prec"), "got: {msg}");
}

#[test]
fn build_error_state_machine_error() {
    let err = GLRError::StateMachine("too many states".into());
    let msg = err.to_string();
    assert!(
        msg.contains("State machine generation failed"),
        "got: {msg}"
    );
    assert!(msg.contains("too many states"), "got: {msg}");
}

#[test]
fn build_error_conflict_resolution_error() {
    let err = GLRError::ConflictResolution("unresolved shift/reduce".into());
    let msg = err.to_string();
    assert!(msg.contains("Conflict resolution failed"), "got: {msg}");
    assert!(msg.contains("unresolved shift/reduce"), "got: {msg}");
}

// ===========================================================================
// 2. Conflict Display format (8 tests)
// ===========================================================================

#[test]
fn conflict_debug_shows_state_field() {
    let c = Conflict {
        state: StateId(5),
        symbol: SymbolId(2),
        actions: vec![Action::Shift(StateId(3)), Action::Reduce(RuleId(1))],
        conflict_type: ConflictType::ShiftReduce,
    };
    let dbg = format!("{c:?}");
    assert!(dbg.contains("state:"), "got: {dbg}");
}

#[test]
fn conflict_debug_shows_symbol_field() {
    let c = Conflict {
        state: StateId(0),
        symbol: SymbolId(42),
        actions: vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))],
        conflict_type: ConflictType::ReduceReduce,
    };
    let dbg = format!("{c:?}");
    assert!(dbg.contains("symbol:"), "got: {dbg}");
}

#[test]
fn conflict_debug_shows_shift_reduce_type() {
    let c = Conflict {
        state: StateId(1),
        symbol: SymbolId(3),
        actions: vec![Action::Shift(StateId(4)), Action::Reduce(RuleId(0))],
        conflict_type: ConflictType::ShiftReduce,
    };
    let dbg = format!("{c:?}");
    assert!(dbg.contains("ShiftReduce"), "got: {dbg}");
}

#[test]
fn conflict_debug_shows_reduce_reduce_type() {
    let c = Conflict {
        state: StateId(2),
        symbol: SymbolId(1),
        actions: vec![Action::Reduce(RuleId(5)), Action::Reduce(RuleId(6))],
        conflict_type: ConflictType::ReduceReduce,
    };
    let dbg = format!("{c:?}");
    assert!(dbg.contains("ReduceReduce"), "got: {dbg}");
}

#[test]
fn conflict_debug_shows_actions_vec() {
    let c = Conflict {
        state: StateId(0),
        symbol: SymbolId(0),
        actions: vec![Action::Shift(StateId(10)), Action::Reduce(RuleId(3))],
        conflict_type: ConflictType::ShiftReduce,
    };
    let dbg = format!("{c:?}");
    assert!(dbg.contains("Shift"), "got: {dbg}");
    assert!(dbg.contains("Reduce"), "got: {dbg}");
}

#[test]
fn conflict_debug_with_accept_action() {
    let c = Conflict {
        state: StateId(0),
        symbol: SymbolId(0),
        actions: vec![Action::Accept, Action::Reduce(RuleId(1))],
        conflict_type: ConflictType::ReduceReduce,
    };
    let dbg = format!("{c:?}");
    assert!(dbg.contains("Accept"), "got: {dbg}");
}

#[test]
fn conflict_debug_with_many_actions() {
    let c = Conflict {
        state: StateId(7),
        symbol: SymbolId(4),
        actions: vec![
            Action::Shift(StateId(1)),
            Action::Reduce(RuleId(2)),
            Action::Reduce(RuleId(3)),
        ],
        conflict_type: ConflictType::ShiftReduce,
    };
    let dbg = format!("{c:?}");
    // Should contain all three action variants
    assert!(dbg.contains("Shift(StateId(1))"), "got: {dbg}");
    assert!(dbg.contains("Reduce(RuleId(2))"), "got: {dbg}");
    assert!(dbg.contains("Reduce(RuleId(3))"), "got: {dbg}");
}

#[test]
fn conflict_type_debug_variants_are_distinct() {
    let sr = format!("{:?}", ConflictType::ShiftReduce);
    let rr = format!("{:?}", ConflictType::ReduceReduce);
    assert_ne!(sr, rr);
    assert_eq!(sr, "ShiftReduce");
    assert_eq!(rr, "ReduceReduce");
}

// ===========================================================================
// 3. Conflict Debug format (5 tests)
// ===========================================================================

#[test]
fn conflict_debug_is_not_empty() {
    let c = Conflict {
        state: StateId(0),
        symbol: SymbolId(0),
        actions: vec![Action::Shift(StateId(1))],
        conflict_type: ConflictType::ShiftReduce,
    };
    let dbg = format!("{c:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn conflict_debug_contains_struct_name() {
    let c = Conflict {
        state: StateId(0),
        symbol: SymbolId(1),
        actions: vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(0))],
        conflict_type: ConflictType::ShiftReduce,
    };
    let dbg = format!("{c:?}");
    assert!(dbg.contains("Conflict"), "got: {dbg}");
}

#[test]
fn conflict_debug_contains_conflict_type_field() {
    let c = Conflict {
        state: StateId(3),
        symbol: SymbolId(5),
        actions: vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))],
        conflict_type: ConflictType::ReduceReduce,
    };
    let dbg = format!("{c:?}");
    assert!(dbg.contains("conflict_type:"), "got: {dbg}");
}

#[test]
fn conflict_debug_shows_state_id_value() {
    let c = Conflict {
        state: StateId(99),
        symbol: SymbolId(0),
        actions: vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
        conflict_type: ConflictType::ShiftReduce,
    };
    let dbg = format!("{c:?}");
    assert!(dbg.contains("99"), "got: {dbg}");
}

#[test]
fn conflict_debug_shows_symbol_id_value() {
    let c = Conflict {
        state: StateId(0),
        symbol: SymbolId(255),
        actions: vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
        conflict_type: ConflictType::ShiftReduce,
    };
    let dbg = format!("{c:?}");
    assert!(dbg.contains("255"), "got: {dbg}");
}

// ===========================================================================
// 4. Error string content (8 tests)
// ===========================================================================

#[test]
fn glr_error_conflict_resolution_exact_format() {
    let err = GLRError::ConflictResolution("ambiguous".into());
    assert_eq!(err.to_string(), "Conflict resolution failed: ambiguous");
}

#[test]
fn glr_error_state_machine_exact_format() {
    let err = GLRError::StateMachine("overflow".into());
    assert_eq!(err.to_string(), "State machine generation failed: overflow");
}

#[test]
fn glr_error_complex_symbols_exact_format() {
    let err = GLRError::ComplexSymbolsNotNormalized {
        operation: "LR(1)".into(),
    };
    assert_eq!(
        err.to_string(),
        "Complex symbols must be normalized before LR(1)"
    );
}

#[test]
fn glr_error_expected_simple_exact_format() {
    let err = GLRError::ExpectedSimpleSymbol {
        expected: "terminal".into(),
    };
    assert_eq!(
        err.to_string(),
        "Expected terminal symbol, found complex symbol"
    );
}

#[test]
fn glr_error_invalid_symbol_state_exact_format() {
    let err = GLRError::InvalidSymbolState {
        operation: "goto".into(),
    };
    assert_eq!(err.to_string(), "Invalid symbol state during goto");
}

#[test]
fn table_error_eof_is_error_exact_format() {
    let err = TableError::EofIsError;
    assert_eq!(err.to_string(), "EOF symbol collides with ERROR");
}

#[test]
fn table_error_eof_missing_from_index_exact_format() {
    let err = TableError::EofMissingFromIndex;
    assert_eq!(err.to_string(), "EOF not present in symbol_to_index");
}

#[test]
fn table_error_eof_not_sentinel_exact_format() {
    let err = TableError::EofNotSentinel {
        eof: 1,
        token_count: 5,
        external_count: 2,
    };
    let msg = err.to_string();
    assert!(msg.contains("EOF: 1"), "got: {msg}");
    assert!(msg.contains("tokens: 5"), "got: {msg}");
    assert!(msg.contains("externals: 2"), "got: {msg}");
}

// ===========================================================================
// 5. Error with specific grammar patterns (8 tests)
// ===========================================================================

#[test]
fn grammar_with_single_token_builds_ok() {
    let grammar = simple_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let result = build_lr1_automaton(&grammar, &ff);
    assert!(result.is_ok(), "simple grammar should succeed");
}

#[test]
fn grammar_with_two_alternatives_builds_ok() {
    let grammar = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let result = build_lr1_automaton(&grammar, &ff);
    assert!(result.is_ok(), "two alternatives should build OK");
}

#[test]
fn ambiguous_grammar_detects_conflicts() {
    let grammar = ambiguous_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &grammar, &ff);
    assert!(
        !resolver.conflicts.is_empty(),
        "ambiguous grammar should have conflicts"
    );
}

#[test]
fn ambiguous_grammar_conflict_type_is_shift_reduce() {
    let grammar = ambiguous_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &grammar, &ff);
    let has_sr = resolver
        .conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::ShiftReduce);
    assert!(has_sr, "expected shift/reduce conflict");
}

#[test]
fn ambiguous_grammar_conflict_has_multiple_actions() {
    let grammar = ambiguous_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &grammar, &ff);
    for conflict in &resolver.conflicts {
        assert!(
            conflict.actions.len() >= 2,
            "conflict must have at least 2 actions, got {}",
            conflict.actions.len()
        );
    }
}

#[test]
fn left_recursive_grammar_builds_ok() {
    let grammar = GrammarBuilder::new("leftrec")
        .token("a", "a")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "a"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let result = build_lr1_automaton(&grammar, &ff);
    assert!(result.is_ok(), "left-recursive grammar should build");
}

#[test]
fn right_recursive_grammar_builds_ok() {
    let grammar = GrammarBuilder::new("rightrec")
        .token("a", "a")
        .token("+", "+")
        .rule("expr", vec!["a", "+", "expr"])
        .rule("expr", vec!["a"])
        .start("expr")
        .build();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let result = build_lr1_automaton(&grammar, &ff);
    assert!(result.is_ok(), "right-recursive grammar should build");
}

#[test]
fn chained_nonterminals_build_ok() {
    let grammar = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("start", vec!["middle"])
        .rule("middle", vec!["leaf"])
        .rule("leaf", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let result = build_lr1_automaton(&grammar, &ff);
    assert!(result.is_ok(), "chained nonterminals should build");
}

// ===========================================================================
// 6. Success vs error result (5 tests)
// ===========================================================================

#[test]
fn ok_result_has_positive_state_count() {
    let grammar = simple_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    assert!(table.state_count > 0);
}

#[test]
fn ok_result_contains_accept_action() {
    let grammar = simple_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let eof = table.eof();
    let has_accept = (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), eof)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    });
    assert!(has_accept, "table must contain Accept");
}

#[test]
fn err_result_from_empty_grammar_is_glr_error() {
    let grammar = Grammar::new("nope".into());
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let result = build_lr1_automaton(&grammar, &ff);
    assert!(result.is_err());
}

#[test]
fn glr_result_type_alias_works() {
    fn produces_result() -> GlrResult<()> {
        Err(GLRError::StateMachine("test alias".into()))
    }
    let result = produces_result();
    assert!(result.is_err());
}

#[test]
fn glr_error_alias_matches_glrerror() {
    let e1: GLRError = GLRError::StateMachine("a".into());
    let e2: GlrError = GlrError::StateMachine("b".into());
    // Both are the same type via type alias
    assert_eq!(
        std::mem::size_of_val(&e1),
        std::mem::size_of_val(&e2),
        "GlrError and GLRError should be the same type"
    );
}

// ===========================================================================
// 7. Multiple errors (5 tests)
// ===========================================================================

#[test]
fn multiple_glr_error_variants_have_distinct_messages() {
    let errors: Vec<GLRError> = vec![
        GLRError::ConflictResolution("a".into()),
        GLRError::StateMachine("b".into()),
        GLRError::ComplexSymbolsNotNormalized {
            operation: "c".into(),
        },
        GLRError::ExpectedSimpleSymbol {
            expected: "d".into(),
        },
        GLRError::InvalidSymbolState {
            operation: "e".into(),
        },
    ];
    let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    // Every message is unique
    for (i, msg) in messages.iter().enumerate() {
        for (j, other) in messages.iter().enumerate() {
            if i != j {
                assert_ne!(msg, other, "messages {i} and {j} collide");
            }
        }
    }
}

#[test]
fn multiple_table_errors_have_distinct_messages() {
    let errors: Vec<TableError> = vec![
        TableError::EofIsError,
        TableError::EofMissingFromIndex,
        TableError::EofParityMismatch(7),
        TableError::EofNotSentinel {
            eof: 0,
            token_count: 3,
            external_count: 1,
        },
    ];
    let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    for (i, msg) in messages.iter().enumerate() {
        for (j, other) in messages.iter().enumerate() {
            if i != j {
                assert_ne!(msg, other, "messages {i} and {j} collide");
            }
        }
    }
}

#[test]
fn multiple_conflicts_from_ambiguous_grammar_have_valid_states() {
    let grammar = ambiguous_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &grammar, &ff);
    for conflict in &resolver.conflicts {
        assert!(
            (conflict.state.0 as usize) < collection.sets.len(),
            "conflict state {} out of range",
            conflict.state.0
        );
    }
}

#[test]
fn multiple_errors_can_be_collected_into_vec() {
    let errors: Vec<Box<dyn Error>> = vec![
        Box::new(GLRError::StateMachine("one".into())),
        Box::new(GLRError::ConflictResolution("two".into())),
        Box::new(GLRError::ComplexSymbolsNotNormalized {
            operation: "three".into(),
        }),
    ];
    assert_eq!(errors.len(), 3);
    for err in &errors {
        assert!(!err.to_string().is_empty());
    }
}

#[test]
fn multiple_conflicts_debug_all_non_empty() {
    let conflicts = [
        Conflict {
            state: StateId(0),
            symbol: SymbolId(1),
            actions: vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
            conflict_type: ConflictType::ShiftReduce,
        },
        Conflict {
            state: StateId(1),
            symbol: SymbolId(2),
            actions: vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))],
            conflict_type: ConflictType::ReduceReduce,
        },
    ];
    for (i, c) in conflicts.iter().enumerate() {
        let dbg = format!("{c:?}");
        assert!(!dbg.is_empty(), "conflict {i} debug was empty");
    }
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn table_error_eof_parity_mismatch_shows_state() {
    let err = TableError::EofParityMismatch(42);
    let msg = err.to_string();
    assert!(msg.contains("42"), "got: {msg}");
    assert!(msg.contains("parity mismatch"), "got: {msg}");
}

#[test]
fn table_error_eof_not_sentinel_with_zeros() {
    let err = TableError::EofNotSentinel {
        eof: 0,
        token_count: 0,
        external_count: 0,
    };
    let msg = err.to_string();
    assert!(msg.contains("EOF: 0"), "got: {msg}");
    assert!(msg.contains("tokens: 0"), "got: {msg}");
}

#[test]
fn glr_error_table_validation_wraps_table_error() {
    let inner = TableError::EofIsError;
    let err = GLRError::TableValidation(inner);
    let msg = err.to_string();
    assert!(msg.contains("Table validation failed"), "got: {msg}");
    assert!(msg.contains("EOF symbol collides with ERROR"), "got: {msg}");
}

#[test]
fn glr_error_from_grammar_error_conversion() {
    let ge = GrammarError::InvalidFieldOrdering;
    let err: GLRError = ge.into();
    assert!(matches!(
        err,
        GLRError::GrammarError(GrammarError::InvalidFieldOrdering)
    ));
}

#[test]
fn glr_error_is_std_error_trait_object() {
    let err: Box<dyn Error> = Box::new(GLRError::StateMachine("dyn".into()));
    assert!(err.to_string().contains("dyn"));
}

#[test]
fn glr_error_source_for_grammar_error_is_some() {
    let err = GLRError::GrammarError(GrammarError::InvalidFieldOrdering);
    // GrammarError is #[from], so source() returns the inner error
    assert!(err.source().is_some());
}

#[test]
fn glr_error_source_for_non_from_variant_is_none() {
    let err = GLRError::StateMachine("plain".into());
    assert!(err.source().is_none());
}

#[test]
fn conflict_with_empty_actions_debug() {
    let c = Conflict {
        state: StateId(0),
        symbol: SymbolId(0),
        actions: vec![],
        conflict_type: ConflictType::ShiftReduce,
    };
    let dbg = format!("{c:?}");
    assert!(dbg.contains("actions: []"), "got: {dbg}");
}

// ===========================================================================
// Additional edge cases and Display tests (bonus tests)
// ===========================================================================

#[test]
fn conflict_type_clone_eq() {
    let a = ConflictType::ShiftReduce;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn conflict_type_ne() {
    assert_ne!(ConflictType::ShiftReduce, ConflictType::ReduceReduce);
}

#[test]
fn table_error_debug_eof_is_error() {
    let dbg = format!("{:?}", TableError::EofIsError);
    assert!(dbg.contains("EofIsError"), "got: {dbg}");
}

#[test]
fn table_error_debug_eof_not_sentinel() {
    let err = TableError::EofNotSentinel {
        eof: 5,
        token_count: 10,
        external_count: 3,
    };
    let dbg = format!("{err:?}");
    assert!(dbg.contains("EofNotSentinel"), "got: {dbg}");
    assert!(dbg.contains("5"), "got: {dbg}");
}

#[test]
fn table_error_debug_eof_parity_mismatch() {
    let dbg = format!("{:?}", TableError::EofParityMismatch(100));
    assert!(dbg.contains("EofParityMismatch"), "got: {dbg}");
    assert!(dbg.contains("100"), "got: {dbg}");
}

#[test]
fn table_error_debug_eof_missing_from_index() {
    let dbg = format!("{:?}", TableError::EofMissingFromIndex);
    assert!(dbg.contains("EofMissingFromIndex"), "got: {dbg}");
}

#[test]
fn glr_error_debug_all_variants() {
    let variants: Vec<GLRError> = vec![
        GLRError::GrammarError(GrammarError::InvalidFieldOrdering),
        GLRError::ConflictResolution("cr".into()),
        GLRError::StateMachine("sm".into()),
        GLRError::TableValidation(TableError::EofIsError),
        GLRError::ComplexSymbolsNotNormalized {
            operation: "op".into(),
        },
        GLRError::ExpectedSimpleSymbol {
            expected: "ex".into(),
        },
        GLRError::InvalidSymbolState {
            operation: "is".into(),
        },
    ];
    for (i, err) in variants.iter().enumerate() {
        let dbg = format!("{err:?}");
        assert!(!dbg.is_empty(), "variant {i} had empty debug output");
    }
}

#[test]
fn glr_error_display_all_variants_non_empty() {
    let variants: Vec<GLRError> = vec![
        GLRError::GrammarError(GrammarError::InvalidFieldOrdering),
        GLRError::ConflictResolution("cr".into()),
        GLRError::StateMachine("sm".into()),
        GLRError::TableValidation(TableError::EofIsError),
        GLRError::ComplexSymbolsNotNormalized {
            operation: "op".into(),
        },
        GLRError::ExpectedSimpleSymbol {
            expected: "ex".into(),
        },
        GLRError::InvalidSymbolState {
            operation: "is".into(),
        },
    ];
    for (i, err) in variants.iter().enumerate() {
        let msg = err.to_string();
        assert!(!msg.is_empty(), "variant {i} had empty display output");
    }
}

#[test]
fn conflict_resolver_no_conflicts_for_simple_grammar() {
    let grammar = simple_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &grammar, &ff);
    assert!(
        resolver.conflicts.is_empty(),
        "simple grammar should have no conflicts, got {}",
        resolver.conflicts.len()
    );
}

#[test]
fn conflict_clone_preserves_fields() {
    let original = Conflict {
        state: StateId(10),
        symbol: SymbolId(20),
        actions: vec![Action::Shift(StateId(30)), Action::Reduce(RuleId(40))],
        conflict_type: ConflictType::ShiftReduce,
    };
    let cloned = original.clone();
    assert_eq!(format!("{original:?}"), format!("{cloned:?}"));
}

#[test]
fn conflict_resolver_clone_preserves_conflicts() {
    let resolver = ConflictResolver {
        conflicts: vec![Conflict {
            state: StateId(0),
            symbol: SymbolId(1),
            actions: vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(0))],
            conflict_type: ConflictType::ShiftReduce,
        }],
    };
    let cloned = resolver.clone();
    assert_eq!(resolver.conflicts.len(), cloned.conflicts.len());
}

#[test]
fn table_error_eof_not_sentinel_large_values() {
    let err = TableError::EofNotSentinel {
        eof: u16::MAX,
        token_count: u32::MAX,
        external_count: u32::MAX,
    };
    let msg = err.to_string();
    assert!(msg.contains(&u16::MAX.to_string()), "got: {msg}");
    assert!(msg.contains(&u32::MAX.to_string()), "got: {msg}");
}
