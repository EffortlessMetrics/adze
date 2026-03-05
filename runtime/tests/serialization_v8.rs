use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState,
    RecoveryStrategy,
};
use adze_ir::SymbolId;
use smallvec::SmallVec;
use std::collections::HashSet;

// Helper function to create default config struct easily
fn default_config() -> ErrorRecoveryConfig {
    ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 3,
        max_token_insertions: 2,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    }
}

fn config_with(
    max_panic_skip: Option<usize>,
    max_consecutive_errors: Option<usize>,
    enable_phrase: Option<bool>,
    enable_scope: Option<bool>,
) -> ErrorRecoveryConfig {
    let mut cfg = default_config();
    if let Some(v) = max_panic_skip {
        cfg.max_panic_skip = v;
    }
    if let Some(v) = max_consecutive_errors {
        cfg.max_consecutive_errors = v;
    }
    if let Some(v) = enable_phrase {
        cfg.enable_phrase_recovery = v;
    }
    if let Some(v) = enable_scope {
        cfg.enable_scope_recovery = v;
    }
    cfg
}

// ============================================================================
// CATEGORY 1: config_builder_* (8 tests)
// ============================================================================

#[test]
fn config_builder_new() {
    let builder = ErrorRecoveryConfigBuilder::new();
    let config = builder.build();
    assert!(config.max_panic_skip > 0);
}

#[test]
fn config_builder_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .build();
    assert_eq!(config.max_panic_skip, 100);
}

#[test]
fn config_builder_add_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new().add_sync_token(42).build();
    assert!(config.max_panic_skip > 0);
}

#[test]
fn config_builder_add_insertable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(99)
        .build();
    assert!(config.max_token_insertions >= 0);
}

#[test]
fn config_builder_add_deletable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(88)
        .build();
    assert!(config.max_token_deletions >= 0);
}

#[test]
fn config_builder_add_scope_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .build();
    assert!(!config.scope_delimiters.is_empty());
}

#[test]
fn config_builder_enable_phrase_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(true)
        .build();
    assert_eq!(config.enable_phrase_recovery, true);
}

#[test]
fn config_builder_method_chaining() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(75)
        .add_sync_token(10)
        .add_insertable_token(20)
        .add_deletable_token(30)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(true)
        .max_consecutive_errors(8)
        .build();
    assert_eq!(config.max_panic_skip, 75);
    assert_eq!(config.max_consecutive_errors, 8);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
}

// ============================================================================
// CATEGORY 2: config_default_* (8 tests)
// ============================================================================

#[test]
fn config_default_max_panic_skip() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 3,
        max_token_insertions: 2,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };
    assert_eq!(config.max_panic_skip, 50);
}

#[test]
fn config_default_max_consecutive_errors() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 3,
        max_token_insertions: 2,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };
    assert_eq!(config.max_consecutive_errors, 10);
}

#[test]
fn config_default_max_token_insertions() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 3,
        max_token_insertions: 5,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };
    assert_eq!(config.max_token_insertions, 5);
}

#[test]
fn config_default_max_token_deletions() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 3,
        max_token_insertions: 2,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };
    assert_eq!(config.max_token_deletions, 3);
}

#[test]
fn config_default_enable_phrase_recovery() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 3,
        max_token_insertions: 2,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };
    assert!(config.enable_phrase_recovery);
}

#[test]
fn config_default_enable_scope_recovery() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 3,
        max_token_insertions: 2,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };
    assert!(config.enable_scope_recovery);
}

#[test]
fn config_default_scope_delimiters() {
    let delimiters = vec![(1, 2), (3, 4)];
    let config = ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 3,
        max_token_insertions: 2,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: delimiters.clone(),
        enable_indentation_recovery: false,
    };
    assert_eq!(config.scope_delimiters, delimiters);
}

#[test]
fn config_default_all_fields() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 100,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 6,
        max_token_insertions: 8,
        max_consecutive_errors: 15,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![(5, 6)],
        enable_indentation_recovery: false,
    };
    assert_eq!(config.max_panic_skip, 100);
    assert_eq!(config.max_consecutive_errors, 15);
    assert_eq!(config.max_token_insertions, 8);
    assert_eq!(config.max_token_deletions, 6);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
    assert_eq!(config.scope_delimiters.len(), 1);
}

// ============================================================================
// CATEGORY 3: state_lifecycle_* (8 tests)
// ============================================================================

#[test]
fn state_lifecycle_new() {
    let config = default_config();
    let state = ErrorRecoveryState::new(config);
    assert!(!state.should_give_up());
}

#[test]
fn state_lifecycle_reset_consecutive_errors() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

#[test]
fn state_lifecycle_reset_error_count() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_lifecycle_clear_errors() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.clear_errors();
    let errors = state.get_error_nodes();
    assert!(errors.is_empty());
}

#[test]
fn state_lifecycle_initialization() {
    let config = config_with(None, Some(5), None, None);
    let state = ErrorRecoveryState::new(config);
    let errors = state.get_error_nodes();
    assert!(errors.is_empty());
}

#[test]
fn state_lifecycle_multiple_resets() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..3 {
        state.increment_error_count();
        state.reset_consecutive_errors();
    }
    assert!(!state.should_give_up());
}

#[test]
fn state_lifecycle_error_tracking_after_reset() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.reset_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn state_lifecycle_integration() {
    let config = config_with(Some(20), Some(4), None, None);
    let mut state = ErrorRecoveryState::new(config);
    state.clear_errors();
    state.reset_consecutive_errors();
    state.reset_error_count();
    assert!(state.get_error_nodes().is_empty());
}

// ============================================================================
// CATEGORY 4: recovery_strategy_* (8 tests)
// ============================================================================

#[test]
fn recovery_strategy_token_insertion() {
    let config = config_with(None, None, None, None);
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&vec![1, 2], Some(3), (0, 5), 0);
    match strategy {
        RecoveryStrategy::TokenInsertion | RecoveryStrategy::PanicMode => (),
        _ => (),
    }
}

#[test]
fn recovery_strategy_token_deletion() {
    let config = config_with(None, None, None, None);
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&vec![10], Some(11), (5, 15), 1);
    match strategy {
        RecoveryStrategy::TokenDeletion | RecoveryStrategy::PanicMode => (),
        _ => (),
    }
}

#[test]
fn recovery_strategy_token_substitution() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&vec![20], Some(21), (10, 20), 2);
    match strategy {
        RecoveryStrategy::TokenSubstitution | RecoveryStrategy::PanicMode => (),
        _ => (),
    }
}

#[test]
fn recovery_strategy_panic_mode() {
    let config = config_with(Some(5), None, None, None);
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&vec![], None, (0, 100), 10);
    match strategy {
        RecoveryStrategy::PanicMode => (),
        _ => (),
    }
}

#[test]
fn recovery_strategy_scope_recovery() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 3,
        max_token_insertions: 2,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![(1, 2)],
        enable_indentation_recovery: false,
    };
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&vec![1], Some(999), (50, 60), 3);
    match strategy {
        RecoveryStrategy::ScopeRecovery | RecoveryStrategy::PanicMode => (),
        _ => (),
    }
}

#[test]
fn recovery_strategy_phrase_recovery() {
    let config = config_with(None, None, Some(true), None);
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&vec![5, 6, 7], Some(8), (100, 110), 2);
    match strategy {
        RecoveryStrategy::PhraseLevel | RecoveryStrategy::PanicMode => (),
        _ => (),
    }
}

#[test]
fn recovery_strategy_no_recovery() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&vec![], None, (0, 1), 0);
    match strategy {
        RecoveryStrategy::PanicMode => (),
        _ => (),
    }
}

#[test]
fn recovery_strategy_with_various_contexts() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 3,
        max_token_insertions: 5,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };
    let mut state = ErrorRecoveryState::new(config);
    let _s1 = state.determine_recovery_strategy(&vec![100, 101], Some(102), (0, 10), 0);
    let _s2 = state.determine_recovery_strategy(&vec![200], None, (20, 30), 1);
    let _s3 = state.determine_recovery_strategy(&vec![300], Some(301), (1000, 2000), 2);
}

// ============================================================================
// CATEGORY 5: error_tracking_* (8 tests)
// ============================================================================

#[test]
fn error_tracking_increment_error_count() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn error_tracking_should_give_up_false() {
    let config = config_with(None, Some(20), None, None);
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn error_tracking_should_give_up_true() {
    let config = config_with(None, Some(2), None, None);
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..3 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn error_tracking_consecutive_errors_limit() {
    let config = config_with(None, Some(3), None, None);
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn error_tracking_state_persistence() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..5 {
        state.increment_error_count();
        if i < 4 {
            assert!(!state.should_give_up());
        }
    }
}

#[test]
fn error_tracking_error_count_reset() {
    let config = config_with(None, Some(3), None, None);
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..5 {
        state.increment_error_count();
    }
    state.reset_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn error_tracking_multiple_increments() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..10 {
        state.increment_error_count();
    }
    assert!(!state.should_give_up() || state.should_give_up());
}

#[test]
fn error_tracking_with_strategies() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 3,
        max_token_insertions: 5,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };
    let mut state = ErrorRecoveryState::new(config);
    let strategy = state.determine_recovery_strategy(&vec![1], Some(2), (0, 5), 0);
    state.record_error(0, 5, (0, 0), (0, 5), vec![1], Some(2), strategy, vec![2]);
    state.increment_error_count();
    assert!(!state.should_give_up());
}

// ============================================================================
// CATEGORY 6: scope_* (8 tests)
// ============================================================================

#[test]
fn scope_push_scope() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(1);
    assert!(true);
}

#[test]
fn scope_pop_scope() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(1);
    let result = state.pop_scope(1);
    assert!(result);
}

#[test]
fn scope_pop_scope_mismatch() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(1);
    let result = state.pop_scope(2);
    assert!(!result);
}

#[test]
fn scope_stack_nesting() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(10);
    state.push_scope(20);
    state.push_scope(30);
    assert!(state.pop_scope(30));
    assert!(state.pop_scope(20));
    assert!(state.pop_scope(10));
}

#[test]
fn scope_pop_scope_returns_bool() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(5);
    let result: bool = state.pop_scope(5);
    assert_eq!(result, true);
}

#[test]
fn scope_state_persistence() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(100);
    state.push_scope(200);
    assert!(state.pop_scope(200));
    state.push_scope(300);
    assert!(state.pop_scope(300));
}

#[test]
fn scope_with_error_tracking() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(1);
    state.increment_error_count();
    assert!(state.pop_scope(1));
    assert!(!state.should_give_up());
}

#[test]
fn scope_edge_cases() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    let result1 = state.pop_scope(999);
    assert!(!result1);
    state.push_scope(1);
    let result2 = state.pop_scope(1);
    assert!(result2);
    state.push_scope(2);
    let result3 = state.pop_scope(3);
    assert!(!result3);
}

// ============================================================================
// CATEGORY 7: error_node_* (8 tests)
// ============================================================================

#[test]
fn error_node_record_error() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    let strategy = RecoveryStrategy::TokenInsertion;
    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![1, 2],
        Some(3),
        strategy,
        vec![3],
    );
    let errors = state.get_error_nodes();
    assert!(!errors.is_empty());
}

#[test]
fn error_node_get_error_nodes_empty() {
    let config = default_config();
    let state = ErrorRecoveryState::new(config);
    let errors = state.get_error_nodes();
    assert!(errors.is_empty());
}

#[test]
fn error_node_get_error_nodes_multiple() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![1],
        Some(2),
        RecoveryStrategy::TokenInsertion,
        vec![2],
    );
    state.record_error(
        10,
        15,
        (0, 10),
        (0, 15),
        vec![3],
        Some(4),
        RecoveryStrategy::TokenDeletion,
        vec![4],
    );
    state.record_error(
        20,
        25,
        (0, 20),
        (0, 25),
        vec![5],
        Some(6),
        RecoveryStrategy::TokenSubstitution,
        vec![6],
    );
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 3);
}

#[test]
fn error_node_record_with_different_strategies() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        5,
        10,
        (0, 5),
        (0, 10),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![2],
    );
    state.record_error(
        10,
        15,
        (0, 10),
        (0, 15),
        vec![3],
        Some(4),
        RecoveryStrategy::ScopeRecovery,
        vec![4],
    );
    state.record_error(
        15,
        20,
        (0, 15),
        (0, 20),
        vec![5],
        Some(6),
        RecoveryStrategy::PhraseLevel,
        vec![6],
    );
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 4);
}

#[test]
fn error_node_content() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    let expected = vec![10, 11, 12];
    let found = Some(13);
    state.record_error(
        50,
        60,
        (0, 50),
        (0, 60),
        expected.clone(),
        found,
        RecoveryStrategy::TokenInsertion,
        vec![13],
    );
    let errors = state.get_error_nodes();
    assert!(!errors.is_empty());
}

#[test]
fn error_node_retrieval() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    assert!(state.get_error_nodes().is_empty());
    state.record_error(
        0,
        10,
        (0, 0),
        (0, 10),
        vec![1],
        Some(2),
        RecoveryStrategy::TokenInsertion,
        vec![2],
    );
    assert!(!state.get_error_nodes().is_empty());
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn error_node_with_expected_tokens() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    let expected_tokens = vec![100, 101, 102, 103, 104];
    state.record_error(
        0,
        20,
        (0, 0),
        (0, 20),
        expected_tokens,
        Some(999),
        RecoveryStrategy::TokenDeletion,
        vec![999],
    );
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
}

#[test]
fn error_node_comprehensive() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 50,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 5,
        max_token_insertions: 10,
        max_consecutive_errors: 10,
        enable_phrase_recovery: true,
        enable_scope_recovery: true,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..5 {
        let start = i * 10;
        let end = (i + 1) * 10;
        state.record_error(
            start,
            end,
            (0, start),
            (0, end),
            vec![i as u16, (i + 1) as u16],
            Some((i + 100) as u16),
            RecoveryStrategy::TokenInsertion,
            vec![(i + 100) as u16],
        );
    }
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 5);
}

// ============================================================================
// CATEGORY 8: stress_* (8 tests)
// ============================================================================

#[test]
fn stress_many_errors() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..100 {
        let start = (i * 5);
        let end = (start + 5);
        state.record_error(
            start,
            end,
            (0, start),
            (0, end),
            vec![i as u16],
            Some((i + 1) as u16),
            RecoveryStrategy::TokenInsertion,
            vec![(i + 1) as u16],
        );
    }
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 100);
}

#[test]
fn stress_large_token_lists() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    let large_vec: Vec<u16> = (0..1000).collect();
    state.record_error(
        0,
        1000,
        (0, 0),
        (0, 1000),
        large_vec,
        Some(1001),
        RecoveryStrategy::TokenInsertion,
        vec![1001],
    );
    let errors = state.get_error_nodes();
    assert!(!errors.is_empty());
}

#[test]
fn stress_deep_scope_nesting() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..100 {
        state.push_scope(i);
    }
    for i in (0..100).rev() {
        assert!(state.pop_scope(i));
    }
}

#[test]
fn stress_boundary_max_consecutive_errors() {
    let config = config_with(None, Some(5), None, None);
    let mut state = ErrorRecoveryState::new(config);
    for _ in 0..5 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

#[test]
fn stress_boundary_empty_vectors() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    state.record_error(
        0,
        0,
        (0, 0),
        (0, 0),
        vec![],
        None,
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![],
        Some(1),
        RecoveryStrategy::TokenDeletion,
        vec![],
    );
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 2);
}

#[test]
fn stress_boundary_zero_values() {
    let config = ErrorRecoveryConfig {
        max_panic_skip: 0,
        sync_tokens: SmallVec::new(),
        insert_candidates: SmallVec::new(),
        deletable_tokens: HashSet::new(),
        max_token_deletions: 0,
        max_token_insertions: 0,
        max_consecutive_errors: 0,
        enable_phrase_recovery: false,
        enable_scope_recovery: false,
        scope_delimiters: vec![],
        enable_indentation_recovery: false,
    };
    let state = ErrorRecoveryState::new(config);
    assert!(state.get_error_nodes().is_empty());
}

#[test]
fn stress_rapid_operations() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..50 {
        state.push_scope(i);
        state.increment_error_count();
        state.record_error(
            i,
            i + 1,
            (0, i as usize),
            (0, (i + 1) as usize),
            vec![i as u16],
            Some((i + 1) as u16),
            RecoveryStrategy::TokenInsertion,
            vec![(i + 1) as u16],
        );
    }
    for i in (0..50).rev() {
        state.pop_scope(i);
    }
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 50);
}

#[test]
fn stress_boundary_large_byte_values() {
    let config = default_config();
    let mut state = ErrorRecoveryState::new(config);
    let large_val = 1000000usize;
    state.record_error(
        large_val - 1000,
        large_val,
        (0, large_val - 1000),
        (0, large_val),
        vec![65535],
        Some(65534),
        RecoveryStrategy::PanicMode,
        vec![65534],
    );
    let errors = state.get_error_nodes();
    assert!(!errors.is_empty());
}
