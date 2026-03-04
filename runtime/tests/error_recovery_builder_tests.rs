//! Tests for the ErrorRecoveryConfigBuilder API.

use adze::error_recovery::ErrorRecoveryConfigBuilder;

#[test]
fn builder_default() {
    let config = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(config.max_panic_skip, 50);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
}

#[test]
fn builder_max_panic_skip() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(200)
        .build();
    assert_eq!(config.max_panic_skip, 200);
}

#[test]
fn builder_add_sync_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_sync_token(1)
        .add_sync_token(2)
        .add_sync_token(3)
        .build();
    assert_eq!(config.sync_tokens.len(), 3);
}

#[test]
fn builder_add_insertable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_insertable_token(10)
        .build();
    assert_eq!(config.insert_candidates.len(), 1);
}

#[test]
fn builder_add_deletable_token() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_deletable_token(5)
        .build();
    assert!(config.deletable_tokens.contains(&5));
}

#[test]
fn builder_max_consecutive_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(20)
        .build();
    assert_eq!(config.max_consecutive_errors, 20);
}

#[test]
fn builder_disable_phrase_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .build();
    assert!(!config.enable_phrase_recovery);
}

#[test]
fn builder_disable_scope_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!config.enable_scope_recovery);
}

#[test]
fn builder_add_scope_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(40, 41) // ( and )
        .add_scope_delimiter(91, 93) // [ and ]
        .build();
    assert_eq!(config.scope_delimiters.len(), 2);
}

#[test]
fn builder_enable_indentation_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_indentation_recovery(true)
        .build();
    assert!(config.enable_indentation_recovery);
}

#[test]
fn builder_chaining() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(100)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .max_consecutive_errors(15)
        .enable_phrase_recovery(true)
        .enable_scope_recovery(true)
        .add_scope_delimiter(40, 41)
        .build();
    assert_eq!(config.max_panic_skip, 100);
    assert_eq!(config.sync_tokens.len(), 1);
    assert_eq!(config.insert_candidates.len(), 1);
    assert!(config.deletable_tokens.contains(&3));
    assert_eq!(config.max_consecutive_errors, 15);
    assert!(config.enable_phrase_recovery);
    assert!(config.enable_scope_recovery);
    assert_eq!(config.scope_delimiters.len(), 1);
}
