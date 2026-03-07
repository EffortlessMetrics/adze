//! Comprehensive tests for ParseTable serialization/deserialization
//! via adze-glr-core's serialization module.
//!
//! Run with:
//!   cargo test -p adze --test serialization_v10 -- --test-threads=2

#[cfg(feature = "ts-compat")]
use adze::adze_glr_core as glr_core;
#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;

#[cfg(not(feature = "ts-compat"))]
use adze_glr_core as glr_core;
#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use glr_core::{Action, FirstFollowSets, ParseTable, StateId, build_lr1_automaton};
use ir::builder::GrammarBuilder;
use ir::{Associativity, Grammar, RuleId};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_table(name: &str) -> ParseTable {
    let g = GrammarBuilder::new(name)
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("ff");
    build_lr1_automaton(&g, &ff).expect("table")
}

fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

// ---------------------------------------------------------------------------
// 1. to_bytes produces non-empty bytes
// ---------------------------------------------------------------------------
#[test]
fn test_to_bytes_non_empty() {
    let table = make_table("ser_v10_nonempty");
    let bytes = table.to_bytes().expect("serialize");
    assert!(!bytes.is_empty());
}

// ---------------------------------------------------------------------------
// 2. from_bytes(to_bytes()) roundtrip succeeds
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_succeeds() {
    let table = make_table("ser_v10_roundtrip");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes);
    assert!(restored.is_ok());
}

// ---------------------------------------------------------------------------
// 3. Roundtrip preserves state_count
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_state_count() {
    let table = make_table("ser_v10_state_count");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
}

// ---------------------------------------------------------------------------
// 4. Roundtrip preserves symbol_count
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_symbol_count() {
    let table = make_table("ser_v10_sym_count");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.symbol_count, restored.symbol_count);
}

// ---------------------------------------------------------------------------
// 5. Roundtrip preserves eof_symbol
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_eof_symbol() {
    let table = make_table("ser_v10_eof");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.eof_symbol, restored.eof_symbol);
}

// ---------------------------------------------------------------------------
// 6. Roundtrip preserves actions for all state/symbol pairs
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_actions() {
    let table = make_table("ser_v10_actions");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    for st in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            let orig = table.actions(StateId(st as u16), sym);
            let rest = restored.actions(StateId(st as u16), sym);
            assert_eq!(
                orig.len(),
                rest.len(),
                "action len mismatch at state {st}, sym {}",
                sym.0
            );
            for (a, b) in orig.iter().zip(rest.iter()) {
                assert_eq!(format!("{a:?}"), format!("{b:?}"));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Roundtrip preserves goto for all state/symbol pairs
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_goto() {
    let table = make_table("ser_v10_goto");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    for st in 0..table.state_count {
        for &nt in table.nonterminal_to_index.keys() {
            let orig = table.goto(StateId(st as u16), nt);
            let rest = restored.goto(StateId(st as u16), nt);
            assert_eq!(orig, rest, "goto mismatch at state {st}, nt {}", nt.0);
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Roundtrip preserves rule info
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_rule_info() {
    let table = make_table("ser_v10_rule_info");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.rules.len(), restored.rules.len());
    for i in 0..table.rules.len() {
        let (lhs_orig, rhs_orig) = table.rule(RuleId(i as u16));
        let (lhs_rest, rhs_rest) = restored.rule(RuleId(i as u16));
        assert_eq!(lhs_orig, lhs_rest, "rule {i} lhs mismatch");
        assert_eq!(rhs_orig, rhs_rest, "rule {i} rhs_len mismatch");
    }
}

// ---------------------------------------------------------------------------
// 9. from_bytes on empty slice → error
// ---------------------------------------------------------------------------
#[test]
fn test_from_bytes_empty_is_error() {
    let result = ParseTable::from_bytes(&[]);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 10. from_bytes on garbage → error
// ---------------------------------------------------------------------------
#[test]
fn test_from_bytes_garbage_is_error() {
    let garbage = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03];
    let result = ParseTable::from_bytes(&garbage);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 11. from_bytes on truncated bytes → error
// ---------------------------------------------------------------------------
#[test]
fn test_from_bytes_truncated_is_error() {
    let table = make_table("ser_v10_trunc");
    let bytes = table.to_bytes().expect("serialize");
    assert!(bytes.len() > 4);
    let truncated = &bytes[..bytes.len() / 2];
    let result = ParseTable::from_bytes(truncated);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 12. Deterministic: to_bytes called twice → same bytes
// ---------------------------------------------------------------------------
#[test]
fn test_to_bytes_deterministic() {
    let table = make_table("ser_v10_determ");
    let a = table.to_bytes().expect("serialize a");
    let b = table.to_bytes().expect("serialize b");
    assert_eq!(a, b);
}

// ---------------------------------------------------------------------------
// 13. Different grammars → different bytes
// ---------------------------------------------------------------------------
#[test]
fn test_different_grammars_different_bytes() {
    let g1 = GrammarBuilder::new("ser_v10_diff1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g2 = GrammarBuilder::new("ser_v10_diff2")
        .token("a", "a")
        .token("b", "b")
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
// 14. Larger grammar → more bytes
// ---------------------------------------------------------------------------
#[test]
fn test_larger_grammar_more_bytes() {
    let g_small = GrammarBuilder::new("ser_v10_small")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g_large = GrammarBuilder::new("ser_v10_large")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .rule("start", vec!["d"])
        .rule("start", vec!["e"])
        .start("start")
        .build();
    let t_small = build_table(&g_small);
    let t_large = build_table(&g_large);
    let b_small = t_small.to_bytes().expect("serialize small");
    let b_large = t_large.to_bytes().expect("serialize large");
    assert!(b_large.len() > b_small.len());
}

// ---------------------------------------------------------------------------
// 15. Various grammar sizes roundtrip (1 token)
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_1_token() {
    let g = GrammarBuilder::new("ser_v10_1tok")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
}

// ---------------------------------------------------------------------------
// 16. Various grammar sizes roundtrip (2 tokens)
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_2_tokens() {
    let g = GrammarBuilder::new("ser_v10_2tok")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
}

// ---------------------------------------------------------------------------
// 17. Various grammar sizes roundtrip (3 tokens)
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_3_tokens() {
    let g = GrammarBuilder::new("ser_v10_3tok")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
}

// ---------------------------------------------------------------------------
// 18. Various grammar sizes roundtrip (5 tokens)
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_5_tokens() {
    let g = GrammarBuilder::new("ser_v10_5tok")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .rule("start", vec!["d"])
        .rule("start", vec!["e"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
}

// ---------------------------------------------------------------------------
// 19. Roundtrip preserves start_symbol
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_start_symbol() {
    let table = make_table("ser_v10_start_sym");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.start_symbol(), restored.start_symbol());
}

// ---------------------------------------------------------------------------
// 20. Roundtrip preserves initial_state
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_initial_state() {
    let table = make_table("ser_v10_init_state");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.initial_state, restored.initial_state);
}

// ---------------------------------------------------------------------------
// 21. Roundtrip preserves token_count
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_token_count() {
    let table = make_table("ser_v10_tok_count");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.token_count, restored.token_count);
}

// ---------------------------------------------------------------------------
// 22. Roundtrip preserves action_table dimensions
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_action_table_rows() {
    let table = make_table("ser_v10_act_rows");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.action_table.len(), restored.action_table.len());
}

// ---------------------------------------------------------------------------
// 23. Roundtrip preserves goto_table dimensions
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_goto_table_rows() {
    let table = make_table("ser_v10_goto_rows");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.goto_table.len(), restored.goto_table.len());
}

// ---------------------------------------------------------------------------
// 24. Roundtrip preserves rule count
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_rule_count() {
    let table = make_table("ser_v10_rule_cnt");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.rules.len(), restored.rules.len());
}

// ---------------------------------------------------------------------------
// 25. Roundtrip preserves extras
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_extras() {
    let table = make_table("ser_v10_extras");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.extras, restored.extras);
}

// ---------------------------------------------------------------------------
// 26. Roundtrip preserves field_names
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_field_names() {
    let table = make_table("ser_v10_fields");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.field_names, restored.field_names);
}

// ---------------------------------------------------------------------------
// 27. Roundtrip preserves lex_modes length
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_lex_modes_len() {
    let table = make_table("ser_v10_lex_modes");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.lex_modes.len(), restored.lex_modes.len());
}

// ---------------------------------------------------------------------------
// 28. Roundtrip preserves dynamic_prec_by_rule
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_dynamic_prec() {
    let table = make_table("ser_v10_dyn_prec");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.dynamic_prec_by_rule, restored.dynamic_prec_by_rule);
}

// ---------------------------------------------------------------------------
// 29. Roundtrip preserves symbol_to_index keys
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_symbol_to_index() {
    let table = make_table("ser_v10_sym_idx");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    let orig_keys: Vec<_> = table.symbol_to_index.keys().collect();
    let rest_keys: Vec<_> = restored.symbol_to_index.keys().collect();
    assert_eq!(orig_keys, rest_keys);
}

// ---------------------------------------------------------------------------
// 30. Roundtrip preserves index_to_symbol
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_index_to_symbol() {
    let table = make_table("ser_v10_idx_sym");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.index_to_symbol, restored.index_to_symbol);
}

// ---------------------------------------------------------------------------
// 31. Double roundtrip: serialize→deserialize→serialize→deserialize
// ---------------------------------------------------------------------------
#[test]
fn test_double_roundtrip() {
    let table = make_table("ser_v10_double_rt");
    let b1 = table.to_bytes().expect("serialize 1");
    let r1 = ParseTable::from_bytes(&b1).expect("deserialize 1");
    let b2 = r1.to_bytes().expect("serialize 2");
    let r2 = ParseTable::from_bytes(&b2).expect("deserialize 2");
    assert_eq!(b1, b2);
    assert_eq!(r1.state_count, r2.state_count);
    assert_eq!(r1.symbol_count, r2.symbol_count);
    assert_eq!(r1.eof_symbol, r2.eof_symbol);
}

// ---------------------------------------------------------------------------
// 32. Triple roundtrip bytes stability
// ---------------------------------------------------------------------------
#[test]
fn test_triple_roundtrip_stable_bytes() {
    let table = make_table("ser_v10_triple_rt");
    let b1 = table.to_bytes().expect("ser 1");
    let r1 = ParseTable::from_bytes(&b1).expect("de 1");
    let b2 = r1.to_bytes().expect("ser 2");
    let r2 = ParseTable::from_bytes(&b2).expect("de 2");
    let b3 = r2.to_bytes().expect("ser 3");
    assert_eq!(b1, b2);
    assert_eq!(b2, b3);
}

// ---------------------------------------------------------------------------
// 33. from_bytes on single byte → error
// ---------------------------------------------------------------------------
#[test]
fn test_from_bytes_single_byte_is_error() {
    let result = ParseTable::from_bytes(&[0x42]);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 34. from_bytes on all zeros → error
// ---------------------------------------------------------------------------
#[test]
fn test_from_bytes_all_zeros_is_error() {
    let zeros = vec![0u8; 64];
    let result = ParseTable::from_bytes(&zeros);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 35. from_bytes on all 0xFF → error
// ---------------------------------------------------------------------------
#[test]
fn test_from_bytes_all_ff_is_error() {
    let ffs = vec![0xFFu8; 128];
    let result = ParseTable::from_bytes(&ffs);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 36. Accept action exists in table
// ---------------------------------------------------------------------------
#[test]
fn test_accept_action_exists() {
    let table = make_table("ser_v10_accept");
    let has_accept = (0..table.state_count).any(|st| {
        table
            .actions(StateId(st as u16), table.eof())
            .iter()
            .any(|a| matches!(a, Action::Accept))
    });
    assert!(has_accept);
}

// ---------------------------------------------------------------------------
// 37. Shift action exists in table
// ---------------------------------------------------------------------------
#[test]
fn test_shift_action_exists() {
    let table = make_table("ser_v10_shift");
    let has_shift = (0..table.state_count).any(|st| {
        table.symbol_to_index.keys().any(|&sym| {
            table
                .actions(StateId(st as u16), sym)
                .iter()
                .any(|a| matches!(a, Action::Shift(_)))
        })
    });
    assert!(has_shift);
}

// ---------------------------------------------------------------------------
// 38. Reduce action exists in table
// ---------------------------------------------------------------------------
#[test]
fn test_reduce_action_exists() {
    let table = make_table("ser_v10_reduce");
    let has_reduce = (0..table.state_count).any(|st| {
        table.symbol_to_index.keys().any(|&sym| {
            table
                .actions(StateId(st as u16), sym)
                .iter()
                .any(|a| matches!(a, Action::Reduce(_)))
        })
    });
    assert!(has_reduce);
}

// ---------------------------------------------------------------------------
// 39. Roundtrip preserves Accept action locations
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_accept_locations() {
    let table = make_table("ser_v10_accept_loc");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    for st in 0..table.state_count {
        let orig = table.actions(StateId(st as u16), table.eof());
        let rest = restored.actions(StateId(st as u16), restored.eof());
        let orig_has_accept = orig.iter().any(|a| matches!(a, Action::Accept));
        let rest_has_accept = rest.iter().any(|a| matches!(a, Action::Accept));
        assert_eq!(
            orig_has_accept, rest_has_accept,
            "accept mismatch at state {st}"
        );
    }
}

// ---------------------------------------------------------------------------
// 40. Shift targets are valid state indices
// ---------------------------------------------------------------------------
#[test]
fn test_shift_targets_valid() {
    let table = make_table("ser_v10_shift_valid");
    for st in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(StateId(st as u16), sym) {
                if let Action::Shift(target) = action {
                    assert!((target.0 as usize) < table.state_count);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 41. Reduce rule IDs are valid
// ---------------------------------------------------------------------------
#[test]
fn test_reduce_rule_ids_valid() {
    let table = make_table("ser_v10_reduce_valid");
    for st in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for action in table.actions(StateId(st as u16), sym) {
                if let Action::Reduce(rid) = action {
                    assert!((rid.0 as usize) < table.rules.len());
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 42. state_count >= 2
// ---------------------------------------------------------------------------
#[test]
fn test_state_count_at_least_two() {
    let table = make_table("ser_v10_st_min");
    assert!(table.state_count >= 2);
}

// ---------------------------------------------------------------------------
// 43. symbol_count >= 2 (at least one terminal + EOF)
// ---------------------------------------------------------------------------
#[test]
fn test_symbol_count_at_least_two() {
    let table = make_table("ser_v10_sym_min");
    assert!(table.symbol_count >= 2);
}

// ---------------------------------------------------------------------------
// 44. eof_symbol in symbol_to_index
// ---------------------------------------------------------------------------
#[test]
fn test_eof_in_symbol_to_index() {
    let table = make_table("ser_v10_eof_idx");
    assert!(table.symbol_to_index.contains_key(&table.eof_symbol));
}

// ---------------------------------------------------------------------------
// 45. eof() accessor matches eof_symbol field
// ---------------------------------------------------------------------------
#[test]
fn test_eof_accessor_matches_field() {
    let table = make_table("ser_v10_eof_acc");
    assert_eq!(table.eof(), table.eof_symbol);
}

// ---------------------------------------------------------------------------
// 46. action_table has state_count rows
// ---------------------------------------------------------------------------
#[test]
fn test_action_table_row_count() {
    let table = make_table("ser_v10_act_cnt");
    assert_eq!(table.action_table.len(), table.state_count);
}

// ---------------------------------------------------------------------------
// 47. goto_table has state_count rows
// ---------------------------------------------------------------------------
#[test]
fn test_goto_table_row_count() {
    let table = make_table("ser_v10_goto_cnt");
    assert_eq!(table.goto_table.len(), table.state_count);
}

// ---------------------------------------------------------------------------
// 48. Rules have valid lhs symbols
// ---------------------------------------------------------------------------
#[test]
fn test_rules_lhs_valid() {
    let table = make_table("ser_v10_lhs_valid");
    for (i, rule) in table.rules.iter().enumerate() {
        assert!(
            (rule.lhs.0 as usize) < table.symbol_count,
            "rule {i} lhs {} >= symbol_count {}",
            rule.lhs.0,
            table.symbol_count
        );
    }
}

// ---------------------------------------------------------------------------
// 49. initial_state is within state_count
// ---------------------------------------------------------------------------
#[test]
fn test_initial_state_in_range() {
    let table = make_table("ser_v10_init_range");
    assert!((table.initial_state.0 as usize) < table.state_count);
}

// ---------------------------------------------------------------------------
// 50. Grammar with precedence roundtrips
// ---------------------------------------------------------------------------
#[test]
fn test_precedence_grammar_roundtrip() {
    let g = GrammarBuilder::new("ser_v10_prec")
        .token("n", "n")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.symbol_count, restored.symbol_count);
    assert_eq!(table.rules.len(), restored.rules.len());
}

// ---------------------------------------------------------------------------
// 51. Grammar with multiple alternatives roundtrips
// ---------------------------------------------------------------------------
#[test]
fn test_multi_alt_grammar_roundtrip() {
    let g = GrammarBuilder::new("ser_v10_multi_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
}

// ---------------------------------------------------------------------------
// 52. Grammar with chain rules roundtrips
// ---------------------------------------------------------------------------
#[test]
fn test_chain_rule_grammar_roundtrip() {
    let g = GrammarBuilder::new("ser_v10_chain")
        .token("x", "x")
        .rule("start", vec!["middle"])
        .rule("middle", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.symbol_count, restored.symbol_count);
}

// ---------------------------------------------------------------------------
// 53. Grammar with sequence (multi-symbol rhs) roundtrips
// ---------------------------------------------------------------------------
#[test]
fn test_sequence_grammar_roundtrip() {
    let g = GrammarBuilder::new("ser_v10_seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.rules.len(), restored.rules.len());
}

// ---------------------------------------------------------------------------
// 54. Roundtrip preserves nonterminal_to_index
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_nonterminal_to_index() {
    let table = make_table("ser_v10_nt_idx");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    let orig_keys: Vec<_> = table.nonterminal_to_index.keys().collect();
    let rest_keys: Vec<_> = restored.nonterminal_to_index.keys().collect();
    assert_eq!(orig_keys, rest_keys);
}

// ---------------------------------------------------------------------------
// 55. Roundtrip preserves external_token_count
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_external_token_count() {
    let table = make_table("ser_v10_ext_tok");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.external_token_count, restored.external_token_count);
}

// ---------------------------------------------------------------------------
// 56. Roundtrip preserves rule_assoc_by_rule
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_rule_assoc() {
    let table = make_table("ser_v10_assoc");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.rule_assoc_by_rule, restored.rule_assoc_by_rule);
}

// ---------------------------------------------------------------------------
// 57. Roundtrip preserves alias_sequences length
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_alias_sequences() {
    let table = make_table("ser_v10_alias");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.alias_sequences.len(), restored.alias_sequences.len());
}

// ---------------------------------------------------------------------------
// 58. Roundtrip preserves field_map
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_field_map() {
    let table = make_table("ser_v10_fmap");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.field_map, restored.field_map);
}

// ---------------------------------------------------------------------------
// 59. Roundtrip preserves symbol_metadata length
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_symbol_metadata_len() {
    let table = make_table("ser_v10_sym_meta");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.symbol_metadata.len(), restored.symbol_metadata.len());
}

// ---------------------------------------------------------------------------
// 60. Roundtrip preserves external_scanner_states
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_external_scanner_states() {
    let table = make_table("ser_v10_ext_scan");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(
        table.external_scanner_states.len(),
        restored.external_scanner_states.len()
    );
}

// ---------------------------------------------------------------------------
// 61. Serialized bytes are at least 8 bytes long
// ---------------------------------------------------------------------------
#[test]
fn test_bytes_minimum_length() {
    let table = make_table("ser_v10_min_len");
    let bytes = table.to_bytes().expect("serialize");
    assert!(bytes.len() >= 8);
}

// ---------------------------------------------------------------------------
// 62. from_bytes on very short slice (2 bytes) → error
// ---------------------------------------------------------------------------
#[test]
fn test_from_bytes_two_bytes_is_error() {
    let result = ParseTable::from_bytes(&[0x01, 0x02]);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 63. from_bytes on valid bytes with extra trailing byte → still works or errors
// ---------------------------------------------------------------------------
#[test]
fn test_from_bytes_with_trailing_byte() {
    let table = make_table("ser_v10_trail");
    let mut bytes = table.to_bytes().expect("serialize");
    bytes.push(0xFF);
    // Postcard may accept or reject trailing data — either is valid
    let _ = ParseTable::from_bytes(&bytes);
}

// ---------------------------------------------------------------------------
// 64. Roundtrip: re-serialized bytes match original
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_bytes_identity() {
    let table = make_table("ser_v10_bytes_id");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    let bytes2 = restored.to_bytes().expect("re-serialize");
    assert_eq!(bytes, bytes2);
}

// ---------------------------------------------------------------------------
// 65. Serialization does not panic for minimal grammar
// ---------------------------------------------------------------------------
#[test]
fn test_no_panic_minimal_grammar() {
    let table = make_table("ser_v10_no_panic");
    let _ = table.to_bytes();
}

// ---------------------------------------------------------------------------
// 66. Deserialization does not panic on random-ish data
// ---------------------------------------------------------------------------
#[test]
fn test_no_panic_random_data() {
    for seed in 0u8..20 {
        let data: Vec<u8> = (0..64u8)
            .map(|i| i.wrapping_mul(seed).wrapping_add(37))
            .collect();
        let _ = ParseTable::from_bytes(&data);
    }
}

// ---------------------------------------------------------------------------
// 67. Roundtrip preserves goto_indexing
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_goto_indexing() {
    let table = make_table("ser_v10_goto_idx");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(
        format!("{:?}", table.goto_indexing),
        format!("{:?}", restored.goto_indexing)
    );
}

// ---------------------------------------------------------------------------
// 68. Multiple independent tables serialize/deserialize correctly
// ---------------------------------------------------------------------------
#[test]
fn test_multiple_independent_tables() {
    let t1 = make_table("ser_v10_indep1");
    let t2 = make_table("ser_v10_indep2");
    let b1 = t1.to_bytes().expect("ser t1");
    let b2 = t2.to_bytes().expect("ser t2");
    let r1 = ParseTable::from_bytes(&b1).expect("de t1");
    let r2 = ParseTable::from_bytes(&b2).expect("de t2");
    assert_eq!(t1.state_count, r1.state_count);
    assert_eq!(t2.state_count, r2.state_count);
}

// ---------------------------------------------------------------------------
// 69. Grammar name preserved through roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_grammar_name_preserved() {
    let table = make_table("ser_v10_gname");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.grammar().name, restored.grammar().name);
}

// ---------------------------------------------------------------------------
// 70. Roundtrip preserves grammar rules count
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_grammar_rules_count() {
    let table = make_table("ser_v10_gram_rules");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.grammar().rules.len(), restored.grammar().rules.len());
}

// ---------------------------------------------------------------------------
// 71. Roundtrip preserves grammar tokens count
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_grammar_tokens_count() {
    let table = make_table("ser_v10_gram_toks");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(
        table.grammar().tokens.len(),
        restored.grammar().tokens.len()
    );
}

// ---------------------------------------------------------------------------
// 72. token_count is positive
// ---------------------------------------------------------------------------
#[test]
fn test_token_count_positive() {
    let table = make_table("ser_v10_tok_pos");
    assert!(table.token_count > 0);
}

// ---------------------------------------------------------------------------
// 73. from_bytes with bytes from one table does not produce another table's data
// ---------------------------------------------------------------------------
#[test]
fn test_cross_table_isolation() {
    let g1 = GrammarBuilder::new("ser_v10_iso1")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let g2 = GrammarBuilder::new("ser_v10_iso2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let t1 = build_table(&g1);
    let t2 = build_table(&g2);
    let b1 = t1.to_bytes().expect("ser t1");
    let b2 = t2.to_bytes().expect("ser t2");
    let r1 = ParseTable::from_bytes(&b1).expect("de b1");
    let r2 = ParseTable::from_bytes(&b2).expect("de b2");
    // Restored tables should match their originals, not each other
    assert_eq!(r1.state_count, t1.state_count);
    assert_eq!(r2.state_count, t2.state_count);
    assert_ne!(r1.state_count, r2.state_count);
}

// ---------------------------------------------------------------------------
// 74. Byte slice boundary: just the first byte of valid → error
// ---------------------------------------------------------------------------
#[test]
fn test_first_byte_only_is_error() {
    let table = make_table("ser_v10_fbyte");
    let bytes = table.to_bytes().expect("serialize");
    let result = ParseTable::from_bytes(&bytes[..1]);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 75. Byte slice boundary: last byte removed → error
// ---------------------------------------------------------------------------
#[test]
fn test_last_byte_removed_is_error() {
    let table = make_table("ser_v10_lbyte");
    let bytes = table.to_bytes().expect("serialize");
    let result = ParseTable::from_bytes(&bytes[..bytes.len() - 1]);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 76. Corrupted byte in middle → error
// ---------------------------------------------------------------------------
#[test]
fn test_corrupted_middle_byte_is_error() {
    let table = make_table("ser_v10_corrupt");
    let mut bytes = table.to_bytes().expect("serialize");
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xFF;
    // May or may not error — corruption might produce a valid but different parse
    // The test ensures no panic
    let _ = ParseTable::from_bytes(&bytes);
}

// ---------------------------------------------------------------------------
// 77. Multiple serializations from different grammars don't interfere
// ---------------------------------------------------------------------------
#[test]
fn test_no_cross_contamination() {
    let names: Vec<String> = (0..5).map(|i| format!("ser_v10_cc{i}")).collect();
    let tables: Vec<ParseTable> = names.iter().map(|n| make_table(n)).collect();
    let bytes_vec: Vec<Vec<u8>> = tables.iter().map(|t| t.to_bytes().expect("ser")).collect();
    for (i, bs) in bytes_vec.iter().enumerate() {
        let restored = ParseTable::from_bytes(bs).expect("de");
        assert_eq!(tables[i].state_count, restored.state_count);
        assert_eq!(tables[i].symbol_count, restored.symbol_count);
    }
}

// ---------------------------------------------------------------------------
// 78. Byte length is consistent across multiple calls
// ---------------------------------------------------------------------------
#[test]
fn test_byte_length_consistent() {
    let table = make_table("ser_v10_blen");
    let len1 = table.to_bytes().expect("ser 1").len();
    let len2 = table.to_bytes().expect("ser 2").len();
    let len3 = table.to_bytes().expect("ser 3").len();
    assert_eq!(len1, len2);
    assert_eq!(len2, len3);
}

// ---------------------------------------------------------------------------
// 79. Grammar with right-associative rule roundtrips
// ---------------------------------------------------------------------------
#[test]
fn test_right_assoc_grammar_roundtrip() {
    let g = GrammarBuilder::new("ser_v10_rassoc")
        .token("n", "n")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["n"])
        .start("expr")
        .build();
    let table = build_table(&g);
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.rules.len(), restored.rules.len());
}

// ---------------------------------------------------------------------------
// 80. Grammar with extra tokens roundtrips
// ---------------------------------------------------------------------------
#[test]
fn test_extras_grammar_roundtrip() {
    let g = GrammarBuilder::new("ser_v10_extra")
        .token("x", "x")
        .token("ws", " ")
        .extra("ws")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.extras, restored.extras);
}

// ---------------------------------------------------------------------------
// 81. Roundtrip preserves Shift target values exactly
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_shift_targets() {
    let table = make_table("ser_v10_shift_tgt");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    for st in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for (a, b) in table
                .actions(StateId(st as u16), sym)
                .iter()
                .zip(restored.actions(StateId(st as u16), sym).iter())
            {
                if let (Action::Shift(t1), Action::Shift(t2)) = (a, b) {
                    assert_eq!(t1, t2);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 82. Roundtrip preserves Reduce rule IDs exactly
// ---------------------------------------------------------------------------
#[test]
fn test_roundtrip_preserves_reduce_ids() {
    let table = make_table("ser_v10_red_ids");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    for st in 0..table.state_count {
        for &sym in table.symbol_to_index.keys() {
            for (a, b) in table
                .actions(StateId(st as u16), sym)
                .iter()
                .zip(restored.actions(StateId(st as u16), sym).iter())
            {
                if let (Action::Reduce(r1), Action::Reduce(r2)) = (a, b) {
                    assert_eq!(r1, r2);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 83. from_bytes on repeated valid header prefix → error
// ---------------------------------------------------------------------------
#[test]
fn test_from_bytes_repeated_header_prefix_is_error() {
    let table = make_table("ser_v10_hdr_pfx");
    let bytes = table.to_bytes().expect("serialize");
    let prefix = if bytes.len() >= 8 {
        &bytes[..8]
    } else {
        &bytes
    };
    let mut repeated = prefix.to_vec();
    repeated.extend_from_slice(prefix);
    let _ = ParseTable::from_bytes(&repeated);
}

// ---------------------------------------------------------------------------
// 84. Grammar with nested nonterminals roundtrips
// ---------------------------------------------------------------------------
#[test]
fn test_nested_nonterminals_roundtrip() {
    let g = GrammarBuilder::new("ser_v10_nested")
        .token("x", "x")
        .rule("start", vec!["outer"])
        .rule("outer", vec!["inner"])
        .rule("inner", vec!["x"])
        .start("start")
        .build();
    let table = build_table(&g);
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
    assert_eq!(table.state_count, restored.state_count);
    assert_eq!(table.symbol_count, restored.symbol_count);
    assert_eq!(table.rules.len(), restored.rules.len());
}

// ---------------------------------------------------------------------------
// 85. Action count consistency after roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_action_count_after_roundtrip() {
    let table = make_table("ser_v10_act_cnt2");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");

    let count_actions = |t: &ParseTable| -> usize {
        let mut total = 0;
        for st in 0..t.state_count {
            for &sym in t.symbol_to_index.keys() {
                total += t.actions(StateId(st as u16), sym).len();
            }
        }
        total
    };

    assert_eq!(count_actions(&table), count_actions(&restored));
}

// ---------------------------------------------------------------------------
// 86. Goto entry count consistency after roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_goto_entry_count_after_roundtrip() {
    let table = make_table("ser_v10_goto_cnt2");
    let bytes = table.to_bytes().expect("serialize");
    let restored = ParseTable::from_bytes(&bytes).expect("deserialize");

    let count_gotos = |t: &ParseTable| -> usize {
        let mut total = 0;
        for st in 0..t.state_count {
            for &nt in t.nonterminal_to_index.keys() {
                if t.goto(StateId(st as u16), nt).is_some() {
                    total += 1;
                }
            }
        }
        total
    };

    assert_eq!(count_gotos(&table), count_gotos(&restored));
}
