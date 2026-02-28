// Bridge between parser_v4::Tree and GLR ForestNode representations.

use crate::arena_allocator::{NodeHandle, TreeArena, TreeNode};
use crate::glr_incremental::{ForestNode, ForkAlternative};
use crate::parser_v4::Tree as V4Tree;
use crate::subtree::{Subtree, SubtreeNode};
use adze_ir::SymbolId;
use std::sync::Arc;

fn make_subtree(symbol: SymbolId, children: Vec<Arc<Subtree>>) -> Arc<Subtree> {
    make_subtree_with_error(symbol, false, children)
}

fn make_subtree_with_error(
    symbol: SymbolId,
    is_error: bool,
    children: Vec<Arc<Subtree>>,
) -> Arc<Subtree> {
    Arc::new(Subtree::new(
        SubtreeNode {
            symbol_id: symbol,
            is_error,
            byte_range: 0..0,
        },
        children,
    ))
}

fn v4_to_forest_node<'arena>(tree: &V4Tree<'arena>, handle: NodeHandle) -> Arc<ForestNode> {
    let node = tree.arena.get(handle);
    let symbol = SymbolId(node.symbol() as u16);

    let children: Vec<Arc<ForestNode>> = node
        .children()
        .iter()
        .copied()
        .map(|child| v4_to_forest_node(tree, child))
        .collect();

    let child_subtrees = children
        .iter()
        .map(|forest_node| {
            let child_symbol = forest_node.symbol;
            forest_node
                .cached_subtree
                .clone()
                .unwrap_or_else(|| make_subtree(child_symbol, Vec::new()))
        })
        .collect();

    let subtree = make_subtree(symbol, child_subtrees);

    Arc::new(ForestNode {
        symbol,
        alternatives: vec![ForkAlternative {
            fork_id: 0,
            rule_id: None,
            children: children.clone(),
            subtree: Arc::clone(&subtree),
        }],
        byte_range: 0..0,
        token_range: 0..0,
        cached_subtree: Some(subtree),
    })
}

/// Convert a parser_v4::Tree to a ForestNode for incremental parsing.
///
/// This creates an unambiguous forest (single alternative) that represents
/// the existing parse tree structure.
pub fn v4_tree_to_forest<'arena>(tree: &V4Tree<'arena>) -> Arc<ForestNode> {
    v4_to_forest_node(tree, tree.root)
}

/// Convert a ForestNode back to a simple parser_v4::Tree.
///
/// This flattens forest alternatives by selecting the first alternative
/// at each node.
pub fn forest_to_v4_tree<'arena>(forest: &ForestNode) -> V4Tree<'arena> {
    let arena = Box::leak(Box::new(TreeArena::new()));
    let root = forest_to_v4_node(arena, forest);
    let error_count = forest
        .cached_subtree
        .as_ref()
        .map(|subtree| usize::from(subtree.is_error()))
        .unwrap_or(0);

    V4Tree {
        root,
        arena,
        error_count,
    }
}

fn forest_to_v4_node(arena: &mut TreeArena, forest: &ForestNode) -> NodeHandle {
    let child_handles = forest
        .alternatives
        .first()
        .map(|alternative| {
            alternative
                .children
                .iter()
                .map(|child| forest_to_v4_node(arena, child))
                .collect()
        })
        .unwrap_or_default();

    let has_cached_structure = forest
        .cached_subtree
        .as_ref()
        .is_some_and(|subtree| !subtree.children.is_empty());

    if child_handles.is_empty() && !has_cached_structure {
        arena.alloc(TreeNode::leaf(forest.symbol.0 as i32))
    } else {
        arena.alloc(TreeNode::branch_with_symbol(
            forest.symbol.0 as i32,
            child_handles,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v4_to_forest_conversion_contract() {
        let arena = Box::leak(Box::new(TreeArena::new()));
        let child = arena.alloc(TreeNode::leaf(7));
        let root = arena.alloc(TreeNode::branch_with_symbol(13, vec![child]));
        let tree = V4Tree {
            root,
            arena,
            error_count: 0,
        };

        let forest = v4_tree_to_forest(&tree);

        assert_eq!(forest.symbol, SymbolId(13));
        assert_eq!(forest.alternatives.len(), 1);
        assert_eq!(forest.alternatives[0].children.len(), 1);
        assert_eq!(forest.alternatives[0].children[0].symbol, SymbolId(7));
        assert_eq!(forest.cached_subtree.as_ref().unwrap().symbol(), 13);
    }

    #[test]
    fn test_forest_to_v4_conversion_contract() {
        let child_subtree = Arc::new(Subtree::new(
            SubtreeNode {
                symbol_id: SymbolId(7),
                is_error: false,
                byte_range: 0..1,
            },
            vec![],
        ));

        let child_forest = Arc::new(ForestNode {
            symbol: SymbolId(7),
            alternatives: vec![],
            byte_range: 0..1,
            token_range: 0..1,
            cached_subtree: Some(Arc::clone(&child_subtree)),
        });

        let root_subtree = Arc::new(Subtree::new(
            SubtreeNode {
                symbol_id: SymbolId(13),
                is_error: false,
                byte_range: 0..2,
            },
            vec![Arc::clone(&child_subtree)],
        ));

        let forest = ForestNode {
            symbol: SymbolId(13),
            alternatives: vec![ForkAlternative {
                fork_id: 0,
                rule_id: None,
                children: vec![Arc::clone(&child_forest)],
                subtree: Arc::clone(&root_subtree),
            }],
            byte_range: 0..2,
            token_range: 0..2,
            cached_subtree: Some(root_subtree),
        };

        let tree = forest_to_v4_tree(&forest);
        let root_node = tree.arena.get(tree.root);

        assert_eq!(root_node.symbol(), 13);
        assert!(!root_node.is_leaf());
        let child_handles = root_node.children();
        assert_eq!(child_handles.len(), 1);

        let child_node = tree.arena.get(child_handles[0]);
        assert!(child_node.is_leaf());
        assert_eq!(child_node.symbol(), 7);
    }

    #[test]
    fn test_forest_to_v4_prefers_cached_structure_over_empty_alternatives() {
        let leaf_subtree = Arc::new(Subtree::new(
            SubtreeNode {
                symbol_id: SymbolId(9),
                is_error: false,
                byte_range: 0..1,
            },
            vec![],
        ));

        let parent_cached = Arc::new(Subtree::new(
            SubtreeNode {
                symbol_id: SymbolId(11),
                is_error: false,
                byte_range: 0..2,
            },
            vec![Arc::clone(&leaf_subtree)],
        ));

        let forest = ForestNode {
            symbol: SymbolId(11),
            alternatives: vec![],
            byte_range: 0..2,
            token_range: 0..2,
            cached_subtree: Some(parent_cached),
        };

        let tree = forest_to_v4_tree(&forest);
        let root_node = tree.arena.get(tree.root);

        assert_eq!(root_node.symbol(), 11);
        assert!(!root_node.is_leaf());
        assert!(root_node.children().is_empty());
    }
}
