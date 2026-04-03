//! Contract lock test - verifies that public API remains stable.

use adze_bdd_grammar_analysis_core::{
    ConflictAnalysis, analyze_conflicts, count_multi_action_cells, resolve_shift_reduce_actions,
};

/// Verify all public types exist and have expected structure.
#[test]
fn test_contract_lock_types() {
    // Verify ConflictAnalysis struct exists with expected fields
    let analysis = ConflictAnalysis {
        total_conflicts: 0,
        shift_reduce_conflicts: 0,
        reduce_reduce_conflicts: 0,
        conflict_details: vec![],
    };

    // Verify fields are accessible
    assert_eq!(analysis.total_conflicts, 0);
    assert_eq!(analysis.shift_reduce_conflicts, 0);
    assert_eq!(analysis.reduce_reduce_conflicts, 0);
    assert!(analysis.conflict_details.is_empty());

    // Verify Debug trait is implemented
    let _debug_str = format!("{analysis:?}");

    // Verify Clone trait is implemented
    let _cloned = analysis.clone();
}

/// Verify all public functions exist with expected signatures.
#[test]
fn test_contract_lock_functions() {
    use adze_glr_core::{Action, ParseTable};
    type ResolveShiftReduceFn =
        fn(&adze_glr_core::Grammar, adze_glr_core::SymbolId, adze_glr_core::RuleId) -> Vec<Action>;

    // Verify count_multi_action_cells function exists
    let pt = ParseTable::default();
    let _count = count_multi_action_cells(&pt);

    // Verify analyze_conflicts function exists
    let _analysis = analyze_conflicts(&pt);

    // Verify resolve_shift_reduce_actions function exists (requires Grammar, SymbolId, RuleId)
    // This function is tested indirectly through its signature availability
    let _fn_ptr: Option<ResolveShiftReduceFn> = Some(resolve_shift_reduce_actions);
}
