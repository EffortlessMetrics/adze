//! Tree-sitter compatibility node metadata canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::{
    adze_ir::SymbolId,
    ts_compat::{Language, Parser},
};
use std::sync::Arc;

fn symbol_named(lang: &Language, name: &str) -> SymbolId {
    let index = lang
        .table
        .symbol_metadata
        .iter()
        .position(|metadata| metadata.name == name)
        .unwrap_or_else(|| panic!("arithmetic fixture should expose '{name}' symbol metadata"));
    let symbol = lang.table.symbol_metadata[index].symbol_id;
    assert_eq!(
        symbol.0 as usize, index,
        "arithmetic fixture metadata should be indexed by symbol id"
    );
    symbol
}

fn minus_symbol(lang: &Language) -> SymbolId {
    symbol_named(lang, "-")
}

#[test]
fn generated_tree_exposes_tree_and_node_language_metadata() {
    let mut lang = (*adze_example::ts_langs::arithmetic()).clone();
    lang.name = "arithmetic_language_probe".to_string();

    let mut parser = Parser::new();
    parser
        .set_language(Arc::new(lang))
        .expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();
    let expression = root.child(0).expect("root should expose expression child");
    let operator = expression
        .child(1)
        .expect("expression should expose operator child");

    assert_eq!(tree.language().name, "arithmetic_language_probe");
    assert!(std::ptr::eq(tree.language(), root.language()));
    assert!(std::ptr::eq(tree.language(), expression.language()));
    assert!(std::ptr::eq(tree.language(), operator.language()));

    let source_file_symbol = symbol_named(tree.language(), "source_file");
    let minus_symbol = symbol_named(operator.language(), "-");
    assert_eq!(root.kind_id(), source_file_symbol.0);
    assert_eq!(operator.kind_id(), minus_symbol.0);
}

#[test]
fn generated_tree_exposes_node_kind_ids() {
    let lang = adze_example::ts_langs::arithmetic();
    let source_file_symbol = symbol_named(&lang, "source_file");
    let expression_symbol = symbol_named(&lang, "expression");
    let minus_symbol = symbol_named(&lang, "-");
    let number_symbol = symbol_named(&lang, "number");

    let mut parser = Parser::new();
    parser.set_language(lang).expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();
    let expression = root.child(0).expect("root should expose expression child");
    let left = expression
        .child(0)
        .expect("expression should expose left child");
    let operator = expression
        .child(1)
        .expect("expression should expose operator child");
    let number = left.child(0).expect("left expression should expose number");

    assert_eq!(root.kind(), "source_file");
    assert_eq!(root.kind_id(), source_file_symbol.0);
    assert_eq!(expression.kind(), "expression");
    assert_eq!(expression.kind_id(), expression_symbol.0);
    assert_eq!(operator.kind(), "-");
    assert_eq!(operator.kind_id(), minus_symbol.0);
    assert_eq!(number.kind(), "number");
    assert_eq!(number.kind_id(), number_symbol.0);
}

#[test]
fn generated_tree_exposes_node_grammar_metadata() {
    let lang = adze_example::ts_langs::arithmetic();

    let mut parser = Parser::new();
    parser.set_language(lang).expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();
    let expression = root.child(0).expect("root should expose expression child");
    let left = expression
        .child(0)
        .expect("expression should expose left child");
    let operator = expression
        .child(1)
        .expect("expression should expose operator child");
    let number = left.child(0).expect("left expression should expose number");

    for (node, expected_kind) in [
        (root, "source_file"),
        (expression, "expression"),
        (operator, "-"),
        (number, "number"),
    ] {
        assert_eq!(node.grammar_id(), node.kind_id());
        assert_eq!(node.grammar_name(), node.kind());
        assert_eq!(node.grammar_name(), expected_kind);
    }
}

#[test]
fn generated_tree_exposes_extra_node_metadata() {
    let mut lang = (*adze_example::ts_langs::arithmetic()).clone();
    let operator_symbol = minus_symbol(&lang);
    lang.table.symbol_metadata[operator_symbol.0 as usize].is_extra = true;

    let mut parser = Parser::new();
    parser
        .set_language(Arc::new(lang))
        .expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();
    let expression = root.child(0).expect("root should expose expression child");
    let left = expression
        .child(0)
        .expect("expression should expose left child");
    let operator = expression
        .child(1)
        .expect("expression should expose operator child");
    let right = expression
        .child(2)
        .expect("expression should expose right child");

    assert_eq!(operator.kind(), "-");
    assert!(!operator.is_named());
    assert!(operator.is_extra());

    assert!(!root.is_extra());
    assert!(!expression.is_extra());
    assert!(!left.is_extra());
    assert!(!right.is_extra());
}

#[test]
fn generated_tree_extra_node_metadata_falls_back_to_extras_list() {
    let mut lang = (*adze_example::ts_langs::arithmetic()).clone();
    let operator_symbol = minus_symbol(&lang);
    lang.table.extras = vec![operator_symbol];
    lang.table
        .symbol_metadata
        .truncate(operator_symbol.0 as usize);

    let mut parser = Parser::new();
    parser
        .set_language(Arc::new(lang))
        .expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();
    let expression = root.child(0).expect("root should expose expression child");
    let operator = expression
        .child(1)
        .expect("expression should expose operator child");

    assert_eq!(operator.kind(), "-");
    assert!(operator.is_extra());
    assert!(!root.is_extra());
    assert!(!expression.is_extra());
}
