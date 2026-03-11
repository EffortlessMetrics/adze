use adze::subtree::{
    ChildEdge as RuntimeChildEdge, FIELD_NONE as RUNTIME_FIELD_NONE, Subtree as RuntimeSubtree,
    SubtreeNode as RuntimeSubtreeNode,
};
use adze_ir::SymbolId;
use adze_subtree_core::{
    ChildEdge as CoreChildEdge, FIELD_NONE as CORE_FIELD_NONE, Subtree as CoreSubtree,
    SubtreeNode as CoreSubtreeNode,
};
use std::sync::Arc;

fn runtime_leaf(sym: u16) -> Arc<RuntimeSubtree> {
    Arc::new(RuntimeSubtree::new(
        RuntimeSubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: false,
            byte_range: 0..1,
        },
        vec![],
    ))
}

fn core_leaf(sym: u16) -> Arc<CoreSubtree> {
    Arc::new(CoreSubtree::new(
        CoreSubtreeNode {
            symbol_id: SymbolId(sym),
            is_error: false,
            byte_range: 0..1,
        },
        vec![],
    ))
}

#[test]
fn runtime_reexport_matches_microcrate_behavior() {
    let runtime = RuntimeSubtree::new(
        RuntimeSubtreeNode {
            symbol_id: SymbolId(1),
            is_error: false,
            byte_range: 0..2,
        },
        vec![runtime_leaf(2), runtime_leaf(3)],
    );

    let core = CoreSubtree::new(
        CoreSubtreeNode {
            symbol_id: SymbolId(1),
            is_error: false,
            byte_range: 0..2,
        },
        vec![core_leaf(2), core_leaf(3)],
    );

    assert_eq!(runtime.symbol(), core.symbol());
    assert_eq!(runtime.byte_range(), core.byte_range());
    assert_eq!(runtime.dynamic_prec, core.dynamic_prec);
}

#[test]
fn runtime_reexport_is_type_compatible() {
    fn accepts_core_type(value: CoreSubtree) -> CoreSubtree {
        value
    }

    let runtime_value = RuntimeSubtree::new(
        RuntimeSubtreeNode {
            symbol_id: SymbolId(7),
            is_error: false,
            byte_range: 5..8,
        },
        vec![],
    );

    let core_value = accepts_core_type(runtime_value);
    assert_eq!(core_value.symbol(), 7);
}

#[test]
fn runtime_child_edge_and_field_none_match_core() {
    let runtime_edge = RuntimeChildEdge::new_without_field(runtime_leaf(4));
    let core_edge = CoreChildEdge::new_without_field(core_leaf(4));

    assert_eq!(RUNTIME_FIELD_NONE, CORE_FIELD_NONE);
    assert_eq!(runtime_edge.field_id, core_edge.field_id);
    assert_eq!(runtime_edge.field_id, RUNTIME_FIELD_NONE);
}
