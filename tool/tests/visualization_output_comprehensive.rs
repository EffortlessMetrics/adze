#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for grammar and tree visualization output in adze-tool.
//!
//! Tests DOT graph output, text-based tree rendering, SVG railroad diagrams,
//! dependency graphs, and edge cases across various grammar structures.

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

fn single_rule_grammar() -> Grammar {
    let mut g = Grammar::new("single_rule".to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(2);
    g.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

fn multi_rule_grammar() -> Grammar {
    GrammarBuilder::new("multi")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .token("LPAREN", r"\(")
        .token("RPAREN", r"\)")
        .rule("expr", vec!["term", "PLUS", "expr"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["factor", "STAR", "term"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["NUM"])
        .rule("factor", vec!["LPAREN", "expr", "RPAREN"])
        .start("expr")
        .build()
}

fn grammar_with_externals() -> Grammar {
    let mut g = Grammar::new("externals".to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "WS".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: false,
        },
    );
    let ext = SymbolId(10);
    g.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: ext,
    });
    g.externals.push(ExternalToken {
        name: "DEDENT".to_string(),
        symbol_id: SymbolId(11),
    });
    let rule_id = SymbolId(2);
    g.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::External(ext), Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

fn complex_symbol_grammar() -> Grammar {
    let mut g = Grammar::new("complex_symbols".to_string());
    let tok_a = SymbolId(1);
    let tok_b = SymbolId(2);
    g.tokens.insert(
        tok_a,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        tok_b,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(10);
    g.rules.entry(rule_id).or_default().push(Rule {
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
    g
}

fn special_chars_grammar() -> Grammar {
    let mut g = Grammar::new("special_chars".to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "a<b>&c".to_string(),
            pattern: TokenPattern::String("<>&\"'".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(2);
    g.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

fn grammar_with_newline_name() -> Grammar {
    let mut g = Grammar::new("newline_test".to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "line\nbreak".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(2);
    g.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

fn grammar_with_precedence_and_assoc() -> Grammar {
    let mut g = Grammar::new("prec_assoc".to_string());
    let num = SymbolId(1);
    let plus = SymbolId(2);
    let star = SymbolId(3);
    g.tokens.insert(
        num,
        Token {
            name: "NUM".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        plus,
        Token {
            name: "PLUS".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        star,
        Token {
            name: "STAR".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    let expr = SymbolId(10);
    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(plus),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(star),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Static(2)),
        associativity: Some(Associativity::Right),
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });
    g
}

fn grammar_with_conflicts() -> Grammar {
    let mut g = Grammar::new("conflicts".to_string());
    let s1 = SymbolId(1);
    let s2 = SymbolId(2);
    g.tokens.insert(
        s1,
        Token {
            name: "X".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        s2,
        Token {
            name: "Y".to_string(),
            pattern: TokenPattern::String("y".to_string()),
            fragile: false,
        },
    );
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![s1, s2],
        resolution: ConflictResolution::GLR,
    });
    g
}

fn grammar_with_precedence_decls() -> Grammar {
    let mut g = Grammar::new("prec_decl".to_string());
    let plus = SymbolId(1);
    let star = SymbolId(2);
    g.tokens.insert(
        plus,
        Token {
            name: "PLUS".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        star,
        Token {
            name: "STAR".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    g.precedences.push(adze_ir::Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![plus],
    });
    g.precedences.push(adze_ir::Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![star],
    });
    g
}

// ===========================================================================
// DOT graph output correctness
// ===========================================================================

#[test]
fn dot_empty_grammar_has_valid_structure() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let dot = viz.to_dot();
    assert!(dot.starts_with("digraph Grammar {"));
    assert!(dot.trim_end().ends_with('}'));
    assert!(dot.contains("rankdir=LR"));
    assert!(dot.contains("node [shape=box]"));
    // Section comments
    assert!(dot.contains("// Terminals"));
    assert!(dot.contains("// Non-terminals"));
    assert!(dot.contains("// External tokens"));
    assert!(dot.contains("// Rules"));
}

#[test]
fn dot_single_rule_terminals_styled() {
    let viz = GrammarVisualizer::new(single_rule_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("shape=ellipse"));
    assert!(dot.contains("style=filled"));
    assert!(dot.contains("fillcolor=lightblue"));
    assert!(dot.contains("NUMBER"));
}

#[test]
fn dot_single_rule_nonterminals_styled() {
    let viz = GrammarVisualizer::new(single_rule_grammar());
    let dot = viz.to_dot();
    assert!(dot.contains("fillcolor=lightgreen"));
}

#[test]
fn dot_multi_rule_has_edges() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let dot = viz.to_dot();
    let edge_count = dot.matches("->").count();
    assert!(
        edge_count >= 6,
        "Multi-rule grammar should have many edges, got {}",
        edge_count
    );
}

#[test]
fn dot_multi_symbol_rules_have_positional_labels() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let dot = viz.to_dot();
    // Rules with multiple RHS symbols get positional labels
    assert!(
        dot.contains("label=\"1\""),
        "Multi-symbol rules should have positional label '1'"
    );
    assert!(
        dot.contains("label=\"2\""),
        "Multi-symbol rules should have positional label '2'"
    );
}

#[test]
fn dot_single_symbol_rule_has_empty_label() {
    let viz = GrammarVisualizer::new(single_rule_grammar());
    let dot = viz.to_dot();
    // Single-symbol rules get empty labels
    assert!(
        dot.contains("label=\"\""),
        "Single-symbol rules should have empty label"
    );
}

#[test]
fn dot_external_tokens_styled_as_diamonds() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let dot = viz.to_dot();
    assert!(dot.contains("shape=diamond"));
    assert!(dot.contains("fillcolor=lightcoral"));
    assert!(dot.contains("INDENT"));
    assert!(dot.contains("DEDENT"));
}

#[test]
fn dot_external_edges_present() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let dot = viz.to_dot();
    // Rule references External(10), so edge should go to e10
    assert!(dot.contains("-> e10"), "Should have edge to external token");
}

#[test]
fn dot_escapes_double_quotes_in_labels() {
    let mut g = Grammar::new("quote_test".to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "has\"quote".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(
        dot.contains("has\\\"quote"),
        "Double quotes should be escaped in DOT"
    );
}

#[test]
fn dot_escapes_backslashes_in_labels() {
    let mut g = Grammar::new("backslash_test".to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "back\\slash".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(
        dot.contains("back\\\\slash"),
        "Backslashes should be escaped in DOT"
    );
}

#[test]
fn dot_escapes_newlines_in_labels() {
    let viz = GrammarVisualizer::new(grammar_with_newline_name());
    let dot = viz.to_dot();
    assert!(
        dot.contains("line\\nbreak"),
        "Newlines should be escaped as \\n in DOT"
    );
}

#[test]
fn dot_epsilon_transitions_skipped() {
    let viz = GrammarVisualizer::new(complex_symbol_grammar());
    let dot = viz.to_dot();
    // Epsilon is skipped in DOT (continue statement in the code)
    // Count edges: 5 non-epsilon symbols in the rule
    let edges: Vec<&str> = dot.lines().filter(|l| l.contains("->")).collect();
    // The rule has 6 symbols, 1 is Epsilon (skipped), so 5 edges
    assert_eq!(edges.len(), 5, "Epsilon should be skipped, expected 5 edges");
}

// ===========================================================================
// Text-based tree rendering
// ===========================================================================

#[test]
fn text_empty_grammar_header() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let text = viz.to_text();
    assert!(text.contains("Grammar: empty"));
    assert!(text.contains(&"=".repeat(50)));
    assert!(text.contains("Tokens:"));
    assert!(text.contains("Rules:"));
}

#[test]
fn text_single_rule_tokens_section() {
    let viz = GrammarVisualizer::new(single_rule_grammar());
    let text = viz.to_text();
    assert!(text.contains("Tokens:"));
    assert!(text.contains("NUMBER"));
    assert!(text.contains(r"/\d+/"));
}

#[test]
fn text_string_vs_regex_pattern_formatting() {
    let mut g = Grammar::new("patterns".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "REGEX_TOK".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "STRING_TOK".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let viz = GrammarVisualizer::new(g);
    let text = viz.to_text();
    assert!(text.contains("/[a-z]+/"), "Regex should be /pattern/");
    assert!(text.contains("\"+\""), "String should be \"pattern\"");
}

#[test]
fn text_single_rule_bnf_notation() {
    let viz = GrammarVisualizer::new(single_rule_grammar());
    let text = viz.to_text();
    assert!(text.contains("Rules:"));
    assert!(text.contains("::="));
    assert!(text.contains("'NUMBER'"));
}

#[test]
fn text_multi_rule_all_rules_present() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let text = viz.to_text();
    let bnf_count = text.matches("::=").count();
    assert!(
        bnf_count >= 6,
        "Expected at least 6 rules, found {} ::= markers",
        bnf_count
    );
}

#[test]
fn text_external_tokens_section() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let text = viz.to_text();
    assert!(text.contains("External Tokens:"));
    assert!(text.contains("INDENT"));
    assert!(text.contains("DEDENT"));
}

#[test]
fn text_no_external_section_when_empty() {
    let viz = GrammarVisualizer::new(single_rule_grammar());
    let text = viz.to_text();
    assert!(!text.contains("External Tokens:"));
    assert!(!text.contains("Conflict Declarations:"));
    assert!(!text.contains("Precedence Declarations:"));
}

#[test]
fn text_external_reference_in_rule() {
    let viz = GrammarVisualizer::new(grammar_with_externals());
    let text = viz.to_text();
    // External symbols are rendered as $id
    assert!(text.contains("$10"), "External ref should show as $id");
}

#[test]
fn text_complex_symbols_rendering() {
    let viz = GrammarVisualizer::new(complex_symbol_grammar());
    let text = viz.to_text();
    assert!(text.contains("A?"), "Optional should render as A?");
    assert!(text.contains("B*"), "Repeat should render as B*");
    assert!(text.contains("A+"), "RepeatOne should render as A+");
    assert!(text.contains("ε"), "Epsilon should render as ε");
    assert!(text.contains("|"), "Choice should use | separator");
}

#[test]
fn text_precedence_metadata_shown() {
    let viz = GrammarVisualizer::new(grammar_with_precedence_and_assoc());
    let text = viz.to_text();
    assert!(text.contains("[precedence:"));
    assert!(text.contains("[associativity:"));
    assert!(text.contains("Left"));
    assert!(text.contains("Right"));
}

#[test]
fn text_precedence_declarations_section() {
    let viz = GrammarVisualizer::new(grammar_with_precedence_decls());
    let text = viz.to_text();
    assert!(text.contains("Precedence Declarations:"));
    assert!(text.contains("Level 1"));
    assert!(text.contains("Level 2"));
}

#[test]
fn text_conflict_declarations_section() {
    let viz = GrammarVisualizer::new(grammar_with_conflicts());
    let text = viz.to_text();
    assert!(text.contains("Conflict Declarations:"));
    assert!(text.contains("GLR"));
}

// ===========================================================================
// SVG railroad diagram
// ===========================================================================

#[test]
fn svg_empty_grammar_well_formed() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
}

#[test]
fn svg_style_classes_defined() {
    let viz = GrammarVisualizer::new(single_rule_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<style>"));
    assert!(svg.contains(".rule-name"));
    assert!(svg.contains(".terminal"));
    assert!(svg.contains(".non-terminal"));
    assert!(svg.contains(".line"));
}

#[test]
fn svg_multi_rule_has_bnf_markers() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let svg = viz.to_railroad_svg();
    let bnf_count = svg.matches("::=").count();
    assert!(bnf_count >= 1, "SVG should have ::= BNF markers");
}

#[test]
fn svg_complex_symbols_rendered() {
    let viz = GrammarVisualizer::new(complex_symbol_grammar());
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("A?"), "Optional should appear as A?");
    assert!(svg.contains("B*"), "Repeat should appear as B*");
    assert!(svg.contains("A+"), "RepeatOne should appear as A+");
    assert!(svg.contains("ε"), "Epsilon should appear as ε");
}

#[test]
fn svg_xml_escapes_special_chars() {
    let viz = GrammarVisualizer::new(special_chars_grammar());
    let svg = viz.to_railroad_svg();
    // XML special characters must be escaped
    assert!(svg.contains("&amp;"), "& should be escaped to &amp;");
    assert!(svg.contains("&lt;"), "< should be escaped to &lt;");
    assert!(svg.contains("&gt;"), "> should be escaped to &gt;");
}

#[test]
fn svg_connecting_lines_for_multi_symbol_rules() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let svg = viz.to_railroad_svg();
    assert!(
        svg.contains("<line"),
        "Multi-symbol rules should have connecting lines"
    );
}

// ===========================================================================
// Dependency graph
// ===========================================================================

#[test]
fn dependency_graph_empty_grammar_header() {
    let viz = GrammarVisualizer::new(empty_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("Symbol Dependencies:"));
    assert!(dep.contains("==================="));
}

#[test]
fn dependency_graph_terminal_only_shows_none() {
    let viz = GrammarVisualizer::new(single_rule_grammar());
    let dep = viz.dependency_graph();
    assert!(dep.contains("depends on:"));
    assert!(
        dep.contains("(none)"),
        "Terminal-only rule should depend on (none)"
    );
}

#[test]
fn dependency_graph_multi_rule_has_nonterminal_deps() {
    let viz = GrammarVisualizer::new(multi_rule_grammar());
    let dep = viz.dependency_graph();
    // expr -> term, term -> factor, factor -> expr (via LPAREN expr RPAREN)
    let dep_lines: Vec<&str> = dep.lines().filter(|l| l.contains("depends on:")).collect();
    assert!(
        dep_lines.len() >= 3,
        "Multi-rule grammar should have at least 3 symbols with dependencies"
    );
    // At least one should have non-trivial deps
    let has_nontrivial = dep_lines.iter().any(|l| !l.contains("(none)"));
    assert!(
        has_nontrivial,
        "Should have at least one non-trivial dependency"
    );
}

// ===========================================================================
// Determinism
// ===========================================================================

#[test]
fn all_outputs_deterministic() {
    let g1 = multi_rule_grammar();
    let g2 = multi_rule_grammar();
    let v1 = GrammarVisualizer::new(g1);
    let v2 = GrammarVisualizer::new(g2);

    assert_eq!(v1.to_dot(), v2.to_dot(), "DOT should be deterministic");
    assert_eq!(
        v1.to_railroad_svg(),
        v2.to_railroad_svg(),
        "SVG should be deterministic"
    );
    assert_eq!(v1.to_text(), v2.to_text(), "Text should be deterministic");

    // Dependency graph uses HashMap, so order may vary; check sorted lines
    let dep1 = v1.dependency_graph();
    let dep2 = v2.dependency_graph();
    let mut lines1: Vec<&str> = dep1.lines().collect();
    let mut lines2: Vec<&str> = dep2.lines().collect();
    lines1.sort();
    lines2.sort();
    assert_eq!(
        lines1, lines2,
        "Dependency graph content should be deterministic"
    );
}

// ===========================================================================
// Non-empty across all formats for various grammars
// ===========================================================================

#[test]
fn all_formats_nonempty_for_each_grammar() {
    let grammars = vec![
        empty_grammar(),
        single_rule_grammar(),
        multi_rule_grammar(),
        grammar_with_externals(),
        complex_symbol_grammar(),
        grammar_with_precedence_and_assoc(),
        grammar_with_conflicts(),
    ];

    for (i, g) in grammars.into_iter().enumerate() {
        let viz = GrammarVisualizer::new(g);
        assert!(!viz.to_dot().is_empty(), "DOT empty for grammar {}", i);
        assert!(
            !viz.to_railroad_svg().is_empty(),
            "SVG empty for grammar {}",
            i
        );
        assert!(!viz.to_text().is_empty(), "Text empty for grammar {}", i);
        assert!(
            !viz.dependency_graph().is_empty(),
            "Dep graph empty for grammar {}",
            i
        );
    }
}
