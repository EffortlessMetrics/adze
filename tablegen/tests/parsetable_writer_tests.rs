//! Comprehensive tests for the parsetable writer and serializer modules.
//!
//! These tests complement the existing unit tests in `parsetable_writer.rs` and
//! `serializer.rs` by exercising writer creation/configuration, output format,
//! serializer field-ordering edge cases, round-trip metadata, magic-number
//! validation, version format strings, and metadata schema.

#![cfg(feature = "serialization")]

use std::collections::BTreeMap;
use std::io::Read;

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable, StateId};
use adze_ir::{ExternalToken, FieldId, Grammar, RuleId, SymbolId, Token, TokenPattern};
use adze_tablegen::parsetable_writer::{
    FORMAT_VERSION, MAGIC_NUMBER, METADATA_SCHEMA_VERSION, ParsetableMetadata, ParsetableWriter,
};
use adze_tablegen::serializer::{SerializableLanguage, serialize_language};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal ParseTable for integration tests.
fn make_empty_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    let nonterms_eff = if nonterms == 0 { 1 } else { nonterms };
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions = vec![vec![vec![]; symbol_count]; states];
    let gotos = vec![vec![INVALID; symbol_count]; states];

    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let eof_symbol = SymbolId(eof_idx as u16);
    let token_count = eof_idx - externals;

    let mut symbol_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let mut nonterminal_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);

    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (symbol_id, index) in &symbol_to_index {
        index_to_symbol[*index] = *symbol_id;
    }

    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        states
    ];

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![],
        state_count: states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![],
        token_count,
        external_token_count: externals,
        eof_symbol,
        start_symbol,
        initial_state: StateId(0),
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: Grammar::default(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

fn test_grammar() -> Grammar {
    Grammar {
        name: "test_lang".to_string(),
        ..Default::default()
    }
}

fn test_parse_table() -> ParseTable {
    make_empty_table(2, 2, 1, 0)
}

// ---------------------------------------------------------------------------
// 1. Writer creation and configuration
// ---------------------------------------------------------------------------

#[test]
fn test_writer_creation_stores_grammar_name_and_version() {
    let grammar = test_grammar();
    let pt = test_parse_table();

    let writer = ParsetableWriter::new(&grammar, &pt, "my_lang", "2.5.0");
    let meta = writer.metadata();

    assert_eq!(meta.grammar.name, "my_lang");
    assert_eq!(meta.grammar.version, "2.5.0");
    assert_eq!(meta.grammar.language, "test_lang");
}

#[test]
fn test_writer_creation_with_string_args() {
    let grammar = test_grammar();
    let pt = test_parse_table();

    let name = String::from("dynamic_name");
    let version = String::from("0.0.1-alpha");
    let writer = ParsetableWriter::new(&grammar, &pt, name, version);
    let meta = writer.metadata();

    assert_eq!(meta.grammar.name, "dynamic_name");
    assert_eq!(meta.grammar.version, "0.0.1-alpha");
}

// ---------------------------------------------------------------------------
// 2. Writer output format verification
// ---------------------------------------------------------------------------

#[test]
fn test_written_file_header_layout() {
    let grammar = test_grammar();
    let pt = test_parse_table();
    let writer = ParsetableWriter::new(&grammar, &pt, "test", "1.0.0");

    let temp = std::env::temp_dir().join("pw_test_header_layout.parsetable");
    writer.write_file(&temp).expect("write must succeed");

    let data = std::fs::read(&temp).expect("read");
    let _ = std::fs::remove_file(&temp);

    // Minimum: magic(4) + version(4) + hash(32) + meta_len(4) + at least 1 byte metadata
    assert!(data.len() >= 45, "file too small: {}", data.len());

    // magic
    assert_eq!(&data[0..4], b"RSPT");
    // version
    let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
    assert_eq!(version, FORMAT_VERSION);
    // hash is 32 bytes (just check it exists, not all zeroes for a non-empty grammar)
    let hash = &data[8..40];
    assert_eq!(hash.len(), 32);
    // metadata length
    let meta_len = u32::from_le_bytes(data[40..44].try_into().unwrap()) as usize;
    assert!(meta_len > 0, "metadata must not be empty");
    // metadata JSON is parseable
    let meta_json = std::str::from_utf8(&data[44..44 + meta_len]).expect("utf8");
    let _: serde_json::Value = serde_json::from_str(meta_json).expect("valid json");
}

#[test]
fn test_written_file_contains_table_data_after_metadata() {
    let grammar = test_grammar();
    let pt = test_parse_table();
    let writer = ParsetableWriter::new(&grammar, &pt, "test", "1.0.0");

    let temp = std::env::temp_dir().join("pw_test_table_after_meta.parsetable");
    writer.write_file(&temp).expect("write must succeed");

    let data = std::fs::read(&temp).expect("read");
    let _ = std::fs::remove_file(&temp);

    let meta_len = u32::from_le_bytes(data[40..44].try_into().unwrap()) as usize;
    let table_offset = 44 + meta_len;

    // After metadata: table_data_len(4) + table_data(N)
    assert!(
        data.len() > table_offset + 4,
        "file must contain table data section"
    );
    let table_len = u32::from_le_bytes(data[table_offset..table_offset + 4].try_into().unwrap());
    assert!(table_len > 0, "table data must not be empty");
    assert_eq!(
        data.len(),
        table_offset + 4 + table_len as usize,
        "file length must equal header + metadata + table data"
    );
}

// ---------------------------------------------------------------------------
// 3. Serializer creation & basic output
// ---------------------------------------------------------------------------

#[test]
fn test_serialize_language_produces_valid_json() {
    let mut grammar = Grammar::new("json_test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex("[0-9]+".to_string()),
            fragile: false,
        },
    );
    let pt = make_empty_table(1, 1, 0, 0);

    let json = serialize_language(&grammar, &pt, None).expect("serialize");
    let parsed: SerializableLanguage = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(
        parsed.version,
        adze_tablegen::abi::TREE_SITTER_LANGUAGE_VERSION
    );
    assert!(parsed.state_count >= 1);
    assert!(parsed.symbol_names.contains(&"end".to_string()));
    assert!(parsed.symbol_names.contains(&"number".to_string()));
}

// ---------------------------------------------------------------------------
// 4. Serializer field ordering edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_field_ordering_empty_fields() {
    let grammar = Grammar::new("empty_fields".to_string());
    let pt = make_empty_table(1, 0, 0, 0);

    let json = serialize_language(&grammar, &pt, None).expect("serialize");
    let parsed: SerializableLanguage = serde_json::from_str(&json).expect("deserialize");

    assert!(parsed.field_names.is_empty());
    assert_eq!(parsed.field_count, 0);
}

#[test]
fn test_field_ordering_unicode_names() {
    let mut grammar = Grammar::new("unicode_fields".to_string());
    grammar.fields.insert(FieldId(0), "ñ_field".to_string());
    grammar.fields.insert(FieldId(1), "α_field".to_string());
    grammar.fields.insert(FieldId(2), "a_field".to_string());
    let pt = make_empty_table(1, 0, 0, 0);

    let json = serialize_language(&grammar, &pt, None).expect("serialize");
    let parsed: SerializableLanguage = serde_json::from_str(&json).expect("deserialize");

    // Lexicographic: a < ñ < α (by Unicode codepoint)
    assert_eq!(parsed.field_names[0], "a_field");
    assert_eq!(parsed.field_names.len(), 3);
    // All present
    assert!(parsed.field_names.contains(&"ñ_field".to_string()));
    assert!(parsed.field_names.contains(&"α_field".to_string()));
}

#[test]
fn test_field_ordering_single_field() {
    let mut grammar = Grammar::new("single_field".to_string());
    grammar.fields.insert(FieldId(0), "only".to_string());
    let pt = make_empty_table(1, 0, 0, 0);

    let json = serialize_language(&grammar, &pt, None).expect("serialize");
    let parsed: SerializableLanguage = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.field_names, vec!["only"]);
    assert_eq!(parsed.field_count, 1);
}

// ---------------------------------------------------------------------------
// 5. Round-trip metadata write/read
// ---------------------------------------------------------------------------

#[test]
fn test_metadata_round_trip_through_file() {
    let grammar = test_grammar();
    let pt = test_parse_table();
    let writer = ParsetableWriter::new(&grammar, &pt, "roundtrip", "4.2.0");

    let temp = std::env::temp_dir().join("pw_test_roundtrip.parsetable");
    writer.write_file(&temp).expect("write");

    let data = std::fs::read(&temp).expect("read");
    let _ = std::fs::remove_file(&temp);

    let meta_len = u32::from_le_bytes(data[40..44].try_into().unwrap()) as usize;
    let meta_bytes = &data[44..44 + meta_len];

    let recovered = ParsetableMetadata::from_bytes(meta_bytes).expect("parse metadata");

    let original = writer.metadata();
    assert_eq!(recovered.schema_version, original.schema_version);
    assert_eq!(recovered.grammar, original.grammar);
    assert_eq!(recovered.statistics, original.statistics);
    assert_eq!(recovered.features, original.features);
}

// ---------------------------------------------------------------------------
// 6. Magic number validation
// ---------------------------------------------------------------------------

#[test]
fn test_magic_number_byte_values() {
    assert_eq!(MAGIC_NUMBER[0], b'R');
    assert_eq!(MAGIC_NUMBER[1], b'S');
    assert_eq!(MAGIC_NUMBER[2], b'P');
    assert_eq!(MAGIC_NUMBER[3], b'T');
    assert_eq!(MAGIC_NUMBER.len(), 4);
}

#[test]
fn test_magic_number_in_written_file_is_first_4_bytes() {
    let grammar = test_grammar();
    let pt = test_parse_table();
    let writer = ParsetableWriter::new(&grammar, &pt, "magic_check", "1.0.0");

    let temp = std::env::temp_dir().join("pw_test_magic_first.parsetable");
    writer.write_file(&temp).expect("write");

    let mut f = std::fs::File::open(&temp).expect("open");
    let mut buf = [0u8; 4];
    f.read_exact(&mut buf).expect("read");
    let _ = std::fs::remove_file(&temp);

    assert_eq!(buf, MAGIC_NUMBER);
    // Also verify it matches known ASCII
    assert_eq!(&buf, b"RSPT");
}

// ---------------------------------------------------------------------------
// 7. Version format strings
// ---------------------------------------------------------------------------

#[test]
fn test_format_version_encoding_le() {
    let le_bytes = FORMAT_VERSION.to_le_bytes();
    assert_eq!(le_bytes, [1, 0, 0, 0]);
    // Round-trip
    assert_eq!(u32::from_le_bytes(le_bytes), FORMAT_VERSION);
}

#[test]
fn test_schema_version_is_semver_like() {
    // METADATA_SCHEMA_VERSION should be a dotted version string
    let parts: Vec<&str> = METADATA_SCHEMA_VERSION.split('.').collect();
    assert!(
        parts.len() >= 2,
        "schema version '{}' should have at least major.minor",
        METADATA_SCHEMA_VERSION
    );
    for part in &parts {
        part.parse::<u32>()
            .unwrap_or_else(|_| panic!("schema version component '{}' must be numeric", part));
    }
}

// ---------------------------------------------------------------------------
// 8. Metadata schema content
// ---------------------------------------------------------------------------

#[test]
fn test_metadata_schema_has_all_required_fields() {
    let grammar = test_grammar();
    let pt = test_parse_table();
    let writer = ParsetableWriter::new(&grammar, &pt, "schema_check", "1.0.0");

    let meta = writer.metadata();
    let json = serde_json::to_string(meta).expect("serialize");
    let val: serde_json::Value = serde_json::from_str(&json).expect("parse");

    // Top-level required keys
    for key in &[
        "schema_version",
        "grammar",
        "generation",
        "statistics",
        "features",
    ] {
        assert!(val.get(key).is_some(), "missing required key: {}", key);
    }

    // grammar sub-keys
    let g = &val["grammar"];
    for key in &["name", "version", "language"] {
        assert!(g.get(key).is_some(), "grammar missing key: {}", key);
    }

    // generation sub-keys
    let generation = &val["generation"];
    for key in &["timestamp", "tool_version", "rust_version", "host_triple"] {
        assert!(
            generation.get(key).is_some(),
            "generation missing key: {}",
            key
        );
    }

    // statistics sub-keys
    let stats = &val["statistics"];
    for key in &[
        "state_count",
        "symbol_count",
        "rule_count",
        "conflict_count",
        "multi_action_cells",
    ] {
        assert!(stats.get(key).is_some(), "statistics missing key: {}", key);
    }

    // features sub-keys
    let feat = &val["features"];
    for key in &["glr_enabled", "external_scanner", "incremental"] {
        assert!(feat.get(key).is_some(), "features missing key: {}", key);
    }
}

#[test]
fn test_metadata_statistics_reflect_parse_table() {
    let grammar = test_grammar();
    let mut pt = make_empty_table(5, 3, 2, 0);
    // Add a multi-action cell (GLR conflict)
    pt.action_table[0][0] = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
    pt.rules = vec![
        ParseRule {
            lhs: SymbolId(5),
            rhs_len: 1,
        },
        ParseRule {
            lhs: SymbolId(5),
            rhs_len: 2,
        },
    ];

    let writer = ParsetableWriter::new(&grammar, &pt, "stats_test", "1.0.0");
    let meta = writer.metadata();

    assert_eq!(meta.statistics.state_count, 5);
    assert_eq!(meta.statistics.rule_count, 2);
    assert_eq!(meta.statistics.multi_action_cells, 1);
    assert_eq!(meta.statistics.conflict_count, 1);
    assert!(meta.features.glr_enabled);
}

#[test]
fn test_metadata_features_external_scanner_detection() {
    let grammar = test_grammar();
    let mut pt = test_parse_table();

    // No external scanners -> external_scanner should be false
    let writer = ParsetableWriter::new(&grammar, &pt, "no_ext", "1.0.0");
    assert!(!writer.metadata().features.external_scanner);

    // Add external scanner states -> external_scanner should be true
    pt.external_scanner_states = vec![vec![true, false]];
    let writer = ParsetableWriter::new(&grammar, &pt, "with_ext", "1.0.0");
    assert!(writer.metadata().features.external_scanner);
}

// ---------------------------------------------------------------------------
// Serializer: symbol names with externals
// ---------------------------------------------------------------------------

#[test]
fn test_serializer_includes_external_tokens_in_symbol_names() {
    let mut grammar = Grammar::new("ext_test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "ident".to_string(),
            pattern: TokenPattern::String("id".to_string()),
            fragile: false,
        },
    );
    grammar.externals.push(ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(100),
    });
    grammar.externals.push(ExternalToken {
        name: "DEDENT".to_string(),
        symbol_id: SymbolId(101),
    });

    let pt = make_empty_table(1, 1, 0, 2);

    let json = serialize_language(&grammar, &pt, None).expect("serialize");
    let parsed: SerializableLanguage = serde_json::from_str(&json).expect("deserialize");

    assert!(parsed.symbol_names.contains(&"INDENT".to_string()));
    assert!(parsed.symbol_names.contains(&"DEDENT".to_string()));
    assert!(parsed.symbol_names.contains(&"ident".to_string()));
    assert_eq!(parsed.external_token_count, 2);
}

// ---------------------------------------------------------------------------
// Serializer: lex mode generation
// ---------------------------------------------------------------------------

#[test]
fn test_serializer_lex_modes_match_state_count() {
    let grammar = Grammar::new("lex_modes_test".to_string());
    let pt = make_empty_table(7, 1, 1, 0);

    let json = serialize_language(&grammar, &pt, None).expect("serialize");
    let parsed: SerializableLanguage = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(parsed.lex_modes.len(), 7);
    for (i, mode) in parsed.lex_modes.iter().enumerate() {
        assert_eq!(mode.lex_state, i as u16);
        assert_eq!(mode.external_lex_state, 0);
    }
}

// ---------------------------------------------------------------------------
// Serializer: symbol metadata encoding
// ---------------------------------------------------------------------------

#[test]
fn test_serializer_symbol_metadata_length_matches_symbol_count() {
    let mut grammar = Grammar::new("meta_len".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "tok_a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "_hidden".to_string(),
            pattern: TokenPattern::String("_".to_string()),
            fragile: false,
        },
    );
    let pt = make_empty_table(1, 2, 0, 0);

    let json = serialize_language(&grammar, &pt, None).expect("serialize");
    let parsed: SerializableLanguage = serde_json::from_str(&json).expect("deserialize");

    // symbol_metadata length == symbol_names length == calculated symbol count
    assert_eq!(parsed.symbol_metadata.len(), parsed.symbol_names.len());
}

// ---------------------------------------------------------------------------
// Serializer: deterministic output
// ---------------------------------------------------------------------------

#[test]
fn test_serializer_deterministic_across_calls() {
    let mut grammar = Grammar::new("det".to_string());
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "z".to_string(),
            pattern: TokenPattern::String("z".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.fields.insert(FieldId(0), "beta".to_string());
    grammar.fields.insert(FieldId(1), "alpha".to_string());

    let pt = make_empty_table(1, 2, 0, 0);

    let json1 = serialize_language(&grammar, &pt, None).expect("first");
    let json2 = serialize_language(&grammar, &pt, None).expect("second");

    assert_eq!(json1, json2, "serialization must be deterministic");
}
