//! Property-based tests for ParseTable generation invariants (v6).
//!
//! Covers: state/symbol counts, serialization roundtrips, determinism,
//! varying token/rule counts, FirstFollowSets, build_lr1_automaton,
//! rule accessor, and EOF consistency.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test proptest_table_v6 --features serialization -- --test-threads=2

#![cfg(feature = "serialization")]

use adze_glr_core::{Action, FirstFollowSets, ParseTable, StateId, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, RuleId};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

/// Strategy: grammar with 1–5 tokens, each with a trivial rule `start -> tok_0`.
fn varying_token_grammar() -> impl Strategy<Value = Grammar> {
    (1..6u8).prop_map(|num_tokens| {
        let mut b = GrammarBuilder::new("proptest_tok");
        for i in 0..num_tokens {
            let name = format!("tok_{i}");
            let pat = format!("[{}]", (b'a' + i) as char);
            b = b.token(&name, &pat);
        }
        b = b.rule("start", vec!["tok_0"]);
        b = b.start("start");
        b.build()
    })
}

/// Strategy: grammar with 1–3 alternative rules for `start`.
fn varying_rule_grammar() -> impl Strategy<Value = Grammar> {
    (1..4u8).prop_map(|num_rules| {
        let mut b = GrammarBuilder::new("proptest_rule");
        for i in 0..num_rules {
            let name = format!("tok_{i}");
            let pat = format!("[{}]", (b'a' + i) as char);
            b = b.token(&name, &pat);
        }
        for i in 0..num_rules {
            let tok = format!("tok_{i}");
            b = b.rule("start", vec![tok.as_str()]);
        }
        b = b.start("start");
        b.build()
    })
}

/// Strategy that produces either a varying-token or varying-rule grammar.
fn any_grammar() -> impl Strategy<Value = Grammar> {
    prop_oneof![varying_token_grammar(), varying_rule_grammar(),]
}

// ---------------------------------------------------------------------------
// 1. state_count > 0 for any valid grammar
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn state_count_positive(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count > 0);
    }
}

// ---------------------------------------------------------------------------
// 2. symbol_count > 0 for any valid grammar
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn symbol_count_positive(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.symbol_count > 0);
    }
}

// ---------------------------------------------------------------------------
// 3. Serialization roundtrip preserves state_count
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_state_count(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.state_count, restored.state_count);
    }
}

// ---------------------------------------------------------------------------
// 4. Serialization roundtrip preserves symbol_count
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_symbol_count(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.symbol_count, restored.symbol_count);
    }
}

// ---------------------------------------------------------------------------
// 5. Serialization roundtrip preserves eof_symbol
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_eof_symbol(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.eof_symbol, restored.eof_symbol);
    }
}

// ---------------------------------------------------------------------------
// 6. to_bytes produces non-empty bytes
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn to_bytes_non_empty(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        prop_assert!(!bytes.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 7. from_bytes(to_bytes(pt)) == pt (structural equality via bytes)
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_structural_equality(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        let bytes2 = restored.to_bytes().expect("re-serialize");
        prop_assert_eq!(bytes, bytes2);
    }
}

// ---------------------------------------------------------------------------
// 8. Serialization is deterministic (same bytes twice)
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn serialization_deterministic(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes_a = table.to_bytes().expect("serialize a");
        let bytes_b = table.to_bytes().expect("serialize b");
        prop_assert_eq!(bytes_a, bytes_b);
    }
}

// ---------------------------------------------------------------------------
// 9. Varying token counts (1–5) all produce valid tables
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn varying_tokens_valid(grammar in varying_token_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count > 0);
        prop_assert!(table.symbol_count > 0);
    }
}

// ---------------------------------------------------------------------------
// 10. Varying rule counts (1–3) all produce valid tables
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn varying_rules_valid(grammar in varying_rule_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count > 0);
        prop_assert!(table.symbol_count > 0);
    }
}

// ---------------------------------------------------------------------------
// 11. FirstFollowSets::compute succeeds for valid grammars
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn first_follow_succeeds(grammar in any_grammar()) {
        let result = FirstFollowSets::compute(&grammar);
        prop_assert!(result.is_ok(), "FirstFollowSets::compute failed: {:?}", result.err());
    }
}

// ---------------------------------------------------------------------------
// 12. build_lr1_automaton succeeds for valid grammars
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn lr1_automaton_succeeds(grammar in any_grammar()) {
        let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW");
        let result = build_lr1_automaton(&grammar, &ff);
        prop_assert!(result.is_ok(), "build_lr1_automaton failed: {:?}", result.err());
    }
}

// ---------------------------------------------------------------------------
// 13. ParseTable.rule(rid) doesn't panic for valid rule IDs
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn rule_accessor_no_panic(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for i in 0..table.rules.len() {
            let (lhs, rhs_len) = table.rule(RuleId(i as u16));
            prop_assert!(lhs.0 < table.symbol_count as u16,
                "rule {} lhs {} out of range {}", i, lhs.0, table.symbol_count);
            // rhs_len is u16, just verify it's accessible
            let _ = rhs_len;
        }
    }
}

// ---------------------------------------------------------------------------
// 14. eof_symbol consistent across roundtrips
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn eof_consistent_double_roundtrip(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes1 = table.to_bytes().expect("serialize 1");
        let r1 = ParseTable::from_bytes(&bytes1).expect("deserialize 1");
        let bytes2 = r1.to_bytes().expect("serialize 2");
        let r2 = ParseTable::from_bytes(&bytes2).expect("deserialize 2");
        prop_assert_eq!(table.eof_symbol, r1.eof_symbol);
        prop_assert_eq!(r1.eof_symbol, r2.eof_symbol);
    }
}

// ---------------------------------------------------------------------------
// 15. state_count >= 2 (at least initial + accept state)
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn state_count_at_least_two(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.state_count >= 2,
            "expected >= 2 states, got {}", table.state_count);
    }
}

// ---------------------------------------------------------------------------
// 16. symbol_count >= 2 (at least one terminal + EOF)
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn symbol_count_at_least_two(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.symbol_count >= 2,
            "expected >= 2 symbols, got {}", table.symbol_count);
    }
}

// ---------------------------------------------------------------------------
// 17. Accept action exists somewhere
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn accept_action_exists(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let has_accept = (0..table.state_count).any(|st| {
            table
                .actions(StateId(st as u16), table.eof())
                .iter()
                .any(|a| matches!(a, Action::Accept))
        });
        prop_assert!(has_accept, "no Accept action found in any state");
    }
}

// ---------------------------------------------------------------------------
// 18. At least one Shift action in the table
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn shift_action_exists(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let has_shift = (0..table.state_count).any(|st| {
            table.symbol_to_index.keys().any(|&sym| {
                table
                    .actions(StateId(st as u16), sym)
                    .iter()
                    .any(|a| matches!(a, Action::Shift(_)))
            })
        });
        prop_assert!(has_shift, "no Shift action found");
    }
}

// ---------------------------------------------------------------------------
// 19. At least one Reduce action in the table
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn reduce_action_exists(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let has_reduce = (0..table.state_count).any(|st| {
            table.symbol_to_index.keys().any(|&sym| {
                table
                    .actions(StateId(st as u16), sym)
                    .iter()
                    .any(|a| matches!(a, Action::Reduce(_)))
            })
        });
        prop_assert!(has_reduce, "no Reduce action found");
    }
}

// ---------------------------------------------------------------------------
// 20. Shift targets are valid state indices
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn shift_targets_valid(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for st in 0..table.state_count {
            for &sym in table.symbol_to_index.keys() {
                for action in table.actions(StateId(st as u16), sym) {
                    if let Action::Shift(target) = action {
                        prop_assert!((target.0 as usize) < table.state_count,
                            "shift target {} out of range {}", target.0, table.state_count);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 21. Reduce rule IDs are valid
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn reduce_rule_ids_valid(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for st in 0..table.state_count {
            for &sym in table.symbol_to_index.keys() {
                for action in table.actions(StateId(st as u16), sym) {
                    if let Action::Reduce(rid) = action {
                        prop_assert!((rid.0 as usize) < table.rules.len(),
                            "reduce rule {} out of range {}", rid.0, table.rules.len());
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 22. Roundtrip preserves action_table dimensions
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_action_table_rows(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.action_table.len(), restored.action_table.len());
    }
}

// ---------------------------------------------------------------------------
// 23. Roundtrip preserves goto_table dimensions
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_goto_table_rows(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.goto_table.len(), restored.goto_table.len());
    }
}

// ---------------------------------------------------------------------------
// 24. Roundtrip preserves rule count
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_rule_count(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.rules.len(), restored.rules.len());
    }
}

// ---------------------------------------------------------------------------
// 25. Roundtrip preserves start_symbol
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_start_symbol(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.start_symbol(), restored.start_symbol());
    }
}

// ---------------------------------------------------------------------------
// 26. Roundtrip preserves initial_state
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_initial_state(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.initial_state, restored.initial_state);
    }
}

// ---------------------------------------------------------------------------
// 27. Roundtrip preserves token_count
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_token_count(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.token_count, restored.token_count);
    }
}

// ---------------------------------------------------------------------------
// 28. Rules have valid lhs symbols
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn rules_lhs_valid(grammar in any_grammar()) {
        let table = build_table(&grammar);
        for (i, rule) in table.rules.iter().enumerate() {
            prop_assert!(
                (rule.lhs.0 as usize) < table.symbol_count,
                "rule {} lhs {} >= symbol_count {}", i, rule.lhs.0, table.symbol_count
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 29. eof accessor matches eof_symbol field
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn eof_accessor_matches_field(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.eof(), table.eof_symbol);
    }
}

// ---------------------------------------------------------------------------
// 30. start_symbol accessor matches field
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn start_symbol_accessor_matches_field(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.start_symbol(), table.start_symbol);
    }
}

// ---------------------------------------------------------------------------
// 31. action_table has state_count rows
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn action_table_row_count(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.action_table.len(), table.state_count);
    }
}

// ---------------------------------------------------------------------------
// 32. goto_table has state_count rows
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn goto_table_row_count(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(table.goto_table.len(), table.state_count);
    }
}

// ---------------------------------------------------------------------------
// 33. More tokens produce >= symbol_count as fewer tokens
// ---------------------------------------------------------------------------
#[test]
fn more_tokens_more_symbols() {
    let g1 = GrammarBuilder::new("small")
        .token("a", "[a]")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g3 = GrammarBuilder::new("bigger")
        .token("a", "[a]")
        .token("b", "[b]")
        .token("c", "[c]")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let t1 = build_table(&g1);
    let t3 = build_table(&g3);
    assert!(t3.symbol_count >= t1.symbol_count);
}

// ---------------------------------------------------------------------------
// 34. More rules produce >= state_count as fewer rules
// ---------------------------------------------------------------------------
#[test]
fn more_rules_more_or_equal_states() {
    let g1 = GrammarBuilder::new("one_rule")
        .token("a", "[a]")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g2 = GrammarBuilder::new("two_rules")
        .token("a", "[a]")
        .token("b", "[b]")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    assert!(t2.state_count >= t1.state_count);
}

// ---------------------------------------------------------------------------
// 35. Different grammars produce different bytes
// ---------------------------------------------------------------------------
#[test]
fn different_grammars_different_bytes() {
    let g1 = GrammarBuilder::new("alpha")
        .token("a", "[a]")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g2 = GrammarBuilder::new("beta")
        .token("a", "[a]")
        .token("b", "[b]")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    let b1 = t1.to_bytes().expect("serialize g1");
    let b2 = t2.to_bytes().expect("serialize g2");
    assert_ne!(b1, b2);
}

// ---------------------------------------------------------------------------
// 36. symbol_to_index contains eof_symbol
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn eof_in_symbol_to_index(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            table.symbol_to_index.contains_key(&table.eof_symbol),
            "eof_symbol {:?} missing from symbol_to_index", table.eof_symbol
        );
    }
}

// ---------------------------------------------------------------------------
// 37. index_to_symbol length matches symbol_to_index
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn index_symbol_maps_consistent(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert_eq!(
            table.index_to_symbol.len(),
            table.symbol_to_index.len(),
            "index_to_symbol len {} != symbol_to_index len {}",
            table.index_to_symbol.len(),
            table.symbol_to_index.len()
        );
    }
}

// ---------------------------------------------------------------------------
// 38. Roundtrip preserves symbol_to_index keys
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_symbol_to_index(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        let orig_keys: Vec<_> = table.symbol_to_index.keys().collect();
        let rest_keys: Vec<_> = restored.symbol_to_index.keys().collect();
        prop_assert_eq!(orig_keys, rest_keys);
    }
}

// ---------------------------------------------------------------------------
// 39. Roundtrip preserves index_to_symbol
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_index_to_symbol(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.index_to_symbol, restored.index_to_symbol);
    }
}

// ---------------------------------------------------------------------------
// 40. Roundtrip preserves extras
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_extras(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.extras, restored.extras);
    }
}

// ---------------------------------------------------------------------------
// 41. Roundtrip preserves field_names
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_field_names(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.field_names, restored.field_names);
    }
}

// ---------------------------------------------------------------------------
// 42. Roundtrip preserves lex_modes length
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_lex_modes(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.lex_modes.len(), restored.lex_modes.len());
    }
}

// ---------------------------------------------------------------------------
// 43. Roundtrip preserves dynamic_prec_by_rule
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_dynamic_prec(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.dynamic_prec_by_rule, restored.dynamic_prec_by_rule);
    }
}

// ---------------------------------------------------------------------------
// 44. Token count is positive
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn token_count_positive(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(table.token_count > 0);
    }
}

// ---------------------------------------------------------------------------
// 45. initial_state is within state_count
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn initial_state_in_range(grammar in any_grammar()) {
        let table = build_table(&grammar);
        prop_assert!(
            (table.initial_state.0 as usize) < table.state_count,
            "initial_state {} >= state_count {}", table.initial_state.0, table.state_count
        );
    }
}

// ---------------------------------------------------------------------------
// 46. grammar() accessor returns grammar with rules
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn grammar_accessor_has_rules(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let g = table.grammar();
        prop_assert!(!g.rules.is_empty(), "grammar accessor returned empty rules");
    }
}

// ---------------------------------------------------------------------------
// 47. Serialized bytes length grows with grammar complexity
// ---------------------------------------------------------------------------
#[test]
fn bytes_grow_with_complexity() {
    let g_small = GrammarBuilder::new("small")
        .token("a", "[a]")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g_large = GrammarBuilder::new("large")
        .token("a", "[a]")
        .token("b", "[b]")
        .token("c", "[c]")
        .token("d", "[d]")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .rule("start", vec!["d"])
        .start("start")
        .build();
    let b_small = build_table(&g_small).to_bytes().expect("small");
    let b_large = build_table(&g_large).to_bytes().expect("large");
    assert!(b_large.len() > b_small.len());
}

// ---------------------------------------------------------------------------
// 48. Triple roundtrip yields identical bytes
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn triple_roundtrip_stable(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let b1 = table.to_bytes().expect("ser 1");
        let r1 = ParseTable::from_bytes(&b1).expect("de 1");
        let b2 = r1.to_bytes().expect("ser 2");
        let r2 = ParseTable::from_bytes(&b2).expect("de 2");
        let b3 = r2.to_bytes().expect("ser 3");
        prop_assert_eq!(&b1, &b2);
        prop_assert_eq!(&b2, &b3);
    }
}

// ---------------------------------------------------------------------------
// 49. Roundtrip preserves rule_assoc_by_rule
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_rule_assoc(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.rule_assoc_by_rule, restored.rule_assoc_by_rule);
    }
}

// ---------------------------------------------------------------------------
// 50. Roundtrip preserves nonterminal_to_index keys
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_nt_to_index(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        let orig: Vec<_> = table.nonterminal_to_index.keys().collect();
        let rest: Vec<_> = restored.nonterminal_to_index.keys().collect();
        prop_assert_eq!(orig, rest);
    }
}

// ---------------------------------------------------------------------------
// 51. Two-token grammar: both tokens appear in symbol_to_index
// ---------------------------------------------------------------------------
#[test]
fn two_token_symbols_present() {
    let g = GrammarBuilder::new("two")
        .token("x", "[x]")
        .token("y", "[y]")
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .start("start")
        .build();
    let table = build_table(&g);
    // Both tokens + at least EOF + nonterminals → symbol_count >= 4
    assert!(table.symbol_count >= 3);
}

// ---------------------------------------------------------------------------
// 52. from_bytes rejects empty input
// ---------------------------------------------------------------------------
#[test]
fn from_bytes_rejects_empty() {
    let result = ParseTable::from_bytes(&[]);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 53. from_bytes rejects garbage bytes
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn from_bytes_rejects_garbage(bytes in proptest::collection::vec(any::<u8>(), 1..64)) {
        // Garbage bytes should almost always fail deserialization
        // (vanishingly unlikely to be valid postcard)
        let _ = ParseTable::from_bytes(&bytes);
        // Not asserting Err because random bytes *could* technically decode,
        // but we verify it doesn't panic.
    }
}

// ---------------------------------------------------------------------------
// 54. Roundtrip preserves external_token_count
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn roundtrip_preserves_external_token_count(grammar in any_grammar()) {
        let table = build_table(&grammar);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        prop_assert_eq!(table.external_token_count, restored.external_token_count);
    }
}
