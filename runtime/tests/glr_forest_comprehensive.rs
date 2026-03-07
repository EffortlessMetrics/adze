// Comprehensive tests for GLR parse forest structures
use adze::adze_ir as ir;
use adze::glr_forest::*;

use ir::{RuleId, SymbolId};
use proptest::prelude::*;
use std::rc::Rc;

// ---------------------------------------------------------------------------
// ForestNode construction
// ---------------------------------------------------------------------------

#[test]
fn terminal_node_construction() {
    let node = ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 3,
        text: b"abc".to_vec(),
    };
    match &node {
        ForestNode::Terminal {
            symbol,
            start,
            end,
            text,
        } => {
            assert_eq!(*symbol, SymbolId(1));
            assert_eq!(*start, 0);
            assert_eq!(*end, 3);
            assert_eq!(text, b"abc");
        }
        _ => panic!("Expected Terminal"),
    }
}

#[test]
fn nonterminal_node_empty_alternatives() {
    let node = ForestNode::NonTerminal {
        symbol: SymbolId(10),
        start: 0,
        end: 5,
        alternatives: vec![],
    };
    match &node {
        ForestNode::NonTerminal { alternatives, .. } => {
            assert!(alternatives.is_empty());
        }
        _ => panic!("Expected NonTerminal"),
    }
}

#[test]
fn nonterminal_with_one_alternative() {
    let child = Rc::new(ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 1,
        text: b"x".to_vec(),
    });
    let packed = PackedNode {
        rule_id: RuleId(0),
        children: vec![child],
    };
    let node = ForestNode::NonTerminal {
        symbol: SymbolId(10),
        start: 0,
        end: 1,
        alternatives: vec![packed],
    };
    match &node {
        ForestNode::NonTerminal { alternatives, .. } => {
            assert_eq!(alternatives.len(), 1);
            assert_eq!(alternatives[0].children.len(), 1);
        }
        _ => panic!("Expected NonTerminal"),
    }
}

#[test]
fn nonterminal_with_multiple_alternatives() {
    let child_a = Rc::new(ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 1,
        text: b"a".to_vec(),
    });
    let child_b = Rc::new(ForestNode::Terminal {
        symbol: SymbolId(2),
        start: 0,
        end: 1,
        text: b"b".to_vec(),
    });
    let alt1 = PackedNode {
        rule_id: RuleId(0),
        children: vec![child_a],
    };
    let alt2 = PackedNode {
        rule_id: RuleId(1),
        children: vec![child_b],
    };
    let node = ForestNode::NonTerminal {
        symbol: SymbolId(10),
        start: 0,
        end: 1,
        alternatives: vec![alt1, alt2],
    };
    match &node {
        ForestNode::NonTerminal { alternatives, .. } => {
            assert_eq!(alternatives.len(), 2);
        }
        _ => panic!("Expected NonTerminal"),
    }
}

// ---------------------------------------------------------------------------
// ForestNode clone/debug
// ---------------------------------------------------------------------------

#[test]
fn terminal_node_clone() {
    let node = ForestNode::Terminal {
        symbol: SymbolId(5),
        start: 10,
        end: 20,
        text: b"hello".to_vec(),
    };
    let cloned = node.clone();
    match cloned {
        ForestNode::Terminal {
            symbol, start, end, ..
        } => {
            assert_eq!(symbol, SymbolId(5));
            assert_eq!(start, 10);
            assert_eq!(end, 20);
        }
        _ => panic!("Expected Terminal"),
    }
}

#[test]
fn terminal_node_debug() {
    let node = ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 1,
        text: b"x".to_vec(),
    };
    let debug = format!("{:?}", node);
    assert!(debug.contains("Terminal"));
}

#[test]
fn nonterminal_node_debug() {
    let node = ForestNode::NonTerminal {
        symbol: SymbolId(10),
        start: 0,
        end: 5,
        alternatives: vec![],
    };
    let debug = format!("{:?}", node);
    assert!(debug.contains("NonTerminal"));
}

// ---------------------------------------------------------------------------
// PackedNode tests
// ---------------------------------------------------------------------------

#[test]
fn packed_node_construction() {
    let packed = PackedNode {
        rule_id: RuleId(3),
        children: vec![],
    };
    assert_eq!(packed.rule_id, RuleId(3));
    assert!(packed.children.is_empty());
}

#[test]
fn packed_node_with_children() {
    let child = Rc::new(ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 1,
        text: b"a".to_vec(),
    });
    let packed = PackedNode {
        rule_id: RuleId(0),
        children: vec![child.clone(), child],
    };
    assert_eq!(packed.children.len(), 2);
}

#[test]
fn packed_node_clone() {
    let packed = PackedNode {
        rule_id: RuleId(5),
        children: vec![],
    };
    let cloned = packed.clone();
    assert_eq!(cloned.rule_id, RuleId(5));
}

#[test]
fn packed_node_debug() {
    let packed = PackedNode {
        rule_id: RuleId(0),
        children: vec![],
    };
    let debug = format!("{:?}", packed);
    assert!(debug.contains("PackedNode"));
}

// ---------------------------------------------------------------------------
// GLRStats tests
// ---------------------------------------------------------------------------

#[test]
fn glr_stats_default() {
    let stats = GLRStats::default();
    assert_eq!(stats.total_nodes_created, 0);
    assert_eq!(stats.max_active_heads, 0);
    assert_eq!(stats.total_forks, 0);
    assert_eq!(stats.total_merges, 0);
    assert_eq!(stats.forest_cache_hits, 0);
}

#[test]
fn glr_stats_debug() {
    let stats = GLRStats::default();
    let debug = format!("{:?}", stats);
    assert!(debug.contains("GLRStats"));
}

#[test]
fn glr_stats_clone() {
    let stats = GLRStats {
        total_nodes_created: 100,
        max_active_heads: 5,
        total_forks: 10,
        total_merges: 3,
        forest_cache_hits: 20,
    };
    let cloned = stats.clone();
    assert_eq!(cloned.total_nodes_created, 100);
    assert_eq!(cloned.total_forks, 10);
}

// ---------------------------------------------------------------------------
// GLRParserState tests
// ---------------------------------------------------------------------------

#[test]
fn glr_parser_state_new() {
    let state = GLRParserState::new();
    let stats = state.get_stats();
    assert_eq!(stats.total_forks, 0);
}

#[test]
fn glr_parser_state_fork() {
    let mut state = GLRParserState::new();
    let idx = state.fork(0, 1);
    assert!(idx > 0);
}

#[test]
fn glr_parser_state_multiple_forks() {
    let mut state = GLRParserState::new();
    let idx1 = state.fork(0, 1);
    let idx2 = state.fork(0, 2);
    assert_ne!(idx1, idx2);
}

// ---------------------------------------------------------------------------
// forest_to_parse_tree tests
// ---------------------------------------------------------------------------

#[test]
fn terminal_to_parse_tree() {
    let node = ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 3,
        text: b"abc".to_vec(),
    };
    let tree = forest_to_parse_tree(&node);
    assert_eq!(tree.symbol, SymbolId(1));
    assert_eq!(tree.start_byte, 0);
    assert_eq!(tree.end_byte, 3);
    assert!(tree.children.is_empty());
}

#[test]
fn nonterminal_single_alt_to_parse_tree() {
    let child = Rc::new(ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 1,
        text: b"x".to_vec(),
    });
    let packed = PackedNode {
        rule_id: RuleId(0),
        children: vec![child],
    };
    let node = ForestNode::NonTerminal {
        symbol: SymbolId(10),
        start: 0,
        end: 1,
        alternatives: vec![packed],
    };
    let tree = forest_to_parse_tree(&node);
    assert_eq!(tree.symbol, SymbolId(10));
    assert_eq!(tree.children.len(), 1);
}

#[test]
fn nonterminal_empty_alt_to_parse_tree() {
    let node = ForestNode::NonTerminal {
        symbol: SymbolId(10),
        start: 0,
        end: 0,
        alternatives: vec![],
    };
    let tree = forest_to_parse_tree(&node);
    assert_eq!(tree.symbol, SymbolId(10));
    assert!(tree.children.is_empty());
}

// ---------------------------------------------------------------------------
// forest_to_parse_trees tests
// ---------------------------------------------------------------------------

#[test]
fn terminal_to_parse_trees() {
    let node = ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 1,
        text: b"a".to_vec(),
    };
    let trees = forest_to_parse_trees(&node);
    assert_eq!(trees.len(), 1);
}

#[test]
fn ambiguous_to_parse_trees() {
    let child_a = Rc::new(ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 1,
        text: b"x".to_vec(),
    });
    let child_b = Rc::new(ForestNode::Terminal {
        symbol: SymbolId(2),
        start: 0,
        end: 1,
        text: b"x".to_vec(),
    });
    let alt1 = PackedNode {
        rule_id: RuleId(0),
        children: vec![child_a],
    };
    let alt2 = PackedNode {
        rule_id: RuleId(1),
        children: vec![child_b],
    };
    let node = ForestNode::NonTerminal {
        symbol: SymbolId(10),
        start: 0,
        end: 1,
        alternatives: vec![alt1, alt2],
    };
    let trees = forest_to_parse_trees(&node);
    assert_eq!(trees.len(), 2);
}

// ---------------------------------------------------------------------------
// Nested forest structures
// ---------------------------------------------------------------------------

#[test]
fn nested_nonterminal_tree() {
    let leaf = Rc::new(ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 1,
        text: b"a".to_vec(),
    });
    let inner = Rc::new(ForestNode::NonTerminal {
        symbol: SymbolId(11),
        start: 0,
        end: 1,
        alternatives: vec![PackedNode {
            rule_id: RuleId(0),
            children: vec![leaf],
        }],
    });
    let outer = ForestNode::NonTerminal {
        symbol: SymbolId(10),
        start: 0,
        end: 1,
        alternatives: vec![PackedNode {
            rule_id: RuleId(1),
            children: vec![inner],
        }],
    };
    let tree = forest_to_parse_tree(&outer);
    assert_eq!(tree.symbol, SymbolId(10));
    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].symbol, SymbolId(11));
    assert_eq!(tree.children[0].children.len(), 1);
}

#[test]
fn shared_forest_node() {
    let shared = Rc::new(ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 1,
        text: b"x".to_vec(),
    });
    // Same node used in two different packed nodes
    let alt1 = PackedNode {
        rule_id: RuleId(0),
        children: vec![shared.clone()],
    };
    let alt2 = PackedNode {
        rule_id: RuleId(1),
        children: vec![shared.clone()],
    };
    let node = ForestNode::NonTerminal {
        symbol: SymbolId(10),
        start: 0,
        end: 1,
        alternatives: vec![alt1, alt2],
    };
    let trees = forest_to_parse_trees(&node);
    assert_eq!(trees.len(), 2);
    // Both trees should have the same leaf
    assert_eq!(trees[0].children[0].symbol, SymbolId(1));
    assert_eq!(trees[1].children[0].symbol, SymbolId(1));
}

// ---------------------------------------------------------------------------
// GSSNode / GSSLink tests
// ---------------------------------------------------------------------------

#[test]
fn gss_node_construction() {
    let node = GSSNode {
        state: 0,
        parents: vec![],
        id: 0,
    };
    assert_eq!(node.state, 0);
    assert!(node.parents.is_empty());
}

#[test]
fn gss_link_construction() {
    let tree = Rc::new(ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 1,
        text: b"a".to_vec(),
    });
    let link = GSSLink {
        parent: 0,
        tree_node: tree,
    };
    assert_eq!(link.parent, 0);
}

#[test]
fn gss_node_with_parents() {
    let tree = Rc::new(ForestNode::Terminal {
        symbol: SymbolId(1),
        start: 0,
        end: 1,
        text: b"a".to_vec(),
    });
    let link = GSSLink {
        parent: 0,
        tree_node: tree,
    };
    let node = GSSNode {
        state: 5,
        parents: vec![link],
        id: 1,
    };
    assert_eq!(node.parents.len(), 1);
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn terminal_preserves_byte_range(start in 0usize..1000, len in 0usize..1000) {
        let end = start + len;
        let node = ForestNode::Terminal {
            symbol: SymbolId(1),
            start,
            end,
            text: vec![b'x'; len],
        };
        let tree = forest_to_parse_tree(&node);
        prop_assert_eq!(tree.start_byte, start);
        prop_assert_eq!(tree.end_byte, end);
    }

    #[test]
    fn terminal_preserves_symbol(sym_id in 0u16..1000) {
        let node = ForestNode::Terminal {
            symbol: SymbolId(sym_id),
            start: 0,
            end: 1,
            text: vec![b'x'],
        };
        let tree = forest_to_parse_tree(&node);
        prop_assert_eq!(tree.symbol, SymbolId(sym_id));
    }

    #[test]
    fn nonterminal_preserves_symbol(sym_id in 0u16..1000) {
        let child = Rc::new(ForestNode::Terminal {
            symbol: SymbolId(1),
            start: 0,
            end: 1,
            text: vec![b'x'],
        });
        let node = ForestNode::NonTerminal {
            symbol: SymbolId(sym_id),
            start: 0,
            end: 1,
            alternatives: vec![PackedNode { rule_id: RuleId(0), children: vec![child] }],
        };
        let tree = forest_to_parse_tree(&node);
        prop_assert_eq!(tree.symbol, SymbolId(sym_id));
    }

    #[test]
    fn alternative_count_matches_trees(n in 1usize..5) {
        let alternatives: Vec<PackedNode> = (0..n).map(|i| {
            let child = Rc::new(ForestNode::Terminal {
                symbol: SymbolId(i as u16 + 1),
                start: 0,
                end: 1,
                text: vec![b'x'],
            });
            PackedNode { rule_id: RuleId(i as u16), children: vec![child] }
        }).collect();
        let node = ForestNode::NonTerminal {
            symbol: SymbolId(10),
            start: 0,
            end: 1,
            alternatives,
        };
        let trees = forest_to_parse_trees(&node);
        prop_assert_eq!(trees.len(), n);
    }

    #[test]
    fn glr_stats_clone_preserves(
        nodes in 0usize..1000,
        heads in 0usize..100,
        forks in 0usize..50,
        merges in 0usize..50,
        hits in 0usize..500,
    ) {
        let stats = GLRStats {
            total_nodes_created: nodes,
            max_active_heads: heads,
            total_forks: forks,
            total_merges: merges,
            forest_cache_hits: hits,
        };
        let cloned = stats.clone();
        prop_assert_eq!(cloned.total_nodes_created, nodes);
        prop_assert_eq!(cloned.max_active_heads, heads);
        prop_assert_eq!(cloned.total_forks, forks);
        prop_assert_eq!(cloned.total_merges, merges);
        prop_assert_eq!(cloned.forest_cache_hits, hits);
    }
}
