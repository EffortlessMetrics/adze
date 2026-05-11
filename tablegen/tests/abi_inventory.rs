//! ABI inventory canary — verifies the generated Language struct has the
//! expected shape: symbols, tokens, states, fields, actions, compressed tables.

use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use adze_ir::*;
use adze_tablegen::helpers::{collect_token_indices, eof_accepts_or_reduces};
use adze_tablegen::{AbiLanguageBuilder, TableCompressor};

fn build_arithmetic_grammar() -> Grammar {
    let mut g = Grammar::new("arithmetic".into());
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "+".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(3),
        Token {
            name: "*".into(),
            pattern: TokenPattern::String("*".into()),
            fragile: false,
        },
    );
    let (expr, term, fact) = (SymbolId(4), SymbolId(5), SymbolId(6));
    let mk = |lhs, rhs, prec, assoc, pid| Rule {
        lhs,
        rhs,
        precedence: prec,
        associativity: assoc,
        fields: vec![],
        production_id: ProductionId(pid),
    };
    g.add_rule(mk(
        expr,
        vec![
            Symbol::NonTerminal(expr),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(term),
        ],
        Some(PrecedenceKind::Static(1)),
        Some(Associativity::Left),
        0,
    ));
    g.add_rule(mk(expr, vec![Symbol::NonTerminal(term)], None, None, 1));
    g.add_rule(mk(
        term,
        vec![
            Symbol::NonTerminal(term),
            Symbol::Terminal(SymbolId(3)),
            Symbol::NonTerminal(fact),
        ],
        Some(PrecedenceKind::Static(2)),
        Some(Associativity::Left),
        2,
    ));
    g.add_rule(mk(term, vec![Symbol::NonTerminal(fact)], None, None, 3));
    g.add_rule(mk(fact, vec![Symbol::Terminal(SymbolId(1))], None, None, 4));
    g.rule_names.insert(expr, "expression".into());
    g.rule_names.insert(term, "term".into());
    g.rule_names.insert(fact, "factor".into());
    g
}

#[test]
fn generated_language_abi_inventory_matches_contract() {
    let g = build_arithmetic_grammar();
    let pt = build_lr1_automaton(&g, &FirstFollowSets::compute(&g).unwrap()).unwrap();

    // 1. Core counts are non-zero (except external tokens)
    assert!(pt.symbol_count > 0);
    assert!(pt.token_count > 0);
    assert!(pt.state_count > 0);
    assert_eq!(pt.external_token_count, 0);

    // 2. All Shift/Reduce actions reference valid states/rules
    for (si, row) in pt.action_table.iter().enumerate() {
        for cell in row.iter().flatten() {
            if let Action::Shift(s) = cell {
                assert!(
                    (s.0 as usize) < pt.state_count,
                    "state {si}: shift to invalid state {}",
                    s.0
                );
            }
        }
    }
    let rule_count = g.rules.values().flatten().count();
    for (si, row) in pt.action_table.iter().enumerate() {
        for cell in row.iter().flatten() {
            if let Action::Reduce(r) = cell {
                assert!(
                    (r.0 as usize) < rule_count,
                    "state {si}: reduce with invalid rule {}",
                    r.0
                );
            }
        }
    }

    // 3. Field names are non-empty for all declared fields
    for name in g.fields.values() {
        assert!(!name.is_empty(), "field name must be non-empty");
    }

    // 4. Compressed tables validate against original parse table
    let tok_idx = collect_token_indices(&g, &pt);
    let compressed = TableCompressor::new()
        .compress(&pt, &tok_idx, eof_accepts_or_reduces(&pt))
        .expect("compression succeeds");
    compressed
        .validate(&pt)
        .expect("compressed tables validate");

    // 5. Generated code contains expected ABI surface
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    for needle in [
        "symbol_count",
        "token_count",
        "state_count",
        "primary_state_ids",
        "field_count",
        "PARSE_TABLE",
        "LANGUAGE_VERSION",
        "tree_sitter_arithmetic",
    ] {
        assert!(code.contains(needle), "generated code missing '{needle}'");
    }
}
