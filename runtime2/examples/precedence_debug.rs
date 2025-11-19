//! Debug test for precedence non-determinism
//!
//! This test runs the precedence case multiple times to detect non-determinism

use rust_sitter_runtime::{Parser, Tree};
use rust_sitter_runtime::tokenizer::{TokenPattern, Matcher};
use rust_sitter_runtime::language::SymbolMetadata;
use rust_sitter_glr_core::{SymbolId, ParseTable, FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{
    Grammar, ProductionId, Rule, Symbol,
    Token as IrToken, TokenPattern as IrTokenPattern,
};

fn create_arithmetic_grammar() -> (&'static ParseTable, Vec<SymbolMetadata>, Vec<TokenPattern>) {
    let mut grammar = Grammar::new("arithmetic".to_string());

    // Terminals
    let number_id = SymbolId(1);
    grammar.tokens.insert(
        number_id,
        IrToken {
            name: "NUMBER".to_string(),
            pattern: IrTokenPattern::String(r"\d+".to_string()),
            fragile: false,
        },
    );

    let minus_id = SymbolId(2);
    grammar.tokens.insert(
        minus_id,
        IrToken {
            name: "MINUS".to_string(),
            pattern: IrTokenPattern::String("-".to_string()),
            fragile: false,
        },
    );

    let star_id = SymbolId(3);
    grammar.tokens.insert(
        star_id,
        IrToken {
            name: "STAR".to_string(),
            pattern: IrTokenPattern::String("*".to_string()),
            fragile: false,
        },
    );

    let expr_id = SymbolId(4);
    grammar.rule_names.insert(expr_id, "expr".to_string());

    // Rule 1: expr → NUMBER
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });

    // Rule 2: expr → expr - expr (precedence 1, left assoc)
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(minus_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(rust_sitter_ir::PrecedenceKind::Static(1)),
        associativity: Some(rust_sitter_ir::Associativity::Left),
        production_id: ProductionId(1),
        fields: vec![],
    });

    // Rule 3: expr → expr * expr (precedence 2, left assoc)
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(star_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(rust_sitter_ir::PrecedenceKind::Static(2)),
        associativity: Some(rust_sitter_ir::Associativity::Left),
        production_id: ProductionId(2),
        fields: vec![],
    });

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let table_static: &'static ParseTable = Box::leak(Box::new(table));

    let symbol_metadata = vec![
        SymbolMetadata { is_terminal: true, is_visible: false, is_supertype: false }, // EOF
        SymbolMetadata { is_terminal: true, is_visible: true, is_supertype: false }, // NUMBER
        SymbolMetadata { is_terminal: true, is_visible: true, is_supertype: false }, // MINUS
        SymbolMetadata { is_terminal: true, is_visible: true, is_supertype: false }, // STAR
        SymbolMetadata { is_terminal: false, is_visible: true, is_supertype: false }, // expr
    ];

    let token_patterns = vec![
        TokenPattern {
            symbol_id: number_id,
            matcher: Matcher::Regex(regex::Regex::new(r"^\d+").unwrap()),
            is_keyword: false,
        },
        TokenPattern {
            symbol_id: minus_id,
            matcher: Matcher::Literal("-".to_string()),
            is_keyword: false,
        },
        TokenPattern {
            symbol_id: star_id,
            matcher: Matcher::Literal("*".to_string()),
            is_keyword: false,
        },
    ];

    (table_static, symbol_metadata, token_patterns)
}

fn parse(input: &str) -> Result<Tree, rust_sitter_runtime::error::ParseError> {
    let (table, metadata, patterns) = create_arithmetic_grammar();

    let mut parser = Parser::new();
    parser.set_glr_table(table)?;
    parser.set_symbol_metadata(metadata)?;
    parser.set_token_patterns(patterns)?;

    parser.parse(input.as_bytes(), None)
}

fn print_tree(tree: &Tree, prefix: &str) {
    let root = tree.root_node();
    print_node(&root, prefix, 0, tree.source_bytes().unwrap_or(b""));
}

fn print_node(node: &rust_sitter_runtime::node::Node, prefix: &str, depth: usize, source: &[u8]) {
    let indent = "  ".repeat(depth);
    let range = node.byte_range();
    let text = std::str::from_utf8(&source[range.clone()]).unwrap_or("<invalid>");

    println!("{}{}Node {{ kind_id: {}, range: {:?}, children: {}, text: {:?} }}",
             prefix, indent, node.kind_id(), range, node.child_count(), text);

    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            print_node(&child, prefix, depth + 1, source);
        }
    }
}

fn main() {
    println!("=== Testing Precedence Non-Determinism ===\n");

    let input = "1-2*3";
    let iterations = 5;

    println!("Parsing '{}' {} times to detect non-determinism\n", input, iterations);

    let mut trees = Vec::new();

    for i in 0..iterations {
        println!("--- Iteration {} ---", i + 1);
        match parse(input) {
            Ok(tree) => {
                print_tree(&tree, "");
                println!();
                trees.push(tree);
            }
            Err(e) => {
                println!("Parse failed: {:?}\n", e);
            }
        }
    }

    // Compare all trees
    println!("=== Comparison ===");

    if trees.len() < 2 {
        println!("Not enough successful parses to compare");
        return;
    }

    let reference = &trees[0];
    let mut all_same = true;

    for (i, tree) in trees.iter().enumerate().skip(1) {
        let ref_root = reference.root_node();
        let tree_root = tree.root_node();

        let same = nodes_equal(&ref_root, &tree_root);

        println!("Tree {} vs Tree 0: {}", i, if same { "IDENTICAL" } else { "DIFFERENT" });

        if !same {
            all_same = false;
            println!("  Tree 0:");
            print_tree(reference, "    ");
            println!("  Tree {}:", i);
            print_tree(tree, "    ");
        }
    }

    if all_same {
        println!("\n✅ All trees identical - No non-determinism detected");
    } else {
        println!("\n❌ Non-determinism detected - Trees differ across runs!");
    }
}

fn nodes_equal(n1: &rust_sitter_runtime::node::Node, n2: &rust_sitter_runtime::node::Node) -> bool {
    if n1.kind_id() != n2.kind_id() { return false; }
    if n1.byte_range() != n2.byte_range() { return false; }
    if n1.child_count() != n2.child_count() { return false; }

    for i in 0..n1.child_count() {
        let c1 = n1.child(i).unwrap();
        let c2 = n2.child(i).unwrap();
        if !nodes_equal(&c1, &c2) { return false; }
    }

    true
}
