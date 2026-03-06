#![allow(clippy::needless_range_loop)]

use adze_runtime::{InputEdit, Point};

// ---------------------------------------------------------------------------
// Point creation and comparison
// ---------------------------------------------------------------------------

#[test]
fn point_new_creates_correct_values() {
    let p = Point::new(3, 7);
    assert_eq!(p.row, 3);
    assert_eq!(p.column, 7);
}

#[test]
fn point_origin() {
    let p = Point::new(0, 0);
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

#[test]
fn point_equality() {
    assert_eq!(Point::new(1, 2), Point::new(1, 2));
    assert_ne!(Point::new(1, 2), Point::new(1, 3));
    assert_ne!(Point::new(0, 2), Point::new(1, 2));
}

#[test]
fn point_ordering_by_row_then_column() {
    let a = Point::new(0, 5);
    let b = Point::new(1, 0);
    let c = Point::new(1, 3);
    let d = Point::new(2, 0);

    assert!(a < b);
    assert!(b < c);
    assert!(c < d);
    assert!(a < d);
}

#[test]
fn point_ordering_same_row() {
    let a = Point::new(5, 0);
    let b = Point::new(5, 10);
    assert!(a < b);
    assert!(b > a);
    assert!(a <= a);
    assert!(a >= a);
}

#[test]
fn point_clone_and_copy() {
    let p = Point::new(4, 8);
    let p2 = p; // Copy
    let p3 = p;
    assert_eq!(p, p2);
    assert_eq!(p, p3);
}

#[test]
fn point_debug_format() {
    let p = Point::new(2, 5);
    let dbg = format!("{:?}", p);
    assert!(dbg.contains("2"));
    assert!(dbg.contains("5"));
}

#[test]
fn point_display_format() {
    let p = Point::new(0, 0);
    // Display uses 1-indexed: row+1, column+1
    assert_eq!(format!("{}", p), "1:1");

    let p2 = Point::new(3, 9);
    assert_eq!(format!("{}", p2), "4:10");
}

#[test]
fn point_sorting() {
    let mut points = vec![
        Point::new(2, 3),
        Point::new(0, 1),
        Point::new(2, 0),
        Point::new(1, 5),
        Point::new(0, 0),
    ];
    points.sort();
    assert_eq!(
        points,
        vec![
            Point::new(0, 0),
            Point::new(0, 1),
            Point::new(1, 5),
            Point::new(2, 0),
            Point::new(2, 3),
        ]
    );
}

// ---------------------------------------------------------------------------
// InputEdit creation
// ---------------------------------------------------------------------------

#[test]
fn input_edit_creation_basic() {
    let edit = InputEdit {
        start_byte: 0,
        old_end_byte: 5,
        new_end_byte: 10,
        start_position: Point::new(0, 0),
        old_end_position: Point::new(0, 5),
        new_end_position: Point::new(0, 10),
    };
    assert_eq!(edit.start_byte, 0);
    assert_eq!(edit.old_end_byte, 5);
    assert_eq!(edit.new_end_byte, 10);
}

#[test]
fn input_edit_equality() {
    let a = InputEdit {
        start_byte: 1,
        old_end_byte: 3,
        new_end_byte: 5,
        start_position: Point::new(0, 1),
        old_end_position: Point::new(0, 3),
        new_end_position: Point::new(0, 5),
    };
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn input_edit_clone_and_copy() {
    let edit = InputEdit {
        start_byte: 10,
        old_end_byte: 20,
        new_end_byte: 15,
        start_position: Point::new(1, 0),
        old_end_position: Point::new(1, 10),
        new_end_position: Point::new(1, 5),
    };
    let edit2 = edit; // Copy
    let edit3 = edit;
    assert_eq!(edit, edit2);
    assert_eq!(edit, edit3);
}

#[test]
fn input_edit_insertion_repr() {
    // An insertion is where old_end_byte == start_byte (zero-length old range)
    let edit = InputEdit {
        start_byte: 5,
        old_end_byte: 5,
        new_end_byte: 12,
        start_position: Point::new(0, 5),
        old_end_position: Point::new(0, 5),
        new_end_position: Point::new(0, 12),
    };
    assert_eq!(edit.old_end_byte - edit.start_byte, 0);
    assert_eq!(edit.new_end_byte - edit.start_byte, 7);
}

#[test]
fn input_edit_deletion_repr() {
    // A deletion is where new_end_byte == start_byte (zero-length new range)
    let edit = InputEdit {
        start_byte: 3,
        old_end_byte: 10,
        new_end_byte: 3,
        start_position: Point::new(0, 3),
        old_end_position: Point::new(0, 10),
        new_end_position: Point::new(0, 3),
    };
    assert_eq!(edit.new_end_byte - edit.start_byte, 0);
    assert_eq!(edit.old_end_byte - edit.start_byte, 7);
}

#[test]
fn input_edit_replacement_repr() {
    // A replacement changes N bytes to M bytes
    let edit = InputEdit {
        start_byte: 2,
        old_end_byte: 8,
        new_end_byte: 5,
        start_position: Point::new(0, 2),
        old_end_position: Point::new(0, 8),
        new_end_position: Point::new(0, 5),
    };
    assert_eq!(edit.old_end_byte - edit.start_byte, 6); // removed 6
    assert_eq!(edit.new_end_byte - edit.start_byte, 3); // inserted 3
}

#[test]
fn input_edit_debug_format() {
    let edit = InputEdit {
        start_byte: 0,
        old_end_byte: 1,
        new_end_byte: 2,
        start_position: Point::new(0, 0),
        old_end_position: Point::new(0, 1),
        new_end_position: Point::new(0, 2),
    };
    let dbg = format!("{:?}", edit);
    assert!(dbg.contains("InputEdit"));
    assert!(dbg.contains("start_byte"));
}

#[test]
fn input_edit_multiline_positions() {
    let edit = InputEdit {
        start_byte: 20,
        old_end_byte: 30,
        new_end_byte: 45,
        start_position: Point::new(1, 5),
        old_end_position: Point::new(2, 3),
        new_end_position: Point::new(3, 10),
    };
    assert!(edit.start_position < edit.old_end_position);
    assert!(edit.old_end_position < edit.new_end_position);
}

// ---------------------------------------------------------------------------
// Tree edit operations (require incremental_glr feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "incremental_glr")]
mod tree_edit_tests {
    use super::*;
    use adze_runtime::EditError;

    fn make_tree(start: usize, end: usize, children: Vec<Tree>) -> Tree {
        Tree::new_for_testing(0, start, end, children)
    }

    fn leaf(sym: u32, start: usize, end: usize) -> Tree {
        Tree::new_for_testing(sym, start, end, vec![])
    }

    fn simple_edit(start: usize, old_end: usize, new_end: usize) -> InputEdit {
        InputEdit {
            start_byte: start,
            old_end_byte: old_end,
            new_end_byte: new_end,
            start_position: Point::new(0, start),
            old_end_position: Point::new(0, old_end),
            new_end_position: Point::new(0, new_end),
        }
    }

    // -- edit error conditions --

    #[test]
    fn edit_error_invalid_range_old_end_before_start() {
        let mut tree = make_tree(0, 10, vec![]);
        let edit = simple_edit(5, 3, 8);
        let result = tree.edit(&edit);
        assert!(matches!(
            result,
            Err(EditError::InvalidRange {
                start: 5,
                old_end: 3
            })
        ));
    }

    #[test]
    fn edit_error_invalid_range_new_end_before_start() {
        let mut tree = make_tree(0, 10, vec![]);
        let edit = simple_edit(5, 8, 3);
        let result = tree.edit(&edit);
        assert!(matches!(
            result,
            Err(EditError::InvalidRange {
                start: 5,
                old_end: 3
            })
        ));
    }

    #[test]
    fn edit_error_display_messages() {
        let e1 = EditError::InvalidRange {
            start: 10,
            old_end: 5,
        };
        assert!(format!("{}", e1).contains("Invalid edit range"));
        assert!(format!("{}", e1).contains("10"));

        let e2 = EditError::ArithmeticOverflow;
        assert!(format!("{}", e2).contains("overflow"));

        let e3 = EditError::ArithmeticUnderflow;
        assert!(format!("{}", e3).contains("underflow"));
    }

    #[test]
    fn edit_error_is_std_error() {
        let e: Box<dyn std::error::Error> = Box::new(EditError::InvalidRange {
            start: 0,
            old_end: 0,
        });
        // Just ensure it compiles and can be used as a trait object
        let _ = format!("{}", e);
    }

    // -- tree edit at beginning --

    #[test]
    fn edit_at_beginning_insertion() {
        let mut tree = make_tree(0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
        // Insert 3 bytes at position 0
        tree.edit(&simple_edit(0, 0, 3)).unwrap();
        let root = tree.root_node();
        assert_eq!(root.end_byte(), 13);
    }

    #[test]
    fn edit_at_beginning_deletion() {
        let mut tree = make_tree(0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
        // Delete first 3 bytes
        tree.edit(&simple_edit(0, 3, 0)).unwrap();
        let root = tree.root_node();
        assert_eq!(root.end_byte(), 7);
    }

    // -- tree edit at end --

    #[test]
    fn edit_at_end_insertion() {
        let mut tree = make_tree(0, 10, vec![]);
        // Insert 5 bytes right at the end boundary.
        // The edit algorithm considers the root "before" the edit
        // (end_byte <= start_byte), so the root itself is not shifted.
        tree.edit(&simple_edit(10, 10, 15)).unwrap();
        let root = tree.root_node();
        assert_eq!(root.end_byte(), 10);
    }

    // -- tree edit in middle --

    #[test]
    fn edit_in_middle_replacement() {
        let mut tree = make_tree(0, 20, vec![leaf(1, 0, 5), leaf(2, 5, 15), leaf(3, 15, 20)]);
        // Replace bytes 5..10 (5 bytes) with 8 bytes → net +3
        tree.edit(&simple_edit(5, 10, 13)).unwrap();
        let root = tree.root_node();
        assert_eq!(root.end_byte(), 23);
    }

    // -- zero-length edits --

    #[test]
    fn zero_length_edit_no_op() {
        let mut tree = make_tree(0, 10, vec![leaf(1, 0, 5), leaf(2, 5, 10)]);
        // Edit where old and new end are the same → no change
        tree.edit(&simple_edit(3, 3, 3)).unwrap();
        let root = tree.root_node();
        assert_eq!(root.start_byte(), 0);
        assert_eq!(root.end_byte(), 10);
    }

    // -- edit with various byte offsets --

    #[test]
    fn edit_shifts_nodes_after_edit_range() {
        let mut tree = make_tree(
            0,
            30,
            vec![leaf(1, 0, 10), leaf(2, 10, 20), leaf(3, 20, 30)],
        );
        // Insert 5 bytes in the first child's range
        tree.edit(&simple_edit(0, 5, 10)).unwrap();

        let root = tree.root_node();
        // Root grows by 5
        assert_eq!(root.end_byte(), 35);
        // Third child (originally 20..30) should be shifted by +5
        let third = root.child(2).unwrap();
        assert_eq!(third.start_byte(), 25);
        assert_eq!(third.end_byte(), 35);
    }

    // -- multiple sequential edits --

    #[test]
    fn multiple_sequential_edits() {
        let mut tree = make_tree(0, 20, vec![leaf(1, 0, 10), leaf(2, 10, 20)]);

        // First edit: insert 5 bytes at position 5
        tree.edit(&simple_edit(5, 5, 10)).unwrap();
        assert_eq!(tree.root_node().end_byte(), 25);

        // Second edit: delete 3 bytes at position 0..3
        tree.edit(&simple_edit(0, 3, 0)).unwrap();
        assert_eq!(tree.root_node().end_byte(), 22);
    }

    #[test]
    fn three_sequential_edits() {
        let mut tree = make_tree(0, 100, vec![]);

        // Insert 10 at beginning
        tree.edit(&simple_edit(0, 0, 10)).unwrap();
        assert_eq!(tree.root_node().end_byte(), 110);

        // Delete 20 from middle
        tree.edit(&simple_edit(50, 70, 50)).unwrap();
        assert_eq!(tree.root_node().end_byte(), 90);

        // Replace 5 bytes with 15 at end region
        tree.edit(&simple_edit(80, 85, 95)).unwrap();
        assert_eq!(tree.root_node().end_byte(), 100);
    }

    // -- edge: edit beyond tree range --

    #[test]
    fn edit_beyond_tree_end_is_no_op_on_root() {
        let mut tree = make_tree(0, 10, vec![]);
        // Edit starts after tree ends
        tree.edit(&simple_edit(20, 20, 25)).unwrap();
        // Root is entirely before the edit → unchanged
        assert_eq!(tree.root_node().start_byte(), 0);
        assert_eq!(tree.root_node().end_byte(), 10);
    }

    // -- stub tree edits --

    #[test]
    fn edit_stub_tree() {
        let mut tree = Tree::new_stub();
        // Stub tree has end_byte=0 and edit starts at 0.
        // The algorithm treats end_byte <= start_byte as "before edit",
        // so the empty root is not modified.
        tree.edit(&simple_edit(0, 0, 5)).unwrap();
        assert_eq!(tree.root_node().end_byte(), 0);
    }

    // -- large offset values --

    #[test]
    fn edit_with_large_offsets() {
        let big = 1_000_000;
        let mut tree = make_tree(0, big, vec![]);
        // Insert 500_000 bytes in the middle
        tree.edit(&simple_edit(big / 2, big / 2, big)).unwrap();
        assert_eq!(tree.root_node().end_byte(), big + big / 2);
    }
}
