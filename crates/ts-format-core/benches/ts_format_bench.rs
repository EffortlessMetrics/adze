//! Benchmarks for ts-format-core hot-path functions.
//!
//! Measures performance of action selection which is called frequently during parsing.

use std::collections::BTreeMap;

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use adze_glr_core::{
    Action, GotoIndexing, Grammar, LexMode, ParseRule, ParseTable, RuleId, StateId, SymbolId,
    SymbolMetadata,
};
use adze_ts_format_core::{choose_action, choose_action_with_precedence};

/// Create a symbol metadata instance for testing.
fn make_symbol_metadata(id: u16) -> SymbolMetadata {
    SymbolMetadata {
        name: format!("symbol_{}", id),
        is_visible: true,
        is_named: true,
        is_supertype: false,
        is_terminal: id < 2,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(id),
    }
}

/// Create a parse table with minimal data for benchmarking.
fn make_minimal_parse_table() -> ParseTable {
    ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![
            make_symbol_metadata(0),
            make_symbol_metadata(1),
            make_symbol_metadata(2),
        ],
        state_count: 1,
        symbol_count: 3,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar: Grammar::default(),
        initial_state: StateId(0),
        token_count: 2,
        external_token_count: 0,
        lex_modes: vec![LexMode {
            lex_state: 0,
            external_lex_state: 0,
        }],
        extras: vec![],
        dynamic_prec_by_rule: vec![0, 10, 20],
        rule_assoc_by_rule: vec![0, 1, 2],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

fn bench_choose_action_empty(c: &mut Criterion) {
    c.bench_function("choose_action_empty_cell", |b| {
        let cell: Vec<Action> = vec![];
        b.iter(|| black_box(choose_action(black_box(&cell))));
    });
}

fn bench_choose_action_single_shift(c: &mut Criterion) {
    c.bench_function("choose_action_single_shift", |b| {
        let cell = vec![Action::Shift(StateId(1))];
        b.iter(|| black_box(choose_action(black_box(&cell))));
    });
}

fn bench_choose_action_single_reduce(c: &mut Criterion) {
    c.bench_function("choose_action_single_reduce", |b| {
        let cell = vec![Action::Reduce(RuleId(0))];
        b.iter(|| black_box(choose_action(black_box(&cell))));
    });
}

fn bench_choose_action_mixed_cell(c: &mut Criterion) {
    c.bench_function("choose_action_mixed_4_actions", |b| {
        let cell = vec![
            Action::Shift(StateId(1)),
            Action::Reduce(RuleId(0)),
            Action::Reduce(RuleId(1)),
            Action::Error,
        ];
        b.iter(|| black_box(choose_action(black_box(&cell))));
    });
}

fn bench_choose_action_with_accept(c: &mut Criterion) {
    c.bench_function("choose_action_with_accept", |b| {
        let cell = vec![
            Action::Shift(StateId(1)),
            Action::Reduce(RuleId(0)),
            Action::Accept,
        ];
        b.iter(|| black_box(choose_action(black_box(&cell))));
    });
}

fn bench_choose_action_large_cell(c: &mut Criterion) {
    c.bench_function("choose_action_large_16_actions", |b| {
        let cell: Vec<Action> = (0..16)
            .flat_map(|i| vec![Action::Shift(StateId(i)), Action::Reduce(RuleId(i as u16))])
            .collect();
        b.iter(|| black_box(choose_action(black_box(&cell))));
    });
}

fn bench_choose_action_with_precedence_simple(c: &mut Criterion) {
    c.bench_function("choose_action_with_precedence_simple", |b| {
        let table = make_minimal_parse_table();
        let cell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(1))];
        b.iter(|| {
            black_box(choose_action_with_precedence(
                black_box(&cell),
                black_box(&table),
            ))
        });
    });
}

fn bench_choose_action_with_precedence_complex(c: &mut Criterion) {
    c.bench_function("choose_action_with_precedence_8_actions", |b| {
        let table = make_minimal_parse_table();
        let cell: Vec<Action> = (0..4)
            .flat_map(|i| vec![Action::Shift(StateId(i)), Action::Reduce(RuleId(i as u16))])
            .collect();
        b.iter(|| {
            black_box(choose_action_with_precedence(
                black_box(&cell),
                black_box(&table),
            ))
        });
    });
}

criterion_group!(
    benches,
    bench_choose_action_empty,
    bench_choose_action_single_shift,
    bench_choose_action_single_reduce,
    bench_choose_action_mixed_cell,
    bench_choose_action_with_accept,
    bench_choose_action_large_cell,
    bench_choose_action_with_precedence_simple,
    bench_choose_action_with_precedence_complex,
);

criterion_main!(benches);
