//! Comprehensive tests for the Tree, Node, and TreeCursor APIs.
//!
//! Covers: tree construction via new_stub, root node properties, node traversal
//! (child/parent/sibling), TreeCursor navigation, byte position tracking,
//! clone/deep copy independence, kind/named/visible properties, debug
//! formatting, edge cases (empty tree, single node), and node equality/comparison.

#![allow(clippy::needless_range_loop)]

#[cfg(feature = "glr")]
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
#[cfg(feature = "glr")]
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken, TokenPattern};
use adze_runtime::language::SymbolMetadata;
use adze_runtime::tree::TreeCursor;
use adze_runtime::{Language, Parser, Point, Token, Tree};

// ---------------------------------------------------------------------------
// Helper: build a simple grammar  start -> a b
// Symbols: 0=EOF, 1=a, 2=b, 3=start
// ---------------------------------------------------------------------------

#[cfg(feature = "glr")]
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

#[cfg(feature = "glr")]
fn parse_ab(input: &str) -> Tree {
    let lang = build_ab_language();
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    parser.parse_utf8(input, None).unwrap()
}

// ===========================================================================
// 1. Tree construction via new_stub
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

/// `root_node()` returns `Node` directly, not `Option<Node>`.
#[test]
fn stub_tree_root_node_returns_node_directly() {
    let tree = Tree::new_stub();
    // This compiles only because root_node() -> Node, not Option<Node>.
    let _root = tree.root_node();
    assert_eq!(_root.kind_id(), 0);
}

// ===========================================================================
// 2. Root node properties
// ===========================================================================

#[test]
fn stub_root_node_kind_is_unknown_without_language() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind(), "unknown");
}

#[test]
fn stub_root_node_kind_id_is_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind_id(), 0);
}

#[test]
fn stub_root_node_byte_range_is_empty() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.byte_range(), 0..0);
}

#[cfg(feature = "glr")]
#[test]
fn parsed_root_kind_resolves_via_language() {
    let tree = parse_ab("ab");
    assert_eq!(tree.root_node().kind(), "start");
    assert_eq!(tree.root_kind(), 3);
}

// ===========================================================================
// 3. Node traversal (child, parent, sibling)
// ===========================================================================

#[test]
fn stub_node_has_no_children() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.child_count(), 0);
    assert!(root.child(0).is_none());
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
fn node_siblings_all_return_none() {
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

#[cfg(feature = "glr")]
#[test]
fn parsed_node_has_expected_children() {
    let tree = parse_ab("ab");
    let root = tree.root_node();
    // start -> a b gives at least 2 children
    assert!(root.child_count() >= 2);

    let first = root.child(0).unwrap();
    assert_eq!(first.kind(), "a");
    assert_eq!(first.kind_id(), 1);

    let second = root.child(1).unwrap();
    assert_eq!(second.kind(), "b");
    assert_eq!(second.kind_id(), 2);
}

#[cfg(feature = "glr")]
#[test]
fn parsed_child_parent_returns_none() {
    // Parent links are not stored — child.parent() always returns None.
    let tree = parse_ab("ab");
    let child = tree.root_node().child(0).unwrap();
    assert!(child.parent().is_none());
}

// ===========================================================================
// 4. TreeCursor navigation
// ===========================================================================

#[test]
fn cursor_on_stub_tree_cannot_move() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_root_has_no_parent() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_root_has_no_sibling() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_next_sibling());
}

#[cfg(feature = "glr")]
#[test]
fn cursor_goto_first_child_and_back() {
    let tree = parse_ab("ab");
    let mut cursor = TreeCursor::new(&tree);

    assert!(cursor.goto_first_child());
    assert!(cursor.goto_parent());
    // Back at root — root has no parent
    assert!(!cursor.goto_parent());
}

#[cfg(feature = "glr")]
#[test]
fn cursor_sibling_traversal() {
    let tree = parse_ab("ab");
    let mut cursor = TreeCursor::new(&tree);

    assert!(cursor.goto_first_child());
    // At least one sibling should exist (token 'b')
    assert!(cursor.goto_next_sibling());
}

#[cfg(feature = "glr")]
#[test]
fn cursor_full_depth_first_visits_all_nodes() {
    let tree = parse_ab("ab");
    let mut cursor = TreeCursor::new(&tree);
    let mut visited = 0;

    loop {
        visited += 1;
        if cursor.goto_first_child() {
            continue;
        }
        if cursor.goto_next_sibling() {
            continue;
        }
        loop {
            if !cursor.goto_parent() {
                assert!(visited >= 3); // root + at least 2 children
                return;
            }
            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

// ===========================================================================
// 5. Byte position tracking
// ===========================================================================

#[test]
fn node_byte_range_equals_start_to_end() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.byte_range(), root.start_byte()..root.end_byte());
}

#[test]
fn node_positions_return_dummy_points() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_position(), Point::new(0, 0));
    assert_eq!(root.end_position(), Point::new(0, 0));
}

#[cfg(feature = "glr")]
#[test]
fn parsed_children_have_contiguous_byte_ranges() {
    let tree = parse_ab("ab");
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 2);

    let first = root.child(0).unwrap();
    assert_eq!(first.start_byte(), 0);
    assert_eq!(first.end_byte(), 1);

    let second = root.child(1).unwrap();
    assert_eq!(second.start_byte(), 1);
    assert_eq!(second.end_byte(), 2);
}

// ===========================================================================
// 6. Clone / deep copy independence
// ===========================================================================

#[test]
fn stub_clone_produces_equal_tree() {
    let tree = Tree::new_stub();
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), cloned.root_kind());
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
    assert_eq!(tree.root_node().end_byte(), cloned.root_node().end_byte());
}

#[cfg(feature = "glr")]
#[test]
fn parsed_clone_preserves_all_fields() {
    let tree = parse_ab("ab");
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), cloned.root_kind());
    assert_eq!(
        tree.root_node().child_count(),
        cloned.root_node().child_count()
    );
    assert_eq!(tree.root_node().end_byte(), cloned.root_node().end_byte());
}

#[cfg(feature = "glr")]
#[test]
fn cloned_tree_language_preserved() {
    let tree = parse_ab("ab");
    let cloned = tree.clone();
    // Language should be cloned — kind() should still resolve.
    assert_eq!(cloned.root_node().kind(), "start");
    assert!(cloned.language().is_some());
}

#[test]
fn node_is_copy_and_clone() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let copied = root; // Copy
    #[allow(clippy::clone_on_copy)]
    let cloned = root.clone();
    assert_eq!(root.kind_id(), copied.kind_id());
    assert_eq!(root.kind_id(), cloned.kind_id());
    assert_eq!(root.byte_range(), copied.byte_range());
    assert_eq!(root.byte_range(), cloned.byte_range());
}

// ===========================================================================
// 7. Kind / named / visible properties
// ===========================================================================

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
fn named_child_count_equals_child_count_on_stub() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[cfg(feature = "glr")]
#[test]
fn named_child_count_equals_child_count_on_parsed() {
    let tree = parse_ab("ab");
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

#[cfg(feature = "glr")]
#[test]
fn named_child_matches_child_for_all_indices() {
    let tree = parse_ab("ab");
    let root = tree.root_node();
    for i in 0..root.child_count() {
        let c = root.child(i);
        let n = root.named_child(i);
        assert_eq!(c.is_some(), n.is_some());
        if let (Some(c), Some(n)) = (c, n) {
            assert_eq!(c.kind_id(), n.kind_id());
            assert_eq!(c.byte_range(), n.byte_range());
        }
    }
}

// ===========================================================================
// 8. Debug formatting
// ===========================================================================

#[test]
fn tree_debug_contains_tree_keyword() {
    let tree = Tree::new_stub();
    let debug = format!("{tree:?}");
    assert!(debug.contains("Tree"));
}

#[test]
fn node_debug_contains_kind_and_range() {
    let tree = Tree::new_stub();
    let debug = format!("{:?}", tree.root_node());
    assert!(debug.contains("Node"));
    assert!(debug.contains("kind"));
    assert!(debug.contains("range"));
}

#[test]
fn point_display_is_one_indexed() {
    let p = Point::new(2, 7);
    assert_eq!(format!("{p}"), "3:8");
}

// ===========================================================================
// 9. Edge cases (empty tree, single node, utf8 text)
// ===========================================================================

#[test]
fn utf8_text_on_empty_stub() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    // range 0..0 extracts empty slice from empty source
    assert_eq!(root.utf8_text(b"").unwrap(), "");
}

#[cfg(feature = "glr")]
#[test]
fn utf8_text_on_parsed_children() {
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

#[test]
fn multiple_stubs_are_identical() {
    let a = Tree::new_stub();
    let b = Tree::new_stub();
    assert_eq!(a.root_kind(), b.root_kind());
    assert_eq!(a.root_node().kind(), b.root_node().kind());
    assert_eq!(a.root_node().byte_range(), b.root_node().byte_range());
}

// ===========================================================================
// 10. Node equality and comparison
// ===========================================================================

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
fn point_new_and_fields() {
    let p = Point::new(5, 10);
    assert_eq!(p.row, 5);
    assert_eq!(p.column, 10);
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

#[test]
fn node_copy_preserves_all_properties() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    let copy = root;
    assert_eq!(root.kind(), copy.kind());
    assert_eq!(root.kind_id(), copy.kind_id());
    assert_eq!(root.start_byte(), copy.start_byte());
    assert_eq!(root.end_byte(), copy.end_byte());
    assert_eq!(root.byte_range(), copy.byte_range());
    assert_eq!(root.child_count(), copy.child_count());
    assert_eq!(root.is_named(), copy.is_named());
    assert_eq!(root.is_missing(), copy.is_missing());
    assert_eq!(root.is_error(), copy.is_error());
}

// ===========================================================================
// Language query helpers via Node
// ===========================================================================

#[cfg(feature = "glr")]
#[test]
fn language_symbol_name_lookup() {
    let lang = build_ab_language();
    assert_eq!(lang.symbol_name(0), Some("EOF"));
    assert_eq!(lang.symbol_name(1), Some("a"));
    assert_eq!(lang.symbol_name(2), Some("b"));
    assert_eq!(lang.symbol_name(3), Some("start"));
    assert_eq!(lang.symbol_name(99), None);
}

#[cfg(feature = "glr")]
#[test]
fn language_is_terminal_query() {
    let lang = build_ab_language();
    assert!(lang.is_terminal(0)); // EOF
    assert!(lang.is_terminal(1)); // a
    assert!(lang.is_terminal(2)); // b
    assert!(!lang.is_terminal(3)); // start is non-terminal
}

#[cfg(feature = "glr")]
#[test]
fn language_is_visible_query() {
    let lang = build_ab_language();
    assert!(!lang.is_visible(0)); // EOF not visible
    assert!(lang.is_visible(1)); // a visible
    assert!(lang.is_visible(2)); // b visible
    assert!(lang.is_visible(3)); // start visible
}
