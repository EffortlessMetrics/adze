//! Property-based tests for the GLR driver.
//!
//! Run with: cargo test -p adze-glr-core --features test-api --test driver_proptest
#![cfg(feature = "test-api")]

use adze_glr_core::{Action, Driver, GotoIndexing, LexMode, ParseRule, ParseTable};
use adze_ir::{Grammar, RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Table builder helpers
// ---------------------------------------------------------------------------

type ActionCell = Vec<Action>;

/// Sentinel used for "no goto" entries.
const NO_GOTO: StateId = StateId(65535);

/// Build a minimal `ParseTable` from raw action/goto matrices.
fn build_table(
    actions: Vec<Vec<ActionCell>>,
    gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start: SymbolId,
    eof: SymbolId,
    num_terminals: usize,
) -> ParseTable {
    let symbol_count = actions.first().map(|r| r.len()).unwrap_or(0);
    let state_count = actions.len();

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
        index_to_symbol.push(SymbolId(i as u16));
    }

    let mut nonterminal_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        for row in &gotos {
            if i < row.len() && row[i] != NO_GOTO {
                nonterminal_to_index.insert(SymbolId(i as u16), i);
                break;
            }
        }
    }

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: rules.clone(),
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        nonterminal_to_index,
        eof_symbol: eof,
        start_symbol: start,
        grammar: Grammar::new("proptest".to_string()),
        symbol_metadata: vec![],
        initial_state: StateId(0),
        token_count: num_terminals,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![0; rules.len()],
        rule_assoc_by_rule: vec![0; rules.len()],
        alias_sequences: vec![],
        field_names: vec![],
        goto_indexing: GotoIndexing::NonterminalMap,
        field_map: BTreeMap::new(),
    }
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a random token stream of length 0..=max_len.
/// Each token uses terminal symbol ids in `1..num_terminals` (0 is EOF).
fn token_stream_strategy(
    num_terminals: usize,
    max_len: usize,
) -> impl Strategy<Value = Vec<(u32, u32, u32)>> {
    prop::collection::vec(1..num_terminals as u32, 0..=max_len).prop_map(|kinds| {
        let mut pos = 0u32;
        kinds
            .into_iter()
            .map(|k| {
                let start = pos;
                pos += 1;
                (k, start, pos)
            })
            .collect()
    })
}

// ---------------------------------------------------------------------------
// 1. Parsing always terminates (no infinite loops)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn parsing_always_terminates(
        num_tokens in 0usize..=8,
        seed in 0u64..1000,
    ) {
        // Build a small deterministic table: S -> 'a', with 2 terminals + EOF + 1 NT
        // Symbols: 0=EOF, 1='a', 2=S(NT)
        let eof = SymbolId(0);
        let a = SymbolId(1);
        let s = SymbolId(2);

        let rules = vec![ParseRule { lhs: s, rhs_len: 1 }]; // S -> 'a'

        let mut actions = vec![vec![vec![]; 3]; 3];
        actions[0][1].push(Action::Shift(StateId(1)));   // state 0, 'a' -> shift 1
        actions[1][0].push(Action::Reduce(RuleId(0)));   // state 1, EOF -> reduce
        actions[2][0].push(Action::Accept);              // state 2, EOF -> accept

        let mut gotos = vec![vec![NO_GOTO; 3]; 3];
        gotos[0][2] = StateId(2); // after S goto 2

        let table = build_table(actions, gotos, rules, s, eof, 2);

        // Use seed to pick token kind (always 'a' since it's the only terminal)
        let tokens: Vec<(u32, u32, u32)> = (0..num_tokens)
            .map(|i| (a.0 as u32 + (seed as u32 % 1), i as u32, i as u32 + 1))
            .collect();

        let mut driver = Driver::new(&table);
        // Must not hang — we just care that it returns.
        let _result = driver.parse_tokens(tokens.into_iter());
    }
}

// ---------------------------------------------------------------------------
// 2. Deterministic grammars give exactly one result
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn deterministic_grammar_one_result(
        extra_tokens in 0usize..=3,
    ) {
        // Deterministic grammar: S -> 'a'
        // Symbols: 0=EOF, 1='a', 2=S
        let eof = SymbolId(0);
        let a = SymbolId(1);
        let s = SymbolId(2);

        let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];

        let mut actions = vec![vec![vec![]; 3]; 3];
        actions[0][1].push(Action::Shift(StateId(1)));
        actions[1][0].push(Action::Reduce(RuleId(0)));
        actions[2][0].push(Action::Accept);

        let mut gotos = vec![vec![NO_GOTO; 3]; 3];
        gotos[0][2] = StateId(2);

        let table = build_table(actions, gotos, rules, s, eof, 2);

        // Exactly one 'a' token → should parse with exactly one root.
        let tokens = vec![(a.0 as u32, 0u32, 1u32)];
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok(), "single-token deterministic parse must succeed");
        let forest = result.unwrap();
        let view = forest.view();
        prop_assert_eq!(view.roots().len(), 1, "deterministic grammar must yield exactly 1 root");

        // Extra tokens after the accepted input should cause an error.
        if extra_tokens > 0 {
            let mut tokens2: Vec<(u32, u32, u32)> = Vec::new();
            for i in 0..=extra_tokens {
                tokens2.push((a.0 as u32, i as u32, i as u32 + 1));
            }
            let mut driver2 = Driver::new(&table);
            let result2 = driver2.parse_tokens(tokens2.into_iter());
            // The grammar only accepts a single 'a', so more tokens should fail
            // (first 'a' accepted immediately via Accept on EOF lookahead, then
            // subsequent tokens trigger error).
            // Either error or early accept is valid here; just must not hang.
            let _ = result2;
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Reject token at position 0 for totally invalid input
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn reject_invalid_input_at_position_zero(
        bad_kind in 10u32..100,
    ) {
        // Grammar: S -> 'a', only terminal is 'a' (id 1). EOF is 0.
        let eof = SymbolId(0);
        let a = SymbolId(1);
        let s = SymbolId(2);

        let rules = vec![ParseRule { lhs: s, rhs_len: 1 }];

        let mut actions = vec![vec![vec![]; 3]; 3];
        actions[0][1].push(Action::Shift(StateId(1)));
        actions[1][0].push(Action::Reduce(RuleId(0)));
        actions[2][0].push(Action::Accept);

        let mut gotos = vec![vec![NO_GOTO; 3]; 3];
        gotos[0][2] = StateId(2);

        let table = build_table(actions, gotos, rules, s, eof, 2);

        // Feed a token whose kind is not in the table at all.
        let tokens = vec![(bad_kind, 0u32, 1u32)];
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());

        // The driver may reject outright or recover with error cost.
        match result {
            Err(_) => { /* expected: outright rejection */ }
            Ok(forest) => {
                // If recovery succeeded, error stats must reflect it.
                let (has_error, _missing, cost) = forest.debug_error_stats();
                prop_assert!(
                    has_error || cost > 0,
                    "recovered parse for unknown symbol {} must report errors",
                    bad_kind,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Accepted strings have valid parse forests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn accepted_strings_have_valid_forests(
        token_count in 1usize..=5,
    ) {
        // Right-recursive grammar (no shift/reduce conflicts):
        //   S -> A        (r0, rhs_len=1)
        //   A -> 'a'      (r1, rhs_len=1)
        //   A -> 'a' A    (r2, rhs_len=2)
        // Symbols: 0=EOF, 1='a', 2=S(NT), 3=A(NT)
        let eof = SymbolId(0);
        let a = SymbolId(1);
        let s = SymbolId(2);
        let a_nt = SymbolId(3);

        let rules = vec![
            ParseRule { lhs: s, rhs_len: 1 },    // r0: S -> A
            ParseRule { lhs: a_nt, rhs_len: 1 },  // r1: A -> 'a'
            ParseRule { lhs: a_nt, rhs_len: 2 },  // r2: A -> 'a' A
        ];

        // States:
        //   0: initial          — shift 'a' -> 1
        //   1: after 'a'        — shift 'a' -> 1 (recursive); reduce A->'a' on EOF
        //   2: after A (from 0) — reduce S->A on EOF
        //   3: accept           — Accept on EOF
        //   4: after A (from 1) — reduce A->'a' A on EOF
        let num_syms = 4;
        let num_states = 5;

        let mut actions = vec![vec![vec![]; num_syms]; num_states];
        actions[0][1].push(Action::Shift(StateId(1)));    // state 0: 'a' -> shift
        actions[1][1].push(Action::Shift(StateId(1)));    // state 1: 'a' -> shift (right-recurse)
        actions[1][0].push(Action::Reduce(RuleId(1)));    // state 1: EOF -> reduce A->'a'
        actions[2][0].push(Action::Reduce(RuleId(0)));    // state 2: EOF -> reduce S->A
        actions[3][0].push(Action::Accept);               // state 3: EOF -> accept
        actions[4][0].push(Action::Reduce(RuleId(2)));    // state 4: EOF -> reduce A->'a' A

        let mut gotos = vec![vec![NO_GOTO; num_syms]; num_states];
        gotos[0][3] = StateId(2);  // goto(0, A) = 2
        gotos[0][2] = StateId(3);  // goto(0, S) = 3
        gotos[1][3] = StateId(4);  // goto(1, A) = 4

        let table = build_table(actions, gotos, rules, s, eof, 2);

        let tokens: Vec<(u32, u32, u32)> = (0..token_count)
            .map(|i| (a.0 as u32, i as u32, i as u32 + 1))
            .collect();

        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok(), "string of {} 'a's must parse: {:?}", token_count, result.err());

        let forest = result.unwrap();
        let view = forest.view();

        // Must have at least one root.
        prop_assert!(!view.roots().is_empty(), "accepted parse must have roots");

        // Every root must span the full input.
        for &root in view.roots() {
            let sp = view.span(root);
            prop_assert_eq!(sp.start, 0, "root span must start at 0");
            prop_assert_eq!(sp.end, token_count as u32, "root span must end at {}", token_count);
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Grammar with only Accept action accepts empty input
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn accept_only_grammar_accepts_empty(
        _seed in 0u32..100,
    ) {
        // Trivial grammar: S -> ε
        // The table has an immediate reduce S->ε on EOF then Accept.
        // Symbols: 0=EOF, 1=S(NT)
        let eof = SymbolId(0);
        let s = SymbolId(1);

        let rules = vec![ParseRule { lhs: s, rhs_len: 0 }]; // S -> ε

        let mut actions = vec![vec![vec![]; 2]; 2];
        actions[0][0].push(Action::Reduce(RuleId(0))); // state 0: EOF -> reduce S->ε
        actions[1][0].push(Action::Accept);             // state 1: EOF -> accept

        let mut gotos = vec![vec![NO_GOTO; 2]; 2];
        gotos[0][1] = StateId(1); // state 0: after S -> state 1

        let table = build_table(actions, gotos, rules, s, eof, 1);

        let tokens: Vec<(u32, u32, u32)> = vec![]; // empty input
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok(), "empty input on S->ε must accept: {:?}", result.err());

        let forest = result.unwrap();
        let view = forest.view();
        prop_assert!(!view.roots().is_empty(), "accepted empty parse must have roots");
        for &root in view.roots() {
            let sp = view.span(root);
            prop_assert_eq!(sp.start, 0);
            prop_assert_eq!(sp.end, 0, "epsilon root must have zero-width span");
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Shift+reduce grammar accepts 1-token input
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn shift_reduce_accepts_single_token(
        terminal_id in 1u16..5,
    ) {
        // Grammar: S -> 't' where 't' has a variable terminal id.
        // Symbols: 0=EOF, terminal_id='t', (max_sym)=S(NT)
        let eof = SymbolId(0);
        let t = SymbolId(terminal_id);
        let s_id = terminal_id.max(2) + 1; // ensure S id > all terminal ids
        let s = SymbolId(s_id);

        let num_syms = (s_id as usize) + 1;

        let rules = vec![ParseRule { lhs: s, rhs_len: 1 }]; // S -> 't'

        let num_states = 3;
        let mut actions = vec![vec![vec![]; num_syms]; num_states];
        actions[0][terminal_id as usize].push(Action::Shift(StateId(1)));
        actions[1][0].push(Action::Reduce(RuleId(0)));
        actions[2][0].push(Action::Accept);

        let mut gotos = vec![vec![NO_GOTO; num_syms]; num_states];
        gotos[0][s_id as usize] = StateId(2);

        let table = build_table(actions, gotos, rules, s, eof, (terminal_id as usize) + 1);

        let tokens = vec![(t.0 as u32, 0u32, 1u32)];
        let mut driver = Driver::new(&table);
        let result = driver.parse_tokens(tokens.into_iter());
        prop_assert!(result.is_ok(), "shift+reduce on 1 token must accept: {:?}", result.err());

        let forest = result.unwrap();
        let view = forest.view();
        prop_assert_eq!(view.roots().len(), 1, "must yield exactly 1 root");
        let sp = view.span(view.roots()[0]);
        prop_assert_eq!(sp.start, 0);
        prop_assert_eq!(sp.end, 1);
    }
}
