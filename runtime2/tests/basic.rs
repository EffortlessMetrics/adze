//! Basic tests to verify the runtime compiles and has the expected API

use adze_runtime::{
    Parser, Tree,
    language::{Language, SymbolMetadata},
    test_helpers::stub_language,
};

fn leak_parse_table() -> &'static adze_glr_core::ParseTable {
    Box::leak(Box::new(adze_glr_core::ParseTable::default()))
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
fn smoke_language_builder_constructs_minimal_language() {
    let language = Language::builder()
        .version(15)
        .parse_table(leak_parse_table())
        .symbol_names(vec!["end".into(), "source".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .field_names(vec!["body".into()])
        .build()
        .expect("builder should construct a language with parse table + metadata");

    assert_eq!(language.version, 15);
    assert_eq!(language.symbol_name(1), Some("source"));
    assert_eq!(language.field_name(0), Some("body"));
    assert!(language.is_visible(1));
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
    // Phase 3.3: Node::kind() returns "unknown" when language is not set
    assert_eq!(root.kind(), "unknown");
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
    use adze_runtime::ParseError;

    let error = ParseError::no_language();
    assert_eq!(error.to_string(), "no language set");

    let error = ParseError::timeout();
    assert_eq!(error.to_string(), "parse timeout exceeded");
}

#[cfg(feature = "external_scanners")]
#[test]
fn external_scanner_trait() {
    use adze_runtime::external_scanner::{ExternalScanner, ScanResult};

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
