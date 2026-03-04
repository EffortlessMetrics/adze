#![no_main]

use adze::glr_tree_bridge::subtree_to_tree;
use adze::subtree::{Subtree, SubtreeNode};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

fn create_test_grammar() -> Grammar {
    let mut grammar = Grammar::new("fuzz_ser".to_string());

    let expr_id = SymbolId(0);
    let number_id = SymbolId(1);
    let plus_id = SymbolId(2);

    grammar.rule_names.insert(expr_id, "expression".to_string());

    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(plus_id),
            Symbol::NonTerminal(expr_id),
        ],
        production_id: ProductionId(0),
        precedence: None,
        associativity: None,
        fields: vec![],
    });

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        production_id: ProductionId(1),
        precedence: None,
        associativity: None,
        fields: vec![],
    });

    grammar
}

/// Build a tree from fuzzer-controlled source bytes.
fn build_tree(data: &[u8]) -> (Arc<Subtree>, Vec<u8>) {
    let source = data.to_vec();
    let len = source.len();

    // Build leaf nodes from byte ranges (one per byte, capped).
    let cap = len.min(32);
    let mut children: Vec<Arc<Subtree>> = Vec::new();
    for i in 0..cap {
        children.push(Arc::new(Subtree::new(
            SubtreeNode {
                symbol_id: SymbolId(1),
                is_error: false,
                byte_range: i..i + 1,
            },
            vec![],
        )));
    }

    let root = Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(0),
            is_error: false,
            byte_range: 0..len,
        },
        children,
    ));

    (root, source)
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > 10_000 {
        return;
    }

    let (root, source) = build_tree(data);
    let grammar = create_test_grammar();
    let tree = subtree_to_tree(root, source.clone(), grammar);
    let root_node = tree.root_node();

    // 1. S-expression roundtrip: serialize and verify it produces a string.
    let sexp = root_node.to_sexp();
    assert!(!sexp.is_empty());

    // 2. Serialize child nodes and verify consistency.
    for i in 0..root_node.child_count() {
        if let Some(child) = root_node.child(i) {
            let child_sexp = child.to_sexp();
            assert!(!child_sexp.is_empty());

            // Byte range must be within source.
            assert!(child.end_byte() <= source.len());
            assert!(child.start_byte() <= child.end_byte());
        }
    }

    // 3. Verify that to_sexp doesn't panic on cursor-based traversal.
    let mut cursor = root_node.walk();
    if cursor.goto_first_child() {
        loop {
            let node = cursor.node();
            let _ = node.to_sexp();
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
});
