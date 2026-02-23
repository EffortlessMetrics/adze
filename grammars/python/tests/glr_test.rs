#[test]
fn test_glr_state_0_fix() {
    // Validate generated table invariants relevant to state-0 dispatch.
    let language = &adze_python::grammar_python::LANGUAGE;

    assert!(
        language.state_count > 0,
        "Python grammar must have at least one parser state",
    );
    assert!(
        !language.parse_actions.is_null(),
        "Parse actions table must be populated",
    );
    assert!(
        !adze_python::grammar_python::SMALL_PARSE_TABLE_MAP.is_empty(),
        "Small parse-table map must contain a state-0 entry",
    );

    let state_0_offset = adze_python::grammar_python::SMALL_PARSE_TABLE_MAP[0] as usize;
    assert!(
        state_0_offset < adze_python::grammar_python::SMALL_PARSE_TABLE.len(),
        "State-0 parse-table offset must be within SMALL_PARSE_TABLE bounds",
    );
}
