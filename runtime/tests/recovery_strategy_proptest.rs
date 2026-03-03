#![allow(clippy::needless_range_loop)]

//! Property-based tests for `RecoveryStrategy` in the adze runtime.
//!
//! Uses proptest to verify creation, trait implementations, pattern matching,
//! round-trip conversion, and integration with `ErrorRecoveryState`.

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, ErrorNode,
    RecoveryStrategy,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// All RecoveryStrategy variants in canonical order.
const ALL_VARIANTS: [RecoveryStrategy; 7] = [
    RecoveryStrategy::PanicMode,
    RecoveryStrategy::TokenInsertion,
    RecoveryStrategy::TokenDeletion,
    RecoveryStrategy::TokenSubstitution,
    RecoveryStrategy::PhraseLevel,
    RecoveryStrategy::ScopeRecovery,
    RecoveryStrategy::IndentationRecovery,
];

/// Map an index in 0..=6 to a RecoveryStrategy variant.
fn variant_from_index(idx: usize) -> RecoveryStrategy {
    ALL_VARIANTS[idx % ALL_VARIANTS.len()]
}

/// Map a RecoveryStrategy variant to its discriminant index.
fn variant_to_index(v: RecoveryStrategy) -> usize {
    match v {
        RecoveryStrategy::PanicMode => 0,
        RecoveryStrategy::TokenInsertion => 1,
        RecoveryStrategy::TokenDeletion => 2,
        RecoveryStrategy::TokenSubstitution => 3,
        RecoveryStrategy::PhraseLevel => 4,
        RecoveryStrategy::ScopeRecovery => 5,
        RecoveryStrategy::IndentationRecovery => 6,
    }
}

/// Map a RecoveryStrategy to its expected Debug name.
fn variant_name(v: RecoveryStrategy) -> &'static str {
    match v {
        RecoveryStrategy::PanicMode => "PanicMode",
        RecoveryStrategy::TokenInsertion => "TokenInsertion",
        RecoveryStrategy::TokenDeletion => "TokenDeletion",
        RecoveryStrategy::TokenSubstitution => "TokenSubstitution",
        RecoveryStrategy::PhraseLevel => "PhraseLevel",
        RecoveryStrategy::ScopeRecovery => "ScopeRecovery",
        RecoveryStrategy::IndentationRecovery => "IndentationRecovery",
    }
}

/// Proptest strategy that yields a random RecoveryStrategy variant.
fn arb_strategy() -> impl Strategy<Value = RecoveryStrategy> {
    (0..ALL_VARIANTS.len()).prop_map(variant_from_index)
}

// ---------------------------------------------------------------------------
// 1. Variant creation tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Every variant index round-trips through variant_from_index → variant_to_index.
    #[test]
    fn test_variant_creation_roundtrip(idx in 0..7usize) {
        let v = variant_from_index(idx);
        prop_assert_eq!(variant_to_index(v), idx);
    }

    /// PanicMode is always constructible.
    #[test]
    fn test_panic_mode_creation(_dummy in 0..10u8) {
        let v = RecoveryStrategy::PanicMode;
        prop_assert_eq!(v, RecoveryStrategy::PanicMode);
    }

    /// TokenInsertion is always constructible.
    #[test]
    fn test_token_insertion_creation(_dummy in 0..10u8) {
        let v = RecoveryStrategy::TokenInsertion;
        prop_assert_eq!(v, RecoveryStrategy::TokenInsertion);
    }

    /// TokenDeletion is always constructible.
    #[test]
    fn test_token_deletion_creation(_dummy in 0..10u8) {
        let v = RecoveryStrategy::TokenDeletion;
        prop_assert_eq!(v, RecoveryStrategy::TokenDeletion);
    }

    /// TokenSubstitution is always constructible.
    #[test]
    fn test_token_substitution_creation(_dummy in 0..10u8) {
        let v = RecoveryStrategy::TokenSubstitution;
        prop_assert_eq!(v, RecoveryStrategy::TokenSubstitution);
    }

    /// PhraseLevel is always constructible.
    #[test]
    fn test_phrase_level_creation(_dummy in 0..10u8) {
        let v = RecoveryStrategy::PhraseLevel;
        prop_assert_eq!(v, RecoveryStrategy::PhraseLevel);
    }
}

// ---------------------------------------------------------------------------
// 2. Clone tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Clone produces an identical value.
    #[test]
    fn test_clone_identity(idx in 0..7usize) {
        let v = variant_from_index(idx);
        let cloned = v;
        prop_assert_eq!(v, cloned);
    }

    /// Clone of a clone is still identical (transitivity).
    #[test]
    fn test_clone_transitivity(idx in 0..7usize) {
        let a = variant_from_index(idx);
        let b = a;
        let c = b;
        prop_assert_eq!(a, c);
    }
}

// ---------------------------------------------------------------------------
// 3. Debug tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Debug output contains the variant name.
    #[test]
    fn test_debug_contains_name(idx in 0..7usize) {
        let v = variant_from_index(idx);
        let dbg = format!("{:?}", v);
        prop_assert!(dbg.contains(variant_name(v)),
            "Debug output {:?} did not contain {:?}", dbg, variant_name(v));
    }

    /// Debug output is non-empty for every variant.
    #[test]
    fn test_debug_non_empty(idx in 0..7usize) {
        let v = variant_from_index(idx);
        let dbg = format!("{:?}", v);
        prop_assert!(!dbg.is_empty());
    }

    /// Debug output is deterministic.
    #[test]
    fn test_debug_deterministic(idx in 0..7usize) {
        let v = variant_from_index(idx);
        let d1 = format!("{:?}", v);
        let d2 = format!("{:?}", v);
        prop_assert_eq!(d1, d2);
    }
}

// ---------------------------------------------------------------------------
// 4. Equality tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Reflexive: every variant equals itself.
    #[test]
    fn test_eq_reflexive(idx in 0..7usize) {
        let v = variant_from_index(idx);
        prop_assert_eq!(v, v);
    }

    /// Symmetric: a == b ⟹ b == a.
    #[test]
    fn test_eq_symmetric(a_idx in 0..7usize, b_idx in 0..7usize) {
        let a = variant_from_index(a_idx);
        let b = variant_from_index(b_idx);
        prop_assert_eq!(a == b, b == a);
    }

    /// Distinct indices produce distinct variants.
    #[test]
    fn test_ne_distinct(a_idx in 0..7usize, b_idx in 0..7usize) {
        let a = variant_from_index(a_idx);
        let b = variant_from_index(b_idx);
        if a_idx == b_idx {
            prop_assert_eq!(a, b);
        } else {
            prop_assert_ne!(a, b);
        }
    }

    /// Equality agrees with index comparison.
    #[test]
    fn test_eq_matches_index(a_idx in 0..7usize, b_idx in 0..7usize) {
        let a = variant_from_index(a_idx);
        let b = variant_from_index(b_idx);
        prop_assert_eq!(a == b, a_idx == b_idx);
    }
}

// ---------------------------------------------------------------------------
// 5. ErrorRecoveryState integration tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// An ErrorNode records whichever RecoveryStrategy is provided.
    #[test]
    fn test_error_node_records_strategy(idx in 0..7usize,
                                        start in 0..1000usize,
                                        end in 0..1000usize) {
        let strategy = variant_from_index(idx);
        let (start, end) = if start <= end { (start, end) } else { (end, start) };
        let config = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(config);
        state.record_error(start, end, (0, 0), (0, end), vec![1], Some(2), strategy, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), 1);
        prop_assert_eq!(nodes[0].recovery, strategy);
        prop_assert_eq!(nodes[0].start_byte, start);
        prop_assert_eq!(nodes[0].end_byte, end);
    }

    /// Multiple errors with different strategies are all recorded.
    #[test]
    fn test_multiple_errors_recorded(count in 1..8usize) {
        let config = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(config);
        for i in 0..count {
            let strategy = variant_from_index(i % ALL_VARIANTS.len());
            state.record_error(i, i + 1, (0, i), (0, i + 1), vec![], None, strategy, vec![]);
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), count);
        for i in 0..count {
            prop_assert_eq!(nodes[i].recovery, variant_from_index(i % ALL_VARIANTS.len()));
        }
    }

    /// ErrorRecoveryState starts with zero consecutive errors.
    #[test]
    fn test_state_starts_clean(max_err in 1..100usize) {
        let config = ErrorRecoveryConfig {
            max_consecutive_errors: max_err,
            ..Default::default()
        };
        let state = ErrorRecoveryState::new(config);
        prop_assert!(!state.should_give_up());
    }

    /// should_give_up triggers at exactly max_consecutive_errors.
    #[test]
    fn test_give_up_threshold(max_err in 1..50usize) {
        let config = ErrorRecoveryConfig {
            max_consecutive_errors: max_err,
            ..Default::default()
        };
        let mut state = ErrorRecoveryState::new(config);
        for _ in 0..max_err - 1 {
            state.increment_error_count();
            prop_assert!(!state.should_give_up());
        }
        state.increment_error_count();
        prop_assert!(state.should_give_up());
    }

    /// reset_error_count brings state back to not giving up.
    #[test]
    fn test_reset_error_count(increments in 1..30usize) {
        let config = ErrorRecoveryConfig {
            max_consecutive_errors: 5,
            ..Default::default()
        };
        let mut state = ErrorRecoveryState::new(config);
        for _ in 0..increments {
            state.increment_error_count();
        }
        state.reset_error_count();
        prop_assert!(!state.should_give_up());
    }
}

// ---------------------------------------------------------------------------
// 6. Pattern matching tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Exhaustive match covers every variant without panic.
    #[test]
    fn test_pattern_match_exhaustive(idx in 0..7usize) {
        let v = variant_from_index(idx);
        let matched = match v {
            RecoveryStrategy::PanicMode => 0,
            RecoveryStrategy::TokenInsertion => 1,
            RecoveryStrategy::TokenDeletion => 2,
            RecoveryStrategy::TokenSubstitution => 3,
            RecoveryStrategy::PhraseLevel => 4,
            RecoveryStrategy::ScopeRecovery => 5,
            RecoveryStrategy::IndentationRecovery => 6,
        };
        prop_assert_eq!(matched, idx);
    }

    /// matches! macro agrees with equality.
    #[test]
    fn test_matches_macro_agrees(idx in 0..7usize) {
        let v = variant_from_index(idx);
        prop_assert!(matches!(v, _ ));
        let is_panic = matches!(v, RecoveryStrategy::PanicMode);
        prop_assert_eq!(is_panic, idx == 0);
        let is_insertion = matches!(v, RecoveryStrategy::TokenInsertion);
        prop_assert_eq!(is_insertion, idx == 1);
    }

    /// if-let matching selects the correct variant.
    #[test]
    fn test_if_let_matching(idx in 0..5usize) {
        let v = variant_from_index(idx);
        let mut found_panic = false;
        let mut found_insertion = false;
        let mut found_deletion = false;
        let mut found_substitution = false;
        let mut found_phrase = false;
        if v == RecoveryStrategy::PanicMode { found_panic = true; }
        if v == RecoveryStrategy::TokenInsertion { found_insertion = true; }
        if v == RecoveryStrategy::TokenDeletion { found_deletion = true; }
        if v == RecoveryStrategy::TokenSubstitution { found_substitution = true; }
        if v == RecoveryStrategy::PhraseLevel { found_phrase = true; }
        let total = [found_panic, found_insertion, found_deletion, found_substitution, found_phrase]
            .iter().filter(|&&b| b).count();
        prop_assert_eq!(total, 1, "Exactly one variant should match");
    }

    /// Pattern matching a pair distinguishes all combinations.
    #[test]
    fn test_pair_pattern_matching(a_idx in 0..7usize, b_idx in 0..7usize) {
        let a = variant_from_index(a_idx);
        let b = variant_from_index(b_idx);
        let same = match (a, b) {
            (RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode) => true,
            (RecoveryStrategy::TokenInsertion, RecoveryStrategy::TokenInsertion) => true,
            (RecoveryStrategy::TokenDeletion, RecoveryStrategy::TokenDeletion) => true,
            (RecoveryStrategy::TokenSubstitution, RecoveryStrategy::TokenSubstitution) => true,
            (RecoveryStrategy::PhraseLevel, RecoveryStrategy::PhraseLevel) => true,
            (RecoveryStrategy::ScopeRecovery, RecoveryStrategy::ScopeRecovery) => true,
            (RecoveryStrategy::IndentationRecovery, RecoveryStrategy::IndentationRecovery) => true,
            _ => false,
        };
        prop_assert_eq!(same, a_idx == b_idx);
    }
}

// ---------------------------------------------------------------------------
// 7. Round-trip tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// Index → variant → index round-trip is identity.
    #[test]
    fn test_index_roundtrip(idx in 0..7usize) {
        prop_assert_eq!(variant_to_index(variant_from_index(idx)), idx);
    }

    /// Variant → index → variant round-trip is identity.
    #[test]
    fn test_variant_roundtrip(idx in 0..7usize) {
        let v = variant_from_index(idx);
        let back = variant_from_index(variant_to_index(v));
        prop_assert_eq!(v, back);
    }

    /// Debug string → match-by-name round-trip.
    #[test]
    fn test_debug_name_roundtrip(idx in 0..7usize) {
        let v = variant_from_index(idx);
        let name = variant_name(v);
        let recovered = match name {
            "PanicMode" => RecoveryStrategy::PanicMode,
            "TokenInsertion" => RecoveryStrategy::TokenInsertion,
            "TokenDeletion" => RecoveryStrategy::TokenDeletion,
            "TokenSubstitution" => RecoveryStrategy::TokenSubstitution,
            "PhraseLevel" => RecoveryStrategy::PhraseLevel,
            "ScopeRecovery" => RecoveryStrategy::ScopeRecovery,
            "IndentationRecovery" => RecoveryStrategy::IndentationRecovery,
            _ => panic!("unexpected name"),
        };
        prop_assert_eq!(v, recovered);
    }

    /// ErrorNode round-trip: record then retrieve preserves strategy.
    #[test]
    fn test_error_node_roundtrip(idx in 0..7usize,
                                  start in 0..500usize,
                                  len in 1..500usize,
                                  expected_count in 0..5usize,
                                  actual in proptest::option::of(0..1000u16)) {
        let strategy = variant_from_index(idx);
        let end = start + len;
        let expected: Vec<u16> = (0..expected_count as u16).collect();
        let config = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(config);
        state.record_error(start, end, (0, start), (0, end), expected.clone(), actual, strategy, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), 1);
        prop_assert_eq!(nodes[0].recovery, strategy);
        prop_assert_eq!(nodes[0].start_byte, start);
        prop_assert_eq!(nodes[0].end_byte, end);
        prop_assert_eq!(&nodes[0].expected, &expected);
        prop_assert_eq!(nodes[0].actual, actual);
    }

    /// Copy semantics: assigning to two bindings yields equal values.
    #[test]
    fn test_copy_roundtrip(idx in 0..7usize) {
        let a = variant_from_index(idx);
        let b = a;
        let c = b;
        prop_assert_eq!(a, b);
        prop_assert_eq!(b, c);
        prop_assert_eq!(a, c);
    }

    /// Collecting all variants into a Vec preserves order and count.
    #[test]
    fn test_all_variants_collect(_dummy in 0..1u8) {
        let collected: Vec<RecoveryStrategy> = (0..7).map(variant_from_index).collect();
        prop_assert_eq!(collected.len(), 7);
        for i in 0..7 {
            prop_assert_eq!(collected[i], ALL_VARIANTS[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Config builder integration
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Builder with configurable max_consecutive_errors respects the threshold.
    #[test]
    fn test_builder_max_errors(max_err in 1..50usize) {
        let config = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(max_err)
            .build();
        let mut state = ErrorRecoveryState::new(config);
        for _ in 0..max_err {
            state.increment_error_count();
        }
        prop_assert!(state.should_give_up());
    }

    /// Builder with scope delimiters enables scope tracking via ErrorNode.
    #[test]
    fn test_builder_scope_delimiters(open in 1..100u16, close in 100..200u16) {
        let config = ErrorRecoveryConfigBuilder::new()
            .add_scope_delimiter(open, close)
            .build();
        let delimiters = &config.scope_delimiters;
        prop_assert!(ErrorRecoveryState::is_scope_delimiter(open, delimiters));
        prop_assert!(ErrorRecoveryState::is_scope_delimiter(close, delimiters));
        prop_assert!(ErrorRecoveryState::is_matching_delimiter(open, close, delimiters));
    }
}
