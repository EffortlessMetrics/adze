//! Comprehensive tests for the grammar visualization module (v3).

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, Grammar, PrecedenceKind,
    ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tool::GrammarVisualizer;

// ---------------------------------------------------------------------------
// Helper builders
// ---------------------------------------------------------------------------

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn two_token_grammar() -> Grammar {
    GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build()
}

fn multi_rule_grammar() -> Grammar {
    GrammarBuilder::new("multi")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn empty_grammar() -> Grammar {
    Grammar::new("empty".to_string())
}

fn grammar_with_externals() -> Grammar {
    GrammarBuilder::new("ext")
        .token("a", "a")
        .external("INDENT")
        .external("DEDENT")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn nullable_start_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("s", vec![])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["d"])
        .rule("d", vec!["x"])
        .start("a")
        .build()
}

fn large_grammar() -> Grammar {
    let mut builder = GrammarBuilder::new("large");
    for i in 0..50 {
        let tok_name: String = format!("t{i}");
        builder = builder.token(&tok_name, &tok_name);
    }
    for i in 0..50 {
        let rule_name = format!("r{i}");
        let tok_name = format!("t{i}");
        builder = builder.rule(&rule_name, vec![&tok_name]);
    }
    builder = builder.start("r0");
    builder.build()
}

fn python_like_grammar() -> Grammar {
    GrammarBuilder::python_like()
}

fn javascript_like_grammar() -> Grammar {
    GrammarBuilder::javascript_like()
}

// ---------------------------------------------------------------------------
// 1–10  to_text basic
// ---------------------------------------------------------------------------

#[test]
fn text_contains_grammar_name_minimal() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let text = viz.to_text();
    assert!(text.contains("Grammar: minimal"));
}

#[test]
fn text_contains_separator_line() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let text = viz.to_text();
    assert!(text.contains("==="));
}

#[test]
fn text_contains_tokens_section() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let text = viz.to_text();
    assert!(text.contains("Tokens:"));
}

#[test]
fn text_contains_rules_section() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let text = viz.to_text();
    assert!(text.contains("Rules:"));
}

#[test]
fn text_lists_token_name() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let text = viz.to_text();
    assert!(text.contains("a"));
}

#[test]
fn text_shows_token_pattern_string() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let text = viz.to_text();
    assert!(text.contains("\"a\""));
}

#[test]
fn text_shows_token_pattern_regex() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let text = viz.to_text();
    // NUM token uses regex
    assert!(text.contains("/"));
}

#[test]
fn text_shows_rule_arrow() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let text = viz.to_text();
    assert!(text.contains("::="));
}

#[test]
fn text_shows_terminal_in_quotes() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let text = viz.to_text();
    assert!(text.contains("'a'"));
}

#[test]
fn text_two_tokens_listed() {
    let viz = GrammarVisualizer::new(two_token_grammar());
    let text = viz.to_text();
    assert!(text.contains("a") && text.contains("b"));
}

// ---------------------------------------------------------------------------
// 11–15  to_text with multiple rules / alternatives
// ---------------------------------------------------------------------------

#[test]
fn text_multiple_alternatives() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let text = viz.to_text();
    // At least two ::= for expr
    let count = text.matches("::=").count();
    assert!(count >= 3, "expected >=3 rules, found {count}");
}

#[test]
fn text_rule_references_nonterminal() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let text = viz.to_text();
    // The recursive expr should mention the non-terminal name
    assert!(text.contains("expr") || text.contains("rule_"));
}

#[test]
fn text_precedence_annotation() {
    let viz = GrammarVisualizer::new(precedence_grammar());
    let text = viz.to_text();
    assert!(text.contains("precedence"));
}

#[test]
fn text_associativity_annotation() {
    let viz = GrammarVisualizer::new(precedence_grammar());
    let text = viz.to_text();
    assert!(text.contains("associativity"));
}

#[test]
fn text_empty_grammar_has_name() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let text = viz.to_text();
    assert!(text.contains("Grammar: empty"));
}

// ---------------------------------------------------------------------------
// 16–20  to_text externals / extras / epsilon
// ---------------------------------------------------------------------------

#[test]
fn text_externals_section_present() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let text = viz.to_text();
    assert!(text.contains("External Tokens:"));
}

#[test]
fn text_external_names_listed() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let text = viz.to_text();
    assert!(text.contains("INDENT"));
    assert!(text.contains("DEDENT"));
}

#[test]
fn text_no_externals_section_when_none() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let text = viz.to_text();
    assert!(!text.contains("External Tokens:"));
}

#[test]
fn text_epsilon_shown() {
    let viz = GrammarVisualizer::new(nullable_start_grammar());
    let text = viz.to_text();
    assert!(text.contains("ε"));
}

#[test]
fn text_nullable_grammar_has_two_rules() {
    let viz = GrammarVisualizer::new(nullable_start_grammar());
    let text = viz.to_text();
    let count = text.matches("::=").count();
    assert!(count >= 2, "expected >=2 rules, found {count}");
}

// ---------------------------------------------------------------------------
// 21–30  to_dot
// ---------------------------------------------------------------------------

#[test]
fn dot_starts_with_digraph() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dot = viz.to_dot();
    assert!(dot.starts_with("digraph Grammar {"));
}

#[test]
fn dot_contains_rankdir() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("rankdir=LR"));
}

#[test]
fn dot_ends_with_closing_brace() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dot = viz.to_dot();
    assert!(dot.trim().ends_with('}'));
}

#[test]
fn dot_terminal_has_ellipse_shape() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("shape=ellipse"));
}

#[test]
fn dot_terminal_filled_lightblue() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("fillcolor=lightblue"));
}

#[test]
fn dot_nonterminal_filled_lightgreen() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("fillcolor=lightgreen"));
}

#[test]
fn dot_has_edge() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("->"));
}

#[test]
fn dot_multi_rule_has_multiple_edges() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let dot = viz.to_dot();
    let edge_count = dot.matches("->").count();
    assert!(edge_count >= 3, "expected >=3 edges, found {edge_count}");
}

#[test]
fn dot_external_uses_diamond() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let dot = viz.to_dot();
    assert!(dot.contains("shape=diamond"));
}

#[test]
fn dot_external_filled_lightcoral() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let dot = viz.to_dot();
    assert!(dot.contains("fillcolor=lightcoral"));
}

// ---------------------------------------------------------------------------
// 31–35  to_dot edge cases
// ---------------------------------------------------------------------------

#[test]
fn dot_empty_grammar_still_valid() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("digraph Grammar"));
    assert!(dot.trim().ends_with('}'));
}

#[test]
fn dot_large_grammar_has_many_nodes() {
    let viz = GrammarVisualizer::new(large_grammar());
    let dot = viz.to_dot();
    // 50 tokens → 50 terminal nodes
    let terminal_count = dot.matches("shape=ellipse").count();
    assert!(
        terminal_count >= 50,
        "expected >=50 terminals, found {terminal_count}"
    );
}

#[test]
fn dot_large_grammar_has_many_nt_nodes() {
    let viz = GrammarVisualizer::new(large_grammar());
    let dot = viz.to_dot();
    let nt_count = dot.matches("fillcolor=lightgreen").count();
    assert!(
        nt_count >= 50,
        "expected >=50 non-terminals, found {nt_count}"
    );
}

#[test]
fn dot_comments_section_terminals() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("// Terminals"));
}

#[test]
fn dot_comments_section_nonterminals() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("// Non-terminals"));
}

// ---------------------------------------------------------------------------
// 36–40  to_railroad_svg
// ---------------------------------------------------------------------------

#[test]
fn svg_contains_svg_tag() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
}

#[test]
fn svg_contains_closing_tag() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("</svg>"));
}

#[test]
fn svg_has_style_section() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<style>"));
}

#[test]
fn svg_has_rule_name_class() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("rule-name"));
}

#[test]
fn svg_contains_rect_element() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<rect"));
}

// ---------------------------------------------------------------------------
// 41–45  dependency_graph
// ---------------------------------------------------------------------------

#[test]
fn dep_graph_header() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("Symbol Dependencies:"));
}

#[test]
fn dep_graph_separator() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("==="));
}

#[test]
fn dep_graph_leaf_depends_on_none() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let dep = viz.dependency_graph();
    // s only uses terminal 'a', so no non-terminal deps
    assert!(dep.contains("(none)"));
}

#[test]
fn dep_graph_chain_has_dependencies() {
    let viz = GrammarVisualizer::new(deep_chain_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("depends on:"));
    // At least 3 symbols depend on something (a→b, b→c, c→d)
    let dep_lines: Vec<&str> = dep
        .lines()
        .filter(|l| l.contains("depends on:") && !l.contains("(none)"))
        .collect();
    assert!(
        dep_lines.len() >= 3,
        "expected >=3 dependency lines, found {}",
        dep_lines.len()
    );
}

#[test]
fn dep_graph_empty_grammar_only_header() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let dep = viz.dependency_graph();
    // No rules → only header
    assert!(dep.contains("Symbol Dependencies:"));
    assert!(!dep.contains("depends on:"));
}

// ---------------------------------------------------------------------------
// 46–50  cross-format consistency & large / special grammars
// ---------------------------------------------------------------------------

#[test]
fn python_like_text_contains_module() {
    let viz = GrammarVisualizer::new(python_like_grammar());
    let text = viz.to_text();
    assert!(text.contains("python_like"));
}

#[test]
fn javascript_like_dot_has_edges() {
    let viz = GrammarVisualizer::new(javascript_like_grammar());
    let dot = viz.to_dot();
    let edge_count = dot.matches("->").count();
    assert!(edge_count >= 5, "expected many edges, found {edge_count}");
}

#[test]
fn all_formats_non_empty_for_minimal() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    assert!(!viz.to_text().is_empty());
    assert!(!viz.to_dot().is_empty());
    assert!(!viz.to_railroad_svg().is_empty());
    assert!(!viz.dependency_graph().is_empty());
}

#[test]
fn large_grammar_text_mentions_all_tokens() {
    let viz = GrammarVisualizer::new(large_grammar());
    let text = viz.to_text();
    for i in 0..50 {
        let tok = format!("t{i}");
        assert!(text.contains(&tok), "missing token {tok}");
    }
}

#[test]
fn large_grammar_dep_graph_has_50_entries() {
    let viz = GrammarVisualizer::new(large_grammar());
    let dep = viz.dependency_graph();
    let entries = dep.lines().filter(|l| l.contains("depends on:")).count();
    assert!(entries >= 50, "expected >=50 entries, found {entries}");
}

// ---------------------------------------------------------------------------
// 51–55  extra edge-case / formatting tests
// ---------------------------------------------------------------------------

#[test]
fn dot_escapes_special_chars_in_labels() {
    // Build a grammar whose token name has a double-quote
    let mut grammar = Grammar::new("escape_test".to_string());
    let tid = SymbolId(1);
    grammar.tokens.insert(
        tid,
        Token {
            name: "say\"hello".to_string(),
            pattern: TokenPattern::String("say\"hello".to_string()),
            fragile: false,
        },
    );
    let sid = SymbolId(2);
    grammar.rules.insert(
        sid,
        vec![Rule {
            lhs: sid,
            rhs: vec![Symbol::Terminal(tid)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let viz = GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    // The escaped form should appear
    assert!(dot.contains(r#"\""#));
}

#[test]
fn svg_escapes_angle_brackets() {
    let mut grammar = Grammar::new("xml_esc".to_string());
    let tid = SymbolId(1);
    grammar.tokens.insert(
        tid,
        Token {
            name: "<tag>".to_string(),
            pattern: TokenPattern::String("<tag>".to_string()),
            fragile: false,
        },
    );
    let sid = SymbolId(2);
    grammar.rules.insert(
        sid,
        vec![Rule {
            lhs: sid,
            rhs: vec![Symbol::Terminal(tid)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let viz = GrammarVisualizer::new(grammar);
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("&lt;") && svg.contains("&gt;"));
}

#[test]
fn text_conflict_section_present() {
    let mut grammar = minimal_grammar();
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: ConflictResolution::GLR,
    });
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("Conflict Declarations:"));
}

#[test]
fn text_no_conflict_section_when_empty() {
    let viz = GrammarVisualizer::new(minimal_grammar());
    let text = viz.to_text();
    assert!(!text.contains("Conflict Declarations:"));
}

#[test]
fn svg_multi_rule_has_multiple_texts() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let svg = viz.to_railroad_svg();
    let text_count = svg.matches("<text").count();
    // At least one <text> per rule alternative plus ::= labels
    assert!(
        text_count >= 3,
        "expected >=3 <text> elements, found {text_count}"
    );
}
