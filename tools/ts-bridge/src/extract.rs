use crate::ffi::{SafeLang, TsbActionKind};
use crate::schema::*;
use std::collections::BTreeMap;

const MAX_ACTIONS_PER_CELL: usize = 32;

pub fn extract(
    language_fn: unsafe extern "C" fn() -> *const crate::ffi::TSLanguage
) -> anyhow::Result<ParseTableData> {
    SafeLang::assert_abi();
    let lang = SafeLang(unsafe { language_fn() });

    let (symc, stc, tokc, extc) = lang.counts();
    
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
        // Actions for terminals
        for sym in 0..term_boundary {
            if let Some((hdr, idx)) = lang.entry(state, sym) {
                // Dynamically resize buffer if needed for large action cells
                if hdr.count as usize > buf.len() {
                    buf.resize(hdr.count as usize, crate::ffi::TsbAction {
                        kind: TsbActionKind::Accept,
                        state: 0,
                        lhs: 0,
                        rhs_len: 0,
                        dynamic_precedence: 0,
                        production_id: 0,
                        extra: false,
                        repetition: false,
                    });
                }
                
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
                            // Width checks for rule components
                            debug_assert!(a.lhs <= u16::MAX as u16, "lhs {} exceeds u16", a.lhs);
                            debug_assert!(a.rhs_len <= u16::MAX as u16, "rhs_len {} exceeds u16", a.rhs_len);
                            debug_assert!(a.production_id <= u16::MAX as u16, "production_id {} exceeds u16", a.production_id);
                            
                            // allocate or get rule id
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
                    actions: seq 
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
        symbols,
        rules,
        actions,
        gotos,
    })
}