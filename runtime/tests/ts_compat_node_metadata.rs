//! Tree-sitter compatibility node metadata canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::{
    adze_ir::SymbolId,
    ts_compat::{Language, Parser},
};
use std::sync::Arc;

fn minus_symbol(lang: &Language) -> SymbolId {
    let index = lang
        .table
        .symbol_metadata
        .iter()
        .position(|metadata| metadata.name == "-")
        .expect("arithmetic fixture should expose '-' symbol metadata");
    let symbol = lang.table.symbol_metadata[index].symbol_id;
    assert_eq!(
        symbol.0 as usize, index,
        "arithmetic fixture metadata should be indexed by symbol id"
    );
    symbol
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
