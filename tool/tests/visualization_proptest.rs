#![allow(clippy::needless_range_loop)]

//! Property-based tests for grammar visualization in adze-tool.
//!
//! Uses proptest to validate invariants of the various visualization outputs
//! produced by `GrammarVisualizer`:
//!   - DOT output is syntactically valid
//!   - Text output includes grammar name
//!   - Visualization of simple and complex grammars
//!   - Multiple rules produce connected graphs
//!   - Empty grammar visualization
//!   - Special characters in names are escaped
//!   - Output is deterministic

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, Grammar, PrecedenceKind,
    ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tool::visualization::GrammarVisualizer;
use proptest::prelude::*;

// ===========================================================================
// Strategies
// ===========================================================================

/// A valid grammar name (lowercase + digits + underscores, starting with letter).
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_filter("must not be empty", |s| !s.is_empty())
}

/// A valid token name (uppercase letters + underscores).
fn token_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9_]{0,8}".prop_filter("must not be empty", |s| !s.is_empty())
}

/// A simple regex pattern for tokens.
fn token_pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r"[a-z]+".to_string()),
        Just(r"\d+".to_string()),
        Just(r"[a-zA-Z_]+".to_string()),
        Just(r"\w+".to_string()),
    ]
}

/// Number of rules to generate (1..=5).
fn rule_count_strategy() -> impl Strategy<Value = usize> {
    1..=5usize
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Build a grammar with N tokens and a single rule referencing them all.
fn grammar_with_n_tokens(name: &str, n: usize) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());
    let mut token_ids = Vec::new();
    for i in 0..n {
        let id = SymbolId((i + 1) as u16);
        grammar.tokens.insert(
            id,
            Token {
                name: format!("T{}", i),
                pattern: TokenPattern::Regex(format!("[a-z]{{{}}}", i + 1)),
                fragile: false,
            },
        );
        token_ids.push(id);
    }
    if !token_ids.is_empty() {
        let rule_id = SymbolId((n + 1) as u16);
        let rhs: Vec<Symbol> = token_ids.iter().map(|id| Symbol::Terminal(*id)).collect();
        grammar.rules.entry(rule_id).or_default().push(Rule {
            lhs: rule_id,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
    }
    grammar
}

/// Build a grammar with multiple non-terminal rules that reference each other.
fn grammar_with_chain(name: &str, depth: usize) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());
    // One terminal
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "LEAF".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    // Chain: rule_N -> rule_{N-1} -> ... -> rule_0 -> LEAF
    for i in 0..depth {
        let rule_id = SymbolId((i + 10) as u16);
        let rhs = if i == 0 {
            vec![Symbol::Terminal(tok)]
        } else {
            vec![Symbol::NonTerminal(SymbolId((i + 10 - 1) as u16))]
        };
        grammar.rules.entry(rule_id).or_default().push(Rule {
            lhs: rule_id,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }
    grammar
}

/// Validate basic DOT structure: starts with `digraph`, contains braces, etc.
fn is_valid_dot(dot: &str) -> bool {
    let trimmed = dot.trim();
    trimmed.starts_with("digraph ")
        && trimmed.ends_with('}')
        && trimmed.contains('{')
        && trimmed.contains("rankdir=LR")
        && trimmed.contains("node [shape=box]")
}

/// Count edges in DOT output.
fn count_dot_edges(dot: &str) -> usize {
    dot.lines().filter(|l| l.contains(" -> ")).count()
}

/// Count node declarations in DOT output (lines with `[label=`).
fn count_dot_nodes(dot: &str) -> usize {
    dot.lines()
        .filter(|l| l.contains("[label=") && !l.contains("->"))
        .count()
}

// ===========================================================================
// Tests
// ===========================================================================

// ---- 1. DOT output structural properties ----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// DOT output is always valid DOT syntax regardless of grammar name.
    #[test]
    fn dot_output_is_valid_syntax(name in grammar_name_strategy()) {
        let grammar = Grammar::new(name);
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        prop_assert!(is_valid_dot(&dot), "Invalid DOT output: {}", dot);
    }

    /// DOT output with tokens has terminal nodes.
    #[test]
    fn dot_terminals_have_ellipse_shape(n in 1..=5usize) {
        let grammar = grammar_with_n_tokens("test", n);
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        // Each terminal should produce a node with shape=ellipse
        for i in 0..n {
            prop_assert!(dot.contains(&format!("t{}", i + 1)), "Missing terminal t{}", i + 1);
        }
        prop_assert!(dot.contains("shape=ellipse"), "No ellipse-shaped nodes found");
    }

    /// DOT output with rules has non-terminal nodes with lightgreen fill.
    #[test]
    fn dot_nonterminals_have_green_fill(depth in 1..=4usize) {
        let grammar = grammar_with_chain("test", depth);
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        prop_assert!(dot.contains("lightgreen"), "Non-terminals should have lightgreen fill");
    }

    /// DOT edges connect non-terminals to their RHS symbols.
    #[test]
    fn dot_edges_match_rule_count(n in 1..=5usize) {
        let grammar = grammar_with_n_tokens("test", n);
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        let edge_count = count_dot_edges(&dot);
        // One rule with N terminals → N edges
        prop_assert_eq!(edge_count, n, "Expected {} edges, got {}", n, edge_count);
    }

    /// DOT node count matches token + rule count.
    #[test]
    fn dot_node_count_matches(n in 1..=5usize) {
        let grammar = grammar_with_n_tokens("test", n);
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        let node_count = count_dot_nodes(&dot);
        // n tokens + 1 rule
        prop_assert_eq!(node_count, n + 1, "Expected {} nodes, got {}", n + 1, node_count);
    }
}

// ---- 2. Text output properties ----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Text output always includes the grammar name.
    #[test]
    fn text_includes_grammar_name(name in grammar_name_strategy()) {
        let grammar = Grammar::new(name.clone());
        let viz = GrammarVisualizer::new(grammar);
        let text = viz.to_text();
        prop_assert!(text.contains(&format!("Grammar: {}", name)),
            "Text should include grammar name '{}', got:\n{}", name, text);
    }

    /// Text output always includes "Tokens:" and "Rules:" sections.
    #[test]
    fn text_has_required_sections(name in grammar_name_strategy()) {
        let grammar = Grammar::new(name);
        let viz = GrammarVisualizer::new(grammar);
        let text = viz.to_text();
        prop_assert!(text.contains("Tokens:"), "Missing 'Tokens:' section");
        prop_assert!(text.contains("Rules:"), "Missing 'Rules:' section");
    }

    /// Text output lists all tokens.
    #[test]
    fn text_lists_all_tokens(n in 1..=5usize) {
        let grammar = grammar_with_n_tokens("test", n);
        let viz = GrammarVisualizer::new(grammar);
        let text = viz.to_text();
        for i in 0..n {
            prop_assert!(text.contains(&format!("T{}", i)),
                "Token T{} not found in text output", i);
        }
    }

    /// Text output includes `::=` for every rule.
    #[test]
    fn text_rules_have_production_arrow(depth in 1..=4usize) {
        let grammar = grammar_with_chain("test", depth);
        let viz = GrammarVisualizer::new(grammar);
        let text = viz.to_text();
        let arrow_count = text.matches("::=").count();
        prop_assert_eq!(arrow_count, depth,
            "Expected {} productions, got {}", depth, arrow_count);
    }
}

// ---- 3. Simple grammar visualization ----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Single-token grammar produces valid DOT and text.
    #[test]
    fn single_token_grammar_visualizes(
        gname in grammar_name_strategy(),
        tname in token_name_strategy(),
    ) {
        let mut grammar = Grammar::new(gname.clone());
        let tok = SymbolId(1);
        grammar.tokens.insert(tok, Token {
            name: tname.clone(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        });
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
        let dot = viz.to_dot();
        let text = viz.to_text();
        prop_assert!(is_valid_dot(&dot));
        prop_assert!(text.contains(&gname));
        prop_assert!(text.contains(&tname));
    }
}

// ---- 4. Complex grammar visualization ----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// Grammars with Optional/Repeat/Choice symbols produce valid DOT.
    #[test]
    fn complex_symbols_produce_valid_dot(name in grammar_name_strategy()) {
        let mut grammar = Grammar::new(name);
        let tok_a = SymbolId(1);
        let tok_b = SymbolId(2);
        grammar.tokens.insert(tok_a, Token {
            name: "A".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        });
        grammar.tokens.insert(tok_b, Token {
            name: "B".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        });
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
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        prop_assert!(is_valid_dot(&dot));
    }

    /// Complex symbols in text output show correct notation.
    #[test]
    fn complex_symbols_text_notation(name in grammar_name_strategy()) {
        let mut grammar = Grammar::new(name);
        let tok = SymbolId(1);
        grammar.tokens.insert(tok, Token {
            name: "X".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        });
        let rule_id = SymbolId(5);
        grammar.rules.entry(rule_id).or_default().push(Rule {
            lhs: rule_id,
            rhs: vec![
                Symbol::Optional(Box::new(Symbol::Terminal(tok))),
                Symbol::Repeat(Box::new(Symbol::Terminal(tok))),
                Symbol::RepeatOne(Box::new(Symbol::Terminal(tok))),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        let viz = GrammarVisualizer::new(grammar);
        let text = viz.to_text();
        prop_assert!(text.contains("X?"), "Missing Optional notation");
        prop_assert!(text.contains("X*"), "Missing Repeat notation");
        prop_assert!(text.contains("X+"), "Missing RepeatOne notation");
    }
}

// ---- 5. Multiple rules and connected graphs ----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Chain grammars produce edges connecting consecutive non-terminals.
    #[test]
    fn chain_grammar_has_connected_edges(depth in 2..=5usize) {
        let grammar = grammar_with_chain("chain", depth);
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        let edge_count = count_dot_edges(&dot);
        // Each rule in the chain has exactly 1 RHS symbol → depth edges
        prop_assert_eq!(edge_count, depth,
            "Chain of depth {} should have {} edges, got {}", depth, depth, edge_count);
    }

    /// Dependency graph lists all non-terminal symbols.
    #[test]
    fn dependency_graph_lists_all_rules(depth in 1..=4usize) {
        let grammar = grammar_with_chain("dep", depth);
        let viz = GrammarVisualizer::new(grammar);
        let dep = viz.dependency_graph();
        prop_assert!(dep.contains("Symbol Dependencies:"),
            "Missing header in dependency graph");
        prop_assert!(dep.contains("depends on:"),
            "Missing dependency info");
    }

    /// Multiple alternative rules for the same LHS produce correct edge count.
    #[test]
    fn multiple_alternatives_edge_count(alts in 1..=4usize) {
        let mut grammar = Grammar::new("alts".to_string());
        let tok = SymbolId(1);
        grammar.tokens.insert(tok, Token {
            name: "T".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        });
        let rule_id = SymbolId(5);
        for i in 0..alts {
            grammar.rules.entry(rule_id).or_default().push(Rule {
                lhs: rule_id,
                rhs: vec![Symbol::Terminal(tok)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        let edge_count = count_dot_edges(&dot);
        prop_assert_eq!(edge_count, alts,
            "Expected {} edges for {} alternatives, got {}", alts, alts, edge_count);
    }
}

// ---- 6. Empty grammar visualization ----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Empty grammar DOT output is valid.
    #[test]
    fn empty_grammar_dot_is_valid(name in grammar_name_strategy()) {
        let grammar = Grammar::new(name);
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        prop_assert!(is_valid_dot(&dot));
        prop_assert_eq!(count_dot_edges(&dot), 0, "Empty grammar should have no edges");
        prop_assert_eq!(count_dot_nodes(&dot), 0, "Empty grammar should have no nodes");
    }

    /// Empty grammar text includes name and section headers.
    #[test]
    fn empty_grammar_text_has_sections(name in grammar_name_strategy()) {
        let grammar = Grammar::new(name.clone());
        let viz = GrammarVisualizer::new(grammar);
        let text = viz.to_text();
        prop_assert!(text.contains(&format!("Grammar: {}", name)),
            "Missing grammar name in text output");
        prop_assert!(text.contains("Tokens:"));
        prop_assert!(text.contains("Rules:"));
        // No external tokens section for empty grammar
        prop_assert!(!text.contains("External Tokens:"),
            "Empty grammar should not have External Tokens section");
    }

    /// Empty grammar dependency graph has header but no entries.
    #[test]
    fn empty_grammar_dependency_graph(name in grammar_name_strategy()) {
        let grammar = Grammar::new(name);
        let viz = GrammarVisualizer::new(grammar);
        let dep = viz.dependency_graph();
        prop_assert!(dep.contains("Symbol Dependencies:"));
        prop_assert!(!dep.contains("depends on:"),
            "Empty grammar should have no dependencies");
    }
}

// ---- 7. Special characters in names ----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// DOT output escapes double quotes in token names.
    #[test]
    fn dot_escapes_quotes_in_names(_seed in 0..10u32) {
        let mut grammar = Grammar::new("special".to_string());
        let tok = SymbolId(1);
        grammar.tokens.insert(tok, Token {
            name: r#"say"hello""#.to_string(),
            pattern: TokenPattern::String(r#""hello""#.to_string()),
            fragile: false,
        });
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
        let dot = viz.to_dot();
        // Escaped quotes should appear as \"
        prop_assert!(dot.contains(r#"\""#), "Quotes should be escaped in DOT output");
        prop_assert!(is_valid_dot(&dot));
    }

    /// DOT output escapes backslashes in token names.
    #[test]
    fn dot_escapes_backslashes(_seed in 0..10u32) {
        let mut grammar = Grammar::new("bs".to_string());
        let tok = SymbolId(1);
        grammar.tokens.insert(tok, Token {
            name: r"back\slash".to_string(),
            pattern: TokenPattern::String(r"\".to_string()),
            fragile: false,
        });
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        prop_assert!(dot.contains(r"\\"), "Backslashes should be escaped");
        prop_assert!(is_valid_dot(&dot));
    }

    /// SVG/railroad output escapes XML special characters.
    #[test]
    fn svg_escapes_xml_special_chars(_seed in 0..10u32) {
        let mut grammar = Grammar::new("xml".to_string());
        let tok = SymbolId(1);
        grammar.tokens.insert(tok, Token {
            name: "lt<gt>amp&".to_string(),
            pattern: TokenPattern::String("<>&".to_string()),
            fragile: false,
        });
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
        // Should not contain raw < or > or & (outside tags)
        prop_assert!(svg.contains("&amp;"), "& should be escaped to &amp;");
        prop_assert!(svg.contains("&lt;"), "< should be escaped to &lt;");
        prop_assert!(svg.contains("&gt;"), "> should be escaped to &gt;");
    }

    /// Newlines in token names are escaped in DOT output.
    #[test]
    fn dot_escapes_newlines(_seed in 0..10u32) {
        let mut grammar = Grammar::new("nl".to_string());
        let tok = SymbolId(1);
        grammar.tokens.insert(tok, Token {
            name: "line\nbreak".to_string(),
            pattern: TokenPattern::String("\n".to_string()),
            fragile: false,
        });
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        prop_assert!(dot.contains(r"\n"), "Newlines should be escaped");
        prop_assert!(is_valid_dot(&dot));
    }
}

// ---- 8. Deterministic output ----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// DOT output is deterministic across multiple invocations.
    #[test]
    fn dot_output_is_deterministic(name in grammar_name_strategy()) {
        let g1 = grammar_with_n_tokens(&name, 3);
        let g2 = grammar_with_n_tokens(&name, 3);
        let dot1 = GrammarVisualizer::new(g1).to_dot();
        let dot2 = GrammarVisualizer::new(g2).to_dot();
        prop_assert_eq!(dot1, dot2, "DOT output should be deterministic");
    }

    /// Text output is deterministic across multiple invocations.
    #[test]
    fn text_output_is_deterministic(name in grammar_name_strategy()) {
        let g1 = grammar_with_n_tokens(&name, 3);
        let g2 = grammar_with_n_tokens(&name, 3);
        let text1 = GrammarVisualizer::new(g1).to_text();
        let text2 = GrammarVisualizer::new(g2).to_text();
        prop_assert_eq!(text1, text2, "Text output should be deterministic");
    }

    /// SVG output is deterministic across multiple invocations.
    #[test]
    fn svg_output_is_deterministic(name in grammar_name_strategy()) {
        let g1 = grammar_with_n_tokens(&name, 3);
        let g2 = grammar_with_n_tokens(&name, 3);
        let svg1 = GrammarVisualizer::new(g1).to_railroad_svg();
        let svg2 = GrammarVisualizer::new(g2).to_railroad_svg();
        prop_assert_eq!(svg1, svg2, "SVG output should be deterministic");
    }

    /// Dependency graph contains the same entries across multiple invocations.
    #[test]
    fn dependency_graph_has_same_content(depth in 1..=4usize) {
        let g1 = grammar_with_chain("det", depth);
        let g2 = grammar_with_chain("det", depth);
        let dep1 = GrammarVisualizer::new(g1).dependency_graph();
        let dep2 = GrammarVisualizer::new(g2).dependency_graph();
        // HashMap iteration order may vary, so compare sorted lines
        let mut lines1: Vec<&str> = dep1.lines().collect();
        let mut lines2: Vec<&str> = dep2.lines().collect();
        lines1.sort();
        lines2.sort();
        prop_assert_eq!(lines1, lines2, "Dependency graph content should be the same");
    }
}

// ---- 9. Additional property tests ----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// External tokens appear in DOT output with diamond shape.
    #[test]
    fn external_tokens_in_dot(_seed in 0..10u32) {
        let mut grammar = Grammar::new("ext".to_string());
        let ext = SymbolId(10);
        grammar.externals.push(ExternalToken {
            name: "INDENT".to_string(),
            symbol_id: ext,
        });
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        prop_assert!(dot.contains("shape=diamond"), "Externals should have diamond shape");
        prop_assert!(dot.contains("lightcoral"), "Externals should have lightcoral fill");
        prop_assert!(dot.contains("INDENT"), "External name should appear in DOT");
    }

    /// Precedence and associativity metadata appear in text output.
    #[test]
    fn precedence_metadata_in_text(_seed in 0..10u32) {
        let grammar = GrammarBuilder::new("prec")
            .token("NUM", r"\d+")
            .token("PLUS", r"\+")
            .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        let viz = GrammarVisualizer::new(grammar);
        let text = viz.to_text();
        prop_assert!(text.contains("precedence"), "Text should show precedence");
        prop_assert!(text.contains("associativity"), "Text should show associativity");
    }

    /// Railroad SVG output contains proper SVG structure.
    #[test]
    fn svg_has_valid_structure(n in 1..=3usize) {
        let grammar = grammar_with_n_tokens("svg", n);
        let viz = GrammarVisualizer::new(grammar);
        let svg = viz.to_railroad_svg();
        prop_assert!(svg.contains("<svg"), "SVG should start with <svg tag");
        prop_assert!(svg.contains("</svg>"), "SVG should end with </svg>");
        prop_assert!(svg.contains("<style>"), "SVG should contain styles");
    }

    /// Epsilon symbols are skipped in DOT edges but shown in text.
    #[test]
    fn epsilon_handling(name in grammar_name_strategy()) {
        let mut grammar = Grammar::new(name);
        let rule_id = SymbolId(1);
        grammar.rules.entry(rule_id).or_default().push(Rule {
            lhs: rule_id,
            rhs: vec![Symbol::Epsilon],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        let text = viz.to_text();
        // Epsilon transitions are skipped in DOT
        prop_assert_eq!(count_dot_edges(&dot), 0,
            "Epsilon should produce no edges in DOT");
        // But shown in text
        prop_assert!(text.contains("ε"), "Epsilon should appear in text as ε");
    }

    /// Conflict declarations appear in text output.
    #[test]
    fn conflicts_in_text(_seed in 0..10u32) {
        let mut grammar = Grammar::new("conflicts".to_string());
        grammar.conflicts.push(ConflictDeclaration {
            symbols: vec![SymbolId(1), SymbolId(2)],
            resolution: ConflictResolution::GLR,
        });
        let viz = GrammarVisualizer::new(grammar);
        let text = viz.to_text();
        prop_assert!(text.contains("Conflict Declarations:"),
            "Text should include conflict section");
        prop_assert!(text.contains("GLR"), "Text should show GLR resolution");
    }
}

// ---- 10. Enum-type (multiple-alternative) grammar visualization ----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(15))]

    /// Enum-style grammar (one LHS with many Choice alternatives) renders all variants in text.
    #[test]
    fn enum_choice_variants_in_text(variant_count in 2..=5usize) {
        let mut grammar = Grammar::new("enum_lang".to_string());
        // Create tokens for each variant
        let mut tok_ids = Vec::new();
        for i in 0..variant_count {
            let id = SymbolId((i + 1) as u16);
            grammar.tokens.insert(id, Token {
                name: format!("VAR{}", i),
                pattern: TokenPattern::String(format!("v{}", i)),
                fragile: false,
            });
            tok_ids.push(id);
        }
        // Single rule with a Choice symbol containing all variants
        let rule_id = SymbolId(100);
        let choices: Vec<Symbol> = tok_ids.iter().map(|id| Symbol::Terminal(*id)).collect();
        grammar.rules.entry(rule_id).or_default().push(Rule {
            lhs: rule_id,
            rhs: vec![Symbol::Choice(choices)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        let viz = GrammarVisualizer::new(grammar);
        let text = viz.to_text();
        // All variant tokens should appear in the choice notation
        for i in 0..variant_count {
            prop_assert!(text.contains(&format!("VAR{}", i)),
                "Variant VAR{} missing from text output", i);
        }
        // Choice uses pipe separator in text
        prop_assert!(text.contains("|"), "Choice should use | separator in text");
    }

    /// Enum-style grammar with separate alternative rules (one per variant) lists each rule.
    #[test]
    fn enum_separate_alternatives_in_text(variant_count in 2..=5usize) {
        let mut grammar = Grammar::new("enum_sep".to_string());
        for i in 0..variant_count {
            let id = SymbolId((i + 1) as u16);
            grammar.tokens.insert(id, Token {
                name: format!("KIND{}", i),
                pattern: TokenPattern::String(format!("k{}", i)),
                fragile: false,
            });
        }
        let rule_id = SymbolId(50);
        for i in 0..variant_count {
            grammar.rules.entry(rule_id).or_default().push(Rule {
                lhs: rule_id,
                rhs: vec![Symbol::Terminal(SymbolId((i + 1) as u16))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        let viz = GrammarVisualizer::new(grammar);
        let text = viz.to_text();
        let arrow_count = text.matches("::=").count();
        prop_assert_eq!(arrow_count, variant_count,
            "Expected {} rule lines for enum alternatives, got {}", variant_count, arrow_count);
    }

    /// Enum-style DOT output has edges for each variant.
    #[test]
    fn enum_dot_has_edges_per_variant(variant_count in 2..=5usize) {
        let mut grammar = Grammar::new("enum_dot".to_string());
        for i in 0..variant_count {
            let id = SymbolId((i + 1) as u16);
            grammar.tokens.insert(id, Token {
                name: format!("E{}", i),
                pattern: TokenPattern::String(format!("e{}", i)),
                fragile: false,
            });
        }
        let rule_id = SymbolId(50);
        for i in 0..variant_count {
            grammar.rules.entry(rule_id).or_default().push(Rule {
                lhs: rule_id,
                rhs: vec![Symbol::Terminal(SymbolId((i + 1) as u16))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        let viz = GrammarVisualizer::new(grammar);
        let dot = viz.to_dot();
        let edge_count = count_dot_edges(&dot);
        prop_assert_eq!(edge_count, variant_count,
            "Enum with {} variants should have {} edges, got {}", variant_count, variant_count, edge_count);
    }

    /// Enum-style grammar SVG renders all variant names.
    #[test]
    fn enum_svg_renders_variant_names(variant_count in 2..=4usize) {
        let mut grammar = Grammar::new("enum_svg".to_string());
        for i in 0..variant_count {
            let id = SymbolId((i + 1) as u16);
            grammar.tokens.insert(id, Token {
                name: format!("SV{}", i),
                pattern: TokenPattern::String(format!("s{}", i)),
                fragile: false,
            });
        }
        let rule_id = SymbolId(50);
        for i in 0..variant_count {
            grammar.rules.entry(rule_id).or_default().push(Rule {
                lhs: rule_id,
                rhs: vec![Symbol::Terminal(SymbolId((i + 1) as u16))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        let viz = GrammarVisualizer::new(grammar);
        let svg = viz.to_railroad_svg();
        for i in 0..variant_count {
            prop_assert!(svg.contains(&format!("SV{}", i)),
                "SVG should contain variant name SV{}", i);
        }
    }
}
