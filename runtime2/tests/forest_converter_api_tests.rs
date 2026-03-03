//! Tests for the forest converter disambiguation strategies and error types.
#![cfg(feature = "pure-rust")]

use adze_runtime::forest_converter::*;
use adze_runtime::glr_engine::{ForestNode, ForestNodeId, ParseForest};

fn empty_forest() -> ParseForest {
    ParseForest {
        nodes: vec![],
        roots: vec![],
    }
}

fn single_node_forest() -> ParseForest {
    use adze_ir::SymbolId;
    ParseForest {
        nodes: vec![ForestNode {
            symbol: SymbolId(1),
            children: vec![],
            range: 0..5,
        }],
        roots: vec![ForestNodeId(0)],
    }
}

#[test]
fn converter_prefer_shift() {
    let converter = ForestConverter::new(DisambiguationStrategy::PreferShift);
    let forest = empty_forest();
    let result = converter.to_tree(&forest, b"hello");
    // Empty forest has no roots
    assert!(result.is_err());
}

#[test]
fn converter_prefer_reduce() {
    let converter = ForestConverter::new(DisambiguationStrategy::PreferReduce);
    let forest = empty_forest();
    let result = converter.to_tree(&forest, b"hello");
    assert!(result.is_err());
}

#[test]
fn converter_precedence() {
    let converter = ForestConverter::new(DisambiguationStrategy::Precedence);
    let forest = empty_forest();
    let result = converter.to_tree(&forest, b"hello");
    assert!(result.is_err());
}

#[test]
fn converter_first() {
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let forest = empty_forest();
    let result = converter.to_tree(&forest, b"hello");
    assert!(result.is_err());
}

#[test]
fn converter_reject_ambiguity() {
    let converter = ForestConverter::new(DisambiguationStrategy::RejectAmbiguity);
    let forest = empty_forest();
    let result = converter.to_tree(&forest, b"hello");
    assert!(result.is_err());
}

#[test]
fn detect_ambiguity_empty_forest() {
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let forest = empty_forest();
    let ambiguity = converter.detect_ambiguity(&forest);
    assert!(ambiguity.is_none());
}

#[test]
fn detect_ambiguity_single_node() {
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let forest = single_node_forest();
    let ambiguity = converter.detect_ambiguity(&forest);
    assert!(ambiguity.is_none());
}

#[test]
fn conversion_error_debug() {
    let err = ConversionError::NoRoots;
    let debug = format!("{err:?}");
    assert!(debug.contains("NoRoots"));
}

#[test]
fn disambiguation_strategy_debug() {
    let strategies = [
        DisambiguationStrategy::PreferShift,
        DisambiguationStrategy::PreferReduce,
        DisambiguationStrategy::Precedence,
        DisambiguationStrategy::First,
        DisambiguationStrategy::RejectAmbiguity,
    ];
    for strategy in &strategies {
        let debug = format!("{strategy:?}");
        assert!(!debug.is_empty());
    }
}

#[test]
fn single_node_converts_to_tree() {
    let converter = ForestConverter::new(DisambiguationStrategy::First);
    let forest = single_node_forest();
    let result = converter.to_tree(&forest, b"hello");
    assert!(
        result.is_ok(),
        "single node forest should convert: {result:?}"
    );
}
