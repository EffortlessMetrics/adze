//! Comprehensive tests for error recovery types and state management.

use adze::adze_ir as ir;
use adze::error_recovery::{
    ErrorNode, ErrorRecoveryConfig, ErrorRecoveryState, RecoveryAction, RecoveryStrategy,
};

use ir::SymbolId;

// ── RecoveryStrategy ──

#[test]
fn recovery_strategy_panic_mode() {
    let s = RecoveryStrategy::PanicMode;
    let d = format!("{:?}", s);
    assert!(d.contains("PanicMode"));
}

#[test]
fn recovery_strategy_token_insertion() {
    let s = RecoveryStrategy::TokenInsertion;
    let d = format!("{:?}", s);
    assert!(d.contains("TokenInsertion"));
}

#[test]
fn recovery_strategy_token_deletion() {
    let s = RecoveryStrategy::TokenDeletion;
    let d = format!("{:?}", s);
    assert!(d.contains("TokenDeletion"));
}

#[test]
fn recovery_strategy_token_substitution() {
    let s = RecoveryStrategy::TokenSubstitution;
    let d = format!("{:?}", s);
    assert!(d.contains("TokenSubstitution"));
}

#[test]
fn recovery_strategy_phrase_level() {
    let s = RecoveryStrategy::PhraseLevel;
    let d = format!("{:?}", s);
    assert!(d.contains("PhraseLevel"));
}

#[test]
fn recovery_strategy_clone() {
    let s1 = RecoveryStrategy::PanicMode;
    let s2 = s1;
    assert_eq!(format!("{:?}", s1), format!("{:?}", s2));
}

#[test]
fn recovery_strategy_eq() {
    assert_eq!(RecoveryStrategy::PanicMode, RecoveryStrategy::PanicMode);
    assert_ne!(
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion
    );
}

// ── RecoveryAction ──

#[test]
fn recovery_action_delete() {
    let a = RecoveryAction::DeleteToken;
    let d = format!("{:?}", a);
    assert!(d.contains("Delete"));
}

#[test]
fn recovery_action_insert() {
    let a = RecoveryAction::InsertToken(SymbolId(1));
    let d = format!("{:?}", a);
    assert!(d.contains("Insert"));
}

#[test]
fn recovery_action_clone() {
    let a1 = RecoveryAction::DeleteToken;
    let a2 = a1.clone();
    assert_eq!(format!("{:?}", a1), format!("{:?}", a2));
}

// ── ErrorRecoveryConfig ──

#[test]
fn config_default() {
    let config = ErrorRecoveryConfig::default();
    assert!(config.max_panic_skip > 0);
}

#[test]
fn config_debug() {
    let config = ErrorRecoveryConfig::default();
    let d = format!("{:?}", config);
    assert!(!d.is_empty());
}

#[test]
fn config_clone() {
    let c1 = ErrorRecoveryConfig::default();
    let c2 = c1.clone();
    assert_eq!(c1.max_panic_skip, c2.max_panic_skip);
}

#[test]
fn config_can_delete_token() {
    let config = ErrorRecoveryConfig::default();
    let _ = config.can_delete_token(SymbolId(1));
}

#[test]
fn config_can_replace_token() {
    let config = ErrorRecoveryConfig::default();
    let _ = config.can_replace_token(SymbolId(1));
}

#[test]
fn config_field_max_token_deletions() {
    let config = ErrorRecoveryConfig::default();
    let _ = config.max_token_deletions;
}

#[test]
fn config_field_max_token_insertions() {
    let config = ErrorRecoveryConfig::default();
    let _ = config.max_token_insertions;
}

#[test]
fn config_field_max_consecutive_errors() {
    let config = ErrorRecoveryConfig::default();
    let _ = config.max_consecutive_errors;
}

#[test]
fn config_field_enable_phrase_recovery() {
    let config = ErrorRecoveryConfig::default();
    let _ = config.enable_phrase_recovery;
}

#[test]
fn config_field_enable_scope_recovery() {
    let config = ErrorRecoveryConfig::default();
    let _ = config.enable_scope_recovery;
}

#[test]
fn config_field_enable_indentation_recovery() {
    let config = ErrorRecoveryConfig::default();
    let _ = config.enable_indentation_recovery;
}

// ── ErrorRecoveryState ──

#[test]
fn state_new() {
    let config = ErrorRecoveryConfig::default();
    let _state = ErrorRecoveryState::new(config);
}

#[test]
fn state_add_recent_token() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    state.add_recent_token(1);
    state.add_recent_token(2);
}

#[test]
fn state_add_many_tokens() {
    let config = ErrorRecoveryConfig::default();
    let mut state = ErrorRecoveryState::new(config);
    for i in 0..100 {
        state.add_recent_token(i);
    }
}

// ── ErrorNode ──

#[test]
fn error_node_construction() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 5,
        start_position: (0, 0),
        end_position: (0, 5),
        expected: vec![1, 2, 3],
        actual: Some(4),
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    assert_eq!(node.start_byte, 0);
    assert_eq!(node.end_byte, 5);
}

#[test]
fn error_node_no_actual() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 0,
        start_position: (0, 0),
        end_position: (0, 0),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::TokenInsertion,
        skipped_tokens: vec![],
    };
    assert!(node.actual.is_none());
}

#[test]
fn error_node_with_skipped() {
    let node = ErrorNode {
        start_byte: 10,
        end_byte: 20,
        start_position: (1, 0),
        end_position: (1, 10),
        expected: vec![5],
        actual: Some(6),
        recovery: RecoveryStrategy::TokenDeletion,
        skipped_tokens: vec![7, 8, 9],
    };
    assert_eq!(node.skipped_tokens.len(), 3);
}

#[test]
fn error_node_debug() {
    let node = ErrorNode {
        start_byte: 0,
        end_byte: 1,
        start_position: (0, 0),
        end_position: (0, 1),
        expected: vec![],
        actual: None,
        recovery: RecoveryStrategy::PanicMode,
        skipped_tokens: vec![],
    };
    let d = format!("{:?}", node);
    assert!(!d.is_empty());
}

// ── Multiple states ──

#[test]
fn multiple_states_independent() {
    let mut s1 = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    let mut s2 = ErrorRecoveryState::new(ErrorRecoveryConfig::default());
    s1.add_recent_token(1);
    s2.add_recent_token(2);
}
