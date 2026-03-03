#![allow(clippy::needless_range_loop)]
//! Property-based tests for `ErrorRecoveryConfigBuilder` in the adze runtime.

use adze::error_recovery::{ErrorRecoveryConfig, ErrorRecoveryConfigBuilder};
use adze_ir::SymbolId;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Assert two configs have identical field values.
fn configs_equal(a: &ErrorRecoveryConfig, b: &ErrorRecoveryConfig) -> bool {
    a.max_panic_skip == b.max_panic_skip
        && a.max_token_deletions == b.max_token_deletions
        && a.max_token_insertions == b.max_token_insertions
        && a.max_consecutive_errors == b.max_consecutive_errors
        && a.enable_phrase_recovery == b.enable_phrase_recovery
        && a.enable_scope_recovery == b.enable_scope_recovery
        && a.enable_indentation_recovery == b.enable_indentation_recovery
        && a.sync_tokens == b.sync_tokens
        && a.insert_candidates == b.insert_candidates
        && a.deletable_tokens == b.deletable_tokens
        && a.scope_delimiters == b.scope_delimiters
}

// ---------------------------------------------------------------------------
// Tests: Default values
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 1. Builder default produces the same config as `ErrorRecoveryConfig::default()`.
    #[test]
    fn builder_default_matches_config_default(_seed in 0u32..1000) {
        let from_builder = ErrorRecoveryConfigBuilder::new().build();
        let from_default = ErrorRecoveryConfig::default();
        prop_assert!(configs_equal(&from_builder, &from_default));
    }

    /// 2. Default builder via `Default` trait matches `new()`.
    #[test]
    fn builder_default_trait_matches_new(_seed in 0u32..1000) {
        let via_new = ErrorRecoveryConfigBuilder::new().build();
        let via_default = ErrorRecoveryConfigBuilder::default().build();
        prop_assert!(configs_equal(&via_new, &via_default));
    }

    /// 3. Default max_panic_skip is 50.
    #[test]
    fn default_max_panic_skip(_seed in 0u32..100) {
        let cfg = ErrorRecoveryConfigBuilder::new().build();
        prop_assert_eq!(cfg.max_panic_skip, 50);
    }

    /// 4. Default max_consecutive_errors is 10.
    #[test]
    fn default_max_consecutive_errors(_seed in 0u32..100) {
        let cfg = ErrorRecoveryConfigBuilder::new().build();
        prop_assert_eq!(cfg.max_consecutive_errors, 10);
    }

    /// 5. Default enables phrase recovery.
    #[test]
    fn default_phrase_recovery_enabled(_seed in 0u32..100) {
        let cfg = ErrorRecoveryConfigBuilder::new().build();
        prop_assert!(cfg.enable_phrase_recovery);
    }

    /// 6. Default enables scope recovery.
    #[test]
    fn default_scope_recovery_enabled(_seed in 0u32..100) {
        let cfg = ErrorRecoveryConfigBuilder::new().build();
        prop_assert!(cfg.enable_scope_recovery);
    }

    /// 7. Default disables indentation recovery.
    #[test]
    fn default_indentation_recovery_disabled(_seed in 0u32..100) {
        let cfg = ErrorRecoveryConfigBuilder::new().build();
        prop_assert!(!cfg.enable_indentation_recovery);
    }
}

// ---------------------------------------------------------------------------
// Tests: Custom max_errors / max_panic_skip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// 8. Custom max_panic_skip is preserved.
    #[test]
    fn custom_max_panic_skip(val in 0usize..10_000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(val)
            .build();
        prop_assert_eq!(cfg.max_panic_skip, val);
    }

    /// 9. Custom max_consecutive_errors is preserved.
    #[test]
    fn custom_max_consecutive_errors(val in 0usize..10_000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(val)
            .build();
        prop_assert_eq!(cfg.max_consecutive_errors, val);
    }

    /// 10. set_max_recovery_attempts sets max_consecutive_errors.
    #[test]
    fn set_max_recovery_attempts_alias(val in 0usize..10_000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .set_max_recovery_attempts(val)
            .build();
        prop_assert_eq!(cfg.max_consecutive_errors, val);
    }

    /// 11. Last write wins for max_panic_skip.
    #[test]
    fn max_panic_skip_last_write_wins(a in 0usize..5000, b in 0usize..5000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(a)
            .max_panic_skip(b)
            .build();
        prop_assert_eq!(cfg.max_panic_skip, b);
    }

    /// 12. Last write wins for max_consecutive_errors.
    #[test]
    fn max_consecutive_errors_last_write_wins(a in 0usize..5000, b in 0usize..5000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_consecutive_errors(a)
            .max_consecutive_errors(b)
            .build();
        prop_assert_eq!(cfg.max_consecutive_errors, b);
    }
}

// ---------------------------------------------------------------------------
// Tests: Custom strategies (toggle booleans)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 13. enable_phrase_recovery respects arbitrary bool.
    #[test]
    fn phrase_recovery_toggle(flag in proptest::bool::ANY) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .enable_phrase_recovery(flag)
            .build();
        prop_assert_eq!(cfg.enable_phrase_recovery, flag);
    }

    /// 14. enable_scope_recovery respects arbitrary bool.
    #[test]
    fn scope_recovery_toggle(flag in proptest::bool::ANY) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .enable_scope_recovery(flag)
            .build();
        prop_assert_eq!(cfg.enable_scope_recovery, flag);
    }

    /// 15. enable_indentation_recovery respects arbitrary bool.
    #[test]
    fn indentation_recovery_toggle(flag in proptest::bool::ANY) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .enable_indentation_recovery(flag)
            .build();
        prop_assert_eq!(cfg.enable_indentation_recovery, flag);
    }

    /// 16. Disabling all strategies leaves them all off.
    #[test]
    fn all_strategies_disabled(_seed in 0u32..100) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .enable_phrase_recovery(false)
            .enable_scope_recovery(false)
            .enable_indentation_recovery(false)
            .build();
        prop_assert!(!cfg.enable_phrase_recovery);
        prop_assert!(!cfg.enable_scope_recovery);
        prop_assert!(!cfg.enable_indentation_recovery);
    }

    /// 17. Enabling all strategies leaves them all on.
    #[test]
    fn all_strategies_enabled(_seed in 0u32..100) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .enable_phrase_recovery(true)
            .enable_scope_recovery(true)
            .enable_indentation_recovery(true)
            .build();
        prop_assert!(cfg.enable_phrase_recovery);
        prop_assert!(cfg.enable_scope_recovery);
        prop_assert!(cfg.enable_indentation_recovery);
    }
}

// ---------------------------------------------------------------------------
// Tests: Chain methods (add_sync_token, add_insertable_token, etc.)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 18. Adding sync tokens accumulates them.
    #[test]
    fn add_sync_tokens_accumulate(tokens in prop::collection::vec(0u16..1000, 1..20)) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for &t in &tokens {
            builder = builder.add_sync_token(t);
        }
        let cfg = builder.build();
        prop_assert_eq!(cfg.sync_tokens.len(), tokens.len());
        for i in 0..tokens.len() {
            prop_assert_eq!(cfg.sync_tokens[i], SymbolId(tokens[i]));
        }
    }

    /// 19. add_sync_token_sym works equivalently to add_sync_token.
    #[test]
    fn sync_token_sym_equiv(token in 0u16..1000) {
        let a = ErrorRecoveryConfigBuilder::new()
            .add_sync_token(token)
            .build();
        let b = ErrorRecoveryConfigBuilder::new()
            .add_sync_token_sym(SymbolId(token))
            .build();
        prop_assert_eq!(a.sync_tokens.len(), b.sync_tokens.len());
        prop_assert_eq!(a.sync_tokens[0], b.sync_tokens[0]);
    }

    /// 20. Adding insertable tokens accumulates them.
    #[test]
    fn add_insertable_tokens_accumulate(tokens in prop::collection::vec(0u16..1000, 1..20)) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for &t in &tokens {
            builder = builder.add_insertable_token(t);
        }
        let cfg = builder.build();
        prop_assert_eq!(cfg.insert_candidates.len(), tokens.len());
        for i in 0..tokens.len() {
            prop_assert_eq!(cfg.insert_candidates[i], SymbolId(tokens[i]));
        }
    }

    /// 21. add_insertable_token_sym works equivalently to add_insertable_token.
    #[test]
    fn insertable_token_sym_equiv(token in 0u16..1000) {
        let a = ErrorRecoveryConfigBuilder::new()
            .add_insertable_token(token)
            .build();
        let b = ErrorRecoveryConfigBuilder::new()
            .add_insertable_token_sym(SymbolId(token))
            .build();
        prop_assert_eq!(a.insert_candidates.len(), b.insert_candidates.len());
        prop_assert_eq!(a.insert_candidates[0], b.insert_candidates[0]);
    }

    /// 22. Adding deletable tokens accumulates in HashSet (deduplicates).
    #[test]
    fn add_deletable_tokens_dedup(tokens in prop::collection::vec(0u16..50, 1..30)) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for &t in &tokens {
            builder = builder.add_deletable_token(t);
        }
        let cfg = builder.build();
        let unique: std::collections::HashSet<u16> = tokens.into_iter().collect();
        prop_assert_eq!(cfg.deletable_tokens.len(), unique.len());
        for t in &unique {
            prop_assert!(cfg.deletable_tokens.contains(t));
        }
    }

    /// 23. Adding scope delimiters accumulates them in order.
    #[test]
    fn add_scope_delimiters_accumulate(
        pairs in prop::collection::vec((0u16..500, 0u16..500), 1..10)
    ) {
        let mut builder = ErrorRecoveryConfigBuilder::new();
        for &(open, close) in &pairs {
            builder = builder.add_scope_delimiter(open, close);
        }
        let cfg = builder.build();
        prop_assert_eq!(cfg.scope_delimiters.len(), pairs.len());
        for i in 0..pairs.len() {
            prop_assert_eq!(cfg.scope_delimiters[i], pairs[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: build() produces valid config
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 24. Fully configured builder produces config with all fields set.
    #[test]
    fn build_full_config(
        max_panic in 0usize..500,
        max_errors in 0usize..500,
        phrase in proptest::bool::ANY,
        scope in proptest::bool::ANY,
        indent in proptest::bool::ANY,
        sync_tok in 0u16..1000,
        insert_tok in 0u16..1000,
        delete_tok in 0u16..1000,
        delim_open in 0u16..500,
        delim_close in 0u16..500,
    ) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(max_panic)
            .max_consecutive_errors(max_errors)
            .enable_phrase_recovery(phrase)
            .enable_scope_recovery(scope)
            .enable_indentation_recovery(indent)
            .add_sync_token(sync_tok)
            .add_insertable_token(insert_tok)
            .add_deletable_token(delete_tok)
            .add_scope_delimiter(delim_open, delim_close)
            .build();

        prop_assert_eq!(cfg.max_panic_skip, max_panic);
        prop_assert_eq!(cfg.max_consecutive_errors, max_errors);
        prop_assert_eq!(cfg.enable_phrase_recovery, phrase);
        prop_assert_eq!(cfg.enable_scope_recovery, scope);
        prop_assert_eq!(cfg.enable_indentation_recovery, indent);
        prop_assert_eq!(cfg.sync_tokens.len(), 1);
        prop_assert_eq!(cfg.sync_tokens[0], SymbolId(sync_tok));
        prop_assert_eq!(cfg.insert_candidates.len(), 1);
        prop_assert_eq!(cfg.insert_candidates[0], SymbolId(insert_tok));
        prop_assert!(cfg.deletable_tokens.contains(&delete_tok));
        prop_assert_eq!(cfg.scope_delimiters, vec![(delim_open, delim_close)]);
    }

    /// 25. Builder does not mutate unrelated defaults when only one field is set.
    #[test]
    fn build_preserves_other_defaults(val in 0usize..10_000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(val)
            .build();
        let defaults = ErrorRecoveryConfig::default();
        prop_assert_eq!(cfg.max_panic_skip, val);
        // Everything else should remain at default.
        prop_assert_eq!(cfg.max_consecutive_errors, defaults.max_consecutive_errors);
        prop_assert_eq!(cfg.max_token_deletions, defaults.max_token_deletions);
        prop_assert_eq!(cfg.max_token_insertions, defaults.max_token_insertions);
        prop_assert_eq!(cfg.enable_phrase_recovery, defaults.enable_phrase_recovery);
        prop_assert_eq!(cfg.enable_scope_recovery, defaults.enable_scope_recovery);
        prop_assert_eq!(cfg.enable_indentation_recovery, defaults.enable_indentation_recovery);
        prop_assert!(cfg.sync_tokens.is_empty());
        prop_assert!(cfg.insert_candidates.is_empty());
        prop_assert!(cfg.deletable_tokens.is_empty());
        prop_assert!(cfg.scope_delimiters.is_empty());
    }
}

// ---------------------------------------------------------------------------
// Tests: Config equality (field-by-field since no PartialEq derive)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 26. Two builders with identical parameters produce equal configs.
    #[test]
    fn identical_builders_produce_equal_configs(
        max_panic in 0usize..1000,
        max_errors in 0usize..1000,
        phrase in proptest::bool::ANY,
    ) {
        let build = || {
            ErrorRecoveryConfigBuilder::new()
                .max_panic_skip(max_panic)
                .max_consecutive_errors(max_errors)
                .enable_phrase_recovery(phrase)
                .build()
        };
        let a = build();
        let b = build();
        prop_assert!(configs_equal(&a, &b));
    }

    /// 27. Configs with different max_panic_skip are not equal.
    #[test]
    fn different_max_panic_skip_not_equal(
        a_val in 0usize..5000,
        b_val in 0usize..5000,
    ) {
        prop_assume!(a_val != b_val);
        let a = ErrorRecoveryConfigBuilder::new().max_panic_skip(a_val).build();
        let b = ErrorRecoveryConfigBuilder::new().max_panic_skip(b_val).build();
        prop_assert!(!configs_equal(&a, &b));
    }

    /// 28. Configs with different sync_tokens are not equal.
    #[test]
    fn different_sync_tokens_not_equal(
        tok_a in 0u16..500,
        tok_b in 500u16..1000,
    ) {
        let a = ErrorRecoveryConfigBuilder::new().add_sync_token(tok_a).build();
        let b = ErrorRecoveryConfigBuilder::new().add_sync_token(tok_b).build();
        prop_assert!(!configs_equal(&a, &b));
    }
}

// ---------------------------------------------------------------------------
// Tests: Config clone
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 29. Clone of default config equals original.
    #[test]
    fn clone_default_config(_seed in 0u32..100) {
        let cfg = ErrorRecoveryConfig::default();
        let cloned = cfg.clone();
        prop_assert!(configs_equal(&cfg, &cloned));
    }

    /// 30. Clone of custom config equals original.
    #[test]
    fn clone_custom_config(
        max_panic in 0usize..1000,
        sync_tok in 0u16..1000,
        delim in (0u16..500, 0u16..500),
    ) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(max_panic)
            .add_sync_token(sync_tok)
            .add_scope_delimiter(delim.0, delim.1)
            .build();
        let cloned = cfg.clone();
        prop_assert!(configs_equal(&cfg, &cloned));
    }

    /// 31. Mutating clone does not affect original (clone independence).
    #[test]
    fn clone_independence(max_panic in 0usize..1000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(max_panic)
            .build();
        let mut cloned = cfg.clone();
        cloned.max_panic_skip = max_panic.wrapping_add(1);
        prop_assert_eq!(cfg.max_panic_skip, max_panic);
        prop_assert_ne!(cfg.max_panic_skip, cloned.max_panic_skip);
    }
}

// ---------------------------------------------------------------------------
// Tests: Config debug
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// 32. Debug output is non-empty for default config.
    #[test]
    fn debug_default_non_empty(_seed in 0u32..100) {
        let cfg = ErrorRecoveryConfig::default();
        let dbg = format!("{:?}", cfg);
        prop_assert!(!dbg.is_empty());
    }

    /// 33. Debug output contains the struct name.
    #[test]
    fn debug_contains_struct_name(_seed in 0u32..100) {
        let cfg = ErrorRecoveryConfig::default();
        let dbg = format!("{:?}", cfg);
        prop_assert!(dbg.contains("ErrorRecoveryConfig"));
    }

    /// 34. Debug output reflects max_panic_skip value.
    #[test]
    fn debug_reflects_max_panic_skip(val in 0usize..10_000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(val)
            .build();
        let dbg = format!("{:?}", cfg);
        prop_assert!(dbg.contains(&val.to_string()));
    }

    /// 35. Debug of cloned config equals debug of original.
    #[test]
    fn debug_clone_identical(val in 0usize..10_000) {
        let cfg = ErrorRecoveryConfigBuilder::new()
            .max_panic_skip(val)
            .build();
        let cloned = cfg.clone();
        prop_assert_eq!(format!("{:?}", cfg), format!("{:?}", cloned));
    }
}
