//! Builder (forest-to-tree conversion) tests.
//!
//! Tests the forest_to_tree path using the GLR engine with a proper
//! grammar-derived parse table.

#![cfg(feature = "glr")]

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken, TokenPattern};
use adze_runtime::{Language, Parser, Token, Tree, language::SymbolMetadata};

/// Build a minimal grammar: start → a
fn make_language() -> Language {
    let mut grammar = Grammar::new("test".to_string());
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
            if !input.is_empty() {
                toks.push(Token {
                    kind: 1,
                    start: 0,
                    end: 1,
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

/// Build a recursive grammar: start -> seq, seq -> a | a seq
fn make_deep_chain_language() -> Language {
    let mut grammar = Grammar::new("deep_chain".to_string());
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
    let seq_id = SymbolId(3);
    grammar.rule_names.insert(start_id, "start".to_string());
    grammar.rule_names.insert(seq_id, "seq".to_string());
    grammar.rules.insert(
        start_id,
        vec![Rule {
            lhs: start_id,
            rhs: vec![Symbol::NonTerminal(seq_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    grammar.rules.insert(
        seq_id,
        vec![
            Rule {
                lhs: seq_id,
                rhs: vec![Symbol::Terminal(a_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
            Rule {
                lhs: seq_id,
                rhs: vec![Symbol::Terminal(a_id), Symbol::NonTerminal(seq_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
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
        .symbol_names(vec!["EOF".into(), "a".into(), "start".into(), "seq".into()])
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
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .tokenizer(|input: &[u8]| {
            let mut toks = Vec::with_capacity(input.len() + 1);
            for (idx, byte) in input.iter().enumerate() {
                if *byte == b'a' {
                    toks.push(Token {
                        kind: 1,
                        start: idx as u32,
                        end: (idx + 1) as u32,
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

#[test]
fn forest_to_tree_produces_valid_root() {
    let lang = make_language();
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    let tree = parser.parse(b"a", None).unwrap();
    let root = tree.root_node();
    assert!(root.start_byte() <= root.end_byte());
}

#[test]
fn forest_to_tree_sets_language_on_tree() {
    let lang = make_language();
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    let tree = parser.parse(b"a", None).unwrap();
    assert!(tree.language().is_some());
}

#[test]
fn forest_to_tree_stores_source_bytes() {
    let lang = make_language();
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    let tree = parser.parse(b"a", None).unwrap();
    assert_eq!(tree.source_bytes(), Some(b"a".as_slice()));
}

#[test]
fn stub_tree_root_has_zero_range() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_node().start_byte(), 0);
    assert_eq!(tree.root_node().end_byte(), 0);
    assert_eq!(tree.root_node().child_count(), 0);
}

#[test]
fn parsed_tree_clone_is_independent() {
    let lang = make_language();
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    let tree = parser.parse(b"a", None).unwrap();
    let cloned = tree.clone();

    assert_eq!(tree.root_kind(), cloned.root_kind());
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
}

/// When incremental feature is disabled, parse_incremental falls back to full parse.
#[test]
fn parse_with_old_tree_succeeds() {
    let lang = make_language();
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    let tree1 = parser.parse(b"a", None).unwrap();
    // Passing old_tree should work (falls back to full parse if incremental disabled)
    let tree2 = parser.parse(b"a", Some(&tree1)).unwrap();
    assert_eq!(tree1.root_kind(), tree2.root_kind());
}

#[test]
fn forest_to_tree_handles_deep_right_recursive_input() {
    let lang = make_deep_chain_language();
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    let input = vec![b'a'; 20_000];
    let tree = parser.parse(&input, None).unwrap();

    assert_eq!(tree.source_bytes(), Some(input.as_slice()));
    assert!(tree.root_node().child_count() > 0);
    assert_eq!(tree.root_node().end_byte(), input.len());
}
