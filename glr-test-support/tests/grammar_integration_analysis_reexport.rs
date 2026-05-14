use glr_test_support as fixtures;

#[test]
fn given_facade_and_owner_module_when_analyzing_the_same_table_results_match() {
    // Given
    let table = fixtures::build_runtime_dangling_else_parse_table()
        .expect("dangling-else parse-table should build");

    // When
    let facade_analysis = fixtures::analyze_conflicts(&table);
    let facade_count = fixtures::count_multi_action_cells(&table);
    let core_analysis = fixtures::grammar_fixtures::analysis::analyze_conflicts(&table);
    let core_count = fixtures::grammar_fixtures::analysis::count_multi_action_cells(&table);

    // Then
    assert_eq!(
        facade_analysis.total_conflicts,
        core_analysis.total_conflicts
    );
    assert_eq!(
        facade_analysis.shift_reduce_conflicts,
        core_analysis.shift_reduce_conflicts
    );
    assert_eq!(
        facade_analysis.reduce_reduce_conflicts,
        core_analysis.reduce_reduce_conflicts
    );
    assert_eq!(
        facade_analysis.conflict_details,
        core_analysis.conflict_details
    );
    assert_eq!(facade_count, core_count);
}
