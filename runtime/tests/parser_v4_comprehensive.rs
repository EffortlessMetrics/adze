//! Comprehensive tests for parser_v4 module.
//!
//! Tests Parser construction, arena metrics, ParseNode, ParseError, ParserState.

#[cfg(feature = "ts-compat")]
use adze::adze_glr_core as glr_core;
#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::parser_v4::*;

#[cfg(not(feature = "ts-compat"))]
use adze_glr_core as glr_core;
#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use glr_core::{FirstFollowSets, StateId, build_lr1_automaton};
use ir::SymbolId;
use ir::builder::GrammarBuilder;

fn make_parser() -> Parser {
    let mut grammar = GrammarBuilder::new("test")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "num"])
        .start("expr")
        .build();
    grammar.normalize();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).expect("ff");
    let pt = build_lr1_automaton(&grammar, &ff).expect("lr1");
    Parser::new(grammar, pt, "test".to_string())
}

fn make_simple_parser() -> Parser {
    let mut grammar = GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    grammar.normalize();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).expect("ff");
    let pt = build_lr1_automaton(&grammar, &ff).expect("lr1");
    Parser::new(grammar, pt, "simple".to_string())
}

// ── 1. Parser construction ──────────────────────────────────────

#[test]
fn test_parser_new() {
    let p = make_parser();
    let _ = p;
}

#[test]
fn test_parser_simple() {
    let p = make_simple_parser();
    let _ = p;
}

#[test]
fn test_parser_grammar_accessible() {
    let p = make_parser();
    let g = p.grammar();
    assert_eq!(g.name, "test");
}

#[test]
fn test_parser_parse_table_accessible() {
    let p = make_parser();
    let pt = p.parse_table();
    assert!(pt.state_count > 0);
}

// ── 2. Arena metrics ────────────────────────────────────────────

#[test]
fn test_initial_arena_metrics() {
    let p = make_parser();
    let metrics = p.arena_metrics();
    assert_eq!(metrics.len(), 0);
}

// ── 3. with_arena_capacity ──────────────────────────────────────

#[test]
fn test_with_arena_capacity() {
    let mut grammar = GrammarBuilder::new("cap")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    grammar.normalize();
    let ff = FirstFollowSets::compute_normalized(&mut grammar).expect("ff");
    let pt = build_lr1_automaton(&grammar, &ff).expect("lr1");
    let p = Parser::with_arena_capacity(grammar, pt, "cap".to_string(), 4096);
    let metrics = p.arena_metrics();
    assert_eq!(metrics.len(), 0);
}

// ── 4. parse_tree ───────────────────────────────────────────────

#[test]
fn test_parse_tree_simple() {
    let mut p = make_simple_parser();
    let result = p.parse_tree("a");
    let _ = result;
}

#[test]
fn test_parse_tree_with_error_count() {
    let mut p = make_simple_parser();
    let result = p.parse_tree_with_error_count("a");
    let _ = result;
}

// ── 5. ParseNode struct ─────────────────────────────────────────

#[test]
fn test_parse_node_construction() {
    let node = ParseNode {
        symbol: SymbolId(1),
        symbol_id: SymbolId(1),
        start_byte: 0,
        end_byte: 5,
        field_name: None,
        children: Vec::new(),
    };
    assert_eq!(node.symbol, SymbolId(1));
    assert_eq!(node.start_byte, 0);
    assert_eq!(node.end_byte, 5);
    assert!(node.field_name.is_none());
}

#[test]
fn test_parse_node_with_children() {
    let child = ParseNode {
        symbol: SymbolId(2),
        symbol_id: SymbolId(2),
        start_byte: 0,
        end_byte: 3,
        field_name: None,
        children: Vec::new(),
    };
    let parent = ParseNode {
        symbol: SymbolId(1),
        symbol_id: SymbolId(1),
        start_byte: 0,
        end_byte: 3,
        field_name: None,
        children: vec![child],
    };
    assert_eq!(parent.children.len(), 1);
    assert_eq!(parent.children[0].symbol, SymbolId(2));
}

#[test]
fn test_parse_node_with_field_name() {
    let node = ParseNode {
        symbol: SymbolId(1),
        symbol_id: SymbolId(1),
        start_byte: 0,
        end_byte: 3,
        field_name: Some("value".to_string()),
        children: Vec::new(),
    };
    assert_eq!(node.field_name.as_deref(), Some("value"));
}

#[test]
fn test_parse_node_clone() {
    let node = ParseNode {
        symbol: SymbolId(5),
        symbol_id: SymbolId(5),
        start_byte: 10,
        end_byte: 20,
        field_name: Some("test".to_string()),
        children: Vec::new(),
    };
    let cloned = node.clone();
    assert_eq!(cloned.symbol, node.symbol);
    assert_eq!(cloned.start_byte, node.start_byte);
}

#[test]
fn test_parse_node_debug() {
    let node = ParseNode {
        symbol: SymbolId(1),
        symbol_id: SymbolId(1),
        start_byte: 0,
        end_byte: 1,
        field_name: None,
        children: Vec::new(),
    };
    let debug = format!("{:?}", node);
    assert!(debug.contains("ParseNode"));
}

// ── 6. ParseError enum ─────────────────────────────────────────

#[test]
fn test_parse_error_no_language() {
    let err = ParseError::NoLanguage;
    let msg = format!("{}", err);
    assert!(msg.to_lowercase().contains("language"), "got: {}", msg);
}

#[test]
fn test_parse_error_lexer_error() {
    let err = ParseError::LexerError("bad token".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("bad token"));
}

#[test]
fn test_parse_error_parser_error() {
    let err = ParseError::ParserError("syntax error".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("syntax error"));
}

#[test]
fn test_parse_error_invalid_action() {
    let err = ParseError::InvalidAction("unknown action".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("unknown action"));
}

#[test]
fn test_parse_error_unexpected_token() {
    let err = ParseError::UnexpectedToken {
        expected: vec!["num".to_string()],
        got: "plus".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("plus"));
}

#[test]
fn test_parse_error_debug() {
    let err = ParseError::NoLanguage;
    let debug = format!("{:?}", err);
    assert!(debug.contains("NoLanguage"));
}

// ── 7. ParserState ──────────────────────────────────────────────

#[test]
fn test_parser_state_empty_stack() {
    let state = ParserState {
        stack: Vec::new(),
        position: 0,
    };
    assert!(state.stack.is_empty());
    assert_eq!(state.position, 0);
}

#[test]
fn test_parser_state_with_items() {
    let state = ParserState {
        stack: vec![(StateId(0), None), (StateId(1), None)],
        position: 5,
    };
    assert_eq!(state.stack.len(), 2);
    assert_eq!(state.position, 5);
}

#[test]
fn test_parser_state_clone() {
    let state = ParserState {
        stack: vec![(StateId(0), None)],
        position: 10,
    };
    let cloned = state.clone();
    assert_eq!(cloned.position, state.position);
    assert_eq!(cloned.stack.len(), state.stack.len());
}

// ── 8. Deep parse tree ──────────────────────────────────────────

#[test]
fn test_deep_parse_node_tree() {
    let mut node = ParseNode {
        symbol: SymbolId(10),
        symbol_id: SymbolId(10),
        start_byte: 0,
        end_byte: 1,
        field_name: None,
        children: Vec::new(),
    };
    for i in (0..10u16).rev() {
        node = ParseNode {
            symbol: SymbolId(i),
            symbol_id: SymbolId(i),
            start_byte: 0,
            end_byte: 1,
            field_name: None,
            children: vec![node],
        };
    }
    assert_eq!(node.symbol, SymbolId(0));
    let mut depth = 0;
    let mut cur = &node;
    while !cur.children.is_empty() {
        cur = &cur.children[0];
        depth += 1;
    }
    assert_eq!(depth, 10);
}

// ── 9. Multiple parses ──────────────────────────────────────────

#[test]
fn test_multiple_parses_same_parser() {
    let mut p = make_simple_parser();
    let _ = p.parse_tree("a");
    let _ = p.parse_tree("a");
}

// ── 10. Wide parse tree ─────────────────────────────────────────

#[test]
fn test_wide_parse_node_tree() {
    let children: Vec<ParseNode> = (0..20u16)
        .map(|i| ParseNode {
            symbol: SymbolId(i),
            symbol_id: SymbolId(i),
            start_byte: i as usize,
            end_byte: (i + 1) as usize,
            field_name: None,
            children: Vec::new(),
        })
        .collect();
    let parent = ParseNode {
        symbol: SymbolId(100),
        symbol_id: SymbolId(100),
        start_byte: 0,
        end_byte: 20,
        field_name: None,
        children,
    };
    assert_eq!(parent.children.len(), 20);
}
