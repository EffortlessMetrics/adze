use rust_sitter_glr_core::{Action, LexMode, ParseRule, ParseTable, SymbolMetadata};
use rust_sitter_ir::{Grammar, RuleId, StateId, SymbolId};
use std::collections::BTreeMap;

/// Sentinel used throughout the tests for "no goto".
pub const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal but fully-formed ParseTable suitable for unit tests.
///
/// Conventions expected by tests:
/// - Symbol layout: ERROR(0), terminals [1..=token_count], EOF(token_count + external_token_count),
///   then non-terminals.
/// - `actions` is indexed by [state][symbol_index] and `gotos` by [state][symbol_index].
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
    let goto_cols = gotos.get(0).map(|r| r.len()).unwrap_or(0);
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
            row.resize_with(symbol_count, || Vec::new());
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
        nonterminal_to_index,
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
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        // display / provenance (defaults are fine for tests)
        grammar: Grammar::default(),
    }
}
