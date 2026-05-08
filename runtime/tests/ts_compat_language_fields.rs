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

    let left_id = tree
        .language()
        .field_id_for_name("left")
        .expect("left field should have a public field id");
    let operator_id = expression
        .field_id_for_child(1)
        .expect("operator field should have a public field id");
    let right_id = tree
        .language()
        .field_id_for_name("right")
        .expect("right field should have a public field id");

    assert_eq!(
        expression
            .child_by_field_id(left_id.get())
            .expect("left field id should resolve")
            .text(source.as_bytes()),
        "1"
    );
    assert_eq!(
        expression
            .child_by_field_id(operator_id.get())
            .expect("operator field id should resolve")
            .text(source.as_bytes()),
        "-"
    );
    assert_eq!(
        expression
            .child_by_field_id(right_id.get())
            .expect("right field id should resolve")
            .text(source.as_bytes()),
        "2"
    );
    assert!(
        expression.child_by_field_id(0).is_none(),
        "zero field id should not resolve to a child"
    );
    assert!(
        expression.child_by_field_id(99).is_none(),
        "out-of-range public field ids should not resolve to a child"
    );

    assert_eq!(
        tree.language().field_name_for_id(operator_id.get()),
        Some("operator")
    );
}

/// Field name → field ID → field name roundtrip must be lossless for every
/// registered field.
#[test]
fn field_name_to_id_roundtrip_is_lossless() {
    let lang = arithmetic_with_fields();

    let expected_fields = &["left", "operator", "right"];

    for &name in expected_fields {
        let id = lang
            .field_id_for_name(name)
            .unwrap_or_else(|| panic!("field_id_for_name({name:?}) should return Some"));
        let roundtrip_name = lang
            .field_name_for_id(id.get())
            .unwrap_or_else(|| panic!("field_name_for_id({}) should return Some", id.get()));
        assert_eq!(
            roundtrip_name,
            name,
            "field name roundtrip failed: {name:?} → id={} → {roundtrip_name:?}",
            id.get()
        );
    }
}

/// Field ID → field name → field ID roundtrip must be lossless for every
/// valid 1-based field ID.
#[test]
fn field_id_to_name_roundtrip_is_lossless() {
    let lang = arithmetic_with_fields();
    let count = lang.field_count();

    for raw_id in 1..=count {
        let name = lang
            .field_name_for_id(raw_id as u16)
            .unwrap_or_else(|| panic!("field_name_for_id({raw_id}) should return Some"));
        let roundtrip_id = lang
            .field_id_for_name(name)
            .unwrap_or_else(|| panic!("field_id_for_name({name:?}) should return Some"));
        assert_eq!(
            roundtrip_id.get(),
            raw_id as u16,
            "field id roundtrip failed: id={raw_id} → name={name:?} → id={}",
            roundtrip_id.get()
        );
    }
}

/// Unknown or out-of-range field IDs must return `None` (not panic).
#[test]
fn unknown_field_ids_return_none() {
    let lang = arithmetic_with_fields();

    // ID 0 is the Tree-sitter sentinel and must not resolve
    assert_eq!(lang.field_name_for_id(0), None);

    // IDs beyond the field count must not resolve
    let count = lang.field_count() as u16;
    assert_eq!(lang.field_name_for_id(count + 1), None);
    assert_eq!(lang.field_name_for_id(u16::MAX), None);
}

/// Unknown field names must return `None` (not panic).
#[test]
fn unknown_field_names_return_none() {
    let lang = arithmetic_with_fields();

    assert!(lang.field_id_for_name("").is_none());
    assert!(lang.field_id_for_name("nonexistent").is_none());
    assert!(
        lang.field_id_for_name("LEFT").is_none(),
        "field lookup is case-sensitive"
    );
    assert!(
        lang.field_id_for_name("left ").is_none(),
        "field lookup must not trim whitespace"
    );
}

/// Field lookups through the parsed tree (via node methods) use the same
/// ID space as the Language-level methods, ensuring no mapping divergence.
#[test]
fn node_field_ids_match_language_field_ids() {
    let lang = Arc::new(arithmetic_with_fields());
    let mut parser = Parser::new();
    parser
        .set_language(Arc::clone(&lang))
        .expect("Failed to set language");

    let tree = parser.parse("1-2", None).expect("Parse failed");
    let expression = tree
        .root_node()
        .child(0)
        .expect("root should expose expression child");

    // For each child with a field name, verify the field id from the node
    // matches the field id from the language
    for i in 0..expression.child_count() {
        if let Some(field_name) = expression.field_name_for_child(i) {
            let node_field_id = expression
                .field_id_for_child(i)
                .expect("child with field_name should have a field_id");
            let lang_field_id = lang
                .field_id_for_name(field_name)
                .expect("field_name from node should resolve via language");

            assert_eq!(
                node_field_id, lang_field_id,
                "node field_id ({node_field_id}) must match language field_id ({lang_field_id}) \
                 for field {field_name:?} at child index {i}"
            );
        }
    }
}
