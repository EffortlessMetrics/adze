//! BDD Scenario Tests: GLR Runtime Fork/Merge (Runtime2)
//!
//! This test suite validates GLR runtime behavior when encountering
//! conflicts during parsing. Tests scenarios 7-8 from BDD specification.
//!
//! Reference: docs/plans/BDD_GLR_CONFLICT_PRESERVATION.md

#![cfg(all(feature = "pure-rust", feature = "serialization"))]

use adze_bdd_scenario_fixtures::{
    BddPhase, DANGLING_ELSE_SYMBOL_METADATA, DANGLING_ELSE_TOKEN_PATTERNS,
    PRECEDENCE_ARITHMETIC_SYMBOL_METADATA, PRECEDENCE_ARITHMETIC_TOKEN_PATTERNS,
    SymbolMetadataSpec, TokenPatternKind, TokenPatternSpec,
    build_runtime_dangling_else_parse_table, build_runtime_precedence_arithmetic_parse_table,
    count_multi_action_cells,
};
use adze_glr_core::ParseTable;
use adze_ir::Associativity;
use adze_runtime::{
    Parser, bdd_progress_report_for_current_profile,
    language::SymbolMetadata,
    tokenizer::{Matcher, TokenPattern as RuntimeTokenPattern},
};

fn build_runtime_symbol_metadata(specs: &[SymbolMetadataSpec]) -> Vec<SymbolMetadata> {
    specs
        .iter()
        .map(|spec| SymbolMetadata {
            is_terminal: spec.is_terminal,
            is_visible: spec.is_visible,
            is_supertype: spec.is_supertype,
        })
        .collect()
}

fn build_runtime_token_patterns(patterns: &[TokenPatternSpec]) -> Vec<RuntimeTokenPattern> {
    patterns
        .iter()
        .map(|pattern| RuntimeTokenPattern {
            symbol_id: pattern.symbol_id,
            matcher: match pattern.matcher {
                TokenPatternKind::Regex(pattern) => Matcher::Regex(
                    regex::Regex::new(pattern).expect("fixture regex pattern should compile"),
                ),
                TokenPatternKind::Literal(literal) => Matcher::Literal(literal.to_string()),
            },
            is_keyword: pattern.is_keyword,
        })
        .collect()
}

fn build_runtime_parser(
    parse_table: &'static ParseTable,
    symbol_metadata: &[SymbolMetadataSpec],
    token_patterns: &[TokenPatternSpec],
) -> Parser {
    let mut parser = Parser::new();
    parser
        .set_glr_table(parse_table)
        .expect("Setting GLR table should succeed");
    parser
        .set_symbol_metadata(build_runtime_symbol_metadata(symbol_metadata))
        .expect("Setting symbol metadata should succeed");
    parser
        .set_token_patterns(build_runtime_token_patterns(token_patterns))
        .expect("Setting token patterns should succeed");
    parser
}

fn build_dangling_else_parse_table() -> &'static ParseTable {
    Box::leak(Box::new(
        build_runtime_dangling_else_parse_table().expect("LR(1) automaton build failed"),
    ))
}

fn create_dangling_else_parser(parse_table: &'static ParseTable) -> Parser {
    build_runtime_parser(
        parse_table,
        DANGLING_ELSE_SYMBOL_METADATA,
        DANGLING_ELSE_TOKEN_PATTERNS,
    )
}

fn build_precedence_arithmetic_parse_table() -> &'static ParseTable {
    Box::leak(Box::new(
        build_runtime_precedence_arithmetic_parse_table(Associativity::Left)
            .expect("LR(1) automaton build failed"),
    ))
}

fn create_precedence_arithmetic_parser(parse_table: &'static ParseTable) -> Parser {
    build_runtime_parser(
        parse_table,
        PRECEDENCE_ARITHMETIC_SYMBOL_METADATA,
        PRECEDENCE_ARITHMETIC_TOKEN_PATTERNS,
    )
}

//
// ============================================================================
// Scenario 7: GLR Runtime Explores Both Paths
// ============================================================================
//

#[test]
fn scenario_7_glr_runtime_parses_ambiguous_input() {
    // GIVEN a parse table with multi-action cells (dangling else grammar)
    let table_static = build_dangling_else_parse_table();
    let mut parser = create_dangling_else_parser(table_static);

    assert!(
        count_multi_action_cells(table_static) > 0,
        "expected multi-action cells in dangling-else table"
    );

    println!("\n=== Scenario 7: GLR Runtime Fork/Merge ===");

    // WHEN the GLR runtime encounters ambiguous input
    let input = b"if expr then if expr then stmt else stmt";
    println!("Parsing: {:?}", std::str::from_utf8(input).unwrap());

    // THEN the parser successfully parses the input (exploring both paths)
    let result = parser.parse(input, None);

    match &result {
        Ok(tree) => {
            let root = tree.root_node();
            println!("✓ Parse succeeded!");
            println!("  Root kind: {}", root.kind());
            println!("  Root kind_id: {:?}", root.kind_id());
            println!("  Child count: {}", root.child_count());

            // Validate tree structure
            assert_eq!(root.kind(), "S", "Root should be statement");
            assert!(
                root.child_count() > 0,
                "Root should have children (parse tree should be non-empty)"
            );

            println!("\n✓ Scenario 7 PASS: GLR runtime successfully parses ambiguous input");
            println!("  (Note: Currently returns single tree; full forest support is future work)");
        }
        Err(e) => {
            println!("✗ Parse failed: {}", e);
            panic!("GLR parser should handle ambiguous input, got error: {}", e);
        }
    }

    // AND the parser produces a valid parse tree
    assert!(
        result.is_ok(),
        "Parser should succeed on well-formed ambiguous input"
    );
}

//
// ============================================================================
// Scenario 7b: Simple Statement (No Ambiguity)
// ============================================================================
//

#[test]
fn scenario_7b_simple_statement_no_ambiguity() {
    // GIVEN a parse table for dangling else grammar
    let table_static = build_dangling_else_parse_table();
    let mut parser = create_dangling_else_parser(table_static);

    println!("\n=== Scenario 7b: Simple Statement (Baseline) ===");

    // WHEN parsing simple non-ambiguous input
    let input = b"stmt";
    println!("Parsing: {:?}", std::str::from_utf8(input).unwrap());

    // THEN parsing succeeds
    // Note: Parser::parse_glr() already applies Phase 3.3 symbol name resolution
    let tree = parser
        .parse(input, None)
        .expect("Should parse simple statement");
    let root = tree.root_node();

    println!("✓ Parse succeeded!");
    println!("  Root kind: {}", root.kind());
    println!("  Root kind_id: {}", root.kind_id());
    println!("  Child count: {}", root.child_count());
    println!("  Parse table symbol_count: {}", table_static.symbol_count);
    println!(
        "  Grammar rule_names: {:?}",
        table_static.grammar.rule_names
    );

    assert_eq!(root.kind(), "S", "Root should be statement");

    // Validate child structure
    assert_eq!(root.child_count(), 1, "Root should have 1 child");
    let child = root.child(0).expect("Should have child node");
    assert_eq!(child.kind(), "stmt", "Child should be 'stmt' token");

    println!("\n✓ Scenario 7b PASS: Simple statement parses correctly with symbol names");
}

//
// ============================================================================
// Scenario 8: Precedence Ordering Affects Deterministic Tree Selection
// ============================================================================
//

#[test]
fn scenario_8_precedence_tree_selection() {
    // GIVEN a grammar with explicit precedence and associativity
    let table_static = build_precedence_arithmetic_parse_table();
    let mut parser = create_precedence_arithmetic_parser(table_static);

    // THEN precedence should resolve conflicts at table-build time
    assert_eq!(
        count_multi_action_cells(table_static),
        0,
        "precedence-annotated grammar should not require runtime forking"
    );

    println!("\n=== Scenario 8: Precedence Affects Tree Selection ===");
    let input = b"1 + 2 * 3";
    let tree = parser
        .parse(input, None)
        .expect("precedence grammar should parse successfully");
    let root = tree.root_node();

    // WHEN parsing 1 + 2 * 3
    // THEN multiplication binds tighter than addition
    assert_eq!(root.kind(), "Expr");
    assert_eq!(root.child_count(), 3, "root should be a binary expression");
    let right = root.child(2).expect("right child should exist");
    assert_eq!(right.kind(), "Expr");
    assert_eq!(
        right.child_count(),
        3,
        "right child should also be a binary expression (2 * 3)"
    );
}

//
// ============================================================================
// BDD Test Summary
// ============================================================================
//

#[test]
fn bdd_runtime_test_summary() {
    let status = bdd_progress_report_for_current_profile(
        BddPhase::Runtime,
        "Phase 2 (runtime2 integration tests)",
    );
    println!("{status}");
}
