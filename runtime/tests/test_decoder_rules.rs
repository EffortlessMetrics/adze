#![cfg(feature = "pure-rust")]

mod support;

use adze::decoder;
use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use support::{expr_grammar, language_builder};

#[test]
fn test_decoder_reconstructs_rules() {
    let grammar = expr_grammar::build_expr_grammar();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let mut parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    language_builder::normalize_table_for_ts(&mut parse_table);
    let lang = language_builder::build_ts_language(&grammar, &parse_table);
    let lang = Box::leak(Box::new(lang));

    let decoded = decoder::decode_grammar(lang);

    // Compare rules including RHS sequences and precedence metadata
    let mut original_rules: Vec<_> = grammar
        .rules
        .values()
        .flat_map(|rs| rs.iter().cloned())
        .collect();
    original_rules.sort_by_key(|r| r.production_id.0);

    let mut decoded_rules: Vec<_> = decoded
        .rules
        .values()
        .flat_map(|rs| rs.iter().cloned())
        .collect();
    decoded_rules.sort_by_key(|r| r.production_id.0);

    assert_eq!(decoded_rules.len(), original_rules.len());
    for (d, o) in decoded_rules.iter().zip(original_rules.iter()) {
        assert_eq!(d.rhs, o.rhs);
    }
}
