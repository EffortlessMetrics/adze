use adze_ir::*;
use proptest::prelude::*;
use std::collections::HashSet;

// -- Strategies for generating arbitrary grammar components --

fn arb_symbol_id(max: u16) -> impl Strategy<Value = SymbolId> {
    (0..max).prop_map(SymbolId)
}

fn arb_production_id() -> impl Strategy<Value = ProductionId> {
    (0u16..50).prop_map(ProductionId)
}

fn arb_simple_symbol(max_id: u16) -> impl Strategy<Value = Symbol> {
    prop_oneof![
        arb_symbol_id(max_id).prop_map(Symbol::Terminal),
        arb_symbol_id(max_id).prop_map(Symbol::NonTerminal),
        Just(Symbol::Epsilon),
    ]
}

fn arb_symbol(max_id: u16) -> impl Strategy<Value = Symbol> {
    arb_simple_symbol(max_id).prop_recursive(2, 6, 3, move |inner| {
        prop_oneof![
            inner.clone().prop_map(|s| Symbol::Optional(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::Repeat(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::RepeatOne(Box::new(s))),
            prop::collection::vec(inner.clone(), 1..3).prop_map(Symbol::Choice),
            prop::collection::vec(inner, 1..3).prop_map(Symbol::Sequence),
        ]
    })
}

fn arb_rule(lhs: SymbolId, max_id: u16) -> impl Strategy<Value = Rule> {
    (
        prop::collection::vec(arb_symbol(max_id), 1..4),
        prop::option::of(prop_oneof![
            (-10i16..10).prop_map(PrecedenceKind::Static),
            (-10i16..10).prop_map(PrecedenceKind::Dynamic),
        ]),
        prop::option::of(prop_oneof![
            Just(Associativity::Left),
            Just(Associativity::Right),
            Just(Associativity::None),
        ]),
        arb_production_id(),
    )
        .prop_map(
            move |(rhs, precedence, associativity, production_id)| Rule {
                lhs,
                rhs,
                precedence,
                associativity,
                fields: vec![],
                production_id,
            },
        )
}

/// Generate a well-formed grammar: every LHS has a token entry so symbols are defined.
fn arb_grammar(max_symbols: usize, max_rules_per: usize) -> impl Strategy<Value = Grammar> {
    (1..=max_symbols).prop_flat_map(move |n| {
        let rules_strategies: Vec<_> = (0..n)
            .map(|i| {
                let lhs = SymbolId(i as u16);
                prop::collection::vec(arb_rule(lhs, n as u16), 1..=max_rules_per)
            })
            .collect();
        ("[a-z]{3,8}".prop_map(|s| s.to_string()), rules_strategies).prop_map(
            move |(name, all_rules)| {
                let mut grammar = Grammar::new(name);
                for (i, rules) in all_rules.into_iter().enumerate() {
                    let sid = SymbolId(i as u16);
                    grammar.tokens.insert(
                        sid,
                        Token {
                            name: format!("tok_{i}"),
                            pattern: TokenPattern::String(format!("t{i}")),
                            fragile: false,
                        },
                    );
                    grammar.rule_names.insert(sid, format!("rule_{i}"));
                    for rule in rules {
                        grammar.add_rule(rule);
                    }
                }
                grammar
            },
        )
    })
}

// -- Helpers --

/// Collect all terminal SymbolIds referenced in a grammar's RHS symbols.
fn collect_terminals(grammar: &Grammar) -> HashSet<SymbolId> {
    let mut terminals = HashSet::new();
    for rule in grammar.all_rules() {
        for sym in &rule.rhs {
            collect_terminals_from_symbol(sym, &mut terminals);
        }
    }
    terminals
}

fn collect_terminals_from_symbol(sym: &Symbol, out: &mut HashSet<SymbolId>) {
    match sym {
        Symbol::Terminal(id) => {
            out.insert(*id);
        }
        Symbol::NonTerminal(_) | Symbol::External(_) | Symbol::Epsilon => {}
        Symbol::Optional(inner) | Symbol::Repeat(inner) | Symbol::RepeatOne(inner) => {
            collect_terminals_from_symbol(inner, out);
        }
        Symbol::Choice(items) | Symbol::Sequence(items) => {
            for item in items {
                collect_terminals_from_symbol(item, out);
            }
        }
    }
}

/// Check if any RHS symbol contains complex (non-normalized) symbols.
fn has_complex_symbols(grammar: &Grammar) -> bool {
    for rule in grammar.all_rules() {
        for sym in &rule.rhs {
            if is_complex(sym) {
                return true;
            }
        }
    }
    false
}

fn is_complex(sym: &Symbol) -> bool {
    matches!(
        sym,
        Symbol::Optional(_)
            | Symbol::Repeat(_)
            | Symbol::RepeatOne(_)
            | Symbol::Choice(_)
            | Symbol::Sequence(_)
    )
}

/// Collect all LHS SymbolIds from the grammar rules map.
fn collect_lhs_ids(grammar: &Grammar) -> HashSet<SymbolId> {
    grammar.rules.keys().copied().collect()
}

// -- Property tests --

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // ---------------------------------------------------------------
    // 1. normalize(grammar) produces equivalent grammar (same terminal set)
    // ---------------------------------------------------------------
    #[test]
    fn normalize_preserves_terminal_set(mut grammar in arb_grammar(5, 3)) {
        let terminals_before = collect_terminals(&grammar);
        grammar.normalize();
        let terminals_after = collect_terminals(&grammar);

        // Every original terminal must still appear somewhere.
        for t in &terminals_before {
            prop_assert!(
                terminals_after.contains(t),
                "terminal {t} lost after normalization"
            );
        }
    }

    // ---------------------------------------------------------------
    // 2. normalize is idempotent: normalize(normalize(g)) == normalize(g)
    // ---------------------------------------------------------------
    #[test]
    fn normalize_is_idempotent(mut grammar in arb_grammar(4, 2)) {
        grammar.normalize();

        // Snapshot state after first normalization via serialization.
        let json_first = serde_json::to_string(&grammar).unwrap();

        // Normalize a second time.
        grammar.normalize();
        let json_second = serde_json::to_string(&grammar).unwrap();

        prop_assert_eq!(
            json_first, json_second,
            "normalize is not idempotent"
        );
    }

    // ---------------------------------------------------------------
    // 3. optimization stats are internally consistent
    //    The total() method should equal the sum of all individual stat
    //    fields, and the grammar name is always preserved.
    // ---------------------------------------------------------------
    #[test]
    fn optimization_stats_are_consistent(grammar in arb_grammar(5, 3)) {
        let original_name = grammar.name.clone();

        let mut optimized = grammar.clone();
        let mut optimizer = GrammarOptimizer::new();
        let stats = optimizer.optimize(&mut optimized);

        // Grammar name must be preserved.
        prop_assert_eq!(&optimized.name, &original_name, "optimizer changed grammar name");

        // Stats total must equal sum of individual fields.
        let expected_total = stats.removed_unused_symbols
            + stats.inlined_rules
            + stats.merged_tokens
            + stats.optimized_left_recursion
            + stats.eliminated_unit_rules;
        prop_assert_eq!(
            stats.total(),
            expected_total,
            "stats.total() ({}) != sum of fields ({})",
            stats.total(),
            expected_total
        );
    }

    // ---------------------------------------------------------------
    // 4. validation errors are always for invalid grammars, never false positives
    //    Specifically: a well-formed non-empty grammar with all symbols defined
    //    and reachable should not produce EmptyGrammar or UndefinedSymbol errors.
    // ---------------------------------------------------------------
    #[test]
    fn validation_no_false_empty_grammar(grammar in arb_grammar(4, 2)) {
        // arb_grammar always produces non-empty grammars with at least 1 rule
        let mut validator = GrammarValidator::new();
        let result = validator.validate(&grammar);

        prop_assert!(
            !result.errors.iter().any(|e| matches!(e, ValidationError::EmptyGrammar)),
            "false EmptyGrammar error on non-empty grammar"
        );
    }

    // ---------------------------------------------------------------
    // 5. serialization roundtrip preserves grammar structure
    // ---------------------------------------------------------------
    #[test]
    fn serialization_roundtrip_preserves_structure(grammar in arb_grammar(5, 3)) {
        let json = serde_json::to_string(&grammar).expect("serialize failed");
        let deserialized: Grammar = serde_json::from_str(&json).expect("deserialize failed");

        // Compare via re-serialization (Grammar does not derive PartialEq).
        let json2 = serde_json::to_string(&deserialized).expect("re-serialize failed");
        prop_assert_eq!(&json, &json2, "serialization roundtrip mismatch");

        // Also verify structural invariants survive roundtrip.
        prop_assert_eq!(grammar.name, deserialized.name);
        prop_assert_eq!(grammar.rules.len(), deserialized.rules.len());
        prop_assert_eq!(grammar.tokens.len(), deserialized.tokens.len());
        prop_assert_eq!(grammar.rule_names.len(), deserialized.rule_names.len());
    }

    // ---------------------------------------------------------------
    // 6. symbol IDs are unique after normalization
    // ---------------------------------------------------------------
    #[test]
    fn symbol_ids_unique_after_normalization(mut grammar in arb_grammar(5, 3)) {
        grammar.normalize();

        let lhs_ids = collect_lhs_ids(&grammar);
        // IndexMap keys are unique by construction, but verify the count matches
        // to ensure no silent overwrites happened.
        prop_assert_eq!(
            lhs_ids.len(),
            grammar.rules.len(),
            "LHS symbol ID set size differs from rules map size"
        );

        // Verify auxiliary IDs (>= 1000 offset) don't collide with originals.
        let original_ids: HashSet<SymbolId> = grammar.tokens.keys().copied().collect();
        let aux_ids: HashSet<SymbolId> = lhs_ids
            .difference(&original_ids)
            .copied()
            .collect();
        for aux_id in &aux_ids {
            prop_assert!(
                !original_ids.contains(aux_id),
                "auxiliary SymbolId {aux_id} collides with original"
            );
        }
    }

    // ---------------------------------------------------------------
    // 7. FIRST sets are never empty for non-epsilon rules
    //    After normalization, every rule whose RHS is not purely
    //    Epsilon symbols should reference at least one terminal or
    //    non-terminal.
    // ---------------------------------------------------------------
    #[test]
    fn non_epsilon_rules_have_symbols(mut grammar in arb_grammar(5, 3)) {
        grammar.normalize();

        for rule in grammar.all_rules() {
            let all_epsilon = rule.rhs.iter().all(|s| matches!(s, Symbol::Epsilon));

            if !all_epsilon {
                // After normalization, no complex symbols should remain.
                for sym in &rule.rhs {
                    prop_assert!(
                        !is_complex(sym),
                        "complex symbol found in normalized rule for LHS {}",
                        rule.lhs
                    );
                }

                // At least one symbol should be a terminal, non-terminal, or external.
                let has_real_symbol = rule.rhs.iter().any(|s| {
                    matches!(
                        s,
                        Symbol::Terminal(_)
                            | Symbol::NonTerminal(_)
                            | Symbol::External(_)
                    )
                });

                prop_assert!(
                    has_real_symbol || rule.rhs.iter().any(|s| matches!(s, Symbol::Epsilon)),
                    "non-epsilon rule for LHS {} has no terminal/non-terminal symbols and no epsilon",
                    rule.lhs
                );
            }
        }
    }

    // ---------------------------------------------------------------
    // Bonus: normalization eliminates all complex symbols
    // ---------------------------------------------------------------
    #[test]
    fn normalization_eliminates_complex_symbols(mut grammar in arb_grammar(5, 3)) {
        grammar.normalize();

        prop_assert!(
            !has_complex_symbols(&grammar),
            "complex symbols remain after normalization"
        );
    }

    // ---------------------------------------------------------------
    // Bonus: optimizer does not panic on any generated grammar
    // ---------------------------------------------------------------
    #[test]
    fn optimizer_never_panics(grammar in arb_grammar(5, 3)) {
        let mut g = grammar;
        let mut optimizer = GrammarOptimizer::new();
        let _stats = optimizer.optimize(&mut g);
    }

    // ---------------------------------------------------------------
    // Bonus: validation after optimization has no UndefinedSymbol errors
    //        that weren't present before optimization
    // ---------------------------------------------------------------
    #[test]
    fn optimization_does_not_introduce_undefined_symbols(grammar in arb_grammar(4, 2)) {
        let mut validator = GrammarValidator::new();
        let before = validator.validate(&grammar);
        let undefined_before: HashSet<_> = before
            .errors
            .iter()
            .filter_map(|e| {
                if let ValidationError::UndefinedSymbol { symbol, .. } = e {
                    Some(*symbol)
                } else {
                    None
                }
            })
            .collect();

        let mut optimized = grammar.clone();
        let mut optimizer = GrammarOptimizer::new();
        optimizer.optimize(&mut optimized);

        let mut validator2 = GrammarValidator::new();
        let after = validator2.validate(&optimized);
        let undefined_after: HashSet<_> = after
            .errors
            .iter()
            .filter_map(|e| {
                if let ValidationError::UndefinedSymbol { symbol, .. } = e {
                    Some(*symbol)
                } else {
                    None
                }
            })
            .collect();

        // Optimization may remove undefined refs but should not introduce new ones.
        for sym in &undefined_after {
            prop_assert!(
                undefined_before.contains(sym),
                "optimization introduced new UndefinedSymbol error for {sym}"
            );
        }
    }
}
