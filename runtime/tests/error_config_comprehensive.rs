#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for ErrorRecoveryConfig and related configuration types.

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryStrategy,
};

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use ir::SymbolId;

// ── 1. Default config construction ─────────────────────────────────────────

#[test]
fn default_config_max_panic_skip() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn default_config_sync_tokens_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.sync_tokens.is_empty());
}

#[test]
fn default_config_insert_candidates_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.insert_candidates.is_empty());
}

#[test]
fn default_config_deletable_tokens_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.deletable_tokens.is_empty());
}

#[test]
fn default_config_max_token_deletions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_deletions, 3);
}

#[test]
fn default_config_max_token_insertions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_insertions, 2);
}

#[test]
fn default_config_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_consecutive_errors, 10);
}

#[test]
fn default_config_phrase_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_phrase_recovery);
}

#[test]
fn default_config_scope_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_scope_recovery);
}

#[test]
fn default_config_scope_delimiters_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.scope_delimiters.is_empty());
}

#[test]
fn default_config_indentation_recovery_disabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(!cfg.enable_indentation_recovery);
}

// ── 2. Clone and Debug ─────────────────────────────────────────────────────

#[test]
fn config_clone_preserves_all_fields() {
    let mut cfg = ErrorRecoveryConfig {
        max_panic_skip: 77,
        ..Default::default()
    };
    cfg.sync_tokens.push(SymbolId(5));
    cfg.insert_candidates.push(SymbolId(9));
    cfg.deletable_tokens.insert(42);
    cfg.max_token_deletions = 8;
    cfg.max_token_insertions = 6;
    cfg.max_consecutive_errors = 25;
    cfg.enable_phrase_recovery = false;
    cfg.enable_scope_recovery = false;
    cfg.scope_delimiters.push((10, 11));
    cfg.enable_indentation_recovery = true;

    let cloned = cfg.clone();
    assert_eq!(cloned.max_panic_skip, 77);
    assert_eq!(cloned.sync_tokens.len(), 1);
    assert_eq!(cloned.sync_tokens[0], SymbolId(5));
    assert_eq!(cloned.insert_candidates.len(), 1);
    assert!(cloned.deletable_tokens.contains(&42));
    assert_eq!(cloned.max_token_deletions, 8);
    assert_eq!(cloned.max_token_insertions, 6);
    assert_eq!(cloned.max_consecutive_errors, 25);
    assert!(!cloned.enable_phrase_recovery);
    assert!(!cloned.enable_scope_recovery);
    assert_eq!(cloned.scope_delimiters, vec![(10, 11)]);
    assert!(cloned.enable_indentation_recovery);
}

#[test]
fn config_debug_output_contains_field_names() {
    let cfg = ErrorRecoveryConfig::default();
    let dbg = format!("{cfg:?}");
    assert!(dbg.contains("max_panic_skip"));
    assert!(dbg.contains("max_consecutive_errors"));
    assert!(dbg.contains("enable_phrase_recovery"));
}

// ── 3. Custom config values via direct mutation ────────────────────────────

#[test]
fn custom_max_panic_skip() {
    let cfg = ErrorRecoveryConfig {
        max_panic_skip: 200,
        ..Default::default()
    };
    assert_eq!(cfg.max_panic_skip, 200);
}

#[test]
fn custom_max_consecutive_errors_zero() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..Default::default()
    };
    assert_eq!(cfg.max_consecutive_errors, 0);
}

#[test]
fn custom_scope_delimiters_multiple_pairs() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.scope_delimiters.push((40, 41)); // ( )
    cfg.scope_delimiters.push((91, 93)); // [ ]
    cfg.scope_delimiters.push((123, 125)); // { }
    assert_eq!(cfg.scope_delimiters.len(), 3);
}

// ── 4. Builder API ─────────────────────────────────────────────────────────

#[test]
fn builder_default_matches_config_default() {
    let from_builder = ErrorRecoveryConfigBuilder::new().build();
    let direct = ErrorRecoveryConfig::default();
    assert_eq!(from_builder.max_panic_skip, direct.max_panic_skip);
    assert_eq!(
        from_builder.max_consecutive_errors,
        direct.max_consecutive_errors
    );
    assert_eq!(
        from_builder.enable_phrase_recovery,
        direct.enable_phrase_recovery
    );
    assert_eq!(
        from_builder.enable_scope_recovery,
        direct.enable_scope_recovery
    );
    assert_eq!(
        from_builder.enable_indentation_recovery,
        direct.enable_indentation_recovery
    );
}

#[test]
fn builder_chained_construction() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .add_sync_token(1)
        .add_sync_token(2)
        .add_insertable_token(10)
        .add_deletable_token(20)
        .add_scope_delimiter(40, 41)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(true)
        .max_consecutive_errors(5)
        .build();

    assert_eq!(cfg.max_panic_skip, 100);
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert!(cfg.insert_candidates.iter().any(|t| t.0 == 10));
    assert!(cfg.deletable_tokens.contains(&20));
    assert_eq!(cfg.scope_delimiters, vec![(40, 41)]);
    assert!(!cfg.enable_phrase_recovery);
    assert!(!cfg.enable_scope_recovery);
    assert!(cfg.enable_indentation_recovery);
    assert_eq!(cfg.max_consecutive_errors, 5);
}

#[test]
fn builder_add_sync_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(99))
        .build();
    assert!(cfg.sync_tokens.contains(&SymbolId(99)));
}

#[test]
fn builder_add_insertable_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(55))
        .build();
    assert!(cfg.insert_candidates.contains(&SymbolId(55)));
}

#[test]
fn builder_set_max_recovery_attempts_aliases_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(42)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 42);
}

#[test]
fn builder_default_trait_impl() {
    // ErrorRecoveryConfigBuilder::default() should behave the same as ::new()
    let from_default = ErrorRecoveryConfigBuilder::default().build();
    let from_new = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(from_default.max_panic_skip, from_new.max_panic_skip);
}

// ── 5. can_delete_token / can_replace_token ────────────────────────────────

#[test]
fn can_delete_non_sync_token() {
    let cfg = ErrorRecoveryConfig::default();
    // No sync tokens → everything is deletable
    assert!(cfg.can_delete_token(SymbolId(99)));
}

#[test]
fn cannot_delete_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(7));
    assert!(!cfg.can_delete_token(SymbolId(7)));
}

#[test]
fn can_delete_explicitly_deletable_even_if_sync() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(7));
    cfg.deletable_tokens.insert(7);
    // deletable_tokens check comes first via OR
    assert!(cfg.can_delete_token(SymbolId(7)));
}

#[test]
fn can_replace_non_sync_token() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.can_replace_token(SymbolId(50)));
}

#[test]
fn cannot_replace_sync_token() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(50));
    assert!(!cfg.can_replace_token(SymbolId(50)));
}

// ── 6. Max error limits and should_give_up ─────────────────────────────────

#[test]
fn should_give_up_at_exact_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // Exactly at the limit → should give up
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn should_not_give_up_below_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 5,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..4 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up());
}

#[test]
fn should_give_up_above_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 2,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    for _ in 0..10 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn max_consecutive_errors_zero_gives_up_immediately() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..Default::default()
    };
    let state = ErrorRecoveryState::new(cfg);
    // Even 0 errors >= 0 limit → should give up
    assert!(state.should_give_up());
}

#[test]
fn reset_error_count_allows_continued_recovery() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 3,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());

    state.reset_error_count();
    assert!(!state.should_give_up());
}

// ── 7. Config effect on recovery strategy ──────────────────────────────────

#[test]
fn panic_mode_when_over_error_limit() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 1,
        ..Default::default()
    };
    let mut state = ErrorRecoveryState::new(cfg);
    // First call bumps count to 1, under limit (>1 required)
    let _first = state.determine_recovery_strategy(&[99], Some(50), (0, 0), 0);
    // Second call bumps to 2, which exceeds limit of 1
    let strategy = state.determine_recovery_strategy(&[99], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

#[test]
fn token_insertion_when_candidate_matches_expected() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    let strategy = state.determine_recovery_strategy(&[10, 20], None, (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
}

#[test]
fn phrase_level_when_enabled_and_no_better_option() {
    let mut cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: true,
        enable_scope_recovery: false,
        ..Default::default()
    };
    // Make token 50 a sync token so deletion is skipped
    cfg.sync_tokens.push(SymbolId(50));
    let mut state = ErrorRecoveryState::new(cfg);
    // actual 50 is sync → not clearly wrong → deletion skipped
    // expected has 2 items → substitution skipped → phrase level
    let strategy = state.determine_recovery_strategy(&[99, 98], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PhraseLevel);
}

#[test]
fn scope_recovery_on_unmatched_close_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_sync_token(41) // make 41 a sync token so deletion is skipped
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    // Close delimiter 41 is sync → not clearly wrong → deletion skipped
    // expected has 2 items → substitution skipped → scope mismatch detected
    let strategy = state.determine_recovery_strategy(&[99, 98], Some(41), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn disabled_phrase_recovery_falls_to_panic_mode() {
    let mut cfg = ErrorRecoveryConfig {
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        ..Default::default()
    };
    // Make token 50 a sync token so deletion and substitution are skipped
    cfg.sync_tokens.push(SymbolId(50));
    let mut state = ErrorRecoveryState::new(cfg);
    // actual 50 is sync → not clearly wrong → deletion skipped
    // expected has 2 items → substitution skipped → no phrase/scope → panic
    let strategy = state.determine_recovery_strategy(&[99, 98], Some(50), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::PanicMode);
}

// ── 8. RecoveryStrategy enum properties ────────────────────────────────────

#[test]
fn recovery_strategy_clone_and_copy() {
    let s = RecoveryStrategy::TokenDeletion;
    let cloned = s;
    let copied: RecoveryStrategy = cloned;
    assert_eq!(s, copied);
}

#[test]
fn recovery_strategy_debug_output() {
    let dbg = format!("{:?}", RecoveryStrategy::IndentationRecovery);
    assert_eq!(dbg, "IndentationRecovery");
}

#[test]
fn recovery_strategy_all_variants_distinct() {
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
