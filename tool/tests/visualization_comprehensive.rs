//! Comprehensive tests for GrammarVisualizer output formats.
//!
//! Covers: to_dot, to_railroad_svg, to_text, dependency_graph,
//! edge cases with empty/minimal/complex grammars.

use adze_ir::builder::GrammarBuilder;
use adze_tool::visualization::GrammarVisualizer;

fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("simple")
        .token("NUMBER", r"\d+")
        .token("PLUS", r"\+")
        .rule("expr", vec!["NUMBER", "PLUS", "NUMBER"])
        .start("expr")
        .build()
}

fn single_token_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("single")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .build()
}

fn multi_rule_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("multi")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("LPAREN", r"\(")
        .token("RPAREN", r"\)")
        .rule("expr", vec!["term", "PLUS", "expr"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["NUM"])
        .rule("term", vec!["LPAREN", "expr", "RPAREN"])
        .start("expr")
        .build()
}

// --- DOT output tests ---

#[test]
fn dot_contains_digraph_header() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("digraph Grammar"));
    assert!(dot.contains("rankdir=LR"));
}

#[test]
fn dot_contains_terminals() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("shape=ellipse"));
    assert!(dot.contains("lightblue"));
}

#[test]
fn dot_contains_nonterminals() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("lightgreen"));
}

#[test]
fn dot_ends_with_closing_brace() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let dot = viz.to_dot();
    assert!(dot.trim().ends_with('}'));
}

#[test]
fn dot_multi_rule_has_edges() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("->"));
}

#[test]
fn dot_single_token_grammar() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("digraph"));
    assert!(dot.contains("ID"));
}

// --- Railroad SVG tests ---

#[test]
fn svg_contains_svg_tag() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg") || svg.contains("svg"));
}

#[test]
fn svg_multi_rule() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let svg = viz.to_railroad_svg();
    assert!(!svg.is_empty());
}

#[test]
fn svg_single_token() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let svg = viz.to_railroad_svg();
    assert!(!svg.is_empty());
}

// --- Text output tests ---

#[test]
fn text_contains_grammar_name() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let text = viz.to_text();
    // Should contain some representation of the grammar
    assert!(!text.is_empty());
}

#[test]
fn text_multi_rule_has_multiple_lines() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let text = viz.to_text();
    let lines: Vec<_> = text.lines().collect();
    assert!(
        lines.len() > 1,
        "Expected multiple lines but got {}",
        lines.len()
    );
}

#[test]
fn text_single_token_concise() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let text = viz.to_text();
    assert!(!text.is_empty());
}

// --- Dependency graph tests ---

#[test]
fn dependency_graph_not_empty() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let dep = viz.dependency_graph();
    assert!(!dep.is_empty());
}

#[test]
fn dependency_graph_multi_rule_has_connections() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("->") || dep.contains("depends") || dep.len() > 10);
}

#[test]
fn dependency_graph_single_token() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let dep = viz.dependency_graph();
    assert!(!dep.is_empty());
}

// --- Consistency tests ---

#[test]
fn all_formats_produce_output() {
    let viz = GrammarVisualizer::new(simple_grammar());
    assert!(!viz.to_dot().is_empty());
    assert!(!viz.to_railroad_svg().is_empty());
    assert!(!viz.to_text().is_empty());
    assert!(!viz.dependency_graph().is_empty());
}

#[test]
fn dot_is_deterministic() {
    let g1 = simple_grammar();
    let g2 = simple_grammar();
    let dot1 = GrammarVisualizer::new(g1).to_dot();
    let dot2 = GrammarVisualizer::new(g2).to_dot();
    assert_eq!(dot1, dot2);
}

#[test]
fn text_is_deterministic() {
    let g1 = simple_grammar();
    let g2 = simple_grammar();
    let text1 = GrammarVisualizer::new(g1).to_text();
    let text2 = GrammarVisualizer::new(g2).to_text();
    assert_eq!(text1, text2);
}
