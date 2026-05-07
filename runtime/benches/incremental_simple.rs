#![cfg(feature = "unstable-benches")]

use adze::{
    glr_incremental::{Edit, GLRToken, IncrementalGLRParser},
    glr_lexer::TokenWithPosition,
    glr_parser::GLRParser,
};
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::Arc;

/// Create a simple left-recursive repetition grammar
fn create_test_grammar() -> Arc<Grammar> {
    let mut grammar = Grammar::new("benchmark".to_string());

    // S -> S A | A
    let s_sym = SymbolId(0);
    let a_sym = SymbolId(1);

    // Mark as start symbol
    grammar.rule_names.insert(s_sym, "source_file".to_string());

    // Add rules
    grammar.add_rule(Rule {
        lhs: s_sym,
        rhs: vec![Symbol::NonTerminal(s_sym), Symbol::Terminal(a_sym)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: Default::default(),
    });

    grammar.add_rule(Rule {
        lhs: s_sym,
        rhs: vec![Symbol::Terminal(a_sym)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: Default::default(),
    });

    Arc::new(grammar)
}

/// Create tokens manually since we have a simple grammar
fn create_tokens(count: usize) -> Vec<GLRToken> {
    let mut tokens = Vec::new();
    let mut byte_offset = 0;

    for _ in 0..count {
        tokens.push(GLRToken {
            symbol: SymbolId(1), // 'a' terminal
            text: b"a".to_vec(),
            start_byte: byte_offset,
            end_byte: byte_offset + 1,
        });
        byte_offset += 2; // 'a' + space
    }

    tokens
}

#[cfg(not(feature = "unstable-benches"))]
fn benchmark_incremental_parsing(_c: &mut Criterion) {
    // Temporarily disabled due to API changes
}

#[cfg(feature = "unstable-benches")]
fn benchmark_incremental_parsing(c: &mut Criterion) {
    let grammar = create_test_grammar();

    // Build parse table
    let ff_sets = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = match build_lr1_automaton(&grammar, &ff_sets) {
        Ok(table) => table,
        Err(e) => {
            eprintln!("Failed to build parse table: {:?}", e);
            return;
        }
    };

    let mut group = c.benchmark_group("incremental");
    group.sample_size(10);

    for size in [10, 50, 100].iter() {
        let tokens = create_tokens(*size);

        // Benchmark initial parse
        group.bench_with_input(BenchmarkId::new("initial", size), &tokens, |b, tokens| {
            b.iter(|| {
                let mut incremental =
                    IncrementalGLRParser::new((*grammar).clone(), parse_table.clone());
                incremental.parse_incremental(black_box(tokens), &[])
            });
        });

        // Create edit: remove token in middle
        let edit_pos = size / 2;
        let mut edited_tokens = tokens.clone();
        if edit_pos < edited_tokens.len() {
            edited_tokens.remove(edit_pos);
        }

        let edit = Edit {
            start_byte: (edit_pos * 2) as usize,
            old_end_byte: (edit_pos * 2 + 1) as usize,
            new_end_byte: (edit_pos * 2) as usize,
        };

        // Benchmark incremental parse
        group.bench_with_input(
            BenchmarkId::new("incremental", size),
            &(tokens.clone(), edited_tokens.clone(), edit.clone()),
            |b, (orig_tokens, new_tokens, edit)| {
                b.iter_batched(
                    || {
                        // Setup: parse the original
                        let mut incremental =
                            IncrementalGLRParser::new((*grammar).clone(), parse_table.clone());
                        let tree = incremental.parse_incremental(orig_tokens, &[]);
                        (incremental, tree)
                    },
                    |(mut incremental, tree)| {
                        if let Ok(tree) = tree {
                            // Benchmark: reparse with edit
                            // TODO: Fix incremental parsing API
                            let _ = incremental.parse_incremental(
                                black_box(new_tokens),
                                black_box(&[]), // Temporarily disable edits
                            );
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Print reuse stats for debugging
        if *size <= 50 {
            let glr_parser = GLRParser::new(parse_table.clone(), (*grammar).clone());
            let mut incremental = IncrementalGLRParser::new(glr_parser, grammar.clone());

            if let Ok(tree) = incremental.parse_incremental(&tokens, &[]) {
                let _ = incremental.parse_incremental(&edited_tokens, &[edit]);
                // TODO: stats() method no longer exists
                // let stats = incremental.stats();
                // println!(
                //     "Size {}: Reused {} bytes out of {} ({:.1}%)",
                //     size,
                //     stats.bytes_reused,
                //     stats.total_bytes,
                //     if stats.total_bytes > 0 {
                //         (stats.bytes_reused as f64 / stats.total_bytes as f64) * 100.0
                //     } else {
                //         0.0
                //     }
                // );
            }
        }
    }

    group.finish();
}
criterion_group!(benches, benchmark_incremental_parsing);
criterion_main!(benches);
