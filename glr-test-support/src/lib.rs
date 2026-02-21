use adze_glr_core::{Action, LexMode, ParseRule, ParseTable};
use adze_ir::{Grammar, StateId, SymbolId};
use std::collections::BTreeMap;

/// Test helpers for constructing minimal parse tables
///
/// ### Invariants captured here
/// - `EOF` column index **must equal** `token_count + external_token_count`.
/// - `ERROR` lives at column 0; terminals occupy the next `token_count` columns.
/// - `initial_state` is in range of `state_count`.
/// - `start_symbol` is a nonterminal present in `nonterminal_to_index`.
///
/// Sentinel used throughout the tests for "no goto".
pub const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal but fully-formed ParseTable suitable for unit tests.
///
/// Conventions expected by tests:
/// - Symbol layout: ERROR(0), terminals `[1..=token_count]`, EOF(token_count + external_token_count),
///   then non-terminals.
/// - `actions` is indexed by `[state][symbol_index]` and `gotos` by `[state][symbol_index]`.
///
/// `external_token_count` is usually 0 for tests.
pub fn make_minimal_table(
    actions: Vec<Vec<Vec<Action>>>,
    gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start_symbol: SymbolId,
    eof_symbol: SymbolId,
    external_token_count: usize,
) -> ParseTable {
    let state_count = actions.len();
    assert!(state_count > 0, "need at least 1 state");
    // Keep columns in actions/gotos aligned
    let action_cols = actions[0].len();
    let goto_cols = gotos.first().map(|r| r.len()).unwrap_or(0);
    let symbol_count = action_cols.max(goto_cols);
    assert!(symbol_count > 0, "need at least 1 symbol column");

    // Derive token_count from EOF; layout is ERROR(0) + terminals [1..=token_count] + EOF
    let eof_idx = eof_symbol.0 as usize;
    assert!(
        eof_idx > 0 && eof_idx <= symbol_count,
        "EOF column must be within symbol table"
    );
    let token_count = eof_idx
        .checked_sub(external_token_count)
        .expect("EOF < externals?");

    // Build a full symbol_to_index map (ERROR + all symbols visible in table)
    let mut symbol_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }

    // Non-terminals: any column that has a real goto in any state
    let mut nonterminal_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for (col, _) in (0..symbol_count).enumerate() {
        let any_real_goto = gotos
            .iter()
            .any(|row| row.get(col).copied().unwrap_or(INVALID) != INVALID);
        if any_real_goto {
            nonterminal_to_index.insert(SymbolId(col as u16), col);
        }
    }
    // Ensure the declared start symbol is present
    nonterminal_to_index
        .entry(start_symbol)
        .or_insert_with(|| start_symbol.0 as usize);

    // Ensure actions/gotos matrices have consistent shapes (pad if needed)
    let mut actions = actions;
    for row in &mut actions {
        if row.len() < symbol_count {
            row.resize_with(symbol_count, Vec::new);
        }
    }
    let mut gotos = gotos;
    if gotos.len() < state_count {
        gotos.resize_with(state_count, || vec![INVALID; symbol_count]);
    }
    for row in &mut gotos {
        if row.len() < symbol_count {
            row.resize(symbol_count, INVALID);
        }
    }

    // Minimal lexing configuration (one mode per state)
    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0
        };
        state_count
    ];

    // Build reverse map for index_to_symbol
    let mut index_to_symbol = vec![SymbolId(u16::MAX); symbol_to_index.len()];
    for (sym, &idx) in &symbol_to_index {
        index_to_symbol[idx] = *sym;
    }

    ParseTable {
        // core grids
        action_table: actions,
        goto_table: gotos,
        // grammar rules
        rules,
        // shapes
        state_count,
        symbol_count,
        // symbol bookkeeping
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        goto_indexing: adze_glr_core::GotoIndexing::NonterminalMap,
        symbol_metadata: vec![], // tests don't need metadata
        // token layout / sentinels
        token_count,
        external_token_count,
        eof_symbol,
        start_symbol,
        // parsing config
        initial_state: StateId(0),
        // lexing config
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        // advanced features (unused in hand tests)
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        // display / provenance (defaults are fine for tests)
        grammar: Grammar::default(),
    }
}

/// Assert that a ParseTable respects all invariants
pub fn assert_parse_table_invariants(table: &ParseTable) {
    // Verify EOF column index matches token counts
    let eof_column = table
        .symbol_to_index
        .get(&table.eof_symbol)
        .expect("EOF symbol must be in symbol_to_index");

    let expected_eof_column = table.token_count + table.external_token_count;
    assert_eq!(
        *eof_column, expected_eof_column,
        "EOF column {} != token_count {} + external_token_count {}",
        *eof_column, table.token_count, table.external_token_count
    );

    // Verify initial state is valid
    assert!(
        (table.initial_state.0 as usize) < table.state_count,
        "initial_state {} out of range (state_count={})",
        table.initial_state.0,
        table.state_count
    );

    // Verify start symbol is a nonterminal
    assert!(
        table.nonterminal_to_index.contains_key(&table.start_symbol),
        "start_symbol {:?} must be in nonterminal_to_index",
        table.start_symbol
    );

    // Verify table dimensions
    assert_eq!(
        table.action_table.len(),
        table.state_count,
        "action_table rows != state_count"
    );
    assert_eq!(
        table.goto_table.len(),
        table.state_count,
        "goto_table rows != state_count"
    );

    for (i, row) in table.action_table.iter().enumerate() {
        assert_eq!(
            row.len(),
            table.symbol_count,
            "action_table row {} has {} cols, expected {}",
            i,
            row.len(),
            table.symbol_count
        );
    }

    for (i, row) in table.goto_table.iter().enumerate() {
        assert_eq!(
            row.len(),
            table.symbol_count,
            "goto_table row {} has {} cols, expected {}",
            i,
            row.len(),
            table.symbol_count
        );
    }
}

pub mod test_utilities {
    pub use super::make_minimal_table;
}

pub mod perf;
