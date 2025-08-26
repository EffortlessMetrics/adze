//! Test streaming lexer with a real JSON grammar

use rust_sitter_glr_core::ts_lexer::NextToken;
use rust_sitter_glr_core::{build_lr1_automaton, Driver, FirstFollowSets, LexMode};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

/// Symbol constants for JSON tokens and nonterminals
const STRING: SymbolId = SymbolId(1);
const NUMBER: SymbolId = SymbolId(2);
const TRUE: SymbolId = SymbolId(3);
const FALSE: SymbolId = SymbolId(4);
const NULL: SymbolId = SymbolId(5);
const LBRACE: SymbolId = SymbolId(6);
const RBRACE: SymbolId = SymbolId(7);
const LBRACKET: SymbolId = SymbolId(8);
const RBRACKET: SymbolId = SymbolId(9);
const COMMA: SymbolId = SymbolId(10);
const COLON: SymbolId = SymbolId(11);

const VALUE: SymbolId = SymbolId(12);
const OBJECT: SymbolId = SymbolId(13);
const MEMBERS: SymbolId = SymbolId(14);
const PAIR: SymbolId = SymbolId(15);
const ARRAY: SymbolId = SymbolId(16);
const ELEMENTS: SymbolId = SymbolId(17);
const DOCUMENT: SymbolId = SymbolId(18);

/// Build a minimal JSON grammar covering objects, arrays and primitives
fn build_json_grammar() -> Grammar {
    let mut g = Grammar::new("json".to_string());

    // --- tokens ---
    g.tokens.insert(
        STRING,
        Token {
            name: "string".into(),
            pattern: TokenPattern::Regex(r#""([^"\\]|\\.)*""# .into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        NUMBER,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(
                r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?".into(),
            ),
            fragile: false,
        },
    );
    g.tokens.insert(
        TRUE,
        Token { name: "true".into(), pattern: TokenPattern::String("true".into()), fragile: false },
    );
    g.tokens.insert(
        FALSE,
        Token { name: "false".into(), pattern: TokenPattern::String("false".into()), fragile: false },
    );
    g.tokens.insert(
        NULL,
        Token { name: "null".into(), pattern: TokenPattern::String("null".into()), fragile: false },
    );
    g.tokens.insert(
        LBRACE,
        Token { name: "{".into(), pattern: TokenPattern::String("{".into()), fragile: false },
    );
    g.tokens.insert(
        RBRACE,
        Token { name: "}".into(), pattern: TokenPattern::String("}".into()), fragile: false },
    );
    g.tokens.insert(
        LBRACKET,
        Token { name: "[".into(), pattern: TokenPattern::String("[".into()), fragile: false },
    );
    g.tokens.insert(
        RBRACKET,
        Token { name: "]".into(), pattern: TokenPattern::String("]".into()), fragile: false },
    );
    g.tokens.insert(
        COMMA,
        Token { name: ",".into(), pattern: TokenPattern::String(",".into()), fragile: false },
    );
    g.tokens.insert(
        COLON,
        Token { name: ":".into(), pattern: TokenPattern::String(":".into()), fragile: false },
    );

    // --- rule names (helps determine start symbol) ---
    g.rule_names.insert(DOCUMENT, "source_file".into());
    g.rule_names.insert(VALUE, "value".into());
    g.rule_names.insert(OBJECT, "object".into());
    g.rule_names.insert(MEMBERS, "members".into());
    g.rule_names.insert(PAIR, "pair".into());
    g.rule_names.insert(ARRAY, "array".into());
    g.rule_names.insert(ELEMENTS, "elements".into());

    // --- helper to add rules ---
    let mut next_prod: u16 = 0;
    let mut add_rule = |g: &mut Grammar, lhs: SymbolId, rhs: Vec<Symbol>| {
        g.add_rule(Rule {
            lhs,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(next_prod),
        });
        next_prod += 1;
    };

    // DOCUMENT → VALUE
    add_rule(&mut g, DOCUMENT, vec![Symbol::NonTerminal(VALUE)]);

    // VALUE → STRING | NUMBER | TRUE | FALSE | NULL | OBJECT | ARRAY
    add_rule(&mut g, VALUE, vec![Symbol::Terminal(STRING)]);
    add_rule(&mut g, VALUE, vec![Symbol::Terminal(NUMBER)]);
    add_rule(&mut g, VALUE, vec![Symbol::Terminal(TRUE)]);
    add_rule(&mut g, VALUE, vec![Symbol::Terminal(FALSE)]);
    add_rule(&mut g, VALUE, vec![Symbol::Terminal(NULL)]);
    add_rule(&mut g, VALUE, vec![Symbol::NonTerminal(OBJECT)]);
    add_rule(&mut g, VALUE, vec![Symbol::NonTerminal(ARRAY)]);

    // OBJECT → { } | { MEMBERS }
    add_rule(
        &mut g,
        OBJECT,
        vec![Symbol::Terminal(LBRACE), Symbol::Terminal(RBRACE)],
    );
    add_rule(
        &mut g,
        OBJECT,
        vec![
            Symbol::Terminal(LBRACE),
            Symbol::NonTerminal(MEMBERS),
            Symbol::Terminal(RBRACE),
        ],
    );

    // MEMBERS → PAIR | PAIR , MEMBERS
    add_rule(&mut g, MEMBERS, vec![Symbol::NonTerminal(PAIR)]);
    add_rule(
        &mut g,
        MEMBERS,
        vec![
            Symbol::NonTerminal(PAIR),
            Symbol::Terminal(COMMA),
            Symbol::NonTerminal(MEMBERS),
        ],
    );

    // PAIR → STRING : VALUE
    add_rule(
        &mut g,
        PAIR,
        vec![
            Symbol::Terminal(STRING),
            Symbol::Terminal(COLON),
            Symbol::NonTerminal(VALUE),
        ],
    );

    // ARRAY → [ ] | [ ELEMENTS ]
    add_rule(
        &mut g,
        ARRAY,
        vec![Symbol::Terminal(LBRACKET), Symbol::Terminal(RBRACKET)],
    );
    add_rule(
        &mut g,
        ARRAY,
        vec![
            Symbol::Terminal(LBRACKET),
            Symbol::NonTerminal(ELEMENTS),
            Symbol::Terminal(RBRACKET),
        ],
    );

    // ELEMENTS → VALUE | VALUE , ELEMENTS
    add_rule(&mut g, ELEMENTS, vec![Symbol::NonTerminal(VALUE)]);
    add_rule(
        &mut g,
        ELEMENTS,
        vec![
            Symbol::NonTerminal(VALUE),
            Symbol::Terminal(COMMA),
            Symbol::NonTerminal(ELEMENTS),
        ],
    );

    g
}

/// Simple streaming lexer for the JSON grammar above
fn json_lexer(input: &str, pos: usize, _mode: LexMode) -> Option<NextToken> {
    let bytes = input.as_bytes();
    if pos >= bytes.len() {
        return None;
    }
    let mut p = pos;
    while p < bytes.len() && bytes[p].is_ascii_whitespace() {
        p += 1;
    }
    if p >= bytes.len() {
        return None;
    }
    let start = p;
    match bytes[p] {
        b'{' => Some(NextToken { kind: LBRACE.0 as u32, start: start as u32, end: (start + 1) as u32 }),
        b'}' => Some(NextToken { kind: RBRACE.0 as u32, start: start as u32, end: (start + 1) as u32 }),
        b'[' => Some(NextToken { kind: LBRACKET.0 as u32, start: start as u32, end: (start + 1) as u32 }),
        b']' => Some(NextToken { kind: RBRACKET.0 as u32, start: start as u32, end: (start + 1) as u32 }),
        b',' => Some(NextToken { kind: COMMA.0 as u32, start: start as u32, end: (start + 1) as u32 }),
        b':' => Some(NextToken { kind: COLON.0 as u32, start: start as u32, end: (start + 1) as u32 }),
        b'"' => {
            let mut end = start + 1;
            while end < bytes.len() && bytes[end] != b'"' {
                if bytes[end] == b'\\' && end + 1 < bytes.len() {
                    end += 2;
                } else {
                    end += 1;
                }
            }
            if end < bytes.len() {
                end += 1;
            }
            Some(NextToken { kind: STRING.0 as u32, start: start as u32, end: end as u32 })
        }
        b't' if bytes[p..].starts_with(b"true") => Some(NextToken {
            kind: TRUE.0 as u32,
            start: start as u32,
            end: (start + 4) as u32,
        }),
        b'f' if bytes[p..].starts_with(b"false") => Some(NextToken {
            kind: FALSE.0 as u32,
            start: start as u32,
            end: (start + 5) as u32,
        }),
        b'n' if bytes[p..].starts_with(b"null") => Some(NextToken {
            kind: NULL.0 as u32,
            start: start as u32,
            end: (start + 4) as u32,
        }),
        b'0'..=b'9' | b'-' => {
            let mut end = start;
            if bytes[end] == b'-' { end += 1; }
            while end < bytes.len() && bytes[end].is_ascii_digit() { end += 1; }
            if end < bytes.len() && bytes[end] == b'.' {
                end += 1;
                while end < bytes.len() && bytes[end].is_ascii_digit() { end += 1; }
            }
            if end < bytes.len() && matches!(bytes[end], b'e' | b'E') {
                end += 1;
                if end < bytes.len() && matches!(bytes[end], b'+' | b'-') { end += 1; }
                while end < bytes.len() && bytes[end].is_ascii_digit() { end += 1; }
            }
            Some(NextToken { kind: NUMBER.0 as u32, start: start as u32, end: end as u32 })
        }
        _ => None,
    }
}

#[test]
fn test_streaming_json_parser() {
    let grammar = build_json_grammar();
    let ff = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &ff).expect("build parse table");

    let test_cases = vec![
        "{}",
        "[]",
        r#"{"key": "value"}"#,
        r#"[1, 2, 3]"#,
        r#"{"nested": {"object": true}}"#,
    ];

    for input in test_cases {
        let mut driver = Driver::new(&parse_table);
        let result = driver.parse_streaming(
            input,
            json_lexer,
            None::<fn(&str, usize, &[bool], LexMode) -> Option<NextToken>>,
        );
        assert!(result.is_ok(), "failed to parse {}", input);
    }
}

