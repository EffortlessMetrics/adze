//! Integration tests for GLR conflict inspection API
//!
//! These tests validate that the conflict inspection API works correctly
//! with ambiguous grammars like dangling_else and ambiguous_expr.
//!
//! Phase 2: GLR Conflict Preservation Validation
//! Spec: docs/specs/CONFLICT_INSPECTION_API.md
//! Related: docs/specs/AMBIGUOUS_GRAMMAR_TEST_SUITE.md

mod conflict_detection {
    use adze_glr_core::conflict_inspection::*;
    use adze_glr_core::{Action, GotoIndexing, ParseTable, StateId};
    use adze_ir::{Grammar, RuleId, SymbolId};

    /// Helper to create a test parse table for validation
    fn create_test_table(action_table: Vec<Vec<Vec<Action>>>) -> ParseTable {
        let state_count = action_table.len();
        ParseTable {
            action_table,
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count,
            symbol_count: 1,
            symbol_to_index: Default::default(),
            index_to_symbol: Default::default(),
            external_scanner_states: vec![],
            rules: vec![],
            nonterminal_to_index: Default::default(),
            goto_indexing: GotoIndexing::NonterminalMap,
            eof_symbol: SymbolId(0),
            start_symbol: SymbolId(0),
            grammar: Grammar::new("test".to_string()),
            initial_state: StateId(0),
            token_count: 0,
            external_token_count: 0,
            lex_modes: vec![],
            extras: vec![],
            dynamic_prec_by_rule: vec![],
            rule_assoc_by_rule: vec![],
            alias_sequences: vec![],
            field_names: vec![],
            field_map: Default::default(),
        }
    }

    #[test]
    fn test_api_structure() {
        // Validate that the conflict inspection API is accessible
        // and has the expected structure

        // Create a simple test table with one shift/reduce conflict
        let table = create_test_table(vec![vec![vec![
            Action::Shift(StateId(1)),
            Action::Reduce(RuleId(0)),
        ]]]);

        // Call the primary API
        let summary = count_conflicts(&table);

        // Validate the structure
        assert_eq!(summary.shift_reduce, 1);
        assert_eq!(summary.reduce_reduce, 0);
        assert_eq!(summary.states_with_conflicts.len(), 1);
        assert_eq!(summary.conflict_details.len(), 1);

        // Validate ConflictDetail structure
        let detail = &summary.conflict_details[0];
        assert_eq!(detail.conflict_type, ConflictType::ShiftReduce);
        assert_eq!(detail.state, StateId(0));
        assert_eq!(detail.actions.len(), 2);

        // Validate Display implementation works
        let summary_str = format!("{}", summary);
        assert!(summary_str.contains("Shift/Reduce conflicts: 1"));
        assert!(summary_str.contains("Reduce/Reduce conflicts: 0"));
    }

    #[test]
    fn test_helper_functions() {
        // Test the helper functions work correctly

        let table = create_test_table(vec![
            vec![vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]],
            vec![vec![Action::Shift(StateId(2))]],
            vec![vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))]],
        ]);

        // Test state_has_conflicts
        assert!(state_has_conflicts(&table, StateId(0)));
        assert!(!state_has_conflicts(&table, StateId(1)));
        assert!(state_has_conflicts(&table, StateId(2)));

        // Test get_state_conflicts
        let state0_conflicts = get_state_conflicts(&table, StateId(0));
        assert_eq!(state0_conflicts.len(), 1);
        assert_eq!(state0_conflicts[0].conflict_type, ConflictType::ShiftReduce);

        let state2_conflicts = get_state_conflicts(&table, StateId(2));
        assert_eq!(state2_conflicts.len(), 1);
        assert_eq!(
            state2_conflicts[0].conflict_type,
            ConflictType::ReduceReduce
        );
    }

    /// Test documentation: Expected conflicts for dangling_else grammar
    ///
    /// This test documents the expected conflicts for TG-001 (Dangling Else).
    /// Once full table generation is wired up, this test should be updated
    /// to actually generate and validate the parse table.
    ///
    /// Expected behavior from spec:
    /// - Input: "if a then if b then s else t"
    /// - Conflict: After "if b then s", on lookahead "else":
    ///   - SHIFT: Continue with inner if → "if a then (if b then s else t)"
    ///   - REDUCE: Complete outer if → "(if a then if b then s) else t"
    /// - Expected conflicts: 1 shift/reduce
    #[test]
    fn test_dangling_else_expected_conflicts() {
        // This test documents expected conflicts for the dangling_else grammar
        // from docs/specs/AMBIGUOUS_GRAMMAR_TEST_SUITE.md (TG-001)

        // Expected: 1 shift/reduce conflict on "else" token
        const EXPECTED_SR_CONFLICTS: usize = 1;
        const EXPECTED_RR_CONFLICTS: usize = 0;

        // TODO: Once table generation is fully wired:
        // 1. Generate ParseTable from dangling_else grammar IR
        // 2. Run count_conflicts on the table
        // 3. Validate against EXPECTED_SR_CONFLICTS and EXPECTED_RR_CONFLICTS

        // For now, document the expectation
        eprintln!("TG-001 Dangling Else:");
        eprintln!("  Expected S/R conflicts: {}", EXPECTED_SR_CONFLICTS);
        eprintln!("  Expected R/R conflicts: {}", EXPECTED_RR_CONFLICTS);
        eprintln!("  Status: Specification documented, awaiting table generation");
    }

    /// Test documentation: Expected conflicts for ambiguous_expr grammar
    ///
    /// This test documents the expected conflicts for TG-002 (Precedence-Free Expression).
    /// Once full table generation is wired up, this test should be updated
    /// to actually generate and validate the parse table.
    ///
    /// Expected behavior from spec:
    /// - Input: "1 + 2 * 3"
    /// - Conflicts: Each operator creates S/R conflict
    /// - Expected conflicts: >= 2 shift/reduce
    #[test]
    fn test_ambiguous_expr_expected_conflicts() {
        // This test documents expected conflicts for the ambiguous_expr grammar
        // from docs/specs/AMBIGUOUS_GRAMMAR_TEST_SUITE.md (TG-002)

        // Expected: >= 2 shift/reduce conflicts (one per operator)
        const MIN_EXPECTED_SR_CONFLICTS: usize = 2;
        const EXPECTED_RR_CONFLICTS: usize = 0;

        // TODO: Once table generation is fully wired:
        // 1. Generate ParseTable from ambiguous_expr grammar IR
        // 2. Run count_conflicts on the table
        // 3. Validate conflicts >= MIN_EXPECTED_SR_CONFLICTS

        // For now, document the expectation
        eprintln!("TG-002 Precedence-Free Expression:");
        eprintln!("  Expected S/R conflicts: >= {}", MIN_EXPECTED_SR_CONFLICTS);
        eprintln!("  Expected R/R conflicts: {}", EXPECTED_RR_CONFLICTS);
        eprintln!("  Status: Specification documented, awaiting table generation");
    }

    #[test]
    fn test_classify_conflict_types() {
        // Validate that conflict type classification works correctly

        // Shift/Reduce
        let sr_actions = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
        assert_eq!(classify_conflict(&sr_actions), ConflictType::ShiftReduce);

        // Reduce/Reduce
        let rr_actions = vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))];
        assert_eq!(classify_conflict(&rr_actions), ConflictType::ReduceReduce);

        // Mixed (multiple shifts)
        let mixed_actions = vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))];
        assert_eq!(classify_conflict(&mixed_actions), ConflictType::Mixed);

        // Fork with S/R inside
        let fork_actions = vec![Action::Fork(vec![
            Action::Shift(StateId(1)),
            Action::Reduce(RuleId(0)),
        ])];
        assert_eq!(classify_conflict(&fork_actions), ConflictType::ShiftReduce);
    }
}

/// Test that the conflict_inspection module is accessible
#[test]
fn test_conflict_inspection_module_exists() {
    // This test ensures the conflict_inspection module is always accessible
    // as it's a core part of the GLR implementation

    // If this test compiles, the module exists and is exported
    use adze_glr_core::conflict_inspection;
    // Force usage to avoid unused import warning
    let _ = std::any::type_name::<conflict_inspection::ConflictSummary>();
}
