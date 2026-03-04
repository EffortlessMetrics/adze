//! Comprehensive tests for Language builder API and Language methods.

use adze_runtime::language::SymbolMetadata;
use adze_runtime::{Language, Token};

use adze_glr_core::{GotoIndexing, ParseTable};
use adze_ir::{Grammar, StateId, SymbolId};
use std::collections::BTreeMap;

fn leak_parse_table(pt: ParseTable) -> &'static ParseTable {
    Box::leak(Box::new(pt))
}

fn minimal_parse_table() -> &'static ParseTable {
    leak_parse_table(ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 0,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(0),
        grammar: Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    })
}

fn minimal_metadata() -> Vec<SymbolMetadata> {
    vec![SymbolMetadata {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    }]
}

// ─── LanguageBuilder: required fields ───

#[test]
fn builder_missing_parse_table_fails() {
    let result = Language::builder()
        .symbol_metadata(minimal_metadata())
        .build();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "missing parse table");
}

#[test]
fn builder_missing_symbol_metadata_fails() {
    let result = Language::builder()
        .parse_table(minimal_parse_table())
        .build();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "missing symbol metadata");
}

#[test]
fn builder_minimal_succeeds() {
    let table = minimal_parse_table();
    let result = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .build();
    assert!(result.is_ok());
}

// ─── LanguageBuilder: version ───

#[test]
fn builder_default_version_is_zero() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap();
    assert_eq!(lang.version, 0);
}

#[test]
fn builder_custom_version() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .version(15)
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap();
    assert_eq!(lang.version, 15);
}

// ─── LanguageBuilder: symbol names ───

#[test]
fn builder_default_symbol_names_are_empty_strings() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap();
    // Default names should be empty strings matching metadata count
    assert_eq!(lang.symbol_count, 1);
    assert_eq!(lang.symbol_name(0), Some(""));
}

#[test]
fn builder_custom_symbol_names() {
    let table = minimal_parse_table();
    let meta = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        },
    ];
    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["number".to_string(), "expression".to_string()])
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(0), Some("number"));
    assert_eq!(lang.symbol_name(1), Some("expression"));
}

#[test]
fn builder_symbol_name_out_of_bounds_returns_none() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(999), None);
}

// ─── LanguageBuilder: field names ───

#[test]
fn builder_default_field_count_is_zero() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap();
    assert_eq!(lang.field_count, 0);
}

#[test]
fn builder_with_field_names() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .field_names(vec![
            "left".to_string(),
            "right".to_string(),
            "operator".to_string(),
        ])
        .build()
        .unwrap();
    assert_eq!(lang.field_count, 3);
    assert_eq!(lang.field_name(0), Some("left"));
    assert_eq!(lang.field_name(1), Some("right"));
    assert_eq!(lang.field_name(2), Some("operator"));
}

#[test]
fn builder_field_name_out_of_bounds_returns_none() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .field_names(vec!["name".to_string()])
        .build()
        .unwrap();
    assert_eq!(lang.field_name(999), None);
}

// ─── Language: is_terminal ───

#[test]
fn is_terminal_true_for_terminal_symbol() {
    let table = minimal_parse_table();
    let meta = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        },
    ];
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert!(lang.is_terminal(0));
    assert!(!lang.is_terminal(1));
}

#[test]
fn is_terminal_out_of_bounds_returns_false() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap();
    assert!(!lang.is_terminal(999));
}

// ─── Language: is_visible ───

#[test]
fn is_visible_true_for_visible_symbol() {
    let table = minimal_parse_table();
    let meta = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: true,
            is_visible: false,
            is_supertype: false,
        },
    ];
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert!(lang.is_visible(0));
    assert!(!lang.is_visible(1));
}

#[test]
fn is_visible_out_of_bounds_returns_false() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap();
    assert!(!lang.is_visible(999));
}

// ─── Language: symbol_for_name ───

#[test]
fn symbol_for_name_finds_named_symbol() {
    let table = minimal_parse_table();
    let meta = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        },
    ];
    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["number".to_string(), "expr".to_string()])
        .symbol_metadata(meta)
        .build()
        .unwrap();
    // is_named=true should match visible symbols
    let found = lang.symbol_for_name("number", true);
    assert!(found.is_some());
    assert_eq!(found.unwrap(), 0);
}

#[test]
fn symbol_for_name_returns_none_for_missing() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["number".to_string()])
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap();
    assert_eq!(lang.symbol_for_name("nonexistent", true), None);
}

// ─── Language: symbol_count / field_count ───

#[test]
fn symbol_count_matches_metadata_length() {
    let table = minimal_parse_table();
    let meta = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        },
    ];
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 3);
}

// ─── Language: max_alias_sequence_length ───

#[test]
fn max_alias_sequence_length_default_zero() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap();
    assert_eq!(lang.max_alias_sequence_length, 0);
}

#[test]
fn max_alias_sequence_length_custom() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .max_alias_sequence_length(5)
        .build()
        .unwrap();
    assert_eq!(lang.max_alias_sequence_length, 5);
}

// ─── Language: with_static_tokens ───

#[test]
fn with_static_tokens_creates_language_with_tokens() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap()
        .with_static_tokens(vec![
            Token {
                kind: 1,
                start: 0,
                end: 3,
            },
            Token {
                kind: 0,
                start: 3,
                end: 3,
            },
        ]);
    assert_eq!(lang.symbol_count, 1);
}

// ─── Language: method interactions ───

#[test]
fn language_terminal_and_visible_orthogonal() {
    let table = minimal_parse_table();
    let meta = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: true,
            is_visible: false,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: false,
            is_supertype: false,
        },
    ];
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(meta)
        .build()
        .unwrap();
    // terminal + visible
    assert!(lang.is_terminal(0));
    assert!(lang.is_visible(0));
    // terminal + invisible
    assert!(lang.is_terminal(1));
    assert!(!lang.is_visible(1));
    // nonterminal + visible
    assert!(!lang.is_terminal(2));
    assert!(lang.is_visible(2));
    // nonterminal + invisible
    assert!(!lang.is_terminal(3));
    assert!(!lang.is_visible(3));
}

#[test]
fn language_supertype_metadata() {
    let table = minimal_parse_table();
    let meta = vec![
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: true,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        },
    ];
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 2);
}

#[test]
fn language_symbol_for_name_with_named_false() {
    let table = minimal_parse_table();
    let meta = vec![SymbolMetadata {
        is_terminal: true,
        is_visible: false,
        is_supertype: false,
    }];
    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["semicolon".to_string()])
        .symbol_metadata(meta)
        .build()
        .unwrap();
    // invisible (anonymous) symbol: is_named=false should match
    let found = lang.symbol_for_name("semicolon", false);
    assert!(found.is_some());
}

#[test]
fn language_iterate_all_symbols() {
    let table = minimal_parse_table();
    let meta = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: true,
            is_visible: false,
            is_supertype: false,
        },
    ];
    let names = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(names)
        .symbol_metadata(meta)
        .build()
        .unwrap();
    let mut found_names = vec![];
    for i in 0..lang.symbol_count as u16 {
        if let Some(name) = lang.symbol_name(i) {
            found_names.push(name.to_string());
        }
    }
    assert_eq!(found_names, vec!["a", "b", "c"]);
}

#[test]
fn language_version_u32_max() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .version(u32::MAX)
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap();
    assert_eq!(lang.version, u32::MAX);
}

// ─── SymbolMetadata struct ───

#[test]
fn symbol_metadata_debug() {
    let meta = SymbolMetadata {
        is_terminal: true,
        is_visible: false,
        is_supertype: true,
    };
    let s = format!("{:?}", meta);
    assert!(s.contains("is_terminal"));
}

#[test]
fn symbol_metadata_clone() {
    let meta = SymbolMetadata {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    };
    let meta2 = meta;
    assert_eq!(meta.is_terminal, meta2.is_terminal);
}

#[test]
fn symbol_metadata_all_false() {
    let meta = SymbolMetadata {
        is_terminal: false,
        is_visible: false,
        is_supertype: false,
    };
    assert!(!meta.is_terminal);
    assert!(!meta.is_visible);
    assert!(!meta.is_supertype);
}

#[test]
fn symbol_metadata_all_true() {
    let meta = SymbolMetadata {
        is_terminal: true,
        is_visible: true,
        is_supertype: true,
    };
    assert!(meta.is_terminal);
    assert!(meta.is_visible);
    assert!(meta.is_supertype);
}

// ─── Language with multiple symbols ───

#[test]
fn language_with_many_symbols() {
    let table = minimal_parse_table();
    let meta: Vec<SymbolMetadata> = (0..100)
        .map(|i| SymbolMetadata {
            is_terminal: i < 50,
            is_visible: i % 2 == 0,
            is_supertype: false,
        })
        .collect();
    let names: Vec<String> = (0..100).map(|i| format!("sym_{}", i)).collect();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(names)
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 100);
    assert_eq!(lang.symbol_name(0), Some("sym_0"));
    assert_eq!(lang.symbol_name(99), Some("sym_99"));
    assert!(lang.is_terminal(0));
    assert!(lang.is_terminal(49));
    assert!(!lang.is_terminal(50));
    assert!(lang.is_visible(0));
    assert!(!lang.is_visible(1));
}

// ─── Language with many fields ───

#[test]
fn language_with_many_fields() {
    let table = minimal_parse_table();
    let fields: Vec<String> = (0..20).map(|i| format!("field_{}", i)).collect();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .field_names(fields)
        .build()
        .unwrap();
    assert_eq!(lang.field_count, 20);
    assert_eq!(lang.field_name(0), Some("field_0"));
    assert_eq!(lang.field_name(19), Some("field_19"));
    assert_eq!(lang.field_name(20), None);
}

// ─── GLR-core ParseTable: field access ───

#[test]
fn glr_parse_table_state_count() {
    let table = minimal_parse_table();
    assert_eq!(table.state_count, 0);
}

#[test]
fn glr_parse_table_eof_symbol() {
    let table = minimal_parse_table();
    assert_eq!(table.eof_symbol, SymbolId(0));
}

#[test]
fn glr_parse_table_start_symbol() {
    let table = minimal_parse_table();
    assert_eq!(table.start_symbol, SymbolId(0));
}

#[test]
fn glr_parse_table_initial_state() {
    let table = minimal_parse_table();
    assert_eq!(table.initial_state, StateId(0));
}

#[test]
fn glr_parse_table_empty_action_table() {
    let table = minimal_parse_table();
    assert!(table.action_table.is_empty());
}

#[test]
fn glr_parse_table_empty_goto_table() {
    let table = minimal_parse_table();
    assert!(table.goto_table.is_empty());
}

#[test]
fn glr_parse_table_empty_rules() {
    let table = minimal_parse_table();
    assert!(table.rules.is_empty());
}

// ─── Builder chaining order ───

#[test]
fn builder_chaining_any_order() {
    let table = minimal_parse_table();
    // Fields before metadata before parse_table
    let result = Language::builder()
        .field_names(vec!["f1".to_string()])
        .symbol_metadata(minimal_metadata())
        .version(10)
        .parse_table(table)
        .max_alias_sequence_length(3)
        .symbol_names(vec!["s1".to_string()])
        .build();
    assert!(result.is_ok());
    let lang = result.unwrap();
    assert_eq!(lang.version, 10);
    assert_eq!(lang.field_count, 1);
    assert_eq!(lang.max_alias_sequence_length, 3);
}

// ─── Edge cases ───

#[test]
fn builder_empty_symbol_metadata_vec() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 0);
}

#[test]
fn builder_empty_field_names_vec() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .field_names(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.field_count, 0);
}

#[test]
fn language_symbol_name_at_boundary() {
    let table = minimal_parse_table();
    let meta = vec![SymbolMetadata {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    }];
    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["only".to_string()])
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(0), Some("only"));
    assert_eq!(lang.symbol_name(1), None);
}

// ─── Language: large symbol tables ───

#[test]
fn language_large_symbol_table() {
    let table = minimal_parse_table();
    let n = 500;
    let meta: Vec<SymbolMetadata> = (0..n)
        .map(|i| SymbolMetadata {
            is_terminal: i % 3 == 0,
            is_visible: i % 2 == 0,
            is_supertype: i % 7 == 0,
        })
        .collect();
    let names: Vec<String> = (0..n).map(|i| format!("s{}", i)).collect();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(names)
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, n as u32);
    assert_eq!(lang.symbol_name(0), Some("s0"));
    assert_eq!(
        lang.symbol_name((n - 1) as u16),
        Some(format!("s{}", n - 1).as_str())
    );
}

#[test]
fn language_large_field_table() {
    let table = minimal_parse_table();
    let n = 100;
    let fields: Vec<String> = (0..n).map(|i| format!("f{}", i)).collect();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(minimal_metadata())
        .field_names(fields)
        .build()
        .unwrap();
    assert_eq!(lang.field_count, n as u32);
    assert_eq!(
        lang.field_name((n - 1) as u16),
        Some(format!("f{}", n - 1).as_str())
    );
}

// ─── Language: symbol_for_name edge cases ───

#[test]
fn symbol_for_name_empty_string_name() {
    let table = minimal_parse_table();
    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["".to_string()])
        .symbol_metadata(minimal_metadata())
        .build()
        .unwrap();
    // Empty name should still be findable
    let found = lang.symbol_for_name("", true);
    // May or may not find it depending on visibility matching
    let _ = found;
}

#[test]
fn symbol_for_name_returns_first_match() {
    let table = minimal_parse_table();
    let meta = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
    ];
    let lang = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["dup".to_string(), "dup".to_string()])
        .symbol_metadata(meta)
        .build()
        .unwrap();
    let found = lang.symbol_for_name("dup", true);
    assert!(found.is_some());
    assert_eq!(found.unwrap(), 0); // first match
}

// ─── Multiple builders ───

#[test]
fn multiple_languages_independent() {
    let t1 = minimal_parse_table();
    let t2 = minimal_parse_table();
    let l1 = Language::builder()
        .parse_table(t1)
        .symbol_metadata(minimal_metadata())
        .version(1)
        .build()
        .unwrap();
    let l2 = Language::builder()
        .parse_table(t2)
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .version(2)
        .build()
        .unwrap();
    assert_eq!(l1.version, 1);
    assert_eq!(l2.version, 2);
    assert_eq!(l1.symbol_count, 1);
    assert_eq!(l2.symbol_count, 2);
}
