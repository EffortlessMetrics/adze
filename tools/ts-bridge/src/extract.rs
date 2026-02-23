use crate::ffi::{SafeLang, TsbActionKind};
use crate::schema::*;
use std::collections::{BTreeMap, HashSet};

const MAX_ACTIONS_PER_CELL: usize = 32;

pub fn extract(
    language_fn: unsafe extern "C" fn() -> *const crate::ffi::TSLanguage,
) -> anyhow::Result<ParseTableData> {
    SafeLang::assert_abi();
    let lang = SafeLang(unsafe { language_fn() });

    let (symc, stc, tokc, extc, lstc) = lang.counts();

    // Width checks to ensure values fit in u16
    debug_assert!(symc <= u16::MAX as u32, "symbol count {} exceeds u16", symc);
    debug_assert!(stc <= u16::MAX as u32, "state count {} exceeds u16", stc);
    debug_assert!(tokc <= u16::MAX as u32, "token count {} exceeds u16", tokc);
    let term_boundary = tokc + extc;

    // Symbol metadata (names, visibility, etc.)
    let mut symbols = Vec::with_capacity(symc as usize);
    for s in 0..symc {
        let meta = lang.symbol_metadata(s);
        symbols.push(Symbol {
            name: lang.symbol_name(s),
            visible: meta.visible,
            named: meta.named,
        });
    }

    // collect actions and rules
    let mut actions = Vec::<ActionCell>::new();
    let mut gotos = Vec::<GotoCell>::new();

    // (lhs,rhs_len,prod_id) -> rule_id
    let mut rule_ids: BTreeMap<(u16, u16, u16), u16> = BTreeMap::new();

    // First pass: scan terminals
    // Start with a reasonable buffer, but expand dynamically if needed
    let mut buf = vec![
        crate::ffi::TsbAction {
            kind: TsbActionKind::Accept,
            state: 0,
            lhs: 0,
            rhs_len: 0,
            dynamic_precedence: 0,
            production_id: 0,
            extra: false,
            repetition: false,
        };
        MAX_ACTIONS_PER_CELL
    ];

    for state in 0..stc {
        if state < 10 || state == 30 {}
        // Actions for terminals
        for sym in 0..term_boundary {
            if let Some((hdr, idx)) = lang.entry(state, sym) {
                // ... (rest of buffer logic)
                let n = lang.unpack(idx, hdr.count, &mut buf);
                if n == 0 {
                    continue;
                }

                let mut seq = Vec::with_capacity(n);
                for a in &buf[..n] {
                    if state < 10 || state == 30 {
                        println!("  sym {}: {:?} (idx {})", sym, a, idx);
                    }
                    if a.kind == TsbActionKind::Accept {}
                    match a.kind {
                        TsbActionKind::Shift => {
                            seq.push(Action::Shift {
                                state: a.state,
                                extra: a.extra,
                                rep: a.repetition,
                            });
                        }
                        TsbActionKind::Reduce => {
                            // ... (rest of reduce logic)
                            let key = (a.lhs, a.rhs_len, a.production_id);
                            let next_id = rule_ids.len() as u16;
                            let rid = *rule_ids.entry(key).or_insert(next_id);
                            seq.push(Action::Reduce {
                                rule: rid,
                                dyn_prec: a.dynamic_precedence,
                            });
                        }
                        TsbActionKind::Accept => seq.push(Action::Accept),
                        TsbActionKind::Recover => seq.push(Action::Recover),
                    }
                }
                actions.push(ActionCell {
                    state: state as u16,
                    symbol: sym as u16,
                    actions: seq,
                });
            }
        }

        // Gotos for nonterminals
        for a in term_boundary..symc {
            let ns = lang.next_state(state, a);
            if ns != 0 {
                gotos.push(GotoCell {
                    state: state as u16,
                    symbol: a as u16,
                    next_state: Some(ns as u16),
                });
            }
        }
    }

    // finalize stable rules in index order
    let mut rules = vec![
        Rule {
            lhs: 0,
            rhs_len: 0,
            production_id: 0
        };
        rule_ids.len()
    ];
    for (k, v) in rule_ids {
        rules[v as usize] = Rule {
            lhs: k.0,
            rhs_len: k.1,
            production_id: k.2,
        };
    }

    // In Tree-sitter: ts_builtin_sym_end = 0, ts_builtin_sym_error = -1
    // Allocate a new symbol ID just past Tree-sitter's symbol space for EOF
    let eof_symbol: u16 = symc as u16;

    // The total symbol count now includes our synthetic EOF sentinel
    let symbol_count = eof_symbol as u32 + 1;

    // Copy Tree-sitter's symbol 0 (ts_builtin_sym_end) actions to our EOF sentinel
    // This ensures the driver's EOF phase sees the right accept/reduce actions
    let ts_end_sym = 0u16; // Tree-sitter's builtin end-of-input symbol
    if eof_symbol != ts_end_sym {
        // Find all action cells for symbol 0 and duplicate them for our EOF
        let mut eof_actions = Vec::new();
        for cell in &actions {
            if cell.symbol == ts_end_sym {
                eof_actions.push(ActionCell {
                    state: cell.state,
                    symbol: eof_symbol,
                    actions: cell.actions.clone(),
                });
            }
        }
        actions.extend(eof_actions);

        // Ensure EOF column exists in every state (defensive check)
        #[cfg(debug_assertions)]
        {
            let states_with_eof: HashSet<u16> = actions
                .iter()
                .filter(|c| c.symbol == eof_symbol)
                .map(|c| c.state)
                .collect();
            let states_with_ts_end: HashSet<u16> = actions
                .iter()
                .filter(|c| c.symbol == ts_end_sym)
                .map(|c| c.state)
                .collect();
            debug_assert_eq!(
                states_with_eof, states_with_ts_end,
                "EOF column must exist in exactly the same states as TS end column"
            );
        }
    }

    // Derive start symbol: nonterminal from state 0 whose next state accepts EOF
    let term_boundary = tokc + extc;
    let accept_states: HashSet<u16> = actions
        .iter()
        .filter(|c| c.symbol == eof_symbol && c.actions.iter().any(|a| matches!(a, Action::Accept)))
        .map(|c| c.state)
        .collect();

    let mut start_symbol = 0u16;
    for g in &gotos {
        if let Some(ns) = g.next_state {
            let is_accept = accept_states.contains(&ns);
            if is_accept {
                // We found a transition to an accept state.
                // In Tree-sitter, the start symbol is the nonterminal that
                // leads to Accept from the initial state (usually state 0 or 1).
                // We prefer smaller state numbers as they are more likely to be initial.
                if start_symbol == 0 || g.state < 2 {
                    start_symbol = g.symbol;
                    if g.state <= 1 {
                        break;
                    } // Good enough
                }
            }
        }
    }

    Ok(ParseTableData {
        version: 1,
        ts_language_version: 15,
        symbol_count,
        state_count: stc,
        token_count: tokc,
        external_token_count: extc,
        eof_symbol,
        start_symbol,
        symbols,
        rules,
        actions,
        gotos,
    })
}
