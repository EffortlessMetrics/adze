//! Convert an engine forest into the public Tree facade.

use crate::tree::{Tree, TreeNode};
use crate::engine::Forest;

#[cfg(feature = "glr-core")]
use rust_sitter_glr_core::ForestView as CoreForestView;

pub fn forest_to_tree(forest: Forest) -> Tree {
    match forest {
        #[cfg(feature = "glr-core")]
        Forest::Glr(core) => build_from_glr(core),
        _ => Tree::new_stub(),
    }
}

#[cfg(feature = "glr-core")]
fn build_from_glr(core: rust_sitter_glr_core::Forest) -> Tree {
    let view = core.view();
    let roots = view.roots();
    
    if roots.is_empty() {
        return Tree::new_stub();
    }
    
    // Take the first root for now (could handle ambiguity later)
    let root_id = roots[0];
    let root_node = build_node(view, root_id);
    Tree::new(root_node)
}

#[cfg(feature = "glr-core")]
fn build_node(view: &dyn CoreForestView, id: u32) -> TreeNode {
    let span = view.span(id);
    let kind = view.kind(id);
    let kids = view.best_children(id)
        .iter()
        .copied()
        .map(|c| build_node(view, c))
        .collect();
    TreeNode::new_with_children(kind, span.start as usize, span.end as usize, kids)
}