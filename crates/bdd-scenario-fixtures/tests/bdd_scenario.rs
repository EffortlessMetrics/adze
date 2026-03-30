//! BDD-style tests for bdd-scenario-fixtures crate.
//!
//! Tests follow the Given/When/Then pattern to verify public API behavior.
//! This crate is a façade that re-exports from bdd-governance-fixtures and
//! bdd-grammar-fixtures.

use adze_bdd_scenario_fixtures::*;

// ---------------------------------------------------------------------------
// BddPhase re-export tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_using_bdd_phase_then_variant_is_core() {
    // Given
    let phase = BddPhase::Core;

    // When / Then
    assert!(matches!(phase, BddPhase::Core));
}

#[test]
fn given_runtime_phase_when_using_bdd_phase_then_variant_is_runtime() {
    // Given
    let phase = BddPhase::Runtime;

    // When / Then
    assert!(matches!(phase, BddPhase::Runtime));
}

// ---------------------------------------------------------------------------
// BddScenarioStatus re-export tests
// ---------------------------------------------------------------------------

#[test]
fn given_implemented_status_when_checking_implemented_then_returns_true() {
    // Given
    let status = BddScenarioStatus::Implemented;

    // When / Then
    assert!(status.implemented());
}

#[test]
fn given_deferred_status_when_checking_implemented_then_returns_false() {
    // Given
    let status = BddScenarioStatus::Deferred {
        reason: "pending implementation",
    };

    // When / Then
    assert!(!status.implemented());
}

#[test]
fn given_deferred_status_when_getting_reason_then_returns_reason() {
    // Given
    let status = BddScenarioStatus::Deferred {
        reason: "pending implementation",
    };

    // When
    let detail = status.detail();

    // Then
    assert_eq!(detail, "pending implementation");
}

// ---------------------------------------------------------------------------
// GLR_CONFLICT_PRESERVATION_GRID re-export tests
// ---------------------------------------------------------------------------

#[test]
fn given_glr_grid_when_checking_is_empty_then_returns_false() {
    // Given / When
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // Then
    assert!(!grid.is_empty());
}

#[test]
fn given_glr_grid_when_iterating_scenarios_then_has_scenarios() {
    // Given
    let grid = GLR_CONFLICT_PRESERVATION_GRID;

    // When
    let count = grid.len();

    // Then
    assert!(count > 0);
}

// ---------------------------------------------------------------------------
// bdd_progress re-export tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_calling_bdd_progress_then_returns_valid_counts() {
    // Given / When
    let (implemented, total) = bdd_progress(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert!(total > 0);
    assert!(implemented <= total);
}

#[test]
fn given_runtime_phase_when_calling_bdd_progress_then_returns_valid_counts() {
    // Given / When
    let (implemented, total) = bdd_progress(BddPhase::Runtime, GLR_CONFLICT_PRESERVATION_GRID);

    // Then
    assert!(total > 0);
    assert!(implemented <= total);
}

#[test]
fn given_empty_scenarios_when_calling_bdd_progress_then_returns_zero_counts() {
    // Given
    let scenarios: &[BddScenario] = &[];

    // When
    let (implemented, total) = bdd_progress(BddPhase::Core, scenarios);

    // Then
    assert_eq!(implemented, 0);
    assert_eq!(total, 0);
}

// ---------------------------------------------------------------------------
// bdd_progress_report re-export tests
// ---------------------------------------------------------------------------

#[test]
fn given_title_when_calling_bdd_progress_report_then_report_contains_title() {
    // Given
    let title = "Test Report Title";

    // When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, title);

    // Then
    assert!(report.contains(title));
}

#[test]
fn given_core_phase_when_calling_bdd_progress_report_then_report_is_non_empty() {
    // Given / When
    let report = bdd_progress_report(BddPhase::Core, GLR_CONFLICT_PRESERVATION_GRID, "Core Phase");

    // Then
    assert!(!report.is_empty());
}

// ---------------------------------------------------------------------------
// bdd_progress_report_for_current_profile re-export tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_calling_bdd_progress_report_for_current_profile_then_report_is_non_empty()
{
    // Given / When
    let report = bdd_progress_report_for_current_profile(BddPhase::Core, "Core Phase");

    // Then
    assert!(!report.is_empty());
    assert!(report.contains("Core Phase"));
}

#[test]
fn given_runtime_phase_when_calling_bdd_progress_report_for_current_profile_then_report_is_non_empty()
 {
    // Given / When
    let report = bdd_progress_report_for_current_profile(BddPhase::Runtime, "Runtime Phase");

    // Then
    assert!(!report.is_empty());
    assert!(report.contains("Runtime Phase"));
}

// ---------------------------------------------------------------------------
// bdd_progress_status_line_for_current_profile re-export tests
// ---------------------------------------------------------------------------

#[test]
fn given_core_phase_when_calling_status_line_for_current_profile_then_line_is_non_empty() {
    // Given / When
    let line = bdd_progress_status_line_for_current_profile(BddPhase::Core);

    // Then
    assert!(!line.is_empty());
}

#[test]
fn given_runtime_phase_when_calling_status_line_for_current_profile_then_line_is_non_empty() {
    // Given / When
    let line = bdd_progress_status_line_for_current_profile(BddPhase::Runtime);

    // Then
    assert!(!line.is_empty());
}

// ---------------------------------------------------------------------------
// ParserFeatureProfile re-export tests
// ---------------------------------------------------------------------------

#[test]
fn given_current_profile_when_calling_current_then_returns_valid_profile() {
    // Given / When
    let profile = ParserFeatureProfile::current();

    // Then
    let _ = format!("{:?}", profile);
}

// ---------------------------------------------------------------------------
// ParserBackend re-export tests
// ---------------------------------------------------------------------------

#[test]
fn given_tree_sitter_backend_when_using_parser_backend_then_variant_is_tree_sitter() {
    // Given
    let backend = ParserBackend::TreeSitter;

    // When / Then
    assert!(matches!(backend, ParserBackend::TreeSitter));
}

#[test]
fn given_glr_backend_when_using_parser_backend_then_variant_is_glr() {
    // Given
    let backend = ParserBackend::GLR;

    // When / Then
    assert!(matches!(backend, ParserBackend::GLR));
}

// ---------------------------------------------------------------------------
// Grammar fixture re-export tests (from bdd-grammar-fixtures)
// ---------------------------------------------------------------------------

#[test]
fn given_dangling_else_grammar_when_building_then_has_correct_name() {
    // Given / When
    let grammar = dangling_else_grammar();

    // Then
    assert_eq!(grammar.name, "if_then_else");
}

#[test]
fn given_dangling_else_grammar_when_building_then_has_tokens_and_rules() {
    // Given / When
    let grammar = dangling_else_grammar();

    // Then
    assert!(!grammar.tokens.is_empty());
    assert!(!grammar.rules.is_empty());
}

#[test]
fn given_precedence_arithmetic_grammar_when_building_then_succeeds() {
    // Given / When
    use adze_ir::Associativity;
    let grammar = precedence_arithmetic_grammar(Associativity::Left);

    // Then
    assert_eq!(grammar.name, "precedence_expr");
}

#[test]
fn given_no_precedence_grammar_when_building_then_succeeds() {
    // Given / When
    let grammar = no_precedence_grammar();

    // Then
    assert_eq!(grammar.name, "no_precedence_expr");
}

// ---------------------------------------------------------------------------
// Parse table builder re-export tests
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
fn given_dangling_else_grammar_when_building_runtime_table_then_succeeds() {
    // Given
    let grammar = dangling_else_grammar();

    // When
    let result = build_runtime_parse_table(&grammar);

    // Then
    assert!(result.is_ok());
}

#[test]
fn given_no_args_when_building_dangling_else_table_then_succeeds() {
    // Given / When
    let result = build_dangling_else_parse_table();

    // Then
    assert!(result.is_ok());
}

#[test]
fn given_no_args_when_building_runtime_dangling_else_table_then_succeeds() {
    // Given / When
    let result = build_runtime_dangling_else_parse_table();

    // Then
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// TokenPatternKind re-export tests
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

// ---------------------------------------------------------------------------
// SymbolMetadataSpec re-export tests
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

// ---------------------------------------------------------------------------
// Fixture constants re-export tests
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

// ---------------------------------------------------------------------------
// Conflict analysis re-export tests
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
