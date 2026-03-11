use adze_bdd_grammar_fixtures::{
    DANGLING_ELSE_SYMBOL_METADATA, DANGLING_ELSE_TOKEN_PATTERNS,
    PRECEDENCE_ARITHMETIC_SYMBOL_METADATA, PRECEDENCE_ARITHMETIC_TOKEN_PATTERNS,
};

#[test]
fn runtime_metadata_constants_match_core_source() {
    assert_eq!(
        DANGLING_ELSE_SYMBOL_METADATA,
        adze_bdd_runtime_fixtures_core::DANGLING_ELSE_SYMBOL_METADATA
    );
    assert_eq!(
        DANGLING_ELSE_TOKEN_PATTERNS,
        adze_bdd_runtime_fixtures_core::DANGLING_ELSE_TOKEN_PATTERNS
    );
    assert_eq!(
        PRECEDENCE_ARITHMETIC_SYMBOL_METADATA,
        adze_bdd_runtime_fixtures_core::PRECEDENCE_ARITHMETIC_SYMBOL_METADATA
    );
    assert_eq!(
        PRECEDENCE_ARITHMETIC_TOKEN_PATTERNS,
        adze_bdd_runtime_fixtures_core::PRECEDENCE_ARITHMETIC_TOKEN_PATTERNS
    );
}
