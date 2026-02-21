#![cfg(feature = "pure-rust")]

mod support;

use adze::decoder;
use adze::ts_format::TSActionTag;
use adze_glr_core::{Action, FirstFollowSets, build_lr1_automaton};

#[test]
fn ts_action_tags_match_ts_runtime() {
    assert_eq!(TSActionTag::Error as u8, 0);
    assert_eq!(TSActionTag::Shift as u8, 1);
    assert_eq!(TSActionTag::Reduce as u8, 3);
    assert_eq!(TSActionTag::Accept as u8, 4);
}

#[test]
fn dense_mapping_and_token_boundary_hold() {
    let g = support::json_grammar::build_json_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let mut t = build_lr1_automaton(&g, &ff).unwrap();
    support::language_builder::normalize_table_for_ts(&mut t);
    let tcols = t.token_count + t.external_token_count;

    // Dense mapping: symbol_to_index[index_to_symbol[i]] == i
    for (i, &sym) in t.index_to_symbol.iter().enumerate() {
        assert_eq!(
            t.symbol_to_index[&sym], i,
            "Non-dense mapping at column {}",
            i
        );
    }

    // Token boundary: tokens in [0..tcols), NTs in [tcols..)
    for col in 0..tcols.min(t.index_to_symbol.len()) {
        assert!(
            t.index_to_symbol[col].0 != 65535,
            "Sentinel leaked into token space at column {}",
            col
        );
        // Heuristic: tokens shouldn't be rule LHS
        assert!(
            !t.rules.iter().any(|r| r.lhs == t.index_to_symbol[col]),
            "Token at column {} is used as rule LHS",
            col
        );
    }
}

#[test]
fn decoded_table_has_accept_and_same_sizes() {
    let g = support::json_grammar::build_json_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let mut t = build_lr1_automaton(&g, &ff).unwrap();
    support::language_builder::normalize_table_for_ts(&mut t);

    let lang = support::language_builder::build_json_ts_language(&g, &t);
    let lang = Box::leak(Box::new(lang));
    let dt = decoder::decode_parse_table(lang);

    // Verify sizes match
    assert_eq!(dt.state_count, t.state_count, "State count mismatch");
    assert_eq!(
        dt.symbol_count,
        t.index_to_symbol.len(),
        "Symbol count mismatch"
    );

    // Verify Accept exists on EOF
    let eof = dt.eof_symbol;
    let eof_col = *dt
        .symbol_to_index
        .get(&eof)
        .expect("EOF must be in symbol_to_index");
    let has_accept = dt
        .action_table
        .iter()
        .any(|row| eof_col < row.len() && row[eof_col].iter().any(|a| matches!(a, Action::Accept)));
    assert!(has_accept, "Decoded table missing Accept on EOF");
}

#[test]
fn normalize_perf_smoke() {
    use std::time::Instant;
    let g = support::json_grammar::build_json_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let mut t = build_lr1_automaton(&g, &ff).unwrap();

    let t0 = Instant::now();
    support::language_builder::normalize_table_for_ts(&mut t);
    let dt = t0.elapsed();

    eprintln!(
        "normalize_table_for_ts: {:?} for {} states × {} symbols",
        dt,
        t.state_count,
        t.index_to_symbol.len()
    );
    assert!(dt.as_millis() < 250, "Normalization took {:?}", dt);
}

#[test]
fn expr_round_trip_accepts() {
    let grammar = support::expr_grammar::build_expr_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let mut table = build_lr1_automaton(&grammar, &ff).expect("build");
    support::language_builder::normalize_table_for_ts(&mut table);

    // Build a simple lexer for expression grammar
    let lang = support::language_builder::build_ts_language(&grammar, &table);
    let lang = Box::leak(Box::new(lang));
    let decoded = decoder::decode_parse_table(lang);

    // Verify the table has Accept action
    let eof = decoded.eof_symbol;
    let eof_col = *decoded
        .symbol_to_index
        .get(&eof)
        .expect("EOF must be in symbol_to_index");
    let has_accept = decoded
        .action_table
        .iter()
        .any(|row| eof_col < row.len() && row[eof_col].iter().any(|a| matches!(a, Action::Accept)));
    assert!(has_accept, "Expression grammar missing Accept on EOF");

    // Verify left-recursive rules are present
    assert!(
        decoded.rules.len() >= 4,
        "Expected at least 4 rules for expr grammar"
    );

    // Verify NT gotos work
    let tcols = decoded.token_count + decoded.external_token_count;
    let nt_cols: Vec<_> = (tcols..decoded.symbol_count).collect();
    assert!(!nt_cols.is_empty(), "No nonterminal columns found");

    // Check that some NT column has a Shift action (goto)
    let has_nt_goto = nt_cols.iter().any(|&col| {
        decoded
            .action_table
            .iter()
            .any(|row| col < row.len() && row[col].iter().any(|a| matches!(a, Action::Shift(_))))
    });
    assert!(has_nt_goto, "No NT gotos found in action table");
}
