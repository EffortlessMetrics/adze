#![cfg(feature = "pure-rust")]

use rust_sitter_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use std::collections::BTreeMap;

// --- Terminals (tokens) -------------------------------------------------------
const LBRACE: SymbolId = SymbolId(0);
const RBRACE: SymbolId = SymbolId(1);
const COLON: SymbolId = SymbolId(2);
const COMMA: SymbolId = SymbolId(3);
const STRING: SymbolId = SymbolId(4);
const NUMBER: SymbolId = SymbolId(5);
// Optional (keeps lexer flexible)
const WS: SymbolId = SymbolId(6);

// --- Nonterminals -------------------------------------------------------------
const START: SymbolId = SymbolId(100);
const VALUE: SymbolId = SymbolId(101);
const OBJECT: SymbolId = SymbolId(102);
const PAIRS: SymbolId = SymbolId(103);
const PAIR: SymbolId = SymbolId(104);

// --- Fields (for pair) --------------------------------------------------------
const F_KEY: FieldId = FieldId(1);
const F_VALUE: FieldId = FieldId(2);

pub fn build_json_grammar() -> Grammar {
    let mut g = Grammar::new("json_min".to_string());

    // Tokens: order matters (the LR(1) builder will use these indices)
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
        COLON,
        Token {
            name: ":".into(),
            pattern: TokenPattern::String(":".into()),
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
        STRING,
        Token {
            name: "string".into(),
            pattern: TokenPattern::Regex(r#""([^"\\]|\\.)*""#.into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        NUMBER,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(r#"-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?"#.into()),
            fragile: false,
        },
    );
    // Optional whitespace token if you plan to use `extras` later:
    // g.tokens.insert(WS, Token { name: "WS".into(), pattern: TokenPattern::Regex(r"\s+".into()), fragile: false });

    // START -> VALUE
    g.rules.insert(
        START,
        vec![Rule {
            lhs: START,
            rhs: vec![Symbol::NonTerminal(VALUE)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    // VALUE -> STRING | NUMBER | OBJECT
    g.rules.insert(
        VALUE,
        vec![
            Rule {
                lhs: VALUE,
                rhs: vec![Symbol::Terminal(STRING)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: VALUE,
                rhs: vec![Symbol::Terminal(NUMBER)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: VALUE,
                rhs: vec![Symbol::NonTerminal(OBJECT)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
        ],
    );

    // OBJECT -> { } | { PAIRS }
    g.rules.insert(
        OBJECT,
        vec![
            Rule {
                lhs: OBJECT,
                rhs: vec![Symbol::Terminal(LBRACE), Symbol::Terminal(RBRACE)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: OBJECT,
                rhs: vec![
                    Symbol::Terminal(LBRACE),
                    Symbol::NonTerminal(PAIRS),
                    Symbol::Terminal(RBRACE),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
        ],
    );

    // PAIRS -> PAIR | PAIR , PAIRS
    g.rules.insert(
        PAIRS,
        vec![
            Rule {
                lhs: PAIRS,
                rhs: vec![Symbol::NonTerminal(PAIR)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: PAIRS,
                rhs: vec![
                    Symbol::NonTerminal(PAIR),
                    Symbol::Terminal(COMMA),
                    Symbol::NonTerminal(PAIRS),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
        ],
    );

    // PAIR -> STRING : VALUE   (with fields)
    g.rules.insert(
        PAIR,
        vec![Rule {
            lhs: PAIR,
            rhs: vec![
                Symbol::Terminal(STRING),
                Symbol::Terminal(COLON),
                Symbol::NonTerminal(VALUE),
            ],
            precedence: None,
            associativity: None,
            fields: vec![(F_KEY, 0), (F_VALUE, 2)],
            production_id: ProductionId(0),
        }],
    );

    // Optional: field name strings for F_KEY/F_VALUE (tablegen/language builder may read these)
    g.fields.insert(F_KEY, "key".to_string());
    g.fields.insert(F_VALUE, "value".to_string());

    // Add rule names so the grammar can identify the start symbol
    g.rule_names.insert(START, "source_file".to_string()); // Use Tree-sitter convention
    g.rule_names.insert(VALUE, "value".to_string());
    g.rule_names.insert(OBJECT, "object".to_string());
    g.rule_names.insert(PAIRS, "pairs".to_string());
    g.rule_names.insert(PAIR, "pair".to_string());

    g
}
