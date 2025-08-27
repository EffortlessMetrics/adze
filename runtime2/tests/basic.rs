//! Basic tests to verify the runtime compiles and has the expected API

use rust_sitter_runtime::{Language, Parser, Tree};

#[test]
fn can_create_parser() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
#[cfg_attr(feature = "glr-core", ignore)]
fn can_set_language() {
    let mut parser = Parser::new();
    let language = Language::new_stub();
    assert!(parser.set_language(language).is_err());
    assert!(parser.language().is_none());
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
