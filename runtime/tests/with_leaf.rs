#![cfg(feature = "tree-sitter-standard")]

use rust_sitter::{Extract, WithLeaf};
use tree_sitter_json::LANGUAGE;
use tree_sitter_runtime_standard::Parser;

#[test]
fn missing_leaf_fn_returns_none_ts() {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let source = b"1";
    let tree = parser
        .parse(std::str::from_utf8(source).unwrap(), None)
        .unwrap();
    let node = tree.root_node().child(0).unwrap();
    let result: Option<i32> = <WithLeaf<i32> as Extract<_>>::extract(Some(node), source, 0, None);
    assert!(result.is_none());
}

#[test]
fn invalid_utf8_returns_none_ts() {
    let mut parser = Parser::new();
    parser.set_language(&LANGUAGE.into()).unwrap();
    let tree = parser.parse("1", None).unwrap();
    let node = tree.root_node().child(0).unwrap();
    let bad_source = [0xff];
    let result: Option<String> = <WithLeaf<String> as Extract<_>>::extract(
        Some(node),
        &bad_source,
        0,
        Some(&|s| s.to_string()),
    );
    assert!(result.is_none());
}
