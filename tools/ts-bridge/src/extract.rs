use crate::ffi::{SafeLang, TsbActionKind};
use crate::schema::*;
use std::collections::BTreeMap;

pub fn extract(
    language_fn: unsafe extern "C" fn() -> *const crate::ffi::TSLanguage
) -> anyhow::Result<ParseTableData> {
    SafeLang::assert_abi();
    let lang = SafeLang(unsafe { language_fn() });

    let (symc, stc, tokc, extc) = lang.counts();
    let term_boundary = tokc + extc;

    // names
    let mut names = Vec::with_capacity(symc as usize);
    for s in 0..symc { 
        names.push(lang.symbol_name(s)); 
    }

    // collect actions and rules
    let mut actions = Vec::<ActionCell>::new();
    let mut gotos = Vec::<GotoCell>::new();

    // (lhs,rhs_len,prod_id) -> rule_id
    let mut rule_ids: BTreeMap<(u16, u16, u16), u16> = BTreeMap::new();

    // First pass: scan terminals
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
        64
    ];

    for state in 0..stc {
        // Actions for terminals
        for sym in 0..term_boundary {
            if let Some((hdr, idx)) = lang.entry(state, sym) {
                let n = lang.unpack(idx, hdr.count, &mut buf);
                if n == 0 { 
                    continue; 
                }

                let mut seq = Vec::with_capacity(n);
                for a in &buf[..n] {
                    match a.kind {
                        TsbActionKind::Shift => {
                            seq.push(Action::Shift { 
                                state: a.state, 
                                extra: a.extra, 
                                rep: a.repetition 
                            });
                        }
                        TsbActionKind::Reduce => {
                            // allocate or get rule id
                            let key = (a.lhs, a.rhs_len, a.production_id);
                            let next_id = rule_ids.len() as u16;
                            let rid = *rule_ids.entry(key).or_insert(next_id);
                            seq.push(Action::Reduce { 
                                rule: rid, 
                                dyn_prec: a.dynamic_precedence, 
                                prod: a.production_id 
                            });
                        }
                        TsbActionKind::Accept => seq.push(Action::Accept),
                        TsbActionKind::Recover => seq.push(Action::Recover),
                    }
                }
                actions.push(ActionCell { 
                    state, 
                    terminal: sym as u16, 
                    seq 
                });
            }
        }
        
        // Gotos for nonterminals
        for a in term_boundary..symc {
            let ns = lang.next_state(state, a);
            if ns != 0 {
                gotos.push(GotoCell { 
                    state, 
                    nonterminal: a as u16, 
                    next_state: ns 
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
            production_id: k.2 
        };
    }

    let start_symbol = lang.detect_start_symbol() as u16;
    debug_assert!(start_symbol != 0, "start symbol shouldn't be EOF");
    let eof_symbol: u16 = 0;

    Ok(ParseTableData {
        version: 1,
        ts_language_version: 15,
        symbol_count: symc,
        state_count: stc,
        token_count: tokc,
        external_token_count: extc,
        eof_symbol,
        start_symbol,
        symbol_names: names,
        rules,
        actions,
        gotos,
    })
}