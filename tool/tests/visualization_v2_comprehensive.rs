//! Comprehensive tests for the `adze_tool::visualization` module.
//!
//! Covers: simple/complex grammars, output formats (DOT, text, SVG, dependency),
//! grammar shapes, edge cases, determinism, multiple grammars, and large grammars.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ExternalToken, Grammar, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tool::GrammarVisualizer;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUMBER", "+", "NUMBER"])
        .start("expr")
        .build()
}

fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "/", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["(", "expr", ")"])
        .start("expr")
        .build()
}

fn empty_grammar() -> Grammar {
    Grammar::new("empty".to_string())
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("ID", r"[a-z]+")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["ID"])
        .start("a")
        .build()
}

fn multi_alternative_grammar() -> Grammar {
    GrammarBuilder::new("multi_alt")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .rule("start", vec!["C"])
        .rule("start", vec!["D"])
        .start("start")
        .build()
}

// ---------------------------------------------------------------------------
// 1. Simple grammar visualization
// ---------------------------------------------------------------------------

#[test]
fn simple_grammar_to_text_contains_name() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let text = viz.to_text();
    assert!(text.contains("Grammar: simple"));
}

#[test]
fn simple_grammar_to_text_lists_tokens() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let text = viz.to_text();
    assert!(text.contains("NUMBER"));
    assert!(text.contains("+"));
}

#[test]
fn simple_grammar_to_text_lists_rules() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let text = viz.to_text();
    assert!(text.contains("::="));
}

#[test]
fn simple_grammar_to_dot_is_valid_digraph() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let dot = viz.to_dot();
    assert!(dot.starts_with("digraph Grammar {"));
    assert!(dot.trim_end().ends_with('}'));
}

#[test]
fn simple_grammar_dependency_graph_header() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("Symbol Dependencies:"));
}

// ---------------------------------------------------------------------------
// 2. Complex grammar visualization
// ---------------------------------------------------------------------------

#[test]
fn arithmetic_grammar_text_contains_all_operators() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let text = viz.to_text();
    for op in &["+", "-", "*", "/"] {
        assert!(text.contains(op), "missing operator {op}");
    }
}

#[test]
fn arithmetic_grammar_text_shows_precedence() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let text = viz.to_text();
    assert!(text.contains("[precedence:"));
}

#[test]
fn arithmetic_grammar_text_shows_associativity() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let text = viz.to_text();
    assert!(text.contains("[associativity:"));
}

#[test]
fn arithmetic_grammar_dot_has_terminal_nodes() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("shape=ellipse"));
    assert!(dot.contains("fillcolor=lightblue"));
}

#[test]
fn arithmetic_grammar_dot_has_nonterminal_nodes() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("fillcolor=lightgreen"));
}

// ---------------------------------------------------------------------------
// 3. Output format correctness
// ---------------------------------------------------------------------------

#[test]
fn dot_output_contains_rankdir() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("rankdir=LR"));
}

#[test]
fn dot_output_contains_edges() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("->"), "DOT output should contain edge arrows");
}

#[test]
fn svg_output_contains_svg_tag() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn svg_output_contains_style() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<style>"));
    assert!(svg.contains(".terminal"));
    assert!(svg.contains(".non-terminal"));
}

#[test]
fn svg_output_contains_rule_name_class() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("class=\"rule-name\""));
}

#[test]
fn text_output_sections_separator() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let text = viz.to_text();
    assert!(text.contains("=".repeat(50).as_str()));
}

#[test]
fn text_output_tokens_section() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let text = viz.to_text();
    assert!(text.contains("Tokens:"));
}

#[test]
fn text_output_rules_section() {
    let viz = GrammarVisualizer::new(simple_grammar());
    let text = viz.to_text();
    assert!(text.contains("Rules:"));
}

// ---------------------------------------------------------------------------
// 4. Different grammar shapes
// ---------------------------------------------------------------------------

#[test]
fn chain_grammar_dependency_graph_shows_chain() {
    let viz = GrammarVisualizer::new(chain_grammar());
    let dep = viz.dependency_graph();
    // a depends on b, b depends on c
    assert!(dep.contains("depends on:"));
}

#[test]
fn chain_grammar_dot_has_multiple_edges() {
    let viz = GrammarVisualizer::new(chain_grammar());
    let dot = viz.to_dot();
    let edge_count = dot.matches("->").count();
    assert!(edge_count >= 2, "chain should have at least 2 edges");
}

#[test]
fn multi_alternative_grammar_text_has_multiple_rules() {
    let viz = GrammarVisualizer::new(multi_alternative_grammar());
    let text = viz.to_text();
    let rule_lines = text.lines().filter(|l| l.contains("::=")).count();
    assert!(
        rule_lines >= 4,
        "expected at least 4 rule lines, got {rule_lines}"
    );
}

#[test]
fn nullable_grammar_text_contains_epsilon() {
    let grammar = GrammarBuilder::new("nullable")
        .token("X", "x")
        .rule("start", vec![])
        .rule("start", vec!["X"])
        .start("start")
        .build();
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains('ε'), "nullable rule should show epsilon");
}

#[test]
fn python_like_grammar_text_shows_externals() {
    let grammar = GrammarBuilder::python_like();
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("External Tokens:"));
}

#[test]
fn python_like_grammar_dot_has_external_nodes() {
    let grammar = GrammarBuilder::python_like();
    let viz = GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    assert!(dot.contains("shape=diamond"), "externals should be diamond");
    assert!(dot.contains("fillcolor=lightcoral"));
}

#[test]
fn javascript_like_grammar_text_shows_all_sections() {
    let grammar = GrammarBuilder::javascript_like();
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("Tokens:"));
    assert!(text.contains("Rules:"));
    assert!(text.contains("[precedence:"));
    assert!(text.contains("[associativity:"));
}

// ---------------------------------------------------------------------------
// 5. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_grammar_to_text_contains_name() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let text = viz.to_text();
    assert!(text.contains("Grammar: empty"));
}

#[test]
fn empty_grammar_to_dot_is_valid() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("digraph Grammar"));
    assert!(dot.trim_end().ends_with('}'));
}

#[test]
fn empty_grammar_dependency_graph_no_crash() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("Symbol Dependencies:"));
}

#[test]
fn empty_grammar_svg_is_valid() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn single_token_grammar_text() {
    let mut grammar = Grammar::new("single_token".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "ONLY".to_string(),
            pattern: TokenPattern::String("only".to_string()),
            fragile: false,
        },
    );
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("ONLY"));
}

#[test]
fn grammar_with_special_chars_in_token_name() {
    let grammar = GrammarBuilder::new("special")
        .token("&&", "&&")
        .token("||", "||")
        .rule("logical", vec!["&&"])
        .start("logical")
        .build();
    let viz = GrammarVisualizer::new(grammar);
    // DOT should escape properly
    let dot = viz.to_dot();
    assert!(dot.contains("digraph Grammar"));
    // SVG should XML-escape
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("&amp;&amp;"));
}

#[test]
fn grammar_with_regex_tokens_text() {
    let grammar = GrammarBuilder::new("regex_toks")
        .token("FLOAT", r"\d+\.\d+")
        .rule("val", vec!["FLOAT"])
        .start("val")
        .build();
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("FLOAT"));
    assert!(text.contains('/'), "regex token should show /pattern/");
}

// ---------------------------------------------------------------------------
// 6. Deterministic output
// ---------------------------------------------------------------------------

#[test]
fn text_output_is_deterministic() {
    let g1 = simple_grammar();
    let g2 = simple_grammar();
    let t1 = GrammarVisualizer::new(g1).to_text();
    let t2 = GrammarVisualizer::new(g2).to_text();
    assert_eq!(t1, t2);
}

#[test]
fn dot_output_is_deterministic() {
    let g1 = simple_grammar();
    let g2 = simple_grammar();
    let d1 = GrammarVisualizer::new(g1).to_dot();
    let d2 = GrammarVisualizer::new(g2).to_dot();
    assert_eq!(d1, d2);
}

#[test]
fn svg_output_is_deterministic() {
    let g1 = simple_grammar();
    let g2 = simple_grammar();
    let s1 = GrammarVisualizer::new(g1).to_railroad_svg();
    let s2 = GrammarVisualizer::new(g2).to_railroad_svg();
    assert_eq!(s1, s2);
}

#[test]
fn dependency_graph_is_deterministic() {
    let g1 = chain_grammar();
    let g2 = chain_grammar();
    let d1 = GrammarVisualizer::new(g1).dependency_graph();
    let d2 = GrammarVisualizer::new(g2).dependency_graph();
    assert_eq!(d1, d2);
}

#[test]
fn arithmetic_text_deterministic_across_calls() {
    let text1 = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    let text2 = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    let text3 = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    assert_eq!(text1, text2);
    assert_eq!(text2, text3);
}

// ---------------------------------------------------------------------------
// 7. Multiple grammars / independence
// ---------------------------------------------------------------------------

#[test]
fn different_grammars_produce_different_text() {
    let t1 = GrammarVisualizer::new(simple_grammar()).to_text();
    let t2 = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    assert_ne!(t1, t2);
}

#[test]
fn different_grammars_produce_different_dot() {
    let d1 = GrammarVisualizer::new(simple_grammar()).to_dot();
    let d2 = GrammarVisualizer::new(chain_grammar()).to_dot();
    assert_ne!(d1, d2);
}

#[test]
fn empty_vs_nonempty_grammar_text_differ() {
    let t1 = GrammarVisualizer::new(empty_grammar()).to_text();
    let t2 = GrammarVisualizer::new(simple_grammar()).to_text();
    assert_ne!(t1, t2);
}

#[test]
fn visualizer_does_not_mutate_grammar() {
    let g = simple_grammar();
    let g_clone = g.clone();
    let viz = GrammarVisualizer::new(g);
    let _ = viz.to_text();
    let _ = viz.to_dot();
    let _ = viz.to_railroad_svg();
    let _ = viz.dependency_graph();
    // Construct another visualizer from the clone and verify same output
    let viz2 = GrammarVisualizer::new(g_clone);
    assert_eq!(viz.to_text(), viz2.to_text());
}

// ---------------------------------------------------------------------------
// 8. Large grammars
// ---------------------------------------------------------------------------

fn large_grammar(rule_count: usize) -> Grammar {
    let mut builder = GrammarBuilder::new("large");
    // Create tokens
    for i in 0..rule_count {
        let tok_name = format!("T{i}");
        let tok_pattern = format!("t{i}");
        builder = builder.token(
            Box::leak(tok_name.into_boxed_str()),
            Box::leak(tok_pattern.into_boxed_str()),
        );
    }
    // Create chain rules: r0 -> T0, r1 -> T1, ... and a top-level that references them
    for i in 0..rule_count {
        let rule_name = format!("r{i}");
        let tok_name = format!("T{i}");
        builder = builder.rule(
            Box::leak(rule_name.into_boxed_str()),
            vec![Box::leak(tok_name.into_boxed_str())],
        );
    }
    builder = builder.start("r0");
    builder.build()
}

#[test]
fn large_grammar_50_rules_text() {
    let viz = GrammarVisualizer::new(large_grammar(50));
    let text = viz.to_text();
    let rule_lines = text.lines().filter(|l| l.contains("::=")).count();
    assert!(
        rule_lines >= 50,
        "expected >=50 rule lines, got {rule_lines}"
    );
}

#[test]
fn large_grammar_50_rules_dot() {
    let viz = GrammarVisualizer::new(large_grammar(50));
    let dot = viz.to_dot();
    assert!(dot.contains("digraph Grammar"));
    // Should have nodes for all 50 tokens
    let terminal_count = dot.matches("shape=ellipse").count();
    assert!(
        terminal_count >= 50,
        "expected >=50 terminal nodes, got {terminal_count}"
    );
}

#[test]
fn large_grammar_100_rules_does_not_crash() {
    let viz = GrammarVisualizer::new(large_grammar(100));
    let _ = viz.to_text();
    let _ = viz.to_dot();
    let _ = viz.to_railroad_svg();
    let _ = viz.dependency_graph();
}

#[test]
fn large_grammar_dependency_graph_has_entries() {
    let viz = GrammarVisualizer::new(large_grammar(20));
    let dep = viz.dependency_graph();
    // Each rule depends on (none) since they only reference terminals
    let none_count = dep.matches("(none)").count();
    assert!(
        none_count >= 20,
        "expected >=20 leaf entries, got {none_count}"
    );
}

// ---------------------------------------------------------------------------
// Additional edge cases & symbol types
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_epsilon_rule_dot() {
    let grammar = GrammarBuilder::new("eps")
        .token("X", "x")
        .rule("start", vec![])
        .start("start")
        .build();
    let viz = GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    // Epsilon transitions are skipped in DOT
    assert!(dot.contains("digraph Grammar"));
}

#[test]
fn grammar_with_external_scanner_dependency_graph() {
    let grammar = GrammarBuilder::new("ext")
        .token("ID", r"[a-z]+")
        .external("INDENT")
        .external("DEDENT")
        .rule("block", vec!["ID"])
        .start("block")
        .build();
    let viz = GrammarVisualizer::new(grammar);
    let dep = viz.dependency_graph();
    assert!(dep.contains("Symbol Dependencies:"));
}

#[test]
fn dot_special_char_escaping() {
    // Build grammar with a token name that has quotes
    let mut grammar = Grammar::new("escape_test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "say\"hello\"".to_string(),
            pattern: TokenPattern::String("say\"hello\"".to_string()),
            fragile: false,
        },
    );
    let viz = GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    // Quotes should be escaped in DOT
    assert!(dot.contains("\\\""), "quotes must be escaped in DOT labels");
}

#[test]
fn svg_xml_entity_escaping() {
    let grammar = GrammarBuilder::new("xml_esc")
        .token("<tag>", "<tag>")
        .rule("elem", vec!["<tag>"])
        .start("elem")
        .build();
    let viz = GrammarVisualizer::new(grammar);
    let svg = viz.to_railroad_svg();
    assert!(
        svg.contains("&lt;tag&gt;"),
        "angle brackets must be XML-escaped in SVG"
    );
}

#[test]
fn text_string_token_uses_quotes() {
    let grammar = GrammarBuilder::new("str_tok")
        .token("HELLO", "hello")
        .rule("greet", vec!["HELLO"])
        .start("greet")
        .build();
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(
        text.contains("\"hello\""),
        "string tokens should be shown with quotes in text output"
    );
}

#[test]
fn text_regex_token_uses_slashes() {
    let grammar = GrammarBuilder::new("re_tok")
        .token("NUM", r"\d+")
        .rule("val", vec!["NUM"])
        .start("val")
        .build();
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(
        text.contains("/\\d+/"),
        "regex tokens should be shown with /pattern/ in text"
    );
}

#[test]
fn precedence_metadata_in_text_for_js_grammar() {
    let grammar = GrammarBuilder::javascript_like();
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    // JS-like grammar has precedence on expression rules
    assert!(text.contains("precedence"));
    assert!(text.contains("associativity"));
}

#[test]
fn multi_alternative_dot_has_edges_for_each() {
    let viz = GrammarVisualizer::new(multi_alternative_grammar());
    let dot = viz.to_dot();
    // 4 alternatives each producing an edge
    let edge_count = dot.matches("->").count();
    assert!(
        edge_count >= 4,
        "expected >=4 edges for 4 alternatives, got {edge_count}"
    );
}
