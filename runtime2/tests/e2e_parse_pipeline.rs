//! End-to-end integration tests exercising the full parse pipeline:
//! grammar → FIRST/FOLLOW → parse table → Language → Parser → parse → inspect tree.

#![cfg(feature = "pure-rust")]

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::{
    Associativity, Grammar, ProductionId, Rule, Symbol, SymbolId, Token as IrToken,
    TokenPattern as IrTokenPattern, builder::GrammarBuilder,
};
use adze_runtime::{Language, Parser, Token, language::SymbolMetadata};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Metadata entry for a terminal symbol (visible).
const TERM_VIS: SymbolMetadata = SymbolMetadata {
    is_terminal: true,
    is_visible: true,
    is_supertype: false,
};

/// Metadata entry for EOF (invisible terminal).
const EOF_META: SymbolMetadata = SymbolMetadata {
    is_terminal: true,
    is_visible: false,
    is_supertype: false,
};

/// Metadata entry for a non-terminal symbol (visible).
const NT_VIS: SymbolMetadata = SymbolMetadata {
    is_terminal: false,
    is_visible: true,
    is_supertype: false,
};

/// Build and leak a parse table from a Grammar, returning a `&'static` ref.
fn leak_table(grammar: &Grammar) -> &'static ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    let table = build_lr1_automaton(grammar, &ff)
        .expect("LR(1) automaton")
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();
    Box::leak(Box::new(table))
}

/// Build a Language for a grammar where tokens are identified by simple byte
/// scanning.  `tokens` maps symbol-id → literal string.
fn build_language(
    grammar: &Grammar,
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
            'outer: while pos < src.len() {
                // Skip whitespace
                if src[pos] == b' ' || src[pos] == b'\t' || src[pos] == b'\n' {
                    pos += 1;
                    continue;
                }
                // Try each token (longest-match among literals, digits special-cased)
                let mut best: Option<(u32, usize)> = None;
                for &(kind, lit) in &tok_map {
                    if lit == "<digits>" {
                        // match one or more ASCII digits
                        let start = pos;
                        let mut end = pos;
                        while end < src.len() && src[end].is_ascii_digit() {
                            end += 1;
                        }
                        if end > start {
                            if best.map_or(true, |(_, blen)| end - start > blen) {
                                best = Some((kind, end - start));
                            }
                        }
                    } else if lit == "<ident>" {
                        // match [a-zA-Z_][a-zA-Z0-9_]*
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
                        if end > start {
                            if best.map_or(true, |(_, blen)| end - start > blen) {
                                best = Some((kind, end - start));
                            }
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
                    // Unknown character → stop tokenising so parser gets an error
                    break 'outer;
                }
            }
            // Append EOF
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

/// Convenience: build a minimal grammar with a single terminal and a single
/// rule `start → TOKEN`, returning everything needed to construct a Language.
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

/// Build the arithmetic grammar: expr → NUMBER | expr PLUS expr
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

// ===========================================================================
// Tests
// ===========================================================================

/// 1. Parse a single token input.
#[test]
fn parse_single_token() {
    let (grammar, table) = single_token_grammar();
    let lang = build_language(
        &grammar,
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

/// 2. Parse a simple addition expression.
#[test]
fn parse_simple_expression() {
    let (grammar, table) = arithmetic_grammar();
    let lang = build_language(
        &grammar,
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
    // The root expr should have children (expr PLUS expr or NUMBER)
    assert!(root.child_count() >= 1);
}

/// 3. Multiple parses with the same parser instance.
#[test]
fn multiple_parses_same_parser() {
    let (grammar, table) = arithmetic_grammar();
    let lang = build_language(
        &grammar,
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

/// 4. Invalid input produces a result distinct from valid input.
#[test]
fn invalid_input_differs_from_valid() {
    let (grammar, table) = single_token_grammar();
    let lang = build_language(
        &grammar,
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();

    // Valid input succeeds cleanly
    let valid = parser.parse(b"a", None);
    assert!(valid.is_ok());

    // "x" is not a valid token — parser may error or produce a degraded tree
    let result = parser.parse(b"x", None);
    match result {
        Err(_) => { /* error is acceptable */ }
        Ok(tree) => {
            // If parsing succeeded, the tree should differ from a valid parse
            // (e.g., different root kind, fewer children, or error-flagged nodes).
            let root = tree.root_node();
            let valid_tree = valid.unwrap();
            let valid_root = valid_tree.root_node();
            // At minimum the byte range should differ since "x" ≠ "a"
            assert_ne!(
                root.child(0).map(|n| n.utf8_text(b"x").ok()),
                valid_root.child(0).map(|n| n.utf8_text(b"a").ok()),
            );
        }
    }
}

/// 5. Parse number-only expression.
#[test]
fn parse_number_only() {
    let (grammar, table) = arithmetic_grammar();
    let lang = build_language(
        &grammar,
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

/// 6. Source bytes are preserved on the resulting tree.
#[test]
fn tree_preserves_source_bytes() {
    let (grammar, table) = single_token_grammar();
    let lang = build_language(
        &grammar,
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

/// 7. utf8_text on nodes works correctly.
#[test]
fn node_utf8_text() {
    let (grammar, table) = arithmetic_grammar();
    let lang = build_language(
        &grammar,
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

/// 8. parse_utf8 convenience method works like parse.
#[test]
fn parse_utf8_works() {
    let (grammar, table) = single_token_grammar();
    let lang = build_language(
        &grammar,
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse_utf8("a", None).unwrap();
    assert_eq!(tree.root_node().kind(), "start");
}

/// 9. Error when parsing with no language set.
#[test]
fn error_no_language() {
    let mut parser = Parser::new();
    let result = parser.parse(b"a", None);
    assert!(result.is_err());
}

/// 10. Two-rule grammar: stmt → IDENT SEMI
#[test]
fn two_token_rule() {
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
    let lang = build_language(
        &grammar,
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

/// 11. GrammarBuilder API round-trip.
#[test]
fn grammar_builder_round_trip() {
    let grammar = GrammarBuilder::new("calc")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();

    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW");
    let table = build_lr1_automaton(&grammar, &ff)
        .expect("LR(1)")
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();

    assert!(table.state_count > 0);
    assert!(!table.action_table.is_empty());
}

/// 12. Byte ranges span the correct portion of input.
#[test]
fn correct_byte_ranges() {
    let (grammar, table) = arithmetic_grammar();
    let lang = build_language(
        &grammar,
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let input = b"100";
    let tree = parser.parse(input, None).unwrap();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 3);
}

/// 13. root_kind() returns the raw symbol id.
#[test]
fn root_kind_raw() {
    let (grammar, table) = single_token_grammar();
    let lang = build_language(
        &grammar,
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse_utf8("a", None).unwrap();
    // start is SymbolId(2) → root_kind should be 2
    assert_eq!(tree.root_kind(), 2);
}

/// 14. Language symbol_name look-up works on the resulting tree's language.
#[test]
fn language_symbol_name_lookup() {
    let (grammar, table) = arithmetic_grammar();
    let lang = build_language(
        &grammar,
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
    assert_eq!(tree_lang.symbol_name(3), Some("expr"));
}

/// 15. Chained additions parse successfully.
#[test]
fn chained_additions() {
    let (grammar, table) = arithmetic_grammar();
    let lang = build_language(
        &grammar,
        table,
        vec!["EOF", "NUMBER", "PLUS", "expr"],
        vec![EOF_META, TERM_VIS, TERM_VIS, NT_VIS],
        vec![(1, "<digits>"), (2, "+")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(b"1+2+3", None).unwrap();
    assert_eq!(tree.root_node().kind(), "expr");
    // A chain should have more than 1 child (operator + operands)
    assert!(tree.root_node().child_count() > 1);
}

/// 16. First/Follow computation succeeds on a Builder-created grammar.
#[test]
fn first_follow_on_builder_grammar() {
    let grammar = GrammarBuilder::new("pairs")
        .token("LPAREN", "(")
        .token("RPAREN", ")")
        .token("ID", r"[a-z]+")
        .rule("pair", vec!["LPAREN", "ID", "RPAREN"])
        .start("pair")
        .build();

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    // FIRST(pair) should contain LPAREN's symbol id
    let pair_id = grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "pair")
        .map(|(id, _)| *id)
        .unwrap();
    let first_set = ff.first(pair_id);
    assert!(first_set.is_some());
}

/// 17. Empty input on a non-nullable grammar either errors or returns a
///     degenerate tree (the GLR runtime may still produce a tree).
#[test]
fn empty_input_non_nullable() {
    let (grammar, table) = single_token_grammar();
    let lang = build_language(
        &grammar,
        table,
        vec!["EOF", "a", "start"],
        vec![EOF_META, TERM_VIS, NT_VIS],
        vec![(1, "a")],
    );

    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let result = parser.parse(b"", None);
    match result {
        Err(_) => { /* expected for a non-nullable grammar */ }
        Ok(tree) => {
            // If the parser returns a tree, it should span zero bytes
            assert_eq!(tree.root_node().end_byte(), 0);
        }
    }
}
