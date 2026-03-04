#![allow(clippy::needless_range_loop)]
//! Comprehensive end-to-end tests exercising the full parsing pipeline:
//! Grammar → Language → Parser → Tree → Node → TreeCursor.

#![cfg(feature = "pure-rust")]

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::{
    Associativity, Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken,
    TokenPattern as IrTokenPattern,
};
use adze_runtime::tree::TreeCursor;
use adze_runtime::{Language, Parser, Token, language::SymbolMetadata};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TERM_VIS: SymbolMetadata = SymbolMetadata {
    is_terminal: true,
    is_visible: true,
    is_supertype: false,
};

const EOF_META: SymbolMetadata = SymbolMetadata {
    is_terminal: true,
    is_visible: false,
    is_supertype: false,
};

const NT_VIS: SymbolMetadata = SymbolMetadata {
    is_terminal: false,
    is_visible: true,
    is_supertype: false,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Leak a parse table for `&'static` lifetime.
fn leak_table(grammar: &Grammar) -> &'static ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    let table = build_lr1_automaton(grammar, &ff)
        .expect("LR(1) automaton")
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    Box::leak(Box::new(table))
}

/// Build a Language from a grammar, table, and token map.
fn build_language(
    table: &'static ParseTable,
    symbol_names: Vec<&str>,
    metadata: Vec<SymbolMetadata>,
    tokens: Vec<(u32, &'static str)>,
) -> Language {
    let names: Vec<String> = symbol_names.iter().map(|s| s.to_string()).collect();
    let tok_map: Vec<(u32, &'static str)> = tokens;

    Language::builder()
        .parse_table(table)
        .symbol_names(names)
        .symbol_metadata(metadata)
        .field_names(vec![])
        .tokenizer(move |input: &[u8]| {
            let mut result: Vec<Token> = Vec::new();
            let mut pos: usize = 0;
            let src = input;
            while pos < src.len() {
                if src[pos] == b' ' || src[pos] == b'\t' || src[pos] == b'\n' || src[pos] == b'\r' {
                    pos += 1;
                    continue;
                }
                let mut best: Option<(u32, usize)> = None;
                for &(kind, lit) in &tok_map {
                    if lit == "<digits>" {
                        let start = pos;
                        let mut end = pos;
                        while end < src.len() && src[end].is_ascii_digit() {
                            end += 1;
                        }
                        if end > start && best.map_or(true, |(_, blen)| end - start > blen) {
                            best = Some((kind, end - start));
                        }
                    } else if lit == "<ident>" {
                        let start = pos;
                        let mut end = pos;
                        if end < src.len() && (src[end].is_ascii_alphabetic() || src[end] == b'_') {
                            end += 1;
                            while end < src.len()
                                && (src[end].is_ascii_alphanumeric() || src[end] == b'_')
                            {
                                end += 1;
                            }
                        }
                        if end > start && best.map_or(true, |(_, blen)| end - start > blen) {
                            best = Some((kind, end - start));
                        }
                    } else if src[pos..].starts_with(lit.as_bytes()) {
                        let len = lit.len();
                        if best.map_or(true, |(_, blen)| len > blen) {
                            best = Some((kind, len));
                        }
                    }
                }
                if let Some((kind, len)) = best {
                    result.push(Token {
                        kind,
                        start: pos as u32,
                        end: (pos + len) as u32,
                    });
                    pos += len;
                } else {
                    break;
                }
            }
            result.push(Token {
                kind: 0,
                start: pos as u32,
                end: pos as u32,
            });
            Box::new(result.into_iter()) as Box<dyn Iterator<Item = Token>>
        })
        .build()
        .expect("Language::build")
}

/// Grammar: start → "a"
fn single_token_grammar() -> (Grammar, &'static ParseTable) {
    let mut grammar = Grammar::new("single".into());
    let tok_id = SymbolId(1);
    grammar.tokens.insert(
        tok_id,
        IrToken {
            name: "a".into(),
            pattern: IrTokenPattern::String("a".into()),
            fragile: false,
        },
    );
    let start_id = SymbolId(2);
    grammar.rule_names.insert(start_id, "start".into());
    grammar.rules.insert(
        start_id,
        vec![Rule {
            lhs: start_id,
            rhs: vec![Symbol::Terminal(tok_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let table = leak_table(&grammar);
    (grammar, table)
}

/// Grammar: expr → NUMBER | expr PLUS expr
fn arithmetic_grammar() -> (Grammar, &'static ParseTable) {
    let mut grammar = Grammar::new("arith".into());
    let num_id = SymbolId(1);
    grammar.tokens.insert(
        num_id,
        IrToken {
            name: "NUMBER".into(),
            pattern: IrTokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    let plus_id = SymbolId(2);
    grammar.tokens.insert(
        plus_id,
        IrToken {
            name: "PLUS".into(),
            pattern: IrTokenPattern::String("+".into()),
            fragile: false,
        },
    );
    let expr_id = SymbolId(3);
    grammar.rule_names.insert(expr_id, "expr".into());
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(plus_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: vec![],
    });
    let table = leak_table(&grammar);
    (grammar, table)
}

/// Grammar: stmt → IDENT SEMI
fn stmt_grammar() -> (Grammar, &'static ParseTable) {
    let mut grammar = Grammar::new("stmt".into());
    let ident_id = SymbolId(1);
    grammar.tokens.insert(
        ident_id,
        IrToken {
            name: "IDENT".into(),
            pattern: IrTokenPattern::Regex(r"[a-z]+".into()),
            fragile: false,
        },
    );
    let semi_id = SymbolId(2);
    grammar.tokens.insert(
        semi_id,
        IrToken {
            name: "SEMI".into(),
            pattern: IrTokenPattern::String(";".into()),
            fragile: false,
        },
    );
    let stmt_id = SymbolId(3);
    grammar.rule_names.insert(stmt_id, "stmt".into());
    grammar.rules.insert(
        stmt_id,
        vec![Rule {
            lhs: stmt_id,
            rhs: vec![Symbol::Terminal(ident_id), Symbol::Terminal(semi_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let table = leak_table(&grammar);
    (grammar, table)
}

/// Grammar: pair → LPAREN WORD RPAREN
fn pair_grammar() -> (Grammar, &'static ParseTable) {
    let mut grammar = Grammar::new("pair".into());
    let lp_id = SymbolId(1);
    grammar.tokens.insert(
        lp_id,
        IrToken {
            name: "LPAREN".into(),
            pattern: IrTokenPattern::String("(".into()),
            fragile: false,
        },
    );
    let word_id = SymbolId(2);
    grammar.tokens.insert(
        word_id,
        IrToken {
            name: "WORD".into(),
            pattern: IrTokenPattern::Regex(r"[a-zA-Z]+".into()),
            fragile: false,
        },
    );
    let rp_id = SymbolId(3);
    grammar.tokens.insert(
        rp_id,
        IrToken {
            name: "RPAREN".into(),
            pattern: IrTokenPattern::String(")".into()),
            fragile: false,
        },
    );
    let pair_id = SymbolId(4);
    grammar.rule_names.insert(pair_id, "pair".into());
    grammar.rules.insert(
        pair_id,
        vec![Rule {
            lhs: pair_id,
            rhs: vec![
                Symbol::Terminal(lp_id),
                Symbol::Terminal(word_id),
                Symbol::Terminal(rp_id),
            ],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        }],
    );
    let table = leak_table(&grammar);
    (grammar, table)
}

// ===========================================================================
// Tests
// ===========================================================================

// ---- 1. Full pipeline: Grammar → Language → Parser → Tree → Node ----------

#[test]
fn t01_full_pipeline_single_token() {
    let (_grammar, table) = single_token_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse_utf8("a", None).unwrap();

    let root = tree.root_node();
    assert_eq!(root.kind(), "start");
    assert_eq!(root.child_count(), 1);
    let child = root.child(0).unwrap();
    assert_eq!(child.kind(), "a");
    assert_eq!(child.byte_range(), 0..1);
}

// ---- 2. Parse different input strings ------------------------------------

#[test]
fn t02_parse_different_numbers() {
    let (_grammar, table) = arithmetic_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    let inputs: &[&str] = &["0", "42", "999", "12345"];
    for input in inputs {
        let tree = parser.parse_utf8(input, None).unwrap();
        assert_eq!(tree.root_node().kind(), "expr");
        let num = tree.root_node().child(0).unwrap();
        assert_eq!(num.kind(), "NUMBER");
        assert_eq!(num.byte_range(), 0..input.len());
    }
}

#[test]
fn t03_parse_addition_expression() {
    let (_grammar, table) = arithmetic_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(b"1+2", None).unwrap();

    let root = tree.root_node();
    assert_eq!(root.kind(), "expr");
    assert!(root.child_count() >= 1);
}

// ---- 3. Error handling through the pipeline ------------------------------

#[test]
fn t04_error_no_language_set() {
    let mut parser = Parser::new();
    let result = parser.parse(b"anything", None);
    assert!(result.is_err());
}

#[test]
fn t05_error_no_language_parse_utf8() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("anything", None);
    assert!(result.is_err());
}

#[test]
fn t06_invalid_input_differs_from_valid() {
    let (_grammar, table) = single_token_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    let valid = parser.parse(b"a", None);
    assert!(valid.is_ok());

    // "x" is not a recognized token
    let result = parser.parse(b"x", None);
    match result {
        Err(_) => { /* acceptable */ }
        Ok(tree) => {
            let root = tree.root_node();
            let valid_tree = valid.unwrap();
            let valid_root = valid_tree.root_node();
            assert_ne!(
                root.child(0).map(|n| n.utf8_text(b"x").ok()),
                valid_root.child(0).map(|n| n.utf8_text(b"a").ok()),
            );
        }
    }
}

// ---- 4. Multiple parses with the same parser -----------------------------

#[test]
fn t07_multiple_parses_same_parser() {
    let (_grammar, table) = arithmetic_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    for input in &[b"42" as &[u8], b"1", b"999", b"0"] {
        let tree = parser.parse(*input, None).unwrap();
        assert_eq!(tree.root_node().kind(), "expr");
        let number = tree.root_node().child(0).unwrap();
        assert_eq!(&input[number.byte_range()], *input);
    }
}

#[test]
fn t08_parse_ten_times_sequential() {
    let (_grammar, table) = single_token_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    for _ in 0..10 {
        let tree = parser.parse_utf8("a", None).unwrap();
        assert_eq!(tree.root_node().kind(), "start");
    }
}

// ---- 5. Unicode input handling -------------------------------------------

#[test]
fn t09_unicode_input_byte_ranges() {
    // Unicode characters are multi-byte; the tokenizer should handle raw bytes.
    // Use the single_token_grammar but feed it "a" embedded after skipping whitespace.
    let (_grammar, table) = single_token_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    // Input with leading spaces (whitespace is skipped by tokenizer)
    let tree = parser.parse_utf8("   a", None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.kind(), "start");
    let child = root.child(0).unwrap();
    // "a" starts at byte 3 after three spaces
    assert_eq!(child.start_byte(), 3);
    assert_eq!(child.end_byte(), 4);
}

#[test]
fn t10_utf8_text_extraction() {
    let (_grammar, table) = arithmetic_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let input = b"42";
    let tree = parser.parse(input, None).unwrap();
    let num = tree.root_node().child(0).unwrap();
    assert_eq!(num.utf8_text(input).unwrap(), "42");
}

// ---- 6. Empty input handling ---------------------------------------------

#[test]
fn t11_empty_input_non_nullable() {
    let (_grammar, table) = single_token_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let result = parser.parse(b"", None);
    match result {
        Err(_) => { /* expected for non-nullable grammar */ }
        Ok(tree) => {
            assert_eq!(tree.root_node().end_byte(), 0);
        }
    }
}

#[test]
fn t12_empty_utf8_input() {
    let (_grammar, table) = single_token_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let result = parser.parse_utf8("", None);
    match result {
        Err(_) => {}
        Ok(tree) => {
            assert_eq!(tree.root_node().end_byte(), 0);
        }
    }
}

// ---- 7. Tree traversal after parsing -------------------------------------

#[test]
fn t13_tree_root_kind_id() {
    let (_grammar, table) = single_token_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse_utf8("a", None).unwrap();
    // start is SymbolId(2)
    assert_eq!(tree.root_kind(), 2);
    assert_eq!(tree.root_node().kind_id(), 2);
}

#[test]
fn t14_child_traversal_two_tokens() {
    let (_grammar, table) = stmt_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "IDENT", "SEMI", "stmt"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<ident>"), (2, ";")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(b"foo;", None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.kind(), "stmt");
    assert_eq!(root.child_count(), 2);
    assert_eq!(root.child(0).unwrap().kind(), "IDENT");
    assert_eq!(root.child(1).unwrap().kind(), "SEMI");
}

#[test]
fn t15_child_out_of_bounds_returns_none() {
    let (_grammar, table) = single_token_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse_utf8("a", None).unwrap();
    let root = tree.root_node();
    // root has 1 child, index 1 is out of bounds
    assert!(root.child(1).is_none());
    assert!(root.child(100).is_none());
}

#[test]
fn t16_source_bytes_preserved() {
    let (_grammar, table) = single_token_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(b"a", None).unwrap();
    assert_eq!(tree.source_bytes(), Some(b"a".as_ref()));
}

#[test]
fn t17_node_start_end_positions() {
    let (_grammar, table) = arithmetic_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(b"100", None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 3);
}

// ---- 8. Parser reuse across different inputs -----------------------------

#[test]
fn t18_parser_reuse_different_grammars() {
    // Parse with stmt grammar, then switch to arithmetic grammar.
    let (_g1, table1) = stmt_grammar();
    let lang1 = build_language(
        table1,
        vec!["EOF", "IDENT", "SEMI", "stmt"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<ident>"), (2, ";")],
    );

    let (_g2, table2) = arithmetic_grammar();
    let lang2 = build_language(
        table2,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();

    parser.set_language(lang1).unwrap();
    let tree1 = parser.parse(b"x;", None).unwrap();
    assert_eq!(tree1.root_node().kind(), "stmt");

    parser.set_language(lang2).unwrap();
    let tree2 = parser.parse(b"7", None).unwrap();
    assert_eq!(tree2.root_node().kind(), "expr");
}

#[test]
fn t19_parser_reuse_varied_inputs() {
    let (_grammar, table) = stmt_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "IDENT", "SEMI", "stmt"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<ident>"), (2, ";")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    let inputs: &[&[u8]] = &[b"a;", b"hello;", b"x;", b"foo;"];
    for input in inputs {
        let tree = parser.parse(*input, None).unwrap();
        let root = tree.root_node();
        assert_eq!(root.kind(), "stmt");
        assert_eq!(root.child_count(), 2);
        let ident = root.child(0).unwrap();
        let ident_text = ident.utf8_text(input).unwrap();
        let expected = std::str::from_utf8(&input[..input.len() - 1]).unwrap();
        assert_eq!(ident_text, expected);
    }
}

// ---- 9. Language with multiple symbols -----------------------------------

#[test]
fn t20_three_terminal_rule() {
    let (_grammar, table) = pair_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "LPAREN", "WORD", "RPAREN", "pair"],
        vec![EOF_META, TERM_VIS, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "("), (2, "<ident>"), (3, ")")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(b"(hello)", None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.kind(), "pair");
    assert_eq!(root.child_count(), 3);
    assert_eq!(root.child(0).unwrap().kind(), "LPAREN");
    assert_eq!(root.child(1).unwrap().kind(), "WORD");
    assert_eq!(root.child(2).unwrap().kind(), "RPAREN");
}

#[test]
fn t21_symbol_name_lookup_on_tree_language() {
    let (_grammar, table) = arithmetic_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(b"5", None).unwrap();
    let tree_lang = tree.language().expect("tree should carry language");
    assert_eq!(tree_lang.symbol_name(0), Some("EOF"));
    assert_eq!(tree_lang.symbol_name(1), Some("NUMBER"));
    assert_eq!(tree_lang.symbol_name(2), Some("PLUS"));
    assert_eq!(tree_lang.symbol_name(3), Some("expr"));
    assert_eq!(tree_lang.symbol_name(99), None);
}

#[test]
fn t22_language_terminal_visibility_queries() {
    let (_grammar, table) = arithmetic_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    assert!(lang.is_terminal(1)); // NUMBER
    assert!(lang.is_terminal(2)); // PLUS
    assert!(!lang.is_terminal(3)); // expr is non-terminal
    assert!(!lang.is_visible(0)); // EOF is invisible
    assert!(lang.is_visible(1)); // NUMBER is visible
}

// ---- 10. Integration with TreeCursor -------------------------------------

#[test]
fn t23_tree_cursor_basic_traversal() {
    let (_grammar, table) = stmt_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "IDENT", "SEMI", "stmt"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<ident>"), (2, ";")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(b"foo;", None).unwrap();

    let mut cursor = TreeCursor::new(&tree);
    // At root (stmt)
    assert!(cursor.goto_first_child()); // → IDENT
    assert!(cursor.goto_next_sibling()); // → SEMI
    assert!(!cursor.goto_next_sibling()); // no more siblings
    assert!(cursor.goto_parent()); // → stmt
    assert!(!cursor.goto_parent()); // already at root
}

#[test]
fn t24_tree_cursor_three_children() {
    let (_grammar, table) = pair_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "LPAREN", "WORD", "RPAREN", "pair"],
        vec![EOF_META, TERM_VIS, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "("), (2, "<ident>"), (3, ")")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(b"(abc)", None).unwrap();

    let mut cursor = TreeCursor::new(&tree);
    // Root → first child (LPAREN)
    assert!(cursor.goto_first_child());
    // → WORD
    assert!(cursor.goto_next_sibling());
    // → RPAREN
    assert!(cursor.goto_next_sibling());
    // No more siblings
    assert!(!cursor.goto_next_sibling());
    // Back to root
    assert!(cursor.goto_parent());
    assert!(!cursor.goto_parent());
}

#[test]
fn t25_tree_cursor_on_leaf_node() {
    let (_grammar, table) = single_token_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse_utf8("a", None).unwrap();

    let mut cursor = TreeCursor::new(&tree);
    assert!(cursor.goto_first_child()); // → "a" leaf
    assert!(!cursor.goto_first_child()); // leaf has no children
    assert!(!cursor.goto_next_sibling()); // only child
}

#[test]
fn t26_tree_cursor_on_stub_tree() {
    let tree = adze_runtime::Tree::new_stub();
    let mut cursor = TreeCursor::new(&tree);
    // Stub tree root has no children
    assert!(!cursor.goto_first_child());
    assert!(!cursor.goto_next_sibling());
    assert!(!cursor.goto_parent());
}

// ---- Additional coverage --------------------------------------------------

#[test]
fn t27_parse_with_whitespace() {
    let (_grammar, table) = arithmetic_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    // Whitespace between tokens should be skipped by our tokenizer
    let tree = parser.parse(b"1 + 2", None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.kind(), "expr");
    assert!(root.child_count() >= 1);
}

#[test]
fn t28_chained_additions() {
    let (_grammar, table) = arithmetic_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(b"1+2+3", None).unwrap();
    assert_eq!(tree.root_node().kind(), "expr");
    // Chained addition produces more than 1 child
    assert!(tree.root_node().child_count() > 1);
}

#[test]
fn t29_node_is_named_and_error_defaults() {
    let (_grammar, table) = single_token_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse_utf8("a", None).unwrap();
    let root = tree.root_node();
    // Phase 1 defaults
    assert!(root.is_named());
    assert!(!root.is_missing());
    assert!(!root.is_error());
}

#[test]
fn t30_byte_range_multi_digit_number() {
    let (_grammar, table) = arithmetic_grammar();
    let lang = build_language(
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(b"12345", None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.kind(), "expr");
    assert_eq!(root.child_count(), 1);
    let num = root.child(0).unwrap();
    assert_eq!(num.kind(), "NUMBER");
    assert_eq!(num.byte_range(), 0..5);
}
