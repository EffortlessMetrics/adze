#![cfg(all(feature = "pure-rust", feature = "glr", feature = "ts-compat"))]

use adze::decoder::decode_parse_table;
use adze::pure_parser::{ExternalScanner, TSLanguage, TSLexState, TSParseAction, TSRule};
use adze_glr_core::Action;
use adze_ir::{RuleId, StateId, SymbolId};
use std::ptr;

static SYMBOL_NAME_END: &[u8] = b"end\0";
static SYMBOL_NAME_NUM: &[u8] = b"num\0";
static SYMBOL_NAME_START: &[u8] = b"start\0";
static FIELD_NAME_VALUE: &[u8] = b"value\0";

#[repr(transparent)]
struct SymbolNames([*const u8; 3]);
unsafe impl Sync for SymbolNames {}

static SYMBOL_NAMES: SymbolNames = SymbolNames([
    SYMBOL_NAME_END.as_ptr(),
    SYMBOL_NAME_NUM.as_ptr(),
    SYMBOL_NAME_START.as_ptr(),
]);

#[repr(transparent)]
struct FieldNames([*const u8; 1]);
unsafe impl Sync for FieldNames {}

static FIELD_NAMES: FieldNames = FieldNames([FIELD_NAME_VALUE.as_ptr()]);

static SYMBOL_METADATA: [u8; 3] = [
    0x00, // EOF
    0x03, // num: visible + named terminal
    0x02, // start: named nonterminal
];

// All states use Tree-sitter small-table rows:
// state 0: num shifts to state 1, start goto state 2
// state 1: EOF reduces production 0
// state 2: EOF accepts
static SMALL_PARSE_TABLE: [u16; 8] = [
    1, 1, // symbol 1 => Shift(1)
    2, 2, // symbol 2 => goto StateId(2)
    0, 0x8001, // EOF => Reduce(RuleId(0))
    0, 0xFFFF, // EOF => Accept
];

static SMALL_PARSE_TABLE_MAP: [u32; 4] = [0, 4, 6, 8];
static LEX_MODES: [TSLexState; 3] = [TSLexState {
    lex_state: 0,
    external_lex_state: 0,
}; 3];
static PUBLIC_SYMBOL_MAP: [u16; 3] = [0, 1, 2];
static PRIMARY_STATE_IDS: [u16; 3] = [0, 1, 2];
static PRODUCTION_ID_MAP: [u16; 1] = [0];
static PRODUCTION_LHS_INDEX: [u16; 1] = [2];
static TS_RULES: [TSRule; 1] = [TSRule {
    lhs: 2,
    rhs_len: 1,
    _pad: 0,
}];
static ALIAS_MAP: [u16; 1] = [0];
static ALIAS_SEQUENCES: [u16; 1] = [1];
static FIELD_MAP_SLICES: [u16; 2] = [0, 1];
static FIELD_MAP_ENTRIES: [u16; 2] = [0, 0];
static PARSE_ACTIONS: [TSParseAction; 1] = [TSParseAction {
    action_type: 3,
    extra: 0,
    child_count: 1,
    dynamic_precedence: 0,
    symbol: 0,
}];

static LANGUAGE: TSLanguage = TSLanguage {
    version: 15,
    symbol_count: 3,
    alias_count: 0,
    token_count: 2,
    external_token_count: 0,
    state_count: 3,
    large_state_count: 0,
    production_id_count: 1,
    field_count: 1,
    max_alias_sequence_length: 1,
    production_id_map: PRODUCTION_ID_MAP.as_ptr(),
    parse_table: ptr::null(),
    small_parse_table: SMALL_PARSE_TABLE.as_ptr(),
    small_parse_table_map: SMALL_PARSE_TABLE_MAP.as_ptr(),
    parse_actions: PARSE_ACTIONS.as_ptr(),
    symbol_names: SYMBOL_NAMES.0.as_ptr(),
    field_names: FIELD_NAMES.0.as_ptr(),
    field_map_slices: FIELD_MAP_SLICES.as_ptr(),
    field_map_entries: FIELD_MAP_ENTRIES.as_ptr(),
    symbol_metadata: SYMBOL_METADATA.as_ptr(),
    public_symbol_map: PUBLIC_SYMBOL_MAP.as_ptr(),
    alias_map: ALIAS_MAP.as_ptr(),
    alias_sequences: ALIAS_SEQUENCES.as_ptr(),
    lex_modes: LEX_MODES.as_ptr(),
    lex_fn: None,
    keyword_lex_fn: None,
    keyword_capture_token: 0,
    external_scanner: ExternalScanner {
        states: ptr::null(),
        symbol_map: ptr::null(),
        create: None,
        destroy: None,
        scan: None,
        serialize: None,
        deserialize: None,
    },
    primary_state_ids: PRIMARY_STATE_IDS.as_ptr(),
    production_lhs_index: PRODUCTION_LHS_INDEX.as_ptr(),
    production_count: 1,
    eof_symbol: 0,
    rules: TS_RULES.as_ptr(),
    rule_count: 1,
};

#[test]
fn compressed_tslanguage_decode_preserves_metadata_actions_and_fields() {
    let decoded = decode_parse_table(&LANGUAGE);

    assert_eq!(decoded.symbol_count, LANGUAGE.symbol_count as usize);
    assert_eq!(decoded.token_count, LANGUAGE.token_count as usize);
    assert_eq!(
        decoded.external_token_count,
        LANGUAGE.external_token_count as usize
    );
    assert_eq!(decoded.state_count, LANGUAGE.state_count as usize);
    assert_eq!(decoded.eof_symbol, SymbolId(0));
    assert_eq!(
        decoded.index_to_symbol,
        vec![SymbolId(0), SymbolId(1), SymbolId(2)]
    );
    assert_eq!(decoded.symbol_to_index.get(&SymbolId(2)), Some(&2));

    assert_eq!(decoded.goto_table[0][2], StateId(2));
    assert_eq!(decoded.action_table[0][1], vec![Action::Shift(StateId(1))]);
    assert_eq!(decoded.action_table[1][0], vec![Action::Reduce(RuleId(0))]);
    assert_eq!(decoded.action_table[2][0], vec![Action::Accept]);

    assert_eq!(decoded.rules.len(), 1);
    assert_eq!(decoded.rules[0].lhs, SymbolId(2));
    assert_eq!(decoded.rules[0].rhs_len, 1);
    assert_eq!(decoded.field_names, vec!["value".to_string()]);
    assert_eq!(decoded.field_map.get(&(RuleId(0), 0)), Some(&0));
}
