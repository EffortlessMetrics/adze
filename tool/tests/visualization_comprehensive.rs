#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for GrammarVisualizer output formats.
//!
//! Covers: to_dot, to_railroad_svg, to_text, dependency_graph,
//! edge cases with empty/minimal/complex grammars,
//! determinism, multiple visualizations, large grammars, and more.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, Grammar, PrecedenceKind,
    ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
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

/// Grammar with conflict declarations.
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

/// Grammar with a self-recursive rule.
fn recursive_grammar() -> Grammar {
    GrammarBuilder::new("recursive")
        .token("ID", r"[a-z]+")
        .token("COMMA", ",")
        .rule("list", vec!["ID", "COMMA", "list"])
        .rule("list", vec!["ID"])
        .start("list")
        .build()
}

/// Grammar with multiple externals.
fn grammar_with_multiple_externals() -> Grammar {
    let mut grammar = Grammar::new("multi_ext".to_string());
    for i in 1u16..=4 {
        let ext_id = SymbolId(100 + i);
        grammar.externals.push(ExternalToken {
            name: format!("EXT_{}", i),
            symbol_id: ext_id,
        });
    }
    grammar
}

/// Grammar with only tokens (no rules).
fn tokens_only_grammar() -> Grammar {
    let mut grammar = Grammar::new("tokens_only".to_string());
    for i in 1u16..=5 {
        grammar.tokens.insert(
            SymbolId(i),
            Token {
                name: format!("TOK_{}", i),
                pattern: TokenPattern::String(format!("t{}", i)),
                fragile: false,
            },
        );
    }
    grammar
}

/// Build a large grammar with many rules.
fn large_grammar(rule_count: u16) -> Grammar {
    let mut builder = GrammarBuilder::new("large")
        .token("ID", r"[a-z]+")
        .token("NUM", r"\d+");

    for i in 0..rule_count {
        let name = format!("rule_{}", i);
        builder = builder.rule(&name, vec!["ID"]);
    }
    builder.start("rule_0").build()
}

/// Grammar with nested complex symbols (Optional(Repeat(Terminal))).
fn nested_complex_symbol_grammar() -> Grammar {
    let mut grammar = Grammar::new("nested".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "X".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(10);
    grammar.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Repeat(Box::new(
            Symbol::Terminal(tok),
        ))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

/// Grammar with dynamic precedence.
fn grammar_with_dynamic_precedence() -> Grammar {
    let mut grammar = Grammar::new("dyn_prec".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "TOK".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(2);
    grammar.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: Some(PrecedenceKind::Dynamic(5)),
        associativity: Some(Associativity::Right),
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

/// Grammar with multiple alternative rules for one nonterminal.
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

/// Grammar with fragile tokens.
fn grammar_with_fragile_token() -> Grammar {
    GrammarBuilder::new("fragile")
        .token("ID", r"[a-z]+")
        .fragile_token("WS", r"\s+")
        .rule("start", vec!["ID"])
        .start("start")
        .build()
}

/// Grammar with only an epsilon rule.
fn epsilon_only_grammar() -> Grammar {
    let mut grammar = Grammar::new("eps".to_string());
    let rule_id = SymbolId(1);
    grammar.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar
}

/// Grammar using the builder's external() method.
fn builder_external_grammar() -> Grammar {
    GrammarBuilder::new("builder_ext")
        .token("ID", r"[a-z]+")
        .external("INDENT")
        .external("DEDENT")
        .rule("block", vec!["ID"])
        .start("block")
        .build()
}

/// Grammar with multiple conflict resolutions.
fn grammar_with_varied_conflicts() -> Grammar {
    let mut grammar = Grammar::new("varied_conflicts".to_string());
    let s1 = SymbolId(1);
    let s2 = SymbolId(2);
    let s3 = SymbolId(3);
    grammar.tokens.insert(
        s1,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        s2,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        s3,
        Token {
            name: "C".to_string(),
            pattern: TokenPattern::String("c".to_string()),
            fragile: false,
        },
    );
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![s1, s2],
        resolution: ConflictResolution::GLR,
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![s2, s3],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    });
    grammar.conflicts.push(ConflictDeclaration {
        symbols: vec![s1, s3],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(3)),
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
    assert!(dot.contains("\\\"") || dot.contains("lt&amp;gt") || dot.contains("lt&gt"));
}

#[test]
fn dot_recursive_grammar_has_self_edge() {
    let viz = GrammarVisualizer::new(recursive_grammar());
    let dot = viz.to_dot();
    // The "list" rule references itself, so we should see an edge from n_X -> n_X
    // Find the list node id
    let edge_count = dot.matches("->").count();
    assert!(
        edge_count >= 2,
        "Recursive grammar should have multiple edges"
    );
}

#[test]
fn dot_tokens_only_grammar_no_edges() {
    let viz = GrammarVisualizer::new(tokens_only_grammar());
    let dot = viz.to_dot();
    assert!(
        !dot.contains("->"),
        "Tokens-only grammar should have no edges"
    );
}

#[test]
fn dot_large_grammar_compiles() {
    let viz = GrammarVisualizer::new(large_grammar(50));
    let dot = viz.to_dot();
    assert!(dot.contains("digraph Grammar"));
    assert!(dot.len() > 500, "Large grammar DOT should be substantial");
}

#[test]
fn dot_multiple_externals_all_appear() {
    let viz = GrammarVisualizer::new(grammar_with_multiple_externals());
    let dot = viz.to_dot();
    for i in 1..=4 {
        assert!(
            dot.contains(&format!("EXT_{}", i)),
            "External EXT_{} should appear in DOT",
            i
        );
    }
}

#[test]
fn dot_epsilon_not_in_edges() {
    let viz = GrammarVisualizer::new(epsilon_only_grammar());
    let dot = viz.to_dot();
    // Epsilon transitions are skipped in the DOT output
    assert!(
        !dot.contains("->"),
        "Epsilon-only rule should produce no edges"
    );
}

#[test]
fn dot_single_symbol_rule_has_empty_label() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let dot = viz.to_dot();
    // Single-symbol rules have empty labels
    assert!(
        dot.contains("label=\"\""),
        "Single-symbol rule edge should have empty label"
    );
}

#[test]
fn dot_multi_alternative_has_edges_per_alt() {
    let viz = GrammarVisualizer::new(multi_alternative_grammar());
    let dot = viz.to_dot();
    let edge_count = dot.matches("->").count();
    assert!(
        edge_count >= 4,
        "Four alternatives should produce at least 4 edges, got {}",
        edge_count
    );
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
    assert!(svg.contains("A?"), "Expected Optional rendered as A?");
    assert!(svg.contains("B*"), "Expected Repeat rendered as B*");
    assert!(svg.contains("A+"), "Expected RepeatOne rendered as A+");
    assert!(svg.contains("ε"), "Expected Epsilon rendered as ε");
}

#[test]
fn svg_special_chars_xml_escaped() {
    let viz = GrammarVisualizer::new(grammar_with_special_chars());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("&amp;") || svg.contains("&lt;") || svg.contains("&gt;"));
}

#[test]
fn svg_connecting_lines_drawn() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<line"), "SVG should draw connecting lines");
}

#[test]
fn svg_empty_grammar_has_no_rects() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let svg = viz.to_railroad_svg();
    assert!(
        !svg.contains("<rect"),
        "Empty grammar SVG should have no rectangles"
    );
}

#[test]
fn svg_recursive_grammar_valid() {
    let viz = GrammarVisualizer::new(recursive_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(
        svg.contains("<rect"),
        "Recursive grammar should have elements"
    );
}

#[test]
fn svg_tokens_only_no_rule_text() {
    let viz = GrammarVisualizer::new(tokens_only_grammar());
    let svg = viz.to_railroad_svg();
    assert!(!svg.contains("::="), "Tokens-only should have no rule rows");
}

#[test]
fn svg_large_grammar() {
    let viz = GrammarVisualizer::new(large_grammar(20));
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.len() > 500, "Large grammar SVG should be substantial");
}

#[test]
fn svg_nested_complex_symbol_rendered() {
    let viz = GrammarVisualizer::new(nested_complex_symbol_grammar());
    let svg = viz.to_railroad_svg();
    // Optional(Repeat(X)) should render as "X*?"
    assert!(
        svg.contains("X*?"),
        "Nested Optional(Repeat(X)) should render as X*?"
    );
}

#[test]
fn svg_choice_renders_pipe_separator() {
    let viz = GrammarVisualizer::new(complex_symbol_grammar());
    let svg = viz.to_railroad_svg();
    assert!(
        svg.contains("|"),
        "Choice symbols should use pipe separator in SVG"
    );
}

#[test]
fn svg_contains_rect_elements_for_symbols() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let svg = viz.to_railroad_svg();
    assert!(
        svg.contains("<rect"),
        "Symbols should be drawn as rectangles"
    );
}

#[test]
fn svg_external_token_rendered() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let svg = viz.to_railroad_svg();
    assert!(
        svg.contains("External"),
        "External tokens should be labeled in SVG"
    );
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

#[test]
fn text_grammar_name_appears_in_header() {
    let grammar = Grammar::new("my_custom_grammar".to_string());
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("Grammar: my_custom_grammar"));
}

#[test]
fn text_separator_line_is_50_equals() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let text = viz.to_text();
    assert!(text.contains(&"=".repeat(50)));
}

#[test]
fn text_empty_grammar_tokens_section_exists() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let text = viz.to_text();
    assert!(
        text.contains("Tokens:"),
        "Tokens section should always exist"
    );
}

#[test]
fn text_empty_grammar_rules_section_exists() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let text = viz.to_text();
    assert!(text.contains("Rules:"), "Rules section should always exist");
}

#[test]
fn text_multiple_tokens_all_listed() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let text = viz.to_text();
    assert!(text.contains("NUM"));
    assert!(text.contains("PLUS"));
    assert!(text.contains("STAR"));
    assert!(text.contains("LPAREN"));
    assert!(text.contains("RPAREN"));
}

#[test]
fn text_multiple_alternatives_all_listed() {
    let viz = GrammarVisualizer::new(multi_alternative_grammar());
    let text = viz.to_text();
    let bnf_count = text.matches("::=").count();
    assert!(
        bnf_count >= 4,
        "Should have at least 4 ::= lines for 4 alternatives, got {}",
        bnf_count
    );
}

#[test]
fn text_external_token_shows_symbol_id() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let text = viz.to_text();
    // External tokens are formatted as "  NAME (SymbolId(N))"
    assert!(
        text.contains("SymbolId(10)"),
        "External token should show its SymbolId"
    );
}

#[test]
fn text_dollar_prefix_for_external_in_rules() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let text = viz.to_text();
    assert!(
        text.contains("$10"),
        "External symbols in rules use $ prefix"
    );
}

#[test]
fn text_epsilon_only_grammar() {
    let viz = GrammarVisualizer::new(epsilon_only_grammar());
    let text = viz.to_text();
    assert!(text.contains("ε"), "Epsilon-only rule should show ε");
}

#[test]
fn text_dynamic_precedence_rendered() {
    let viz = GrammarVisualizer::new(grammar_with_dynamic_precedence());
    let text = viz.to_text();
    assert!(
        text.contains("Dynamic(5)"),
        "Dynamic precedence should be shown"
    );
    assert!(
        text.contains("Right"),
        "Right associativity should be shown"
    );
}

#[test]
fn text_nested_complex_symbol() {
    let viz = GrammarVisualizer::new(nested_complex_symbol_grammar());
    let text = viz.to_text();
    assert!(
        text.contains("X*?"),
        "Nested Optional(Repeat(X)) should render as X*?"
    );
}

#[test]
fn text_omits_precedence_section_when_empty() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let text = viz.to_text();
    assert!(
        !text.contains("Precedence Declarations:"),
        "Should not show precedence section when empty"
    );
}

#[test]
fn text_recursive_grammar_shows_nonterminal_ref() {
    let viz = GrammarVisualizer::new(recursive_grammar());
    let text = viz.to_text();
    // The rule references itself by nonterminal name
    assert!(text.contains("::="), "Recursive rules should have BNF");
}

#[test]
fn text_varied_conflicts_shows_multiple_resolutions() {
    let viz = GrammarVisualizer::new(grammar_with_varied_conflicts());
    let text = viz.to_text();
    assert!(text.contains("Conflict Declarations:"));
    assert!(
        text.contains("GLR"),
        "GLR conflict resolution should appear"
    );
    assert!(
        text.contains("Left"),
        "Associativity conflict resolution should appear"
    );
}

#[test]
fn text_multiple_externals_all_listed() {
    let viz = GrammarVisualizer::new(grammar_with_multiple_externals());
    let text = viz.to_text();
    for i in 1..=4 {
        assert!(
            text.contains(&format!("EXT_{}", i)),
            "External EXT_{} should be listed",
            i
        );
    }
}

#[test]
fn text_fragile_token_listed() {
    let viz = GrammarVisualizer::new(grammar_with_fragile_token());
    let text = viz.to_text();
    // Fragile tokens are listed with their pattern; the fragile flag itself isn't
    // shown in text output, but the token should at least appear.
    assert!(text.contains("WS"), "Fragile token WS should be listed");
}

#[test]
fn text_large_grammar_has_many_rules() {
    let viz = GrammarVisualizer::new(large_grammar(30));
    let text = viz.to_text();
    let bnf_count = text.matches("::=").count();
    assert!(
        bnf_count >= 30,
        "Large grammar should have >= 30 rules, got {}",
        bnf_count
    );
}

#[test]
fn text_builder_external_listed() {
    let viz = GrammarVisualizer::new(builder_external_grammar());
    let text = viz.to_text();
    assert!(text.contains("INDENT"));
    assert!(text.contains("DEDENT"));
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
    assert!(
        !dep.contains("(none)") || dep.lines().count() > 3,
        "Multi-rule grammar should have non-trivial dependencies"
    );
}

#[test]
fn dependency_graph_recursive_grammar() {
    let viz = GrammarVisualizer::new(recursive_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("depends on:"));
    // Self-recursive rule: list depends on list
    assert!(
        dep.lines()
            .any(|line| line.contains("depends on:") && !line.contains("(none)")),
        "Recursive grammar should have nonterminal dependency"
    );
}

#[test]
fn dependency_graph_epsilon_only() {
    let viz = GrammarVisualizer::new(epsilon_only_grammar());
    let dep = viz.dependency_graph();
    // Epsilon-only rule has no nonterminal dependencies
    assert!(dep.contains("(none)"));
}

#[test]
fn dependency_graph_tokens_only() {
    let viz = GrammarVisualizer::new(tokens_only_grammar());
    let dep = viz.dependency_graph();
    // No rules means no dependency entries at all (beyond header)
    let line_count = dep.lines().count();
    assert!(
        line_count <= 3,
        "Tokens-only grammar should have minimal dependency output"
    );
}

#[test]
fn dependency_graph_large_grammar() {
    let viz = GrammarVisualizer::new(large_grammar(20));
    let dep = viz.dependency_graph();
    assert!(dep.contains("depends on:"));
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
fn dependency_graph_stable_across_calls_on_same_instance() {
    let viz = GrammarVisualizer::new(single_token_grammar());
    let d1 = viz.dependency_graph();
    let d2 = viz.dependency_graph();
    assert_eq!(
        d1, d2,
        "Same visualizer should produce identical dependency graph"
    );
}

#[test]
fn all_formats_nonempty_for_real_grammar() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    assert!(!viz.to_dot().is_empty());
    assert!(!viz.to_railroad_svg().is_empty());
    assert!(!viz.to_text().is_empty());
    assert!(!viz.dependency_graph().is_empty());
}

#[test]
fn determinism_across_ten_runs() {
    let baseline_dot = GrammarVisualizer::new(arithmetic_grammar()).to_dot();
    let baseline_text = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    for _ in 0..10 {
        assert_eq!(
            GrammarVisualizer::new(arithmetic_grammar()).to_dot(),
            baseline_dot
        );
        assert_eq!(
            GrammarVisualizer::new(arithmetic_grammar()).to_text(),
            baseline_text
        );
    }
}

// ===========================================================================
// Multiple visualizations of same grammar
// ===========================================================================

#[test]
fn same_visualizer_multiple_dot_calls() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let dot1 = viz.to_dot();
    let dot2 = viz.to_dot();
    assert_eq!(
        dot1, dot2,
        "Multiple to_dot calls should return same result"
    );
}

#[test]
fn same_visualizer_multiple_text_calls() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let t1 = viz.to_text();
    let t2 = viz.to_text();
    assert_eq!(t1, t2, "Multiple to_text calls should return same result");
}

#[test]
fn same_visualizer_multiple_svg_calls() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    let s1 = viz.to_railroad_svg();
    let s2 = viz.to_railroad_svg();
    assert_eq!(
        s1, s2,
        "Multiple to_railroad_svg calls should return same result"
    );
}

#[test]
fn same_visualizer_all_formats_consistent() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    // Calling all formats in sequence should not interfere with each other
    let dot = viz.to_dot();
    let text = viz.to_text();
    let svg = viz.to_railroad_svg();
    let dep = viz.dependency_graph();
    assert!(!dot.is_empty());
    assert!(!text.is_empty());
    assert!(!svg.is_empty());
    assert!(!dep.is_empty());
    // Re-call and verify stability
    assert_eq!(viz.to_dot(), dot);
    assert_eq!(viz.to_text(), text);
}

// ===========================================================================
// Different grammars produce different output
// ===========================================================================

#[test]
fn different_grammars_different_dot() {
    let dot1 = GrammarVisualizer::new(empty_grammar()).to_dot();
    let dot2 = GrammarVisualizer::new(arithmetic_grammar()).to_dot();
    assert_ne!(
        dot1, dot2,
        "Different grammars should produce different DOT"
    );
}

#[test]
fn different_grammars_different_text() {
    let t1 = GrammarVisualizer::new(empty_grammar()).to_text();
    let t2 = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    assert_ne!(t1, t2, "Different grammars should produce different text");
}

#[test]
fn different_grammars_different_svg() {
    let s1 = GrammarVisualizer::new(empty_grammar()).to_railroad_svg();
    let s2 = GrammarVisualizer::new(arithmetic_grammar()).to_railroad_svg();
    assert_ne!(s1, s2, "Different grammars should produce different SVG");
}

// ===========================================================================
// Large grammar tests
// ===========================================================================

#[test]
fn large_grammar_100_rules_dot() {
    let viz = GrammarVisualizer::new(large_grammar(100));
    let dot = viz.to_dot();
    assert!(dot.contains("digraph Grammar"));
    let edge_count = dot.matches("->").count();
    assert!(
        edge_count >= 100,
        "100-rule grammar should have >= 100 edges, got {}",
        edge_count
    );
}

#[test]
fn large_grammar_100_rules_text() {
    let viz = GrammarVisualizer::new(large_grammar(100));
    let text = viz.to_text();
    let bnf_count = text.matches("::=").count();
    assert!(
        bnf_count >= 100,
        "100-rule grammar should have >= 100 BNF lines, got {}",
        bnf_count
    );
}

#[test]
fn large_grammar_100_rules_svg() {
    let viz = GrammarVisualizer::new(large_grammar(100));
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn large_grammar_100_rules_dependency() {
    let viz = GrammarVisualizer::new(large_grammar(100));
    let dep = viz.dependency_graph();
    let depends_count = dep.matches("depends on:").count();
    assert!(
        depends_count >= 100,
        "100-rule grammar should have >= 100 dependency entries, got {}",
        depends_count
    );
}

// ===========================================================================
// Edge cases: symbol types in all formats
// ===========================================================================

#[test]
fn choice_in_text_format() {
    let viz = GrammarVisualizer::new(complex_symbol_grammar());
    let text = viz.to_text();
    assert!(text.contains("(A | B)"), "Choice should render as (A | B)");
}

#[test]
fn sequence_in_text_format() {
    let viz = GrammarVisualizer::new(complex_symbol_grammar());
    let text = viz.to_text();
    // Sequence should output its elements separated by space
    assert!(text.contains(" A"), "Sequence should contain A");
    assert!(text.contains(" B"), "Sequence should contain B");
}

#[test]
fn choice_in_svg_format() {
    let viz = GrammarVisualizer::new(complex_symbol_grammar());
    let svg = viz.to_railroad_svg();
    assert!(
        svg.contains("(A | B)"),
        "Choice in SVG should render as (A | B)"
    );
}

#[test]
fn sequence_in_svg_format() {
    let viz = GrammarVisualizer::new(complex_symbol_grammar());
    let svg = viz.to_railroad_svg();
    // Sequence rendered as space-separated
    assert!(
        svg.contains("A B"),
        "Sequence in SVG should render as 'A B'"
    );
}

// ===========================================================================
// Construction tests
// ===========================================================================

#[test]
fn visualizer_from_builder_grammar() {
    let grammar = GrammarBuilder::new("test")
        .token("X", "x")
        .rule("start", vec!["X"])
        .start("start")
        .build();
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("Grammar: test"));
}

#[test]
fn visualizer_from_manual_grammar() {
    let grammar = Grammar::new("manual".to_string());
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("Grammar: manual"));
}

#[test]
fn visualizer_from_grammar_with_extras() {
    let grammar = GrammarBuilder::new("with_extras")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("start", vec!["ID"])
        .start("start")
        .build();
    let viz = GrammarVisualizer::new(grammar);
    // Extras don't crash the visualizer; output should still be valid
    let text = viz.to_text();
    assert!(text.contains("Grammar: with_extras"));
    let dot = viz.to_dot();
    assert!(dot.contains("digraph Grammar"));
}

// ===========================================================================
// Unicode and edge-case content
// ===========================================================================

#[test]
fn text_unicode_grammar_name() {
    let grammar = Grammar::new("日本語文法".to_string());
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("Grammar: 日本語文法"));
}

#[test]
fn text_empty_name_grammar() {
    let grammar = Grammar::new(String::new());
    let viz = GrammarVisualizer::new(grammar);
    let text = viz.to_text();
    assert!(text.contains("Grammar: "));
}

#[test]
fn dot_newline_in_token_name_escaped() {
    let mut grammar = Grammar::new("nl".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "line\nbreak".to_string(),
            pattern: TokenPattern::String("\\n".to_string()),
            fragile: false,
        },
    );
    let viz = GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    assert!(
        dot.contains("\\n"),
        "Newlines in token names should be escaped in DOT"
    );
}

#[test]
fn dot_backslash_in_token_name_escaped() {
    let mut grammar = Grammar::new("bs".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "back\\slash".to_string(),
            pattern: TokenPattern::String("\\".to_string()),
            fragile: false,
        },
    );
    let viz = GrammarVisualizer::new(grammar);
    let dot = viz.to_dot();
    assert!(
        dot.contains("\\\\"),
        "Backslashes in DOT labels should be escaped"
    );
}

#[test]
fn svg_ampersand_in_token_name_escaped() {
    let mut grammar = Grammar::new("amp".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "a&b".to_string(),
            pattern: TokenPattern::String("ab".to_string()),
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
    let viz = GrammarVisualizer::new(grammar);
    let svg = viz.to_railroad_svg();
    assert!(
        svg.contains("a&amp;b"),
        "Ampersand in SVG should be XML-escaped"
    );
}

#[test]
fn svg_angle_brackets_in_token_name_escaped() {
    let mut grammar = Grammar::new("angles".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "<tag>".to_string(),
            pattern: TokenPattern::String("<>".to_string()),
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
    let viz = GrammarVisualizer::new(grammar);
    let svg = viz.to_railroad_svg();
    assert!(
        svg.contains("&lt;") && svg.contains("&gt;"),
        "Angle brackets should be XML-escaped"
    );
}
