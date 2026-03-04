//! Comprehensive tests for Point and Node API structures.

use adze_runtime::node::Point;

// === Point construction ===

#[test]
fn point_new() {
    let p = Point::new(0, 0);
    assert_eq!(p.row, 0);
    assert_eq!(p.column, 0);
}

#[test]
fn point_new_nonzero() {
    let p = Point::new(5, 10);
    assert_eq!(p.row, 5);
    assert_eq!(p.column, 10);
}

#[test]
fn point_fields_direct() {
    let p = Point { row: 3, column: 7 };
    assert_eq!(p.row, 3);
    assert_eq!(p.column, 7);
}

// === Point Display ===

#[test]
fn point_display_origin() {
    let p = Point::new(0, 0);
    assert_eq!(p.to_string(), "1:1");
}

#[test]
fn point_display_nonzero() {
    let p = Point::new(4, 9);
    assert_eq!(p.to_string(), "5:10");
}

#[test]
fn point_display_large() {
    let p = Point::new(999, 499);
    assert_eq!(p.to_string(), "1000:500");
}

// === Point equality ===

#[test]
fn point_eq() {
    let a = Point::new(1, 2);
    let b = Point::new(1, 2);
    assert_eq!(a, b);
}

#[test]
fn point_ne_row() {
    let a = Point::new(0, 0);
    let b = Point::new(1, 0);
    assert_ne!(a, b);
}

#[test]
fn point_ne_column() {
    let a = Point::new(0, 0);
    let b = Point::new(0, 1);
    assert_ne!(a, b);
}

// === Point Copy/Clone ===

#[test]
fn point_copy() {
    let a = Point::new(3, 4);
    let b = a; // Copy
    assert_eq!(a, b);
}

#[test]
fn point_clone() {
    let a = Point::new(3, 4);
    let b = a.clone();
    assert_eq!(a, b);
}

// === Point Debug ===

#[test]
fn point_debug() {
    let p = Point::new(0, 0);
    let d = format!("{:?}", p);
    assert!(d.contains("Point") || d.contains("row") || d.contains("0"));
}

// === Point Ord ===

#[test]
fn point_ord_same() {
    let a = Point::new(1, 1);
    let b = Point::new(1, 1);
    assert!(a == b);
}

#[test]
fn point_ord_less_row() {
    let a = Point::new(0, 5);
    let b = Point::new(1, 0);
    assert!(a < b);
}

#[test]
fn point_ord_less_col() {
    let a = Point::new(1, 0);
    let b = Point::new(1, 5);
    assert!(a < b);
}

#[test]
fn point_ord_greater() {
    let a = Point::new(5, 5);
    let b = Point::new(0, 0);
    assert!(a > b);
}

// === Point edge cases ===

#[test]
fn point_max_values() {
    let p = Point::new(usize::MAX, usize::MAX);
    assert_eq!(p.row, usize::MAX);
    assert_eq!(p.column, usize::MAX);
}

#[test]
fn point_const_new() {
    // Verify const fn works at compile time
    const P: Point = Point::new(42, 99);
    assert_eq!(P.row, 42);
    assert_eq!(P.column, 99);
}

// === Point collections ===

#[test]
fn point_vec_sort() {
    let mut points = vec![
        Point::new(2, 3),
        Point::new(0, 0),
        Point::new(1, 5),
        Point::new(1, 0),
    ];
    points.sort();
    assert_eq!(points[0], Point::new(0, 0));
    assert_eq!(points[1], Point::new(1, 0));
    assert_eq!(points[2], Point::new(1, 5));
    assert_eq!(points[3], Point::new(2, 3));
}

#[test]
fn point_btree_set() {
    use std::collections::BTreeSet;
    let mut set = BTreeSet::new();
    set.insert(Point::new(3, 3));
    set.insert(Point::new(1, 1));
    set.insert(Point::new(2, 2));
    let v: Vec<Point> = set.into_iter().collect();
    assert_eq!(v[0], Point::new(1, 1));
    assert_eq!(v[1], Point::new(2, 2));
    assert_eq!(v[2], Point::new(3, 3));
}

// === Multiple display checks ===

#[test]
fn point_display_deterministic() {
    let p = Point::new(10, 20);
    assert_eq!(p.to_string(), p.to_string());
}

#[test]
fn point_display_format_macro() {
    let p = Point::new(0, 0);
    assert_eq!(format!("{}", p), "1:1");
}
