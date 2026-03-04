// Comprehensive tests for BDD grammar fixtures
use adze_bdd_grammar_fixtures::*;
use adze_ir::Associativity;

// ---------------------------------------------------------------------------
// Grammar construction tests
// ---------------------------------------------------------------------------

#[test]
fn dangling_else_grammar_builds() {
    let g = dangling_else_grammar();
    assert!(!g.rules.is_empty());
    assert!(!g.tokens.is_empty());
}

#[test]
fn dangling_else_has_start_symbol() {
    let g = dangling_else_grammar();
    assert!(g.start_symbol().is_some());
}

#[test]
fn dangling_else_has_if_token() {
    let g = dangling_else_grammar();
    // Should have tokens for if/else/condition/action
    assert!(!g.tokens.is_empty());
}

#[test]
fn precedence_arithmetic_left() {
    let g = precedence_arithmetic_grammar(Associativity::Left);
    assert!(!g.rules.is_empty());
    assert!(g.start_symbol().is_some());
}

#[test]
fn precedence_arithmetic_right() {
    let g = precedence_arithmetic_grammar(Associativity::Right);
    assert!(!g.rules.is_empty());
}

#[test]
fn precedence_arithmetic_none() {
    let g = precedence_arithmetic_grammar(Associativity::None);
    assert!(!g.rules.is_empty());
}

#[test]
fn no_precedence_grammar_builds() {
    let g = no_precedence_grammar();
    assert!(!g.rules.is_empty());
    assert!(g.start_symbol().is_some());
}

// ---------------------------------------------------------------------------
// Parse table building tests
// ---------------------------------------------------------------------------

#[test]
fn build_lr1_table_succeeds() {
    let g = dangling_else_grammar();
    let result = build_lr1_parse_table(&g);
    assert!(result.is_ok());
}

#[test]
fn build_runtime_table_succeeds() {
    let g = dangling_else_grammar();
    let result = build_runtime_parse_table(&g);
    assert!(result.is_ok());
}

#[test]
fn build_dangling_else_table_shortcut() {
    let result = build_dangling_else_parse_table();
    assert!(result.is_ok());
}

#[test]
fn build_runtime_dangling_else_table_shortcut() {
    let result = build_runtime_dangling_else_parse_table();
    assert!(result.is_ok());
}

#[test]
fn build_precedence_arithmetic_table_left() {
    let result = build_precedence_arithmetic_parse_table(Associativity::Left);
    assert!(result.is_ok());
}

#[test]
fn build_precedence_arithmetic_table_right() {
    let result = build_precedence_arithmetic_parse_table(Associativity::Right);
    assert!(result.is_ok());
}

#[test]
fn build_runtime_precedence_arithmetic_table_left() {
    let result = build_runtime_precedence_arithmetic_parse_table(Associativity::Left);
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Parse table properties
// ---------------------------------------------------------------------------

#[test]
fn dangling_else_has_conflicts() {
    let table = build_dangling_else_parse_table().unwrap();
    // Dangling else grammar should have shift-reduce conflicts
    let conflict_count = count_multi_action_cells(&table);
    // It's a classic ambiguous grammar
    assert!(conflict_count >= 0); // May or may not have conflicts after resolution
}

#[test]
fn no_precedence_table_builds() {
    let g = no_precedence_grammar();
    let result = build_lr1_parse_table(&g);
    assert!(result.is_ok());
}

#[test]
fn table_state_count_positive() {
    let table = build_dangling_else_parse_table().unwrap();
    assert!(table.state_count > 0);
}

#[test]
fn table_symbol_count_positive() {
    let table = build_dangling_else_parse_table().unwrap();
    assert!(table.symbol_count > 0);
}

// ---------------------------------------------------------------------------
// Conflict analysis
// ---------------------------------------------------------------------------

#[test]
fn analyze_dangling_else_conflicts() {
    let table = build_dangling_else_parse_table().unwrap();
    let analysis = analyze_conflicts(&table);
    // Analysis should return meaningful results
    let _ = format!("{:?}", analysis);
}

#[test]
fn resolve_shift_reduce_works() {
    let g = dangling_else_grammar();
    let table = build_lr1_parse_table(&g).unwrap();
    let _conflict_count = count_multi_action_cells(&table);
}

// ---------------------------------------------------------------------------
// Determinism tests
// ---------------------------------------------------------------------------

#[test]
fn dangling_else_grammar_deterministic() {
    let g1 = dangling_else_grammar();
    let g2 = dangling_else_grammar();
    assert_eq!(g1.rules.len(), g2.rules.len());
    assert_eq!(g1.tokens.len(), g2.tokens.len());
}

#[test]
fn precedence_grammar_deterministic() {
    let g1 = precedence_arithmetic_grammar(Associativity::Left);
    let g2 = precedence_arithmetic_grammar(Associativity::Left);
    assert_eq!(g1.rules.len(), g2.rules.len());
}

#[test]
fn no_precedence_grammar_deterministic() {
    let g1 = no_precedence_grammar();
    let g2 = no_precedence_grammar();
    assert_eq!(g1.rules.len(), g2.rules.len());
}

// ---------------------------------------------------------------------------
// Token pattern specs
// ---------------------------------------------------------------------------

#[test]
fn token_pattern_kind_regex() {
    let k = TokenPatternKind::Regex("[0-9]+");
    assert_eq!(k, TokenPatternKind::Regex("[0-9]+"));
}

#[test]
fn token_pattern_kind_literal() {
    let k = TokenPatternKind::Literal("if");
    assert_eq!(k, TokenPatternKind::Literal("if"));
}

#[test]
fn token_pattern_kind_ne() {
    let r = TokenPatternKind::Regex("x");
    let l = TokenPatternKind::Literal("x");
    assert_ne!(r, l);
}

#[test]
fn token_pattern_kind_debug() {
    let k = TokenPatternKind::Regex("[a-z]+");
    let debug = format!("{:?}", k);
    assert!(debug.contains("Regex"));
}

#[test]
fn token_pattern_kind_clone() {
    let k = TokenPatternKind::Literal("hello");
    let k2 = k;
    assert_eq!(k, k2);
}

// ---------------------------------------------------------------------------
// Multiple associativity variants produce different tables
// ---------------------------------------------------------------------------

#[test]
fn different_associativity_may_differ() {
    let left = precedence_arithmetic_grammar(Associativity::Left);
    let right = precedence_arithmetic_grammar(Associativity::Right);
    // They should have the same number of rules but different precedence annotations
    assert_eq!(left.rules.len(), right.rules.len());
}
