#![no_main]

use adze::glr_tree_bridge::subtree_to_tree;
use adze::subtree::{Subtree, SubtreeNode};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use std::sync::Arc;

fn create_test_grammar() -> Grammar {
    let mut grammar = Grammar::new("fuzz_cursor".to_string());

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

/// Navigation actions the fuzzer can choose from.
#[derive(Debug, Arbitrary)]
enum NavAction {
    GotoFirstChild,
    GotoNextSibling,
    GotoParent,
}

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    /// How many children the root node has (capped to keep trees small).
    child_count: u8,
    /// Sequence of cursor navigation actions.
    actions: Vec<NavAction>,
}

/// Build a synthetic tree with `n` children (each a leaf) under a root node.
fn build_tree(n: usize) -> (Arc<Subtree>, Vec<u8>) {
    let source = "1+2+3+4+5+6+7+8";
    let bytes = source.as_bytes().to_vec();
    let len = bytes.len();

    let mut children: Vec<Arc<Subtree>> = Vec::new();
    for i in 0..n {
        let start = i.min(len);
        let end = (i + 1).min(len);
        children.push(Arc::new(Subtree::new(
            SubtreeNode {
                symbol_id: SymbolId(1),
                is_error: false,
                byte_range: start..end,
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

    (root, bytes)
}

fuzz_target!(|input: FuzzInput| {
    let n = (input.child_count as usize).min(16);
    if input.actions.len() > 1000 {
        return;
    }

    let (root, source) = build_tree(n);
    let grammar = create_test_grammar();
    let tree = subtree_to_tree(root, source, grammar);
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();

    for action in &input.actions {
        match action {
            NavAction::GotoFirstChild => {
                let _ = cursor.goto_first_child();
            }
            NavAction::GotoNextSibling => {
                let _ = cursor.goto_next_sibling();
            }
            NavAction::GotoParent => {
                let _ = cursor.goto_parent();
            }
        }

        // Access node properties at current position — should never panic.
        let node = cursor.node();
        let _ = node.kind();
        let _ = node.start_byte();
        let _ = node.end_byte();
        let _ = node.child_count();
        let _ = node.is_error();
    }
});
