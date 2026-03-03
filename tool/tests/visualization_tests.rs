//! Tests for the grammar visualization module.

use adze_tool::{GrammarConverter, GrammarVisualizer};

fn sample_visualizer() -> GrammarVisualizer {
    let grammar = GrammarConverter::create_sample_grammar();
    GrammarVisualizer::new(grammar)
}

#[test]
fn visualizer_to_dot_non_empty() {
    let viz = sample_visualizer();
    let dot = viz.to_dot();
    assert!(!dot.is_empty(), "DOT output should not be empty");
    assert!(
        dot.contains("digraph"),
        "DOT output should contain 'digraph'"
    );
}

#[test]
fn visualizer_to_text_non_empty() {
    let viz = sample_visualizer();
    let text = viz.to_text();
    assert!(!text.is_empty(), "text output should not be empty");
}

#[test]
fn visualizer_to_railroad_svg_non_empty() {
    let viz = sample_visualizer();
    let svg = viz.to_railroad_svg();
    assert!(!svg.is_empty(), "SVG output should not be empty");
}

#[test]
fn visualizer_dependency_graph_non_empty() {
    let viz = sample_visualizer();
    let graph = viz.dependency_graph();
    assert!(!graph.is_empty(), "dependency graph should not be empty");
}
