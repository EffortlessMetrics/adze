#![allow(clippy::needless_range_loop)]

//! Property-based tests for AliasSequence in adze-ir.

use adze_ir::{AliasSequence, Grammar, ProductionId};
use proptest::prelude::*;

/// Strategy for optional alias names.
fn alias_entry_strategy() -> impl Strategy<Value = Option<String>> {
    prop_oneof![Just(None), "[a-z][a-z0-9_]{0,15}".prop_map(Some),]
}

/// Strategy for a Vec of optional alias entries.
fn alias_vec_strategy(max_len: usize) -> impl Strategy<Value = Vec<Option<String>>> {
    prop::collection::vec(alias_entry_strategy(), 0..=max_len)
}

/// Strategy for an AliasSequence.
fn _alias_sequence_strategy() -> impl Strategy<Value = AliasSequence> {
    alias_vec_strategy(10).prop_map(|aliases| AliasSequence { aliases })
}

/// Strategy for a ProductionId.
fn production_id_strategy() -> impl Strategy<Value = ProductionId> {
    (0u16..500).prop_map(ProductionId)
}

// ── Creation tests ──────────────────────────────────────────────────

proptest! {
    #[test]
    fn test_creation_preserves_length(aliases in alias_vec_strategy(20)) {
        let seq = AliasSequence { aliases: aliases.clone() };
        prop_assert_eq!(seq.aliases.len(), aliases.len());
    }

    #[test]
    fn test_creation_preserves_entries(aliases in alias_vec_strategy(15)) {
        let seq = AliasSequence { aliases: aliases.clone() };
        for i in 0..aliases.len() {
            prop_assert_eq!(&seq.aliases[i], &aliases[i]);
        }
    }

    #[test]
    fn test_creation_from_all_none(len in 0usize..20) {
        let aliases: Vec<Option<String>> = vec![None; len];
        let seq = AliasSequence { aliases: aliases.clone() };
        prop_assert_eq!(seq.aliases.len(), len);
        for i in 0..len {
            prop_assert!(seq.aliases[i].is_none());
        }
    }

    #[test]
    fn test_creation_from_all_some(names in prop::collection::vec("[a-z]{1,8}", 0..=10)) {
        let aliases: Vec<Option<String>> = names.iter().map(|n| Some(n.clone())).collect();
        let seq = AliasSequence { aliases };
        for i in 0..names.len() {
            prop_assert_eq!(seq.aliases[i].as_deref(), Some(names[i].as_str()));
        }
    }
}

// ── Serde roundtrip tests ───────────────────────────────────────────

proptest! {
    #[test]
    fn test_serde_json_roundtrip(aliases in alias_vec_strategy(15)) {
        let seq = AliasSequence { aliases };
        let json = serde_json::to_string(&seq).unwrap();
        let deserialized: AliasSequence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(seq, deserialized);
    }

    #[test]
    fn test_serde_json_pretty_roundtrip(aliases in alias_vec_strategy(10)) {
        let seq = AliasSequence { aliases };
        let json = serde_json::to_string_pretty(&seq).unwrap();
        let deserialized: AliasSequence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(seq, deserialized);
    }

    #[test]
    fn test_serde_bincode_roundtrip(aliases in alias_vec_strategy(15)) {
        let seq = AliasSequence { aliases };
        let bytes = postcard::to_allocvec(&seq).unwrap();
        let deserialized: AliasSequence = postcard::from_bytes(&bytes).unwrap();
        prop_assert_eq!(seq, deserialized);
    }

    #[test]
    fn test_serde_json_value_roundtrip(aliases in alias_vec_strategy(10)) {
        let seq = AliasSequence { aliases };
        let val = serde_json::to_value(&seq).unwrap();
        let deserialized: AliasSequence = serde_json::from_value(val).unwrap();
        prop_assert_eq!(seq, deserialized);
    }
}

// ── Entry tests ─────────────────────────────────────────────────────

proptest! {
    #[test]
    fn test_some_entries_count(aliases in alias_vec_strategy(20)) {
        let seq = AliasSequence { aliases: aliases.clone() };
        let expected = aliases.iter().filter(|a| a.is_some()).count();
        let actual = seq.aliases.iter().filter(|a| a.is_some()).count();
        prop_assert_eq!(actual, expected);
    }

    #[test]
    fn test_none_entries_count(aliases in alias_vec_strategy(20)) {
        let seq = AliasSequence { aliases: aliases.clone() };
        let expected = aliases.iter().filter(|a| a.is_none()).count();
        let actual = seq.aliases.iter().filter(|a| a.is_none()).count();
        prop_assert_eq!(actual, expected);
    }

    #[test]
    fn test_entries_sum_to_length(aliases in alias_vec_strategy(20)) {
        let seq = AliasSequence { aliases };
        let some_count = seq.aliases.iter().filter(|a| a.is_some()).count();
        let none_count = seq.aliases.iter().filter(|a| a.is_none()).count();
        prop_assert_eq!(some_count + none_count, seq.aliases.len());
    }

    #[test]
    fn test_mixed_entries_preserved(
        nones in 0usize..5,
        name in "[a-z]{1,8}",
        trailing_nones in 0usize..5,
    ) {
        let mut aliases: Vec<Option<String>> = vec![None; nones];
        aliases.push(Some(name.clone()));
        aliases.extend(vec![None; trailing_nones]);
        let seq = AliasSequence { aliases };
        prop_assert_eq!(seq.aliases[nones].as_deref(), Some(name.as_str()));
        prop_assert_eq!(seq.aliases.len(), nones + 1 + trailing_nones);
    }
}

// ── Empty AliasSequence tests ───────────────────────────────────────

proptest! {
    #[test]
    fn test_empty_sequence_is_empty(_dummy in 0..1i32) {
        let seq = AliasSequence { aliases: vec![] };
        prop_assert!(seq.aliases.is_empty());
        prop_assert_eq!(seq.aliases.len(), 0);
    }

    #[test]
    fn test_empty_serde_roundtrip(_dummy in 0..1i32) {
        let seq = AliasSequence { aliases: vec![] };
        let json = serde_json::to_string(&seq).unwrap();
        let deserialized: AliasSequence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(seq, deserialized);
    }

    #[test]
    fn test_empty_clone_eq(_dummy in 0..1i32) {
        let seq = AliasSequence { aliases: vec![] };
        let cloned = seq.clone();
        prop_assert_eq!(&seq, &cloned);
    }
}

// ── Clone / Eq tests ────────────────────────────────────────────────

proptest! {
    #[test]
    fn test_clone_equals_original(aliases in alias_vec_strategy(15)) {
        let seq = AliasSequence { aliases };
        let cloned = seq.clone();
        prop_assert_eq!(&seq, &cloned);
    }

    #[test]
    fn test_clone_is_independent(aliases in alias_vec_strategy(10)) {
        let seq = AliasSequence { aliases };
        let mut cloned = seq.clone();
        cloned.aliases.push(Some("extra".to_string()));
        prop_assert_ne!(seq.aliases.len(), cloned.aliases.len());
    }

    #[test]
    fn test_eq_reflexive(aliases in alias_vec_strategy(10)) {
        let seq = AliasSequence { aliases };
        prop_assert_eq!(&seq, &seq);
    }

    #[test]
    fn test_eq_symmetric(a in alias_vec_strategy(10)) {
        let x = AliasSequence { aliases: a.clone() };
        let y = AliasSequence { aliases: a };
        prop_assert_eq!(&x, &y);
        prop_assert_eq!(&y, &x);
    }

    #[test]
    fn test_ne_different_lengths(
        a in alias_vec_strategy(5),
        extra in alias_entry_strategy(),
    ) {
        let x = AliasSequence { aliases: a.clone() };
        let mut b = a;
        b.push(extra);
        let y = AliasSequence { aliases: b };
        prop_assert_ne!(&x, &y);
    }
}

// ── Grammar context tests ───────────────────────────────────────────

proptest! {
    #[test]
    fn test_grammar_alias_insert_and_retrieve(
        prod_id in production_id_strategy(),
        aliases in alias_vec_strategy(10),
    ) {
        let mut grammar = Grammar::new("test_grammar".to_string());
        let seq = AliasSequence { aliases: aliases.clone() };
        grammar.alias_sequences.insert(prod_id, seq.clone());
        let retrieved = grammar.alias_sequences.get(&prod_id).unwrap();
        prop_assert_eq!(retrieved, &seq);
    }

    #[test]
    fn test_grammar_max_alias_length_tracking(
        aliases in alias_vec_strategy(15),
        prod_id in production_id_strategy(),
    ) {
        let mut grammar = Grammar::new("test_grammar".to_string());
        let seq = AliasSequence { aliases: aliases.clone() };
        grammar.alias_sequences.insert(prod_id, seq);
        grammar.max_alias_sequence_length = aliases.len();
        prop_assert_eq!(grammar.max_alias_sequence_length, aliases.len());
    }

    #[test]
    fn test_grammar_alias_overwrite(
        prod_id in production_id_strategy(),
        first in alias_vec_strategy(8),
        second in alias_vec_strategy(8),
    ) {
        let mut grammar = Grammar::new("test_grammar".to_string());
        grammar.alias_sequences.insert(prod_id, AliasSequence { aliases: first });
        grammar.alias_sequences.insert(prod_id, AliasSequence { aliases: second.clone() });
        let retrieved = grammar.alias_sequences.get(&prod_id).unwrap();
        prop_assert_eq!(&retrieved.aliases, &second);
    }

    #[test]
    fn test_grammar_serde_with_aliases(
        prod_id in production_id_strategy(),
        aliases in alias_vec_strategy(8),
    ) {
        let mut grammar = Grammar::new("test_grammar".to_string());
        grammar.alias_sequences.insert(prod_id, AliasSequence { aliases });
        let json = serde_json::to_string(&grammar).unwrap();
        let deserialized: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(
            grammar.alias_sequences.get(&prod_id),
            deserialized.alias_sequences.get(&prod_id),
        );
    }
}

// ── Multiple AliasSequences tests ───────────────────────────────────

proptest! {
    #[test]
    fn test_multiple_sequences_stored(
        entries in prop::collection::vec(
            (production_id_strategy(), alias_vec_strategy(8)),
            1..=10,
        )
    ) {
        let mut grammar = Grammar::new("multi".to_string());
        // Use distinct production IDs by deduplicating
        let mut seen = std::collections::HashSet::new();
        let unique: Vec<_> = entries.into_iter().filter(|(pid, _)| seen.insert(*pid)).collect();
        for (pid, aliases) in &unique {
            grammar.alias_sequences.insert(*pid, AliasSequence { aliases: aliases.clone() });
        }
        prop_assert_eq!(grammar.alias_sequences.len(), unique.len());
    }

    #[test]
    fn test_multiple_sequences_independent(
        a in alias_vec_strategy(8),
        b in alias_vec_strategy(8),
    ) {
        let seq_a = AliasSequence { aliases: a.clone() };
        let seq_b = AliasSequence { aliases: b.clone() };
        let mut grammar = Grammar::new("multi".to_string());
        grammar.alias_sequences.insert(ProductionId(0), seq_a);
        grammar.alias_sequences.insert(ProductionId(1), seq_b);
        prop_assert_eq!(&grammar.alias_sequences[&ProductionId(0)].aliases, &a);
        prop_assert_eq!(&grammar.alias_sequences[&ProductionId(1)].aliases, &b);
    }

    #[test]
    fn test_multiple_sequences_serde_roundtrip(
        entries in prop::collection::vec(
            (0u16..100, alias_vec_strategy(6)),
            1..=8,
        )
    ) {
        let mut grammar = Grammar::new("roundtrip".to_string());
        let mut seen = std::collections::HashSet::new();
        for (id, aliases) in &entries {
            if seen.insert(*id) {
                grammar.alias_sequences.insert(
                    ProductionId(*id),
                    AliasSequence { aliases: aliases.clone() },
                );
            }
        }
        let json = serde_json::to_string(&grammar).unwrap();
        let deserialized: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(grammar.alias_sequences, deserialized.alias_sequences);
    }

    #[test]
    fn test_max_alias_length_across_sequences(
        entries in prop::collection::vec(alias_vec_strategy(12), 1..=8)
    ) {
        let max_len = entries.iter().map(|e| e.len()).max().unwrap_or(0);
        let mut grammar = Grammar::new("maxlen".to_string());
        for (i, aliases) in entries.into_iter().enumerate() {
            grammar.alias_sequences.insert(
                ProductionId(i as u16),
                AliasSequence { aliases },
            );
        }
        grammar.max_alias_sequence_length = grammar
            .alias_sequences
            .values()
            .map(|s| s.aliases.len())
            .max()
            .unwrap_or(0);
        prop_assert_eq!(grammar.max_alias_sequence_length, max_len);
    }
}

// ── Debug format test ───────────────────────────────────────────────

proptest! {
    #[test]
    fn test_debug_format_does_not_panic(aliases in alias_vec_strategy(10)) {
        let seq = AliasSequence { aliases };
        let debug = format!("{:?}", seq);
        prop_assert!(!debug.is_empty());
    }
}
