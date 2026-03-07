#![cfg(feature = "pure-rust")]

use adze::adze_ir as ir;
use adze::decoder::{decode_grammar, decode_parse_table};

use ir::ProductionId;

#[test]
#[ignore = "KNOWN_RED #9: decoder rule mapping for Python grammar"]
fn test_python_decoder_roundtrip() {
    let lang = adze_python::get_language();

    let grammar = decode_grammar(lang);
    assert!(
        !grammar.rules.is_empty(),
        "decoded grammar should have rules"
    );
    assert!(
        !grammar.fields.is_empty(),
        "field names should be populated"
    );

    let table = decode_parse_table(lang);
    assert_eq!(
        table.lex_modes.len(),
        lang.state_count as usize,
        "lex modes length should match state count"
    );
    assert_eq!(table.extras, grammar.extras, "extras should round trip");
    assert_eq!(
        table.field_names.len(),
        grammar.fields.len(),
        "field names should round trip"
    );

    for (i, pr) in table.rules.iter().enumerate() {
        let pid = ProductionId(i as u16);
        let gr = grammar
            .rules
            .get(&pr.lhs)
            .and_then(|v| v.iter().find(|r| r.production_id == pid))
            .expect("rule should exist in grammar");
        assert_eq!(gr.rhs.len(), pr.rhs_len as usize);
    }
}
