//! Basic tests to verify the runtime compiles and has the expected API

use adze_runtime::{Parser, Tree, test_helpers::stub_language};

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
fn language_smoke_exposes_metadata_queries() {
    let language = stub_language();

    assert_eq!(language.symbol_count, 1);
    assert_eq!(language.field_count, 0);

    assert_eq!(language.symbol_name(0), Some("placeholder"));
    assert_eq!(language.symbol_for_name("placeholder", true), Some(0));
    assert!(language.is_terminal(0));
    assert!(language.is_visible(0));
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
