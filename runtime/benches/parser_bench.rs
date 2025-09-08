// Benchmarks for the pure-Rust Tree-sitter parser
// This measures performance of various parsing operations
#![cfg(feature = "unstable-benches")]
#![allow(unused_imports, dead_code)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_sitter::lexer::{ErrorRecoveringLexer, ErrorRecoveryMode, GrammarLexer};
use rust_sitter::parser_v4::{ParserV4 as ParserV2, Token};
// use rust_sitter::incremental::{IncrementalParser, Edit, IncrementalTree};
use rust_sitter_glr_core::{Action, ParseTable, SymbolMetadata};
use rust_sitter_ir::{
    Grammar, ProductionId, Rule, RuleId, StateId, Symbol, SymbolId, Token as IrToken, TokenPattern,
};
use std::collections::HashMap;

/// Create a simple arithmetic grammar for benchmarking
fn create_arithmetic_grammar() -> Grammar {
    let mut grammar = Grammar::new();

    // Define tokens
    grammar.tokens.insert(
        SymbolId(1),
        IrToken {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(2),
        IrToken {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(3),
        IrToken {
            name: "times".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(4),
        IrToken {
            name: "lparen".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(5),
        IrToken {
            name: "rparen".to_string(),
            pattern: TokenPattern::String(")".to_string()),
            fragile: false,
        },
    );

    // Add whitespace as skip token
    grammar.skip_tokens.insert(
        SymbolId(6),
        IrToken {
            name: "whitespace".to_string(),
            pattern: TokenPattern::Regex(r"[ \t\n\r]+".to_string()),
            fragile: false,
        },
    );

    // Define non-terminals
    grammar
        .non_terminals
        .insert(SymbolId(10), "expression".to_string());
    grammar
        .non_terminals
        .insert(SymbolId(11), "term".to_string());
    grammar
        .non_terminals
        .insert(SymbolId(12), "factor".to_string());

    // Define rules
    // expression -> expression + term
    grammar.rules.insert(
        RuleId(0),
        Rule {
            lhs: SymbolId(10),
            rhs: vec![
                Symbol::NonTerminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(2)),
                Symbol::NonTerminal(SymbolId(11)),
            ],
            precedence: Some(1),
            associativity: None,
            production_id: ProductionId(0),
            fields: Default::default(),
        },
    );

    // expression -> term
    grammar.rules.insert(
        RuleId(1),
        Rule {
            lhs: SymbolId(10),
            rhs: vec![Symbol::NonTerminal(SymbolId(11))],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: Default::default(),
        },
    );

    // term -> term * factor
    grammar.rules.insert(
        RuleId(2),
        Rule {
            lhs: SymbolId(11),
            rhs: vec![
                Symbol::NonTerminal(SymbolId(11)),
                Symbol::Terminal(SymbolId(3)),
                Symbol::NonTerminal(SymbolId(12)),
            ],
            precedence: Some(2),
            associativity: None,
            production_id: ProductionId(0),
            fields: Default::default(),
        },
    );

    // term -> factor
    grammar.rules.insert(
        RuleId(3),
        Rule {
            lhs: SymbolId(11),
            rhs: vec![Symbol::NonTerminal(SymbolId(12))],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: Default::default(),
        },
    );

    // factor -> number
    grammar.rules.insert(
        RuleId(4),
        Rule {
            lhs: SymbolId(12),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: Default::default(),
        },
    );

    // factor -> ( expression )
    grammar.rules.insert(
        RuleId(5),
        Rule {
            lhs: SymbolId(12),
            rhs: vec![
                Symbol::Terminal(SymbolId(4)),
                Symbol::NonTerminal(SymbolId(10)),
                Symbol::Terminal(SymbolId(5)),
            ],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: Default::default(),
        },
    );

    grammar
}

/// Create a sample parse table for benchmarking
fn create_parse_table() -> ParseTable {
    let mut table = ParseTable {
        states: vec![],
        symbol_metadata: HashMap::new(),
    };

    // Add symbol metadata
    table.symbol_metadata.insert(
        SymbolId(1),
        SymbolMetadata {
            name: "number".to_string(),
            is_terminal: true,
        },
    );

    // Create a simple state with actions
    let mut actions = HashMap::new();
    actions.insert(SymbolId(1), Action::Shift(StateId(1)));
    actions.insert(SymbolId(4), Action::Shift(StateId(2)));

    table.states.push(rust_sitter_glr_core::State {
        actions,
        gotos: HashMap::new(),
        default_reduction: None,
    });

    // Add more states...
    for i in 1..10 {
        let mut actions = HashMap::new();
        actions.insert(SymbolId(2), Action::Shift(StateId((i + 1) % 10)));
        actions.insert(SymbolId(3), Action::Shift(StateId((i + 2) % 10)));
        actions.insert(SymbolId(0), Action::Accept); // EOF

        table.states.push(rust_sitter_glr_core::State {
            actions,
            gotos: HashMap::new(),
            default_reduction: Some(RuleId(i % 6)),
        });
    }

    table
}

fn benchmark_lexer(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();
    let token_patterns: Vec<_> = grammar
        .tokens
        .iter()
        .map(|(symbol_id, token)| (*symbol_id, token.pattern.clone(), 0))
        .collect();

    c.bench_function("lexer_simple", |b| {
        b.iter(|| {
            let mut lexer = GrammarLexer::new(&token_patterns);
            let input = "123 + 456 * 789";
            let tokens = lexer.tokenize(black_box(input));
            assert_eq!(tokens.len(), 5);
        });
    });

    c.bench_function("lexer_complex", |b| {
        b.iter(|| {
            let mut lexer = GrammarLexer::new(&token_patterns);
            let input = "(123 + 456) * (789 + 321) * (654 + 987)";
            let tokens = lexer.tokenize(black_box(input));
            assert_eq!(tokens.len(), 19);
        });
    });

    c.bench_function("lexer_with_errors", |b| {
        b.iter(|| {
            let base_lexer = GrammarLexer::new(&token_patterns);
            let mut lexer = ErrorRecoveringLexer::new(base_lexer, ErrorRecoveryMode::SkipChar);
            let input = "123 + @ 456 * # 789";
            let tokens = lexer.tokenize(black_box(input));
            assert!(tokens.len() >= 5);
        });
    });
}

fn benchmark_parser(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();
    let table = create_parse_table();

    c.bench_function("parser_simple", |b| {
        b.iter(|| {
            let mut parser = ParserV2::new(grammar.clone(), table.clone());
            let tokens = vec![Token {
                symbol: SymbolId(1),
                text: "123".to_string(),
                start: 0,
                end: 3,
            }];
            let _ = parser.parse(black_box(&tokens));
        });
    });

    c.bench_function("parser_expression", |b| {
        b.iter(|| {
            let mut parser = ParserV2::new(grammar.clone(), table.clone());
            let tokens = vec![
                Token {
                    symbol: SymbolId(1),
                    text: "123".to_string(),
                    start: 0,
                    end: 3,
                },
                Token {
                    symbol: SymbolId(2),
                    text: "+".to_string(),
                    start: 4,
                    end: 5,
                },
                Token {
                    symbol: SymbolId(1),
                    text: "456".to_string(),
                    start: 6,
                    end: 9,
                },
                Token {
                    symbol: SymbolId(3),
                    text: "*".to_string(),
                    start: 10,
                    end: 11,
                },
                Token {
                    symbol: SymbolId(1),
                    text: "789".to_string(),
                    start: 12,
                    end: 15,
                },
            ];
            let _ = parser.parse(black_box(&tokens));
        });
    });
}

/*
fn benchmark_incremental(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();
    let table = create_parse_table();

    c.bench_function("incremental_small_edit", |b| {
        let tokens = vec![
            Token { symbol: SymbolId(1), text: "123".to_string(), start: 0, end: 3 },
            Token { symbol: SymbolId(2), text: "+".to_string(), start: 4, end: 5 },
            Token { symbol: SymbolId(1), text: "456".to_string(), start: 6, end: 9 },
        ];

        let mut parser = IncrementalParser::new(grammar.clone(), table.clone());
        let old_tree = parser.parse_incremental(&tokens, None, &[]).unwrap();

        b.iter(|| {
            let edit = Edit::new(0, 3, 4); // Change "123" to "1234"
            let new_tokens = vec![
                Token { symbol: SymbolId(1), text: "1234".to_string(), start: 0, end: 4 },
                Token { symbol: SymbolId(2), text: "+".to_string(), start: 5, end: 6 },
                Token { symbol: SymbolId(1), text: "456".to_string(), start: 7, end: 10 },
            ];
            let _ = parser.parse_incremental(black_box(&new_tokens), Some(&old_tree), &[edit]);
        });
    });

    c.bench_function("incremental_large_edit", |b| {
        let tokens: Vec<Token> = (0..100)
            .flat_map(|i| vec![
                Token { symbol: SymbolId(1), text: format!("{}", i), start: i * 4, end: i * 4 + 3 },
                Token { symbol: SymbolId(2), text: "+".to_string(), start: i * 4 + 3, end: i * 4 + 4 },
            ])
            .collect();

        let mut parser = IncrementalParser::new(grammar.clone(), table.clone());
        let old_tree = parser.parse_incremental(&tokens[..tokens.len()-1], None, &[]).unwrap();

        b.iter(|| {
            // Insert in the middle
            let edit = Edit::new(200, 200, 210);
            let _ = parser.parse_incremental(black_box(&tokens), Some(&old_tree), &[edit]);
        });
    });
}
*/

criterion_group!(benches, benchmark_lexer, benchmark_parser); // , benchmark_incremental);
criterion_main!(benches);
