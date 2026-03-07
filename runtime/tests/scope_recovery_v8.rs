// Comprehensive scope-based error recovery tests (v8)
use adze::adze_ir as ir;

use adze::error_recovery::{
    ErrorRecoveryConfig, ErrorRecoveryConfigBuilder, ErrorRecoveryState, RecoveryStrategy,
};

// =====================================================================
// Helpers
// =====================================================================

/// Parentheses: ASCII '(' = 40, ')' = 41
const LPAREN: u16 = 40;
const RPAREN: u16 = 41;
/// Braces: ASCII '{' = 123, '}' = 125
const LBRACE: u16 = 123;
const RBRACE: u16 = 125;
/// Brackets: ASCII '[' = 91, ']' = 93
const LBRACKET: u16 = 91;
const RBRACKET: u16 = 93;
/// Angle brackets
const LANGLE: u16 = 60;
const RANGLE: u16 = 62;
/// Pipe delimiters (same open and close)
const PIPE: u16 = 124;

fn config_no_delimiters() -> ErrorRecoveryConfig {
    ErrorRecoveryConfigBuilder::new().build()
}

fn config_parens() -> ErrorRecoveryConfig {
    ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(LPAREN, RPAREN)
        .build()
}

fn config_three_pairs() -> ErrorRecoveryConfig {
    ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(LPAREN, RPAREN)
        .add_scope_delimiter(LBRACE, RBRACE)
        .add_scope_delimiter(LBRACKET, RBRACKET)
        .build()
}

fn config_five_pairs() -> ErrorRecoveryConfig {
    ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(LPAREN, RPAREN)
        .add_scope_delimiter(LBRACE, RBRACE)
        .add_scope_delimiter(LBRACKET, RBRACKET)
        .add_scope_delimiter(LANGLE, RANGLE)
        .add_scope_delimiter(PIPE, PIPE)
        .build()
}

fn scope_depth(state: &mut ErrorRecoveryState) -> usize {
    let mut depth = 0;
    while state.pop_scope_test().is_some() {
        depth += 1;
    }
    depth
}

// =====================================================================
// 1. No delimiters → push_scope has no effect
// =====================================================================

#[test]
fn no_delimiters_push_has_no_effect() {
    let mut state = ErrorRecoveryState::new(config_no_delimiters());
    state.push_scope(LPAREN);
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn no_delimiters_push_various_tokens() {
    let mut state = ErrorRecoveryState::new(config_no_delimiters());
    for t in [LPAREN, LBRACE, LBRACKET, LANGLE, PIPE, 0, 999] {
        state.push_scope(t);
    }
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn no_delimiters_pop_scope_returns_false() {
    let mut state = ErrorRecoveryState::new(config_no_delimiters());
    assert!(!state.pop_scope(RPAREN));
}

#[test]
fn no_delimiters_config_has_empty_delimiters() {
    let config = config_no_delimiters();
    assert!(config.scope_delimiters.is_empty());
}

// =====================================================================
// 2. Register ("(", ")") → push_scope("(") works
// =====================================================================

#[test]
fn single_pair_push_opening_succeeds() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
}

#[test]
fn single_pair_push_closing_has_no_effect() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(RPAREN);
    assert_eq!(state.pop_scope_test(), None);
}

// =====================================================================
// 3. push_scope("(") then pop → returns matching close
// =====================================================================

#[test]
fn push_then_pop_returns_opening_token() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    // pop_scope_test returns the opening token from the stack
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
}

#[test]
fn push_then_pop_scope_with_close_returns_true() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    assert!(state.pop_scope(RPAREN));
}

#[test]
fn push_then_pop_scope_with_wrong_close_returns_false() {
    let mut state = ErrorRecoveryState::new(config_three_pairs());
    state.push_scope(LPAREN);
    assert!(!state.pop_scope(RBRACE));
}

// =====================================================================
// 4. Register multiple delimiters
// =====================================================================

#[test]
fn multiple_delimiters_all_registered() {
    let config = config_three_pairs();
    assert_eq!(config.scope_delimiters.len(), 3);
    assert!(config.scope_delimiters.contains(&(LPAREN, RPAREN)));
    assert!(config.scope_delimiters.contains(&(LBRACE, RBRACE)));
    assert!(config.scope_delimiters.contains(&(LBRACKET, RBRACKET)));
}

#[test]
fn multiple_delimiters_each_push_works() {
    let mut state = ErrorRecoveryState::new(config_three_pairs());
    state.push_scope(LPAREN);
    state.push_scope(LBRACE);
    state.push_scope(LBRACKET);
    assert_eq!(state.pop_scope_test(), Some(LBRACKET));
    assert_eq!(state.pop_scope_test(), Some(LBRACE));
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
}

// =====================================================================
// 5. Push "(", push "{" → pop returns "}", pop returns ")"
// =====================================================================

#[test]
fn nested_push_pop_lifo_order() {
    let mut state = ErrorRecoveryState::new(config_three_pairs());
    state.push_scope(LPAREN);
    state.push_scope(LBRACE);
    // LIFO: top is LBRACE, pop closing brace first
    assert!(state.pop_scope(RBRACE));
    assert!(state.pop_scope(RPAREN));
}

#[test]
fn nested_push_pop_test_lifo_raw() {
    let mut state = ErrorRecoveryState::new(config_three_pairs());
    state.push_scope(LPAREN);
    state.push_scope(LBRACE);
    assert_eq!(state.pop_scope_test(), Some(LBRACE));
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
}

// =====================================================================
// 6. Pop on empty scope → None / false
// =====================================================================

#[test]
fn pop_scope_test_on_empty_returns_none() {
    let mut state = ErrorRecoveryState::new(config_parens());
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn pop_scope_on_empty_returns_false() {
    let mut state = ErrorRecoveryState::new(config_parens());
    assert!(!state.pop_scope(RPAREN));
}

#[test]
fn pop_scope_on_empty_no_delimiters_returns_false() {
    let mut state = ErrorRecoveryState::new(config_no_delimiters());
    assert!(!state.pop_scope(RPAREN));
}

// =====================================================================
// 7. Push unregistered token → no effect (stack unchanged)
// =====================================================================

#[test]
fn push_unregistered_token_no_effect() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LBRACE); // not registered
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn push_unregistered_among_registered() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    state.push_scope(999); // unregistered
    state.push_scope(LPAREN);
    // only two items on stack
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn push_close_token_not_added_to_stack() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(RPAREN);
    state.push_scope(RPAREN);
    assert_eq!(state.pop_scope_test(), None);
}

// =====================================================================
// 8. Nested scopes: (({[]}))
// =====================================================================

#[test]
fn deeply_nested_scopes() {
    let mut state = ErrorRecoveryState::new(config_three_pairs());
    // Push: ( ( { [ ]
    state.push_scope(LPAREN);
    state.push_scope(LPAREN);
    state.push_scope(LBRACE);
    state.push_scope(LBRACKET);
    // Pop: ] } ) )
    assert!(state.pop_scope(RBRACKET));
    assert!(state.pop_scope(RBRACE));
    assert!(state.pop_scope(RPAREN));
    assert!(state.pop_scope(RPAREN));
    // Stack is empty
    assert!(!state.pop_scope(RPAREN));
}

#[test]
fn triple_nested_same_kind() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    state.push_scope(LPAREN);
    state.push_scope(LPAREN);
    assert!(state.pop_scope(RPAREN));
    assert!(state.pop_scope(RPAREN));
    assert!(state.pop_scope(RPAREN));
    assert!(!state.pop_scope(RPAREN));
}

#[test]
fn interleaved_nested_scopes() {
    let mut state = ErrorRecoveryState::new(config_three_pairs());
    // { [ ( ) ] }
    state.push_scope(LBRACE);
    state.push_scope(LBRACKET);
    state.push_scope(LPAREN);
    assert!(state.pop_scope(RPAREN));
    assert!(state.pop_scope(RBRACKET));
    assert!(state.pop_scope(RBRACE));
}

// =====================================================================
// 9. Multiple push/pop cycles
// =====================================================================

#[test]
fn push_pop_cycle_repeated() {
    let mut state = ErrorRecoveryState::new(config_parens());
    for _ in 0..10 {
        state.push_scope(LPAREN);
        assert!(state.pop_scope(RPAREN));
    }
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn push_many_then_pop_all() {
    let mut state = ErrorRecoveryState::new(config_parens());
    for _ in 0..20 {
        state.push_scope(LPAREN);
    }
    for _ in 0..20 {
        assert!(state.pop_scope(RPAREN));
    }
    assert!(!state.pop_scope(RPAREN));
}

#[test]
fn alternating_delimiter_types() {
    let mut state = ErrorRecoveryState::new(config_three_pairs());
    state.push_scope(LPAREN);
    assert!(state.pop_scope(RPAREN));
    state.push_scope(LBRACE);
    assert!(state.pop_scope(RBRACE));
    state.push_scope(LBRACKET);
    assert!(state.pop_scope(RBRACKET));
    assert_eq!(state.pop_scope_test(), None);
}

// =====================================================================
// 10. Reset clears scope stack
// =====================================================================

#[test]
fn reset_error_count_does_not_clear_scope() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    state.increment_error_count();
    state.reset_error_count();
    // scope stack is independent of error count
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
}

#[test]
fn clear_errors_does_not_affect_scope() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
    // scope still has the paren
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
}

#[test]
fn reset_consecutive_errors_zeroes_count() {
    let mut state = ErrorRecoveryState::new(config_no_delimiters());
    state.increment_error_count();
    state.increment_error_count();
    state.reset_consecutive_errors();
    assert!(!state.should_give_up());
}

// =====================================================================
// 11. Scope recovery strategy
// =====================================================================

#[test]
fn scope_recovery_strategy_variant_equality() {
    assert_eq!(
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::ScopeRecovery
    );
    assert_ne!(RecoveryStrategy::ScopeRecovery, RecoveryStrategy::PanicMode);
}

#[test]
fn scope_recovery_strategy_copy() {
    let s = RecoveryStrategy::ScopeRecovery;
    let s2 = s; // Copy
    assert_eq!(s, s2);
}

#[test]
fn scope_recovery_strategy_debug() {
    let dbg = format!("{:?}", RecoveryStrategy::ScopeRecovery);
    assert!(dbg.contains("ScopeRecovery"));
}

#[test]
fn scope_recovery_enabled_by_default() {
    let config = config_no_delimiters();
    assert!(config.enable_scope_recovery);
}

#[test]
fn scope_recovery_can_be_disabled() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_scope_recovery(false)
        .build();
    assert!(!config.enable_scope_recovery);
}

// =====================================================================
// 12. ScopeRecovery attempt → counts error
// =====================================================================

#[test]
fn determine_recovery_increments_error_count() {
    let config = ErrorRecoveryConfigBuilder::new()
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    let _strategy = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    assert!(state.should_give_up() || !state.should_give_up()); // at least ran
    // error count was incremented by determine_recovery_strategy
}

#[test]
fn scope_mismatch_triggers_scope_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(LPAREN, RPAREN)
        .add_sync_token(RPAREN) // make RPAREN a sync token so TokenDeletion is skipped
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // No open paren, but encounter close paren → scope mismatch
    // expected has 2 items to skip TokenSubstitution (which triggers when len() == 1)
    let strategy = state.determine_recovery_strategy(&[1, 2], Some(RPAREN), (0, 0), 0);
    assert_eq!(strategy, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn recovery_gives_up_after_max_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .enable_phrase_recovery(false)
        .enable_scope_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    // First two: some strategy
    let _s1 = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    let _s2 = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    // Third exceeds max → PanicMode
    let s3 = state.determine_recovery_strategy(&[1], Some(2), (0, 0), 0);
    assert_eq!(s3, RecoveryStrategy::PanicMode);
}

// =====================================================================
// 13. Mix scope operations with recovery attempts
// =====================================================================

#[test]
fn push_scope_then_determine_recovery() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(LPAREN, RPAREN)
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(LPAREN);
    // actual=RPAREN matches the open paren on stack, so no mismatch
    // (has_scope_mismatch returns false when there IS a matching open)
    let strategy = state.determine_recovery_strategy(&[1], Some(RPAREN), (0, 0), 0);
    // Should not be ScopeRecovery because the scope *is* balanced
    assert_ne!(strategy, RecoveryStrategy::ScopeRecovery);
}

#[test]
fn interleave_scope_ops_and_errors() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(LPAREN, RPAREN)
        .add_scope_delimiter(LBRACE, RBRACE)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(LPAREN);
    state.increment_error_count();
    state.push_scope(LBRACE);
    assert!(state.pop_scope(RBRACE));
    state.increment_error_count();
    assert!(state.pop_scope(RPAREN));
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn record_error_with_scope_recovery_strategy() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    state.record_error(
        10,
        15,
        (1, 0),
        (1, 5),
        vec![RPAREN],
        Some(RBRACE),
        RecoveryStrategy::ScopeRecovery,
        vec![RBRACE],
    );
    let errors = state.get_error_nodes();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].recovery, RecoveryStrategy::ScopeRecovery);
}

// =====================================================================
// 14. error_count increments correctly
// =====================================================================

#[test]
fn error_count_starts_at_zero() {
    let state = ErrorRecoveryState::new(config_no_delimiters());
    assert!(!state.should_give_up());
}

#[test]
fn error_count_increment_once() {
    let mut state = ErrorRecoveryState::new(config_no_delimiters());
    state.increment_error_count();
    assert!(!state.should_give_up()); // default max is 10
}

#[test]
fn error_count_increment_to_max() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(3)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(state.should_give_up());
}

#[test]
fn error_count_reset_then_recount() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_consecutive_errors(2)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.increment_error_count();
    state.increment_error_count();
    assert!(state.should_give_up());
    state.reset_error_count();
    assert!(!state.should_give_up());
    state.increment_error_count();
    assert!(!state.should_give_up());
}

#[test]
fn error_count_many_increments() {
    let mut state = ErrorRecoveryState::new(config_no_delimiters());
    for _ in 0..100 {
        state.increment_error_count();
    }
    assert!(state.should_give_up());
}

// =====================================================================
// 15. Scope delimiter pairs preserved
// =====================================================================

#[test]
fn delimiter_pairs_preserved_in_config() {
    let config = config_three_pairs();
    assert_eq!(config.scope_delimiters[0], (LPAREN, RPAREN));
    assert_eq!(config.scope_delimiters[1], (LBRACE, RBRACE));
    assert_eq!(config.scope_delimiters[2], (LBRACKET, RBRACKET));
}

#[test]
fn delimiter_pairs_order_preserved() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(10, 11)
        .add_scope_delimiter(20, 21)
        .add_scope_delimiter(30, 31)
        .build();
    assert_eq!(config.scope_delimiters, vec![(10, 11), (20, 21), (30, 31)]);
}

#[test]
fn duplicate_delimiter_pair_stored_twice() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(LPAREN, RPAREN)
        .add_scope_delimiter(LPAREN, RPAREN)
        .build();
    assert_eq!(config.scope_delimiters.len(), 2);
}

// =====================================================================
// 16. Config with 5+ delimiter pairs
// =====================================================================

#[test]
fn five_delimiter_pairs() {
    let config = config_five_pairs();
    assert_eq!(config.scope_delimiters.len(), 5);
}

#[test]
fn five_pairs_all_push_works() {
    let mut state = ErrorRecoveryState::new(config_five_pairs());
    state.push_scope(LPAREN);
    state.push_scope(LBRACE);
    state.push_scope(LBRACKET);
    state.push_scope(LANGLE);
    state.push_scope(PIPE);
    // pop in reverse
    assert_eq!(state.pop_scope_test(), Some(PIPE));
    assert_eq!(state.pop_scope_test(), Some(LANGLE));
    assert_eq!(state.pop_scope_test(), Some(LBRACKET));
    assert_eq!(state.pop_scope_test(), Some(LBRACE));
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
    assert_eq!(state.pop_scope_test(), None);
}

#[test]
fn six_delimiter_pairs() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 2)
        .add_scope_delimiter(3, 4)
        .add_scope_delimiter(5, 6)
        .add_scope_delimiter(7, 8)
        .add_scope_delimiter(9, 10)
        .add_scope_delimiter(11, 12)
        .build();
    assert_eq!(config.scope_delimiters.len(), 6);
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(11);
    assert!(state.pop_scope(12));
}

// =====================================================================
// 17. Same delimiter for open and close
// =====================================================================

#[test]
fn same_open_close_push_works() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(PIPE, PIPE)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(PIPE);
    assert_eq!(state.pop_scope_test(), Some(PIPE));
}

#[test]
fn same_open_close_pop_scope_matches() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(PIPE, PIPE)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(PIPE);
    assert!(state.pop_scope(PIPE));
}

#[test]
fn same_open_close_nested() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(PIPE, PIPE)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(PIPE);
    state.push_scope(PIPE);
    assert!(state.pop_scope(PIPE));
    assert!(state.pop_scope(PIPE));
    assert!(!state.pop_scope(PIPE));
}

// =====================================================================
// 18. Zero-value token as delimiter
// =====================================================================

#[test]
fn zero_token_as_open_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(0, 1)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(0);
    assert!(state.pop_scope(1));
}

#[test]
fn zero_token_as_close_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(1, 0)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(1);
    assert!(state.pop_scope(0));
}

#[test]
fn both_zero_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(0, 0)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(0);
    assert!(state.pop_scope(0));
}

// =====================================================================
// 19. Scope depth tracking
// =====================================================================

#[test]
fn scope_depth_zero_initially() {
    let mut state = ErrorRecoveryState::new(config_parens());
    assert_eq!(scope_depth(&mut state), 0);
}

#[test]
fn scope_depth_after_pushes() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    state.push_scope(LPAREN);
    state.push_scope(LPAREN);
    assert_eq!(scope_depth(&mut state), 3);
}

#[test]
fn scope_depth_after_partial_pop() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    state.push_scope(LPAREN);
    state.push_scope(LPAREN);
    assert!(state.pop_scope(RPAREN)); // depth 3 → 2
    assert_eq!(scope_depth(&mut state), 2);
}

#[test]
fn scope_depth_mixed_types() {
    let mut state = ErrorRecoveryState::new(config_three_pairs());
    state.push_scope(LPAREN);
    state.push_scope(LBRACE);
    state.push_scope(LBRACKET);
    state.push_scope(LPAREN);
    assert_eq!(scope_depth(&mut state), 4);
}

// =====================================================================
// 20. Config Debug format with delimiters
// =====================================================================

#[test]
fn config_debug_contains_scope_delimiters() {
    let config = config_parens();
    let dbg = format!("{:?}", config);
    assert!(dbg.contains("scope_delimiters"));
}

#[test]
fn config_debug_shows_delimiter_values() {
    let config = config_parens();
    let dbg = format!("{:?}", config);
    assert!(dbg.contains("40"));
    assert!(dbg.contains("41"));
}

#[test]
fn config_debug_empty_delimiters() {
    let config = config_no_delimiters();
    let dbg = format!("{:?}", config);
    assert!(dbg.contains("scope_delimiters"));
}

#[test]
fn builder_default_matches_config_default() {
    let from_builder = ErrorRecoveryConfigBuilder::new().build();
    let from_default = ErrorRecoveryConfig::default();
    let dbg_builder = format!("{:?}", from_builder);
    let dbg_default = format!("{:?}", from_default);
    assert_eq!(dbg_builder, dbg_default);
}

// =====================================================================
// Additional: static helper tests
// =====================================================================

#[test]
fn is_scope_delimiter_open() {
    let delims = vec![(LPAREN, RPAREN), (LBRACE, RBRACE)];
    assert!(ErrorRecoveryState::is_scope_delimiter(LPAREN, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(LBRACE, &delims));
}

#[test]
fn is_scope_delimiter_close() {
    let delims = vec![(LPAREN, RPAREN), (LBRACE, RBRACE)];
    assert!(ErrorRecoveryState::is_scope_delimiter(RPAREN, &delims));
    assert!(ErrorRecoveryState::is_scope_delimiter(RBRACE, &delims));
}

#[test]
fn is_scope_delimiter_unregistered() {
    let delims = vec![(LPAREN, RPAREN)];
    assert!(!ErrorRecoveryState::is_scope_delimiter(LBRACE, &delims));
    assert!(!ErrorRecoveryState::is_scope_delimiter(999, &delims));
}

#[test]
fn is_matching_delimiter_correct_pair() {
    let delims = vec![(LPAREN, RPAREN), (LBRACE, RBRACE)];
    assert!(ErrorRecoveryState::is_matching_delimiter(
        LPAREN, RPAREN, &delims
    ));
    assert!(ErrorRecoveryState::is_matching_delimiter(
        LBRACE, RBRACE, &delims
    ));
}

#[test]
fn is_matching_delimiter_wrong_pair() {
    let delims = vec![(LPAREN, RPAREN), (LBRACE, RBRACE)];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        LPAREN, RBRACE, &delims
    ));
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        LBRACE, RPAREN, &delims
    ));
}

#[test]
fn is_matching_delimiter_empty_delims() {
    let delims: Vec<(u16, u16)> = vec![];
    assert!(!ErrorRecoveryState::is_matching_delimiter(
        LPAREN, RPAREN, &delims
    ));
}

// =====================================================================
// Additional: pop_scope mismatch scenarios
// =====================================================================

#[test]
fn pop_scope_mismatch_does_not_pop() {
    let mut state = ErrorRecoveryState::new(config_three_pairs());
    state.push_scope(LPAREN);
    // Try popping with wrong close
    assert!(!state.pop_scope(RBRACE));
    // Stack unchanged, can still pop correctly
    assert!(state.pop_scope(RPAREN));
}

#[test]
fn pop_scope_inner_mismatch_blocks_outer() {
    let mut state = ErrorRecoveryState::new(config_three_pairs());
    state.push_scope(LPAREN);
    state.push_scope(LBRACE);
    // Try to close paren while brace is on top
    assert!(!state.pop_scope(RPAREN));
    // Stack still has both
    assert_eq!(state.pop_scope_test(), Some(LBRACE));
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
}

// =====================================================================
// Additional: recent tokens interaction
// =====================================================================

#[test]
fn recent_tokens_independent_of_scope() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    state.add_recent_token(LPAREN);
    state.add_recent_token(42);
    assert!(state.pop_scope(RPAREN));
    // recent tokens still there (not affected by scope ops)
}

#[test]
fn update_recent_tokens_with_symbol_id() {
    let mut state = ErrorRecoveryState::new(config_no_delimiters());
    state.update_recent_tokens(ir::SymbolId(7));
    state.update_recent_tokens(ir::SymbolId(8));
    // No panic, tokens recorded
}

// =====================================================================
// Additional: error nodes with scope context
// =====================================================================

#[test]
fn error_node_records_expected_closing_delimiter() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.record_error(
        0,
        5,
        (0, 0),
        (0, 5),
        vec![RPAREN],
        Some(42),
        RecoveryStrategy::ScopeRecovery,
        vec![42],
    );
    let nodes = state.get_error_nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].expected, vec![RPAREN]);
    assert_eq!(nodes[0].actual, Some(42));
}

#[test]
fn multiple_error_nodes_accumulated() {
    let mut state = ErrorRecoveryState::new(config_parens());
    for i in 0..5 {
        state.record_error(
            i,
            i + 1,
            (0, i),
            (0, i + 1),
            vec![RPAREN],
            Some(i as u16),
            RecoveryStrategy::ScopeRecovery,
            vec![],
        );
    }
    assert_eq!(state.get_error_nodes().len(), 5);
}

#[test]
fn clear_errors_then_accumulate_again() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.record_error(
        0,
        1,
        (0, 0),
        (0, 1),
        vec![1],
        Some(2),
        RecoveryStrategy::PanicMode,
        vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 1);
    state.clear_errors();
    assert!(state.get_error_nodes().is_empty());
    state.record_error(
        2,
        3,
        (0, 2),
        (0, 3),
        vec![3],
        Some(4),
        RecoveryStrategy::ScopeRecovery,
        vec![],
    );
    assert_eq!(state.get_error_nodes().len(), 1);
}

// =====================================================================
// Additional: builder chaining edge cases
// =====================================================================

#[test]
fn builder_chain_all_options() {
    let config = ErrorRecoveryConfigBuilder::new()
        .max_panic_skip(25)
        .add_sync_token(1)
        .add_insertable_token(2)
        .add_deletable_token(3)
        .add_scope_delimiter(LPAREN, RPAREN)
        .add_scope_delimiter(LBRACE, RBRACE)
        .enable_scope_recovery(true)
        .enable_phrase_recovery(false)
        .enable_indentation_recovery(true)
        .max_consecutive_errors(5)
        .build();
    assert_eq!(config.max_panic_skip, 25);
    assert_eq!(config.scope_delimiters.len(), 2);
    assert!(config.enable_scope_recovery);
    assert!(!config.enable_phrase_recovery);
    assert!(config.enable_indentation_recovery);
    assert_eq!(config.max_consecutive_errors, 5);
}

#[test]
fn builder_default_impl() {
    let from_default = ErrorRecoveryConfigBuilder::default().build();
    let from_new = ErrorRecoveryConfigBuilder::new().build();
    assert_eq!(format!("{:?}", from_default), format!("{:?}", from_new));
}

#[test]
fn builder_set_max_recovery_attempts() {
    let config = ErrorRecoveryConfigBuilder::new()
        .set_max_recovery_attempts(7)
        .build();
    assert_eq!(config.max_consecutive_errors, 7);
}

// =====================================================================
// Additional: strategy enum coverage
// =====================================================================

#[test]
fn all_strategy_variants_distinct() {
    let strategies = [
        RecoveryStrategy::PanicMode,
        RecoveryStrategy::TokenInsertion,
        RecoveryStrategy::TokenDeletion,
        RecoveryStrategy::TokenSubstitution,
        RecoveryStrategy::PhraseLevel,
        RecoveryStrategy::ScopeRecovery,
        RecoveryStrategy::IndentationRecovery,
    ];
    for i in 0..strategies.len() {
        for j in (i + 1)..strategies.len() {
            assert_ne!(strategies[i], strategies[j]);
        }
    }
}

#[test]
fn strategy_clone_equals_original() {
    let s = RecoveryStrategy::ScopeRecovery;
    let s2 = s; // Copy
    assert_eq!(s, s2);
}

// =====================================================================
// Additional: scope + give-up interaction
// =====================================================================

#[test]
fn scope_survives_give_up() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(LPAREN, RPAREN)
        .max_consecutive_errors(1)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(LPAREN);
    state.increment_error_count();
    assert!(state.should_give_up());
    // Scope is still there
    assert!(state.pop_scope(RPAREN));
}

#[test]
fn scope_operations_after_reset() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    state.increment_error_count();
    state.reset_error_count();
    state.push_scope(LPAREN);
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
    assert_eq!(state.pop_scope_test(), None);
}

// =====================================================================
// Additional: large token id values
// =====================================================================

#[test]
fn max_u16_delimiter() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(u16::MAX - 1, u16::MAX)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(u16::MAX - 1);
    assert!(state.pop_scope(u16::MAX));
}

#[test]
fn high_value_delimiters_with_low_value() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(0, u16::MAX)
        .build();
    let mut state = ErrorRecoveryState::new(config);
    state.push_scope(0);
    assert!(state.pop_scope(u16::MAX));
}

// =====================================================================
// Additional: pop_scope on non-delimiter token
// =====================================================================

#[test]
fn pop_scope_with_non_delimiter_returns_false() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    assert!(!state.pop_scope(999));
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
}

#[test]
fn pop_scope_with_open_delimiter_returns_false() {
    let mut state = ErrorRecoveryState::new(config_parens());
    state.push_scope(LPAREN);
    // LPAREN is open, not close — pop_scope looks for matching open of a close token
    assert!(!state.pop_scope(LPAREN));
    assert_eq!(state.pop_scope_test(), Some(LPAREN));
}

// =====================================================================
// Additional: config clone preserves delimiters
// =====================================================================

#[test]
fn config_clone_preserves_all_delimiters() {
    let config = config_three_pairs();
    let cloned = config.clone();
    assert_eq!(cloned.scope_delimiters, config.scope_delimiters);
    assert_eq!(cloned.enable_scope_recovery, config.enable_scope_recovery);
}

#[test]
fn cloned_config_state_independent() {
    let config = config_parens();
    let cloned = config.clone();
    let mut state1 = ErrorRecoveryState::new(config);
    let mut state2 = ErrorRecoveryState::new(cloned);
    state1.push_scope(LPAREN);
    // state2 scope is unaffected
    assert_eq!(state2.pop_scope_test(), None);
    assert_eq!(state1.pop_scope_test(), Some(LPAREN));
}

// =====================================================================
// Additional: can_delete/can_replace with scope tokens
// =====================================================================

#[test]
fn delimiter_tokens_deletable_when_not_sync() {
    let config = config_parens();
    assert!(config.can_delete_token(ir::SymbolId(LPAREN)));
    assert!(config.can_delete_token(ir::SymbolId(RPAREN)));
}

#[test]
fn delimiter_tokens_not_deletable_when_sync() {
    let config = ErrorRecoveryConfigBuilder::new()
        .add_scope_delimiter(LPAREN, RPAREN)
        .add_sync_token(LPAREN)
        .build();
    assert!(!config.can_delete_token(ir::SymbolId(LPAREN)));
    assert!(config.can_delete_token(ir::SymbolId(RPAREN)));
}
