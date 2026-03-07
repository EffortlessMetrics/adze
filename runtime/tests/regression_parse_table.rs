use adze_glr_core::{Action, ParseRule, StateId};
use adze_ir::SymbolId;
use glr_test_support::{INVALID, make_minimal_table};

/// Regression: ParseTable invariant — EOF column == token_count + external_token_count.
/// See: https://github.com/EffortlessMetrics/adze/issues/89
#[test]
fn issue_89_parse_table_eof_invariant() {
    // Layout: ERROR(0), terminal NUM(1), EOF(2), non-terminal EXPR(3)
    let eof = SymbolId(2);
    let expr = SymbolId(3);

    let actions = vec![
        // state 0: shift NUM
        vec![
            vec![],                          // col 0 ERROR
            vec![Action::Shift(StateId(1))], // col 1 NUM
            vec![],                          // col 2 EOF
            vec![],                          // col 3 EXPR
        ],
        // state 1: accept on EOF
        vec![vec![], vec![], vec![Action::Accept], vec![]],
    ];

    let gotos = vec![
        vec![
            INVALID,
            INVALID,
            INVALID,
            StateId(1), // EXPR goto
        ],
        vec![INVALID, INVALID, INVALID, INVALID],
    ];

    let rules = vec![ParseRule {
        lhs: expr,
        rhs_len: 1,
    }];

    let table = make_minimal_table(
        actions, gotos, rules, expr, eof, /*external_token_count=*/ 0,
    );

    // The invariant: EOF's column index must equal token_count + external_token_count.
    let eof_col = table
        .symbol_to_index
        .get(&table.eof_symbol)
        .expect("EOF must be in symbol_to_index");
    assert_eq!(
        *eof_col,
        table.token_count + table.external_token_count,
        "EOF column must be token_count + external_token_count"
    );
}
