//! Property-based tests for bdd-grammar-fixtures.

use proptest::prelude::*;

use adze_bdd_grammar_fixtures::{SymbolMetadataSpec, TokenPatternKind, TokenPatternSpec};
use adze_ir::SymbolId;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate arbitrary TokenPatternKind values.
fn arb_token_pattern_kind() -> impl Strategy<Value = TokenPatternKind> {
    prop_oneof![
        Just(TokenPatternKind::Regex("[a-z]+")),
        Just(TokenPatternKind::Regex("[0-9]+")),
        Just(TokenPatternKind::Literal("if")),
        Just(TokenPatternKind::Literal("else")),
        Just(TokenPatternKind::Literal("then")),
    ]
}

/// Generate arbitrary SymbolId values.
fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0u32..100).prop_map(SymbolId)
}

/// Generate arbitrary TokenPatternSpec values.
fn arb_token_pattern_spec() -> impl Strategy<Value = TokenPatternSpec> {
    (arb_symbol_id(), arb_token_pattern_kind(), any::<bool>()).prop_map(
        |(symbol_id, matcher, is_keyword)| TokenPatternSpec {
            symbol_id,
            matcher,
            is_keyword,
        },
    )
}

/// Generate arbitrary SymbolMetadataSpec values.
fn arb_symbol_metadata_spec() -> impl Strategy<Value = SymbolMetadataSpec> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(is_terminal, is_visible, is_supertype)| SymbolMetadataSpec {
            is_terminal,
            is_visible,
            is_supertype,
        },
    )
}

// ---------------------------------------------------------------------------
// 1 – TokenPatternKind tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn token_pattern_kind_copy_preserves_value(kind in arb_token_pattern_kind()) {
        let kind2 = kind;
        prop_assert_eq!(kind, kind2);
    }

    #[test]
    fn token_pattern_kind_eq_reflexive(kind in arb_token_pattern_kind()) {
        prop_assert_eq!(kind, kind);
    }

    #[test]
    fn token_pattern_kind_debug_non_empty(kind in arb_token_pattern_kind()) {
        let debug = format!("{:?}", kind);
        prop_assert!(!debug.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 2 – TokenPatternSpec tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn token_pattern_spec_copy_preserves_value(spec in arb_token_pattern_spec()) {
        let spec2 = spec;
        prop_assert_eq!(spec, spec2);
    }

    #[test]
    fn token_pattern_spec_eq_reflexive(spec in arb_token_pattern_spec()) {
        prop_assert_eq!(spec, spec);
    }

    #[test]
    fn token_pattern_spec_debug_non_empty(spec in arb_token_pattern_spec()) {
        let debug = format!("{:?}", spec);
        prop_assert!(!debug.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 3 – SymbolMetadataSpec tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_metadata_spec_copy_preserves_value(spec in arb_symbol_metadata_spec()) {
        let spec2 = spec;
        prop_assert_eq!(spec, spec2);
    }

    #[test]
    fn symbol_metadata_spec_eq_reflexive(spec in arb_symbol_metadata_spec()) {
        prop_assert_eq!(spec, spec);
    }

    #[test]
    fn symbol_metadata_spec_debug_non_empty(spec in arb_symbol_metadata_spec()) {
        let debug = format!("{:?}", spec);
        prop_assert!(!debug.is_empty());
    }

    #[test]
    fn symbol_metadata_spec_fields_consistent(
        is_terminal in any::<bool>(),
        is_visible in any::<bool>(),
        is_supertype in any::<bool>()
    ) {
        let spec = SymbolMetadataSpec { is_terminal, is_visible, is_supertype };
        prop_assert_eq!(spec.is_terminal, is_terminal);
        prop_assert_eq!(spec.is_visible, is_visible);
        prop_assert_eq!(spec.is_supertype, is_supertype);
    }
}

// ---------------------------------------------------------------------------
// 4 – Fixture constant tests
// ---------------------------------------------------------------------------

#[test]
fn dangling_else_symbol_metadata_not_empty() {
    assert!(!adze_bdd_grammar_fixtures::DANGLING_ELSE_SYMBOL_METADATA.is_empty());
}

#[test]
fn dangling_else_token_patterns_not_empty() {
    assert!(!adze_bdd_grammar_fixtures::DANGLING_ELSE_TOKEN_PATTERNS.is_empty());
}
