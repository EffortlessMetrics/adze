//! Comprehensive tests for error handling v4: ErrorRecoveryConfig, RecoveryStrategy,
//! RecoveryAction, builder, edge cases, traits, and cross-type interactions.

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryAction,
    RecoveryStrategy,
};
use adze_ir::SymbolId;

// ═══════════════════════════════════════════════════════════════════════════
// 1. ErrorRecoveryConfig construction and default values (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn config_default_max_panic_skip() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn config_default_sync_tokens_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.sync_tokens.is_empty());
}

#[test]
fn config_default_insert_candidates_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.insert_candidates.is_empty());
}

#[test]
fn config_default_deletable_tokens_empty() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.deletable_tokens.is_empty());
}

#[test]
fn config_default_max_token_deletions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_deletions, 3);
}

#[test]
fn config_default_max_token_insertions() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_token_insertions, 2);
}

#[test]
fn config_default_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig::default();
    assert_eq!(cfg.max_consecutive_errors, 10);
}

#[test]
fn config_default_phrase_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_phrase_recovery);
}

#[test]
fn config_default_scope_recovery_enabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.enable_scope_recovery);
}

#[test]
fn config_default_indentation_recovery_disabled() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(!cfg.enable_indentation_recovery);
    assert!(cfg.scope_delimiters.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. RecoveryStrategy variants and properties (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn strategy_panic_mode_identity() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
}

#[test]
fn strategy_token_insertion_identity() {
    assert_eq!(
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenInsertion
    );
}

#[test]
fn strategy_token_deletion_identity() {
    assert_eq!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenDeletion
    );
}

#[test]
fn strategy_token_substitution_identity() {
    assert_eq!(
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::TokenSubstitution
    );
}

#[test]
fn strategy_phrase_level_identity() {
    assert_eq!(RecoveryStrategy::PhraseLevel, RecoveryStrategy::PhraseLevel);
}

#[test]
fn strategy_scope_recovery_identity() {
    assert_eq!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::ScopeRecovery
    );
}

#[test]
fn strategy_indentation_recovery_identity() {
    assert_eq!(
        RecoveryStrategy::IndentationRecovery,
        RecoveryStrategy::IndentationRecovery
    );
}

#[test]
fn strategy_different_variants_not_equal() {
    assert_ne!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion
    );
    assert_ne!(
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::PhraseLevel
    );
    assert_ne!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery
    );
}

#[test]
fn strategy_copy_semantics() {
    let s = RecoveryStrategy::PanicMode;
    let s2 = s; // Copy
    assert_eq!(s, s2);
}

#[test]
fn strategy_all_seven_variants_distinct() {
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
            assert_ne!(
                variants[i], variants[j],
                "variants {i} and {j} should differ"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. RecoveryAction creation and comparison (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_insert_token_creation() {
    let a = RecoveryAction::InsertToken(SymbolId(42));
    assert!(matches!(a, RecoveryAction::InsertToken(SymbolId(42))));
}

#[test]
fn action_delete_token_creation() {
    let a = RecoveryAction::DeleteToken;
    assert!(matches!(a, RecoveryAction::DeleteToken));
}

#[test]
fn action_replace_token_creation() {
    let a = RecoveryAction::ReplaceToken(SymbolId(7));
    assert!(matches!(a, RecoveryAction::ReplaceToken(SymbolId(7))));
}

#[test]
fn action_create_error_node_creation() {
    let a = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2)]);
    if let RecoveryAction::CreateErrorNode(ids) = &a {
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], SymbolId(1));
        assert_eq!(ids[1], SymbolId(2));
    } else {
        panic!("expected CreateErrorNode");
    }
}

#[test]
fn action_insert_token_equality() {
    let a = RecoveryAction::InsertToken(SymbolId(10));
    let b = RecoveryAction::InsertToken(SymbolId(10));
    assert_eq!(a, b);
}

#[test]
fn action_insert_token_different_ids_not_equal() {
    let a = RecoveryAction::InsertToken(SymbolId(10));
    let b = RecoveryAction::InsertToken(SymbolId(20));
    assert_ne!(a, b);
}

#[test]
fn action_different_variants_not_equal() {
    let insert = RecoveryAction::InsertToken(SymbolId(1));
    let delete = RecoveryAction::DeleteToken;
    assert_ne!(insert, delete);
}

#[test]
fn action_delete_token_equality() {
    assert_eq!(RecoveryAction::DeleteToken, RecoveryAction::DeleteToken);
}

#[test]
fn action_replace_token_equality() {
    let a = RecoveryAction::ReplaceToken(SymbolId(5));
    let b = RecoveryAction::ReplaceToken(SymbolId(5));
    assert_eq!(a, b);
}

#[test]
fn action_create_error_node_equality() {
    let a = RecoveryAction::CreateErrorNode(vec![SymbolId(3)]);
    let b = RecoveryAction::CreateErrorNode(vec![SymbolId(3)]);
    assert_eq!(a, b);
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Config with different strategy combinations (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

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
fn builder_set_max_panic_skip() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(cfg.max_panic_skip, 200);
}

#[test]
fn builder_add_sync_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(10)
        .add_sync_token(20)
        .build();
    assert_eq!(cfg.sync_tokens.len(), 2);
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 10));
    assert!(cfg.sync_tokens.iter().any(|t| t.0 == 20));
}

#[test]
fn builder_add_sync_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_sync_token_sym(SymbolId(99))
        .build();
    assert_eq!(cfg.sync_tokens.len(), 1);
    assert_eq!(cfg.sync_tokens[0], SymbolId(99));
}

#[test]
fn builder_add_insertable_tokens() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(5)
        .add_insertable_token(6)
        .build();
    assert_eq!(cfg.insert_candidates.len(), 2);
}

#[test]
fn builder_add_insertable_token_sym() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token_sym(SymbolId(77))
        .build();
    assert_eq!(cfg.insert_candidates[0], SymbolId(77));
}

#[test]
fn builder_add_deletable_token() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(15)
        .add_deletable_token(25)
        .build();
    assert!(cfg.deletable_tokens.contains(&15));
    assert!(cfg.deletable_tokens.contains(&25));
    assert_eq!(cfg.deletable_tokens.len(), 2);
}

#[test]
fn builder_add_scope_delimiter() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41)
        .add_scope_delimiter(60, 61)
        .build();
    assert_eq!(cfg.scope_delimiters, vec![(40, 41), (60, 61)]);
}

#[test]
fn builder_enable_indentation_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(cfg.enable_indentation_recovery);
}

#[test]
fn builder_disable_phrase_and_scope_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    assert!(!cfg.enable_phrase_recovery);
    assert!(!cfg.enable_scope_recovery);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Edge cases: empty config, max values, zero values (10 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn config_zero_max_panic_skip() {
    let cfg = ErrorRecoveryConfig {
        max_panic_skip: 0,
        ..Default::default()
    };
    assert_eq!(cfg.max_panic_skip, 0);
}

#[test]
fn config_zero_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfig {
        max_consecutive_errors: 0,
        ..Default::default()
    };
    assert_eq!(cfg.max_consecutive_errors, 0);
}

#[test]
fn config_zero_max_token_deletions() {
    let cfg = ErrorRecoveryConfig {
        max_token_deletions: 0,
        ..Default::default()
    };
    assert_eq!(cfg.max_token_deletions, 0);
}

#[test]
fn config_zero_max_token_insertions() {
    let cfg = ErrorRecoveryConfig {
        max_token_insertions: 0,
        ..Default::default()
    };
    assert_eq!(cfg.max_token_insertions, 0);
}

#[test]
fn config_large_max_panic_skip() {
    let cfg = ErrorRecoveryConfig {
        max_panic_skip: usize::MAX,
        ..Default::default()
    };
    assert_eq!(cfg.max_panic_skip, usize::MAX);
}

#[test]
fn config_large_max_consecutive_errors() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(usize::MAX)
        .build();
    assert_eq!(cfg.max_consecutive_errors, usize::MAX);
}

#[test]
fn config_empty_scope_delimiters_can_delete_any() {
    let cfg = ErrorRecoveryConfig::default();
    // No sync tokens → any token can be deleted
    assert!(cfg.can_delete_token(SymbolId(0)));
    assert!(cfg.can_delete_token(SymbolId(999)));
}

#[test]
fn config_empty_sync_tokens_can_replace_any() {
    let cfg = ErrorRecoveryConfig::default();
    assert!(cfg.can_replace_token(SymbolId(0)));
    assert!(cfg.can_replace_token(SymbolId(u16::MAX)));
}

#[test]
fn config_deletable_tokens_explicit() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(50));
    cfg.deletable_tokens.insert(50);
    // Explicitly deletable even if it's a sync token
    assert!(cfg.can_delete_token(SymbolId(50)));
}

#[test]
fn config_many_scope_delimiters() {
    let mut builder = ErrorRecoveryConfigBuilder::new();
    for i in 0..100u16 {
        builder = builder.add_scope_delimiter(i * 2, i * 2 + 1);
    }
    let cfg = builder.build();
    assert_eq!(cfg.scope_delimiters.len(), 100);
    assert_eq!(cfg.scope_delimiters[0], (0, 1));
    assert_eq!(cfg.scope_delimiters[99], (198, 199));
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Debug/Clone trait verification (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn strategy_debug_format() {
    let s = RecoveryStrategy::TokenInsertion;
    let dbg = format!("{:?}", s);
    assert_eq!(dbg, "TokenInsertion");
}

#[test]
fn action_debug_format_insert() {
    let a = RecoveryAction::InsertToken(SymbolId(42));
    let dbg = format!("{:?}", a);
    assert!(dbg.contains("InsertToken"));
    assert!(dbg.contains("42"));
}

#[test]
fn action_debug_format_delete() {
    let a = RecoveryAction::DeleteToken;
    let dbg = format!("{:?}", a);
    assert_eq!(dbg, "DeleteToken");
}

#[test]
fn action_clone_preserves_value() {
    let a = RecoveryAction::CreateErrorNode(vec![SymbolId(1), SymbolId(2), SymbolId(3)]);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn config_clone_preserves_all_fields() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(77)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .add_scope_delimiter(4, 5)
        .max_consecutive_errors(99)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .enable_indentation_recovery(true)
        .build();
    let cloned = cfg.clone();
    assert_eq!(cloned.max_panic_skip, 77);
    assert_eq!(cloned.sync_tokens.len(), 1);
    assert_eq!(cloned.insert_candidates.len(), 1);
    assert!(cloned.deletable_tokens.contains(&3));
    assert_eq!(cloned.scope_delimiters, vec![(4, 5)]);
    assert_eq!(cloned.max_consecutive_errors, 99);
    assert!(!cloned.enable_phrase_recovery);
    assert!(!cloned.enable_scope_recovery);
    assert!(cloned.enable_indentation_recovery);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Cross-type interactions (5 tests)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn state_with_custom_config_preserves_settings() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(10)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_scope_delimiter(3, 4)
        .max_consecutive_errors(5)
        .build();
    let state = ErrorRecoveryState::new(cfg);
    // State initializes cleanly
    assert!(!state.should_give_up());
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn state_scope_push_pop_with_delimiters() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.push_scope(10);
    assert!(state.pop_scope(11));
    // Popping again with no stack should fail
    assert!(!state.pop_scope(11));
}

#[test]
fn state_error_count_reaches_threshold() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    assert!(!state.should_give_up());
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn state_reset_error_count_allows_recovery() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(cfg);
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn static_delimiter_helpers() {
    let delimiters = vec![(10u16, 11u16), (20, 21)];
    assert!(ErrorRecoveryState::is_scope_delimiter(10, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(11, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(20, &delimiters));
    assert!(ErrorRecoveryState::is_scope_delimiter(21, &delimiters));
    assert!(!ErrorRecoveryState::is_scope_delimiter(99, &delimiters));
    assert!(ErrorRecoveryState::is_matching_delimiter(
        10,
        11,
        &delimiters
    ));
    assert!(ErrorRecoveryState::is_matching_delimiter(
        20,
        21,
        &delimiters
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        10,
        21,
        &delimiters
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        20,
        11,
        &delimiters
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Additional builder chaining and set_max_recovery_attempts (bonus)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn builder_set_max_recovery_attempts_alias() {
    let cfg = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(42)
        .build();
    assert_eq!(cfg.max_consecutive_errors, 42);
}

#[test]
fn builder_default_trait() {
    let builder: ErrorRecoveryConfigBuilder = Default::default();
    let cfg = builder.build();
    assert_eq!(cfg.max_panic_skip, 50);
}

#[test]
fn config_can_delete_sync_token_is_false() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(100));
    assert!(!cfg.can_delete_token(SymbolId(100)));
}

#[test]
fn config_can_replace_sync_token_is_false() {
    let mut cfg = ErrorRecoveryConfig::default();
    cfg.sync_tokens.push(SymbolId(100));
    assert!(!cfg.can_replace_token(SymbolId(100)));
}

#[test]
fn config_debug_output_contains_field_names() {
    let cfg = ErrorRecoveryConfig::default();
    let dbg = format!("{:?}", cfg);
    assert!(dbg.contains("max_panic_skip"));
    assert!(dbg.contains("sync_tokens"));
    assert!(dbg.contains("enable_phrase_recovery"));
}
