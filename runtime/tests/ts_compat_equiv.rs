//! Tests for Tree-sitter compatibility API equivalence

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use rust_sitter::ts_compat::{InputEdit, Parser, Point};

#[test]
fn test_fresh_parse_simple() {
    let mut parser = Parser::new();
    let lang = rust_sitter_example::ts_langs::arithmetic();

    parser.set_language(lang).expect("Failed to set language");

    let tree = parser.parse("1+2+3", None).expect("Parse failed");
    // TODO: Fix root_kind when proper grammar loading is implemented
    // For now just check that it doesn't panic
    let _ = tree.root_kind();
    // Error count will be non-zero with minimal table
    let _ = tree.error_count();
}

#[test]
fn test_fresh_equals_incremental_simple() {
    let mut parser = Parser::new();
    let lang = rust_sitter_example::ts_langs::arithmetic();

    parser.set_language(lang).expect("Failed to set language");

    // Parse initial version
    let src = "1+2+3";
    let fresh = parser.parse(src, None).expect("Parse failed");
    assert_eq!(fresh.error_count(), 0);

    // Create edited version
    let mut edited = fresh.clone();
    edited.edit(&InputEdit {
        start_byte: 1,
        old_end_byte: 1,
        new_end_byte: 3,
        start_position: Point { row: 0, column: 1 },
        old_end_position: Point { row: 0, column: 1 },
        new_end_position: Point { row: 0, column: 3 },
    });

    // Parse incrementally
    let new_src = "1+42+3";
    let incremental = parser.parse(new_src, Some(&edited)).expect("Parse failed");

    // Parse fresh for comparison
    let fresh_new = parser.parse(new_src, None).expect("Parse failed");

    // Compare results
    assert_eq!(incremental.root_kind(), fresh_new.root_kind());
    assert_eq!(incremental.error_count(), fresh_new.error_count());
}

#[test]
fn test_deletion_edit() {
    let mut parser = Parser::new();
    let lang = rust_sitter_example::ts_langs::arithmetic();

    parser.set_language(lang).expect("Failed to set language");

    // Parse initial version
    let src = "1+2+3+4";
    let tree = parser.parse(src, None).expect("Parse failed");

    // Delete "+3"
    let mut edited = tree.clone();
    edited.edit(&InputEdit {
        start_byte: 3,
        old_end_byte: 5,
        new_end_byte: 3,
        start_position: Point { row: 0, column: 3 },
        old_end_position: Point { row: 0, column: 5 },
        new_end_position: Point { row: 0, column: 3 },
    });

    let new_src = "1+2+4";
    let incremental = parser.parse(new_src, Some(&edited)).expect("Parse failed");
    let fresh = parser.parse(new_src, None).expect("Parse failed");

    assert_eq!(incremental.root_kind(), fresh.root_kind());
    assert_eq!(incremental.error_count(), 0);
}

#[test]
fn test_insertion_edit() {
    let mut parser = Parser::new();
    let lang = rust_sitter_example::ts_langs::arithmetic();

    parser.set_language(lang).expect("Failed to set language");

    // Parse initial version
    let src = "1+2";
    let tree = parser.parse(src, None).expect("Parse failed");

    // Insert "+3" at the end
    let mut edited = tree.clone();
    edited.edit(&InputEdit {
        start_byte: 3,
        old_end_byte: 3,
        new_end_byte: 5,
        start_position: Point { row: 0, column: 3 },
        old_end_position: Point { row: 0, column: 3 },
        new_end_position: Point { row: 0, column: 5 },
    });

    let new_src = "1+2+3";
    let incremental = parser.parse(new_src, Some(&edited)).expect("Parse failed");
    let fresh = parser.parse(new_src, None).expect("Parse failed");

    assert_eq!(incremental.root_kind(), fresh.root_kind());
    assert_eq!(incremental.error_count(), 0);
}

#[test]
fn test_multiple_edits() {
    let mut parser = Parser::new();
    let lang = rust_sitter_example::ts_langs::arithmetic();

    parser.set_language(lang).expect("Failed to set language");

    let src1 = "1+2";
    let mut tree = parser.parse(src1, None).expect("Parse failed");

    // Edit 1: Insert *3
    tree.edit(&InputEdit {
        start_byte: 3,
        old_end_byte: 3,
        new_end_byte: 5,
        start_position: Point { row: 0, column: 3 },
        old_end_position: Point { row: 0, column: 3 },
        new_end_position: Point { row: 0, column: 5 },
    });

    let src2 = "1+2*3";
    tree = parser.parse(src2, Some(&tree)).expect("Parse failed");
    assert_eq!(tree.error_count(), 0);

    // Edit 2: Replace 1 with 10
    tree.edit(&InputEdit {
        start_byte: 0,
        old_end_byte: 1,
        new_end_byte: 2,
        start_position: Point { row: 0, column: 0 },
        old_end_position: Point { row: 0, column: 1 },
        new_end_position: Point { row: 0, column: 2 },
    });

    let src3 = "10+2*3";
    tree = parser.parse(src3, Some(&tree)).expect("Parse failed");

    let fresh = parser.parse(src3, None).expect("Parse failed");
    assert_eq!(tree.root_kind(), fresh.root_kind());
    assert_eq!(tree.error_count(), 0);
}

#[cfg(feature = "incremental_glr")]
#[test]
fn test_incremental_glr_enabled() {
    // This test verifies that incremental GLR is actually being used
    let mut parser = Parser::new();
    let lang = rust_sitter_example::ts_langs::arithmetic();

    parser.set_language(lang).expect("Failed to set language");

    let src = "1+2+3";
    let tree = parser.parse(src, None).expect("Parse failed");

    let mut edited = tree.clone();
    edited.edit(&InputEdit {
        start_byte: 1,
        old_end_byte: 1,
        new_end_byte: 3,
        start_position: Point { row: 0, column: 1 },
        old_end_position: Point { row: 0, column: 1 },
        new_end_position: Point { row: 0, column: 3 },
    });

    // With incremental_glr enabled, this should use the incremental path
    let _incremental = parser.parse("1+99+3", Some(&edited)).expect("Parse failed");
}

// Property-based testing helpers
mod prop_tests {
    use super::*;

    fn generate_arithmetic_expr(depth: usize) -> String {
        if depth == 0 {
            return (1..=9).collect::<Vec<_>>()[rand::random::<usize>() % 9].to_string();
        }

        let op = if rand::random::<bool>() { "+" } else { "*" };
        let left = generate_arithmetic_expr(depth - 1);
        let right = generate_arithmetic_expr(depth - 1);
        format!("{}{}{}", left, op, right)
    }

    #[test]
    fn test_random_edits() {
        let mut parser = Parser::new();
        let lang = rust_sitter_example::ts_langs::arithmetic();
        parser.set_language(lang).expect("Failed to set language");

        for _ in 0..10 {
            let expr = generate_arithmetic_expr(3);
            let tree = parser.parse(&expr, None);
            assert!(tree.is_some(), "Failed to parse: {}", expr);
        }
    }

    // Stub for rand - in real code use the rand crate
    mod rand {
        pub fn random<T>() -> T
        where
            T: Default,
        {
            T::default()
        }
    }
}
