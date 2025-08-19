#![cfg(feature = "unstable-benches")]

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rust_sitter::{
    glr_incremental::{Edit, IncrementalGLRParser, Position},
    glr_lexer::{GLRLexer, TokenWithPosition},
    glr_parser::GLRParser,
};
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use std::sync::Arc;

/// Create arithmetic expression grammar
fn create_arithmetic_grammar() -> Grammar {
    let mut grammar = Grammar::new("arithmetic".to_string());

    // Symbol IDs
    let expr_id = SymbolId(0);
    let number_id = SymbolId(1);
    let plus_id = SymbolId(2);

    // Mark expr as the start symbol
    grammar.rule_names.insert(expr_id, "expression".to_string());

    // Tokens
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

    // Rules: E -> E + E | NUMBER
    grammar.add_rule(Rule {
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
    });

    grammar.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        production_id: ProductionId(1),
        precedence: None,
        associativity: None,
        fields: vec![],
    });

    grammar
}

/// Generate input string with given number of additions
fn generate_input(count: usize) -> String {
    (0..count)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" + ")
}

fn benchmark_incremental_parsing(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();

    // Build parse table
    let ff_sets = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &ff_sets).expect("Failed to build parse table");

    let mut group = c.benchmark_group("incremental_parsing");
    group.sample_size(20);

    for size in [10, 50, 100].iter() {
        // Original input
        let input = generate_input(*size);
        let lexer = GLRLexer::new(&grammar, &input).expect("Failed to create lexer");
        let tokens: Vec<TokenWithPosition> = lexer.collect();

        // Modified input - change one number in the middle
        let edit_pos = size / 2;
        let modified_input = generate_input(edit_pos)
            + " + 999 + "
            + &(edit_pos + 1..*size)
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(" + ");
        let modified_lexer =
            GLRLexer::new(&grammar, &modified_input).expect("Failed to create lexer");
        let modified_tokens: Vec<TokenWithPosition> = modified_lexer.collect();

        // Calculate edit
        let old_token = &tokens[edit_pos * 2]; // account for operators
        let new_token = &modified_tokens[edit_pos * 2];
        let edit = Edit {
            start_byte: old_token.byte_offset,
            old_end_byte: old_token.byte_offset + old_token.byte_length,
            new_end_byte: new_token.byte_offset + new_token.byte_length,
            start_position: Position {
                line: 0,
                column: old_token.byte_offset,
            },
            old_end_position: Position {
                line: 0,
                column: old_token.byte_offset + old_token.byte_length,
            },
            new_end_position: Position {
                line: 0,
                column: new_token.byte_offset + new_token.byte_length,
            },
        };

        // Benchmark initial parse
        group.bench_with_input(
            BenchmarkId::new("initial_parse", size),
            &tokens,
            |b, tokens| {
                b.iter(|| {
                    let glr_parser = GLRParser::new(parse_table.clone(), Arc::new(grammar.clone()));
                    let mut incremental =
                        IncrementalGLRParser::new(glr_parser, Arc::new(grammar.clone()));
                    incremental.parse_incremental(black_box(tokens), &[], None)
                });
            },
        );

        // Benchmark incremental parse
        group.bench_with_input(
            BenchmarkId::new("incremental_edit", size),
            &(tokens.clone(), modified_tokens.clone(), edit.clone()),
            |b, (orig_tokens, new_tokens, edit)| {
                b.iter_batched(
                    || {
                        // Setup: parse the original
                        let glr_parser =
                            GLRParser::new(parse_table.clone(), Arc::new(grammar.clone()));
                        let mut incremental =
                            IncrementalGLRParser::new(glr_parser, Arc::new(grammar.clone()));
                        let tree = incremental
                            .parse_incremental(orig_tokens, &[], None)
                            .expect("Initial parse failed");
                        (incremental, tree)
                    },
                    |(mut incremental, tree)| {
                        // Benchmark: reparse with edit
                        incremental.parse_incremental(
                            black_box(new_tokens),
                            black_box(&[edit.clone()]),
                        )
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Report reuse statistics
        if *size <= 100 {
            let glr_parser = GLRParser::new(parse_table.clone(), Arc::new(grammar.clone()));
            let mut incremental = IncrementalGLRParser::new(glr_parser, Arc::new(grammar.clone()));

            let tree = incremental
                .parse_incremental(&tokens, &[], None)
                .expect("Initial parse failed");

            let _ = incremental.parse_incremental(&modified_tokens, &[edit.clone()], Some(tree));
            let stats = incremental.stats();

            println!(
                "Size {}: Reuse {}/{} bytes ({:.1}%), {} subtrees reused",
                size,
                stats.bytes_reused,
                stats.total_bytes,
                if stats.total_bytes > 0 {
                    (stats.bytes_reused as f64 / stats.total_bytes as f64) * 100.0
                } else {
                    0.0
                },
                stats.subtrees_reused
            );
        }
    }

    group.finish();
}

fn benchmark_edit_location_impact(c: &mut Criterion) {
    let grammar = create_arithmetic_grammar();

    // Build parse table
    let ff_sets = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &ff_sets).expect("Failed to build parse table");

    let size = 50;
    let input = generate_input(size);
    let lexer = GLRLexer::new(&grammar, &input).expect("Failed to create lexer");
    let tokens: Vec<TokenWithPosition> = lexer.collect();

    let mut group = c.benchmark_group("edit_location_impact");
    group.sample_size(20);

    for location in ["start", "middle", "end"].iter() {
        let edit_pos = match *location {
            "start" => 0,
            "middle" => size / 2,
            "end" => size - 1,
            _ => unreachable!(),
        };

        // Create modified input
        let mut parts: Vec<String> = (0..size).map(|i| i.to_string()).collect();
        parts[edit_pos] = "999".to_string();
        let modified_input = parts.join(" + ");

        let modified_lexer =
            GLRLexer::new(&grammar, &modified_input).expect("Failed to create lexer");
        let modified_tokens: Vec<TokenWithPosition> = modified_lexer.collect();

        // Calculate edit
        let old_token = &tokens[edit_pos * 2];
        let new_token = &modified_tokens[edit_pos * 2];
        let edit = Edit {
            start_byte: old_token.byte_offset,
            old_end_byte: old_token.byte_offset + old_token.byte_length,
            new_end_byte: new_token.byte_offset + new_token.byte_length,
            start_position: Position {
                line: 0,
                column: old_token.byte_offset,
            },
            old_end_position: Position {
                line: 0,
                column: old_token.byte_offset + old_token.byte_length,
            },
            new_end_position: Position {
                line: 0,
                column: new_token.byte_offset + new_token.byte_length,
            },
        };

        group.bench_with_input(
            BenchmarkId::new("edit_at", location),
            &(tokens.clone(), modified_tokens.clone(), edit.clone()),
            |b, (orig_tokens, new_tokens, edit)| {
                b.iter_batched(
                    || {
                        let glr_parser =
                            GLRParser::new(parse_table.clone(), Arc::new(grammar.clone()));
                        let mut incremental =
                            IncrementalGLRParser::new(glr_parser, Arc::new(grammar.clone()));
                        let tree = incremental
                            .parse_incremental(orig_tokens, &[], None)
                            .expect("Initial parse failed");
                        (incremental, tree)
                    },
                    |(mut incremental, tree)| {
                        incremental.parse_incremental(
                            black_box(new_tokens),
                            black_box(&[edit.clone()]),
                        )
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_incremental_parsing,
    benchmark_edit_location_impact
);
criterion_main!(benches);
