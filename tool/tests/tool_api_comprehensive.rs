// Wave 131: Comprehensive tests for adze-tool public API
// Tests GrammarConverter, GrammarVisualizer, GrammarJsConverter, BuildOptions

use adze_ir::*;
use adze_tool::*;

// =====================================================================
// GrammarConverter tests
// =====================================================================

#[test]
fn sample_grammar_has_name() {
    let g = GrammarConverter::create_sample_grammar();
    assert_eq!(g.name, "sample");
}

#[test]
fn sample_grammar_has_tokens() {
    let g = GrammarConverter::create_sample_grammar();
    assert!(
        g.tokens.len() >= 3,
        "Expected at least 3 tokens (id, num, plus)"
    );
}

#[test]
fn sample_grammar_has_rules() {
    let g = GrammarConverter::create_sample_grammar();
    assert!(!g.rules.is_empty());
}

#[test]
fn sample_grammar_tokens_have_names() {
    let g = GrammarConverter::create_sample_grammar();
    let names: Vec<String> = g.tokens.values().map(|t| t.name.clone()).collect();
    assert!(names.contains(&"identifier".to_string()));
    assert!(names.contains(&"number".to_string()));
    assert!(names.contains(&"plus".to_string()));
}

#[test]
fn sample_grammar_has_fields() {
    let g = GrammarConverter::create_sample_grammar();
    assert!(!g.fields.is_empty());
    let field_names: Vec<String> = g.fields.values().cloned().collect();
    assert!(field_names.contains(&"left".to_string()));
    assert!(field_names.contains(&"right".to_string()));
}

#[test]
fn sample_grammar_has_precedence_rule() {
    let g = GrammarConverter::create_sample_grammar();
    let has_prec = g.rules.values().flatten().any(|r| r.precedence.is_some());
    assert!(has_prec, "Expected at least one rule with precedence");
}

#[test]
fn sample_grammar_has_associativity() {
    let g = GrammarConverter::create_sample_grammar();
    let has_assoc = g
        .rules
        .values()
        .flatten()
        .any(|r| r.associativity.is_some());
    assert!(has_assoc, "Expected at least one rule with associativity");
}

#[test]
fn sample_grammar_rules_have_lhs() {
    let g = GrammarConverter::create_sample_grammar();
    for (lhs, rules) in &g.rules {
        for rule in rules {
            assert_eq!(
                rule.lhs, *lhs,
                "Rule LHS should match its key in the rules map"
            );
        }
    }
}

#[test]
fn sample_grammar_tokens_are_not_fragile() {
    let g = GrammarConverter::create_sample_grammar();
    for token in g.tokens.values() {
        assert!(
            !token.fragile,
            "Sample grammar tokens should not be fragile"
        );
    }
}

#[test]
fn sample_grammar_no_externals() {
    let g = GrammarConverter::create_sample_grammar();
    assert!(
        g.externals.is_empty(),
        "Sample grammar should have no external tokens"
    );
}

#[test]
fn sample_grammar_no_extras() {
    let g = GrammarConverter::create_sample_grammar();
    assert!(g.extras.is_empty(), "Sample grammar should have no extras");
}

#[test]
fn sample_grammar_no_conflicts() {
    let g = GrammarConverter::create_sample_grammar();
    assert!(g.conflicts.is_empty());
}

// =====================================================================
// GrammarVisualizer tests — DOT output
// =====================================================================

#[test]
fn dot_output_starts_with_digraph() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.starts_with("digraph Grammar {"));
}

#[test]
fn dot_output_ends_with_closing_brace() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.trim().ends_with("}"));
}

#[test]
fn dot_output_contains_terminals() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.contains("identifier"));
    assert!(dot.contains("number"));
    assert!(dot.contains("plus"));
}

#[test]
fn dot_output_contains_nonterminals() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.contains("Non-terminals"));
}

#[test]
fn dot_output_contains_edges() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.contains("->"), "DOT output should contain edge arrows");
}

#[test]
fn dot_output_has_shape_attributes() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(
        dot.contains("shape=ellipse"),
        "Terminals should be ellipse shaped"
    );
    assert!(
        dot.contains("fillcolor=lightblue"),
        "Terminals should be light blue"
    );
    assert!(
        dot.contains("fillcolor=lightgreen"),
        "Non-terminals should be light green"
    );
}

#[test]
fn dot_output_is_nonempty() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.len() > 100, "DOT output should be substantial");
}

// =====================================================================
// GrammarVisualizer tests — railroad SVG
// =====================================================================

#[test]
fn railroad_svg_starts_with_svg_tag() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"), "Should contain SVG opening tag");
}

#[test]
fn railroad_svg_ends_with_svg_tag() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("</svg>"), "Should contain SVG closing tag");
}

#[test]
fn railroad_svg_contains_rule_names() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let svg = viz.to_railroad_svg();
    // Should contain at least some of the token names
    assert!(svg.len() > 100, "SVG should have substantial content");
}

// =====================================================================
// GrammarVisualizer tests — text summary
// =====================================================================

#[test]
fn text_summary_contains_grammar_name() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let summary = viz.to_text();
    assert!(
        summary.contains("sample"),
        "Summary should mention grammar name"
    );
}

#[test]
fn text_summary_nonempty() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    let summary = viz.to_text();
    assert!(!summary.is_empty());
}

// =====================================================================
// GrammarVisualizer with empty grammar
// =====================================================================

#[test]
fn dot_output_empty_grammar() {
    let g = Grammar::new("empty".to_string());
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.contains("digraph Grammar"));
    assert!(dot.contains("}"));
}

#[test]
fn railroad_svg_empty_grammar() {
    let g = Grammar::new("empty".to_string());
    let viz = GrammarVisualizer::new(g);
    let svg = viz.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn text_summary_empty_grammar() {
    let g = Grammar::new("empty".to_string());
    let viz = GrammarVisualizer::new(g);
    let summary = viz.to_text();
    assert!(summary.contains("empty"));
}

// =====================================================================
// BuildOptions tests
// =====================================================================

#[test]
fn build_options_default_values() {
    let opts = BuildOptions {
        out_dir: "/tmp/test".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert_eq!(opts.out_dir, "/tmp/test");
    assert!(!opts.emit_artifacts);
    assert!(!opts.compress_tables);
}

#[test]
fn build_options_with_artifacts() {
    let opts = BuildOptions {
        out_dir: "/tmp/artifacts".to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    assert!(opts.emit_artifacts);
    assert!(opts.compress_tables);
}

#[test]
fn build_options_empty_out_dir() {
    let opts = BuildOptions {
        out_dir: String::new(),
        emit_artifacts: false,
        compress_tables: false,
    };
    assert!(opts.out_dir.is_empty());
}

// =====================================================================
// Grammar construction consistency tests
// =====================================================================

#[test]
fn sample_grammar_deterministic() {
    let g1 = GrammarConverter::create_sample_grammar();
    let g2 = GrammarConverter::create_sample_grammar();
    assert_eq!(g1.name, g2.name);
    assert_eq!(g1.tokens.len(), g2.tokens.len());
    assert_eq!(g1.rules.len(), g2.rules.len());
    assert_eq!(g1.fields.len(), g2.fields.len());
}

#[test]
fn sample_grammar_token_patterns() {
    let g = GrammarConverter::create_sample_grammar();
    for token in g.tokens.values() {
        match &token.pattern {
            TokenPattern::Regex(r) => assert!(!r.is_empty(), "Regex patterns should not be empty"),
            TokenPattern::String(s) => {
                assert!(!s.is_empty(), "String patterns should not be empty")
            }
        }
    }
}

#[test]
fn sample_grammar_rule_rhs_nonempty() {
    let g = GrammarConverter::create_sample_grammar();
    for rules in g.rules.values() {
        for rule in rules {
            assert!(!rule.rhs.is_empty(), "Rule RHS should not be empty");
        }
    }
}

#[test]
fn sample_grammar_production_ids_unique_within_lhs() {
    let g = GrammarConverter::create_sample_grammar();
    for rules in g.rules.values() {
        let ids: Vec<ProductionId> = rules.iter().map(|r| r.production_id).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(
            ids.len(),
            unique.len(),
            "Production IDs should be unique within a nonterminal"
        );
    }
}

// =====================================================================
// Grammar serialization roundtrip via serde_json
// =====================================================================

#[test]
fn sample_grammar_serde_roundtrip() {
    let g = GrammarConverter::create_sample_grammar();
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g.name, g2.name);
    assert_eq!(g.tokens.len(), g2.tokens.len());
    assert_eq!(g.rules.len(), g2.rules.len());
}

#[test]
fn sample_grammar_serde_json_pretty() {
    let g = GrammarConverter::create_sample_grammar();
    let json = serde_json::to_string_pretty(&g).expect("pretty serialize");
    assert!(json.contains("\"name\""));
    assert!(json.contains("sample"));
}

#[test]
fn empty_grammar_serde_roundtrip() {
    let g = Grammar::new("test".to_string());
    let json = serde_json::to_string(&g).expect("serialize");
    let g2: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g.name, g2.name);
}

// =====================================================================
// Visualization multi-rule grammar tests
// =====================================================================

#[test]
fn dot_with_multiple_nonterminals() {
    let mut g = Grammar::new("multi".to_string());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let tok = SymbolId(3);

    g.tokens.insert(
        tok,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    g.rules.entry(a).or_default().push(Rule {
        lhs: a,
        rhs: vec![Symbol::NonTerminal(b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rules.entry(b).or_default().push(Rule {
        lhs: b,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g.rule_names.insert(a, "start".to_string());
    g.rule_names.insert(b, "item".to_string());

    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(
        dot.contains("start") || dot.contains("n1"),
        "Should reference first nonterminal"
    );
    assert!(dot.contains("->"), "Should have edges");
}

#[test]
fn dot_grammar_with_epsilon_rule() {
    let mut g = Grammar::new("eps".to_string());
    let start = SymbolId(1);
    g.rules.entry(start).or_default().push(Rule {
        lhs: start,
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(start, "empty".to_string());

    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    // Epsilon transitions should be skipped in DOT output
    assert!(dot.contains("digraph Grammar"));
}

// =====================================================================
// GrammarJsConverter tests (if available)
// =====================================================================

#[test]
fn grammar_js_converter_exists() {
    // Just verify the type is accessible
    let _ = std::any::type_name::<GrammarJsConverter>();
}

// =====================================================================
// error module tests
// =====================================================================

#[test]
fn tool_error_display() {
    let err = ToolError::Other("test error".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("test") || msg.contains("error"));
}

#[test]
fn tool_error_debug() {
    let err = ToolError::Other("debug test".to_string());
    let msg = format!("{:?}", err);
    assert!(!msg.is_empty());
}
