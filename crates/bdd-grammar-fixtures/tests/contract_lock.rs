//! Contract lock tests - verify API stability
//! These tests ensure the public API remains stable.

#[cfg(test)]
mod contract_lock {
    use adze_bdd_grammar_fixtures::*;

    #[test]
    fn contract_lock_types() {
        // Verify token pattern types exist
        let _pattern_kind = TokenPatternKind::Regex("\\d+");
        let _pattern_kind = TokenPatternKind::Literal("if");

        // Verify spec types exist (using SymbolId constructor from lib.rs)
        let _token_spec = TokenPatternSpec {
            symbol_id: adze_ir::SymbolId(0),
            matcher: TokenPatternKind::Literal("test"),
            is_keyword: false,
        };

        let _symbol_spec = SymbolMetadataSpec {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        };

        // Verify conflict analysis type from re-export is accessible
        let _: ConflictAnalysis;
    }

    #[test]
    fn contract_lock_functions() {
        // Verify grammar builder functions exist and are callable
        let _grammar = dangling_else_grammar();
        let _grammar = precedence_arithmetic_grammar(adze_ir::Associativity::Left);

        // Verify re-exported analysis functions are accessible
        // (They require ParseTable which is complex to construct, so just verify they exist)
        let _ = analyze_conflicts as fn(&adze_glr_core::ParseTable) -> ConflictAnalysis;
        let _ = count_multi_action_cells as fn(&adze_glr_core::ParseTable) -> usize;
    }

    #[test]
    fn contract_lock_constants() {
        // Verify fixture constants exist and are accessible
        assert!(!DANGLING_ELSE_SYMBOL_METADATA.is_empty());
        assert!(!DANGLING_ELSE_TOKEN_PATTERNS.is_empty());
        assert!(!PRECEDENCE_ARITHMETIC_SYMBOL_METADATA.is_empty());
        assert!(!PRECEDENCE_ARITHMETIC_TOKEN_PATTERNS.is_empty());
    }
}
