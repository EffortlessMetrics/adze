//! Tree-sitter compatibility node metadata canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::{
    adze_glr_core::SymbolMetadata,
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

fn arithmetic_with_expression_child_alias(alias_name: &str) -> (Language, SymbolId, SymbolId) {
    let mut lang = (*adze_example::ts_langs::arithmetic()).clone();
    let source_file = symbol_named(&lang, "source_file");
    let expression = symbol_named(&lang, "expression");
    let alias_symbol = SymbolId(lang.table.symbol_metadata.len() as u16);

    lang.table.symbol_metadata.push(SymbolMetadata {
        name: alias_name.to_string(),
        is_visible: true,
        is_named: true,
        is_supertype: false,
        is_terminal: false,
        is_extra: false,
        is_fragile: false,
        symbol_id: alias_symbol,
    });
    lang.table.symbol_count = lang
        .table
        .symbol_count
        .max(lang.table.symbol_metadata.len());
    lang.table
        .index_to_symbol
        .resize(lang.table.symbol_metadata.len(), SymbolId(0));
    lang.table.index_to_symbol[alias_symbol.0 as usize] = alias_symbol;

    let source_file_rule = lang
        .table
        .rules
        .iter()
        .position(|rule| rule.lhs == source_file && rule.rhs_len == 1)
        .expect("arithmetic fixture should reduce source_file from expression");
    lang.table
        .alias_sequences
        .resize_with(source_file_rule + 1, Vec::new);
    lang.table.alias_sequences[source_file_rule] = vec![Some(alias_symbol)];

    (lang, expression, alias_symbol)
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
fn alias_visible_kind_and_grammar_identity_are_distinct() {
    let (lang, expression_symbol, alias_symbol) =
        arithmetic_with_expression_child_alias("binary_expression");

    let mut parser = Parser::new();
    parser
        .set_language(Arc::new(lang))
        .expect("Failed to set language");

    let tree = parser.parse("1-2", None).expect("Parse failed");
    let expression = tree
        .root_node()
        .child(0)
        .expect("root should expose aliased expression child");

    assert_eq!(expression.kind(), "binary_expression");
    assert_eq!(expression.kind_id(), alias_symbol.0);
    assert!(expression.is_named());
    assert_eq!(expression.grammar_name(), "expression");
    assert_eq!(expression.grammar_id(), expression_symbol.0);
    assert_eq!(
        tree.language().id_for_node_kind("binary_expression", true),
        alias_symbol.0
    );
    assert_eq!(
        tree.language().node_kind_for_id(expression.kind_id()),
        Some("binary_expression")
    );
    assert_ne!(expression.kind_id(), expression.grammar_id());
    assert_ne!(expression.kind(), expression.grammar_name());
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
