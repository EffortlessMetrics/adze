//! Convert an engine forest into the public Tree facade.

use crate::engine::Forest;
use crate::tree::{Tree, TreeNode};

#[cfg(feature = "glr-core")]
use rust_sitter_glr_core::ForestView as CoreForestView;

#[cfg(feature = "glr-core")]
pub fn forest_to_tree(forest: Forest) -> Tree {
    match forest {
        Forest::Glr(core) => build_from_glr(core),
    }
}

#[cfg(not(feature = "glr-core"))]
pub fn forest_to_tree(_forest: Forest) -> Tree {
    // Should not be called without GLR support, but return stub for completeness
    Tree::new_stub()
}

#[cfg(feature = "glr-core")]
fn build_from_glr(core: rust_sitter_glr_core::Forest) -> Tree {
    use std::time::Instant;

    let start_time = Instant::now();
    let view = core.view();
    let roots = view.roots();

    if roots.is_empty() {
        return Tree::new_stub();
    }

    // Performance metrics
    let mut node_count = 0;
    let mut max_depth = 0;

    // Take the first root for now (could handle ambiguity later)
    let root_id = roots[0];
    let root_node = build_node_with_metrics(view, root_id, 0, &mut node_count, &mut max_depth);

    let conversion_time = start_time.elapsed();

    // Log performance metrics (can be enabled via environment variable)
    if std::env::var("RUST_SITTER_LOG_PERFORMANCE").is_ok() {
        eprintln!(
            "🚀 Forest->Tree conversion: {} nodes, depth {}, took {:?}",
            node_count, max_depth, conversion_time
        );
    }

    Tree::new(root_node)
}

#[cfg(feature = "glr-core")]
#[allow(dead_code)]
fn build_node(view: &dyn CoreForestView, id: u32) -> TreeNode {
    let span = view.span(id);
    let kind = view.kind(id);
    let kids = view
        .best_children(id)
        .iter()
        .copied()
        .map(|c| build_node(view, c))
        .collect();
    TreeNode::new_with_children(kind, span.start as usize, span.end as usize, kids)
}

#[cfg(feature = "glr-core")]
fn build_node_with_metrics(
    view: &dyn CoreForestView,
    id: u32,
    depth: usize,
    node_count: &mut usize,
    max_depth: &mut usize,
) -> TreeNode {
    *node_count += 1;
    *max_depth = (*max_depth).max(depth);

    let span = view.span(id);
    let kind = view.kind(id);
    let kids = view
        .best_children(id)
        .iter()
        .copied()
        .map(|c| build_node_with_metrics(view, c, depth + 1, node_count, max_depth))
        .collect();
    TreeNode::new_with_children(kind, span.start as usize, span.end as usize, kids)
}
