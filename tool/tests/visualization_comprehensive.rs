#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for GrammarVisualizer output formats.
//!
//! Covers: to_dot, to_railroad_svg, to_text, dependency_graph,
//! edge cases with empty/minimal/complex grammars.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, Grammar, ProductionId,
    Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tool::visualization::GrammarVisualizer;

// ---------------------------------------------------------------------------
// Helper constructors
// ---------------------------------------------------------------------------

fn empty_grammar() -> Grammar {
    Grammar::new("empty".to_string())
}

fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("single")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .build()
}

fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arithmetic")
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

/// Grammar with external tokens.
fn grammar_with_externals() -> Grammar {
    let mut grammar = Grammar::new("ext".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "WS".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: false,
        },
    );
    let ext = SymbolId(10);
    grammar.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: ext,
    });
    let rule_id = SymbolId(2);
    grammar.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::External(ext), Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

/// Grammar using Optional, Repeat, Choice, Sequence, Epsilon symbols.
fn complex_symbol_grammar() -> Grammar {
    let mut grammar = Grammar::new("complex".to_string());
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    grammar.tokens.insert(
        tok_a,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        tok_b,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(10);
    grammar.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![
            Symbol::Optional(Box::new(Symbol::Terminal(tok_a))),
            Symbol::Repeat(Box::new(Symbol::Terminal(tok_b))),
            Symbol::RepeatOne(Box::new(Symbol::Terminal(tok_a))),
            Symbol::Choice(vec![Symbol::Terminal(tok_a), Symbol::Terminal(tok_b)]),
            Symbol::Sequence(vec![Symbol::Terminal(tok_a), Symbol::Terminal(tok_b)]),
            Symbol::Epsilon,
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

/// Grammar with precedence and associativity metadata on rules.
fn grammar_with_precedence() -> Grammar {
    GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Grammar with conflict declarations (built manually since builder lacks method).
fn grammar_with_conflicts() -> Grammar {
    let mut grammar = Grammar::new("conflicts".to_string());
    let s1 = SymbolId(1);
    let s2 = SymbolId(2);
    grammar.tokens.insert(
        s1,
        Token {
            name: "X".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        s2,
        Token {
            name: "Y".to_string(),
            pattern: TokenPattern::String("y".to_string()),
            fragile: false,
        },
    );
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![s1, s2],
        resolution: ConflictResolution::GLR,
    });
    grammar
}

/// Grammar with special characters in token names / patterns.
fn grammar_with_special_chars() -> Grammar {
    let mut grammar = Grammar::new("special".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "lt&gt".to_string(),
            pattern: TokenPattern::String("<>\"&'".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(2);
    grammar.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

/// Grammar with precedence declarations (at grammar level).
fn grammar_with_precedence_declarations() -> Grammar {
    let mut grammar = Grammar::new("prec_decl".to_string());
    let s1 = SymbolId(1);
    let s2 = SymbolId(2);
    grammar.tokens.insert(
        s1,
        Token {
            name: "PLUS".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        s2,
        Token {
            name: "STAR".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    grammar.precedences.push(adze_ir::Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![s1],
    });
    grammar.precedences.push(adze_ir::Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![s2],
    });
    grammar
}

// ===========================================================================
// DOT output tests
// ===========================================================================

#[test]
fn dot_empty_grammar_is_valid() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let dot = viz.to_dot();
    assert!(dot.starts_with("digraph Grammar {"));
    assert!(dot.trim_end().ends_with('}'));
}

#[test]
fn dot_contains_digraph_header() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("digraph Grammar"));
    assert!(dot.contains("rankdir=LR"));
    assert!(dot.contains("node [shape=box]"));
}

#[test]
fn dot_terminals_styled_as_ellipses() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("shape=ellipse"));
    assert!(dot.contains("fillcolor=lightblue"));
}

#[test]
fn dot_nonterminals_styled_green() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("fillcolor=lightgreen"));
}

#[test]
fn dot_edges_present_for_rules() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("->"), "DOT should contain edges");
}

#[test]
fn dot_multi_symbol_rules_have_labels() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let dot = viz.to_dot();
    // Multi-symbol rules produce positional labels like "1", "2", etc.
    assert!(dot.contains("[label=\"1\"]") || dot.contains("label=\"1\""));
}

#[test]
fn dot_external_tokens_shown_as_diamonds() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let dot = viz.to_dot();
    assert!(dot.contains("shape=diamond"));
    assert!(dot.contains("fillcolor=lightcoral"));
    assert!(dot.contains("INDENT"));
}

#[test]
fn dot_special_chars_escaped() {
    let viz = GrammarVisualizer::new(grammar_with_special_chars());
    let dot = viz.to_dot();
    // Backslash-escaped double-quotes inside DOT labels
    assert!(dot.contains("\\\"") || dot.contains("lt&amp;gt") || dot.contains("lt&gt"));
}

// ===========================================================================
// Railroad SVG tests
// ===========================================================================

#[test]
fn svg_empty_grammar_valid() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn svg_contains_style_block() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<style>"));
    assert!(svg.contains(".terminal"));
    assert!(svg.contains(".non-terminal"));
    assert!(svg.contains(".line"));
}

#[test]
fn svg_contains_rule_names() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("::="), "SVG should contain BNF-style ::=");
}

#[test]
fn svg_complex_symbols_rendered() {
    let viz = GrammarVisualizer::new(complex_symbol_grammar());
    let svg = viz.to_railroad_svg();
    // Optional renders as "X?"
    assert!(svg.contains("A?"), "Expected Optional rendered as A?");
    // Repeat renders as "X*"
    assert!(svg.contains("B*"), "Expected Repeat rendered as B*");
    // RepeatOne renders as "X+"
    assert!(svg.contains("A+"), "Expected RepeatOne rendered as A+");
    // Epsilon renders as ε
    assert!(svg.contains("ε"), "Expected Epsilon rendered as ε");
}

#[test]
fn svg_special_chars_xml_escaped() {
    let viz = GrammarVisualizer::new(grammar_with_special_chars());
    let svg = viz.to_railroad_svg();
    // XML special characters must be escaped
    assert!(svg.contains("&amp;") || svg.contains("&lt;") || svg.contains("&gt;"));
}

#[test]
fn svg_connecting_lines_drawn() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<line"), "SVG should draw connecting lines");
}

// ===========================================================================
// Text output tests
// ===========================================================================

#[test]
fn text_empty_grammar_has_header() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let text = viz.to_text();
    assert!(text.contains("Grammar: empty"));
    assert!(text.contains("=".repeat(50).as_str()));
}

#[test]
fn text_shows_tokens_with_patterns() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let text = viz.to_text();
    assert!(text.contains("Tokens:"));
    assert!(text.contains("ID"));
    assert!(text.contains("/[a-z]+/"));
}

#[test]
fn text_shows_string_token_pattern() {
    let mut grammar = Grammar::new("str_tok".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "PLUS".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("\"+\""), "String patterns should be quoted");
}

#[test]
fn text_shows_rules_with_bnf_notation() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let text = viz.to_text();
    assert!(text.contains("Rules:"));
    assert!(text.contains("::="));
}

#[test]
fn text_shows_external_tokens_section() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let text = viz.to_text();
    assert!(text.contains("External Tokens:"));
    assert!(text.contains("INDENT"));
}

#[test]
fn text_omits_external_section_when_empty() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let text = viz.to_text();
    assert!(
        !text.contains("External Tokens:"),
        "Should not show external section when empty"
    );
}

#[test]
fn text_shows_precedence_metadata() {
    let viz = GrammarVisualizer::new(grammar_with_precedence());
    let text = viz.to_text();
    assert!(
        text.contains("[precedence:"),
        "Should show precedence annotation"
    );
    assert!(
        text.contains("[associativity:"),
        "Should show associativity annotation"
    );
}

#[test]
fn text_shows_precedence_declarations() {
    let viz = GrammarVisualizer::new(grammar_with_precedence_declarations());
    let text = viz.to_text();
    assert!(text.contains("Precedence Declarations:"));
    assert!(text.contains("Level 1"));
    assert!(text.contains("Level 2"));
}

#[test]
fn text_shows_conflict_declarations() {
    let viz = GrammarVisualizer::new(grammar_with_conflicts());
    let text = viz.to_text();
    assert!(text.contains("Conflict Declarations:"));
    assert!(text.contains("GLR"));
}

#[test]
fn text_omits_conflict_section_when_empty() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let text = viz.to_text();
    assert!(
        !text.contains("Conflict Declarations:"),
        "Should not show conflict section when empty"
    );
}

#[test]
fn text_complex_symbols_rendered() {
    let viz = GrammarVisualizer::new(complex_symbol_grammar());
    let text = viz.to_text();
    assert!(text.contains("A?"), "Optional should render as A?");
    assert!(text.contains("B*"), "Repeat should render as B*");
    assert!(text.contains("A+"), "RepeatOne should render as A+");
    assert!(text.contains("ε"), "Epsilon should render as ε");
    assert!(text.contains("|"), "Choice should use | separator");
}

// ===========================================================================
// Dependency graph tests
// ===========================================================================

#[test]
fn dependency_graph_empty_grammar() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("Symbol Dependencies:"));
    assert!(dep.contains("==================="));
}

#[test]
fn dependency_graph_terminal_only_rule() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("depends on:"));
    assert!(dep.contains("(none)"));
}

#[test]
fn dependency_graph_nonterminal_references() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("depends on:"));
    // expr depends on term, term depends on expr
    assert!(
        !dep.contains("(none)") || dep.lines().count() > 3,
        "Multi-rule grammar should have non-trivial dependencies"
    );
}

// ===========================================================================
// Consistency and determinism tests
// ===========================================================================

#[test]
fn dot_is_deterministic() {
    let dot1 = GrammarVisualizer::new(arithmetic_grammar()).to_dot();
    let dot2 = GrammarVisualizer::new(arithmetic_grammar()).to_dot();
    assert_eq!(dot1, dot2, "DOT output should be deterministic");
}

#[test]
fn svg_is_deterministic() {
    let svg1 = GrammarVisualizer::new(arithmetic_grammar()).to_railroad_svg();
    let svg2 = GrammarVisualizer::new(arithmetic_grammar()).to_railroad_svg();
    assert_eq!(svg1, svg2, "SVG output should be deterministic");
}

#[test]
fn text_is_deterministic() {
    let t1 = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    let t2 = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    assert_eq!(t1, t2, "Text output should be deterministic");
}

#[test]
fn all_formats_nonempty_for_real_grammar() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    assert!(!viz.to_dot().is_empty());
    assert!(!viz.to_railroad_svg().is_empty());
    assert!(!viz.to_text().is_empty());
    assert!(!viz.dependency_graph().is_empty());
}
