// Bridge between parser_v4::Tree and GLR ForestNode representations
//
// TODO(Phase 2 Day 5): Update for Tree<'arena> with NodeHandle
// This module needs updates to work with the new arena-based Tree type.
// For Day 4, we're establishing type signatures only.

#![allow(dead_code)] // Temporarily allow until Day 5 updates

use crate::glr_incremental::ForestNode;
use crate::parser_v4::Tree as V4Tree;
use std::sync::Arc;

/// Convert a simple parser_v4::Tree to a ForestNode for incremental parsing
///
/// This creates an unambiguous forest (single alternative) that represents
/// the existing parse tree structure.
///
/// TODO(Phase 2 Day 5): Update for Tree<'arena>
#[allow(unused_variables)]
pub fn v4_tree_to_forest<'arena>(tree: &V4Tree<'arena>) -> Arc<ForestNode> {
    // TODO(Phase 2 Day 5): Update for Tree<'arena> with NodeHandle
    // This function needs to access root node via tree.root_node()
    // and traverse the arena-allocated tree structure
    unimplemented!("v4_tree_to_forest will be updated for Tree<'arena> in Day 5")
}

/// Convert a ForestNode back to a simple parser_v4::Tree
///
/// This flattens the potentially ambiguous forest by selecting the first
/// valid alternative at each node.
///
/// TODO(Phase 2 Day 5): Update for Tree<'arena>
#[allow(unused_variables)]
pub fn forest_to_v4_tree<'arena>(forest: &ForestNode) -> V4Tree<'arena> {
    // TODO(Phase 2 Day 5): Construct Tree<'arena> with NodeHandle
    // This function needs to allocate nodes in an arena and construct
    // a Tree with proper root handle and arena reference
    unimplemented!("forest_to_v4_tree will be updated for Tree<'arena> in Day 5")
}

/// Count errors in a forest by traversing all nodes
fn count_errors_in_forest(forest: &ForestNode) -> usize {
    let mut error_count = 0;

    // For simplicity, just check the first alternative
    if let Some(alt) = forest.alternatives.first() {
        if alt.subtree.is_error() {
            error_count += 1;
        }
        // Recursively count errors in children
        for child in &alt.children {
            error_count += count_errors_in_forest(child);
        }
    }

    error_count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glr_incremental::ForkAlternative;
    use crate::subtree::{Subtree, SubtreeNode};
    use rust_sitter_ir::SymbolId;

    #[test]
    #[ignore = "v4_tree_to_forest is unimplemented - requires arena-based Tree construction"]
    fn test_v4_to_forest_conversion() {
        // TODO: This test cannot run because:
        // 1. V4Tree (parser_v4::Tree<'arena>) doesn't have root_kind/source fields
        // 2. v4_tree_to_forest is unimplemented
        // Once Phase 2 Day 5 is complete, update this test.
        let _forest: Arc<ForestNode> = unimplemented!();
    }

    #[test]
    #[ignore = "forest_to_v4_tree is unimplemented - requires arena-based Tree construction"]
    fn test_forest_to_v4_conversion() {
        let subtree_node = SubtreeNode {
            symbol_id: SymbolId(42),
            is_error: false,
            byte_range: 0..11,
        };

        let subtree = Arc::new(Subtree::new(subtree_node, vec![]));
        let _forest = ForestNode {
            symbol: SymbolId(42),
            alternatives: vec![ForkAlternative {
                fork_id: 0,
                rule_id: None,
                children: vec![],
                subtree: subtree.clone(),
            }],
            byte_range: 0..11,
            token_range: 0..1,
            cached_subtree: Some(subtree),
        };

        // TODO: Implement forest_to_v4_tree and update test
        // let v4_tree = forest_to_v4_tree(&forest);
        // assert_eq!(v4_tree.error_count(), 0);
    }
}
