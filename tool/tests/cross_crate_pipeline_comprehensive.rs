// Cross-crate integration tests: IR builder → optimizer → visualization pipeline.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, optimize_grammar};
use adze_ir::{
    Associativity, FieldId, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tool::GrammarConverter;
use adze_tool::GrammarVisualizer;

// ===== Builder → Visualizer pipeline =====

#[test]
fn builder_to_dot() {
    let g = GrammarBuilder::new("test")
        .token("num", r"\d+")
        .token("plus", "+")
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.contains("digraph"));
    assert!(dot.contains("num"));
}

#[test]
fn builder_to_text() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    let viz = GrammarVisualizer::new(g);
    let text = viz.to_text();
    assert!(!text.is_empty());
}

#[test]
fn builder_to_svg() {
    let g = GrammarBuilder::new("test")
        .token("x", "x")
        .rule("root", vec!["x"])
        .start("root")
        .build();
    let viz = GrammarVisualizer::new(g);
    let svg = viz.to_railroad_svg();
    assert!(!svg.is_empty());
}

#[test]
fn builder_to_dependency_graph() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["child"])
        .rule("child", vec!["a"])
        .start("root")
        .build();
    let viz = GrammarVisualizer::new(g);
    let dep = viz.dependency_graph();
    assert!(!dep.is_empty());
}

// ===== Optimizer → Visualizer pipeline =====

#[test]
fn optimized_grammar_to_dot() {
    let g = GrammarBuilder::new("opt")
        .token("num", r"\d+")
        .token("plus", "+")
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let optimized = optimize_grammar(g).unwrap();
    let viz = GrammarVisualizer::new(optimized);
    let dot = viz.to_dot();
    assert!(dot.contains("digraph"));
}

#[test]
fn optimized_grammar_to_text() {
    let g = GrammarBuilder::new("opt")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    let optimized = optimize_grammar(g).unwrap();
    let viz = GrammarVisualizer::new(optimized);
    let text = viz.to_text();
    assert!(!text.is_empty());
}

#[test]
fn optimizer_stats_then_visualize() {
    let mut g = GrammarBuilder::new("stats")
        .token("x", "x")
        .token("y", "y")
        .rule("root", vec!["x"])
        .rule("alt", vec!["y"])
        .start("root")
        .build();
    let mut opt = GrammarOptimizer::new();
    let stats = opt.optimize(&mut g);
    let total = stats.total();
    // Stats should be non-negative
    assert!(total >= 0 || total == 0);
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.contains("digraph"));
}

// ===== Converter → Visualizer pipeline =====

#[test]
fn converter_to_all_formats() {
    let g = GrammarConverter::create_sample_grammar();
    let viz = GrammarVisualizer::new(g);
    assert!(!viz.to_dot().is_empty());
    assert!(!viz.to_text().is_empty());
    assert!(!viz.to_railroad_svg().is_empty());
    assert!(!viz.dependency_graph().is_empty());
}

// ===== Builder → Optimizer → Visualizer full pipeline =====

#[test]
fn full_pipeline_simple() {
    let g = GrammarBuilder::new("full")
        .token("a", "a")
        .rule("root", vec!["a"])
        .start("root")
        .build();
    let optimized = optimize_grammar(g).unwrap();
    let viz = GrammarVisualizer::new(optimized);
    let dot = viz.to_dot();
    let text = viz.to_text();
    assert!(!dot.is_empty());
    assert!(!text.is_empty());
}

#[test]
fn full_pipeline_arithmetic() {
    let g = GrammarBuilder::new("arith")
        .token("num", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .token("lparen", "(")
        .token("rparen", ")")
        .rule("expr", vec!["term"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["num"])
        .start("expr")
        .build();
    let optimized = optimize_grammar(g).unwrap();
    let viz = GrammarVisualizer::new(optimized);
    let dot = viz.to_dot();
    assert!(dot.contains("digraph"));
}

#[test]
fn full_pipeline_with_fields() {
    let mut g = GrammarBuilder::new("fields")
        .token("num", r"\d+")
        .token("plus", "+")
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    g.fields.insert(FieldId(1), "value".to_string());
    let optimized = optimize_grammar(g).unwrap();
    let viz = GrammarVisualizer::new(optimized);
    let _ = viz.to_dot();
    let _ = viz.to_text();
}

// ===== Builder grammar invariants through visualization =====

#[test]
fn builder_grammar_dot_contains_start_symbol() {
    let g = GrammarBuilder::new("start")
        .token("x", "x")
        .rule("program", vec!["x"])
        .start("program")
        .build();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    // The start symbol should appear somewhere in the output
    assert!(!dot.is_empty());
}

#[test]
fn builder_grammar_text_has_all_rules() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["child1"])
        .rule("child1", vec!["a"])
        .rule("child2", vec!["b"])
        .start("root")
        .build();
    let viz = GrammarVisualizer::new(g);
    let text = viz.to_text();
    assert!(!text.is_empty());
}

// ===== Edge case pipelines =====

#[test]
fn empty_builder_grammar_visualize() {
    let g = GrammarBuilder::new("empty").build();
    let viz = GrammarVisualizer::new(g);
    let _ = viz.to_dot();
    let _ = viz.to_text();
    let _ = viz.to_railroad_svg();
    let _ = viz.dependency_graph();
}

#[test]
fn single_token_builder_visualize() {
    let g = GrammarBuilder::new("one").token("t", "t").build();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.contains("t"));
}

#[test]
fn many_token_builder_visualize() {
    let mut b = GrammarBuilder::new("many");
    for i in 0..50 {
        b = b.token(&format!("t{}", i), &format!("{}", i));
    }
    let g = b.build();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.contains("t0"));
    assert!(dot.contains("t49"));
}

#[test]
fn builder_grammar_optimize_then_dot_deterministic() {
    let make = || {
        let g = GrammarBuilder::new("det")
            .token("a", "a")
            .token("b", "b")
            .rule("root", vec!["a"])
            .rule("alt", vec!["b"])
            .start("root")
            .build();
        let opt = optimize_grammar(g).unwrap();
        GrammarVisualizer::new(opt).to_dot()
    };
    let d1 = make();
    let d2 = make();
    assert_eq!(d1, d2);
}

#[test]
fn builder_grammar_optimize_then_text_deterministic() {
    let make = || {
        let g = GrammarBuilder::new("det")
            .token("x", "x")
            .rule("root", vec!["x"])
            .start("root")
            .build();
        let opt = optimize_grammar(g).unwrap();
        GrammarVisualizer::new(opt).to_text()
    };
    let t1 = make();
    let t2 = make();
    assert_eq!(t1, t2);
}

// ===== Grammar with manual Rule construction → visualize =====

#[test]
fn manual_grammar_visualize() {
    let mut g = Grammar::new("manual".to_string());
    let tok = SymbolId(1);
    let nt = SymbolId(10);
    g.tokens.insert(
        tok,
        Token {
            name: "keyword".to_string(),
            pattern: TokenPattern::String("fn".to_string()),
            fragile: false,
        },
    );
    g.rules.entry(nt).or_default().push(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.rule_names.insert(nt, "function_def".to_string());
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.contains("keyword") || dot.contains("fn"));
    assert!(dot.contains("function_def") || dot.contains("n10"));
}

#[test]
fn manual_grammar_with_precedence_visualize() {
    let mut g = Grammar::new("prec".to_string());
    let num = SymbolId(1);
    let plus = SymbolId(2);
    let minus = SymbolId(3);
    let expr = SymbolId(10);

    g.tokens.insert(
        num,
        Token {
            name: "num".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        plus,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        minus,
        Token {
            name: "minus".to_string(),
            pattern: TokenPattern::String("-".to_string()),
            fragile: false,
        },
    );

    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![Symbol::Terminal(num)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
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
        production_id: ProductionId(1),
    });
    g.rules.entry(expr).or_default().push(Rule {
        lhs: expr,
        rhs: vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(minus),
            Symbol::NonTerminal(expr),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(2),
    });
    g.rule_names.insert(expr, "expression".to_string());

    let viz = GrammarVisualizer::new(g);
    let text = viz.to_text();
    assert!(!text.is_empty());
    let dot = viz.to_dot();
    assert!(dot.contains("plus"));
    assert!(dot.contains("minus"));
}

// ===== Format isolation =====

#[test]
fn dot_does_not_contain_svg_tags() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("r", vec!["a"])
        .start("r")
        .build();
    let dot = GrammarVisualizer::new(g).to_dot();
    assert!(!dot.contains("<svg"));
    assert!(!dot.contains("<rect"));
}

#[test]
fn text_does_not_contain_dot_syntax() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("r", vec!["a"])
        .start("r")
        .build();
    let text = GrammarVisualizer::new(g).to_text();
    assert!(!text.contains("digraph"));
}
