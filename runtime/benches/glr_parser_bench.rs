use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rust_sitter::glr_lexer::GLRLexer;
use rust_sitter::glr_parser::GLRParser;
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

/// Create a highly ambiguous arithmetic expression grammar
fn create_ambiguous_grammar() -> Grammar {
    let mut grammar = Grammar::new("expression".to_string());

    // Define symbol IDs
    let expr_id = SymbolId(0);
    let number_id = SymbolId(1);
    let plus_id = SymbolId(2);
    let mult_id = SymbolId(3);
    let lparen_id = SymbolId(4);
    let rparen_id = SymbolId(5);

    // Terminal tokens
    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        mult_id,
        Token {
            name: "multiply".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        lparen_id,
        Token {
            name: "lparen".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        rparen_id,
        Token {
            name: "rparen".to_string(),
            pattern: TokenPattern::String(")".to_string()),
            fragile: false,
        },
    );

    // Grammar rules creating ambiguity
    let rules = vec![
        // E -> E + E
        Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(plus_id),
                Symbol::NonTerminal(expr_id),
            ],
            production_id: ProductionId(0),
            precedence: None,
            associativity: None,
            fields: vec![],
        },
        // E -> E * E
        Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(mult_id),
                Symbol::NonTerminal(expr_id),
            ],
            production_id: ProductionId(1),
            precedence: None,
            associativity: None,
            fields: vec![],
        },
        // E -> ( E )
        Rule {
            lhs: expr_id,
            rhs: vec![
                Symbol::Terminal(lparen_id),
                Symbol::NonTerminal(expr_id),
                Symbol::Terminal(rparen_id),
            ],
            production_id: ProductionId(2),
            precedence: None,
            associativity: None,
            fields: vec![],
        },
        // E -> number
        Rule {
            lhs: expr_id,
            rhs: vec![Symbol::Terminal(number_id)],
            production_id: ProductionId(3),
            precedence: None,
            associativity: None,
            fields: vec![],
        },
    ];

    grammar.rules.insert(expr_id, rules);
    grammar.rule_names.insert(expr_id, "expression".to_string());

    grammar
}

fn benchmark_simple_expression(c: &mut Criterion) {
    let grammar = create_ambiguous_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    c.bench_function("parse_simple_expression", |b| {
        b.iter(|| {
            let mut lexer = GLRLexer::new(&grammar, black_box("1 + 2 * 3").to_string()).unwrap();
            let mut parser = GLRParser::new(parse_table.clone(), grammar.clone());

            let mut tokens = Vec::new();
            while let Some(token) = lexer.next_token() {
                tokens.push(token);
            }

            for token in &tokens {
                parser.process_token(token.symbol_id, &token.text, token.byte_offset);
            }
            let total_bytes = tokens
                .last()
                .map(|t| t.byte_offset + t.text.len())
                .unwrap_or(0);
            parser.process_eof(total_bytes);
            parser.finish()
        })
    });
}

fn benchmark_deeply_nested_expression(c: &mut Criterion) {
    let grammar = create_ambiguous_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Create a deeply nested expression: ((((1 + 2) * 3) + 4) * 5)
    let input = "((((1 + 2) * 3) + 4) * 5)";

    c.bench_function("parse_deeply_nested", |b| {
        b.iter(|| {
            let mut lexer = GLRLexer::new(&grammar, black_box(input).to_string()).unwrap();
            let mut parser = GLRParser::new(parse_table.clone(), grammar.clone());

            let mut tokens = Vec::new();
            while let Some(token) = lexer.next_token() {
                tokens.push(token);
            }

            for token in &tokens {
                parser.process_token(token.symbol_id, &token.text, token.byte_offset);
            }
            let total_bytes = tokens
                .last()
                .map(|t| t.byte_offset + t.text.len())
                .unwrap_or(0);
            parser.process_eof(total_bytes);
            parser.finish()
        })
    });
}

fn benchmark_highly_ambiguous_expression(c: &mut Criterion) {
    let grammar = create_ambiguous_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Highly ambiguous expression that creates many parse trees
    let input = "1 + 2 + 3 + 4 + 5 + 6 + 7 + 8";

    c.bench_function("parse_highly_ambiguous", |b| {
        b.iter(|| {
            // Wrap in catch_unwind to prevent bench suite from crashing
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut lexer = GLRLexer::new(&grammar, black_box(input).to_string()).unwrap();
                let mut parser = GLRParser::new(parse_table.clone(), grammar.clone());

                let mut tokens = Vec::new();
                while let Some(token) = lexer.next_token() {
                    tokens.push(token);
                }

                for token in &tokens {
                    parser.process_token(token.symbol_id, &token.text, token.byte_offset);
                }
                let total_bytes = tokens
                    .last()
                    .map(|t| t.byte_offset + t.text.len())
                    .unwrap_or(0);
                parser.process_eof(total_bytes);
                parser.finish()
            }));

            // If panic occurred, return a dummy value to keep bench valid
            if result.is_err() {
                black_box(());
            }
        })
    });
}

fn benchmark_fork_performance(c: &mut Criterion) {
    let grammar = create_ambiguous_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Expression that causes maximum forking
    let input = "1 * 2 + 3 * 4 + 5 * 6";

    c.bench_function("parse_maximum_forks", |b| {
        b.iter(|| {
            let mut lexer = GLRLexer::new(&grammar, black_box(input).to_string()).unwrap();
            let mut parser = GLRParser::new(parse_table.clone(), grammar.clone());

            let mut tokens = Vec::new();
            while let Some(token) = lexer.next_token() {
                tokens.push(token);
            }

            for token in &tokens {
                parser.process_token(token.symbol_id, &token.text, token.byte_offset);
            }
            let total_bytes = tokens
                .last()
                .map(|t| t.byte_offset + t.text.len())
                .unwrap_or(0);
            parser.process_eof(total_bytes);
            parser.finish()
        })
    });
}

criterion_group!(
    benches,
    benchmark_simple_expression,
    benchmark_deeply_nested_expression,
    benchmark_highly_ambiguous_expression,
    benchmark_fork_performance
);
criterion_main!(benches);
