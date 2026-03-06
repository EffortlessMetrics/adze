// Property tests for parser_v4 ParseNode and ParserState
#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::parser_v4::*;

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use ir::*;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// ParseError tests
// ---------------------------------------------------------------------------

#[test]
fn parse_error_no_language_display() {
    let err = ParseError::NoLanguage;
    let msg = format!("{}", err);
    assert!(msg.contains("No language"));
}

#[test]
fn parse_error_lexer_debug() {
    let err = ParseError::LexerError("bad token".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("LexerError"));
}

#[test]
fn parse_error_parser_error_display() {
    let err = ParseError::ParserError("syntax error".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("syntax error"));
}

#[test]
fn parse_error_invalid_action() {
    let err = ParseError::InvalidAction("bad action".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("bad action"));
}

#[test]
fn parse_error_unexpected_token() {
    let err = ParseError::UnexpectedToken {
        expected: vec!["foo".to_string()],
        got: "bar".to_string(),
    };
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

// ---------------------------------------------------------------------------
// ParseNode tests
// ---------------------------------------------------------------------------

fn make_node(sym: u16, start: usize, end: usize) -> ParseNode {
    ParseNode {
        symbol: SymbolId(sym),
        symbol_id: SymbolId(sym),
        start_byte: start,
        end_byte: end,
        field_name: None,
        children: vec![],
    }
}

#[test]
fn parse_node_leaf() {
    let node = make_node(1, 0, 5);
    assert_eq!(node.symbol, SymbolId(1));
    assert!(node.children.is_empty());
}

#[test]
fn parse_node_with_children() {
    let child1 = make_node(2, 0, 2);
    let child2 = make_node(3, 3, 5);
    let parent = ParseNode {
        symbol: SymbolId(1),
        symbol_id: SymbolId(1),
        start_byte: 0,
        end_byte: 5,
        field_name: None,
        children: vec![child1, child2],
    };
    assert_eq!(parent.children.len(), 2);
}

#[test]
fn parse_node_with_field_name() {
    let node = ParseNode {
        symbol: SymbolId(5),
        symbol_id: SymbolId(5),
        start_byte: 0,
        end_byte: 10,
        field_name: Some("value".to_string()),
        children: vec![],
    };
    assert_eq!(node.field_name.as_deref(), Some("value"));
}

#[test]
fn parse_node_clone() {
    let node = make_node(10, 5, 15);
    let cloned = node.clone();
    assert_eq!(cloned.symbol, node.symbol);
    assert_eq!(cloned.start_byte, node.start_byte);
    assert_eq!(cloned.end_byte, node.end_byte);
}

#[test]
fn parse_node_debug() {
    let node = make_node(1, 0, 1);
    let debug = format!("{:?}", node);
    assert!(debug.contains("ParseNode"));
}

// ---------------------------------------------------------------------------
// ParseNode property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn parse_node_byte_range_preserved(start in 0usize..1000, len in 0usize..1000) {
        let end = start + len;
        let node = make_node(0, start, end);
        prop_assert_eq!(node.start_byte, start);
        prop_assert_eq!(node.end_byte, end);
        prop_assert!(node.end_byte >= node.start_byte);
    }

    #[test]
    fn parse_node_symbol_id_preserved(id in 0u16..1000) {
        let node = make_node(id, 0, 1);
        prop_assert_eq!(node.symbol, SymbolId(id));
        prop_assert_eq!(node.symbol_id, SymbolId(id));
    }

    #[test]
    fn parse_node_child_count_matches(n in 0usize..10) {
        let children: Vec<ParseNode> = (0..n)
            .map(|i| make_node(i as u16 + 10, i * 2, i * 2 + 1))
            .collect();
        let parent = ParseNode {
            symbol: SymbolId(1),
            symbol_id: SymbolId(1),
            start_byte: 0,
            end_byte: n * 2,
            field_name: None,
            children,
        };
        prop_assert_eq!(parent.children.len(), n);
    }

    #[test]
    fn parse_node_clone_preserves_all(id in 0u16..1000, start in 0usize..1000, len in 0usize..1000) {
        let end = start + len;
        let node = make_node(id, start, end);
        let cloned = node.clone();
        prop_assert_eq!(cloned.symbol, node.symbol);
        prop_assert_eq!(cloned.symbol_id, node.symbol_id);
        prop_assert_eq!(cloned.start_byte, node.start_byte);
        prop_assert_eq!(cloned.end_byte, node.end_byte);
        prop_assert_eq!(cloned.field_name, node.field_name);
    }

    #[test]
    fn parse_node_field_name_clone(name in "[a-z]{1,20}") {
        let node = ParseNode {
            symbol: SymbolId(1),
            symbol_id: SymbolId(1),
            start_byte: 0,
            end_byte: 5,
            field_name: Some(name.clone()),
            children: vec![],
        };
        let cloned = node.clone();
        prop_assert_eq!(cloned.field_name.as_deref(), Some(name.as_str()));
    }
}

// ---------------------------------------------------------------------------
// ParserState tests
// ---------------------------------------------------------------------------

#[test]
fn parser_state_empty_stack() {
    let state = ParserState {
        stack: vec![],
        position: 0,
    };
    assert!(state.stack.is_empty());
    assert_eq!(state.position, 0);
}

#[test]
fn parser_state_with_stack() {
    let state = ParserState {
        stack: vec![(StateId(0), None)],
        position: 5,
    };
    assert_eq!(state.stack.len(), 1);
    assert_eq!(state.position, 5);
}

#[test]
fn parser_state_clone() {
    let state = ParserState {
        stack: vec![(StateId(0), None), (StateId(1), Some(make_node(1, 0, 3)))],
        position: 10,
    };
    let cloned = state.clone();
    assert_eq!(cloned.stack.len(), 2);
    assert_eq!(cloned.position, 10);
}

#[test]
fn parser_state_debug() {
    let state = ParserState {
        stack: vec![],
        position: 0,
    };
    let debug = format!("{:?}", state);
    assert!(debug.contains("ParserState"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn parser_state_position_preserved(pos in 0usize..10000) {
        let state = ParserState {
            stack: vec![],
            position: pos,
        };
        prop_assert_eq!(state.position, pos);
    }

    #[test]
    fn parser_state_stack_size(n in 0usize..20) {
        let stack: Vec<(StateId, Option<ParseNode>)> = (0..n)
            .map(|i| (StateId(i as u16), None))
            .collect();
        let state = ParserState {
            stack,
            position: 0,
        };
        prop_assert_eq!(state.stack.len(), n);
    }
}

// ---------------------------------------------------------------------------
// Nested ParseNode tree tests
// ---------------------------------------------------------------------------

#[test]
fn parse_node_deep_nesting() {
    let mut current = make_node(100, 0, 1);
    for i in (0..10).rev() {
        current = ParseNode {
            symbol: SymbolId(i),
            symbol_id: SymbolId(i),
            start_byte: 0,
            end_byte: 10 - i as usize,
            field_name: None,
            children: vec![current],
        };
    }
    assert_eq!(current.symbol, SymbolId(0));
    assert_eq!(current.children.len(), 1);
}

#[test]
fn parse_node_wide_tree() {
    let children: Vec<ParseNode> = (0..50)
        .map(|i| make_node(i + 10, i as usize * 2, (i as usize + 1) * 2))
        .collect();
    let root = ParseNode {
        symbol: SymbolId(1),
        symbol_id: SymbolId(1),
        start_byte: 0,
        end_byte: 100,
        field_name: None,
        children,
    };
    assert_eq!(root.children.len(), 50);
}
