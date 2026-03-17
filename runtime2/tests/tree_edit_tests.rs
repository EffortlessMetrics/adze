//! Tests for Tree editing and InputEdit functionality.

#[cfg(feature = "incremental_glr")]
mod edit_tests {
    use adze_runtime::tree::{EditError, Tree};
    use adze_runtime::{InputEdit, Point};

    #[test]
    fn edit_stub_tree() {
        let mut tree = Tree::new_stub();
        let edit = InputEdit {
            start_byte: 0,
            old_end_byte: 0,
            new_end_byte: 5,
            start_position: Point { row: 0, column: 0 },
            old_end_position: Point { row: 0, column: 0 },
            new_end_position: Point { row: 0, column: 5 },
        };
        // Stub tree edit may succeed or fail depending on implementation
        let _result = tree.edit(&edit);
    }

    #[test]
    fn edit_error_display() {
        let err = EditError::InvalidRange {
            start: 10,
            old_end: 5,
        };
        let msg = format!("{err}");
        assert!(!msg.is_empty());
    }

    #[test]
    fn edit_error_debug() {
        let err = EditError::InvalidRange {
            start: 10,
            old_end: 5,
        };
        let dbg = format!("{err:?}");
        assert!(dbg.contains("InvalidRange"));
    }
}

// Tests that don't need incremental_glr
mod basic_tree_tests {
    use adze_runtime::tree::Tree;

    #[test]
    fn stub_tree_root_kind() {
        let tree = Tree::new_stub();
        let _kind = tree.root_kind();
    }

    #[test]
    fn stub_tree_has_no_language() {
        let tree = Tree::new_stub();
        assert!(tree.language().is_none());
    }

    #[test]
    fn stub_tree_has_no_source() {
        let tree = Tree::new_stub();
        assert!(tree.source_bytes().is_none());
    }

    #[test]
    fn stub_tree_root_node_exists() {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        let _kind = root.kind_id();
    }

    #[test]
    fn stub_tree_debug() {
        let tree = Tree::new_stub();
        let debug = format!("{tree:?}");
        assert!(!debug.is_empty());
    }

    #[test]
    fn stub_tree_clone() {
        let tree = Tree::new_stub();
        let cloned = tree.clone();
        assert_eq!(tree.root_kind(), cloned.root_kind());
    }
}
