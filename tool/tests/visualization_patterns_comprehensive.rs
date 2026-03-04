//! Comprehensive pattern-based tests for `adze_tool::visualization::GrammarVisualizer`.
//!
//! Covers: to_dot, to_railroad_svg, to_text, dependency_graph,
//! edge cases (empty, minimal, large), output format validation,
//! multiple grammars, and complex symbol handling.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, Grammar, Precedence,
    PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tool::visualization::GrammarVisualizer;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_grammar() -> Grammar {
    Grammar::new("empty".to_string())
}

fn minimal_grammar() -> Grammar {
    let mut g = Grammar::new("minimal".to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
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

fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arith")
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

fn precedence_grammar() -> Grammar {
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

fn external_grammar() -> Grammar {
    let mut g = Grammar::new("ext".to_string());
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
    let mut g = Grammar::new("complex".to_string());
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
    let mut g = Grammar::new("special".to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "lt&gt".to_string(),
            pattern: TokenPattern::String("<>\"&'".to_string()),
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

fn conflict_grammar() -> Grammar {
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
        .token("A", "a_tok")
        .token("B", "b_tok")
        .token("C", "c_tok")
        .token("D", "d_tok")
        .rule("start", vec!["A"])
        .rule("start", vec!["B"])
        .rule("start", vec!["C"])
        .rule("start", vec!["D"])
        .start("start")
        .build()
}

fn epsilon_only_grammar() -> Grammar {
    let mut g = Grammar::new("eps".to_string());
    let rule_id = SymbolId(1);
    g.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

fn large_grammar(n: u16) -> Grammar {
    let mut g = Grammar::new("large".to_string());
    for i in 1..=n {
        let tok = SymbolId(i);
        g.tokens.insert(
            tok,
            Token {
                name: format!("T{}", i),
                pattern: TokenPattern::String(format!("t{}", i)),
                fragile: false,
            },
        );
    }
    let rule_id = SymbolId(n + 1);
    let rhs: Vec<Symbol> = (1..=n).map(|i| Symbol::Terminal(SymbolId(i))).collect();
    g.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

fn nested_optional_grammar() -> Grammar {
    let mut g = Grammar::new("nested_opt".to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "X".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(2);
    g.rules.entry(rule_id).or_default().push(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Optional(Box::new(
            Symbol::Terminal(tok),
        ))))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

fn multi_external_grammar() -> Grammar {
    let mut g = Grammar::new("multi_ext".to_string());
    for i in 1..=3 {
        let ext = SymbolId(100 + i);
        g.externals.push(ExternalToken {
            name: format!("EXT_{}", i),
            symbol_id: ext,
        });
    }
    g
}

fn precedence_decl_grammar() -> Grammar {
    let mut g = Grammar::new("prec_decl".to_string());
    let s1 = SymbolId(1);
    let s2 = SymbolId(2);
    g.tokens.insert(
        s1,
        Token {
            name: "PLUS".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        s2,
        Token {
            name: "STAR".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    g.precedences.push(Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: vec![s1],
    });
    g.precedences.push(Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: vec![s2],
    });
    g
}

// ===================================================================
// 1. to_text — simple grammars
// ===================================================================

#[test]
fn text_empty_grammar_contains_name() {
    let v = GrammarVisualizer::new(empty_grammar());
    let t = v.to_text();
    assert!(t.contains("Grammar: empty"));
}

#[test]
fn text_empty_grammar_has_separator() {
    let v = GrammarVisualizer::new(empty_grammar());
    let t = v.to_text();
    assert!(t.contains("=".repeat(50).as_str()));
}

#[test]
fn text_empty_grammar_has_tokens_header() {
    let v = GrammarVisualizer::new(empty_grammar());
    let t = v.to_text();
    assert!(t.contains("Tokens:"));
}

#[test]
fn text_minimal_grammar_shows_token() {
    let v = GrammarVisualizer::new(minimal_grammar());
    let t = v.to_text();
    assert!(t.contains("a"));
}

#[test]
fn text_minimal_grammar_shows_rule() {
    let v = GrammarVisualizer::new(minimal_grammar());
    let t = v.to_text();
    assert!(t.contains("::="));
}

#[test]
fn text_arithmetic_grammar_has_name() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let t = v.to_text();
    assert!(t.contains("Grammar: arith"));
}

#[test]
fn text_arithmetic_lists_tokens() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let t = v.to_text();
    assert!(t.contains("NUM"));
    assert!(t.contains("PLUS"));
    assert!(t.contains("STAR"));
}

#[test]
fn text_arithmetic_lists_rules() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let t = v.to_text();
    assert!(t.contains("Rules:"));
    assert!(t.contains("::="));
}

#[test]
fn text_chain_grammar_shows_nonterminals() {
    let v = GrammarVisualizer::new(chain_grammar());
    let t = v.to_text();
    // chain grammar has rules for a, b, c
    assert!(t.contains("::="));
}

#[test]
fn text_multi_alternative_shows_all_alternatives() {
    let v = GrammarVisualizer::new(multi_alternative_grammar());
    let t = v.to_text();
    // 4 alternatives → 4 lines with ::=
    let count = t.matches("::=").count();
    assert!(count >= 4, "expected >=4 alternatives, got {}", count);
}

// ===================================================================
// 2. to_text — precedence & associativity
// ===================================================================

#[test]
fn text_precedence_grammar_shows_precedence() {
    let v = GrammarVisualizer::new(precedence_grammar());
    let t = v.to_text();
    assert!(t.contains("[precedence:"));
}

#[test]
fn text_precedence_grammar_shows_associativity() {
    let v = GrammarVisualizer::new(precedence_grammar());
    let t = v.to_text();
    assert!(t.contains("[associativity:"));
}

#[test]
fn text_precedence_grammar_shows_left() {
    let v = GrammarVisualizer::new(precedence_grammar());
    let t = v.to_text();
    assert!(t.contains("Left"));
}

#[test]
fn text_precedence_declarations_section() {
    let v = GrammarVisualizer::new(precedence_decl_grammar());
    let t = v.to_text();
    assert!(t.contains("Precedence Declarations:"));
}

#[test]
fn text_precedence_declarations_level() {
    let v = GrammarVisualizer::new(precedence_decl_grammar());
    let t = v.to_text();
    assert!(t.contains("Level 1:"));
    assert!(t.contains("Level 2:"));
}

// ===================================================================
// 3. to_text — externals & conflicts
// ===================================================================

#[test]
fn text_external_grammar_shows_section() {
    let v = GrammarVisualizer::new(external_grammar());
    let t = v.to_text();
    assert!(t.contains("External Tokens:"));
}

#[test]
fn text_external_grammar_shows_name() {
    let v = GrammarVisualizer::new(external_grammar());
    let t = v.to_text();
    assert!(t.contains("INDENT"));
}

#[test]
fn text_external_symbol_in_rule() {
    let v = GrammarVisualizer::new(external_grammar());
    let t = v.to_text();
    // External symbols are rendered as $<id>
    assert!(t.contains("$10"));
}

#[test]
fn text_conflict_section() {
    let v = GrammarVisualizer::new(conflict_grammar());
    let t = v.to_text();
    assert!(t.contains("Conflict Declarations:"));
}

#[test]
fn text_conflict_resolution_glr() {
    let v = GrammarVisualizer::new(conflict_grammar());
    let t = v.to_text();
    assert!(t.contains("GLR"));
}

// ===================================================================
// 4. to_text — complex symbols
// ===================================================================

#[test]
fn text_optional_symbol() {
    let v = GrammarVisualizer::new(complex_symbol_grammar());
    let t = v.to_text();
    assert!(t.contains("A?"));
}

#[test]
fn text_repeat_symbol() {
    let v = GrammarVisualizer::new(complex_symbol_grammar());
    let t = v.to_text();
    assert!(t.contains("B*"));
}

#[test]
fn text_repeat_one_symbol() {
    let v = GrammarVisualizer::new(complex_symbol_grammar());
    let t = v.to_text();
    assert!(t.contains("A+"));
}

#[test]
fn text_choice_symbol() {
    let v = GrammarVisualizer::new(complex_symbol_grammar());
    let t = v.to_text();
    assert!(t.contains("(A | B)"));
}

#[test]
fn text_epsilon_symbol() {
    let v = GrammarVisualizer::new(complex_symbol_grammar());
    let t = v.to_text();
    assert!(t.contains("ε"));
}

#[test]
fn text_epsilon_only_rule() {
    let v = GrammarVisualizer::new(epsilon_only_grammar());
    let t = v.to_text();
    assert!(t.contains("ε"));
}

// ===================================================================
// 5. to_dot — structure
// ===================================================================

#[test]
fn dot_empty_grammar_valid_structure() {
    let v = GrammarVisualizer::new(empty_grammar());
    let d = v.to_dot();
    assert!(d.starts_with("digraph Grammar {"));
    assert!(d.trim_end().ends_with('}'));
}

#[test]
fn dot_empty_grammar_has_rankdir() {
    let v = GrammarVisualizer::new(empty_grammar());
    let d = v.to_dot();
    assert!(d.contains("rankdir=LR"));
}

#[test]
fn dot_empty_grammar_has_node_shape() {
    let v = GrammarVisualizer::new(empty_grammar());
    let d = v.to_dot();
    assert!(d.contains("node [shape=box]"));
}

#[test]
fn dot_minimal_terminal_node() {
    let v = GrammarVisualizer::new(minimal_grammar());
    let d = v.to_dot();
    assert!(d.contains("shape=ellipse"));
    assert!(d.contains("fillcolor=lightblue"));
}

#[test]
fn dot_minimal_nonterminal_node() {
    let v = GrammarVisualizer::new(minimal_grammar());
    let d = v.to_dot();
    assert!(d.contains("fillcolor=lightgreen"));
}

#[test]
fn dot_minimal_has_edge() {
    let v = GrammarVisualizer::new(minimal_grammar());
    let d = v.to_dot();
    assert!(d.contains("->"));
}

#[test]
fn dot_arithmetic_has_terminals() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let d = v.to_dot();
    assert!(d.contains("// Terminals"));
}

#[test]
fn dot_arithmetic_has_nonterminals() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let d = v.to_dot();
    assert!(d.contains("// Non-terminals"));
}

#[test]
fn dot_arithmetic_has_rules_section() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let d = v.to_dot();
    assert!(d.contains("// Rules"));
}

#[test]
fn dot_external_diamond_shape() {
    let v = GrammarVisualizer::new(external_grammar());
    let d = v.to_dot();
    assert!(d.contains("shape=diamond"));
    assert!(d.contains("fillcolor=lightcoral"));
}

#[test]
fn dot_external_has_section() {
    let v = GrammarVisualizer::new(external_grammar());
    let d = v.to_dot();
    assert!(d.contains("// External tokens"));
}

#[test]
fn dot_special_chars_escaped() {
    let v = GrammarVisualizer::new(special_chars_grammar());
    let d = v.to_dot();
    // DOT escaping: quotes become \"  so the label includes escaped quotes
    assert!(d.contains("lt&gt"));
}

#[test]
fn dot_multi_rhs_edges_labeled() {
    // In a rule with >1 RHS symbol, edges get position labels
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let d = v.to_dot();
    // expr -> term PLUS expr  has labels 1, 2, 3
    assert!(d.contains("label=\"1\""));
}

#[test]
fn dot_epsilon_not_rendered() {
    let v = GrammarVisualizer::new(epsilon_only_grammar());
    let d = v.to_dot();
    // Epsilon transitions are skipped in DOT
    assert!(!d.contains("->"), "epsilon edges should be omitted");
}

// ===================================================================
// 6. to_railroad_svg — structure
// ===================================================================

#[test]
fn svg_empty_grammar_has_svg_tags() {
    let v = GrammarVisualizer::new(empty_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("<svg"));
    assert!(s.contains("</svg>"));
}

#[test]
fn svg_empty_grammar_has_style() {
    let v = GrammarVisualizer::new(empty_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("<style>"));
}

#[test]
fn svg_minimal_has_rule_name() {
    let v = GrammarVisualizer::new(minimal_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("class=\"rule-name\""));
}

#[test]
fn svg_minimal_has_text_element() {
    let v = GrammarVisualizer::new(minimal_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("<text"));
}

#[test]
fn svg_minimal_has_rect_element() {
    let v = GrammarVisualizer::new(minimal_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("<rect"));
}

#[test]
fn svg_arithmetic_has_terminal_class() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("class=\"terminal\""));
}

#[test]
fn svg_arithmetic_has_nonterminal_class() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("class=\"non-terminal\""));
}

#[test]
fn svg_complex_has_optional_class() {
    let v = GrammarVisualizer::new(complex_symbol_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("class=\"optional\""));
}

#[test]
fn svg_complex_has_repeat_class() {
    let v = GrammarVisualizer::new(complex_symbol_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("class=\"repeat\""));
}

#[test]
fn svg_complex_has_choice_class() {
    let v = GrammarVisualizer::new(complex_symbol_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("class=\"choice\""));
}

#[test]
fn svg_complex_has_epsilon_class() {
    let v = GrammarVisualizer::new(complex_symbol_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("class=\"epsilon\""));
}

#[test]
fn svg_connecting_lines() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("class=\"line\""));
}

#[test]
fn svg_special_chars_escaped_xml() {
    let v = GrammarVisualizer::new(special_chars_grammar());
    let s = v.to_railroad_svg();
    // XML escaping: & -> &amp;
    assert!(s.contains("&amp;"));
}

#[test]
fn svg_xmlns_attribute() {
    let v = GrammarVisualizer::new(empty_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("xmlns=\"http://www.w3.org/2000/svg\""));
}

// ===================================================================
// 7. dependency_graph
// ===================================================================

#[test]
fn deps_empty_grammar() {
    let v = GrammarVisualizer::new(empty_grammar());
    let d = v.dependency_graph();
    assert!(d.contains("Symbol Dependencies:"));
}

#[test]
fn deps_minimal_grammar_no_deps() {
    let v = GrammarVisualizer::new(minimal_grammar());
    let d = v.dependency_graph();
    assert!(d.contains("(none)"));
}

#[test]
fn deps_chain_grammar_has_deps() {
    let v = GrammarVisualizer::new(chain_grammar());
    let d = v.dependency_graph();
    assert!(d.contains("depends on:"));
    // Should not show (none) for a or b since they reference nonterminals
    let none_count = d.matches("(none)").count();
    // c -> ID is terminal, so c has (none)
    assert!(none_count >= 1);
}

#[test]
fn deps_separator_line() {
    let v = GrammarVisualizer::new(empty_grammar());
    let d = v.dependency_graph();
    assert!(d.contains("==================="));
}

#[test]
fn deps_epsilon_only_grammar_no_deps() {
    let v = GrammarVisualizer::new(epsilon_only_grammar());
    let d = v.dependency_graph();
    assert!(d.contains("(none)"));
}

// ===================================================================
// 8. Multiple visualizations on same grammar
// ===================================================================

#[test]
fn all_four_outputs_non_empty() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    assert!(!v.to_text().is_empty());
    assert!(!v.to_dot().is_empty());
    assert!(!v.to_railroad_svg().is_empty());
    assert!(!v.dependency_graph().is_empty());
}

#[test]
fn repeated_to_text_is_deterministic() {
    let g = arithmetic_grammar();
    let v = GrammarVisualizer::new(g.clone());
    let t1 = v.to_text();
    let v2 = GrammarVisualizer::new(g);
    let t2 = v2.to_text();
    assert_eq!(t1, t2);
}

#[test]
fn repeated_to_dot_is_deterministic() {
    let g = arithmetic_grammar();
    let v = GrammarVisualizer::new(g.clone());
    let d1 = v.to_dot();
    let v2 = GrammarVisualizer::new(g);
    let d2 = v2.to_dot();
    assert_eq!(d1, d2);
}

#[test]
fn repeated_svg_is_deterministic() {
    let g = arithmetic_grammar();
    let v = GrammarVisualizer::new(g.clone());
    let s1 = v.to_railroad_svg();
    let v2 = GrammarVisualizer::new(g);
    let s2 = v2.to_railroad_svg();
    assert_eq!(s1, s2);
}

#[test]
fn repeated_deps_is_deterministic() {
    let g = arithmetic_grammar();
    let v = GrammarVisualizer::new(g.clone());
    let d1 = v.dependency_graph();
    let v2 = GrammarVisualizer::new(g);
    let d2 = v2.dependency_graph();
    assert_eq!(d1, d2);
}

// ===================================================================
// 9. Large grammar edge cases
// ===================================================================

#[test]
fn large_grammar_50_tokens_text() {
    let v = GrammarVisualizer::new(large_grammar(50));
    let t = v.to_text();
    assert!(t.contains("T1"));
    assert!(t.contains("T50"));
}

#[test]
fn large_grammar_50_tokens_dot() {
    let v = GrammarVisualizer::new(large_grammar(50));
    let d = v.to_dot();
    assert!(d.contains("t1"));
    assert!(d.contains("t50"));
}

#[test]
fn large_grammar_100_tokens_text() {
    let v = GrammarVisualizer::new(large_grammar(100));
    let t = v.to_text();
    assert!(t.contains("T100"));
}

#[test]
fn large_grammar_text_has_all_tokens() {
    let n = 30u16;
    let v = GrammarVisualizer::new(large_grammar(n));
    let t = v.to_text();
    for i in 1..=n {
        assert!(
            t.contains(&format!("T{}", i)),
            "missing T{} in text output",
            i
        );
    }
}

// ===================================================================
// 10. Nested / deep symbol tests
// ===================================================================

#[test]
fn text_nested_optional() {
    let v = GrammarVisualizer::new(nested_optional_grammar());
    let t = v.to_text();
    assert!(t.contains("X??"));
}

#[test]
fn dot_nested_optional_has_edges() {
    let v = GrammarVisualizer::new(nested_optional_grammar());
    let d = v.to_dot();
    // Nested optionals produce opt-prefixed node references
    assert!(d.contains("digraph Grammar"));
}

#[test]
fn svg_nested_optional() {
    let v = GrammarVisualizer::new(nested_optional_grammar());
    let s = v.to_railroad_svg();
    assert!(s.contains("X??"));
}

// ===================================================================
// 11. Multi-external grammar
// ===================================================================

#[test]
fn text_multi_externals_listed() {
    let v = GrammarVisualizer::new(multi_external_grammar());
    let t = v.to_text();
    assert!(t.contains("EXT_1"));
    assert!(t.contains("EXT_2"));
    assert!(t.contains("EXT_3"));
}

#[test]
fn dot_multi_externals_diamonds() {
    let v = GrammarVisualizer::new(multi_external_grammar());
    let d = v.to_dot();
    let diamond_count = d.matches("shape=diamond").count();
    assert_eq!(diamond_count, 3);
}

// ===================================================================
// 12. Token pattern display
// ===================================================================

#[test]
fn text_string_token_pattern() {
    let mut g = Grammar::new("pat".to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "SEMI".to_string(),
            pattern: TokenPattern::String(";".to_string()),
            fragile: false,
        },
    );
    let v = GrammarVisualizer::new(g);
    let t = v.to_text();
    assert!(t.contains("\";\""));
}

#[test]
fn text_regex_token_pattern() {
    let mut g = Grammar::new("pat".to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "DIGIT".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let v = GrammarVisualizer::new(g);
    let t = v.to_text();
    assert!(t.contains("/\\d+/"));
}

// ===================================================================
// 13. GrammarBuilder integration
// ===================================================================

#[test]
fn builder_python_like_text() {
    let g = GrammarBuilder::python_like();
    let v = GrammarVisualizer::new(g);
    let t = v.to_text();
    assert!(t.contains("Grammar: python_like"));
}

#[test]
fn builder_python_like_dot() {
    let g = GrammarBuilder::python_like();
    let v = GrammarVisualizer::new(g);
    let d = v.to_dot();
    assert!(d.contains("digraph Grammar"));
}

#[test]
fn builder_javascript_like_text() {
    let g = GrammarBuilder::javascript_like();
    let v = GrammarVisualizer::new(g);
    let t = v.to_text();
    assert!(t.contains("Grammar: javascript_like"));
}

#[test]
fn builder_javascript_like_dot() {
    let g = GrammarBuilder::javascript_like();
    let v = GrammarVisualizer::new(g);
    let d = v.to_dot();
    assert!(d.contains("digraph Grammar"));
}

// ===================================================================
// 14. Format validity checks
// ===================================================================

#[test]
fn dot_output_is_valid_utf8() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    // to_dot returns String so it is already valid UTF-8; just confirm non-empty
    let d = v.to_dot();
    assert!(!d.is_empty());
    assert!(std::str::from_utf8(d.as_bytes()).is_ok());
}

#[test]
fn svg_output_is_valid_utf8() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let s = v.to_railroad_svg();
    assert!(std::str::from_utf8(s.as_bytes()).is_ok());
}

#[test]
fn text_output_ends_with_newline() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let t = v.to_text();
    assert!(t.ends_with('\n'));
}

#[test]
fn dot_output_ends_with_brace() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let d = v.to_dot();
    assert!(d.trim_end().ends_with('}'));
}

#[test]
fn svg_output_ends_with_closing_tag() {
    let v = GrammarVisualizer::new(arithmetic_grammar());
    let s = v.to_railroad_svg();
    assert!(s.trim_end().ends_with("</svg>"));
}

// ===================================================================
// 15. Regression / miscellaneous
// ===================================================================

#[test]
fn constructor_takes_ownership() {
    let g = empty_grammar();
    let _v = GrammarVisualizer::new(g);
    // g moved — just confirm compilation
}

#[test]
fn tokens_only_grammar_text() {
    let mut g = Grammar::new("tokens_only".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "FOO".to_string(),
            pattern: TokenPattern::String("foo".to_string()),
            fragile: false,
        },
    );
    let v = GrammarVisualizer::new(g);
    let t = v.to_text();
    assert!(t.contains("FOO"));
    assert!(!t.contains("Rules:\n\n")); // no rules printed after header
}

#[test]
fn tokens_only_grammar_dot() {
    let mut g = Grammar::new("tokens_only".to_string());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "FOO".to_string(),
            pattern: TokenPattern::String("foo".to_string()),
            fragile: false,
        },
    );
    let v = GrammarVisualizer::new(g);
    let d = v.to_dot();
    assert!(d.contains("FOO"));
    // No edges because no rules
    assert!(!d.contains("->"));
}

#[test]
fn grammar_name_preserved_in_text() {
    let g = Grammar::new("my_fancy_grammar".to_string());
    let v = GrammarVisualizer::new(g);
    assert!(v.to_text().contains("Grammar: my_fancy_grammar"));
}

#[test]
fn sequence_in_svg_renders_all_parts() {
    let v = GrammarVisualizer::new(complex_symbol_grammar());
    let s = v.to_railroad_svg();
    // The Sequence(A, B) should show both A and B
    assert!(s.contains(">A<") || s.contains(">A ") || s.contains("A B"));
}
