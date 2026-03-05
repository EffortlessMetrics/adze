//! Comprehensive tests for ErrorRecoveryConfigBuilder (v9).
//!
//! Covers builder defaults, field setters, chaining, strategy/delimiter
//! accumulation, Config accessors, Debug/Clone, and edge cases.

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryStrategy,
};
use adze_ir::SymbolId;

// =====================================================================
// 1–4. Builder defaults & max_consecutive_errors
// =====================================================================

#[test]
fn t01_builder_default_produces_valid_config() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(config.max_consecutive_errors, 10);
}

#[test]
fn t02_max_consecutive_errors_10() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(10)
        .build();
    assert_eq!(config.max_consecutive_errors, 10);
}

#[test]
fn t03_max_consecutive_errors_zero() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(0)
        .build();
    assert_eq!(config.max_consecutive_errors, 0);
}

#[test]
fn t04_max_consecutive_errors_usize_max() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(usize::MAX)
        .build();
    assert_eq!(config.max_consecutive_errors, usize::MAX);
}

// =====================================================================
// 5. set_max_recovery_attempts (alias for max_consecutive_errors)
// =====================================================================

#[test]
fn t05_set_max_recovery_attempts_stores_value() {
    let config = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(5)
        .build();
    assert_eq!(config.max_consecutive_errors, 5);
}

#[test]
fn t06_set_max_recovery_attempts_zero() {
    let config = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(0)
        .build();
    assert_eq!(config.max_consecutive_errors, 0);
}

#[test]
fn t07_set_max_recovery_attempts_usize_max() {
    let config = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(usize::MAX)
        .build();
    assert_eq!(config.max_consecutive_errors, usize::MAX);
}

// =====================================================================
// 6–7. Scope delimiters
// =====================================================================

#[test]
fn t08_add_scope_delimiter_single() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    assert_eq!(config.scope_delimiters, vec![(40, 41)]);
}

#[test]
fn t09_add_multiple_scope_delimiters() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .add_scope_delimiter(123, 125)
        .build();
    assert_eq!(config.scope_delimiters.len(), 3);
    assert_eq!(config.scope_delimiters[0], (40, 41));
    assert_eq!(config.scope_delimiters[1], (91, 93));
    assert_eq!(config.scope_delimiters[2], (123, 125));
}

#[test]
fn t10_no_delimiters_yields_empty() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert!(config.scope_delimiters.is_empty());
}

#[test]
fn t11_duplicate_delimiters_are_stored() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .add_scope_delimiter(1, 2)
        .build();
    assert_eq!(config.scope_delimiters.len(), 2);
}

// =====================================================================
// 8–10. Recovery strategies via enable_* flags & all variants
// =====================================================================

#[test]
fn t12_enable_phrase_recovery_default_true() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert!(config.enable_phrase_recovery);
}

#[test]
fn t13_enable_phrase_recovery_false() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    assert!(!config.enable_phrase_recovery);
}

#[test]
fn t14_enable_scope_recovery_default_true() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert!(config.enable_scope_recovery);
}

#[test]
fn t15_enable_scope_recovery_false() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!config.enable_scope_recovery);
}

#[test]
fn t16_enable_indentation_recovery_default_false() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert!(!config.enable_indentation_recovery);
}

#[test]
fn t17_enable_indentation_recovery_true() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(config.enable_indentation_recovery);
}

#[test]
fn t18_recovery_strategy_panic_mode_variant() {
    let s = RecoveryStrategy::PanicMode;
    assert_eq!(s, RecoveryStrategy::PanicMode);
}

#[test]
fn t19_recovery_strategy_token_insertion_variant() {
    let s = RecoveryStrategy::TokenInsertion;
    assert_eq!(s, RecoveryStrategy::TokenInsertion);
}

#[test]
fn t20_recovery_strategy_token_deletion_variant() {
    let s = RecoveryStrategy::TokenDeletion;
    assert_eq!(s, RecoveryStrategy::TokenDeletion);
}

#[test]
fn t21_recovery_strategy_token_substitution_variant() {
    let s = RecoveryStrategy::TokenSubstitution;
    assert_eq!(s, RecoveryStrategy::TokenSubstitution);
}

#[test]
fn t22_recovery_strategy_phrase_level_variant() {
    let s = RecoveryStrategy::PhraseLevel;
    assert_eq!(s, RecoveryStrategy::PhraseLevel);
}

#[test]
fn t23_recovery_strategy_scope_recovery_variant() {
    let s = RecoveryStrategy::ScopeRecovery;
    assert_eq!(s, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn t24_recovery_strategy_indentation_recovery_variant() {
    let s = RecoveryStrategy::IndentationRecovery;
    assert_eq!(s, RecoveryStrategy::IndentationRecovery);
}

#[test]
fn t25_all_strategy_variants_are_distinct() {
    let variants = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    for i in 0..variants.len() {
        for j in (i + 1)..variants.len() {
            assert_ne!(variants[i], variants[j]);
        }
    }
}

// =====================================================================
// 11. Builder chaining preserves all settings
// =====================================================================

#[test]
fn t26_full_builder_chain() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .max_consecutive_errors(20)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(3)
        .add_deletable_token(4)
        .add_scope_delimiter(40, 41)
        .enable_indentation_recovery(true)
        .enable_scope_recovery(false)
        .enable_phrase_recovery(false)
        .build();

    assert_eq!(config.max_panic_skip, 100);
    assert_eq!(config.max_consecutive_errors, 20);
    assert!(config.sync_tokens.contains(&SymbolId(1)));
    assert!(config.sync_tokens.contains(&SymbolId(2)));
    assert!(config.insert_candidates.contains(&SymbolId(3)));
    assert!(config.deletable_tokens.contains(&4));
    assert_eq!(config.scope_delimiters, vec![(40, 41)]);
    assert!(config.enable_indentation_recovery);
    assert!(!config.enable_scope_recovery);
    assert!(!config.enable_phrase_recovery);
}

// =====================================================================
// 12–13. Config Debug and Clone
// =====================================================================

#[test]
fn t27_config_debug_format() {
    let config = ErrorRecoveryConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("ErrorRecoveryConfig"));
}

#[test]
fn t28_config_debug_contains_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(77)
        .build();
    let debug = format!("{config:?}");
    assert!(debug.contains("77"));
}

#[test]
fn t29_config_clone_is_equal() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(42)
        .max_consecutive_errors(7)
        .add_scope_delimiter(10, 20)
        .build();
    let cloned = config.clone();
    assert_eq!(cloned.max_panic_skip, 42);
    assert_eq!(cloned.max_consecutive_errors, 7);
    assert_eq!(cloned.scope_delimiters, vec![(10, 20)]);
}

#[test]
fn t30_config_clone_is_independent() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .build();
    let mut cloned = config.clone();
    cloned.max_panic_skip = 999;
    assert_eq!(config.max_panic_skip, 10);
    assert_eq!(cloned.max_panic_skip, 999);
}

// =====================================================================
// 14–15. Build produces immutable config; multiple builds
// =====================================================================

#[test]
fn t31_build_consumes_builder() {
    let builder = ErrorRecoveryConfigBuilder::new().max_panic_skip(5);
    let config = builder.build();
    assert_eq!(config.max_panic_skip, 5);
}

#[test]
fn t32_two_independent_builders() {
    let c1 = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .build();
    let c2 = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    assert_eq!(c1.max_consecutive_errors, 1);
    assert_eq!(c2.max_consecutive_errors, 2);
}

// =====================================================================
// 16–17. Empty strategies / delimiters
// =====================================================================

#[test]
fn t33_no_sync_tokens_yields_empty() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert!(config.sync_tokens.is_empty());
}

#[test]
fn t34_no_insert_candidates_yields_empty() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert!(config.insert_candidates.is_empty());
}

#[test]
fn t35_no_deletable_tokens_yields_empty() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert!(config.deletable_tokens.is_empty());
}

// =====================================================================
// 18. Config with all options set
// =====================================================================

#[test]
fn t36_config_all_options() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .max_consecutive_errors(50)
        .add_sync_token(10)
        .add_sync_token(20)
        .add_sync_token(30)
        .add_insertable_token(40)
        .add_insertable_token(50)
        .add_deletable_token(60)
        .add_deletable_token(70)
        .add_scope_delimiter(100, 101)
        .add_scope_delimiter(200, 201)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(true)
        .enable_indentation_recovery(true)
        .build();

    assert_eq!(config.max_panic_skip, 200);
    assert_eq!(config.max_consecutive_errors, 50);
    assert_eq!(config.sync_tokens.len(), 3);
    assert_eq!(config.insert_candidates.len(), 2);
    assert_eq!(config.deletable_tokens.len(), 2);
    assert_eq!(config.scope_delimiters.len(), 2);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
    assert!(config.enable_indentation_recovery);
}

// =====================================================================
// 19–20. Default field values
// =====================================================================

#[test]
fn t37_default_max_panic_skip() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_panic_skip, 50);
}

#[test]
fn t38_default_max_consecutive_errors() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_consecutive_errors, 10);
}

#[test]
fn t39_default_max_token_deletions() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_token_deletions, 3);
}

#[test]
fn t40_default_max_token_insertions() {
    let config = ErrorRecoveryConfig::default();
    assert_eq!(config.max_token_insertions, 2);
}

// =====================================================================
// Builder method: max_panic_skip
// =====================================================================

#[test]
fn t41_max_panic_skip_set() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(123)
        .build();
    assert_eq!(config.max_panic_skip, 123);
}

#[test]
fn t42_max_panic_skip_zero() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(0)
        .build();
    assert_eq!(config.max_panic_skip, 0);
}

#[test]
fn t43_max_panic_skip_usize_max() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(usize::MAX)
        .build();
    assert_eq!(config.max_panic_skip, usize::MAX);
}

// =====================================================================
// Builder method: add_sync_token / add_sync_token_sym
// =====================================================================

#[test]
fn t44_add_sync_token_single() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(42)
        .build();
    assert!(config.sync_tokens.contains(&SymbolId(42)));
    assert_eq!(config.sync_tokens.len(), 1);
}

#[test]
fn t45_add_sync_token_multiple() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_sync_token(2)
        .add_sync_token(3)
        .build();
    assert_eq!(config.sync_tokens.len(), 3);
}

#[test]
fn t46_add_sync_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(99))
        .build();
    assert!(config.sync_tokens.contains(&SymbolId(99)));
}

#[test]
fn t47_add_sync_token_sym_multiple() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(10))
        .add_sync_token_sym(SymbolId(20))
        .build();
    assert_eq!(config.sync_tokens.len(), 2);
    assert!(config.sync_tokens.contains(&SymbolId(10)));
    assert!(config.sync_tokens.contains(&SymbolId(20)));
}

// =====================================================================
// Builder method: add_insertable_token / add_insertable_token_sym
// =====================================================================

#[test]
fn t48_add_insertable_token_single() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(7)
        .build();
    assert!(config.insert_candidates.contains(&SymbolId(7)));
}

#[test]
fn t49_add_insertable_token_multiple() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(7)
        .add_insertable_token(8)
        .add_insertable_token(9)
        .build();
    assert_eq!(config.insert_candidates.len(), 3);
}

#[test]
fn t50_add_insertable_token_sym() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(55))
        .build();
    assert!(config.insert_candidates.contains(&SymbolId(55)));
}

// =====================================================================
// Builder method: add_deletable_token
// =====================================================================

#[test]
fn t51_add_deletable_token_single() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(15)
        .build();
    assert!(config.deletable_tokens.contains(&15));
}

#[test]
fn t52_add_deletable_token_multiple() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(15)
        .add_deletable_token(16)
        .add_deletable_token(17)
        .build();
    assert_eq!(config.deletable_tokens.len(), 3);
}

#[test]
fn t53_add_deletable_token_dedup() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(15)
        .add_deletable_token(15)
        .build();
    // HashSet deduplicates
    assert_eq!(config.deletable_tokens.len(), 1);
}

// =====================================================================
// Config methods: can_delete_token / can_replace_token
// =====================================================================

#[test]
fn t54_can_delete_token_in_deletable_set() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .build();
    assert!(config.can_delete_token(SymbolId(5)));
}

#[test]
fn t55_can_delete_token_not_in_sync_tokens() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    // No sync tokens → !sync_tokens.contains(x) is true for any x
    assert!(config.can_delete_token(SymbolId(999)));
}

#[test]
fn t56_can_delete_token_that_is_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(5)
        .build();
    // Token 5 is in sync_tokens and not in deletable_tokens
    assert!(!config.can_delete_token(SymbolId(5)));
}

#[test]
fn t57_can_delete_token_in_both_deletable_and_sync() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(5)
        .add_deletable_token(5)
        .build();
    // In deletable_tokens → true (short-circuit OR)
    assert!(config.can_delete_token(SymbolId(5)));
}

#[test]
fn t58_can_replace_token_not_sync() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert!(config.can_replace_token(SymbolId(99)));
}

#[test]
fn t59_cannot_replace_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(42)
        .build();
    assert!(!config.can_replace_token(SymbolId(42)));
}

// =====================================================================
// RecoveryStrategy derives: Debug, Clone, Copy, PartialEq, Eq
// =====================================================================

#[test]
fn t60_recovery_strategy_debug() {
    let debug = format!("{:?}", RecoveryStrategy::PanicMode);
    assert_eq!(debug, "PanicMode");
}

#[test]
fn t61_recovery_strategy_clone() {
    let s = RecoveryStrategy::TokenInsertion;
    let s2 = s;
    assert_eq!(s, s2);
}

#[test]
fn t62_recovery_strategy_eq_reflexive() {
    let s = RecoveryStrategy::ScopeRecovery;
    assert_eq!(s, s);
}

#[test]
fn t63_recovery_strategy_ne() {
    assert_ne!(RecoveryStrategy::PanicMode, RecoveryStrategy::TokenDeletion);
}

// =====================================================================
// ErrorRecoveryState creation from builder config
// =====================================================================

#[test]
fn t64_state_from_builder_config() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let state = ErrorRecoveryState::new(config);
    assert!(!state.should_give_up());
}

#[test]
fn t65_state_should_give_up_at_max() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn t66_state_should_not_give_up_below_max() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(5)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn t67_state_reset_error_count() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

// =====================================================================
// ErrorRecoveryState scope operations with builder-configured delimiters
// =====================================================================

#[test]
fn t68_state_push_scope_with_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    let popped = state.pop_scope_test();
    assert_eq!(popped, Some(40));
}

#[test]
fn t69_state_push_non_delimiter_ignored() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(99); // Not a delimiter open
    let popped = state.pop_scope_test();
    assert_eq!(popped, None);
}

#[test]
fn t70_state_pop_scope_matching() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    assert!(state.pop_scope(41));
}

#[test]
fn t71_state_pop_scope_not_matching() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(91, 93)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(40);
    assert!(!state.pop_scope(93)); // Mismatched close
}

// =====================================================================
// ErrorRecoveryState recent tokens with builder config
// =====================================================================

#[test]
fn t72_state_add_recent_token() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.add_recent_token(42);
    // No panic means success; internal state is private
}

#[test]
fn t73_state_update_recent_tokens_sym() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.update_recent_tokens(SymbolId(7));
}

// =====================================================================
// ErrorRecoveryState error recording
// =====================================================================

#[test]
fn t74_state_record_error_and_retrieve() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1, 2],
        Some(3),
        RecoveryStrategy::PanicMode,
        vec![3],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].start_byte, 0);
    assert_eq!(nodes[0].end_byte, 5);
    assert_eq!(nodes[0].expected, vec![1, 2]);
    assert_eq!(nodes[0].actual, Some(3));
}

#[test]
fn t75_state_no_errors_initially() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let state = ErrorRecoveryState::new(config);
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn t76_state_clear_errors() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0, 1, (0, 0), (0, 1), vec![], None,
        RecoveryStrategy::TokenDeletion, vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 1);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

// =====================================================================
// Builder Default impl
// =====================================================================

#[test]
fn t77_builder_default_trait() {
    let config = ErrorRecoveryConfigBuilder::default().build();
    assert_eq!(config.max_panic_skip, 50);
    assert_eq!(config.max_consecutive_errors, 10);
}

// =====================================================================
// Static helper methods
// =====================================================================

#[test]
fn t78_is_scope_delimiter_open() {
    let delimiters = vec![(40, 41), (91, 93)];
    assert!(ErrorRecoveryState::is_scope_delimiter(40, &delimiters));
}

#[test]
fn t79_is_scope_delimiter_close() {
    let delimiters = vec![(40, 41), (91, 93)];
    assert!(ErrorRecoveryState::is_scope_delimiter(41, &delimiters));
}

#[test]
fn t80_is_scope_delimiter_not_found() {
    let delimiters = vec![(40, 41)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delimiters));
}

#[test]
fn t81_is_matching_delimiter_true() {
    let delimiters = vec![(40, 41)];
    assert!(ErrorRecoveryState::is_matching_delimiter(40, 41, &delimiters));
}

#[test]
fn t82_is_matching_delimiter_false() {
    let delimiters = vec![(40, 41)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(40, 93, &delimiters));
}

#[test]
fn t83_is_matching_delimiter_empty() {
    let delimiters: Vec<(u16, u16)> = vec![];
    assert!(!ErrorRecoveryState::is_matching_delimiter(40, 41, &delimiters));
}

// =====================================================================
// Overwrite semantics: last setter wins
// =====================================================================

#[test]
fn t84_max_panic_skip_last_wins() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .max_panic_skip(20)
        .max_panic_skip(30)
        .build();
    assert_eq!(config.max_panic_skip, 30);
}

#[test]
fn t85_max_consecutive_errors_last_wins() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .max_consecutive_errors(2)
        .build();
    assert_eq!(config.max_consecutive_errors, 2);
}

#[test]
fn t86_enable_flags_last_wins() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_phrase_recovery(true)
        .build();
    assert!(config.enable_phrase_recovery);
}

// =====================================================================
// Edge cases: mixing set_max_recovery_attempts and max_consecutive_errors
// =====================================================================

#[test]
fn t87_alias_overrides_max_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(100)
        .set_max_recovery_attempts(3)
        .build();
    assert_eq!(config.max_consecutive_errors, 3);
}

#[test]
fn t88_max_consecutive_errors_overrides_alias() {
    let config = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(3)
        .max_consecutive_errors(100)
        .build();
    assert_eq!(config.max_consecutive_errors, 100);
}

// =====================================================================
// Config used with ErrorRecoveryState::determine_recovery_strategy
// =====================================================================

#[test]
fn t89_determine_strategy_with_insertable() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&[10], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn t90_determine_strategy_panic_mode_on_exceed() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(1)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // First call increments to 1
    let _ = state.determine_recovery_strategy(&[], None, (0, 0), 0);
    // Second call increments to 2, which exceeds max of 1
    let strategy = state.determine_recovery_strategy(&[], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

// =====================================================================
// Config Debug with various settings
// =====================================================================

#[test]
fn t91_debug_shows_scope_delimiters() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .build();
    let debug = format!("{config:?}");
    assert!(debug.contains("scope_delimiters"));
}

#[test]
fn t92_debug_shows_enable_flags() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    let debug = format!("{config:?}");
    assert!(debug.contains("enable_indentation_recovery"));
}

// =====================================================================
// Builder accumulates tokens across multiple calls
// =====================================================================

#[test]
fn t93_sync_tokens_accumulate() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_sync_token(2)
        .add_sync_token_sym(SymbolId(3))
        .build();
    assert_eq!(config.sync_tokens.len(), 3);
    assert!(config.sync_tokens.contains(&SymbolId(1)));
    assert!(config.sync_tokens.contains(&SymbolId(2)));
    assert!(config.sync_tokens.contains(&SymbolId(3)));
}

#[test]
fn t94_insert_candidates_accumulate() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .add_insertable_token_sym(SymbolId(20))
        .build();
    assert_eq!(config.insert_candidates.len(), 2);
}

#[test]
fn t95_deletable_tokens_accumulate_unique() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(1)
        .add_deletable_token(2)
        .add_deletable_token(3)
        .build();
    assert_eq!(config.deletable_tokens.len(), 3);
}

// =====================================================================
// RecoveryStrategy Debug for all variants
// =====================================================================

#[test]
fn t96_recovery_strategy_debug_all_variants() {
    assert_eq!(format!("{:?}", RecoveryStrategy::PanicMode), "PanicMode");
    assert_eq!(format!("{:?}", RecoveryStrategy::TokenInsertion), "TokenInsertion");
    assert_eq!(format!("{:?}", RecoveryStrategy::TokenDeletion), "TokenDeletion");
    assert_eq!(format!("{:?}", RecoveryStrategy::TokenSubstitution), "TokenSubstitution");
    assert_eq!(format!("{:?}", RecoveryStrategy::PhraseLevel), "PhraseLevel");
    assert_eq!(format!("{:?}", RecoveryStrategy::ScopeRecovery), "ScopeRecovery");
    assert_eq!(
        format!("{:?}", RecoveryStrategy::IndentationRecovery),
        "IndentationRecovery"
    );
}

// =====================================================================
// Boundary: max_consecutive_errors = 0 means immediate give-up
// =====================================================================

#[test]
fn t97_zero_max_errors_gives_up_immediately() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(0)
        .build();
    let state = ErrorRecoveryState::new(config);
    assert!(state.should_give_up());
}

// =====================================================================
// Scope delimiter ordering
// =====================================================================

#[test]
fn t98_scope_delimiters_preserve_order() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .add_scope_delimiter(3, 4)
        .add_scope_delimiter(5, 6)
        .build();
    assert_eq!(config.scope_delimiters[0], (1, 2));
    assert_eq!(config.scope_delimiters[1], (3, 4));
    assert_eq!(config.scope_delimiters[2], (5, 6));
}

// =====================================================================
// Config Default trait
// =====================================================================

#[test]
fn t99_config_default_trait_matches_builder_default() {
    let from_default = ErrorRecoveryConfig::default();
    let from_builder = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(from_default.max_panic_skip, from_builder.max_panic_skip);
    assert_eq!(
        from_default.max_consecutive_errors,
        from_builder.max_consecutive_errors
    );
    assert_eq!(
        from_default.max_token_deletions,
        from_builder.max_token_deletions
    );
    assert_eq!(
        from_default.max_token_insertions,
        from_builder.max_token_insertions
    );
    assert_eq!(
        from_default.enable_phrase_recovery,
        from_builder.enable_phrase_recovery
    );
    assert_eq!(
        from_default.enable_scope_recovery,
        from_builder.enable_scope_recovery
    );
    assert_eq!(
        from_default.enable_indentation_recovery,
        from_builder.enable_indentation_recovery
    );
}

// =====================================================================
// Multiple error nodes accumulate
// =====================================================================

#[test]
fn t100_multiple_error_nodes_accumulate() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..5 {
        state.record_error(
            i, i + 1, (0, i), (0, i + 1), vec![],
            None, RecoveryStrategy::PanicMode, vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}
