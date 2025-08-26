//! Integration test for streaming lexer and GLR parser using a JSON grammar.

use rust_sitter_glr_core::ts_lexer::NextToken;
use rust_sitter_glr_core::{build_lr1_automaton, Driver, FirstFollowSets, LexMode};
use rust_sitter_ir::{
    FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};

// ---------------------------------------------------------------------------
// Symbol definitions (terminals first, then non-terminals)
// ---------------------------------------------------------------------------
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
const ARRAY: SymbolId = SymbolId(14);
const MEMBERS: SymbolId = SymbolId(15);
const MEMBER: SymbolId = SymbolId(16);
const ELEMENTS: SymbolId = SymbolId(17);

// ---------------------------------------------------------------------------
// Grammar and parse table construction
// ---------------------------------------------------------------------------
fn build_json_grammar() -> Grammar {
    let mut g = Grammar::new("json".to_string());

    // Tokens
    g.tokens.insert(
        STRING,
        Token {
            name: "string".into(),
            pattern: TokenPattern::Regex(r#"\"([^\"\\]|\\.)*\""#.into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        NUMBER,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(
                r"-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?".into(),
            ),
            fragile: false,
        },
    );
    g.tokens.insert(
        TRUE,
        Token {
            name: "true".into(),
            pattern: TokenPattern::String("true".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        FALSE,
        Token {
            name: "false".into(),
            pattern: TokenPattern::String("false".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        NULL,
        Token {
            name: "null".into(),
            pattern: TokenPattern::String("null".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        LBRACE,
        Token {
            name: "{".into(),
            pattern: TokenPattern::String("{".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        RBRACE,
        Token {
            name: "}".into(),
            pattern: TokenPattern::String("}".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        LBRACKET,
        Token {
            name: "[".into(),
            pattern: TokenPattern::String("[".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        RBRACKET,
        Token {
            name: "]".into(),
            pattern: TokenPattern::String("]".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        COMMA,
        Token {
            name: ",".into(),
            pattern: TokenPattern::String(",".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        COLON,
        Token {
            name: ":".into(),
            pattern: TokenPattern::String(":".into()),
            fragile: false,
        },
    );

    // Rule and field names
    g.rule_names.insert(VALUE, "value".into());
    g.rule_names.insert(OBJECT, "object".into());
    g.rule_names.insert(ARRAY, "array".into());
    g.rule_names.insert(MEMBERS, "members".into());
    g.rule_names.insert(MEMBER, "member".into());
    g.rule_names.insert(ELEMENTS, "elements".into());

    let key_field = FieldId(0);
    let value_field = FieldId(1);
    g.fields.insert(key_field, "key".into());
    g.fields.insert(value_field, "value".into());

    // Rules
    let mut pid: u16 = 0;
    let mut next = || {
        let id = ProductionId(pid);
        pid += 1;
        id
    };

    // value → terminals and composites
    g.add_rule(Rule { lhs: VALUE, rhs: vec![Symbol::Terminal(STRING)], precedence: None, associativity: None, fields: vec![], production_id: next() });
    g.add_rule(Rule { lhs: VALUE, rhs: vec![Symbol::Terminal(NUMBER)], precedence: None, associativity: None, fields: vec![], production_id: next() });
    g.add_rule(Rule { lhs: VALUE, rhs: vec![Symbol::Terminal(TRUE)], precedence: None, associativity: None, fields: vec![], production_id: next() });
    g.add_rule(Rule { lhs: VALUE, rhs: vec![Symbol::Terminal(FALSE)], precedence: None, associativity: None, fields: vec![], production_id: next() });
    g.add_rule(Rule { lhs: VALUE, rhs: vec![Symbol::Terminal(NULL)], precedence: None, associativity: None, fields: vec![], production_id: next() });
    g.add_rule(Rule { lhs: VALUE, rhs: vec![Symbol::NonTerminal(OBJECT)], precedence: None, associativity: None, fields: vec![], production_id: next() });
    g.add_rule(Rule { lhs: VALUE, rhs: vec![Symbol::NonTerminal(ARRAY)], precedence: None, associativity: None, fields: vec![], production_id: next() });

    // object → { } | { members }
    g.add_rule(Rule { lhs: OBJECT, rhs: vec![Symbol::Terminal(LBRACE), Symbol::Terminal(RBRACE)], precedence: None, associativity: None, fields: vec![], production_id: next() });
    g.add_rule(Rule { lhs: OBJECT, rhs: vec![Symbol::Terminal(LBRACE), Symbol::NonTerminal(MEMBERS), Symbol::Terminal(RBRACE)], precedence: None, associativity: None, fields: vec![], production_id: next() });

    // members → member | member , members
    g.add_rule(Rule { lhs: MEMBERS, rhs: vec![Symbol::NonTerminal(MEMBER)], precedence: None, associativity: None, fields: vec![], production_id: next() });
    g.add_rule(Rule { lhs: MEMBERS, rhs: vec![Symbol::NonTerminal(MEMBER), Symbol::Terminal(COMMA), Symbol::NonTerminal(MEMBERS)], precedence: None, associativity: None, fields: vec![], production_id: next() });

    // member → string : value
    g.add_rule(Rule { lhs: MEMBER, rhs: vec![Symbol::Terminal(STRING), Symbol::Terminal(COLON), Symbol::NonTerminal(VALUE)], precedence: None, associativity: None, fields: vec![(key_field, 0), (value_field, 2)], production_id: next() });

    // array → [ ] | [ elements ]
    g.add_rule(Rule { lhs: ARRAY, rhs: vec![Symbol::Terminal(LBRACKET), Symbol::Terminal(RBRACKET)], precedence: None, associativity: None, fields: vec![], production_id: next() });
    g.add_rule(Rule { lhs: ARRAY, rhs: vec![Symbol::Terminal(LBRACKET), Symbol::NonTerminal(ELEMENTS), Symbol::Terminal(RBRACKET)], precedence: None, associativity: None, fields: vec![], production_id: next() });

    // elements → value | value , elements
    g.add_rule(Rule { lhs: ELEMENTS, rhs: vec![Symbol::NonTerminal(VALUE)], precedence: None, associativity: None, fields: vec![], production_id: next() });
    g.add_rule(Rule { lhs: ELEMENTS, rhs: vec![Symbol::NonTerminal(VALUE), Symbol::Terminal(COMMA), Symbol::NonTerminal(ELEMENTS)], precedence: None, associativity: None, fields: vec![], production_id: next() });

    g
}

// ---------------------------------------------------------------------------
// Lexer
// ---------------------------------------------------------------------------
fn json_lexer(input: &str, pos: usize, _mode: LexMode) -> Option<NextToken> {
    let bytes = input.as_bytes();
    if pos >= bytes.len() {
        return None;
    }

    // Skip whitespace
    let mut p = pos;
    while p < bytes.len() && matches!(bytes[p], b' ' | b'\t' | b'\n' | b'\r') {
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
        b':' => Some(NextToken { kind: COLON.0 as u32, start: start as u32, end: (start + 1) as u32 }),
        b',' => Some(NextToken { kind: COMMA.0 as u32, start: start as u32, end: (start + 1) as u32 }),
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

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------
#[test]
fn test_streaming_json_parser() {
    let grammar = build_json_grammar();
    let ff = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &ff).expect("build parse table");
    let mut driver = Driver::new(&parse_table);

    let test_cases = vec![
        "{}",
        "[]",
        r#"{"key": "value"}"#,
        r#"[1, 2, 3]"#,
        r#"{"nested": {"object": true}}"#,
    ];

    for input in test_cases {
        // Tokenize input using streaming lexer
        let mut tokens = Vec::new();
        let mut pos = 0;
        while pos < input.len() {
            if let Some(tok) = json_lexer(input, pos, LexMode { lex_state: 0, external_lex_state: 0 }) {
                pos = tok.end as usize;
                tokens.push((tok.kind, tok.start, tok.end));
            } else {
                panic!("lexer stalled at byte {}", pos);
            }
        }

        // Parse token stream
        let forest = driver.parse_tokens(tokens).expect("parse");
        assert!(
            !forest.view().roots().is_empty(),
            "no parse trees returned for {}",
            input
        );
    }
}

