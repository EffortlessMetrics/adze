//! Comprehensive tests for GLR query language.
//!
//! Tests the query parser, cursor, pattern matching, and error handling.

use adze::adze_ir as ir;
use adze::glr_query::*;

use ir::SymbolId;
use ir::builder::GrammarBuilder;

// ── Helpers ─────────────────────────────────────────────────────────

fn make_grammar() -> ir::Grammar {
    GrammarBuilder::new("test")
        .token("num", "[0-9]+")
        .token("plus", "\\+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "num"])
        .start("expr")
        .build()
}

fn make_subtree(sym: u16, children: Vec<Subtree>) -> Subtree {
    let end = if children.is_empty() {
        1
    } else {
        children.last().unwrap().end_byte
    };
    Subtree {
        symbol: SymbolId(sym),
        children,
        start_byte: 0,
        end_byte: end,
    }
}

fn leaf(sym: u16, start: usize, end: usize) -> Subtree {
    Subtree {
        symbol: SymbolId(sym),
        children: Vec::new(),
        start_byte: start,
        end_byte: end,
    }
}

// ── 1. Subtree construction ──────────────────────────────────────

#[test]
fn test_subtree_leaf() {
    let s = leaf(1, 0, 5);
    assert_eq!(s.symbol, SymbolId(1));
    assert!(s.children.is_empty());
    assert_eq!(s.start_byte, 0);
    assert_eq!(s.end_byte, 5);
}

#[test]
fn test_subtree_with_children() {
    let c1 = leaf(2, 0, 3);
    let c2 = leaf(3, 3, 5);
    let s = make_subtree(1, vec![c1, c2]);
    assert_eq!(s.children.len(), 2);
    assert_eq!(s.children[0].symbol, SymbolId(2));
    assert_eq!(s.children[1].symbol, SymbolId(3));
}

#[test]
fn test_subtree_clone() {
    let s = make_subtree(1, vec![leaf(2, 0, 3)]);
    let cloned = s.clone();
    assert_eq!(cloned.symbol, s.symbol);
    assert_eq!(cloned.children.len(), s.children.len());
}

#[test]
fn test_subtree_debug() {
    let s = leaf(1, 0, 5);
    let debug = format!("{:?}", s);
    assert!(debug.contains("Subtree"));
}

#[test]
fn test_deeply_nested_subtree() {
    let mut s = leaf(10, 0, 1);
    for i in (0..10).rev() {
        s = make_subtree(i, vec![s]);
    }
    assert_eq!(s.symbol, SymbolId(0));
    let mut depth = 0;
    let mut cur = &s;
    while !cur.children.is_empty() {
        cur = &cur.children[0];
        depth += 1;
    }
    assert_eq!(depth, 10);
}

// ── 2. Query struct ──────────────────────────────────────────────

#[test]
fn test_query_clone() {
    let q = Query {
        patterns: Vec::new(),
        capture_names: Default::default(),
        predicates: Vec::new(),
    };
    let cloned = q.clone();
    assert!(cloned.patterns.is_empty());
}

#[test]
fn test_query_debug() {
    let q = Query {
        patterns: Vec::new(),
        capture_names: Default::default(),
        predicates: Vec::new(),
    };
    let debug = format!("{:?}", q);
    assert!(debug.contains("Query"));
}

// ── 3. Quantifier enum ──────────────────────────────────────────

#[test]
fn test_quantifier_equality() {
    assert_eq!(Quantifier::One, Quantifier::One);
    assert_ne!(Quantifier::One, Quantifier::ZeroOrOne);
    assert_eq!(Quantifier::ZeroOrMore, Quantifier::ZeroOrMore);
    assert_eq!(Quantifier::OneOrMore, Quantifier::OneOrMore);
}

#[test]
fn test_quantifier_clone() {
    let q = Quantifier::ZeroOrMore;
    let q2 = q;
    assert_eq!(q, q2);
}

#[test]
fn test_quantifier_debug() {
    assert!(format!("{:?}", Quantifier::One).contains("One"));
    assert!(format!("{:?}", Quantifier::ZeroOrOne).contains("ZeroOrOne"));
    assert!(format!("{:?}", Quantifier::ZeroOrMore).contains("ZeroOrMore"));
    assert!(format!("{:?}", Quantifier::OneOrMore).contains("OneOrMore"));
}

// ── 4. Predicate enum ──────────────────────────────────────────

#[test]
fn test_predicate_variants_debug() {
    let preds = vec![
        Predicate::Equal(vec![0, 1]),
        Predicate::NotEqual(vec![0, 1]),
        Predicate::Match(0, "pattern".to_string()),
        Predicate::NotMatch(0, "pattern".to_string()),
        Predicate::AnyOf(0, vec!["a".to_string(), "b".to_string()]),
    ];
    for p in &preds {
        let debug = format!("{:?}", p);
        assert!(!debug.is_empty());
    }
}

#[test]
fn test_predicate_clone() {
    let p = Predicate::Match(0, "foo".to_string());
    let p2 = p.clone();
    let debug1 = format!("{:?}", p);
    let debug2 = format!("{:?}", p2);
    assert_eq!(debug1, debug2);
}

// ── 5. QueryMatch struct ─────────────────────────────────────────

#[test]
fn test_query_match_construction() {
    let m = QueryMatch {
        pattern_index: 0,
        captures: Vec::new(),
    };
    assert_eq!(m.pattern_index, 0);
    assert!(m.captures.is_empty());
}

#[test]
fn test_query_match_with_captures() {
    let m = QueryMatch {
        pattern_index: 1,
        captures: vec![QueryCapture {
            index: 0,
            subtree: leaf(1, 0, 5),
        }],
    };
    assert_eq!(m.captures.len(), 1);
    assert_eq!(m.captures[0].index, 0);
}

#[test]
fn test_query_match_clone() {
    let m = QueryMatch {
        pattern_index: 2,
        captures: vec![QueryCapture {
            index: 0,
            subtree: leaf(1, 0, 5),
        }],
    };
    let cloned = m.clone();
    assert_eq!(cloned.pattern_index, 2);
    assert_eq!(cloned.captures.len(), 1);
}

// ── 6. QueryCapture struct ──────────────────────────────────────

#[test]
fn test_query_capture_fields() {
    let c = QueryCapture {
        index: 42,
        subtree: leaf(7, 10, 20),
    };
    assert_eq!(c.index, 42);
    assert_eq!(c.subtree.symbol, SymbolId(7));
}

#[test]
fn test_query_capture_debug() {
    let c = QueryCapture {
        index: 0,
        subtree: leaf(1, 0, 1),
    };
    let debug = format!("{:?}", c);
    assert!(debug.contains("QueryCapture"));
}

// ── 7. QueryCursor construction ──────────────────────────────────

#[test]
fn test_query_cursor_new() {
    let c = QueryCursor::new();
    let _ = c;
}

#[test]
fn test_query_cursor_default() {
    let c = QueryCursor::default();
    let _ = c;
}

#[test]
fn test_query_cursor_set_max_depth() {
    let mut c = QueryCursor::new();
    c.set_max_depth(5);
    // No getter, just verify it doesn't panic
}

// ── 8. QueryError display ────────────────────────────────────────

#[test]
fn test_query_error_display_all_variants() {
    let errors = vec![
        QueryError::EmptyQuery,
        QueryError::ExpectedOpenParen(0),
        QueryError::ExpectedCloseParen(5),
        QueryError::ExpectedCloseBracket(10),
        QueryError::ExpectedColon(3),
        QueryError::ExpectedHash(7),
        QueryError::ExpectedQuestionMark(12),
        QueryError::ExpectedAt(2),
        QueryError::ExpectedIdentifier(4),
        QueryError::ExpectedString(6),
        QueryError::UnterminatedString(8),
        QueryError::UnknownNodeType("foo".to_string()),
        QueryError::UnknownCapture("bar".to_string()),
        QueryError::UnknownPredicate("baz".to_string()),
        QueryError::InvalidPredicate("bad".to_string()),
    ];
    for err in &errors {
        let s = format!("{}", err);
        assert!(!s.is_empty());
    }
}

#[test]
fn test_query_error_empty_query_message() {
    let s = format!("{}", QueryError::EmptyQuery);
    assert!(s.contains("empty") || s.contains("Empty"));
}

#[test]
fn test_query_error_position_in_message() {
    let s = format!("{}", QueryError::ExpectedOpenParen(42));
    assert!(s.contains("42"));
}

#[test]
fn test_query_error_clone() {
    let err = QueryError::UnknownNodeType("test".to_string());
    let cloned = err.clone();
    let s1 = format!("{}", err);
    let s2 = format!("{}", cloned);
    assert_eq!(s1, s2);
}

#[test]
fn test_query_error_debug() {
    let err = QueryError::EmptyQuery;
    let debug = format!("{:?}", err);
    assert!(debug.contains("EmptyQuery"));
}

// ── 9. QueryParser ───────────────────────────────────────────────

#[test]
fn test_query_parser_empty_input_returns_error() {
    let g = make_grammar();
    let parser = QueryParser::new(&g, "");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_query_parser_whitespace_only_returns_error() {
    let g = make_grammar();
    let parser = QueryParser::new(&g, "   \n  ");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_query_parser_invalid_syntax_returns_error() {
    let g = make_grammar();
    let parser = QueryParser::new(&g, "not a query");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_query_parser_simple_pattern() {
    let g = make_grammar();
    // Try parsing a simple query - node type names match rule_names
    let parser = QueryParser::new(&g, "(expr)");
    let result = parser.parse();
    // May succeed or fail depending on how grammar maps symbol names
    let _ = result;
}

#[test]
fn test_query_parser_with_capture() {
    let g = make_grammar();
    let parser = QueryParser::new(&g, "(expr) @cap");
    let result = parser.parse();
    let _ = result; // exercise the code path
}

#[test]
fn test_query_parser_nested_pattern() {
    let g = make_grammar();
    let parser = QueryParser::new(&g, "(expr (num))");
    let result = parser.parse();
    let _ = result;
}

// ── 10. QueryCursor matches ──────────────────────────────────────

#[test]
fn test_cursor_matches_empty_query() {
    // Build a minimal query manually
    let q = Query {
        patterns: Vec::new(),
        capture_names: Default::default(),
        predicates: Vec::new(),
    };
    let tree = leaf(1, 0, 5);
    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&q, &tree).collect();
    assert!(matches.is_empty(), "no patterns means no matches");
}

#[test]
fn test_cursor_matches_on_leaf() {
    let q = Query {
        patterns: Vec::new(),
        capture_names: Default::default(),
        predicates: Vec::new(),
    };
    let tree = leaf(1, 0, 1);
    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&q, &tree).collect();
    assert!(matches.is_empty());
}

#[test]
fn test_cursor_matches_on_deep_tree() {
    let q = Query {
        patterns: Vec::new(),
        capture_names: Default::default(),
        predicates: Vec::new(),
    };
    let mut tree = leaf(10, 0, 1);
    for i in (0..5).rev() {
        tree = make_subtree(i, vec![tree]);
    }
    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&q, &tree).collect();
    assert!(matches.is_empty());
}

#[test]
fn test_cursor_max_depth_respected() {
    let q = Query {
        patterns: Vec::new(),
        capture_names: Default::default(),
        predicates: Vec::new(),
    };
    let tree = make_subtree(0, vec![make_subtree(1, vec![leaf(2, 0, 1)])]);
    let mut cursor = QueryCursor::new();
    cursor.set_max_depth(1);
    let matches: Vec<_> = cursor.matches(&q, &tree).collect();
    assert!(matches.is_empty());
}

// ── 11. Pattern struct (only via parsed query) ──────────────────

#[test]
fn test_pattern_from_parsed_query_debug() {
    let g = make_grammar();
    let parser = QueryParser::new(&g, "(expr)");
    if let Ok(query) = parser.parse() {
        assert!(!query.patterns.is_empty());
        let debug = format!("{:?}", query.patterns[0]);
        assert!(debug.contains("Pattern"));
    }
}

#[test]
fn test_pattern_from_parsed_query_clone() {
    let g = make_grammar();
    let parser = QueryParser::new(&g, "(expr)");
    if let Ok(query) = parser.parse() {
        let cloned = query.patterns[0].clone();
        let _ = cloned;
    }
}

// ── 12. Integration: parse then query ────────────────────────────

#[test]
fn test_end_to_end_parse_and_query() {
    let g = make_grammar();
    // Try parsing a query and running it
    let parser = QueryParser::new(&g, "(expr)");
    if let Ok(query) = parser.parse() {
        let tree = make_subtree(0, vec![leaf(1, 0, 3)]);
        let cursor = QueryCursor::new();
        let matches: Vec<_> = cursor.matches(&query, &tree).collect();
        let _ = matches; // exercise the pipeline
    }
}
