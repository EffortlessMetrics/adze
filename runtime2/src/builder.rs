//! Convert an engine forest into the public Tree facade.
//!
//! This module provides conversion functions that transform GLR parse forests
//! (which may contain multiple parse trees representing different interpretations)
//! into a single concrete Tree-sitter compatible tree structure.

use crate::engine::Forest;
use crate::tree::{Tree, TreeNode};

#[cfg(feature = "glr")]
use adze_glr_core::ForestView as CoreForestView;
#[cfg(feature = "glr-core")]
use rustc_hash::FxHashSet;

/// Converts a GLR parse forest into a Tree-sitter compatible tree.
///
/// This function takes a parse forest (which may contain multiple parse trees
/// representing different valid interpretations of ambiguous input) and converts
/// it into a single concrete tree structure compatible with the Tree-sitter API.
///
/// # How It Works
///
/// 1. **Forest Selection**: Extracts the forest view and identifies root nodes
/// 2. **Disambiguation**: Selects the first root when multiple interpretations exist
/// 3. **Tree Construction**: Recursively builds a tree by selecting the "best"
///    children at each ambiguous node using `best_children()`
/// 4. **Performance Tracking**: Optionally logs conversion metrics when
///    `ADZE_LOG_PERFORMANCE` environment variable is set
///
/// # Arguments
///
/// * `forest` - The parse forest to convert (either `Forest::Glr` or `Forest::Stub`)
///
/// # Returns
///
/// Returns a `Tree` containing:
/// - A concrete parse tree when the forest is non-empty
/// - A stub tree when the forest is empty or GLR features are disabled
///
/// # Disambiguation Strategy
///
/// When multiple parse trees exist in the forest (due to ambiguous grammar),
/// this function currently uses a simple "first choice" strategy:
/// - Takes the first root from available roots
/// - At each node, selects `best_children()` (currently the first alternative)
///
/// Future versions may support:
/// - Custom disambiguation strategies
/// - Access to all parse alternatives
/// - Probability-weighted tree selection
///
/// # Performance Monitoring
///
/// Set the `ADZE_LOG_PERFORMANCE` environment variable to enable detailed
/// performance logging:
///
/// ```bash
/// ADZE_LOG_PERFORMANCE=1 cargo run
/// ```
///
/// This will log:
/// - Total node count in the resulting tree
/// - Maximum tree depth
/// - Conversion time in milliseconds
///
/// # Example
///
/// ```ignore
/// use runtime2::engine::parse_full;
/// use runtime2::builder::forest_to_tree;
///
/// // Parse input and get forest
/// let forest = parse_full(&language, b"1 + 2 * 3")?;
///
/// // Convert to concrete tree
/// let tree = forest_to_tree(forest);
///
/// // Now use standard Tree-sitter API
/// let root = tree.root_node();
/// println!("Root kind: {}", root.kind());
/// ```
///
/// # Feature Gates
///
/// - Requires `glr-core` feature for full functionality
/// - Without `glr-core`, returns a stub tree
#[cfg(feature = "glr")]
pub fn forest_to_tree(forest: Forest) -> Tree {
    match forest {
        Forest::Glr(core) => build_from_glr(core),
    }
}

/// Stub version of `forest_to_tree()` when GLR features are disabled.
///
/// This function exists to maintain API compatibility when the `glr-core`
/// feature is not enabled. It always returns a stub tree.
#[cfg(not(feature = "glr"))]
pub fn forest_to_tree(_forest: Forest) -> Tree {
    // Should not be called without GLR support, but return stub for completeness
    Tree::new_stub()
}

/// Internal function to build a Tree from a GLR-core forest with performance tracking.
///
/// This function handles the actual conversion from GLR forest to Tree-sitter tree,
/// including performance instrumentation when enabled via environment variable.
///
/// # Process
///
/// 1. Creates a forest view to access parse forest data
/// 2. Retrieves all root nodes (multiple roots indicate parse ambiguity)
/// 3. Selects the first root for tree construction
/// 4. Recursively builds the tree with performance metrics tracking
/// 5. Logs metrics if `ADZE_LOG_PERFORMANCE` is set
///
/// # Performance Metrics
///
/// Tracks and optionally reports:
/// - **Node Count**: Total number of nodes in the resulting tree
/// - **Max Depth**: Maximum depth from root to any leaf
/// - **Conversion Time**: Wall-clock time for the entire conversion
#[cfg(feature = "glr")]
fn build_from_glr(core: adze_glr_core::Forest) -> Tree {
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
    if std::env::var("ADZE_LOG_PERFORMANCE").is_ok() {
        eprintln!(
            "🚀 Forest->Tree conversion: {} nodes, depth {}, took {:?}",
            node_count, max_depth, conversion_time
        );
    }

    Tree::new(root_node)
}

/// Iterative tree builder with performance metrics tracking.
///
/// This function builds a `TreeNode` tree from a forest view while tracking
/// performance metrics like node count and maximum depth. It uses an explicit
/// traversal stack so deeply nested user input cannot overflow the Rust call
/// stack during forest-to-tree conversion.
///
/// # Arguments
///
/// * `view` - Forest view providing access to node data
/// * `id` - Node identifier in the forest
/// * `depth` - Current depth in the tree (0 for root)
/// * `node_count` - Mutable reference to total node counter
/// * `max_depth` - Mutable reference to maximum depth tracker
///
/// # Returns
///
/// A `TreeNode` with all descendants, having updated the metrics counters.
///
/// # Disambiguation
///
/// At each node, this function calls `view.best_children(id)` to select among
/// multiple parse alternatives. The "best" strategy currently selects the first
/// alternative, but could be enhanced to use heuristics like:
/// - Preference for longer matches
/// - Grammar rule priorities
/// - Probability-based selection
#[cfg(feature = "glr")]
fn build_node_with_metrics(
    view: &dyn CoreForestView,
    id: u32,
    depth: usize,
    node_count: &mut usize,
    max_depth: &mut usize,
) -> TreeNode {
    #[derive(Debug, Clone, Copy)]
    struct PendingNode {
        id: u32,
        depth: usize,
        expanded: bool,
    }

    let mut pending = vec![PendingNode {
        id,
        depth,
        expanded: false,
    }];
    let mut active = FxHashSet::default();
    let mut built = Vec::new();

    while let Some(node) = pending.pop() {
        if !node.expanded {
            *node_count += 1;
            *max_depth = (*max_depth).max(node.depth);

            if !active.insert(node.id) {
                let span = view.span(node.id);
                let kind = view.kind(node.id);
                built.push(TreeNode::new_with_children(
                    kind,
                    span.start as usize,
                    span.end as usize,
                    Vec::new(),
                ));
                continue;
            }

            pending.push(PendingNode {
                id: node.id,
                depth: node.depth,
                expanded: true,
            });

            for &child_id in view.best_children(node.id).iter().rev() {
                pending.push(PendingNode {
                    id: child_id,
                    depth: node.depth + 1,
                    expanded: false,
                });
            }

            continue;
        }

        active.remove(&node.id);

        let span = view.span(node.id);
        let kind = view.kind(node.id);
        let child_count = view.best_children(node.id).len();
        let split_at = built.len() - child_count;
        let children = built.split_off(split_at);
        built.push(TreeNode::new_with_children(
            kind,
            span.start as usize,
            span.end as usize,
            children,
        ));
    }

    built
        .pop()
        .expect("forest-to-tree conversion should build a root node")
}
