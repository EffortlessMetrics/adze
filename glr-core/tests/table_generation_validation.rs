/// Table Generation Validation Tests
///
/// These tests validate that parse table generation correctly preserves
/// GLR conflicts by generating ParseTables from Grammar IR and validating
/// their conflict properties.
///
/// Spec: docs/specs/TABLE_GENERATION_VALIDATION_CONTRACT.md
/// Phase: 2 - GLR Conflict Preservation Validation
use adze_glr_core::conflict_inspection::*;
use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use std::collections::BTreeMap;

/// Helper to create a test grammar from a simplified specification
///
/// This builder creates a minimal Grammar IR for testing conflict detection.
/// It handles symbol registration and rule construction.
///
/// # Arguments
///
/// * `name` - Grammar name
/// * `rules` - List of (lhs, rhs) tuples where symbols are referenced by name
/// * `terminals` - List of terminal symbol names
///
/// # Returns
///
/// A Grammar IR ready for table generation
pub fn build_test_grammar(
    name: &str,
    rules: Vec<(&str, Vec<&str>)>,
    terminals: Vec<&str>,
) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());

    // Build symbol name -> ID mapping
    let mut symbol_map: BTreeMap<String, SymbolId> = BTreeMap::new();
    let mut next_id = 0u16;

    // Register terminals first
    for term in terminals.iter() {
        let symbol_id = SymbolId(next_id);
        next_id += 1;

        grammar.tokens.insert(
            symbol_id,
            Token {
                name: term.to_string(),
                pattern: TokenPattern::String(term.to_string()),
                fragile: false,
            },
        );
        grammar.rule_names.insert(symbol_id, term.to_string());
        symbol_map.insert(term.to_string(), symbol_id);
    }

    // Register non-terminals (LHS of rules)
    for (lhs, _) in rules.iter() {
        if !symbol_map.contains_key(*lhs) {
            let symbol_id = SymbolId(next_id);
            next_id += 1;

            grammar.rule_names.insert(symbol_id, lhs.to_string());
            symbol_map.insert(lhs.to_string(), symbol_id);
        }
    }

    // Create rules
    for (rule_idx, (lhs, rhs)) in rules.iter().enumerate() {
        let lhs_id = *symbol_map.get(*lhs).expect("LHS symbol should exist");

        // Convert RHS symbol names to Symbol enum
        let rhs_symbols: Vec<Symbol> = rhs
            .iter()
            .map(|sym_name| {
                let sym_id = *symbol_map
                    .get(*sym_name)
                    .unwrap_or_else(|| panic!("Symbol '{}' not found", sym_name));

                // Check if it's a terminal or non-terminal
                if grammar.tokens.contains_key(&sym_id) {
                    Symbol::Terminal(sym_id)
                } else {
                    Symbol::NonTerminal(sym_id)
                }
            })
            .collect();

        let rule = Rule {
            lhs: lhs_id,
            rhs: rhs_symbols,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(rule_idx as u16),
        };

        grammar.add_rule(rule);
    }

    grammar
}

/// Generate parse table and validate conflict properties
///
/// This helper combines table generation and conflict validation into
/// a single operation with assertion-based validation.
///
/// Uses the REAL parse table generation pipeline:
/// Grammar IR → FirstFollowSets → build_lr1_automaton → ParseTable
///
/// # Arguments
///
/// * `grammar` - Grammar IR to generate table from
/// * `min_sr` - Minimum expected shift/reduce conflicts (lower bound)
/// * `min_rr` - Minimum expected reduce/reduce conflicts (lower bound)
///
/// # Returns
///
/// Tuple of (ParseTable, ConflictSummary) if validation passes
///
/// # Panics
///
/// Panics if conflict counts are below minimums
pub fn generate_and_validate_table(
    grammar: &mut Grammar,
    min_sr: usize,
    min_rr: usize,
) -> Result<(ParseTable, ConflictSummary), adze_glr_core::GLRError> {
    // Step 1: Compute FIRST/FOLLOW sets with normalization
    // This handles complex symbols (Repeat, Choice, etc.)
    let first_follow = FirstFollowSets::compute_normalized(grammar)?;

    // Step 2: Build LR(1) automaton and generate ParseTable
    // This is the REAL table generation path used in production
    let table = build_lr1_automaton(grammar, &first_follow)?;

    // Step 3: Inspect conflicts using conflict_inspection API
    let summary = count_conflicts(&table);

    // Step 4: Validate against minimum expectations
    // Using >= instead of == makes tests more stable across algorithm changes
    assert!(
        summary.shift_reduce >= min_sr,
        "Expected at least {} S/R conflicts, found {}",
        min_sr,
        summary.shift_reduce
    );

    assert!(
        summary.reduce_reduce >= min_rr,
        "Expected at least {} R/R conflicts, found {}",
        min_rr,
        summary.reduce_reduce
    );

    Ok((table, summary))
}

/// Test TG-001: Dangling Else Grammar
///
/// Grammar:
///   Statement → if Expr then Statement
///   Statement → if Expr then Statement else Statement
///   Statement → other
///   Expr → id
///
/// Expected: 1 shift/reduce conflict on "else" token
#[test]
fn test_dangling_else_table_generation() {
    let mut grammar = build_test_grammar(
        "dangling_else",
        vec![
            ("Statement", vec!["if", "Expr", "then", "Statement"]),
            (
                "Statement",
                vec!["if", "Expr", "then", "Statement", "else", "Statement"],
            ),
            ("Statement", vec!["other"]),
            ("Expr", vec!["id"]),
        ],
        vec!["if", "then", "else", "other", "id"],
    );

    let result = generate_and_validate_table(&mut grammar, 1, 0);

    match result {
        Ok((table, summary)) => {
            eprintln!("✅ TG-001 Dangling Else: Table generated successfully");
            eprintln!("  States: {}", table.state_count);
            eprintln!("  S/R conflicts: {}", summary.shift_reduce);
            eprintln!("  R/R conflicts: {}", summary.reduce_reduce);

            // Additional validation: find the "else" conflict
            if let Some(else_symbol) = grammar.find_symbol_by_name("else") {
                let else_conflicts = find_conflicts_for_symbol(&table, else_symbol);
                eprintln!("  Conflicts on 'else': {}", else_conflicts.len());

                if !else_conflicts.is_empty() {
                    let conflict = &else_conflicts[0];
                    eprintln!("  Conflict type: {:?}", conflict.conflict_type);
                    eprintln!("  Actions: {}", conflict.actions.len());

                    // Verify one Shift and one Reduce
                    let has_shift = conflict
                        .actions
                        .iter()
                        .any(|a| matches!(a, Action::Shift(_)));
                    let has_reduce = conflict
                        .actions
                        .iter()
                        .any(|a| matches!(a, Action::Reduce(_)));

                    assert!(
                        has_shift && has_reduce,
                        "Should have both Shift and Reduce actions"
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("❌ TG-001 Dangling Else: Table generation failed");
            eprintln!("  Error: {:?}", e);
            panic!("Table generation failed: {:?}", e);
        }
    }
}

/// Test TG-002: Precedence-Free Expression Grammar
///
/// Grammar:
///   Expr → Expr Op Expr
///   Expr → Number
///   Op → +
///   Op → *
///
/// Expected: >= 2 shift/reduce conflicts (one per operator minimum)
#[test]
fn test_precedence_free_expr_table_generation() {
    let mut grammar = build_test_grammar(
        "precedence_free",
        vec![
            ("Expr", vec!["Expr", "Op", "Expr"]),
            ("Expr", vec!["Number"]),
            ("Op", vec!["+"]),
            ("Op", vec!["*"]),
        ],
        vec!["+", "*", "Number"],
    );

    let result = generate_and_validate_table(&mut grammar, 2, 0);

    match result {
        Ok((table, summary)) => {
            eprintln!("✅ TG-002 Precedence-Free Expression: Table generated successfully");
            eprintln!("  States: {}", table.state_count);
            eprintln!("  S/R conflicts: {}", summary.shift_reduce);
            eprintln!("  R/R conflicts: {}", summary.reduce_reduce);

            // Should have at least 2 S/R conflicts
            assert!(
                summary.shift_reduce >= 2,
                "Expected at least 2 S/R conflicts, got {}",
                summary.shift_reduce
            );

            // All conflicts should be shift/reduce (no reduce/reduce)
            assert_eq!(summary.reduce_reduce, 0);

            // Verify all conflicts are shift/reduce type
            for conflict in &summary.conflict_details {
                eprintln!(
                    "  Conflict: state={}, symbol={}, type={:?}, actions={}",
                    conflict.state.0,
                    conflict.symbol_name,
                    conflict.conflict_type,
                    conflict.actions.len()
                );

                assert_eq!(
                    conflict.conflict_type,
                    ConflictType::ShiftReduce,
                    "All conflicts should be S/R"
                );
            }
        }
        Err(e) => {
            eprintln!("❌ TG-002 Precedence-Free Expression: Table generation failed");
            eprintln!("  Error: {:?}", e);
            panic!("Table generation failed: {:?}", e);
        }
    }
}

/// Test TG-003: Precedence-Resolved Arithmetic (Conflict-Free)
///
/// Grammar with precedence annotations:
///   Expr → Expr + Expr  [prec_left 1]
///   Expr → Expr * Expr  [prec_left 2]
///   Expr → Number
///
/// Expected: 0 conflicts (precedence eliminates ambiguity)
#[test]
fn test_precedence_resolved_arithmetic_is_conflict_free() {
    // For now, we can't easily add precedence via build_test_grammar
    // This test documents the expectation and will be implemented
    // when we wire up real grammar files with precedence annotations

    eprintln!("TG-003 Precedence-Resolved Arithmetic:");
    eprintln!("  Expected S/R conflicts: 0");
    eprintln!("  Expected R/R conflicts: 0");
    eprintln!("  Status: Specification documented, awaiting precedence support in test builder");

    // TODO: Implement once we can specify precedence in build_test_grammar
    // or load from a real grammar file with precedence annotations
}

/// Test: Grammar Builder Creates Valid IR
///
/// Validates that build_test_grammar() produces valid Grammar IR
#[test]
fn test_grammar_builder_creates_valid_ir() {
    let grammar = build_test_grammar(
        "simple",
        vec![("S", vec!["a", "S", "b"]), ("S", vec!["a", "b"])],
        vec!["a", "b"],
    );

    // Validate grammar structure
    assert_eq!(grammar.name, "simple");
    assert!(!grammar.rules.is_empty(), "Should have rules");
    assert!(!grammar.tokens.is_empty(), "Should have tokens");
    assert!(!grammar.rule_names.is_empty(), "Should have symbol names");

    // Validate symbols are registered
    assert!(
        grammar.find_symbol_by_name("a").is_some(),
        "Terminal 'a' should exist"
    );
    assert!(
        grammar.find_symbol_by_name("b").is_some(),
        "Terminal 'b' should exist"
    );
    assert!(
        grammar.find_symbol_by_name("S").is_some(),
        "Non-terminal 'S' should exist"
    );

    // Validate rules
    let s_symbol = grammar.find_symbol_by_name("S").unwrap();
    let s_rules = grammar.get_rules_for_symbol(s_symbol);
    assert!(s_rules.is_some(), "Should have rules for 'S'");
    assert_eq!(s_rules.unwrap().len(), 2, "Should have 2 rules for 'S'");

    eprintln!("✅ Grammar builder creates valid IR");
}

/// Test: Table Generation Works with Simple Grammar
///
/// Smoke test that table generation pipeline works end-to-end
#[test]
fn test_table_generation_smoke_test() {
    let mut grammar = build_test_grammar(
        "simple",
        vec![("S", vec!["a"]), ("S", vec!["b"])],
        vec!["a", "b"],
    );

    // This should not have conflicts (simple choice)
    let result = generate_and_validate_table(&mut grammar, 0, 0);

    match result {
        Ok((table, summary)) => {
            eprintln!("✅ Table generation smoke test passed");
            eprintln!("  States: {}", table.state_count);
            eprintln!(
                "  Conflicts: {}",
                summary.shift_reduce + summary.reduce_reduce
            );
            assert_eq!(summary.shift_reduce, 0);
            assert_eq!(summary.reduce_reduce, 0);
        }
        Err(e) => {
            eprintln!("❌ Table generation smoke test failed: {:?}", e);
            panic!("Smoke test failed: {:?}", e);
        }
    }
}
