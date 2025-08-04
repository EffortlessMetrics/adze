use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_sitter::{
    glr_incremental::{Edit, IncrementalGLRParser, Position},
    glr_parser::GLRParser,
    glr_lexer::TokenWithPosition,
};
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::{Grammar, Rule, Symbol, SymbolId, ProductionId};
use std::sync::Arc;

/// Create a simple repetition grammar for benchmarking
fn create_test_grammar() -> Arc<Grammar> {
    let mut grammar = Grammar::new("benchmark".to_string());
    
    // S -> S A | A (left-recursive repetition)
    let s_sym = SymbolId(0);
    let a_sym = SymbolId(1);
    
    // Add symbol names for start symbol detection
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

/// Generate tokens for a sequence of 'a' characters
fn generate_tokens(count: usize) -> Vec<TokenWithPosition> {
    let mut tokens = Vec::new();
    let mut byte_offset = 0;
    
    for i in 0..count {
        tokens.push(TokenWithPosition {
            symbol_id: SymbolId(1), // 'a' terminal
            text: "a".to_string(),
            byte_offset,
            byte_length: 1,
        });
        byte_offset += 1;
        
        // Add spaces between tokens
        if i < count - 1 {
            byte_offset += 1; // space
        }
    }
    
    tokens
}

fn benchmark_incremental_parsing(c: &mut Criterion) {
    let grammar = create_test_grammar();
    
    // Build parse table
    let ff_sets = FirstFollowSets::compute(&grammar);
    let parse_table = match build_lr1_automaton(&grammar, &ff_sets) {
        Ok(table) => table,
        Err(e) => {
            eprintln!("Failed to build parse table: {:?}", e);
            return;
        }
    };
    
    let mut group = c.benchmark_group("incremental_parsing");
    group.sample_size(10); // Reduce sample size for faster benchmarking
    
    for size in [10, 50, 100, 500].iter() {
        let tokens = generate_tokens(*size);
        
        // Benchmark initial parse
        group.bench_with_input(
            BenchmarkId::new("initial_parse", size),
            &tokens,
            |b, tokens| {
                b.iter(|| {
                    let glr_parser = GLRParser::new(parse_table.clone(), (*grammar).clone());
                    let mut incremental = IncrementalGLRParser::new(glr_parser, grammar.clone());
                    incremental.parse_incremental(black_box(tokens), &[], None)
                });
            },
        );
        
        // Benchmark incremental parse with small edit in the middle
        let edit_pos = size / 2;
        let mut edited_tokens = tokens.clone();
        if edit_pos < edited_tokens.len() {
            // Remove one token to create a valid edit
            edited_tokens.remove(edit_pos);
        }
        
        let edit = Edit {
            start_byte: edit_pos * 2, // account for spaces
            old_end_byte: edit_pos * 2 + 1,
            new_end_byte: edit_pos * 2, // Deletion - end is same as start
            start_position: Position { line: 0, column: edit_pos * 2 },
            old_end_position: Position { line: 0, column: edit_pos * 2 + 1 },
            new_end_position: Position { line: 0, column: edit_pos * 2 }, // Same as start for deletion
        };
        
        group.bench_with_input(
            BenchmarkId::new("incremental_small_edit", size),
            &(tokens.clone(), edited_tokens.clone(), edit.clone()),
            |b, (orig_tokens, new_tokens, edit)| {
                b.iter_batched(
                    || {
                        // Setup: parse the original
                        let glr_parser = GLRParser::new(parse_table.clone(), (*grammar).clone());
                        let mut incremental = IncrementalGLRParser::new(glr_parser, grammar.clone());
                        let tree = incremental.parse_incremental(orig_tokens, &[], None).unwrap();
                        (incremental, tree)
                    },
                    |(mut incremental, tree)| {
                        // Benchmark: reparse with edit
                        incremental.parse_incremental(
                            black_box(new_tokens), 
                            black_box(&[edit.clone()]), 
                            Some(tree)
                        )
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
        
        // Measure reuse percentage
        if *size <= 100 { // Only for smaller sizes to avoid spam
            let glr_parser = GLRParser::new(parse_table.clone(), (*grammar).clone());
            let mut incremental = IncrementalGLRParser::new(glr_parser, grammar.clone());
            let tree = incremental.parse_incremental(&tokens, &[], None).unwrap();
            
            let _ = incremental.parse_incremental(&edited_tokens, &[edit.clone()], Some(tree));
            let stats = incremental.stats();
            
            println!(
                "Size {}: Reuse stats: {}/{} bytes ({:.1}%), {} subtrees reused",
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
    let grammar = create_test_grammar();
    
    // Build parse table
    let ff_sets = FirstFollowSets::compute(&grammar);
    let parse_table = match build_lr1_automaton(&grammar, &ff_sets) {
        Ok(table) => table,
        Err(e) => {
            eprintln!("Failed to build parse table: {:?}", e);
            return;
        }
    };
    
    let size = 100;
    let tokens = generate_tokens(size);
    
    let mut group = c.benchmark_group("edit_location_impact");
    group.sample_size(10);
    
    // Test edits at different locations
    for location in ["start", "middle", "end"].iter() {
        let edit_pos = match *location {
            "start" => 0,
            "middle" => size / 2,
            "end" => size - 1,
            _ => unreachable!(),
        };
        
        let mut edited_tokens = tokens.clone();
        if edit_pos < edited_tokens.len() {
            // Remove one token to create a valid edit
            edited_tokens.remove(edit_pos);
        }
        
        let edit = Edit {
            start_byte: edit_pos * 2,
            old_end_byte: edit_pos * 2 + 1,
            new_end_byte: edit_pos * 2, // Deletion
            start_position: Position { line: 0, column: edit_pos * 2 },
            old_end_position: Position { line: 0, column: edit_pos * 2 + 1 },
            new_end_position: Position { line: 0, column: edit_pos * 2 }, // Deletion
        };
        
        group.bench_with_input(
            BenchmarkId::new("edit_at", location),
            &(tokens.clone(), edited_tokens.clone(), edit.clone()),
            |b, (orig_tokens, new_tokens, edit)| {
                b.iter_batched(
                    || {
                        let glr_parser = GLRParser::new(parse_table.clone(), (*grammar).clone());
                        let mut incremental = IncrementalGLRParser::new(glr_parser, grammar.clone());
                        let tree = incremental.parse_incremental(orig_tokens, &[], None).unwrap();
                        (incremental, tree)
                    },
                    |(mut incremental, tree)| {
                        incremental.parse_incremental(
                            black_box(new_tokens), 
                            black_box(&[edit.clone()]), 
                            Some(tree)
                        )
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_incremental_parsing, benchmark_edit_location_impact);
criterion_main!(benches);