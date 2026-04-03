//! BDD-style tests for bdd-grammar-fixtures crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.

use adze_bdd_grammar_fixtures::*;
use adze_ir::{Associativity, SymbolId};

// ---------------------------------------------------------------------------
// TokenPatternKind tests
// ---------------------------------------------------------------------------

#[test]
fn given_literal_pattern_when_creating_token_pattern_kind_then_equals_works() {
    // Given
    let pattern = TokenPatternKind::Literal("if");

    // When
    let copied = pattern;

    // Then
    assert_eq!(pattern, copied);
}

#[test]
fn given_regex_pattern_when_creating_token_pattern_kind_then_debug_contains_regex() {
    // Given
    let pattern = TokenPatternKind::Regex(r"\d+");

    // When
    let debug_str = format!("{:?}", pattern);

    // Then
    assert!(debug_str.contains("Regex"));
}

#[test]
fn given_two_different_patterns_when_comparing_then_not_equal() {
    // Given
    let literal = TokenPatternKind::Literal("x");
    let regex = TokenPatternKind::Regex(r"\d+");

    // When / Then
    assert_ne!(literal, regex);
}

// ---------------------------------------------------------------------------
// TokenPatternSpec tests
// ---------------------------------------------------------------------------

#[test]
fn given_token_pattern_spec_when_accessing_fields_then_returns_correct_values() {
    // Given
    let spec = TokenPatternSpec {
        symbol_id: SymbolId(1),
        matcher: TokenPatternKind::Literal("if"),
        is_keyword: true,
    };

    // When / Then
    assert_eq!(spec.symbol_id, SymbolId(1));
    assert!(spec.is_keyword);
}

#[test]
fn given_token_pattern_spec_when_debug_then_contains_is_keyword() {
    // Given
    let spec = TokenPatternSpec {
        symbol_id: SymbolId(1),
        matcher: TokenPatternKind::Literal("if"),
        is_keyword: true,
    };

    // When
    let debug_str = format!("{:?}", spec);

    // Then
    assert!(debug_str.contains("is_keyword"));
}

// ---------------------------------------------------------------------------
// SymbolMetadataSpec tests
// ---------------------------------------------------------------------------

#[test]
fn given_symbol_metadata_spec_when_accessing_fields_then_returns_correct_values() {
    // Given
    let spec = SymbolMetadataSpec {
        is_terminal: true,
        is_visible: false,
        is_supertype: false,
    };

    // When / Then
    assert!(spec.is_terminal);
    assert!(!spec.is_visible);
    assert!(!spec.is_supertype);
}

#[test]
fn given_symbol_metadata_spec_when_debug_then_contains_is_terminal() {
    // Given
    let spec = SymbolMetadataSpec {
        is_terminal: true,
        is_visible: false,
        is_supertype: false,
    };

    // When
    let debug_str = format!("{:?}", spec);

    // Then
    assert!(debug_str.contains("is_terminal"));
}

// ---------------------------------------------------------------------------
// Fixture constants tests
// ---------------------------------------------------------------------------

#[test]
fn given_dangling_else_symbol_metadata_when_checking_is_empty_then_returns_false() {
    // Given / When
    let metadata = DANGLING_ELSE_SYMBOL_METADATA;

    // Then
    assert!(!metadata.is_empty());
}

#[test]
fn given_dangling_else_token_patterns_when_checking_is_empty_then_returns_false() {
    // Given / When
    let patterns = DANGLING_ELSE_TOKEN_PATTERNS;

    // Then
    assert!(!patterns.is_empty());
}

#[test]
fn given_precedence_arithmetic_symbol_metadata_when_checking_is_empty_then_returns_false() {
    // Given / When
    let metadata = PRECEDENCE_ARITHMETIC_SYMBOL_METADATA;

    // Then
    assert!(!metadata.is_empty());
}

#[test]
fn given_precedence_arithmetic_token_patterns_when_checking_is_empty_then_returns_false() {
    // Given / When
    let patterns = PRECEDENCE_ARITHMETIC_TOKEN_PATTERNS;

    // Then
    assert!(!patterns.is_empty());
}

// ---------------------------------------------------------------------------
// dangling_else_grammar tests
// ---------------------------------------------------------------------------

#[test]
fn given_dangling_else_grammar_when_building_then_has_correct_name() {
    // Given / When
    let grammar = dangling_else_grammar();

    // Then
    assert_eq!(grammar.name, "if_then_else");
}

#[test]
fn given_dangling_else_grammar_when_building_then_has_tokens() {
    // Given / When
    let grammar = dangling_else_grammar();

    // Then
    assert!(!grammar.tokens.is_empty());
}

#[test]
fn given_dangling_else_grammar_when_building_then_has_rules() {
    // Given / When
    let grammar = dangling_else_grammar();

    // Then
    assert!(!grammar.rules.is_empty());
}

#[test]
fn given_dangling_else_grammar_when_getting_start_symbol_then_returns_some() {
    // Given / When
    let grammar = dangling_else_grammar();

    // Then
    assert!(grammar.start_symbol().is_some());
}

// ---------------------------------------------------------------------------
// precedence_arithmetic_grammar tests
// ---------------------------------------------------------------------------

#[test]
fn given_left_associativity_when_building_precedence_grammar_then_succeeds() {
    // Given / When
    let grammar = precedence_arithmetic_grammar(Associativity::Left);

    // Then
    assert_eq!(grammar.name, "precedence_expr");
    assert!(!grammar.rules.is_empty());
}

#[test]
fn given_right_associativity_when_building_precedence_grammar_then_succeeds() {
    // Given / When
    let grammar = precedence_arithmetic_grammar(Associativity::Right);

    // Then
    assert_eq!(grammar.name, "precedence_expr");
    assert!(!grammar.rules.is_empty());
}

#[test]
fn given_none_associativity_when_building_precedence_grammar_then_succeeds() {
    // Given / When
    let grammar = precedence_arithmetic_grammar(Associativity::None);

    // Then
    assert_eq!(grammar.name, "precedence_expr");
    assert!(!grammar.rules.is_empty());
}

#[test]
fn given_precedence_arithmetic_grammar_when_getting_start_symbol_then_returns_some() {
    // Given / When
    let grammar = precedence_arithmetic_grammar(Associativity::Left);

    // Then
    assert!(grammar.start_symbol().is_some());
}

// ---------------------------------------------------------------------------
// no_precedence_grammar tests
// ---------------------------------------------------------------------------

#[test]
fn given_no_precedence_grammar_when_building_then_has_correct_name() {
    // Given / When
    let grammar = no_precedence_grammar();

    // Then
    assert_eq!(grammar.name, "no_precedence_expr");
}

#[test]
fn given_no_precedence_grammar_when_building_then_has_rules() {
    // Given / When
    let grammar = no_precedence_grammar();

    // Then
    assert!(!grammar.rules.is_empty());
}

#[test]
fn given_no_precedence_grammar_when_getting_start_symbol_then_returns_some() {
    // Given / When
    let grammar = no_precedence_grammar();

    // Then
    assert!(grammar.start_symbol().is_some());
}

// ---------------------------------------------------------------------------
// build_lr1_parse_table tests
// ---------------------------------------------------------------------------

#[test]
fn given_dangling_else_grammar_when_building_lr1_table_then_succeeds() {
    // Given
    let grammar = dangling_else_grammar();

    // When
    let result = build_lr1_parse_table(&grammar);

    // Then
    assert!(result.is_ok());
}

#[test]
fn given_precedence_arithmetic_grammar_when_building_lr1_table_then_succeeds() {
    // Given
    let grammar = precedence_arithmetic_grammar(Associativity::Left);

    // When
    let result = build_lr1_parse_table(&grammar);

    // Then
    assert!(result.is_ok());
}

#[test]
fn given_no_precedence_grammar_when_building_lr1_table_then_succeeds() {
    // Given
    let grammar = no_precedence_grammar();

    // When
    let result = build_lr1_parse_table(&grammar);

    // Then
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// build_runtime_parse_table tests
// ---------------------------------------------------------------------------

#[test]
fn given_dangling_else_grammar_when_building_runtime_table_then_succeeds() {
    // Given
    let grammar = dangling_else_grammar();

    // When
    let result = build_runtime_parse_table(&grammar);

    // Then
    assert!(result.is_ok());
}

#[test]
fn given_precedence_arithmetic_grammar_when_building_runtime_table_then_succeeds() {
    // Given
    let grammar = precedence_arithmetic_grammar(Associativity::Left);

    // When
    let result = build_runtime_parse_table(&grammar);

    // Then
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// build_dangling_else_parse_table tests
// ---------------------------------------------------------------------------

#[test]
fn given_no_args_when_building_dangling_else_table_then_succeeds() {
    // Given / When
    let result = build_dangling_else_parse_table();

    // Then
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// build_runtime_dangling_else_parse_table tests
// ---------------------------------------------------------------------------

#[test]
fn given_no_args_when_building_runtime_dangling_else_table_then_succeeds() {
    // Given / When
    let result = build_runtime_dangling_else_parse_table();

    // Then
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// build_precedence_arithmetic_parse_table tests
// ---------------------------------------------------------------------------

#[test]
fn given_left_assoc_when_building_precedence_table_then_succeeds() {
    // Given / When
    let result = build_precedence_arithmetic_parse_table(Associativity::Left);

    // Then
    assert!(result.is_ok());
}

#[test]
fn given_right_assoc_when_building_precedence_table_then_succeeds() {
    // Given / When
    let result = build_precedence_arithmetic_parse_table(Associativity::Right);

    // Then
    assert!(result.is_ok());
}

#[test]
fn given_none_assoc_when_building_precedence_table_then_succeeds() {
    // Given / When
    let result = build_precedence_arithmetic_parse_table(Associativity::None);

    // Then
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// build_runtime_precedence_arithmetic_parse_table tests
// ---------------------------------------------------------------------------

#[test]
fn given_left_assoc_when_building_runtime_precedence_table_then_succeeds() {
    // Given / When
    let result = build_runtime_precedence_arithmetic_parse_table(Associativity::Left);

    // Then
    assert!(result.is_ok());
}

#[test]
fn given_right_assoc_when_building_runtime_precedence_table_then_succeeds() {
    // Given / When
    let result = build_runtime_precedence_arithmetic_parse_table(Associativity::Right);

    // Then
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Re-exported conflict analysis tests
// ---------------------------------------------------------------------------

#[test]
fn given_conflict_analysis_struct_when_importing_then_is_available() {
    // Given / When / Then
    // Just verify the struct is available by using it in a type annotation
    let _ = Some::<ConflictAnalysis>;
}

#[test]
fn given_analyze_conflicts_fn_when_importing_then_is_available() {
    // Given / When / Then
    // Just verify the function is available
    let _ = analyze_conflicts;
}

#[test]
fn given_count_multi_action_cells_fn_when_importing_then_is_available() {
    // Given / When / Then
    // Just verify the function is available
    let _ = count_multi_action_cells;
}

#[test]
fn given_resolve_shift_reduce_actions_fn_when_importing_then_is_available() {
    // Given / When / Then
    // Just verify the function is available
    let _ = resolve_shift_reduce_actions;
}
