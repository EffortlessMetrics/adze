// Bridge between parser_v4::Tree and GLR ForestNode representations

use crate::glr_incremental::{ForkAlternative, ForestNode};
use crate::parser_v4::Tree as V4Tree;
use crate::subtree::{Subtree, SubtreeNode};
use rust_sitter_ir::SymbolId;
use std::ops::Range;
use std::sync::Arc;

/// Convert a simple parser_v4::Tree to a ForestNode for incremental parsing
/// 
/// This creates an unambiguous forest (single alternative) that represents
/// the existing parse tree structure.
pub fn v4_tree_to_forest(tree: &V4Tree) -> Arc<ForestNode> {
    // For now, create a minimal forest node representing just the root
    // In a real implementation, this would walk the entire tree structure
    
    let subtree_node = SubtreeNode {
        symbol_id: SymbolId(tree.root_kind),
        is_error: tree.error_count > 0,
        byte_range: 0..tree.source.len(),
    };
    
    let subtree = Arc::new(Subtree::new(subtree_node, vec![]));
    
    Arc::new(ForestNode {
        symbol: SymbolId(tree.root_kind),
        alternatives: vec![ForkAlternative {
            fork_id: 0,
            rule_id: None,
            children: vec![], // Would be populated from tree structure
            subtree,
        }],
        byte_range: 0..tree.source.len(),
        token_range: 0..0, // Would need proper token counting
    })
}

/// Convert a ForestNode back to a simple parser_v4::Tree
/// 
/// This flattens the potentially ambiguous forest by selecting the first
/// valid alternative at each node.
pub fn forest_to_v4_tree(forest: &ForestNode, source: String) -> V4Tree {
    // Select the first alternative (disambiguation strategy)
    let _primary_alt = forest.alternatives.first()
        .expect("ForestNode must have at least one alternative");
    
    // Count errors by traversing the forest
    let error_count = count_errors_in_forest(forest);
    
    V4Tree {
        root_kind: forest.symbol.0,
        error_count,
        source,
    }
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
    
    #[test]
    fn test_v4_to_forest_conversion() {
        let v4_tree = V4Tree {
            root_kind: 42,
            error_count: 0,
            source: "let x = 42;".to_string(),
        };
        
        let forest = v4_tree_to_forest(&v4_tree);
        assert_eq!(forest.symbol.0, 42);
        assert_eq!(forest.alternatives.len(), 1);
        assert_eq!(forest.byte_range, 0..11);
    }
    
    #[test]
    fn test_forest_to_v4_conversion() {
        let subtree_node = SubtreeNode {
            symbol_id: SymbolId(42),
            is_error: false,
            byte_range: 0..11,
        };
        
        let forest = ForestNode {
            symbol: SymbolId(42),
            alternatives: vec![ForkAlternative {
                fork_id: 0,
                rule_id: None,
                children: vec![],
                subtree: Arc::new(Subtree::new(subtree_node, vec![])),
            }],
            byte_range: 0..11,
            token_range: 0..1,
        };
        
        let v4_tree = forest_to_v4_tree(&forest, "let x = 42;".to_string());
        assert_eq!(v4_tree.root_kind, 42);
        assert_eq!(v4_tree.error_count, 0);
        assert_eq!(v4_tree.source, "let x = 42;");
    }
}