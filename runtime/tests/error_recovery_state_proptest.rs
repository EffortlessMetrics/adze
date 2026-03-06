#![allow(clippy::needless_range_loop)]

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryStrategy,
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

// --- Tests ---

proptest! {
    // 1. ErrorRecoveryState creation with default config
    #[test]
    fn test_creation_default_has_no_errors(_ in 0u8..1) {
        let state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        prop_assert!(state.get_error_nodes().is_empty());
    }

    // 2. Creation with custom max_consecutive_errors
    #[test]
    fn test_creation_with_custom_max(max in 1usize..100) {
        let config = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(max)
            .build();
        let state = ErrorRecoveryState::new(config);
        prop_assert!(state.get_error_nodes().is_empty());
    }

    // 3. Single record_error then get_error_nodes returns one entry
    #[test]
    fn test_single_record_error(
        start_byte in 0usize..10_000,
        end_byte in 0usize..10_000,
        start_pos in arb_position(),
        end_pos in arb_position(),
        expected in arb_expected(),
        actual in arb_actual(),
        recovery in arb_recovery_strategy(),
        skipped in arb_skipped_tokens(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(
            start_byte, end_byte, start_pos, end_pos,
            expected.clone(), actual, recovery, skipped.clone(),
        );
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), 1);
    }

    // 4. Recorded error preserves start_byte
    #[test]
    fn test_start_byte_preserved(start in 0usize..100_000) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(start, start + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].start_byte, start);
    }

    // 5. Recorded error preserves end_byte
    #[test]
    fn test_end_byte_preserved(end in 1usize..100_000) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, end, (0, 0), (0, 0), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].end_byte, end);
    }

    // 6. Recorded error preserves start_position
    #[test]
    fn test_start_position_preserved(row in 0usize..10_000, col in 0usize..500) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (row, col), (0, 0), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].start_position, (row, col));
    }

    // 7. Recorded error preserves end_position
    #[test]
    fn test_end_position_preserved(row in 0usize..10_000, col in 0usize..500) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (row, col), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].end_position, (row, col));
    }

    // 8. Recorded error preserves expected tokens
    #[test]
    fn test_expected_tokens_preserved(expected in arb_expected()) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), expected.clone(), None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(&state.get_error_nodes()[0].expected, &expected);
    }

    // 9. Recorded error preserves actual token (Some)
    #[test]
    fn test_actual_some_preserved(tok in 0u16..1000) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], Some(tok), RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].actual, Some(tok));
    }

    // 10. Recorded error preserves actual token (None)
    #[test]
    fn test_actual_none_preserved(_ in 0u8..1) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].actual, None);
    }

    // 11. Recorded error preserves recovery strategy
    #[test]
    fn test_recovery_strategy_preserved(strategy in arb_recovery_strategy()) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, strategy, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].recovery, strategy);
    }

    // 12. Multiple errors: count matches
    #[test]
    fn test_multiple_error_count(n in 1usize..50) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..n {
            state.record_error(i, i + 1, (0, i), (0, i + 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        prop_assert_eq!(state.get_error_nodes().len(), n);
    }

    // 13. Multiple errors: each start_byte is correct
    #[test]
    fn test_multiple_error_start_bytes(n in 1usize..30) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..n {
            state.record_error(i * 10, i * 10 + 5, (0, 0), (0, 5), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        for i in 0..n {
            prop_assert_eq!(nodes[i].start_byte, i * 10);
        }
    }

    // 14. Multiple errors: ordering is preserved
    #[test]
    fn test_error_ordering(offsets in prop::collection::vec(0usize..10_000, 1..20)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for &off in &offsets {
            state.record_error(off, off + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), offsets.len());
        for i in 0..offsets.len() {
            prop_assert_eq!(nodes[i].start_byte, offsets[i]);
        }
    }

    // 15. Multiple errors with different strategies
    #[test]
    fn test_multiple_strategies(strategies in prop::collection::vec(arb_recovery_strategy(), 1..20)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for (i, strat) in strategies.iter().enumerate() {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, *strat, vec![]);
        }
        let nodes = state.get_error_nodes();
        for i in 0..strategies.len() {
            prop_assert_eq!(nodes[i].recovery, strategies[i]);
        }
    }

    // 16. Error byte range preserved
    #[test]
    fn test_byte_range(start in 0usize..50_000, len in 1usize..1000) {
        let end = start + len;
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(start, end, (0, 0), (0, 0), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.start_byte, start);
        prop_assert_eq!(node.end_byte, end);
        prop_assert!(node.end_byte > node.start_byte);
    }

    // 17. Position row/col both preserved across multiple errors
    #[test]
    fn test_positions_multi(
        rows in prop::collection::vec(0usize..5000, 2..10),
        cols in prop::collection::vec(0usize..200, 2..10),
    ) {
        let count = rows.len().min(cols.len());
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..count {
            state.record_error(i, i + 1, (rows[i], cols[i]), (rows[i], cols[i] + 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), count);
        for i in 0..count {
            prop_assert_eq!(nodes[i].start_position, (rows[i], cols[i]));
        }
    }

    // 18. Expected tokens of varying lengths
    #[test]
    fn test_varying_expected_lengths(
        expected_lists in prop::collection::vec(arb_expected(), 1..10),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for (i, exp) in expected_lists.iter().enumerate() {
            state.record_error(i, i + 1, (0, 0), (0, 1), exp.clone(), None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        for i in 0..expected_lists.len() {
            prop_assert_eq!(&nodes[i].expected, &expected_lists[i]);
        }
    }

    // 19. Skipped tokens preserved
    #[test]
    fn test_skipped_tokens_preserved(skipped in arb_skipped_tokens()) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, skipped.clone());
        prop_assert_eq!(&state.get_error_nodes()[0].skipped_tokens, &skipped);
    }

    // 20. Empty expected tokens
    #[test]
    fn test_empty_expected(_ in 0u8..1) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert!(state.get_error_nodes()[0].expected.is_empty());
    }

    // 21. get_error_nodes returns independent clone
    #[test]
    fn test_get_error_nodes_is_independent(n in 1usize..10) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..n {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let snapshot = state.get_error_nodes();
        // Record another error after snapshot
        state.record_error(999, 1000, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        // Snapshot should be unchanged
        prop_assert_eq!(snapshot.len(), n);
        // But live state should have one more
        prop_assert_eq!(state.get_error_nodes().len(), n + 1);
    }

    // 22. Zero-length error range (start == end)
    #[test]
    fn test_zero_length_error(pos in 0usize..100_000) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(pos, pos, (0, 0), (0, 0), vec![], None, RecoveryStrategy::TokenInsertion, vec![]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.start_byte, pos);
        prop_assert_eq!(node.end_byte, pos);
    }

    // 23. Token insertion strategy preserved
    #[test]
    fn test_token_insertion_strategy(n in 1usize..20) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..n {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![42], None, RecoveryStrategy::TokenInsertion, vec![]);
        }
        let nodes = state.get_error_nodes();
        for i in 0..n {
            prop_assert_eq!(nodes[i].recovery, RecoveryStrategy::TokenInsertion);
        }
    }

    // 24. Token deletion strategy preserved
    #[test]
    fn test_token_deletion_strategy(actual_tok in 0u16..1000) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![1, 2], Some(actual_tok), RecoveryStrategy::TokenDeletion, vec![actual_tok]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.recovery, RecoveryStrategy::TokenDeletion);
        prop_assert_eq!(node.actual, Some(actual_tok));
    }

    // 25. Token substitution strategy preserved
    #[test]
    fn test_token_substitution_strategy(actual_tok in 0u16..1000, expected_tok in 0u16..1000) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 5, (0, 0), (0, 5), vec![expected_tok], Some(actual_tok), RecoveryStrategy::TokenSubstitution, vec![]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.recovery, RecoveryStrategy::TokenSubstitution);
        prop_assert_eq!(&node.expected, &vec![expected_tok]);
        prop_assert_eq!(node.actual, Some(actual_tok));
    }

    // 26. Error count via get_error_nodes().len() matches recordings
    #[test]
    fn test_error_count_matches(counts in prop::collection::vec(1usize..5, 1..8)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        let mut total = 0usize;
        for batch in &counts {
            for _ in 0..*batch {
                state.record_error(total, total + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
                total += 1;
            }
        }
        prop_assert_eq!(state.get_error_nodes().len(), total);
    }

    // 27. Large expected token list preserved
    #[test]
    fn test_large_expected(expected in prop::collection::vec(0u16..5000, 50..200)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), expected.clone(), None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(&state.get_error_nodes()[0].expected, &expected);
    }

    // 28. Scope recovery strategy preserved
    #[test]
    fn test_scope_recovery_strategy(start in 0usize..10_000, end in 0usize..10_000) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(start, end, (0, 0), (0, 0), vec![], None, RecoveryStrategy::ScopeRecovery, vec![]);
        prop_assert_eq!(state.get_error_nodes()[0].recovery, RecoveryStrategy::ScopeRecovery);
    }

    // 29. Phrase level recovery strategy preserved
    #[test]
    fn test_phrase_level_strategy(expected in arb_expected()) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 10, (0, 0), (0, 10), expected.clone(), None, RecoveryStrategy::PhraseLevel, vec![]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.recovery, RecoveryStrategy::PhraseLevel);
        prop_assert_eq!(&node.expected, &expected);
    }

    // 30. Interleaved strategies ordering
    #[test]
    fn test_interleaved_strategies(
        strats in prop::collection::vec(arb_recovery_strategy(), 5..15),
        actuals in prop::collection::vec(arb_actual(), 5..15),
    ) {
        let count = strats.len().min(actuals.len());
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..count {
            state.record_error(i, i + 1, (i, 0), (i, 1), vec![], actuals[i], strats[i], vec![]);
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), count);
        for i in 0..count {
            prop_assert_eq!(nodes[i].recovery, strats[i]);
            prop_assert_eq!(nodes[i].actual, actuals[i]);
        }
    }

    // 31. Max u16 token values
    #[test]
    fn test_max_token_values(tok in (u16::MAX - 100)..=u16::MAX) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![tok], Some(tok), RecoveryStrategy::PanicMode, vec![tok]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(&node.expected, &vec![tok]);
        prop_assert_eq!(node.actual, Some(tok));
        prop_assert_eq!(&node.skipped_tokens, &vec![tok]);
    }

    // 32. End position tracks line progression
    #[test]
    fn test_line_progression(lines in 1usize..100) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for line in 0..lines {
            state.record_error(line * 80, (line + 1) * 80, (line, 0), (line, 80), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), lines);
        for line in 0..lines {
            prop_assert_eq!(nodes[line].start_position.0, line);
            prop_assert_eq!(nodes[line].end_position.0, line);
        }
    }

    // 33. Full round-trip: all fields preserved together
    #[test]
    fn test_full_roundtrip(
        sb in 0usize..10_000,
        eb in 0usize..10_000,
        sp in arb_position(),
        ep in arb_position(),
        expected in arb_expected(),
        actual in arb_actual(),
        recovery in arb_recovery_strategy(),
        skipped in arb_skipped_tokens(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(sb, eb, sp, ep, expected.clone(), actual, recovery, skipped.clone());
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.start_byte, sb);
        prop_assert_eq!(node.end_byte, eb);
        prop_assert_eq!(node.start_position, sp);
        prop_assert_eq!(node.end_position, ep);
        prop_assert_eq!(&node.expected, &expected);
        prop_assert_eq!(node.actual, actual);
        prop_assert_eq!(node.recovery, recovery);
        prop_assert_eq!(&node.skipped_tokens, &skipped);
    }

    // 34. Builder config propagates to state behavior
    #[test]
    fn test_builder_config_propagation(max_skip in 1usize..200) {
        let config = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(max_skip)
            .build();
        let state = ErrorRecoveryState::new(config);
        prop_assert!(state.get_error_nodes().is_empty());
    }

    // 35. Indentation recovery strategy preserved
    #[test]
    fn test_indentation_recovery_strategy(
        indent_col in 0usize..100,
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 10, (0, indent_col), (1, 0), vec![], None, RecoveryStrategy::IndentationRecovery, vec![]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.recovery, RecoveryStrategy::IndentationRecovery);
        prop_assert_eq!(node.start_position.1, indent_col);
    }

    // 36. clear_errors then re-record produces fresh list
    #[test]
    fn test_clear_then_rerecord(n in 1usize..20) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..n {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        state.clear_errors();
        state.record_error(999, 1000, (0, 0), (0, 1), vec![42], Some(7), RecoveryStrategy::TokenDeletion, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), 1);
        prop_assert_eq!(nodes[0].start_byte, 999);
        prop_assert_eq!(nodes[0].recovery, RecoveryStrategy::TokenDeletion);
    }

    // 37. Multiple clears leave state empty
    #[test]
    fn test_repeated_clears(rounds in 1usize..5) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for _ in 0..rounds {
            state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
            state.clear_errors();
        }
        prop_assert!(state.get_error_nodes().is_empty());
    }

    // 38. Skipped tokens length matches across multiple errors
    #[test]
    fn test_skipped_tokens_lengths(
        skipped_lists in prop::collection::vec(arb_skipped_tokens(), 2..10),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for (i, sk) in skipped_lists.iter().enumerate() {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, sk.clone());
        }
        let nodes = state.get_error_nodes();
        for i in 0..skipped_lists.len() {
            prop_assert_eq!(nodes[i].skipped_tokens.len(), skipped_lists[i].len());
        }
    }

    // 39. Byte spans with large values near usize boundaries
    #[test]
    fn test_large_byte_offsets(base in (usize::MAX / 2)..(usize::MAX - 1000)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(base, base + 100, (0, 0), (0, 0), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.start_byte, base);
        prop_assert_eq!(node.end_byte, base + 100);
    }

    // 40. All seven strategy variants are distinct
    #[test]
    fn test_all_strategies_distinct(_ in 0u8..1) {
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

    // 41. Actual token u16::MIN preserved
    #[test]
    fn test_actual_token_min(_ in 0u8..1) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![0], Some(0), RecoveryStrategy::PanicMode, vec![0]);
        let node = &state.get_error_nodes()[0];
        prop_assert_eq!(node.actual, Some(0u16));
        prop_assert_eq!(&node.expected, &vec![0u16]);
        prop_assert_eq!(&node.skipped_tokens, &vec![0u16]);
    }

    // 42. Empty skipped tokens across batch of errors
    #[test]
    fn test_batch_empty_skipped(n in 1usize..30) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..n {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        let nodes = state.get_error_nodes();
        for i in 0..n {
            prop_assert!(nodes[i].skipped_tokens.is_empty());
        }
    }

    // 43. Config builder with phrase_recovery disabled
    #[test]
    fn test_builder_disable_phrase_recovery(_ in 0u8..1) {
        let config = ErrorRecoveryConfigBuilder::new()
            .enable_phrase_recovery(false)
            .build();
        let state = ErrorRecoveryState::new(config);
        prop_assert!(state.get_error_nodes().is_empty());
    }

    // 44. Same position different expected tokens
    #[test]
    fn test_same_pos_different_expected(
        exp_a in arb_expected(),
        exp_b in arb_expected(),
    ) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 5, (0, 0), (0, 5), exp_a.clone(), Some(1), RecoveryStrategy::PanicMode, vec![]);
        state.record_error(0, 5, (0, 0), (0, 5), exp_b.clone(), Some(1), RecoveryStrategy::PanicMode, vec![]);
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), 2);
        prop_assert_eq!(&nodes[0].expected, &exp_a);
        prop_assert_eq!(&nodes[1].expected, &exp_b);
    }

    // 45. add_recent_token does not affect error nodes
    #[test]
    fn test_add_recent_token_no_side_effect(tok in 0u16..1000) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.add_recent_token(tok);
        prop_assert!(state.get_error_nodes().is_empty());
    }

    // 46. push_scope / pop_scope do not affect error nodes
    #[test]
    fn test_scope_ops_no_error_side_effect(_ in 0u8..1) {
        let cfg = ErrorRecoveryConfig {
            scope_delimiters: vec![(40, 41)], // '(' / ')'
            ..Default::default()
        };
        let mut state = ErrorRecoveryState::new(cfg);
        state.push_scope(40);
        state.pop_scope(41);
        prop_assert!(state.get_error_nodes().is_empty());
    }

    // 47. reset_consecutive_errors does not clear error nodes
    #[test]
    fn test_reset_consecutive_does_not_clear(n in 1usize..10) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for i in 0..n {
            state.record_error(i, i + 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, vec![]);
        }
        state.reset_consecutive_errors();
        prop_assert_eq!(state.get_error_nodes().len(), n);
    }

    // 48. Duplicate token values in expected list preserved
    #[test]
    fn test_duplicate_expected_tokens(tok in 0u16..500, count in 2usize..20) {
        let expected: Vec<u16> = vec![tok; count];
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), expected.clone(), None, RecoveryStrategy::PanicMode, vec![]);
        prop_assert_eq!(&state.get_error_nodes()[0].expected, &expected);
    }

    // 49. Duplicate token values in skipped list preserved
    #[test]
    fn test_duplicate_skipped_tokens(tok in 0u16..500, count in 2usize..20) {
        let skipped: Vec<u16> = vec![tok; count];
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        state.record_error(0, 1, (0, 0), (0, 1), vec![], None, RecoveryStrategy::PanicMode, skipped.clone());
        prop_assert_eq!(&state.get_error_nodes()[0].skipped_tokens, &skipped);
    }

    // 50. End byte can equal start byte (zero-width) in batch
    #[test]
    fn test_zero_width_batch(positions in prop::collection::vec(0usize..50_000, 3..15)) {
        let mut state = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
        for &pos in &positions {
            state.record_error(pos, pos, (0, pos), (0, pos), vec![], None, RecoveryStrategy::TokenInsertion, vec![]);
        }
        let nodes = state.get_error_nodes();
        prop_assert_eq!(nodes.len(), positions.len());
        for (i, &pos) in positions.iter().enumerate() {
            prop_assert_eq!(nodes[i].start_byte, nodes[i].end_byte);
            prop_assert_eq!(nodes[i].start_byte, pos);
        }
    }
}
