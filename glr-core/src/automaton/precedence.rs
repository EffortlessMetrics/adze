use adze_ir::*;
use std::collections::BTreeMap;

/// Internal EOF sentinel used by FirstFollowSets.
/// This is NOT the actual EOF symbol - use `parse_table.eof_symbol` for that.
const EOF_SENTINEL: SymbolId = SymbolId(0);

/// Map a symbol from FOLLOW set output to actual parse table symbol.
/// Replaces the EOF sentinel (SymbolId(0)) with the actual EOF symbol.
#[inline]
pub(super) fn map_follow_symbol(sym: SymbolId, eof_symbol: SymbolId) -> SymbolId {
    if sym == EOF_SENTINEL { eof_symbol } else { sym }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Assoc {
    Left,
    Right,
    None,
}

#[derive(Copy, Clone, Debug)]
struct TokPrec {
    prec: u8,
    assoc: Assoc,
}

#[derive(Copy, Clone, Debug)]
struct RulePrec {
    prec: u8,
    assoc: Assoc,
}

pub(super) struct PrecTables {
    // table-indexed; entries 0..token_count-1 may be Some(..); others None
    tok_prec_by_index: Vec<Option<TokPrec>>,
    // production_id -> precedence and associativity
    rule_prec: Vec<RulePrec>,
}

pub(super) fn build_prec_tables(
    grammar: &Grammar,
    symbol_to_index: &BTreeMap<SymbolId, usize>,
    token_count: u32,
    production_count: u32,
) -> PrecTables {
    use adze_ir::{Associativity, PrecedenceKind};

    debug_assert!(production_count > 0, "production_count must be positive");

    let mut tok_prec_by_index = vec![None; symbol_to_index.len()];
    let tok_prec_len = tok_prec_by_index.len();

    let mut set_tok_prec = |tok_idx: usize, new: TokPrec| {
        if tok_idx >= tok_prec_by_index.len() {
            return;
        }
        tok_prec_by_index[tok_idx] = match tok_prec_by_index[tok_idx] {
            None => Some(new),
            Some(old) => Some(if new.prec > old.prec { new } else { old }),
        };
    };

    let mut rule_prec = vec![
        RulePrec {
            prec: 0,
            assoc: Assoc::None,
        };
        production_count as usize
    ];

    for rules in grammar.rules.values() {
        for rule in rules {
            let pid = rule.production_id.0 as usize;
            if pid >= production_count as usize {
                continue;
            }

            let explicit = rule.precedence.and_then(|p| {
                if let PrecedenceKind::Static(level) = p {
                    Some(level as u8)
                } else {
                    None
                }
            });

            let rule_assoc = rule
                .associativity
                .map(|assoc| match assoc {
                    Associativity::Left => Assoc::Left,
                    Associativity::Right => Assoc::Right,
                    Associativity::None => Assoc::None,
                })
                .unwrap_or(Assoc::None);

            if let Some(level) = explicit {
                let tok_idx_opt = rule.rhs.iter().rev().find_map(|sym| {
                    if let Symbol::Terminal(id) = sym {
                        symbol_to_index.get(id).copied()
                    } else {
                        None
                    }
                });

                if let Some(tok_idx) = tok_idx_opt
                    && tok_idx < tok_prec_len
                {
                    set_tok_prec(
                        tok_idx,
                        TokPrec {
                            prec: level,
                            assoc: rule_assoc,
                        },
                    );
                }
            }

            rule_prec[pid] = RulePrec {
                prec: explicit.unwrap_or(0),
                assoc: rule_assoc,
            };
        }
    }

    for rules in grammar.rules.values() {
        for rule in rules {
            let pid = rule.production_id.0 as usize;
            if pid >= production_count as usize {
                continue;
            }

            if rule_prec[pid].prec > 0 {
                continue;
            }

            let derived = rule
                .rhs
                .iter()
                .rev()
                .find_map(|sym| {
                    if let Symbol::Terminal(id) = sym {
                        symbol_to_index.get(id).and_then(|&idx| {
                            if (idx as u32) < token_count {
                                tok_prec_by_index[idx]
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    }
                })
                .unwrap_or(TokPrec {
                    prec: 0,
                    assoc: Assoc::None,
                });

            rule_prec[pid] = RulePrec {
                prec: derived.prec,
                assoc: derived.assoc,
            };
        }
    }

    PrecTables {
        tok_prec_by_index,
        rule_prec,
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(super) enum PrecDecision {
    PreferShift,
    PreferReduce,
    Error,
    NoInfo,
}

#[inline]
pub(super) fn decide_with_precedence(
    lookahead_tok_idx: usize,
    reduce_prod_id: u16,
    prec: &PrecTables,
) -> PrecDecision {
    if reduce_prod_id as usize >= prec.rule_prec.len() {
        return PrecDecision::NoInfo;
    }

    let tokp = match prec
        .tok_prec_by_index
        .get(lookahead_tok_idx)
        .and_then(|o| *o)
    {
        Some(p) => p,
        None => return PrecDecision::NoInfo,
    };
    let rulep = prec.rule_prec[reduce_prod_id as usize];

    if tokp.prec == 0 || rulep.prec == 0 {
        return PrecDecision::NoInfo;
    }

    use core::cmp::Ordering::*;
    match (tokp.prec.cmp(&rulep.prec), rulep.assoc) {
        (Greater, _) => PrecDecision::PreferShift,
        (Less, _) => PrecDecision::PreferReduce,
        (Equal, Assoc::Left) => PrecDecision::PreferReduce,
        (Equal, Assoc::Right) => PrecDecision::PreferShift,
        (Equal, Assoc::None) => PrecDecision::Error,
    }
}

#[inline]
pub(super) fn decide_reduce_reduce(a: u16, b: u16, prec: &PrecTables) -> u16 {
    let pa = prec.rule_prec.get(a as usize).map(|r| r.prec).unwrap_or(0);
    let pb = prec.rule_prec.get(b as usize).map(|r| r.prec).unwrap_or(0);
    if pa > pb {
        a
    } else if pb > pa {
        b
    } else {
        a.min(b)
    }
}
