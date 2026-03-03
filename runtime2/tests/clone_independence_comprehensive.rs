#![allow(clippy::needless_range_loop)]

use adze_runtime::error::ErrorLocation;
use adze_runtime::external_scanner::ScanResult;
use adze_runtime::language::{Action, ParseTable, SymbolMetadata};
use adze_runtime::node::Point;
use adze_runtime::test_helpers::{multi_symbol_test_language, stub_language};
use adze_runtime::token::Token;
use adze_runtime::tree::Tree;
use adze_runtime::{InputEdit, Parser};
use std::time::Duration;

// ===== Tree clone tests =====

#[test]
fn tree_clone_preserves_root_kind() {
    let tree = Tree::new_for_testing(42, 0, 10, vec![]);
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), cloned.root_kind());
}

#[test]
fn tree_clone_preserves_byte_range() {
    let tree = Tree::new_for_testing(1, 5, 20, vec![]);
    let cloned = tree.clone();
    let orig = tree.root_node();
    let copy = cloned.root_node();
    assert_eq!(orig.start_byte(), copy.start_byte());
    assert_eq!(orig.end_byte(), copy.end_byte());
}

#[test]
fn tree_clone_preserves_children_count() {
    let child1 = Tree::new_for_testing(1, 0, 3, vec![]);
    let child2 = Tree::new_for_testing(2, 3, 6, vec![]);
    let tree = Tree::new_for_testing(0, 0, 6, vec![child1, child2]);
    let cloned = tree.clone();
    assert_eq!(tree.root_node().child_count(), cloned.root_node().child_count());
    assert_eq!(cloned.root_node().child_count(), 2);
}

#[test]
fn tree_clone_preserves_child_structure() {
    let grandchild = Tree::new_for_testing(3, 0, 2, vec![]);
    let child = Tree::new_for_testing(2, 0, 5, vec![grandchild]);
    let tree = Tree::new_for_testing(1, 0, 10, vec![child]);
    let cloned = tree.clone();

    let orig_child = tree.root_node().child(0).unwrap();
    let clone_child = cloned.root_node().child(0).unwrap();
    assert_eq!(orig_child.kind_id(), clone_child.kind_id());
    assert_eq!(orig_child.child_count(), clone_child.child_count());

    let orig_gc = orig_child.child(0).unwrap();
    let clone_gc = clone_child.child(0).unwrap();
    assert_eq!(orig_gc.kind_id(), clone_gc.kind_id());
    assert_eq!(orig_gc.start_byte(), clone_gc.start_byte());
}

#[test]
fn tree_clone_preserves_none_language() {
    let tree = Tree::new_for_testing(0, 0, 5, vec![]);
    let cloned = tree.clone();
    assert!(tree.language().is_none());
    assert!(cloned.language().is_none());
}

#[test]
fn tree_clone_stub_preserves_structure() {
    let tree = Tree::new_stub();
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), 0);
    assert_eq!(cloned.root_kind(), 0);
    assert_eq!(tree.root_node().child_count(), 0);
    assert_eq!(cloned.root_node().child_count(), 0);
}

#[test]
fn tree_clone_deeply_nested_preserves_structure() {
    let leaf = Tree::new_for_testing(5, 0, 1, vec![]);
    let level4 = Tree::new_for_testing(4, 0, 2, vec![leaf]);
    let level3 = Tree::new_for_testing(3, 0, 3, vec![level4]);
    let level2 = Tree::new_for_testing(2, 0, 4, vec![level3]);
    let tree = Tree::new_for_testing(1, 0, 5, vec![level2]);
    let cloned = tree.clone();

    let mut orig = tree.root_node();
    let mut copy = cloned.root_node();
    for expected_kind in [1u16, 2, 3, 4, 5] {
        assert_eq!(orig.kind_id(), expected_kind);
        assert_eq!(copy.kind_id(), expected_kind);
        if expected_kind < 5 {
            orig = orig.child(0).unwrap();
            copy = copy.child(0).unwrap();
        }
    }
}

#[test]
fn tree_clone_many_children() {
    let children: Vec<Tree> = (0..20)
        .map(|i| Tree::new_for_testing(i + 1, i as usize * 5, (i as usize + 1) * 5, vec![]))
        .collect();
    let tree = Tree::new_for_testing(0, 0, 100, children);
    let cloned = tree.clone();

    assert_eq!(tree.root_node().child_count(), 20);
    assert_eq!(cloned.root_node().child_count(), 20);
    for i in 0..20 {
        let orig_child = tree.root_node().child(i).unwrap();
        let clone_child = cloned.root_node().child(i).unwrap();
        assert_eq!(orig_child.kind_id(), clone_child.kind_id());
        assert_eq!(orig_child.byte_range(), clone_child.byte_range());
    }
}

// ===== Language clone tests =====

#[test]
fn language_clone_preserves_version() {
    let lang = stub_language();
    let cloned = lang.clone();
    assert_eq!(lang.version, cloned.version);
}

#[test]
fn language_clone_preserves_symbol_count() {
    let lang = multi_symbol_test_language(5);
    let cloned = lang.clone();
    assert_eq!(lang.symbol_count, cloned.symbol_count);
    assert_eq!(cloned.symbol_count, 5);
}

#[test]
fn language_clone_preserves_symbol_names() {
    let lang = multi_symbol_test_language(3);
    let cloned = lang.clone();
    assert_eq!(lang.symbol_names.len(), cloned.symbol_names.len());
    for i in 0..lang.symbol_names.len() {
        assert_eq!(lang.symbol_names[i], cloned.symbol_names[i]);
    }
}

#[test]
fn language_clone_preserves_symbol_metadata() {
    let lang = stub_language();
    let cloned = lang.clone();
    assert_eq!(lang.symbol_metadata.len(), cloned.symbol_metadata.len());
    for i in 0..lang.symbol_metadata.len() {
        assert_eq!(
            lang.symbol_metadata[i].is_terminal,
            cloned.symbol_metadata[i].is_terminal
        );
        assert_eq!(
            lang.symbol_metadata[i].is_visible,
            cloned.symbol_metadata[i].is_visible
        );
        assert_eq!(
            lang.symbol_metadata[i].is_supertype,
            cloned.symbol_metadata[i].is_supertype
        );
    }
}

#[test]
fn language_clone_preserves_field_names() {
    let lang = stub_language();
    let cloned = lang.clone();
    assert_eq!(lang.field_names.len(), cloned.field_names.len());
    assert_eq!(lang.field_count, cloned.field_count);
}

#[test]
fn language_clone_symbol_names_independent() {
    let lang = multi_symbol_test_language(3);
    let mut cloned = lang.clone();
    cloned.symbol_names[0] = "modified".to_string();
    assert_ne!(lang.symbol_names[0], cloned.symbol_names[0]);
    assert_eq!(lang.symbol_names[0], "symbol_0");
}

// ===== Token clone tests =====

#[test]
fn token_clone_preserves_all_fields() {
    let token = Token {
        kind: 42,
        start: 10,
        end: 20,
    };
    let cloned = token.clone();
    assert_eq!(token.kind, cloned.kind);
    assert_eq!(token.start, cloned.start);
    assert_eq!(token.end, cloned.end);
}

#[test]
fn token_clone_independent_modification() {
    let token = Token {
        kind: 1,
        start: 0,
        end: 5,
    };
    let mut cloned = token;
    cloned.kind = 99;
    cloned.start = 100;
    cloned.end = 200;
    assert_eq!(token.kind, 1);
    assert_eq!(token.start, 0);
    assert_eq!(token.end, 5);
}

#[test]
fn token_copy_semantic() {
    let token = Token {
        kind: 7,
        start: 3,
        end: 8,
    };
    let copied = token;
    let _still_valid = token; // Copy trait: original still usable
    assert_eq!(copied.kind, 7);
    assert_eq!(token.kind, 7);
}

// ===== SymbolMetadata clone tests =====

#[test]
fn symbol_metadata_clone_preserves_fields() {
    let meta = SymbolMetadata {
        is_terminal: true,
        is_visible: false,
        is_supertype: true,
    };
    let cloned = meta.clone();
    assert_eq!(meta.is_terminal, cloned.is_terminal);
    assert_eq!(meta.is_visible, cloned.is_visible);
    assert_eq!(meta.is_supertype, cloned.is_supertype);
}

#[test]
fn symbol_metadata_clone_independent_modification() {
    let meta = SymbolMetadata {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    };
    let mut cloned = meta;
    cloned.is_terminal = false;
    cloned.is_visible = false;
    cloned.is_supertype = true;
    assert!(meta.is_terminal);
    assert!(meta.is_visible);
    assert!(!meta.is_supertype);
}

#[test]
fn symbol_metadata_copy_semantic() {
    let meta = SymbolMetadata {
        is_terminal: false,
        is_visible: true,
        is_supertype: false,
    };
    let copied = meta;
    let _still_valid = meta;
    assert!(copied.is_visible);
}

// ===== Point clone tests =====

#[test]
fn point_clone_preserves_fields() {
    let point = Point::new(10, 25);
    let cloned = point.clone();
    assert_eq!(point.row, cloned.row);
    assert_eq!(point.column, cloned.column);
}

#[test]
fn point_clone_independent_modification() {
    let point = Point::new(5, 15);
    let mut cloned = point;
    cloned.row = 99;
    cloned.column = 200;
    assert_eq!(point.row, 5);
    assert_eq!(point.column, 15);
}

#[test]
fn point_copy_semantic() {
    let point = Point::new(3, 7);
    let copied = point;
    let _still_valid = point;
    assert_eq!(copied, point);
}

// ===== ScanResult clone tests =====

#[test]
fn scan_result_clone_preserves_fields() {
    let result = ScanResult {
        token_type: 42,
        bytes_consumed: 10,
    };
    let cloned = result.clone();
    assert_eq!(result, cloned);
}

#[test]
fn scan_result_copy_independent() {
    let result = ScanResult {
        token_type: 1,
        bytes_consumed: 5,
    };
    let mut cloned = result;
    cloned.token_type = 99;
    assert_eq!(result.token_type, 1);
    assert_eq!(cloned.token_type, 99);
}

// ===== InputEdit clone tests =====

#[test]
fn input_edit_clone_preserves_fields() {
    let edit = InputEdit {
        start_byte: 10,
        old_end_byte: 20,
        new_end_byte: 25,
        start_position: Point::new(1, 10),
        old_end_position: Point::new(1, 20),
        new_end_position: Point::new(1, 25),
    };
    let cloned = edit.clone();
    assert_eq!(edit, cloned);
}

#[test]
fn input_edit_copy_independent() {
    let edit = InputEdit {
        start_byte: 0,
        old_end_byte: 5,
        new_end_byte: 10,
        start_position: Point::new(0, 0),
        old_end_position: Point::new(0, 5),
        new_end_position: Point::new(0, 10),
    };
    let mut cloned = edit;
    cloned.start_byte = 100;
    assert_eq!(edit.start_byte, 0);
    assert_eq!(cloned.start_byte, 100);
}

// ===== ErrorLocation clone tests =====

#[test]
fn error_location_clone_preserves_fields() {
    let loc = ErrorLocation {
        byte_offset: 42,
        line: 3,
        column: 7,
    };
    let cloned = loc.clone();
    assert_eq!(loc, cloned);
}

#[test]
fn error_location_clone_independent() {
    let loc = ErrorLocation {
        byte_offset: 10,
        line: 1,
        column: 5,
    };
    let mut cloned = loc.clone();
    cloned.byte_offset = 999;
    cloned.line = 100;
    cloned.column = 200;
    assert_eq!(loc.byte_offset, 10);
    assert_eq!(loc.line, 1);
    assert_eq!(loc.column, 5);
}

// ===== Action clone tests =====

#[test]
fn action_clone_all_variants() {
    let shift = Action::Shift(42);
    assert_eq!(shift, shift.clone());

    let reduce = Action::Reduce {
        symbol: 5,
        child_count: 3,
    };
    assert_eq!(reduce, reduce.clone());

    assert_eq!(Action::Accept, Action::Accept.clone());
    assert_eq!(Action::Error, Action::Error.clone());
}

#[test]
fn action_copy_independent() {
    let action = Action::Shift(10);
    let mut cloned = action;
    cloned = Action::Shift(99);
    assert_eq!(action, Action::Shift(10));
    assert_eq!(cloned, Action::Shift(99));
}

// ===== ParseTable (language module) clone tests =====

#[test]
fn parse_table_clone_preserves_fields() {
    let table = ParseTable {
        state_count: 5,
        action_table: vec![vec![vec![Action::Shift(1)]]],
        small_parse_table: Some(vec![1, 2, 3]),
        small_parse_table_map: Some(vec![0, 1]),
    };
    let cloned = table.clone();
    assert_eq!(table.state_count, cloned.state_count);
    assert_eq!(table.action_table.len(), cloned.action_table.len());
    assert_eq!(table.small_parse_table, cloned.small_parse_table);
    assert_eq!(table.small_parse_table_map, cloned.small_parse_table_map);
}

#[test]
fn parse_table_clone_independent() {
    let table = ParseTable {
        state_count: 3,
        action_table: vec![vec![vec![Action::Accept]]],
        small_parse_table: None,
        small_parse_table_map: None,
    };
    let mut cloned = table.clone();
    cloned.state_count = 100;
    cloned.action_table.push(vec![]);
    assert_eq!(table.state_count, 3);
    assert_eq!(table.action_table.len(), 1);
}

// ===== Parser independence tests =====

#[test]
fn parser_instances_independent() {
    let mut parser1 = Parser::new();
    let parser2 = Parser::new();
    parser1.set_timeout(Duration::from_secs(5));
    assert_eq!(parser1.timeout(), Some(Duration::from_secs(5)));
    assert_eq!(parser2.timeout(), None);
}

#[test]
fn parser_language_independent() {
    let mut parser1 = Parser::new();
    let parser2 = Parser::new();
    let lang = stub_language();
    parser1.set_language(lang).unwrap();
    assert!(parser1.language().is_some());
    assert!(parser2.language().is_none());
}
