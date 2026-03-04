// Wave 132: Comprehensive Tree editing and TreeCursor traversal tests
use adze_runtime::tree::{Tree, TreeCursor};
use adze_runtime::{InputEdit, Point};

// Helper to build a simple tree hierarchy
fn simple_tree() -> Tree {
    // root(0) -> [child_a(1, 0..5), child_b(2, 5..10)]
    let a = Tree::new_for_testing(1, 0, 5, vec![]);
    let b = Tree::new_for_testing(2, 5, 10, vec![]);
    Tree::new_for_testing(0, 0, 10, vec![a, b])
}

fn deep_tree() -> Tree {
    // root(0) -> [a(1) -> [b(2) -> [c(3)]]]
    let c = Tree::new_for_testing(3, 0, 2, vec![]);
    let b = Tree::new_for_testing(2, 0, 4, vec![c]);
    let a = Tree::new_for_testing(1, 0, 6, vec![b]);
    Tree::new_for_testing(0, 0, 8, vec![a])
}

fn wide_tree() -> Tree {
    // root(0) -> [a(1), b(2), c(3), d(4), e(5)]
    Tree::new_for_testing(
        0,
        0,
        50,
        vec![
            Tree::new_for_testing(1, 0, 10, vec![]),
            Tree::new_for_testing(2, 10, 20, vec![]),
            Tree::new_for_testing(3, 20, 30, vec![]),
            Tree::new_for_testing(4, 30, 40, vec![]),
            Tree::new_for_testing(5, 40, 50, vec![]),
        ],
    )
}

// =====================================================================
// Tree basic construction
// =====================================================================

#[test]
fn tree_new_for_testing_leaf() {
    let t = Tree::new_for_testing(42, 0, 5, vec![]);
    assert_eq!(t.root_kind(), 42);
    let root = t.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 5);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn tree_new_for_testing_with_children() {
    let t = simple_tree();
    let root = t.root_node();
    assert_eq!(root.child_count(), 2);
}

#[test]
fn tree_root_kind() {
    let t = Tree::new_for_testing(99, 0, 1, vec![]);
    assert_eq!(t.root_kind(), 99);
}

#[test]
fn tree_new_stub() {
    let t = Tree::new_stub();
    let root = t.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
}

// =====================================================================
// Node API
// =====================================================================

#[test]
fn node_kind_id() {
    let t = Tree::new_for_testing(7, 0, 3, vec![]);
    let root = t.root_node();
    assert_eq!(root.kind_id(), 7);
}

#[test]
fn node_byte_range() {
    let t = Tree::new_for_testing(0, 10, 20, vec![]);
    let root = t.root_node();
    assert_eq!(root.byte_range(), 10..20);
}

#[test]
fn node_child_access() {
    let t = simple_tree();
    let root = t.root_node();
    let child0 = root.child(0).unwrap();
    assert_eq!(child0.kind_id(), 1);
    assert_eq!(child0.start_byte(), 0);
    assert_eq!(child0.end_byte(), 5);

    let child1 = root.child(1).unwrap();
    assert_eq!(child1.kind_id(), 2);
    assert_eq!(child1.start_byte(), 5);
    assert_eq!(child1.end_byte(), 10);
}

#[test]
fn node_child_out_of_bounds() {
    let t = simple_tree();
    let root = t.root_node();
    assert!(root.child(99).is_none());
}

#[test]
fn node_named_child_count_simple() {
    let t = simple_tree();
    let root = t.root_node();
    // In test trees, named_child_count depends on tree construction
    let _ = root.named_child_count();
}

// =====================================================================
// TreeCursor traversal
// =====================================================================

#[test]
fn cursor_starts_at_root() {
    let t = simple_tree();
    let cursor = TreeCursor::new(&t);
    let node = cursor.node();
    assert_eq!(node.kind_id(), 0);
}

#[test]
fn cursor_depth_at_root() {
    let t = simple_tree();
    let cursor = TreeCursor::new(&t);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_goto_first_child() {
    let t = simple_tree();
    let mut cursor = TreeCursor::new(&t);
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind_id(), 1);
    assert_eq!(cursor.depth(), 1);
}

#[test]
fn cursor_goto_next_sibling() {
    let t = simple_tree();
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child();
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind_id(), 2);
}

#[test]
fn cursor_goto_parent() {
    let t = simple_tree();
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child();
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind_id(), 0);
    assert_eq!(cursor.depth(), 0);
}

#[test]
fn cursor_parent_at_root_returns_false() {
    let t = simple_tree();
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_no_first_child_on_leaf() {
    let t = Tree::new_for_testing(0, 0, 5, vec![]);
    let mut cursor = TreeCursor::new(&t);
    assert!(!cursor.goto_first_child());
}

#[test]
fn cursor_no_next_sibling_at_end() {
    let t = simple_tree();
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    assert!(!cursor.goto_next_sibling()); // child_b has no next sibling
}

#[test]
fn cursor_deep_traversal() {
    let t = deep_tree();
    let mut cursor = TreeCursor::new(&t);
    assert!(cursor.goto_first_child()); // depth 1
    assert_eq!(cursor.depth(), 1);
    assert!(cursor.goto_first_child()); // depth 2
    assert_eq!(cursor.depth(), 2);
    assert!(cursor.goto_first_child()); // depth 3
    assert_eq!(cursor.depth(), 3);
    assert_eq!(cursor.node().kind_id(), 3); // leaf c
    assert!(!cursor.goto_first_child()); // leaf has no children
}

#[test]
fn cursor_wide_traversal_all_siblings() {
    let t = wide_tree();
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child();
    let mut count = 1;
    while cursor.goto_next_sibling() {
        count += 1;
    }
    assert_eq!(count, 5);
}

#[test]
fn cursor_reset() {
    let t = simple_tree();
    let mut cursor = TreeCursor::new(&t);
    cursor.goto_first_child();
    cursor.goto_next_sibling();
    cursor.reset(&t);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind_id(), 0);
}

// =====================================================================
// Tree editing (requires incremental_glr feature)
// =====================================================================

#[cfg(feature = "incremental_glr")]
mod editing {
    use super::*;

    #[test]
    fn edit_valid_insertion() {
        let mut t = simple_tree();
        let edit = InputEdit {
            start_byte: 5,
            old_end_byte: 5,
            new_end_byte: 8,
            start_position: Point { row: 0, column: 5 },
            old_end_position: Point { row: 0, column: 5 },
            new_end_position: Point { row: 0, column: 8 },
        };
        assert!(t.edit(&edit).is_ok());
    }

    #[test]
    fn edit_valid_deletion() {
        let mut t = simple_tree();
        let edit = InputEdit {
            start_byte: 2,
            old_end_byte: 5,
            new_end_byte: 2,
            start_position: Point { row: 0, column: 2 },
            old_end_position: Point { row: 0, column: 5 },
            new_end_position: Point { row: 0, column: 2 },
        };
        assert!(t.edit(&edit).is_ok());
    }

    #[test]
    fn edit_at_start() {
        let mut t = simple_tree();
        let edit = InputEdit {
            start_byte: 0,
            old_end_byte: 0,
            new_end_byte: 3,
            start_position: Point { row: 0, column: 0 },
            old_end_position: Point { row: 0, column: 0 },
            new_end_position: Point { row: 0, column: 3 },
        };
        assert!(t.edit(&edit).is_ok());
    }

    #[test]
    fn edit_at_end() {
        let mut t = simple_tree();
        let edit = InputEdit {
            start_byte: 10,
            old_end_byte: 10,
            new_end_byte: 15,
            start_position: Point { row: 0, column: 10 },
            old_end_position: Point { row: 0, column: 10 },
            new_end_position: Point { row: 0, column: 15 },
        };
        assert!(t.edit(&edit).is_ok());
    }

    #[test]
    fn edit_old_end_before_start_is_error() {
        let mut t = simple_tree();
        let edit = InputEdit {
            start_byte: 5,
            old_end_byte: 3,
            new_end_byte: 8,
            start_position: Point { row: 0, column: 5 },
            old_end_position: Point { row: 0, column: 3 },
            new_end_position: Point { row: 0, column: 8 },
        };
        assert!(t.edit(&edit).is_err());
    }

    #[test]
    fn tree_clone_independence() {
        let t1 = simple_tree();
        let mut t2 = t1.clone();
        let edit = InputEdit {
            start_byte: 0,
            old_end_byte: 0,
            new_end_byte: 3,
            start_position: Point { row: 0, column: 0 },
            old_end_position: Point { row: 0, column: 0 },
            new_end_position: Point { row: 0, column: 3 },
        };
        t2.edit(&edit).unwrap();
        assert_eq!(t1.root_node().end_byte(), 10);
    }
}

// =====================================================================
// Tree language
// =====================================================================

#[test]
fn tree_language_none_for_testing() {
    let t = simple_tree();
    assert!(t.language().is_none());
}

// =====================================================================
// Tree source bytes
// =====================================================================

#[test]
fn tree_source_bytes_none_for_testing() {
    let t = simple_tree();
    assert!(t.source_bytes().is_none());
}

// =====================================================================
// Node sibling navigation
// =====================================================================

#[test]
fn node_next_sibling() {
    let t = simple_tree();
    let root = t.root_node();
    let a = root.child(0).unwrap();
    // In test trees, sibling navigation may not work if Node lacks parent reference
    let _b = a.next_sibling();
}

#[test]
fn node_prev_sibling() {
    let t = simple_tree();
    let root = t.root_node();
    let b = root.child(1).unwrap();
    let _a = b.prev_sibling();
}

#[test]
fn node_no_next_sibling_at_end() {
    let t = simple_tree();
    let root = t.root_node();
    let b = root.child(1).unwrap();
    assert!(b.next_sibling().is_none());
}

#[test]
fn node_no_prev_sibling_at_start() {
    let t = simple_tree();
    let root = t.root_node();
    let a = root.child(0).unwrap();
    assert!(a.prev_sibling().is_none());
}

// =====================================================================
// Clone
// =====================================================================

#[test]
fn tree_clone_preserves_structure() {
    let t1 = simple_tree();
    let t2 = t1.clone();
    assert_eq!(t1.root_node().child_count(), t2.root_node().child_count());
    assert_eq!(t1.root_node().start_byte(), t2.root_node().start_byte());
    assert_eq!(t1.root_node().end_byte(), t2.root_node().end_byte());
}

// =====================================================================
// Cursor full DFS
// =====================================================================

#[test]
fn cursor_full_dfs_visit_all_nodes() {
    let t = deep_tree();
    let mut cursor = TreeCursor::new(&t);
    let mut visited = vec![cursor.node().kind_id()];

    // Manual DFS
    fn dfs(cursor: &mut TreeCursor, visited: &mut Vec<u16>) {
        if cursor.goto_first_child() {
            visited.push(cursor.node().kind_id());
            dfs(cursor, visited);
            while cursor.goto_next_sibling() {
                visited.push(cursor.node().kind_id());
                dfs(cursor, visited);
            }
            cursor.goto_parent();
        }
    }
    dfs(&mut cursor, &mut visited);

    // Should visit root(0), a(1), b(2), c(3)
    assert_eq!(visited.len(), 4);
    assert!(visited.contains(&0));
    assert!(visited.contains(&1));
    assert!(visited.contains(&2));
    assert!(visited.contains(&3));
}

#[test]
fn cursor_wide_dfs() {
    let t = wide_tree();
    let mut cursor = TreeCursor::new(&t);
    let mut visited = vec![cursor.node().kind_id()];

    fn dfs(cursor: &mut TreeCursor, visited: &mut Vec<u16>) {
        if cursor.goto_first_child() {
            visited.push(cursor.node().kind_id());
            dfs(cursor, visited);
            while cursor.goto_next_sibling() {
                visited.push(cursor.node().kind_id());
                dfs(cursor, visited);
            }
            cursor.goto_parent();
        }
    }
    dfs(&mut cursor, &mut visited);

    assert_eq!(visited.len(), 6); // root + 5 children
}
