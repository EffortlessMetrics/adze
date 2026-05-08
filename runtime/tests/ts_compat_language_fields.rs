//! Tree-sitter compatibility language field metadata canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::{
    adze_ir::RuleId,
    ts_compat::{Language, Parser},
};
use std::sync::Arc;

fn arithmetic_with_fields() -> Language {
    let mut lang = (*adze_example::ts_langs::arithmetic()).clone();
    lang.table.field_names = vec![
        "left".to_string(),
        "operator".to_string(),
        "right".to_string(),
    ];
    lang.table.field_map.insert((RuleId(2), 0), 0);
    lang.table.field_map.insert((RuleId(2), 1), 1);
    lang.table.field_map.insert((RuleId(2), 2), 2);
    lang
}

#[test]
fn language_exposes_tree_sitter_shaped_field_metadata() {
    let lang = arithmetic_with_fields();

    assert_eq!(lang.field_count(), 3);
    assert_eq!(lang.field_name_for_id(0), None);
    assert_eq!(lang.field_name_for_id(1), Some("left"));
    assert_eq!(lang.field_name_for_id(2), Some("operator"));
    assert_eq!(lang.field_name_for_id(3), Some("right"));
    assert_eq!(lang.field_name_for_id(4), None);

    assert_eq!(lang.field_id_for_name("left").map(|id| id.get()), Some(1));
    assert_eq!(
        lang.field_id_for_name("operator").map(|id| id.get()),
        Some(2)
    );
    assert_eq!(lang.field_id_for_name(b"right").map(|id| id.get()), Some(3));
    assert_eq!(lang.field_id_for_name("missing"), None);
}

#[test]
fn language_field_ids_are_metadata_ids_not_internal_field_map_indexes() {
    let lang = Arc::new(arithmetic_with_fields());
    let mut parser = Parser::new();
    parser
        .set_language(Arc::clone(&lang))
        .expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let expression = tree
        .root_node()
        .child(0)
        .expect("root should expose expression child");

    assert_eq!(
        tree.language().field_id_for_name("left").map(|id| id.get()),
        Some(1)
    );
    assert_eq!(
        tree.language()
            .field_id_for_name("operator")
            .map(|id| id.get()),
        Some(2)
    );
    assert_eq!(
        tree.language()
            .field_id_for_name("right")
            .map(|id| id.get()),
        Some(3)
    );

    assert_eq!(expression.field_name_for_child(0), Some("left"));
    assert_eq!(expression.field_name_for_child(1), Some("operator"));
    assert_eq!(expression.field_name_for_child(2), Some("right"));
    assert_eq!(expression.field_id_for_child(0).map(|id| id.get()), Some(1));
    assert_eq!(expression.field_id_for_child(1).map(|id| id.get()), Some(2));
    assert_eq!(expression.field_id_for_child(2).map(|id| id.get()), Some(3));
    assert_eq!(expression.field_id_for_child(3), None);
    assert_eq!(
        expression
            .child_by_field_name("operator")
            .expect("operator field should resolve")
            .text(source.as_bytes()),
        "-"
    );

    let operator_id = expression
        .field_id_for_child(1)
        .expect("operator field should have a public field id");
    assert_eq!(
        tree.language().field_name_for_id(operator_id.get()),
        Some("operator")
    );
}
