#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for SymbolMetadata handling in adze-glr-core.

use adze_glr_core::{FirstFollowSets, ParseTable, SymbolId, SymbolMetadata, build_lr1_automaton};
use adze_ir::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a minimal `SymbolMetadata` with the given flags.
fn meta(
    name: &str,
    terminal: bool,
    named: bool,
    visible: bool,
    supertype: bool,
    extra: bool,
    fragile: bool,
    id: u16,
) -> SymbolMetadata {
    SymbolMetadata {
        name: name.to_string(),
        is_terminal: terminal,
        is_named: named,
        is_visible: visible,
        is_supertype: supertype,
        is_extra: extra,
        is_fragile: fragile,
        symbol_id: SymbolId(id),
    }
}

/// Build a trivial grammar: S → a
fn simple_grammar() -> Grammar {
    let mut g = Grammar::new("simple".into());
    let a = SymbolId(1);
    let s = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

/// Build a grammar with extras: S → a, with whitespace as extra
fn grammar_with_extras() -> Grammar {
    let mut g = Grammar::new("extras".into());
    let a = SymbolId(1);
    let ws = SymbolId(2);
    let s = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        ws,
        Token {
            name: "ws".into(),
            pattern: TokenPattern::Regex(r"\s+".into()),
            fragile: false,
        },
    );
    g.extras.push(ws);
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

/// Build a grammar with supertypes: S → a, _expression is supertype
fn grammar_with_supertypes() -> Grammar {
    let mut g = Grammar::new("supertypes".into());
    let a = SymbolId(1);
    let expr = SymbolId(10);
    let s = SymbolId(11);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(expr, "expression".into());
    g.rules.insert(
        expr,
        vec![Rule {
            lhs: expr,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::NonTerminal(expr)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );
    g.supertypes.push(expr);
    g
}

/// Build a grammar with external tokens
fn grammar_with_externals() -> Grammar {
    let mut g = Grammar::new("externals".into());
    let a = SymbolId(1);
    let s = SymbolId(10);
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(50),
    });
    g.externals.push(ExternalToken {
        name: "_dedent".into(),
        symbol_id: SymbolId(51),
    });
    g
}

fn build_table(g: &Grammar) -> ParseTable {
    let mut gc = g.clone();
    let ff = FirstFollowSets::compute_normalized(&mut gc).expect("FIRST/FOLLOW");
    build_lr1_automaton(&gc, &ff).expect("build_lr1_automaton")
}

// ---------------------------------------------------------------------------
// 1. Construction / field access
// ---------------------------------------------------------------------------

#[test]
fn test_basic_field_access() {
    let m = meta("ident", false, true, true, false, false, false, 42);
    assert_eq!(m.name, "ident");
    assert!(!m.is_terminal);
    assert!(m.is_named);
    assert!(m.is_visible);
    assert!(!m.is_supertype);
    assert!(!m.is_extra);
    assert!(!m.is_fragile);
    assert_eq!(m.symbol_id, SymbolId(42));
}

#[test]
fn test_terminal_metadata() {
    let m = meta("+", true, false, true, false, false, false, 3);
    assert!(m.is_terminal);
    assert!(!m.is_named);
}

#[test]
fn test_named_nonterminal() {
    let m = meta("expression", false, true, true, false, false, false, 10);
    assert!(!m.is_terminal);
    assert!(m.is_named);
    assert!(m.is_visible);
}

#[test]
fn test_supertype_metadata() {
    let m = meta("_type", false, true, true, true, false, false, 20);
    assert!(m.is_supertype);
}

#[test]
fn test_extra_metadata() {
    let m = meta("comment", true, true, true, false, true, false, 5);
    assert!(m.is_extra);
    assert!(m.is_terminal);
}

#[test]
fn test_fragile_metadata() {
    let m = meta("kw", true, true, true, false, false, true, 7);
    assert!(m.is_fragile);
}

#[test]
fn test_all_flags_true() {
    let m = meta("x", true, true, true, true, true, true, 0);
    assert!(
        m.is_terminal && m.is_named && m.is_visible && m.is_supertype && m.is_extra && m.is_fragile
    );
}

#[test]
fn test_all_flags_false() {
    let m = meta("y", false, false, false, false, false, false, 99);
    assert!(
        !m.is_terminal
            && !m.is_named
            && !m.is_visible
            && !m.is_supertype
            && !m.is_extra
            && !m.is_fragile
    );
}

// ---------------------------------------------------------------------------
// 2. Clone
// ---------------------------------------------------------------------------

#[test]
fn test_clone_preserves_fields() {
    let m = meta("foo", true, false, true, false, true, false, 8);
    let c = m.clone();
    assert_eq!(c.name, m.name);
    assert_eq!(c.is_terminal, m.is_terminal);
    assert_eq!(c.is_named, m.is_named);
    assert_eq!(c.is_visible, m.is_visible);
    assert_eq!(c.is_supertype, m.is_supertype);
    assert_eq!(c.is_extra, m.is_extra);
    assert_eq!(c.is_fragile, m.is_fragile);
    assert_eq!(c.symbol_id, m.symbol_id);
}

// ---------------------------------------------------------------------------
// 3. Debug formatting
// ---------------------------------------------------------------------------

#[test]
fn test_debug_contains_name() {
    let m = meta("number", true, true, true, false, false, false, 4);
    let dbg = format!("{:?}", m);
    assert!(
        dbg.contains("number"),
        "Debug output should contain the name"
    );
}

#[test]
fn test_debug_contains_flags() {
    let m = meta("x", true, false, true, true, false, true, 1);
    let dbg = format!("{:?}", m);
    assert!(dbg.contains("is_terminal: true"));
    assert!(dbg.contains("is_supertype: true"));
    assert!(dbg.contains("is_fragile: true"));
}

// ---------------------------------------------------------------------------
// 4. Vec<SymbolMetadata> operations
// ---------------------------------------------------------------------------

#[test]
fn test_metadata_vec_len() {
    let v: Vec<SymbolMetadata> = (0..5)
        .map(|i| {
            meta(
                &format!("s{}", i),
                i % 2 == 0,
                true,
                true,
                false,
                false,
                false,
                i as u16,
            )
        })
        .collect();
    assert_eq!(v.len(), 5);
}

#[test]
fn test_metadata_vec_filter_terminals() {
    let v: Vec<SymbolMetadata> = vec![
        meta("a", true, false, true, false, false, false, 1),
        meta("B", false, true, true, false, false, false, 2),
        meta("c", true, true, true, false, false, false, 3),
    ];
    let terminals: Vec<_> = v.iter().filter(|m| m.is_terminal).collect();
    assert_eq!(terminals.len(), 2);
}

#[test]
fn test_metadata_vec_filter_named() {
    let v: Vec<SymbolMetadata> = vec![
        meta("+", true, false, true, false, false, false, 1),
        meta("expr", false, true, true, false, false, false, 2),
        meta("-", true, false, true, false, false, false, 3),
    ];
    let named: Vec<_> = v.iter().filter(|m| m.is_named).collect();
    assert_eq!(named.len(), 1);
    assert_eq!(named[0].name, "expr");
}

#[test]
fn test_metadata_lookup_by_symbol_id() {
    let v: Vec<SymbolMetadata> = vec![
        meta("a", true, false, true, false, false, false, 1),
        meta("b", true, true, true, false, false, false, 5),
        meta("c", false, true, true, false, false, false, 10),
    ];
    let found = v.iter().find(|m| m.symbol_id == SymbolId(5));
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "b");
}

#[test]
fn test_metadata_no_duplicate_symbol_ids() {
    let v: Vec<SymbolMetadata> = (0..10)
        .map(|i| {
            meta(
                &format!("s{}", i),
                true,
                true,
                true,
                false,
                false,
                false,
                i as u16,
            )
        })
        .collect();
    let mut ids: Vec<SymbolId> = v.iter().map(|m| m.symbol_id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), v.len(), "symbol IDs should be unique");
}

// ---------------------------------------------------------------------------
// 5. ParseTable integration — simple grammar
// ---------------------------------------------------------------------------

#[test]
fn test_simple_grammar_has_metadata() {
    let g = simple_grammar();
    let table = build_table(&g);
    assert!(
        !table.symbol_metadata.is_empty(),
        "parse table should contain symbol metadata"
    );
}

#[test]
fn test_simple_grammar_terminal_marked() {
    let g = simple_grammar();
    let table = build_table(&g);
    let terminal_meta: Vec<_> = table
        .symbol_metadata
        .iter()
        .filter(|m| m.is_terminal)
        .collect();
    assert!(
        !terminal_meta.is_empty(),
        "should have at least one terminal in metadata"
    );
}

#[test]
fn test_simple_grammar_nonterminal_marked() {
    let g = simple_grammar();
    let table = build_table(&g);
    let nt_meta: Vec<_> = table
        .symbol_metadata
        .iter()
        .filter(|m| !m.is_terminal)
        .collect();
    assert!(
        !nt_meta.is_empty(),
        "should have at least one non-terminal in metadata"
    );
}

#[test]
fn test_simple_grammar_nonterminals_are_named() {
    let g = simple_grammar();
    let table = build_table(&g);
    for m in &table.symbol_metadata {
        if !m.is_terminal {
            assert!(m.is_named, "non-terminal '{}' should be named", m.name);
        }
    }
}

#[test]
fn test_simple_grammar_nonterminals_are_visible() {
    let g = simple_grammar();
    let table = build_table(&g);
    for m in &table.symbol_metadata {
        if !m.is_terminal {
            assert!(m.is_visible, "non-terminal '{}' should be visible", m.name);
        }
    }
}

#[test]
fn test_string_token_is_anonymous() {
    // Tokens with TokenPattern::String are anonymous (is_named == false)
    let g = simple_grammar();
    let table = build_table(&g);
    let string_terminals: Vec<_> = table
        .symbol_metadata
        .iter()
        .filter(|m| m.is_terminal && m.name == "a")
        .collect();
    assert!(!string_terminals.is_empty(), "should find terminal 'a'");
    for m in &string_terminals {
        assert!(
            !m.is_named,
            "string literal terminal 'a' should be anonymous"
        );
    }
}

#[test]
fn test_regex_token_is_named() {
    // Tokens with TokenPattern::Regex are named (is_named == true)
    let mut g = Grammar::new("regex_tok".into());
    let num = SymbolId(1);
    let s = SymbolId(10);
    g.tokens.insert(
        num,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(num)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let table = build_table(&g);
    let regex_terminals: Vec<_> = table
        .symbol_metadata
        .iter()
        .filter(|m| m.is_terminal && m.name == "number")
        .collect();
    assert!(!regex_terminals.is_empty());
    for m in &regex_terminals {
        assert!(m.is_named, "regex terminal 'number' should be named");
    }
}

// ---------------------------------------------------------------------------
// 6. Extras
// ---------------------------------------------------------------------------

#[test]
fn test_extras_marked_as_extra() {
    let g = grammar_with_extras();
    let table = build_table(&g);
    let ws_meta: Vec<_> = table
        .symbol_metadata
        .iter()
        .filter(|m| m.name == "ws")
        .collect();
    assert!(!ws_meta.is_empty(), "should find ws metadata");
    for m in &ws_meta {
        assert!(m.is_extra, "ws should be marked as extra");
    }
}

#[test]
fn test_non_extra_token_not_marked_extra() {
    let g = grammar_with_extras();
    let table = build_table(&g);
    let a_meta: Vec<_> = table
        .symbol_metadata
        .iter()
        .filter(|m| m.name == "a")
        .collect();
    assert!(!a_meta.is_empty());
    for m in &a_meta {
        assert!(!m.is_extra, "token 'a' should not be extra");
    }
}

#[test]
fn test_nonterminals_never_extra() {
    let g = grammar_with_extras();
    let table = build_table(&g);
    for m in &table.symbol_metadata {
        if !m.is_terminal {
            assert!(
                !m.is_extra,
                "non-terminal '{}' should never be extra",
                m.name
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Supertypes
// ---------------------------------------------------------------------------

#[test]
fn test_supertype_marked() {
    let g = grammar_with_supertypes();
    let table = build_table(&g);
    let supertype_meta: Vec<_> = table
        .symbol_metadata
        .iter()
        .filter(|m| m.is_supertype)
        .collect();
    assert!(
        !supertype_meta.is_empty(),
        "should have at least one supertype symbol"
    );
}

#[test]
fn test_non_supertype_not_marked() {
    let g = simple_grammar();
    let table = build_table(&g);
    for m in &table.symbol_metadata {
        assert!(!m.is_supertype, "simple grammar should have no supertypes");
    }
}

// ---------------------------------------------------------------------------
// 8. External tokens
// ---------------------------------------------------------------------------

#[test]
fn test_externals_are_terminal() {
    let g = grammar_with_externals();
    let table = build_table(&g);
    for ext in &g.externals {
        let found = table
            .symbol_metadata
            .iter()
            .find(|m| m.symbol_id == ext.symbol_id);
        if let Some(m) = found {
            assert!(m.is_terminal, "external '{}' should be terminal", m.name);
        }
    }
}

#[test]
fn test_externals_are_named() {
    let g = grammar_with_externals();
    let table = build_table(&g);
    for ext in &g.externals {
        let found = table
            .symbol_metadata
            .iter()
            .find(|m| m.symbol_id == ext.symbol_id);
        if let Some(m) = found {
            assert!(m.is_named, "external '{}' should be named", m.name);
        }
    }
}

#[test]
fn test_external_visibility_by_name() {
    // Names starting with '_' are hidden, others visible
    let g = grammar_with_externals();
    let table = build_table(&g);
    let indent_meta = table.symbol_metadata.iter().find(|m| m.name == "indent");
    let dedent_meta = table.symbol_metadata.iter().find(|m| m.name == "_dedent");
    if let Some(m) = indent_meta {
        assert!(m.is_visible, "indent should be visible");
    }
    if let Some(m) = dedent_meta {
        assert!(!m.is_visible, "_dedent should be hidden");
    }
}

// ---------------------------------------------------------------------------
// 9. Visibility convention (underscore prefix)
// ---------------------------------------------------------------------------

#[test]
fn test_underscore_token_hidden() {
    let mut g = Grammar::new("hidden".into());
    let hidden = SymbolId(1);
    let s = SymbolId(10);
    g.tokens.insert(
        hidden,
        Token {
            name: "_ws".into(),
            pattern: TokenPattern::Regex(r"\s+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(SymbolId(2))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let table = build_table(&g);
    let ws_meta = table.symbol_metadata.iter().find(|m| m.name == "_ws");
    if let Some(m) = ws_meta {
        assert!(!m.is_visible, "_ws should not be visible");
    }
}

// ---------------------------------------------------------------------------
// 10. Default ParseTable metadata
// ---------------------------------------------------------------------------

#[test]
fn test_default_parse_table_empty_metadata() {
    let table = ParseTable::default();
    assert!(
        table.symbol_metadata.is_empty(),
        "default ParseTable should have empty symbol_metadata"
    );
}

// ---------------------------------------------------------------------------
// 11. Mutation / collection helpers
// ---------------------------------------------------------------------------

#[test]
fn test_push_metadata_to_vec() {
    let mut v: Vec<SymbolMetadata> = Vec::new();
    v.push(meta("a", true, false, true, false, false, false, 1));
    v.push(meta("B", false, true, true, false, false, false, 2));
    assert_eq!(v.len(), 2);
    assert!(v[0].is_terminal);
    assert!(!v[1].is_terminal);
}

#[test]
fn test_metadata_in_btreemap() {
    let mut map: BTreeMap<SymbolId, SymbolMetadata> = BTreeMap::new();
    map.insert(
        SymbolId(1),
        meta("a", true, false, true, false, false, false, 1),
    );
    map.insert(
        SymbolId(2),
        meta("b", true, true, true, false, false, false, 2),
    );
    assert_eq!(map.len(), 2);
    assert_eq!(map[&SymbolId(1)].name, "a");
}

// ---------------------------------------------------------------------------
// 12. Multi-token grammar metadata counts
// ---------------------------------------------------------------------------

#[test]
fn test_multi_token_grammar_metadata_counts() {
    let mut g = Grammar::new("multi".into());
    let plus = SymbolId(1);
    let num = SymbolId(2);
    let expr = SymbolId(10);
    g.tokens.insert(
        plus,
        Token {
            name: "+".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        num,
        Token {
            name: "number".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(expr, "expr".into());
    g.rules.insert(
        expr,
        vec![
            Rule {
                lhs: expr,
                rhs: vec![Symbol::Terminal(num)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: expr,
                rhs: vec![
                    Symbol::NonTerminal(expr),
                    Symbol::Terminal(plus),
                    Symbol::NonTerminal(expr),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    let table = build_table(&g);
    let terminal_count = table
        .symbol_metadata
        .iter()
        .filter(|m| m.is_terminal)
        .count();
    let nt_count = table
        .symbol_metadata
        .iter()
        .filter(|m| !m.is_terminal)
        .count();
    // At least the tokens we declared
    assert!(
        terminal_count >= 2,
        "should have at least 2 terminals, got {}",
        terminal_count
    );
    // At least the nonterminal we declared
    assert!(
        nt_count >= 1,
        "should have at least 1 non-terminal, got {}",
        nt_count
    );
}

// ---------------------------------------------------------------------------
// 13. symbol_id roundtrip (metadata → lookup)
// ---------------------------------------------------------------------------

#[test]
fn test_symbol_id_roundtrip_in_table() {
    let g = simple_grammar();
    let table = build_table(&g);
    for m in &table.symbol_metadata {
        // Every symbol_id in metadata should be a valid SymbolId
        assert!(
            m.symbol_id.0 < u16::MAX,
            "symbol_id should be a finite value"
        );
    }
}

// ---------------------------------------------------------------------------
// 14. Fragile defaults to false in build_lr1_automaton
// ---------------------------------------------------------------------------

#[test]
fn test_built_metadata_fragile_default() {
    let g = simple_grammar();
    let table = build_table(&g);
    for m in &table.symbol_metadata {
        assert!(
            !m.is_fragile,
            "built metadata should default fragile to false"
        );
    }
}

// ---------------------------------------------------------------------------
// 15. Name correctness
// ---------------------------------------------------------------------------

#[test]
fn test_terminal_name_preserved() {
    let g = simple_grammar();
    let table = build_table(&g);
    let has_a = table.symbol_metadata.iter().any(|m| m.name == "a");
    assert!(has_a, "terminal name 'a' should appear in metadata");
}

#[test]
fn test_nonterminal_name_format() {
    // Non-terminals get names like "rule_<id>"
    let g = simple_grammar();
    let table = build_table(&g);
    let nt_names: Vec<_> = table
        .symbol_metadata
        .iter()
        .filter(|m| !m.is_terminal)
        .map(|m| m.name.clone())
        .collect();
    for name in &nt_names {
        assert!(
            name.starts_with("rule_"),
            "non-terminal name '{}' should start with 'rule_'",
            name,
        );
    }
}

// ---------------------------------------------------------------------------
// 16. Iteration patterns
// ---------------------------------------------------------------------------

#[test]
fn test_partition_terminals_nonterminals() {
    let g = grammar_with_extras();
    let table = build_table(&g);
    let (terms, nonterms): (Vec<_>, Vec<_>) =
        table.symbol_metadata.iter().partition(|m| m.is_terminal);
    assert_eq!(
        terms.len() + nonterms.len(),
        table.symbol_metadata.len(),
        "partition should cover all metadata entries"
    );
}

#[test]
fn test_count_visible() {
    let g = simple_grammar();
    let table = build_table(&g);
    let visible_count = table
        .symbol_metadata
        .iter()
        .filter(|m| m.is_visible)
        .count();
    assert!(visible_count > 0, "should have at least one visible symbol");
}

#[test]
fn test_index_based_access() {
    let g = simple_grammar();
    let table = build_table(&g);
    for i in 0..table.symbol_metadata.len() {
        let m = &table.symbol_metadata[i];
        // Should not panic and should have a non-empty name
        assert!(!m.name.is_empty(), "metadata at index {} has empty name", i);
    }
}
