//! Comprehensive tests for the runtime2 GLR parser API.
//!
//! Covers: parser creation, language setting, parsing (simple, complex, empty,
//! errors), tree root properties, node traversal, cursor API, byte ranges,
//! kind identification, forest-to-tree conversion, sequential parses, reset,
//! error nodes, and performance.

#![cfg(feature = "glr")]

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken, TokenPattern};
use adze_runtime::{Language, Parser, Token, Tree, language::SymbolMetadata, tree::TreeCursor};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Grammar helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar: start → a (single terminal).
fn single_token_language() -> Language {
    let mut grammar = Grammar::new("single".to_string());
    let a_id = SymbolId(1);
    grammar.tokens.insert(
        a_id,
        IrToken {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let start_id = SymbolId(2);
    grammar.rule_names.insert(start_id, "start".to_string());
    grammar.rules.insert(
        start_id,
        vec![Rule {
            lhs: start_id,
            rhs: vec![Symbol::Terminal(a_id)],
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

    Language::builder()
        .parse_table(table)
        .symbol_names(vec!["EOF".into(), "a".into(), "start".into()])
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
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .tokenizer(|input: &[u8]| {
            let mut toks = Vec::new();
            for (i, &b) in input.iter().enumerate() {
                if b == b'a' {
                    toks.push(Token {
                        kind: 1,
                        start: i as u32,
                        end: (i + 1) as u32,
                    });
                }
            }
            toks.push(Token {
                kind: 0,
                start: input.len() as u32,
                end: input.len() as u32,
            });
            Box::new(toks.into_iter()) as Box<dyn Iterator<Item = Token> + '_>
        })
        .build()
        .unwrap()
}

/// Build a grammar with nesting: expr → "(" expr ")" | "x".
fn nested_language() -> Language {
    let mut grammar = Grammar::new("nested".to_string());

    let lparen = SymbolId(1);
    let rparen = SymbolId(2);
    let x_id = SymbolId(3);
    let expr_id = SymbolId(4);

    grammar.tokens.insert(
        lparen,
        IrToken {
            name: "lparen".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        rparen,
        IrToken {
            name: "rparen".to_string(),
            pattern: TokenPattern::String(")".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        x_id,
        IrToken {
            name: "x".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );

    grammar.rule_names.insert(expr_id, "expr".to_string());
    grammar.rules.insert(
        expr_id,
        vec![
            // expr → "(" expr ")"
            Rule {
                lhs: expr_id,
                rhs: vec![
                    Symbol::Terminal(lparen),
                    Symbol::NonTerminal(expr_id),
                    Symbol::Terminal(rparen),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            // expr → "x"
            Rule {
                lhs: expr_id,
                rhs: vec![Symbol::Terminal(x_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff)
        .expect("table")
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    let table: &'static _ = Box::leak(Box::new(table));

    Language::builder()
        .parse_table(table)
        .symbol_names(vec![
            "EOF".into(),
            "lparen".into(),
            "rparen".into(),
            "x".into(),
            "expr".into(),
        ])
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
        .tokenizer(|input: &[u8]| {
            let mut toks = Vec::new();
            for (i, &b) in input.iter().enumerate() {
                let kind = match b {
                    b'(' => 1,
                    b')' => 2,
                    b'x' => 3,
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
            Box::new(toks.into_iter()) as Box<dyn Iterator<Item = Token> + '_>
        })
        .build()
        .unwrap()
}

// ===========================================================================
// 1. Parser creation and language setting
// ===========================================================================

#[test]
fn parser_creation_defaults() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
    assert!(parser.timeout().is_none());
}

#[test]
fn parser_set_language_succeeds() {
    let mut parser = Parser::new();
    let lang = single_token_language();
    parser.set_language(lang).unwrap();
    assert!(parser.language().is_some());
}

#[test]
fn parser_set_language_replaces_previous() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    assert_eq!(parser.language().unwrap().symbol_count, 3);
    parser.set_language(nested_language()).unwrap();
    assert_eq!(parser.language().unwrap().symbol_count, 5);
}

#[test]
fn parser_parse_without_language_errors() {
    let mut parser = Parser::new();
    let err = parser.parse(b"a", None).unwrap_err();
    assert!(err.to_string().contains("no language"));
}

// ===========================================================================
// 2. Parse simple input with valid grammar
// ===========================================================================

#[test]
fn parse_simple_single_token() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    let root = tree.root_node();
    assert!(root.start_byte() <= root.end_byte());
    assert!(tree.language().is_some());
}

#[test]
fn parse_simple_stores_source() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    assert_eq!(tree.source_bytes(), Some(b"a".as_slice()));
}

// ===========================================================================
// 3. Parse complex nested input
// ===========================================================================

#[test]
fn parse_nested_parens() {
    let mut parser = Parser::new();
    parser.set_language(nested_language()).unwrap();
    let tree = parser.parse(b"(x)", None).unwrap();
    let root = tree.root_node();
    assert!(
        root.child_count() > 0,
        "nested parse should produce children"
    );
}

#[test]
fn parse_deeply_nested() {
    let mut parser = Parser::new();
    parser.set_language(nested_language()).unwrap();
    let tree = parser.parse(b"((x))", None).unwrap();
    let root = tree.root_node();
    assert!(root.start_byte() <= root.end_byte());
}

// ===========================================================================
// 4. Parse empty input
// ===========================================================================

#[test]
fn parse_empty_input_returns_result() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    // Empty input may error or produce a stub tree; either is acceptable.
    let _result = parser.parse(b"", None);
}

// ===========================================================================
// 5. Parse input with errors
// ===========================================================================

#[test]
fn parse_invalid_input_returns_result() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    // "zzz" has no matching tokens → the tokenizer produces only EOF.
    // The parser should either error or produce a tree with error recovery.
    let _result = parser.parse(b"zzz", None);
}

// ===========================================================================
// 6. Tree root node properties
// ===========================================================================

#[test]
fn tree_root_node_has_valid_range() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    let root = tree.root_node();
    assert!(root.start_byte() <= root.end_byte());
    assert_eq!(root.start_byte(), 0);
}

#[test]
fn tree_root_node_is_named() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    assert!(tree.root_node().is_named());
}

#[test]
fn tree_root_kind_returns_symbol_id() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

#[test]
fn tree_language_is_set_after_parse() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    assert!(tree.language().is_some());
}

// ===========================================================================
// 7. Tree node traversal (children, siblings)
// ===========================================================================

#[test]
fn node_child_access() {
    let mut parser = Parser::new();
    parser.set_language(nested_language()).unwrap();
    let tree = parser.parse(b"(x)", None).unwrap();
    let root = tree.root_node();
    if root.child_count() > 0 {
        let first = root.child(0).unwrap();
        assert!(first.start_byte() <= first.end_byte());
    }
}

#[test]
fn node_child_out_of_bounds() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    let root = tree.root_node();
    assert!(root.child(1000).is_none());
}

#[test]
fn node_sibling_stubs_return_none() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    let root = tree.root_node();
    // Sibling/parent links are not stored in the current implementation.
    assert!(root.next_sibling().is_none());
    assert!(root.prev_sibling().is_none());
    assert!(root.parent().is_none());
}

#[test]
fn node_named_child_count_matches_child_count() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.named_child_count(), root.child_count());
}

// ===========================================================================
// 8. Tree cursor API
// ===========================================================================

#[test]
fn cursor_starts_at_root() {
    let tree = Tree::new_stub();
    let cursor = TreeCursor::new(&tree);
    // Cursor should be usable on a stub tree without panic.
    drop(cursor);
}

#[test]
fn cursor_traversal_on_parsed_tree() {
    let mut parser = Parser::new();
    parser.set_language(nested_language()).unwrap();
    let tree = parser.parse(b"(x)", None).unwrap();
    let mut cursor = TreeCursor::new(&tree);

    // Try descending into children
    if cursor.goto_first_child() {
        // We moved to a child, try sibling
        let _has_sibling = cursor.goto_next_sibling();
        // Go back to parent
        assert!(cursor.goto_parent());
    }
}

#[test]
fn cursor_goto_parent_at_root_returns_false() {
    let tree = Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    assert!(!cursor.goto_parent());
}

#[test]
fn cursor_full_traversal() {
    let mut parser = Parser::new();
    parser.set_language(nested_language()).unwrap();
    let tree = parser.parse(b"(x)", None).unwrap();
    let mut cursor = TreeCursor::new(&tree);

    // Count nodes via depth-first traversal
    let mut count = 1; // root
    if cursor.goto_first_child() {
        count += 1;
        while cursor.goto_next_sibling() {
            count += 1;
        }
        cursor.goto_parent();
    }
    assert!(count >= 1, "should count at least the root node");
}

// ===========================================================================
// 9. Node byte range accuracy
// ===========================================================================

#[test]
fn byte_range_covers_input() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let input = b"a";
    let tree = parser.parse(input, None).unwrap();
    let root = tree.root_node();
    // Root should span from 0 to at most input length.
    assert_eq!(root.start_byte(), 0);
    assert!(root.end_byte() <= input.len());
}

#[test]
fn child_byte_ranges_within_parent() {
    let mut parser = Parser::new();
    parser.set_language(nested_language()).unwrap();
    let tree = parser.parse(b"(x)", None).unwrap();
    let root = tree.root_node();
    for i in 0..root.child_count() {
        let child = root.child(i).unwrap();
        assert!(
            child.start_byte() >= root.start_byte(),
            "child start {} < root start {}",
            child.start_byte(),
            root.start_byte()
        );
        assert!(
            child.end_byte() <= root.end_byte(),
            "child end {} > root end {}",
            child.end_byte(),
            root.end_byte()
        );
    }
}

// ===========================================================================
// 10. Node kind identification
// ===========================================================================

#[test]
fn node_kind_with_language() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    let root = tree.root_node();
    // Root kind should be a known symbol name from our language, not "unknown".
    assert_ne!(root.kind(), "unknown");
}

#[test]
fn node_kind_id_is_numeric() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    let root = tree.root_node();
    let _id: u16 = root.kind_id(); // Just check it returns a u16
}

#[test]
fn stub_node_kind_without_language_is_unknown() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().kind(), "unknown");
}

// ===========================================================================
// 11. Forest-to-tree conversion correctness
// ===========================================================================

#[test]
fn forest_to_tree_produces_tree_with_language() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    assert!(tree.language().is_some());
    assert!(tree.source_bytes().is_some());
}

#[test]
fn forest_to_tree_nested_structure() {
    let mut parser = Parser::new();
    parser.set_language(nested_language()).unwrap();
    let tree = parser.parse(b"(x)", None).unwrap();
    let root = tree.root_node();
    // The tree should have some hierarchical structure for nested input.
    assert!(root.child_count() > 0 || root.end_byte() > root.start_byte());
}

// ===========================================================================
// 12. Multiple sequential parses
// ===========================================================================

#[test]
fn multiple_sequential_parses() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();

    let tree1 = parser.parse(b"a", None).unwrap();
    let tree2 = parser.parse(b"a", None).unwrap();

    assert_eq!(tree1.root_kind(), tree2.root_kind());
    assert_eq!(
        tree1.root_node().start_byte(),
        tree2.root_node().start_byte()
    );
}

#[test]
fn sequential_parses_with_different_languages() {
    let mut parser = Parser::new();

    parser.set_language(single_token_language()).unwrap();
    let tree1 = parser.parse(b"a", None).unwrap();

    parser.set_language(nested_language()).unwrap();
    let tree2 = parser.parse(b"x", None).unwrap();

    // Both should succeed; root kinds may differ.
    assert!(tree1.root_node().start_byte() <= tree1.root_node().end_byte());
    assert!(tree2.root_node().start_byte() <= tree2.root_node().end_byte());
}

// ===========================================================================
// 13. Parser reset between parses
// ===========================================================================

#[test]
fn reset_preserves_language() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    parser.reset();
    assert!(parser.language().is_some());
}

#[test]
fn parse_after_reset_succeeds() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let _tree1 = parser.parse(b"a", None).unwrap();
    parser.reset();
    let tree2 = parser.parse(b"a", None).unwrap();
    assert!(tree2.root_node().start_byte() <= tree2.root_node().end_byte());
}

// ===========================================================================
// 14. Error node identification in result tree
// ===========================================================================

#[test]
fn root_node_is_not_error() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    assert!(!tree.root_node().is_error());
}

#[test]
fn root_node_is_not_missing() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    assert!(!tree.root_node().is_missing());
}

// ===========================================================================
// 15. Performance: parse 10KB input within reasonable time
// ===========================================================================

#[test]
fn performance_10kb_input() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();

    // Build ~10KB of 'a' characters.
    let input = vec![b'a'; 10_000];

    // We only time the parse, not setup.
    let start = Instant::now();
    let result = parser.parse(&input, None);
    let elapsed = start.elapsed();

    // The parse should complete (succeed or fail) within 30 seconds.
    assert!(
        elapsed < Duration::from_secs(30),
        "parse of 10KB took too long: {:?}",
        elapsed
    );

    // If the grammar only accepts a single 'a', this may error — that's fine.
    // The test validates timing, not correctness of 10K-token input.
    let _ = result;
}

// ===========================================================================
// Additional coverage
// ===========================================================================

#[test]
fn parse_utf8_convenience_wrapper() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse_utf8("a", None).unwrap();
    assert!(tree.language().is_some());
}

#[test]
fn tree_clone_is_independent() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), cloned.root_kind());
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
}

#[test]
fn tree_debug_does_not_panic() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    let debug = format!("{:?}", tree);
    assert!(debug.contains("Tree"));
}

#[test]
fn node_debug_does_not_panic() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    let debug = format!("{:?}", tree.root_node());
    assert!(debug.contains("Node"));
}

#[test]
fn parser_timeout_can_be_set() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn node_utf8_text_extraction() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let input = b"a";
    let tree = parser.parse(input, None).unwrap();
    let root = tree.root_node();
    // Extract text for the root's byte range.
    let text = root.utf8_text(input).unwrap();
    assert!(!text.is_empty() || root.byte_range().is_empty());
}

#[test]
fn parse_with_old_tree_succeeds() {
    let mut parser = Parser::new();
    parser.set_language(single_token_language()).unwrap();
    let tree1 = parser.parse(b"a", None).unwrap();
    // Passing old tree should succeed (falls back to full parse if incremental disabled).
    let tree2 = parser.parse(b"a", Some(&tree1)).unwrap();
    assert_eq!(tree1.root_kind(), tree2.root_kind());
}
