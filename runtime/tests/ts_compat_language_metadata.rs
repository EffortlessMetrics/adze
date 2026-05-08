//! Tree-sitter compatibility language node-kind metadata canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::{adze_ir::SymbolId, ts_compat::Parser};

fn symbol_named(lang: &adze::ts_compat::Language, name: &str) -> SymbolId {
    let metadata = lang
        .table
        .symbol_metadata
        .iter()
        .find(|metadata| metadata.name == name)
        .unwrap_or_else(|| panic!("arithmetic fixture should expose '{name}' symbol metadata"));
    metadata.symbol_id
}

#[test]
fn language_exposes_node_kind_metadata_for_generated_arithmetic() {
    let lang = adze_example::ts_langs::arithmetic();
    let source_file = symbol_named(&lang, "source_file");
    let expression = symbol_named(&lang, "expression");
    let minus = symbol_named(&lang, "-");
    let number = symbol_named(&lang, "number");

    assert_eq!(lang.node_kind_count(), lang.table.symbol_count);
    assert_eq!(lang.table.symbol_metadata.len(), lang.table.symbol_count);

    for (symbol, expected_kind) in [
        (source_file, "source_file"),
        (expression, "expression"),
        (minus, "-"),
        (number, "number"),
    ] {
        let metadata = &lang.table.symbol_metadata[symbol.0 as usize];
        assert_eq!(metadata.symbol_id, symbol);
        assert_eq!(lang.node_kind_for_id(symbol.0), Some(expected_kind));
        assert_eq!(lang.node_kind_is_named(symbol.0), metadata.is_named);
        assert_eq!(lang.node_kind_is_visible(symbol.0), metadata.is_visible);
        assert_eq!(lang.node_kind_is_supertype(symbol.0), metadata.is_supertype);
        assert_eq!(
            lang.id_for_node_kind(expected_kind, metadata.is_named),
            symbol.0
        );
    }

    assert_eq!(lang.id_for_node_kind("not_a_node_kind", true), 0);
    assert_eq!(lang.node_kind_for_id(u16::MAX), None);
    assert!(!lang.node_kind_is_named(u16::MAX));
    assert!(!lang.node_kind_is_visible(u16::MAX));
    assert!(!lang.node_kind_is_supertype(u16::MAX));
}

#[test]
fn parsed_nodes_resolve_kinds_through_tree_language_metadata() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let tree = parser.parse("1-2", None).expect("Parse failed");
    let root = tree.root_node();
    let expression = root.child(0).expect("root should expose expression child");
    let operator = expression
        .child(1)
        .expect("expression should expose operator child");

    for node in [root, expression, operator] {
        assert_eq!(
            tree.language().node_kind_for_id(node.kind_id()),
            Some(node.kind())
        );
        assert_eq!(
            tree.language()
                .id_for_node_kind(node.kind(), node.is_named()),
            node.kind_id()
        );
    }
}
