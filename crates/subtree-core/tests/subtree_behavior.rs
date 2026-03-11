use adze_ir::SymbolId;
use adze_subtree_core::{ChildEdge, FIELD_NONE, Subtree, SubtreeNode};
use std::sync::Arc;

fn leaf(symbol: u16, prec: i32) -> Arc<Subtree> {
    Arc::new(Subtree {
        node: SubtreeNode {
            symbol_id: SymbolId(symbol),
            is_error: false,
            byte_range: 0..1,
        },
        dynamic_prec: prec,
        children: vec![],
        alternatives: Default::default(),
    })
}

#[test]
fn constructor_propagates_max_child_prec() {
    let left = leaf(1, 2);
    let right = leaf(2, 9);

    let tree = Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(3),
            is_error: false,
            byte_range: 0..2,
        },
        vec![left, right],
    );

    assert_eq!(tree.dynamic_prec, 9);
}

#[test]
fn push_alt_deduplicates_by_pointer() {
    let alt = leaf(8, 7);

    let base = Subtree::new(
        SubtreeNode {
            symbol_id: SymbolId(1),
            is_error: false,
            byte_range: 0..1,
        },
        vec![],
    )
    .push_alt(alt.clone())
    .push_alt(alt);

    assert_eq!(base.alternatives.len(), 1);
    assert_eq!(base.dynamic_prec, 7);
}

#[test]
fn child_edge_without_field_sets_field_none() {
    let edge = ChildEdge::new_without_field(leaf(1, 0));
    assert_eq!(edge.field_id, FIELD_NONE);
}
