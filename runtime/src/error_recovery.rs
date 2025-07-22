// Comprehensive error recovery strategies for the pure-Rust Tree-sitter implementation
// This module implements various error recovery techniques to produce useful parse trees
// even when the input contains syntax errors.

use std::collections::{HashSet, VecDeque};

/// Error recovery strategies that can be applied during parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Skip tokens until a synchronization point is found
    PanicMode,
    /// Insert a missing token
    TokenInsertion,
    /// Delete an unexpected token
    TokenDeletion,
    /// Replace an unexpected token with an expected one
    TokenSubstitution,
    /// Use phrase-level recovery to skip to next valid construct
    PhraseLevel,
    /// Use scope-based recovery (e.g., balance brackets)
    ScopeRecovery,
    /// Use indentation-based recovery for languages like Python
    IndentationRecovery,
}

/// Error recovery configuration
#[derive(Debug, Clone)]
pub struct ErrorRecoveryConfig {
    /// Maximum number of tokens to skip during panic mode
    pub max_panic_skip: usize,
    /// Synchronization tokens for panic mode recovery
    pub sync_tokens: HashSet<u16>,
    /// Tokens that can be auto-inserted
    pub insertable_tokens: HashSet<u16>,
    /// Maximum number of consecutive errors before giving up
    pub max_consecutive_errors: usize,
    /// Enable phrase-level recovery
    pub enable_phrase_recovery: bool,
    /// Enable scope-based recovery
    pub enable_scope_recovery: bool,
    /// Scope delimiters (open, close) pairs
    pub scope_delimiters: Vec<(u16, u16)>,
    /// Enable indentation-based recovery
    pub enable_indentation_recovery: bool,
}

impl Default for ErrorRecoveryConfig {
    fn default() -> Self {
        Self {
            max_panic_skip: 50,
            sync_tokens: HashSet::new(),
            insertable_tokens: HashSet::new(),
            max_consecutive_errors: 10,
            enable_phrase_recovery: true,
            enable_scope_recovery: true,
            scope_delimiters: Vec::new(),
            enable_indentation_recovery: false,
        }
    }
}

/// Error recovery state during parsing
pub struct ErrorRecoveryState {
    /// Configuration for error recovery
    config: ErrorRecoveryConfig,
    /// Number of consecutive errors encountered
    consecutive_errors: usize,
    /// Stack of open scopes for scope-based recovery
    scope_stack: Vec<u16>,
    /// Recent tokens for context-aware recovery
    recent_tokens: VecDeque<u16>,
    /// Indentation levels for indentation-based recovery
    indentation_stack: Vec<usize>,
    /// Error nodes created during recovery
    error_nodes: Vec<ErrorNode>,
}

/// Represents an error node in the parse tree
#[derive(Debug, Clone)]
pub struct ErrorNode {
    /// Start byte position of the error
    pub start_byte: usize,
    /// End byte position of the error
    pub end_byte: usize,
    /// Start position (row, column)
    pub start_position: (usize, usize),
    /// End position (row, column)
    pub end_position: (usize, usize),
    /// Expected symbols at this position
    pub expected: Vec<u16>,
    /// Actual symbol encountered
    pub actual: Option<u16>,
    /// Recovery strategy used
    pub recovery: RecoveryStrategy,
    /// Skipped tokens during recovery
    pub skipped_tokens: Vec<u16>,
}

impl ErrorRecoveryState {
    pub fn new(config: ErrorRecoveryConfig) -> Self {
        Self {
            config,
            consecutive_errors: 0,
            scope_stack: Vec::new(),
            recent_tokens: VecDeque::with_capacity(10),
            indentation_stack: vec![0],
            error_nodes: Vec::new(),
        }
    }

    /// Record an error and determine recovery strategy
    pub fn determine_recovery_strategy(
        &mut self,
        expected: &[u16],
        actual: Option<u16>,
        position: (usize, usize),
        byte_offset: usize,
    ) -> RecoveryStrategy {
        self.consecutive_errors += 1;

        // Check if we've hit the error limit
        if self.consecutive_errors > self.config.max_consecutive_errors {
            return RecoveryStrategy::PanicMode;
        }

        // Try strategies in order of preference
        
        // 1. Token insertion - if the missing token is insertable
        if actual.is_none() || self.can_insert_token(expected) {
            if let Some(_token) = self.find_insertable_token(expected) {
                self.consecutive_errors = 0; // Reset on successful recovery
                return RecoveryStrategy::TokenInsertion;
            }
        }

        // 2. Token deletion - if current token is clearly wrong
        if let Some(token) = actual {
            if self.is_clearly_wrong(token, expected) {
                return RecoveryStrategy::TokenDeletion;
            }
        }

        // 3. Token substitution - if there's a clear candidate
        if let Some(token) = actual {
            if self.can_substitute_token(token, expected) {
                return RecoveryStrategy::TokenSubstitution;
            }
        }

        // 4. Scope recovery - if we're in a scope mismatch
        if self.config.enable_scope_recovery && self.has_scope_mismatch(actual) {
            return RecoveryStrategy::ScopeRecovery;
        }

        // 5. Phrase-level recovery - skip to next major construct
        if self.config.enable_phrase_recovery {
            return RecoveryStrategy::PhraseLevel;
        }

        // 6. Default to panic mode
        RecoveryStrategy::PanicMode
    }

    /// Record an error node
    pub fn record_error(
        &mut self,
        start_byte: usize,
        end_byte: usize,
        start_position: (usize, usize),
        end_position: (usize, usize),
        expected: Vec<u16>,
        actual: Option<u16>,
        recovery: RecoveryStrategy,
        skipped_tokens: Vec<u16>,
    ) {
        self.error_nodes.push(ErrorNode {
            start_byte,
            end_byte,
            start_position,
            end_position,
            expected,
            actual,
            recovery,
            skipped_tokens,
        });
    }

    /// Update recent tokens for context-aware recovery
    pub fn add_recent_token(&mut self, token: u16) {
        if self.recent_tokens.len() >= 10 {
            self.recent_tokens.pop_front();
        }
        self.recent_tokens.push_back(token);
    }

    /// Update scope stack for scope-based recovery
    pub fn push_scope(&mut self, token: u16) {
        if self.is_opening_delimiter(token) {
            self.scope_stack.push(token);
        }
    }

    /// Update scope stack when closing delimiter is found
    pub fn pop_scope(&mut self, token: u16) -> bool {
        if let Some(expected_open) = self.find_matching_open(token) {
            if self.scope_stack.last() == Some(&expected_open) {
                self.scope_stack.pop();
                return true;
            }
        }
        false
    }

    /// Get error nodes collected during parsing
    pub fn get_error_nodes(&self) -> &[ErrorNode] {
        &self.error_nodes
    }

    /// Reset consecutive error count (called on successful parse)
    pub fn reset_consecutive_errors(&mut self) {
        self.consecutive_errors = 0;
    }

    /// Clear all error nodes
    pub fn clear_errors(&mut self) {
        self.error_nodes.clear();
    }

    // Helper methods

    fn can_insert_token(&self, expected: &[u16]) -> bool {
        expected.iter().any(|s| self.config.insertable_tokens.contains(s))
    }

    fn find_insertable_token(&self, expected: &[u16]) -> Option<u16> {
        expected.iter()
            .find(|s| self.config.insertable_tokens.contains(s))
            .copied()
    }

    fn is_clearly_wrong(&self, token: u16, expected: &[u16]) -> bool {
        // Token is clearly wrong if it's not in expected set
        // and it's not a sync token
        !expected.contains(&token) && !self.config.sync_tokens.contains(&token)
    }

    fn can_substitute_token(&self, actual: u16, expected: &[u16]) -> bool {
        // In a real implementation, check if tokens are similar
        // For now, just check if there's exactly one expected token
        expected.len() == 1
    }

    fn has_scope_mismatch(&self, actual: Option<u16>) -> bool {
        if let Some(token) = actual {
            // Check if it's a closing delimiter without matching open
            self.config.scope_delimiters.iter().any(|(_, close)| {
                token == *close && !self.scope_stack.iter().any(|open| {
                    self.config.scope_delimiters.iter().any(|(o, c)| o == open && c == close)
                })
            })
        } else {
            false
        }
    }

    fn is_opening_delimiter(&self, token: u16) -> bool {
        self.config.scope_delimiters.iter().any(|(open, _)| *open == token)
    }

    fn find_matching_open(&self, close_token: u16) -> Option<u16> {
        self.config.scope_delimiters.iter()
            .find(|(_, close)| *close == close_token)
            .map(|(open, _)| *open)
    }
}

/// Builder for ErrorRecoveryConfig
pub struct ErrorRecoveryConfigBuilder {
    config: ErrorRecoveryConfig,
}

impl ErrorRecoveryConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: ErrorRecoveryConfig::default(),
        }
    }

    pub fn max_panic_skip(mut self, max: usize) -> Self {
        self.config.max_panic_skip = max;
        self
    }

    pub fn add_sync_token(mut self, token: u16) -> Self {
        self.config.sync_tokens.insert(token);
        self
    }

    pub fn add_insertable_token(mut self, token: u16) -> Self {
        self.config.insertable_tokens.insert(token);
        self
    }

    pub fn add_scope_delimiter(mut self, open: u16, close: u16) -> Self {
        self.config.scope_delimiters.push((open, close));
        self
    }

    pub fn enable_indentation_recovery(mut self, enable: bool) -> Self {
        self.config.enable_indentation_recovery = enable;
        self
    }

    pub fn build(self) -> ErrorRecoveryConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_config() {
        let config = ErrorRecoveryConfig::default();
        assert_eq!(config.max_panic_skip, 50);
        assert_eq!(config.max_consecutive_errors, 10);
        assert!(config.enable_phrase_recovery);
        assert!(config.enable_scope_recovery);
    }

    #[test]
    fn test_recovery_state_creation() {
        let config = ErrorRecoveryConfig::default();
        let state = ErrorRecoveryState::new(config);
        assert_eq!(state.consecutive_errors, 0);
        assert!(state.scope_stack.is_empty());
        assert_eq!(state.indentation_stack, vec![0]);
    }

    #[test]
    fn test_config_builder() {
        let config = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(100)
            .add_sync_token(1)
            .add_sync_token(2)
            .add_insertable_token(3)
            .add_scope_delimiter(4, 5)
            .enable_indentation_recovery(true)
            .build();
        
        assert_eq!(config.max_panic_skip, 100);
        assert!(config.sync_tokens.contains(&1));
        assert!(config.sync_tokens.contains(&2));
        assert!(config.insertable_tokens.contains(&3));
        assert_eq!(config.scope_delimiters, vec![(4, 5)]);
        assert!(config.enable_indentation_recovery);
    }

    #[test]
    fn test_recovery_strategy_selection() {
        let mut config = ErrorRecoveryConfig::default();
        config.insertable_tokens.insert(10);
        config.sync_tokens.insert(20);
        
        let mut state = ErrorRecoveryState::new(config);
        
        // Test token insertion strategy
        let strategy = state.determine_recovery_strategy(&[10, 11], None, (0, 0), 0);
        assert_eq!(strategy, RecoveryStrategy::TokenInsertion);
        
        // Test panic mode after too many errors
        state.consecutive_errors = 11;
        let strategy = state.determine_recovery_strategy(&[10, 11], Some(15), (0, 0), 0);
        assert_eq!(strategy, RecoveryStrategy::PanicMode);
    }

    #[test]
    fn test_scope_tracking() {
        let mut config = ErrorRecoveryConfig::default();
        config.scope_delimiters.push((1, 2)); // ( and )
        config.scope_delimiters.push((3, 4)); // { and }
        
        let mut state = ErrorRecoveryState::new(config);
        
        // Push opening delimiters
        state.push_scope(1);
        state.push_scope(3);
        assert_eq!(state.scope_stack, vec![1, 3]);
        
        // Pop matching delimiter
        assert!(state.pop_scope(4));
        assert_eq!(state.scope_stack, vec![1]);
        
        // Try to pop non-matching delimiter
        assert!(!state.pop_scope(4));
        assert_eq!(state.scope_stack, vec![1]);
        
        // Pop correct delimiter
        assert!(state.pop_scope(2));
        assert!(state.scope_stack.is_empty());
    }

    #[test]
    fn test_error_recording() {
        let config = ErrorRecoveryConfig::default();
        let mut state = ErrorRecoveryState::new(config);
        
        state.record_error(
            0, 5,
            (0, 0), (0, 5),
            vec![1, 2, 3],
            Some(4),
            RecoveryStrategy::TokenDeletion,
            vec![4],
        );
        
        let errors = state.get_error_nodes();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].start_byte, 0);
        assert_eq!(errors[0].end_byte, 5);
        assert_eq!(errors[0].expected, vec![1, 2, 3]);
        assert_eq!(errors[0].actual, Some(4));
        assert_eq!(errors[0].recovery, RecoveryStrategy::TokenDeletion);
    }
}