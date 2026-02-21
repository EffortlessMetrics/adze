use adze_ir::builder::GrammarBuilder;
use adze_testing::{BetaTester, TestConfig};
use std::fs;
use tempfile::tempdir;

#[test]
fn grammar_loads_and_generates_parse_table() {
    // Build a simple grammar and write it to disk as JSON
    let grammar = GrammarBuilder::new("test")
        .token("NUMBER", r"\\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let dir = tempdir().unwrap();
    let grammar_path = dir.path().join("grammar.json");
    fs::write(&grammar_path, serde_json::to_string(&grammar).unwrap()).unwrap();

    let config = TestConfig {
        grammar_path: grammar_path.clone(),
        test_files: vec![],
        tree_sitter_path: None,
        compare_output: false,
        benchmark: false,
        external_scanner: None,
    };

    let tester = BetaTester::new(config);
    let loaded = tester.load_grammar(&grammar_path).expect("grammar loads");
    let table = tester
        .generate_parse_table(&loaded)
        .expect("parse table generation");

    assert!(table.state_count > 0);
    assert!(!table.action_table.is_empty());
}
