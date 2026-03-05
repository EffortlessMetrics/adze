//! Property-based tests for error recovery: config, mode transitions, state tracking,
//! error node ordering, and determinism.

use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState,
    RecoveryStrategy,
};
use proptest::prelude::*;

// --- Strategies ---

fn arb_recovery_strategy() -> impl Strategy<Value = RecoveryStrategy> {
    prop_oneof![
        Just(RecoveryStrategy::PanicMode),
        Just(RecoveryStrategy::TokenInsertion),
        Just(RecoveryStrategy::TokenDeletion),
        Just(RecoveryStrategy::TokenSubstitution),
        Just(RecoveryStrategy::PhraseLevel),
        Just(RecoveryStrategy::ScopeRecovery),
        Just(RecoveryStrategy::IndentationRecovery),
    ]
}

fn arb_position() -> impl Strategy<Value = (usize, usize)> {
    (0usize..10_000, 0usize..500)
}

fn arb_expected() -> impl Strategy<Value = Vec<u16>> {
    prop::collection::vec(0u16..1000, 0..20)
}

fn arb_actual() -> impl Strategy<Value = Option<u16>> {
    prop::option::of(0u16..1000)
}

fn arb_skipped_tokens() -> impl Strategy<Value = Vec<u16>> {
    prop::collection::vec(0u16..1000, 0..10)
}

fn arb_scope_delimiters() -> impl Strategy<Value = Vec<(u16, u16)>> {
    prop::collection::vec((1u16..500, 501u16..1000), 0..5)
}

/// Helper: record one error and return the collected nodes.
fn record_one(
    state: &mut ErrorRecoveryState,
    start_byte: usize,
    end_byte: usize,
    start_pos: (usize, usize),
    end_pos: (usize, usize),
    expected: Vec<u16>,
    actual: Option<u16>,
    recovery: RecoveryStrategy,
    skipped: Vec<u16>,
) -> Vec<ErrorNode> {
    state.record_error(
        start_byte, end_byte, start_pos, end_pos, expected, actual, recovery, skipped,
    );
    state.get_error_nodes()
}

// ========================================================================
// Config properties
// ========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    // 1. Default config has expected field values
    #[test]
    fn prop_default_config_max_panic_skip(_ in 0u8..1) {
        let cfg = ErrorRecoveryConfig::default();
        prop_assert_eq!(cfg.max_panic_skip, 50);
        prop_assert_eq!(cfg.max_token_deletions, 3);
        prop_assert_eq!(cfg.max_token_insertions, 2);
        prop_assert_eq!(cfg.max_consecutive_errors, 10);
        prop_assert!(cfg.enable_phrase_recovery);
        prop_assert!(cfg.enable_scope_recovery);
        prop_assert!(!cfg.enable_indentation_recovery);
        prop_assert!(cfg.sync_tokens.is_empty());
        prop_assert!(cfg.insert_candidates.is_empty());
        prop_assert!(cfg.deletable_tokens.is_empty());
        prop_assert!(cfg.scope_delimiters.is_empty());
    }

    // 2. Builder max_panic_skip round-trips
    #[test]
    fn prop_builder_max_panic_skip(val in 1usize..10_000) {
        let cfg = ErrorRecoveryConfigBuilder::new().max_panic_skip(val).build();
        prop_assert_eq!(cfg.max_panic_skip, val);
    }

    // 3. Builder max_consecutive_errors round-trips
    #[test]
    fn prop_builder_max_consecutive_errors(val in 1usize..10_000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(val)
            .build();
        prop_assert_eq!(cfg.max_consecutive_errors, val);
    }

    // 4. Builder set_max_recovery_attempts aliases max_consecutive_errors
    #[test]
    fn prop_builder_recovery_attempts_alias(val in 1usize..500) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .set_max_recovery_attempts(val)
            .build();
        prop_assert_eq!(cfg.max_consecutive_errors, val);
    }

    // 5. Builder sync tokens accumulate
    #[test]
    fn prop_builder_sync_tokens(tokens in prop::collection::vec(0u16..1000, 1..20)) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for &tok in &tokens {
            builder = builder.add_sync_token(tok);
        }
        let cfg = builder.build();
        prop_assert_eq!(cfg.sync_tokens.len(), tokens.len());
        for (i, &tok) in tokens.iter().enumerate() {
            prop_assert_eq!(cfg.sync_tokens[i].0, tok);
        }
    }

    // 6. Builder insertable tokens accumulate
    #[test]
    fn prop_builder_insertable_tokens(tokens in prop::collection::vec(0u16..1000, 1..15)) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for &tok in &tokens {
            builder = builder.add_insertable_token(tok);
        }
        let cfg = builder.build();
        prop_assert_eq!(cfg.insert_candidates.len(), tokens.len());
    }

    // 7. Builder deletable tokens accumulate (set semantics)
    #[test]
    fn prop_builder_deletable_tokens(tokens in prop::collection::vec(0u16..1000, 1..30)) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for &tok in &tokens {
            builder = builder.add_deletable_token(tok);
        }
        let cfg = builder.build();
        for &tok in &tokens {
            prop_assert!(cfg.deletable_tokens.contains(&tok));
        }
    }

    // 8. Builder scope delimiters accumulate
    #[test]
    fn prop_builder_scope_delimiters(delims in arb_scope_delimiters()) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for &(open, close) in &delims {
            builder = builder.add_scope_delimiter(open, close);
        }
        let cfg = builder.build();
        prop_assert_eq!(cfg.scope_delimiters.len(), delims.len());
        for (i, &(open, close)) in delims.iter().enumerate() {
            prop_assert_eq!(cfg.scope_delimiters[i], (open, close));
        }
    }

    // 9. Builder boolean flags round-trip
    #[test]
    fn prop_builder_boolean_flags(
        phrase_val in proptest::bool::ANY,
        scope_val in proptest::bool::ANY,
        indent_val in proptest::bool::ANY,
    ) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .enable_phrase_recovery(phrase_val)
            .enable_scope_recovery(scope_val)
            .enable_indentation_recovery(indent_val)
            .build();
        prop_assert_eq!(cfg.enable_phrase_recovery, phrase_val);
        prop_assert_eq!(cfg.enable_scope_recovery, scope_val);
        prop_assert_eq!(cfg.enable_indentation_recovery, indent_val);
    }

    // 10. can_delete_token is false for sync tokens not in deletable set
    #[test]
    fn prop_sync_token_not_deletable(tok in 0u16..500) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .add_sync_token(tok)
            .build();
        // Sync token alone is not deletable (it IS a sync token and not explicitly deletable)
        prop_assert!(!cfg.can_delete_token(adze_ir::SymbolId(tok)));
    }

    // 11. can_delete_token is true for explicitly deletable tokens even if sync
    #[test]
    fn prop_deletable_overrides_sync(tok in 0u16..500) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .add_sync_token(tok)
            .add_deletable_token(tok)
            .build();
        prop_assert!(cfg.can_delete_token(adze_ir::SymbolId(tok)));
    }

    // 12. can_replace_token is false for sync tokens
    #[test]
    fn prop_cannot_replace_sync_token(tok in 0u16..500) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .add_sync_token(tok)
            .build();
        prop_assert!(!cfg.can_replace_token(adze_ir::SymbolId(tok)));
    }

    // 13. can_replace_token is true for non-sync tokens
    #[test]
    fn prop_can_replace_non_sync(tok in 501u16..1000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .add_sync_token(0)
            .build();
        prop_assert!(cfg.can_replace_token(adze_ir::SymbolId(tok)));
    }

    // ====================================================================
    // State tracking consistency
    // ====================================================================

    // 14. Fresh state has no error nodes
    #[test]
    fn prop_fresh_state_empty(max in 1usize..200) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(max)
            .build();
        let state = ErrorRecoveryState::new(cfg);
        prop_assert!(state.get_error_nodes().is_empty());
    }

    // 15. increment / should_give_up tracks threshold
    #[test]
    fn prop_give_up_at_threshold(threshold in 1usize..50) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(threshold)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        for _ in 0..threshold {
            prop_assert!(!state.should_give_up());
            state.increment_error_count();
        }
        prop_assert!(state.should_give_up());
    }

    // 16. reset_error_count clears the counter
    #[test]
    fn prop_reset_error_count(bumps in 1usize..30) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(100)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        for _ in 0..bumps {
            state.increment_error_count();
        }
        state.reset_error_count();
        prop_assert!(!state.should_give_up());
    }

    // 17. reset_consecutive_errors does not clear recorded nodes
    #[test]
    fn prop_reset_consecutive_preserves_nodes(count in 1usize..20) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..count {
            state.record_error(
                i, i + 1, (0, 0), (0, 1), vec![], None,
                RecoveryStrategy::PanicMode, vec![],
            );
        }
        state.reset_consecutive_errors();
        prop_assert_eq!(state.get_error_nodes().len(), count);
    }

    // 18. clear_errors removes all nodes but state remains usable
    #[test]
    fn prop_clear_then_reuse(n in 1usize..20) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..n {
            state.record_error(
                i, i + 1, (0, 0), (0, 1), vec![], None,
                RecoveryStrategy::PanicMode, vec![],
            );
        }
        state.clear_errors();
        prop_assert!(state.get_error_nodes().is_empty());
        // Re-record after clear
        state.record_error(0, 1, (0, 0), (0, 1), vec![42], None, RecoveryStrategy::TokenDeletion, vec![]);
        prop_assert_eq!(state.get_error_nodes().len(), 1);
        prop_assert_eq!(&state.get_error_nodes()[0].expected, &[42u16]);
    }

    // 19. Multiple clear cycles always leave empty state
    #[test]
    fn prop_repeated_clear_cycles(rounds in 1usize..8) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for _ in 0..rounds {
            state.record_error(
                0, 1, (0, 0), (0, 1), vec![], None,
                RecoveryStrategy::PanicMode, vec![],
            );
            state.clear_errors();
        }
        prop_assert!(state.get_error_nodes().is_empty());
    }

    // 20. get_error_nodes snapshot is independent of later mutations
    #[test]
    fn prop_snapshot_independence(n in 1usize..15) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..n {
            state.record_error(
                i, i + 1, (0, 0), (0, 1), vec![], None,
                RecoveryStrategy::PanicMode, vec![],
            );
        }
        let snap = state.get_error_nodes();
        state.record_error(
            999, 1000, (0, 0), (0, 1), vec![], None,
            RecoveryStrategy::PanicMode, vec![],
        );
        prop_assert_eq!(snap.len(), n);
        prop_assert_eq!(state.get_error_nodes().len(), n + 1);
    }

    // ====================================================================
    // Error node field preservation
    // ====================================================================

    // 21. Full round-trip of all ErrorNode fields
    #[test]
    fn prop_error_node_full_roundtrip(
        sb in 0usize..50_000,
        eb in 0usize..50_000,
        sp in arb_position(),
        ep in arb_position(),
        expected in arb_expected(),
        actual in arb_actual(),
        strat in arb_recovery_strategy(),
        skipped in arb_skipped_tokens(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        let nodes = record_one(&mut state, sb, eb, sp, ep, expected.clone(), actual, strat, skipped.clone());
        let node = &nodes[0];
        prop_assert_eq!(node.start_byte, sb);
        prop_assert_eq!(node.end_byte, eb);
        prop_assert_eq!(node.start_position, sp);
        prop_assert_eq!(node.end_position, ep);
        prop_assert_eq!(&node.expected, &expected);
        prop_assert_eq!(node.actual, actual);
        prop_assert_eq!(node.recovery, strat);
        prop_assert_eq!(&node.skipped_tokens, &skipped);
    }

    // 22. Zero-length error span (start == end) preserved
    #[test]
    fn prop_zero_length_span(pos in 0usize..100_000) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        let nodes = record_one(
            &mut state, pos, pos, (0, 0), (0, 0),
            vec![], None, RecoveryStrategy::TokenInsertion, vec![],
        );
        prop_assert_eq!(nodes[0].start_byte, pos);
        prop_assert_eq!(nodes[0].end_byte, pos);
    }

    // 23. u16::MAX token values preserved in expected, actual, skipped
    #[test]
    fn prop_max_u16_tokens(tok in (u16::MAX - 100)..=u16::MAX) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        let nodes = record_one(
            &mut state, 0, 1, (0, 0), (0, 1),
            vec![tok], Some(tok), RecoveryStrategy::PanicMode, vec![tok],
        );
        prop_assert_eq!(&nodes[0].expected, &vec![tok]);
        prop_assert_eq!(nodes[0].actual, Some(tok));
        prop_assert_eq!(&nodes[0].skipped_tokens, &vec![tok]);
    }

    // 24. u16::MIN (0) token value preserved
    #[test]
    fn prop_min_u16_token(_ in 0u8..1) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        let nodes = record_one(
            &mut state, 0, 1, (0, 0), (0, 1),
            vec![0], Some(0), RecoveryStrategy::PanicMode, vec![0],
        );
        prop_assert_eq!(nodes[0].actual, Some(0u16));
        prop_assert_eq!(&nodes[0].expected, &[0u16]);
    }

    // ====================================================================
    // Error node ordering
    // ====================================================================

    // 25. Insertion order preserved for sequential byte offsets
    #[test]
    fn prop_insertion_order_sequential(n in 2usize..40) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..n {
            state.record_error(
                i * 10, i * 10 + 5, (i, 0), (i, 5), vec![], None,
                RecoveryStrategy::PanicMode, vec![],
            );
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), n);
        for i in 0..n {
            prop_assert_eq!(nodes[i].start_byte, i * 10);
        }
    }

    // 26. Arbitrary offsets preserve insertion order (not sorted)
    #[test]
    fn prop_arbitrary_offsets_order(offsets in prop::collection::vec(0usize..100_000, 2..30)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for &off in &offsets {
            state.record_error(
                off, off + 1, (0, 0), (0, 1), vec![], None,
                RecoveryStrategy::PanicMode, vec![],
            );
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), offsets.len());
        for (node, &off) in nodes.iter().zip(offsets.iter()) {
            prop_assert_eq!(node.start_byte, off);
        }
    }

    // 27. Interleaved strategies maintain per-node correctness
    #[test]
    fn prop_interleaved_strategies(
        strats in prop::collection::vec(arb_recovery_strategy(), 3..20),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for (i, &strat) in strats.iter().enumerate() {
            state.record_error(
                i, i + 1, (0, 0), (0, 1), vec![], None, strat, vec![],
            );
        }
        let nodes = state.get_error_nodes();
        for (i, &strat) in strats.iter().enumerate() {
            prop_assert_eq!(nodes[i].recovery, strat);
        }
    }

    // 28. Batch error count always matches number of recordings
    #[test]
    fn prop_batch_count(batches in prop::collection::vec(1usize..6, 1..8)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        let mut total = 0usize;
        for batch_size in &batches {
            for _ in 0..*batch_size {
                state.record_error(
                    total, total + 1, (0, 0), (0, 1), vec![], None,
                    RecoveryStrategy::PanicMode, vec![],
                );
                total += 1;
            }
        }
        prop_assert_eq!(state.get_error_nodes().len(), total);
    }

    // 29. Large expected-token vectors preserved exactly
    #[test]
    fn prop_large_expected_vec(expected in prop::collection::vec(0u16..5000, 50..200)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(
            0, 1, (0, 0), (0, 1), expected.clone(), None,
            RecoveryStrategy::PanicMode, vec![],
        );
        prop_assert_eq!(&state.get_error_nodes()[0].expected, &expected);
    }

    // ====================================================================
    // Mode / strategy transitions
    // ====================================================================

    // 30. determine_recovery_strategy falls to PanicMode when over threshold
    #[test]
    fn prop_panic_mode_on_exceed(threshold in 1usize..20) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(threshold)
            .enable_phrase_recovery(false)
            .enable_scope_recovery(false)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        // Exhaust the threshold
        for _ in 0..=threshold {
            state.increment_error_count();
        }
        let strat = state.determine_recovery_strategy(&[], None, (0, 0), 0);
        prop_assert_eq!(strat, RecoveryStrategy::PanicMode);
    }

    // 31. determine_recovery_strategy yields PhraseLevel when phrase recovery enabled
    //     and no insertion / deletion / substitution is possible (actual=None, no insertable)
    #[test]
    fn prop_phrase_level_fallback(_ in 0u8..1) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(100)
            .enable_phrase_recovery(true)
            .enable_scope_recovery(false)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        // actual=None skips deletion/substitution; no insert_candidates skips insertion
        let strat = state.determine_recovery_strategy(&[10, 20], None, (0, 0), 0);
        prop_assert_eq!(strat, RecoveryStrategy::PhraseLevel);
    }

    // 32. determine_recovery_strategy yields TokenDeletion for clearly-wrong token
    //     when phrase recovery is disabled
    #[test]
    fn prop_token_deletion_for_wrong_token(
        actual_tok in 100u16..500,
    ) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(100)
            .enable_phrase_recovery(false)
            .enable_scope_recovery(false)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        // expected tokens don't include actual, >1 expected so substitution is skipped
        let strat = state.determine_recovery_strategy(
            &[actual_tok + 1, actual_tok + 2],
            Some(actual_tok),
            (0, 0),
            0,
        );
        prop_assert_eq!(strat, RecoveryStrategy::TokenDeletion);
    }

    // 33. determine_recovery_strategy yields TokenSubstitution when exactly 1 expected,
    //     actual is a sync token (so is_clearly_wrong=false), and phrase/scope disabled
    #[test]
    fn prop_token_substitution_single_expected(
        actual_tok in 100u16..500,
        expected_tok in 501u16..1000,
    ) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(100)
            .add_sync_token(actual_tok)
            .enable_phrase_recovery(false)
            .enable_scope_recovery(false)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        let strat = state.determine_recovery_strategy(
            &[expected_tok],
            Some(actual_tok),
            (0, 0),
            0,
        );
        prop_assert_eq!(strat, RecoveryStrategy::TokenSubstitution);
    }

    // 34. RecoveryStrategy Copy — all seven variants equal themselves
    #[test]
    fn prop_recovery_strategy_copy_eq(strat in arb_recovery_strategy()) {
        let copied = strat;  // Copy, not clone
        prop_assert_eq!(copied, strat);
    }

    // 35. All seven strategy variants are pairwise distinct
    #[test]
    fn prop_all_strategies_distinct(_ in 0u8..1) {
        let all = [
            RecoveryStrategy::PanicMode,
            RecoveryStrategy::TokenInsertion,
            RecoveryStrategy::TokenDeletion,
            RecoveryStrategy::TokenSubstitution,
            RecoveryStrategy::PhraseLevel,
            RecoveryStrategy::ScopeRecovery,
            RecoveryStrategy::IndentationRecovery,
        ];
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                prop_assert_ne!(all[i], all[j]);
            }
        }
    }

    // ====================================================================
    // Scope tracking
    // ====================================================================

    // 36. push_scope + pop_scope round-trip with matching delimiters
    #[test]
    fn prop_scope_push_pop_roundtrip(open in 1u16..100, close in 101u16..200) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .add_scope_delimiter(open, close)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        state.push_scope(open);
        let popped = state.pop_scope(close);
        prop_assert!(popped);
    }

    // 37. pop_scope returns false for non-matching delimiter
    #[test]
    fn prop_scope_pop_mismatch(open in 1u16..100, close in 101u16..200) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .add_scope_delimiter(open, close)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        state.push_scope(open);
        // Try to pop with a non-registered close token
        let popped = state.pop_scope(close + 1);
        prop_assert!(!popped);
    }

    // 38. Nested scopes: LIFO ordering
    #[test]
    fn prop_nested_scopes_lifo(_ in 0u8..1) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .add_scope_delimiter(10, 11)
            .add_scope_delimiter(20, 21)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        state.push_scope(10);
        state.push_scope(20);
        // Must pop inner first
        prop_assert!(state.pop_scope(21));
        prop_assert!(state.pop_scope(11));
    }

    // 39. Scope ops do not create error nodes
    #[test]
    fn prop_scope_ops_no_errors(open in 1u16..100, close in 101u16..200) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .add_scope_delimiter(open, close)
            .build();
        let mut state = ErrorRecoveryState::new(cfg);
        state.push_scope(open);
        state.pop_scope(close);
        prop_assert!(state.get_error_nodes().is_empty());
    }

    // ====================================================================
    // Recent tokens
    // ====================================================================

    // 40. add_recent_token does not produce error nodes
    #[test]
    fn prop_recent_token_no_errors(tok in 0u16..1000) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.add_recent_token(tok);
        prop_assert!(state.get_error_nodes().is_empty());
    }

    // ====================================================================
    // Determinism
    // ====================================================================

    // 41. Same inputs produce identical error-node lists
    #[test]
    fn prop_deterministic_record(
        sb in 0usize..10_000,
        eb in 0usize..10_000,
        sp in arb_position(),
        ep in arb_position(),
        expected in arb_expected(),
        actual in arb_actual(),
        strat in arb_recovery_strategy(),
        skipped in arb_skipped_tokens(),
    ) {
        let mut s1 = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        let mut s2 = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        s1.record_error(sb, eb, sp, ep, expected.clone(), actual, strat, skipped.clone());
        s2.record_error(sb, eb, sp, ep, expected.clone(), actual, strat, skipped.clone());
        let n1 = s1.get_error_nodes();
        let n2 = s2.get_error_nodes();
        prop_assert_eq!(n1.len(), n2.len());
        prop_assert_eq!(n1[0].start_byte, n2[0].start_byte);
        prop_assert_eq!(n1[0].end_byte, n2[0].end_byte);
        prop_assert_eq!(n1[0].recovery, n2[0].recovery);
        prop_assert_eq!(&n1[0].expected, &n2[0].expected);
        prop_assert_eq!(n1[0].actual, n2[0].actual);
        prop_assert_eq!(&n1[0].skipped_tokens, &n2[0].skipped_tokens);
    }

    // 42. determine_recovery_strategy is deterministic for same inputs
    #[test]
    fn prop_deterministic_strategy(
        expected_toks in prop::collection::vec(0u16..1000, 0..10),
        actual in arb_actual(),
    ) {
        let build = || {
            ErrorRecoveryConfigBuilder::new()
                .max_consecutive_errors(100)
                .enable_phrase_recovery(false)
                .enable_scope_recovery(false)
                .build()
        };
        let mut s1 = ErrorRecoveryState::new(build());
        let mut s2 = ErrorRecoveryState::new(build());
        let r1 = s1.determine_recovery_strategy(&expected_toks, actual, (0, 0), 0);
        let r2 = s2.determine_recovery_strategy(&expected_toks, actual, (0, 0), 0);
        prop_assert_eq!(r1, r2);
    }

    // ====================================================================
    // Static helpers
    // ====================================================================

    // 43. is_scope_delimiter returns true for open/close tokens in delimiter set
    #[test]
    fn prop_is_scope_delimiter_open(open in 1u16..500, close in 501u16..1000) {
        let delimiters = vec![(open, close)];
        prop_assert!(ErrorRecoveryState::is_scope_delimiter(open, &delimiters));
        prop_assert!(ErrorRecoveryState::is_scope_delimiter(close, &delimiters));
    }

    // 44. is_scope_delimiter returns false for unregistered token
    #[test]
    fn prop_is_scope_delimiter_false(tok in 1001u16..2000) {
        let delimiters = vec![(1, 2), (3, 4)];
        prop_assert!(!ErrorRecoveryState::is_scope_delimiter(tok, &delimiters));
    }

    // 45. is_matching_delimiter positive case
    #[test]
    fn prop_matching_delimiter(open in 1u16..500, close in 501u16..1000) {
        let delimiters = vec![(open, close)];
        prop_assert!(ErrorRecoveryState::is_matching_delimiter(
            open, close, &delimiters,
        ));
    }

    // 46. is_matching_delimiter negative case — swapped
    #[test]
    fn prop_matching_delimiter_swapped(open in 1u16..500, close in 501u16..1000) {
        let delimiters = vec![(open, close)];
        prop_assert!(!ErrorRecoveryState::is_matching_delimiter(
            close, open, &delimiters,
        ));
    }
}
