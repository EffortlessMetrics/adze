//! Property-based and unit tests for the tool crate grammar pipeline.
//!
//! Covers:
//!   - GrammarConverter construction and determinism
//!   - GrammarVisualizer output properties (DOT, SVG, text, dependency graph)
//!   - GrammarBuilder → Visualizer roundtrip
//!   - Large random grammars don't crash
//!   - Token/rule counts reflected in output

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tool::GrammarConverter;
use adze_tool::visualization::GrammarVisualizer;
use proptest::prelude::*;

// ===========================================================================
// Strategies
// ===========================================================================

fn grammar_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_filter("non-empty", |s| !s.is_empty())
}

fn _token_name_upper() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9]{0,6}".prop_filter("non-empty", |s| !s.is_empty())
}

fn _token_count() -> impl Strategy<Value = usize> {
    1..=8usize
}

fn _rule_depth() -> impl Strategy<Value = usize> {
    1..=6usize
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Build a grammar with `n` tokens and one rule referencing all of them.
fn make_grammar_n_tokens(name: &str, n: usize) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    let mut ids = Vec::new();
    for i in 0..n {
        let id = SymbolId((i + 1) as u16);
        g.tokens.insert(
            id,
            Token {
                name: format!("TOK{}", i),
                pattern: TokenPattern::Regex(format!("[a-z]{{{}}}", i + 1)),
                fragile: false,
            },
        );
        ids.push(id);
    }
    if !ids.is_empty() {
        let rule_id = SymbolId((n + 1) as u16);
        g.rules.entry(rule_id).or_default().push(Rule {
            lhs: rule_id,
            rhs: ids.iter().map(|id| Symbol::Terminal(*id)).collect(),
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
    }
    g
}

/// Build a chain grammar: rule_k → rule_{k-1} → … → rule_0 → terminal.
fn make_chain_grammar(name: &str, depth: usize) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "LEAF".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    for i in 0..depth {
        let id = SymbolId((i as u16) + 10);
        let rhs = if i == 0 {
            vec![Symbol::Terminal(tok)]
        } else {
            vec![Symbol::NonTerminal(SymbolId((i as u16) + 9))]
        };
        g.rules.entry(id).or_default().push(Rule {
            lhs: id,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }
    g
}

/// Build a grammar using GrammarBuilder with the given number of token/rule pairs.
fn builder_grammar(name: &str, token_count: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    // Add tokens named t0..t{n-1}
    for i in 0..token_count {
        let tname = format!("t{}", i);
        let pattern = format!("pat{}", i);
        b = b.token(&tname, &pattern);
    }
    // At least one rule referencing the first token
    if token_count > 0 {
        b = b.rule("start", vec!["t0"]).start("start");
    } else {
        // Empty rule
        b = b.rule("start", vec![]).start("start");
    }
    b.build()
}

/// Build a grammar with multiple alternative rules for one non-terminal.
fn make_multi_alt_grammar(name: &str, alt_count: usize) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    // Create `alt_count` tokens
    for i in 0..alt_count {
        let id = SymbolId((i + 1) as u16);
        g.tokens.insert(
            id,
            Token {
                name: format!("ALT{}", i),
                pattern: TokenPattern::String(format!("a{}", i)),
                fragile: false,
            },
        );
    }
    let rule_id = SymbolId(100);
    for i in 0..alt_count {
        g.rules.entry(rule_id).or_default().push(Rule {
            lhs: rule_id,
            rhs: vec![Symbol::Terminal(SymbolId((i + 1) as u16))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }
    g
}

// ===========================================================================
// Property-based tests — GrammarConverter
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// 1. GrammarConverter::create_sample_grammar always succeeds.
    #[test]
    fn converter_construction_always_succeeds(_seed in 0u32..1000) {
        let g = GrammarConverter::create_sample_grammar();
        prop_assert!(!g.name.is_empty());
    }

    /// 2. Sample grammar is deterministic across calls.
    #[test]
    fn converter_deterministic(_seed in 0u32..1000) {
        let a = GrammarConverter::create_sample_grammar();
        let b = GrammarConverter::create_sample_grammar();
        prop_assert_eq!(a, b);
    }

    /// 3. Sample grammar always has tokens.
    #[test]
    fn converter_sample_has_tokens(_seed in 0u32..500) {
        let g = GrammarConverter::create_sample_grammar();
        prop_assert!(!g.tokens.is_empty());
    }

    /// 4. Sample grammar always has rules.
    #[test]
    fn converter_sample_has_rules(_seed in 0u32..500) {
        let g = GrammarConverter::create_sample_grammar();
        prop_assert!(!g.rules.is_empty());
    }
}

// ===========================================================================
// Property-based tests — Visualization (text)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// 5. Text output always contains grammar name.
    #[test]
    fn text_contains_grammar_name(name in grammar_name()) {
        let g = Grammar::new(name.clone());
        let viz = GrammarVisualizer::new(g);
        let text = viz.to_text();
        prop_assert!(text.contains(&name), "Name '{}' not found in text", name);
    }

    /// 6. Text output is non-empty for any grammar.
    #[test]
    fn text_output_non_empty(name in grammar_name()) {
        let g = Grammar::new(name);
        let viz = GrammarVisualizer::new(g);
        prop_assert!(!viz.to_text().is_empty());
    }

    /// 7. Text output with N tokens mentions all N token names.
    #[test]
    fn text_mentions_all_tokens(n in 1..=6usize) {
        let g = make_grammar_n_tokens("tok_test", n);
        let viz = GrammarVisualizer::new(g);
        let text = viz.to_text();
        for i in 0..n {
            let tok_name = format!("TOK{}", i);
            prop_assert!(text.contains(&tok_name), "Missing token {}", tok_name);
        }
    }

    /// 8. Text output with M rule-groups has ≥ M "::=" occurrences.
    #[test]
    fn text_rule_count_matches(depth in 1..=5usize) {
        let g = make_chain_grammar("chain", depth);
        let viz = GrammarVisualizer::new(g);
        let text = viz.to_text();
        let rule_markers = text.matches("::=").count();
        prop_assert!(rule_markers >= depth, "Expected ≥{} rules, got {}", depth, rule_markers);
    }

    /// 9. Text output is deterministic.
    #[test]
    fn text_deterministic(n in 1..=4usize) {
        let g1 = make_grammar_n_tokens("det", n);
        let g2 = make_grammar_n_tokens("det", n);
        let t1 = GrammarVisualizer::new(g1).to_text();
        let t2 = GrammarVisualizer::new(g2).to_text();
        prop_assert_eq!(t1, t2);
    }
}

// ===========================================================================
// Property-based tests — Visualization (DOT)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// 10. DOT output always starts with `digraph`.
    #[test]
    fn dot_starts_with_digraph(name in grammar_name()) {
        let g = Grammar::new(name);
        let viz = GrammarVisualizer::new(g);
        prop_assert!(viz.to_dot().trim().starts_with("digraph"));
    }

    /// 11. DOT output always ends with `}`.
    #[test]
    fn dot_ends_with_brace(name in grammar_name()) {
        let g = Grammar::new(name);
        let viz = GrammarVisualizer::new(g);
        let dot = viz.to_dot();
        let ok = dot.trim().ends_with('}');
        prop_assert!(ok, "DOT should end with closing brace");
    }

    /// 12. DOT output is non-empty.
    #[test]
    fn dot_non_empty(name in grammar_name()) {
        let g = Grammar::new(name);
        let viz = GrammarVisualizer::new(g);
        prop_assert!(!viz.to_dot().is_empty());
    }

    /// 13. DOT with N tokens produces N terminal nodes.
    #[test]
    fn dot_terminal_count(n in 1..=6usize) {
        let g = make_grammar_n_tokens("dotcnt", n);
        let viz = GrammarVisualizer::new(g);
        let dot = viz.to_dot();
        let terminal_lines = dot
            .lines()
            .filter(|l| l.contains("shape=ellipse"))
            .count();
        prop_assert_eq!(terminal_lines, n);
    }

    /// 14. DOT edges equal RHS length for single-rule grammar.
    #[test]
    fn dot_edge_count(n in 1..=6usize) {
        let g = make_grammar_n_tokens("edges", n);
        let viz = GrammarVisualizer::new(g);
        let dot = viz.to_dot();
        let edges = dot.lines().filter(|l| l.contains(" -> ")).count();
        prop_assert_eq!(edges, n);
    }

    /// 15. DOT output is deterministic.
    #[test]
    fn dot_deterministic(n in 1..=4usize) {
        let g1 = make_grammar_n_tokens("detdot", n);
        let g2 = make_grammar_n_tokens("detdot", n);
        prop_assert_eq!(
            GrammarVisualizer::new(g1).to_dot(),
            GrammarVisualizer::new(g2).to_dot(),
        );
    }

    /// 16. Chain grammar DOT has edges for every level.
    #[test]
    fn dot_chain_edges(depth in 1..=5usize) {
        let g = make_chain_grammar("chain", depth);
        let viz = GrammarVisualizer::new(g);
        let dot = viz.to_dot();
        let edges = dot.lines().filter(|l| l.contains(" -> ")).count();
        prop_assert_eq!(edges, depth);
    }
}

// ===========================================================================
// Property-based tests — Visualization (SVG railroad)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// 17. SVG output is non-empty.
    #[test]
    fn svg_non_empty(name in grammar_name()) {
        let g = Grammar::new(name);
        let viz = GrammarVisualizer::new(g);
        prop_assert!(!viz.to_railroad_svg().is_empty());
    }

    /// 18. SVG output contains `<svg` tag.
    #[test]
    fn svg_has_svg_tag(n in 1..=4usize) {
        let g = make_grammar_n_tokens("svg", n);
        let viz = GrammarVisualizer::new(g);
        prop_assert!(viz.to_railroad_svg().contains("<svg"));
    }

    /// 19. SVG output contains `</svg>` closing tag.
    #[test]
    fn svg_closed(n in 1..=4usize) {
        let g = make_grammar_n_tokens("svg", n);
        let viz = GrammarVisualizer::new(g);
        prop_assert!(viz.to_railroad_svg().contains("</svg>"));
    }

    /// 20. SVG mentions token names for a grammar with tokens.
    #[test]
    fn svg_mentions_tokens(n in 1..=4usize) {
        let g = make_grammar_n_tokens("svgtok", n);
        let viz = GrammarVisualizer::new(g);
        let svg = viz.to_railroad_svg();
        for i in 0..n {
            let tok_name = format!("TOK{i}");
            let ok = svg.contains(&tok_name);
            prop_assert!(ok, "Missing token in SVG");
        }
    }

    /// 21. SVG is deterministic.
    #[test]
    fn svg_deterministic(n in 1..=3usize) {
        let g1 = make_grammar_n_tokens("svgdet", n);
        let g2 = make_grammar_n_tokens("svgdet", n);
        prop_assert_eq!(
            GrammarVisualizer::new(g1).to_railroad_svg(),
            GrammarVisualizer::new(g2).to_railroad_svg(),
        );
    }
}

// ===========================================================================
// Property-based tests — Dependency graph
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// 22. Dependency graph output is non-empty for any grammar.
    #[test]
    fn dep_graph_non_empty(name in grammar_name()) {
        let g = Grammar::new(name);
        let viz = GrammarVisualizer::new(g);
        prop_assert!(!viz.dependency_graph().is_empty());
    }

    /// 23. Dependency graph header is always present.
    #[test]
    fn dep_graph_header(name in grammar_name()) {
        let g = Grammar::new(name);
        let viz = GrammarVisualizer::new(g);
        prop_assert!(viz.dependency_graph().contains("Symbol Dependencies"));
    }

    /// 24. Chain grammar dependency graph mentions each level.
    #[test]
    fn dep_graph_chain_levels(depth in 2..=5usize) {
        let g = make_chain_grammar("dep", depth);
        let viz = GrammarVisualizer::new(g);
        let dep = viz.dependency_graph();
        let depends_count = dep.matches("depends on").count();
        prop_assert!(depends_count >= depth, "Expected ≥{} entries, got {}", depth, depends_count);
    }

    /// 25. Dependency graph is deterministic in content (not order).
    #[test]
    fn dep_graph_deterministic(depth in 1..=4usize) {
        let g1 = make_chain_grammar("depdet", depth);
        let g2 = make_chain_grammar("depdet", depth);
        let d1 = GrammarVisualizer::new(g1).dependency_graph();
        let d2 = GrammarVisualizer::new(g2).dependency_graph();
        // Lines may come in different HashMap order; compare sorted lines.
        let mut l1: Vec<&str> = d1.lines().collect();
        let mut l2: Vec<&str> = d2.lines().collect();
        l1.sort();
        l2.sort();
        prop_assert_eq!(l1, l2);
    }
}

// ===========================================================================
// Property-based tests — GrammarBuilder roundtrip
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// 26. GrammarBuilder grammars visualize without panic.
    #[test]
    fn builder_grammar_visualizes(n in 1..=5usize) {
        let g = builder_grammar("bld", n);
        let viz = GrammarVisualizer::new(g);
        let _ = viz.to_text();
        let _ = viz.to_dot();
        let _ = viz.to_railroad_svg();
        let _ = viz.dependency_graph();
    }

    /// 27. Builder grammar text output mentions "bst" (grammar name).
    #[test]
    fn builder_text_has_start(n in 1..=4usize) {
        let g = builder_grammar("bst", n);
        let text = GrammarVisualizer::new(g).to_text();
        // Grammar name always appears
        prop_assert!(text.contains("bst"));
        // The start rule is present as a rule line
        let has_rule = text.contains("::=");
        prop_assert!(has_rule, "Should have at least one rule");
    }

    /// 28. Builder grammar tokens appear in text.
    #[test]
    fn builder_tokens_in_text(n in 1..=4usize) {
        let g = builder_grammar("btk", n);
        let text = GrammarVisualizer::new(g).to_text();
        for i in 0..n {
            let tok_name = format!("t{i}");
            let ok = text.contains(&tok_name);
            prop_assert!(ok, "Missing token in text");
        }
    }
}

// ===========================================================================
// Property-based tests — Large random grammars
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// 29. Large token-count grammar doesn't crash visualizer.
    #[test]
    fn large_token_grammar_no_crash(n in 10..=50usize) {
        let g = make_grammar_n_tokens("large", n);
        let viz = GrammarVisualizer::new(g);
        let _ = viz.to_text();
        let _ = viz.to_dot();
    }

    /// 30. Large chain grammar doesn't crash.
    #[test]
    fn large_chain_no_crash(depth in 10..=50usize) {
        let g = make_chain_grammar("bigchain", depth);
        let viz = GrammarVisualizer::new(g);
        let _ = viz.to_text();
        let _ = viz.to_dot();
        let _ = viz.dependency_graph();
    }

    /// 31. Large multi-alt grammar doesn't crash.
    #[test]
    fn large_multi_alt_no_crash(alt in 10..=40usize) {
        let g = make_multi_alt_grammar("multialt", alt);
        let viz = GrammarVisualizer::new(g);
        let _ = viz.to_text();
        let _ = viz.to_dot();
        let _ = viz.to_railroad_svg();
    }
}

// ===========================================================================
// Property-based tests — Multi-alternative rules
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// 32. Multi-alt grammar text has one "::=" per alternative.
    #[test]
    fn multi_alt_text_rule_lines(alt in 1..=6usize) {
        let g = make_multi_alt_grammar("malt", alt);
        let text = GrammarVisualizer::new(g).to_text();
        let count = text.matches("::=").count();
        prop_assert_eq!(count, alt);
    }

    /// 33. Multi-alt grammar DOT has one edge per alternative.
    #[test]
    fn multi_alt_dot_edges(alt in 1..=6usize) {
        let g = make_multi_alt_grammar("medge", alt);
        let dot = GrammarVisualizer::new(g).to_dot();
        let edges = dot.lines().filter(|l| l.contains(" -> ")).count();
        prop_assert_eq!(edges, alt);
    }

    /// 34. Multi-alt text mentions every token alternative.
    #[test]
    fn multi_alt_text_all_tokens(alt in 1..=6usize) {
        let g = make_multi_alt_grammar("matk", alt);
        let text = GrammarVisualizer::new(g).to_text();
        for i in 0..alt {
            let tok_name = format!("ALT{i}");
            let ok = text.contains(&tok_name);
            prop_assert!(ok, "Missing alt token in text");
        }
    }
}

// ===========================================================================
// Unit tests — GrammarConverter specifics
// ===========================================================================

#[test]
fn converter_sample_grammar_has_correct_name() {
    let g = GrammarConverter::create_sample_grammar();
    assert_eq!(g.name, "sample");
}

#[test]
fn converter_sample_grammar_has_three_tokens() {
    let g = GrammarConverter::create_sample_grammar();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn converter_sample_grammar_has_identifier_token() {
    let g = GrammarConverter::create_sample_grammar();
    assert!(g.tokens.values().any(|t| t.name == "identifier"));
}

#[test]
fn converter_sample_grammar_has_number_token() {
    let g = GrammarConverter::create_sample_grammar();
    assert!(g.tokens.values().any(|t| t.name == "number"));
}

#[test]
fn converter_sample_grammar_has_plus_token() {
    let g = GrammarConverter::create_sample_grammar();
    assert!(g.tokens.values().any(|t| t.name == "plus"));
}

#[test]
fn converter_sample_grammar_has_rules() {
    let g = GrammarConverter::create_sample_grammar();
    assert!(!g.rules.is_empty());
    let total_rules: usize = g.rules.values().map(|v| v.len()).sum();
    assert_eq!(total_rules, 3);
}

#[test]
fn converter_sample_grammar_has_fields() {
    let g = GrammarConverter::create_sample_grammar();
    assert_eq!(g.fields.len(), 2);
    assert!(g.fields.values().any(|f| f == "left"));
    assert!(g.fields.values().any(|f| f == "right"));
}

#[test]
fn converter_sample_grammar_has_precedence_rule() {
    let g = GrammarConverter::create_sample_grammar();
    let all_rules: Vec<&Rule> = g.all_rules().collect();
    let prec_rules: Vec<&&Rule> = all_rules
        .iter()
        .filter(|r| r.precedence.is_some())
        .collect();
    assert_eq!(prec_rules.len(), 1);
    assert_eq!(prec_rules[0].precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(prec_rules[0].associativity, Some(Associativity::Left));
}

// ===========================================================================
// Unit tests — Visualization specifics
// ===========================================================================

#[test]
fn empty_grammar_text_has_header() {
    let g = Grammar::new("empty".to_string());
    let text = GrammarVisualizer::new(g).to_text();
    assert!(text.contains("Grammar: empty"));
    assert!(text.contains("Tokens:"));
    assert!(text.contains("Rules:"));
}

#[test]
fn empty_grammar_dot_is_valid() {
    let g = Grammar::new("empty".to_string());
    let dot = GrammarVisualizer::new(g).to_dot();
    assert!(dot.starts_with("digraph"));
    assert!(dot.trim().ends_with('}'));
}

#[test]
fn empty_grammar_svg_is_valid() {
    let g = Grammar::new("empty".to_string());
    let svg = GrammarVisualizer::new(g).to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn sample_grammar_text_mentions_all_tokens() {
    let g = GrammarConverter::create_sample_grammar();
    let text = GrammarVisualizer::new(g).to_text();
    assert!(text.contains("identifier"));
    assert!(text.contains("number"));
    assert!(text.contains("plus"));
}

#[test]
fn sample_grammar_dot_has_edges() {
    let g = GrammarConverter::create_sample_grammar();
    let dot = GrammarVisualizer::new(g).to_dot();
    let edges = dot.lines().filter(|l| l.contains(" -> ")).count();
    // 3 rules: id(1 edge) + num(1 edge) + expr+expr(3 edges) = 5
    assert_eq!(edges, 5);
}

#[test]
fn sample_grammar_dependency_graph() {
    let g = GrammarConverter::create_sample_grammar();
    let dep = GrammarVisualizer::new(g).dependency_graph();
    assert!(dep.contains("depends on"));
}

#[test]
fn builder_python_like_visualizes() {
    let g = GrammarBuilder::python_like();
    let viz = GrammarVisualizer::new(g);
    let text = viz.to_text();
    assert!(text.contains("python_like"));
    assert!(!text.is_empty());
}

#[test]
fn builder_javascript_like_visualizes() {
    let g = GrammarBuilder::javascript_like();
    let viz = GrammarVisualizer::new(g);
    let text = viz.to_text();
    assert!(text.contains("javascript_like"));
    assert!(!text.is_empty());
}

#[test]
fn builder_python_like_dot_valid() {
    let g = GrammarBuilder::python_like();
    let dot = GrammarVisualizer::new(g).to_dot();
    assert!(dot.starts_with("digraph"));
    assert!(dot.contains("rankdir=LR"));
}

#[test]
fn builder_javascript_like_dot_has_edges() {
    let g = GrammarBuilder::javascript_like();
    let dot = GrammarVisualizer::new(g).to_dot();
    assert!(dot.lines().any(|l| l.contains(" -> ")));
}

#[test]
fn builder_simple_roundtrip_text() {
    let g = GrammarBuilder::new("simple")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUM", "+", "NUM"])
        .start("expr")
        .build();
    let text = GrammarVisualizer::new(g).to_text();
    assert!(text.contains("simple"));
    assert!(text.contains("NUM"));
}

#[test]
fn builder_with_precedence_text_shows_metadata() {
    let g = GrammarBuilder::new("prec")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("e", vec!["e", "+", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "*", "e"], 2, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let text = GrammarVisualizer::new(g).to_text();
    assert!(text.contains("precedence"));
    assert!(text.contains("associativity"));
}

#[test]
fn single_epsilon_rule_text() {
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
    let text = GrammarVisualizer::new(g).to_text();
    assert!(text.contains("ε"));
}

#[test]
fn single_epsilon_rule_dot() {
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
    let dot = GrammarVisualizer::new(g).to_dot();
    // Epsilon transitions are skipped in DOT
    let edges = dot.lines().filter(|l| l.contains(" -> ")).count();
    assert_eq!(edges, 0);
}

#[test]
fn grammar_with_externals_dot_shows_diamond() {
    let g = GrammarBuilder::new("ext")
        .token("A", "a")
        .external("INDENT")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let dot = GrammarVisualizer::new(g).to_dot();
    assert!(dot.contains("shape=diamond"));
}

#[test]
fn grammar_with_externals_text_shows_external_section() {
    let g = GrammarBuilder::new("ext2")
        .token("A", "a")
        .external("INDENT")
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let text = GrammarVisualizer::new(g).to_text();
    assert!(text.contains("External Tokens:"));
    assert!(text.contains("INDENT"));
}
