//! Basic tests to verify the runtime compiles and has the expected API

use rust_sitter_runtime::{language::SymbolMetadata, Language, Parser, Tree};

#[cfg(feature = "glr-core")]
fn empty_parse_table() -> &'static rust_sitter_glr_core::ParseTable {
    use rust_sitter_glr_core::{GotoIndexing, ParseTable};
    use rust_sitter_ir::{Grammar, StateId, SymbolId};
    use std::collections::BTreeMap;

    Box::leak(Box::new(ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 0,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(0),
        grammar: Grammar::new("stub".to_string()),
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }))
}

#[cfg(not(feature = "glr-core"))]
fn empty_parse_table() -> rust_sitter_runtime::language::ParseTable {
    rust_sitter_runtime::language::ParseTable {
        state_count: 0,
        action_table: vec![],
        small_parse_table: None,
        small_parse_table_map: None,
    }
}

fn stub_language() -> Language {
    let table = empty_parse_table();
    let builder = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["placeholder".into()])
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .field_names(vec![]);

    #[cfg(feature = "glr-core")]
    let builder = builder.tokenizer(|_| Box::new(std::iter::empty()));

    builder.build().unwrap()
}

#[test]
fn can_create_parser() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn can_set_language() {
    let mut parser = Parser::new();
    let language = stub_language();
    parser.set_language(language).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn parse_requires_language() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("test input", None);
    assert!(result.is_err());
}

#[test]
fn can_access_tree_nodes() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.kind(), "placeholder");
    assert_eq!(root.child_count(), 0);
}

#[test]
fn node_text_extraction() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let source = b"test source";
    let text = root.utf8_text(source).unwrap();
    assert_eq!(text, ""); // Empty because stub node has 0..0 range
}

#[test]
fn error_display() {
    use rust_sitter_runtime::ParseError;

    let error = ParseError::no_language();
    assert_eq!(error.to_string(), "no language set");

    let error = ParseError::timeout();
    assert_eq!(error.to_string(), "parse timeout exceeded");
}

#[cfg(feature = "external-scanners")]
#[test]
fn external_scanner_trait() {
    use rust_sitter_runtime::external_scanner::{ExternalScanner, ScanResult};

    struct TestScanner;

    impl ExternalScanner for TestScanner {
        fn init(&mut self) {}

        fn scan(&mut self, _valid_symbols: &[bool], _input: &[u8]) -> Option<ScanResult> {
            None
        }

        fn serialize(&self) -> Vec<u8> {
            vec![]
        }

        fn deserialize(&mut self, _data: &[u8]) {}
    }

    let mut scanner = TestScanner;
    scanner.init();
    assert!(scanner.scan(&[], b"test").is_none());
}
