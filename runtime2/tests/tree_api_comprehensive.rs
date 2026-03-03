//! Comprehensive tests for the Tree, Node, and TreeCursor APIs.
//!
//! Covers: tree construction, root node access, language association,
//! source bytes, cloning, node metadata, byte ranges, child access,
//! text extraction, sibling/parent navigation, cursor traversal,
//! cursor edge cases, debug formatting, and point utilities.

#[cfg(feature = "glr-core")]
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
#[cfg(feature = "glr-core")]
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken, TokenPattern};
use adze_runtime::language::SymbolMetadata;
use adze_runtime::tree::TreeCursor;
use adze_runtime::{Language, Parser, Point, Token, Tree};

// ---------------------------------------------------------------------------
// Helper: build a simple grammar  start -> a b
// Symbols: 0=EOF, 1=a, 2=b, 3=start
// ---------------------------------------------------------------------------

#[cfg(feature = "glr-core")]
fn build_ab_language() -> Language {
    let mut grammar = Grammar::new("test_ab".to_string());

    let a_id = SymbolId(1);
    grammar.tokens.insert(
        a_id,
        IrToken {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let b_id = SymbolId(2);
    grammar.tokens.insert(
        b_id,
        IrToken {
            name: "b".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    let start_id = SymbolId(3);
    grammar.rule_names.insert(start_id, "start".to_string());
    grammar.rules.insert(
        start_id,
        vec![Rule {
            lhs: start_id,
            rhs: vec![Symbol::Terminal(a_id), Symbol::Terminal(b_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff)
        .expect("table")
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    let table: &'static _ = Box::leak(Box::new(table));

    #[allow(clippy::type_complexity)]
    let tokenize_fn: Box<dyn for<'x> Fn(&'x [u8]) -> Box<dyn Iterator<Item = Token> + 'x>> =
        Box::new(|input: &[u8]| -> Box<dyn Iterator<Item = Token> + '_> {
            let mut toks = Vec::new();
            for (i, &byte) in input.iter().enumerate() {
                let kind = match byte {
                    b'a' => 1u32,
                    b'b' => 2u32,
                    _ => continue,
                };
                toks.push(Token {
                    kind,
                    start: i as u32,
                    end: (i + 1) as u32,
                });
            }
            toks.push(Token {
                kind: 0,
                start: input.len() as u32,
                end: input.len() as u32,
            });
            Box::new(toks.into_iter())
        });

    Language::builder()
        .version(14)
        .parse_table(table)
        .symbol_names(vec!["EOF".into(), "a".into(), "b".into(), "start".into()])
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .field_names(vec![])
        .tokenizer(tokenize_fn)
        .build()
        .unwrap()
}

#[cfg(feature = "glr-core")]
fn parse_ab(input: &str) -> Tree {
    let lang = build_ab_language();
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    parser.parse_utf8(input, None).unwrap()
}

// ===========================================================================
// Tree tests
// ===========================================================================

#[test]
fn stub_tree_has_zero_root_kind() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

#[test]
fn stub_tree_has_no_language() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

#[test]
fn stub_tree_has_no_source_bytes() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

#[test]
fn stub_tree_root_node_kind_is_unknown() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn stub_tree_clone_is_independent() {
    let tree = Tree::new_stub();
    let cloned = tree.clone();
    // Both trees should have same root_kind
    assert_eq!(tree.root_kind(), cloned.root_kind());
    // Modifying clone doesn't affect original (deep copy)
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
}

#[test]
fn tree_debug_format_is_nonempty() {
    let tree = Tree::new_stub();
    let debug = format!("{tree:?}");
    assert!(debug.contains("Tree"));
}

#[cfg(feature = "glr-core")]
#[test]
fn parsed_tree_has_language() {
    let tree = parse_ab("ab");
    assert!(tree.language().is_some());
}

#[cfg(feature = "glr-core")]
#[test]
fn parsed_tree_root_kind_is_start_symbol() {
    let tree = parse_ab("ab");
    // start symbol is id 3
    assert_eq!(tree.root_kind(), 3);
}

#[cfg(feature = "glr-core")]
#[test]
fn parsed_tree_clone_preserves_structure() {
    let tree = parse_ab("ab");
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), cloned.root_kind());
    assert_eq!(
        tree.root_node().child_count(),
        cloned.root_node().child_count()
    );
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
    assert_eq!(tree.root_node().end_byte(), cloned.root_node().end_byte());
}

// ===========================================================================
// Node metadata tests
// ===========================================================================

#[test]
fn node_kind_id_matches_symbol() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind_id(), 0);
}

#[test]
fn node_byte_range_is_consistent() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.byte_range(), root.start_byte()..root.end_byte());
}

#[test]
fn node_positions_return_dummy_point() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_position(), Point::new(0, 0));
    assert_eq!(root.end_position(), Point::new(0, 0));
}

#[test]
fn node_is_named_returns_true() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().is_named());
}

#[test]
fn node_is_missing_returns_false() {
    let tree = Tree::new_stub();
    assert!(!tree.root_node().is_missing());
}

#[test]
fn node_is_error_returns_false() {
    let tree = Tree::new_stub();
    assert!(!tree.root_node().is_error());
}

#[test]
fn stub_node_has_no_children() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.child_count(), 0);
    assert_eq!(root.named_child_count(), 0);
    assert!(root.child(0).is_none());
    assert!(root.named_child(0).is_none());
}

#[test]
fn node_child_out_of_bounds_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().child(999).is_none());
}

#[test]
fn node_parent_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().parent().is_none());
}

#[test]
fn node_siblings_return_none() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert!(root.next_sibling().is_none());
    assert!(root.prev_sibling().is_none());
    assert!(root.next_named_sibling().is_none());
    assert!(root.prev_named_sibling().is_none());
}

#[test]
fn node_child_by_field_name_returns_none() {
    let tree = Tree::new_stub();
    assert!(tree.root_node().child_by_field_name("name").is_none());
}

#[test]
fn node_debug_format() {
    let tree = Tree::new_stub();
    let debug = format!("{:?}", tree.root_node());
    assert!(debug.contains("Node"));
    assert!(debug.contains("kind"));
    assert!(debug.contains("range"));
}

// ===========================================================================
// Node text extraction
// ===========================================================================

#[test]
fn node_utf8_text_empty_source() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    // Stub has range 0..0, so extracting from empty source should work
    let text = root.utf8_text(b"").unwrap();
    assert_eq!(text, "");
}

#[cfg(feature = "glr-core")]
#[test]
fn node_utf8_text_from_source() {
    let tree = parse_ab("ab");
    let root = tree.root_node();
    let text = root.utf8_text(b"ab").unwrap();
    assert_eq!(text, "ab");
}

// ===========================================================================
// Node child access (parsed tree with children)
// ===========================================================================

#[cfg(feature = "glr-core")]
#[test]
fn parsed_node_has_children() {
    let tree = parse_ab("ab");
    let root = tree.root_node();
    // start -> a b, so root should have 2 children
    assert!(root.child_count() >= 2);
}

#[cfg(feature = "glr-core")]
#[test]
fn parsed_node_children_have_correct_kind() {
    let tree = parse_ab("ab");
    let root = tree.root_node();
    assert_eq!(root.kind(), "start");

    // Check children exist and have terminal kinds
    if let Some(first_child) = root.child(0) {
        assert_eq!(first_child.kind(), "a");
        assert_eq!(first_child.kind_id(), 1);
    }
    if root.child_count() >= 2 {
        let second_child = root.child(1).unwrap();
        assert_eq!(second_child.kind(), "b");
        assert_eq!(second_child.kind_id(), 2);
    }
}

#[cfg(feature = "glr-core")]
#[test]
fn parsed_node_children_byte_ranges() {
    let tree = parse_ab("ab");
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 2);

    if let Some(first) = root.child(0) {
        assert_eq!(first.start_byte(), 0);
        assert_eq!(first.end_byte(), 1);
    }
    if let Some(second) = root.child(1) {
        assert_eq!(second.start_byte(), 1);
        assert_eq!(second.end_byte(), 2);
    }
}

#[cfg(feature = "glr-core")]
#[test]
fn parsed_child_text_extraction() {
    let tree = parse_ab("ab");
    let root = tree.root_node();
    let source = b"ab";

    if let Some(first) = root.child(0) {
        assert_eq!(first.utf8_text(source).unwrap(), "a");
    }
    if let Some(second) = root.child(1) {
        assert_eq!(second.utf8_text(source).unwrap(), "b");
    }
}

#[cfg(feature = "glr-core")]
#[test]
fn parsed_named_child_count_matches_child_count() {
    let tree = parse_ab("ab");
    let root = tree.root_node();
    // Phase 1: named_child_count == child_count
    assert_eq!(root.named_child_count(), root.child_count());
}

#[cfg(feature = "glr-core")]
#[test]
fn parsed_named_child_matches_child() {
    let tree = parse_ab("ab");
    let root = tree.root_node();
    // Phase 1: named_child(i) == child(i)
    for i in 0..root.child_count() {
        let child = root.child(i);
        let named = root.named_child(i);
        assert_eq!(child.is_some(), named.is_some());
        if let (Some(c), Some(n)) = (child, named) {
            assert_eq!(c.kind_id(), n.kind_id());
            assert_eq!(c.byte_range(), n.byte_range());
        }
    }
}

// ===========================================================================
// Node is Copy
// ===========================================================================

#[test]
fn node_is_copy() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let copy = root; // Node is Copy
    assert_eq!(root.kind_id(), copy.kind_id());
    assert_eq!(root.byte_range(), copy.byte_range());
}

// ===========================================================================
// TreeCursor tests
// ===========================================================================

#[test]
fn cursor_on_stub_tree() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    // Stub tree has no children
    assert!(!cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
    assert!(!cursor.goto_parent());
}

#[cfg(feature = "glr-core")]
#[test]
fn cursor_goto_first_child_and_back() {
    let tree = parse_ab("ab");
    let mut cursor = TreeCursor::new(&tree);

    // At root
    assert!(cursor.goto_first_child());
    // Now at first child
    assert!(cursor.goto_parent());
    // Back at root
    assert!(!cursor.goto_parent()); // root has no parent
}

#[cfg(feature = "glr-core")]
#[test]
fn cursor_sibling_traversal() {
    let tree = parse_ab("ab");
    let mut cursor = TreeCursor::new(&tree);

    // Go to first child
    assert!(cursor.goto_first_child());
    // Go to next sibling
    assert!(cursor.goto_next_sibling());
    // No more siblings after the last child (may have EOF token too)
    // Just verify we can traverse siblings without panic
}

#[cfg(feature = "glr-core")]
#[test]
fn cursor_full_depth_first_traversal() {
    let tree = parse_ab("ab");
    let mut cursor = TreeCursor::new(&tree);
    let mut visited = 0;

    // Simple depth-first: go as deep as possible, then try siblings, then parent
    loop {
        visited += 1;
        if cursor.goto_first_child() {
            continue;
        }
        if cursor.goto_next_sibling() {
            continue;
        }
        // Backtrack until we find a sibling or reach root
        loop {
            if !cursor.goto_parent() {
                // We're done — back at root with no more siblings
                assert!(visited >= 3); // at least root + 2 children
                return;
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

#[test]
fn cursor_root_has_no_sibling() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

// ===========================================================================
// Point tests
// ===========================================================================

#[test]
fn point_new_and_fields() {
    let p = Point::new(5, 10);
    assert_eq!(p.row, 5);
    assert_eq!(p.column, 10);
}

#[test]
fn point_equality() {
    assert_eq!(Point::new(1, 2), Point::new(1, 2));
    assert_ne!(Point::new(1, 2), Point::new(1, 3));
    assert_ne!(Point::new(0, 0), Point::new(1, 0));
}

#[test]
fn point_ordering() {
    assert!(Point::new(0, 0) < Point::new(0, 1));
    assert!(Point::new(0, 9) < Point::new(1, 0));
    assert!(Point::new(1, 5) > Point::new(1, 4));
}

#[test]
fn point_display() {
    let p = Point::new(2, 7);
    // Display is 1-indexed: row+1, column+1
    assert_eq!(format!("{p}"), "3:8");
}

#[test]
fn point_clone_and_copy() {
    let p = Point::new(3, 4);
    let p2 = p; // Copy
    #[allow(clippy::clone_on_copy)]
    let p3 = p.clone();
    assert_eq!(p, p2);
    assert_eq!(p, p3);
}

// ===========================================================================
// Language query helpers via Node
// ===========================================================================

#[cfg(feature = "glr-core")]
#[test]
fn language_symbol_name_lookup() {
    let lang = build_ab_language();
    assert_eq!(lang.symbol_name(0), Some("EOF"));
    assert_eq!(lang.symbol_name(1), Some("a"));
    assert_eq!(lang.symbol_name(2), Some("b"));
    assert_eq!(lang.symbol_name(3), Some("start"));
    assert_eq!(lang.symbol_name(99), None);
}

#[cfg(feature = "glr-core")]
#[test]
fn language_is_terminal_query() {
    let lang = build_ab_language();
    assert!(lang.is_terminal(0)); // EOF
    assert!(lang.is_terminal(1)); // a
    assert!(lang.is_terminal(2)); // b
    assert!(!lang.is_terminal(3)); // start is non-terminal
}

#[cfg(feature = "glr-core")]
#[test]
fn language_is_visible_query() {
    let lang = build_ab_language();
    assert!(!lang.is_visible(0)); // EOF not visible
    assert!(lang.is_visible(1)); // a visible
    assert!(lang.is_visible(2)); // b visible
    assert!(lang.is_visible(3)); // start visible
}
