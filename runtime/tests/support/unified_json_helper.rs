// Minimal "tablegen → unified parser" glue for tests.
// Now uses a real JSON grammar and LR(1) table so the pure-Rust tests run
// without needing pre-generated artifacts.

mod language_builder;

use rust_sitter::pure_parser::TSLanguage;
use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::{
    FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};

/// Build a fully-wired `TSLanguage` for a small JSON grammar.
pub fn unified_json_language() -> &'static TSLanguage {
    let grammar = build_json_grammar();
    let mut table = build_json_parse_table(&grammar);

    // Basic sanity checks to ensure the table looks reasonable
    assert_eq!(table.token_count, grammar.tokens.len(), "token_count drift");
    assert!(table.state_count > 0, "no states generated");
    assert!(!table.action_table.is_empty(), "action table is empty");

    // Normalize the table to Tree-sitter format before building the language
    language_builder::normalize_table_for_ts(&mut table);

    let lang = language_builder::build_json_ts_language(&grammar, &table);
    Box::leak(Box::new(lang))
}

// --- Grammar & parse table helpers -----------------------------------------

fn build_json_grammar() -> Grammar {
    let mut g = Grammar::new("json_min".to_string());

    // Tokens
    g.tokens.insert(
        SymbolId(0),
        Token {
            name: "{".into(),
            pattern: TokenPattern::String("{".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "}".into(),
            pattern: TokenPattern::String("}".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: ":".into(),
            pattern: TokenPattern::String(":".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(3),
        Token {
            name: ",".into(),
            pattern: TokenPattern::String(",".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(4),
        Token {
            name: "string".into(),
            pattern: TokenPattern::Regex(r#""([^"\\]|\\.)*""#.into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(5),
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(
                r#"-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?"#.into(),
            ),
            fragile: false,
        },
    );

    // Nonterminals
    const DOCUMENT: SymbolId = SymbolId(99);
    const START: SymbolId = SymbolId(100);
    const VALUE: SymbolId = SymbolId(101);
    const OBJECT: SymbolId = SymbolId(102);
    const PAIRS: SymbolId = SymbolId(103);
    const PAIR: SymbolId = SymbolId(104);

    // Fields
    const F_KEY: FieldId = FieldId(1);
    const F_VALUE: FieldId = FieldId(2);

    // DOCUMENT -> OBJECT
    g.rules.insert(
        DOCUMENT,
        vec![Rule {
            lhs: DOCUMENT,
            rhs: vec![Symbol::NonTerminal(OBJECT)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    // START -> VALUE
    g.rules.insert(
        START,
        vec![Rule {
            lhs: START,
            rhs: vec![Symbol::NonTerminal(VALUE)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );

    // VALUE -> STRING | NUMBER | OBJECT
    g.rules.insert(
        VALUE,
        vec![
            Rule {
                lhs: VALUE,
                rhs: vec![Symbol::Terminal(SymbolId(4))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(2),
            },
            Rule {
                lhs: VALUE,
                rhs: vec![Symbol::Terminal(SymbolId(5))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(3),
            },
            Rule {
                lhs: VALUE,
                rhs: vec![Symbol::NonTerminal(OBJECT)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(4),
            },
        ],
    );

    // OBJECT -> { } | { PAIRS }
    g.rules.insert(
        OBJECT,
        vec![
            Rule {
                lhs: OBJECT,
                rhs: vec![Symbol::Terminal(SymbolId(0)), Symbol::Terminal(SymbolId(1))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(5),
            },
            Rule {
                lhs: OBJECT,
                rhs: vec![
                    Symbol::Terminal(SymbolId(0)),
                    Symbol::NonTerminal(PAIRS),
                    Symbol::Terminal(SymbolId(1)),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(6),
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
                production_id: ProductionId(7),
            },
            Rule {
                lhs: PAIRS,
                rhs: vec![
                    Symbol::NonTerminal(PAIR),
                    Symbol::Terminal(SymbolId(3)),
                    Symbol::NonTerminal(PAIRS),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(8),
            },
        ],
    );

    // PAIR -> STRING : VALUE
    g.rules.insert(
        PAIR,
        vec![Rule {
            lhs: PAIR,
            rhs: vec![
                Symbol::Terminal(SymbolId(4)),
                Symbol::Terminal(SymbolId(2)),
                Symbol::NonTerminal(VALUE),
            ],
            precedence: None,
            associativity: None,
            fields: vec![(F_KEY, 0), (F_VALUE, 2)],
            production_id: ProductionId(9),
        }],
    );

    // Field names and rule names
    g.fields.insert(F_KEY, "key".to_string());
    g.fields.insert(F_VALUE, "value".to_string());

    g.rule_names.insert(DOCUMENT, "source_file".to_string());
    g.rule_names.insert(START, "start".to_string());
    g.rule_names.insert(VALUE, "value".to_string());
    g.rule_names.insert(OBJECT, "object".to_string());
    g.rule_names.insert(PAIRS, "pairs".to_string());
    g.rule_names.insert(PAIR, "pair".to_string());

    g
}

fn build_json_parse_table(grammar: &Grammar) -> ParseTable {
    use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
    let ff = FirstFollowSets::compute(grammar);
    build_lr1_automaton(grammar, &ff).expect("build LR(1) automaton")
}

