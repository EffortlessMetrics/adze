//! Tests for ambiguity preservation in compression

#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;
use adze::subtree::{Subtree, SubtreeNode};

#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use ir::SymbolId;
use std::sync::Arc;

#[test]
fn test_merge_ambiguous_no_duplicates() {
    // Create two different subtrees with same top
    let node = SubtreeNode {
        symbol_id: SymbolId(1),
        is_error: false,
        byte_range: 0..5,
    };

    let tree1 = Subtree::new(node.clone(), vec![]);
    let tree2 = Arc::new(Subtree::with_dynamic_prec(node.clone(), vec![], 2));

    // Merge them
    let merged = tree1.merge_ambiguous(tree2.clone());

    // Should have one alternative
    assert_eq!(merged.alternatives.len(), 1);
    assert!(Arc::ptr_eq(&merged.alternatives[0], &tree2));

    // Dynamic precedence should be the max (2)
    assert_eq!(merged.dynamic_prec, 2);
}

#[test]
fn test_merge_ambiguous_with_existing_alts() {
    let node = SubtreeNode {
        symbol_id: SymbolId(1),
        is_error: false,
        byte_range: 0..5,
    };

    // Create tree1 with an existing alternative
    let alt1 = Arc::new(Subtree::new(node.clone(), vec![]));
    let mut tree1 = Subtree::new(node.clone(), vec![]);
    tree1.alternatives.push(alt1.clone());

    // Create tree2 with different alternatives
    let alt2 = Arc::new(Subtree::with_dynamic_prec(node.clone(), vec![], 3));
    let mut tree2 = Subtree::new(node.clone(), vec![]);
    tree2.alternatives.push(alt2.clone());

    // Merge them
    let merged = tree1.merge_ambiguous(Arc::new(tree2));

    // Should have all unique alternatives
    assert_eq!(merged.alternatives.len(), 3); // alt1, alt2, and tree2 itself
    assert!(merged.alternatives.iter().any(|a| Arc::ptr_eq(a, &alt1)));
    assert!(merged.alternatives.iter().any(|a| Arc::ptr_eq(a, &alt2)));
}

#[test]
fn test_merge_prevents_duplicate_by_pointer() {
    let node = SubtreeNode {
        symbol_id: SymbolId(1),
        is_error: false,
        byte_range: 0..5,
    };

    let tree1 = Subtree::new(node.clone(), vec![]);
    let tree2 = Arc::new(Subtree::new(node.clone(), vec![]));

    // Merge the same tree twice
    let merged = tree1
        .merge_ambiguous(tree2.clone())
        .merge_ambiguous(tree2.clone());

    // Should only have one alternative (no duplicates)
    assert_eq!(merged.alternatives.len(), 1);
}

#[test]
fn test_has_alts_helper() {
    let node = SubtreeNode {
        symbol_id: SymbolId(1),
        is_error: false,
        byte_range: 0..5,
    };

    let mut tree = Subtree::new(node.clone(), vec![]);
    assert!(!tree.has_alts());

    tree.alternatives.push(Arc::new(Subtree::new(node, vec![])));
    assert!(tree.has_alts());
}

#[test]
fn test_push_alt_helper() {
    let node = SubtreeNode {
        symbol_id: SymbolId(1),
        is_error: false,
        byte_range: 0..5,
    };

    let tree = Subtree::new(node.clone(), vec![]);
    let alt = Arc::new(Subtree::with_dynamic_prec(node.clone(), vec![], 5));

    let with_alt = tree.push_alt(alt.clone());

    assert_eq!(with_alt.alternatives.len(), 1);
    assert!(Arc::ptr_eq(&with_alt.alternatives[0], &alt));
    assert_eq!(with_alt.dynamic_prec, 5); // Max precedence propagated
}

#[test]
fn test_concat_alts_merges_all() {
    let node = SubtreeNode {
        symbol_id: SymbolId(1),
        is_error: false,
        byte_range: 0..5,
    };

    // Create tree1 with one alt
    let alt1 = Arc::new(Subtree::new(node.clone(), vec![]));
    let mut tree1 = Subtree::new(node.clone(), vec![]);
    tree1.alternatives.push(alt1.clone());

    // Create tree2 with two alts
    let alt2 = Arc::new(Subtree::new(node.clone(), vec![]));
    let alt3 = Arc::new(Subtree::new(node.clone(), vec![]));
    let mut tree2 = Subtree::new(node.clone(), vec![]);
    tree2.alternatives.push(alt2.clone());
    tree2.alternatives.push(alt3.clone());

    // Concat them
    let concatenated = tree1.concat_alts(Arc::new(tree2.clone()));

    // Should have all alternatives: alt1, tree2, alt2, alt3
    assert_eq!(concatenated.alternatives.len(), 4);
    assert!(
        concatenated
            .alternatives
            .iter()
            .any(|a| Arc::ptr_eq(a, &alt1))
    );
    assert!(
        concatenated
            .alternatives
            .iter()
            .any(|a| Arc::ptr_eq(a, &alt2))
    );
    assert!(
        concatenated
            .alternatives
            .iter()
            .any(|a| Arc::ptr_eq(a, &alt3))
    );
    // tree2 itself should be added too
    assert!(
        concatenated
            .alternatives
            .iter()
            .any(|a| a.alternatives.len() == 2
                && a.alternatives.iter().any(|b| Arc::ptr_eq(b, &alt2)))
    );
}
