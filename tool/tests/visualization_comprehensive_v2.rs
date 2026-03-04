//! Comprehensive visualization tests v2 — coverage expansion for GrammarVisualizer.

use adze_ir::builder::GrammarBuilder;
use adze_tool::visualization::GrammarVisualizer;

fn simple_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

fn arithmetic_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("arith")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .token("star", "\\*")
        .token("lparen", "\\(")
        .token("rparen", "\\)")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "plus", "term"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "star", "factor"])
        .rule("factor", vec!["num"])
        .rule("factor", vec!["lparen", "expr", "rparen"])
        .start("expr")
        .build()
}

fn single_token_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("single")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build()
}

fn chain_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("a")
        .build()
}

fn diamond_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("diamond")
        .token("x", "x")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("a", vec!["c"])
        .rule("b", vec!["c"])
        .rule("c", vec!["x"])
        .start("s")
        .build()
}

fn recursive_grammar() -> adze_ir::Grammar {
    GrammarBuilder::new("recursive")
        .token("a", "a")
        .rule("list", vec!["a"])
        .rule("list", vec!["list", "a"])
        .start("list")
        .build()
}

// === Determinism tests ===

#[test]
fn dot_deterministic() {
    let a = GrammarVisualizer::new(simple_grammar()).to_dot();
    let b = GrammarVisualizer::new(simple_grammar()).to_dot();
    assert_eq!(a, b);
}

#[test]
fn text_deterministic() {
    let a = GrammarVisualizer::new(simple_grammar()).to_text();
    let b = GrammarVisualizer::new(simple_grammar()).to_text();
    assert_eq!(a, b);
}

#[test]
fn svg_deterministic() {
    let a = GrammarVisualizer::new(simple_grammar()).to_railroad_svg();
    let b = GrammarVisualizer::new(simple_grammar()).to_railroad_svg();
    assert_eq!(a, b);
}

#[test]
fn deps_deterministic() {
    let a = GrammarVisualizer::new(simple_grammar()).dependency_graph();
    let b = GrammarVisualizer::new(simple_grammar()).dependency_graph();
    assert_eq!(a, b);
}

#[test]
fn arithmetic_dot_deterministic() {
    let a = GrammarVisualizer::new(arithmetic_grammar()).to_dot();
    let b = GrammarVisualizer::new(arithmetic_grammar()).to_dot();
    assert_eq!(a, b);
}

// === Structural DOT validation ===

#[test]
fn dot_has_digraph_header() {
    let dot = GrammarVisualizer::new(simple_grammar()).to_dot();
    assert!(dot.starts_with("digraph Grammar {"));
}

#[test]
fn dot_has_closing_brace() {
    let dot = GrammarVisualizer::new(simple_grammar()).to_dot();
    assert!(dot.trim_end().ends_with('}'));
}

#[test]
fn dot_has_rankdir() {
    let dot = GrammarVisualizer::new(simple_grammar()).to_dot();
    assert!(dot.contains("rankdir=LR"));
}

#[test]
fn dot_terminal_styling() {
    let dot = GrammarVisualizer::new(simple_grammar()).to_dot();
    assert!(dot.contains("ellipse"));
    assert!(dot.contains("lightblue"));
}

#[test]
fn dot_has_edges() {
    let dot = GrammarVisualizer::new(simple_grammar()).to_dot();
    assert!(dot.contains("->"), "DOT should have edges");
}

#[test]
fn dot_edge_count_arithmetic() {
    let dot = GrammarVisualizer::new(arithmetic_grammar()).to_dot();
    let edges = dot.matches("->").count();
    assert!(edges >= 6, "arithmetic should have 6+ edges, got {edges}");
}

#[test]
fn dot_edge_count_chain() {
    let dot = GrammarVisualizer::new(chain_grammar()).to_dot();
    let edges = dot.matches("->").count();
    assert!(edges >= 3, "chain should have 3+ edges, got {edges}");
}

#[test]
fn dot_edge_count_diamond() {
    let dot = GrammarVisualizer::new(diamond_grammar()).to_dot();
    let edges = dot.matches("->").count();
    assert!(edges >= 5, "diamond should have 5+ edges, got {edges}");
}

// === Text format tests ===

#[test]
fn text_not_empty_simple() {
    let text = GrammarVisualizer::new(simple_grammar()).to_text();
    assert!(!text.is_empty());
}

#[test]
fn text_not_empty_arithmetic() {
    let text = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    assert!(text.len() > 30);
}

#[test]
fn text_not_empty_recursive() {
    let text = GrammarVisualizer::new(recursive_grammar()).to_text();
    assert!(!text.is_empty());
}

#[test]
fn text_not_empty_chain() {
    let text = GrammarVisualizer::new(chain_grammar()).to_text();
    assert!(!text.is_empty());
}

#[test]
fn text_not_empty_diamond() {
    let text = GrammarVisualizer::new(diamond_grammar()).to_text();
    assert!(!text.is_empty());
}

// === Railroad SVG tests ===

#[test]
fn svg_not_empty_simple() {
    let svg = GrammarVisualizer::new(simple_grammar()).to_railroad_svg();
    assert!(!svg.is_empty());
}

#[test]
fn svg_not_empty_arithmetic() {
    let svg = GrammarVisualizer::new(arithmetic_grammar()).to_railroad_svg();
    assert!(!svg.is_empty());
}

#[test]
fn svg_not_empty_recursive() {
    let svg = GrammarVisualizer::new(recursive_grammar()).to_railroad_svg();
    assert!(!svg.is_empty());
}

#[test]
fn svg_not_empty_chain() {
    let svg = GrammarVisualizer::new(chain_grammar()).to_railroad_svg();
    assert!(!svg.is_empty());
}

// === Dependency graph tests ===

#[test]
fn deps_not_empty_simple() {
    let deps = GrammarVisualizer::new(simple_grammar()).dependency_graph();
    assert!(!deps.is_empty());
}

#[test]
fn deps_not_empty_arithmetic() {
    let deps = GrammarVisualizer::new(arithmetic_grammar()).dependency_graph();
    assert!(!deps.is_empty());
}

#[test]
fn deps_not_empty_chain() {
    let deps = GrammarVisualizer::new(chain_grammar()).dependency_graph();
    assert!(!deps.is_empty());
}

#[test]
fn deps_not_empty_diamond() {
    let deps = GrammarVisualizer::new(diamond_grammar()).dependency_graph();
    assert!(!deps.is_empty());
}

#[test]
fn deps_not_empty_recursive() {
    let deps = GrammarVisualizer::new(recursive_grammar()).dependency_graph();
    assert!(!deps.is_empty());
}

// === Size reasonableness ===

#[test]
fn dot_size_simple_reasonable() {
    let dot = GrammarVisualizer::new(simple_grammar()).to_dot();
    assert!(dot.len() < 10_000);
}

#[test]
fn dot_size_arithmetic_reasonable() {
    let dot = GrammarVisualizer::new(arithmetic_grammar()).to_dot();
    assert!(dot.len() < 50_000);
}

#[test]
fn text_size_reasonable() {
    let text = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    assert!(text.len() < 50_000);
}

// === Many-token grammar ===

#[test]
fn many_tokens_dot() {
    let mut b = GrammarBuilder::new("many");
    let names: Vec<String> = (0..20).map(|i| format!("t{i}")).collect();
    for n in &names {
        b = b.token(n, n);
    }
    let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    b = b.rule("start", refs).start("start");
    let viz = GrammarVisualizer::new(b.build());
    let dot = viz.to_dot();
    assert!(dot.len() > 200);
}

#[test]
fn many_rules_text() {
    let mut b = GrammarBuilder::new("many_rules")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c");
    for i in 0..10 {
        let name = format!("r{i}");
        b = b
            .rule(&name, vec!["a"])
            .rule(&name, vec!["b"])
            .rule(&name, vec!["c"]);
    }
    b = b.start("r0");
    let viz = GrammarVisualizer::new(b.build());
    assert!(!viz.to_text().is_empty());
    assert!(!viz.to_dot().is_empty());
}

// === Special characters ===

#[test]
fn dot_special_chars_no_panic() {
    let g = GrammarBuilder::new("special")
        .token("plus", "\\+")
        .token("quote", "\"")
        .rule("start", vec!["plus", "quote"])
        .start("start")
        .build();
    let viz = GrammarVisualizer::new(g);
    let dot = viz.to_dot();
    assert!(dot.contains("digraph"));
}

#[test]
fn dot_unicode_tokens() {
    let g = GrammarBuilder::new("unicode")
        .token("alpha", "α")
        .token("beta", "β")
        .rule("start", vec!["alpha", "beta"])
        .start("start")
        .build();
    let viz = GrammarVisualizer::new(g);
    assert!(!viz.to_dot().is_empty());
}

// === All-format coverage per grammar ===

#[test]
fn simple_all_formats() {
    let viz = GrammarVisualizer::new(simple_grammar());
    assert!(!viz.to_dot().is_empty());
    assert!(!viz.to_railroad_svg().is_empty());
    assert!(!viz.to_text().is_empty());
    assert!(!viz.dependency_graph().is_empty());
}

#[test]
fn arithmetic_all_formats() {
    let viz = GrammarVisualizer::new(arithmetic_grammar());
    assert!(!viz.to_dot().is_empty());
    assert!(!viz.to_railroad_svg().is_empty());
    assert!(!viz.to_text().is_empty());
    assert!(!viz.dependency_graph().is_empty());
}

#[test]
fn chain_all_formats() {
    let viz = GrammarVisualizer::new(chain_grammar());
    assert!(!viz.to_dot().is_empty());
    assert!(!viz.to_railroad_svg().is_empty());
    assert!(!viz.to_text().is_empty());
    assert!(!viz.dependency_graph().is_empty());
}

#[test]
fn diamond_all_formats() {
    let viz = GrammarVisualizer::new(diamond_grammar());
    assert!(!viz.to_dot().is_empty());
    assert!(!viz.to_railroad_svg().is_empty());
    assert!(!viz.to_text().is_empty());
    assert!(!viz.dependency_graph().is_empty());
}

#[test]
fn recursive_all_formats() {
    let viz = GrammarVisualizer::new(recursive_grammar());
    assert!(!viz.to_dot().is_empty());
    assert!(!viz.to_railroad_svg().is_empty());
    assert!(!viz.to_text().is_empty());
    assert!(!viz.dependency_graph().is_empty());
}

// === Comparative tests ===

#[test]
fn arithmetic_bigger_dot_than_simple() {
    let simple_dot = GrammarVisualizer::new(simple_grammar()).to_dot();
    let arith_dot = GrammarVisualizer::new(arithmetic_grammar()).to_dot();
    assert!(
        arith_dot.len() > simple_dot.len(),
        "arithmetic ({}) should be bigger than simple ({})",
        arith_dot.len(),
        simple_dot.len()
    );
}

#[test]
fn arithmetic_bigger_text_than_simple() {
    let simple_text = GrammarVisualizer::new(simple_grammar()).to_text();
    let arith_text = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    assert!(arith_text.len() > simple_text.len());
}

#[test]
fn different_grammars_different_dot() {
    let a = GrammarVisualizer::new(simple_grammar()).to_dot();
    let b = GrammarVisualizer::new(arithmetic_grammar()).to_dot();
    assert_ne!(a, b);
}

#[test]
fn different_grammars_different_text() {
    let a = GrammarVisualizer::new(simple_grammar()).to_text();
    let b = GrammarVisualizer::new(arithmetic_grammar()).to_text();
    assert_ne!(a, b);
}

// === Constructor ===

#[test]
fn new_simple_no_panic() {
    let _ = GrammarVisualizer::new(simple_grammar());
}

#[test]
fn new_arithmetic_no_panic() {
    let _ = GrammarVisualizer::new(arithmetic_grammar());
}

#[test]
fn new_chain_no_panic() {
    let _ = GrammarVisualizer::new(chain_grammar());
}

#[test]
fn new_diamond_no_panic() {
    let _ = GrammarVisualizer::new(diamond_grammar());
}

#[test]
fn new_recursive_no_panic() {
    let _ = GrammarVisualizer::new(recursive_grammar());
}
