//! Comprehensive tests for symbol metadata operations in adze-ir.

use std::collections::{HashMap, HashSet};

use adze_ir::{
    AliasSequence, Associativity, ExternalToken, FieldId, PrecedenceKind, ProductionId, Rule,
    RuleId, StateId, Symbol, SymbolId, SymbolMetadata, SymbolRegistry,
};

// ---------------------------------------------------------------------------
// 1. SymbolId creation and comparison
// ---------------------------------------------------------------------------

#[test]
fn symbol_id_creation_and_equality() {
    let a = SymbolId(0);
    let b = SymbolId(0);
    let c = SymbolId(1);

    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn symbol_id_copy_semantics() {
    let a = SymbolId(42);
    let b = a; // Copy
    assert_eq!(a, b);
    assert_eq!(a.0, 42);
}

// ---------------------------------------------------------------------------
// 2. SymbolId ordering is consistent
// ---------------------------------------------------------------------------

#[test]
fn symbol_id_ordering_consistent() {
    let a = SymbolId(1);
    let b = SymbolId(2);
    let c = SymbolId(3);

    assert!(a < b);
    assert!(b < c);
    assert!(a < c); // transitivity
}

#[test]
fn symbol_id_sorting() {
    let mut ids = vec![
        SymbolId(5),
        SymbolId(1),
        SymbolId(3),
        SymbolId(0),
        SymbolId(2),
    ];
    ids.sort();
    let expected: Vec<SymbolId> = (0..=5).filter(|&i| i != 4).map(SymbolId).collect();
    assert_eq!(ids, expected);
}

// ---------------------------------------------------------------------------
// 3. SymbolId display/debug formatting
// ---------------------------------------------------------------------------

#[test]
fn symbol_id_display() {
    assert_eq!(format!("{}", SymbolId(7)), "Symbol(7)");
    assert_eq!(format!("{}", SymbolId(0)), "Symbol(0)");
}

#[test]
fn symbol_id_debug() {
    let dbg = format!("{:?}", SymbolId(42));
    assert!(dbg.contains("SymbolId"));
    assert!(dbg.contains("42"));
}

#[test]
fn other_id_display_formats() {
    assert_eq!(format!("{}", RuleId(3)), "Rule(3)");
    assert_eq!(format!("{}", StateId(10)), "State(10)");
    assert_eq!(format!("{}", FieldId(1)), "Field(1)");
    assert_eq!(format!("{}", ProductionId(5)), "Production(5)");
}

// ---------------------------------------------------------------------------
// 4. Symbol types: Terminal, NonTerminal, External, Epsilon
// ---------------------------------------------------------------------------

#[test]
fn symbol_variant_terminal() {
    let sym = Symbol::Terminal(SymbolId(1));
    assert!(matches!(sym, Symbol::Terminal(_)));
}

#[test]
fn symbol_variant_nonterminal() {
    let sym = Symbol::NonTerminal(SymbolId(2));
    assert!(matches!(sym, Symbol::NonTerminal(_)));
}

#[test]
fn symbol_variant_external() {
    let sym = Symbol::External(SymbolId(3));
    assert!(matches!(sym, Symbol::External(_)));
}

#[test]
fn symbol_variant_epsilon() {
    let sym = Symbol::Epsilon;
    assert!(matches!(sym, Symbol::Epsilon));
}

#[test]
fn symbol_optional_and_repeat_variants() {
    let inner = Symbol::Terminal(SymbolId(1));
    let opt = Symbol::Optional(Box::new(inner.clone()));
    let rep = Symbol::Repeat(Box::new(inner.clone()));
    let rep1 = Symbol::RepeatOne(Box::new(inner));
    assert!(matches!(opt, Symbol::Optional(_)));
    assert!(matches!(rep, Symbol::Repeat(_)));
    assert!(matches!(rep1, Symbol::RepeatOne(_)));
}

// ---------------------------------------------------------------------------
// 5. Symbol equality and hash consistency
// ---------------------------------------------------------------------------

#[test]
fn symbol_equality() {
    let a = Symbol::Terminal(SymbolId(5));
    let b = Symbol::Terminal(SymbolId(5));
    let c = Symbol::Terminal(SymbolId(6));

    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(
        Symbol::Terminal(SymbolId(5)),
        Symbol::NonTerminal(SymbolId(5))
    );
}

#[test]
fn symbol_hash_consistency() {
    let mut set = HashSet::new();
    let a = Symbol::Terminal(SymbolId(10));
    let b = Symbol::Terminal(SymbolId(10));

    set.insert(a.clone());
    // Same symbol should not increase set size
    set.insert(b);
    assert_eq!(set.len(), 1);
}

#[test]
fn symbol_hash_different_variants() {
    let mut set = HashSet::new();
    set.insert(Symbol::Terminal(SymbolId(1)));
    set.insert(Symbol::NonTerminal(SymbolId(1)));
    set.insert(Symbol::External(SymbolId(1)));
    set.insert(Symbol::Epsilon);
    assert_eq!(set.len(), 4);
}

#[test]
fn symbol_id_hash_as_map_key() {
    let mut map = HashMap::new();
    map.insert(SymbolId(0), "eof");
    map.insert(SymbolId(1), "number");
    map.insert(SymbolId(0), "eof_updated");
    assert_eq!(map.len(), 2);
    assert_eq!(map[&SymbolId(0)], "eof_updated");
}

// ---------------------------------------------------------------------------
// 6. SymbolRegistry: register and lookup terminals
// ---------------------------------------------------------------------------

#[test]
fn registry_register_and_lookup_terminal() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let id = reg.register("plus", meta);

    assert_eq!(reg.get_id("plus"), Some(id));
    assert_eq!(reg.get_name(id), Some("plus"));
    assert_eq!(reg.get_metadata(id), Some(meta));
}

// ---------------------------------------------------------------------------
// 7. SymbolRegistry: register and lookup non-terminals
// ---------------------------------------------------------------------------

#[test]
fn registry_register_and_lookup_nonterminal() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    };
    let id = reg.register("expression", meta);

    assert_eq!(reg.get_id("expression"), Some(id));
    assert_eq!(reg.get_name(id), Some("expression"));
    let stored = reg.get_metadata(id).unwrap();
    assert!(stored.named);
    assert!(!stored.terminal);
}

// ---------------------------------------------------------------------------
// 8. SymbolRegistry: register and lookup external tokens
// ---------------------------------------------------------------------------

#[test]
fn registry_register_and_lookup_external() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let id = reg.register("_indent", meta);

    assert!(reg.contains_id(id));
    assert_eq!(reg.get_name(id), Some("_indent"));
}

// ---------------------------------------------------------------------------
// 9. SymbolRegistry: duplicate registration handling
// ---------------------------------------------------------------------------

#[test]
fn registry_duplicate_returns_same_id() {
    let mut reg = SymbolRegistry::new();
    let meta1 = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let meta2 = SymbolMetadata {
        visible: false,
        named: true,
        hidden: true,
        terminal: false,
    };

    let id1 = reg.register("tok", meta1);
    let id2 = reg.register("tok", meta2);

    // Same ID returned
    assert_eq!(id1, id2);
    // Metadata updated to latest
    assert_eq!(reg.get_metadata(id1), Some(meta2));
}

#[test]
fn registry_duplicate_does_not_increase_count() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let before = reg.len();
    reg.register("number", meta);
    let after_first = reg.len();
    reg.register("number", meta);
    let after_dup = reg.len();

    assert_eq!(after_first, before + 1);
    assert_eq!(after_dup, after_first);
}

// ---------------------------------------------------------------------------
// 10. SymbolRegistry: ID allocation is sequential
// ---------------------------------------------------------------------------

#[test]
fn registry_sequential_id_allocation() {
    let mut reg = SymbolRegistry::new();
    // "end" is registered at SymbolId(0) during new()
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };

    let id_a = reg.register("alpha", meta);
    let id_b = reg.register("beta", meta);
    let id_c = reg.register("gamma", meta);

    assert_eq!(id_a, SymbolId(1));
    assert_eq!(id_b, SymbolId(2));
    assert_eq!(id_c, SymbolId(3));
}

#[test]
fn registry_eof_is_symbol_zero() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
    assert_eq!(reg.get_name(SymbolId(0)), Some("end"));
}

#[test]
fn registry_len_starts_with_eof() {
    let reg = SymbolRegistry::new();
    // New registry has "end" symbol pre-registered
    assert_eq!(reg.len(), 1);
    assert!(!reg.is_empty());
}

// ---------------------------------------------------------------------------
// 11. Rule definition with single symbol
// ---------------------------------------------------------------------------

#[test]
fn rule_single_symbol() {
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };

    assert_eq!(rule.lhs, SymbolId(10));
    assert_eq!(rule.rhs.len(), 1);
    assert!(matches!(rule.rhs[0], Symbol::Terminal(SymbolId(1))));
}

// ---------------------------------------------------------------------------
// 12. Rule definition with multiple symbols (sequence)
// ---------------------------------------------------------------------------

#[test]
fn rule_multiple_symbols() {
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(10)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(10)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };

    assert_eq!(rule.rhs.len(), 3);
    assert!(matches!(rule.rhs[1], Symbol::Terminal(SymbolId(2))));
}

// ---------------------------------------------------------------------------
// 13. Rule definition with precedence and associativity
// ---------------------------------------------------------------------------

#[test]
fn rule_with_static_precedence() {
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(10)),
            Symbol::Terminal(SymbolId(3)),
            Symbol::NonTerminal(SymbolId(10)),
        ],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Left),
        fields: vec![],
        production_id: ProductionId(1),
    };

    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(5)));
    assert_eq!(rule.associativity, Some(Associativity::Left));
}

#[test]
fn rule_with_dynamic_precedence() {
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::NonTerminal(SymbolId(10))],
        precedence: Some(PrecedenceKind::Dynamic(-1)),
        associativity: Some(Associativity::Right),
        fields: vec![],
        production_id: ProductionId(2),
    };

    assert_eq!(rule.precedence, Some(PrecedenceKind::Dynamic(-1)));
    assert_eq!(rule.associativity, Some(Associativity::Right));
}

#[test]
fn associativity_none_variant() {
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: Some(PrecedenceKind::Static(0)),
        associativity: Some(Associativity::None),
        fields: vec![],
        production_id: ProductionId(0),
    };
    assert_eq!(rule.associativity, Some(Associativity::None));
}

// ---------------------------------------------------------------------------
// 14. Production metadata: rule_id, symbol count, field mapping
// ---------------------------------------------------------------------------

#[test]
fn production_metadata_fields() {
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(11)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(12)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 2)],
        production_id: ProductionId(7),
    };

    assert_eq!(rule.production_id, ProductionId(7));
    assert_eq!(rule.rhs.len(), 3);
    assert_eq!(rule.fields.len(), 2);
    // First field maps FieldId(0) -> position 0
    assert_eq!(rule.fields[0], (FieldId(0), 0));
    // Second field maps FieldId(1) -> position 2
    assert_eq!(rule.fields[1], (FieldId(1), 2));
}

#[test]
fn production_id_ordering() {
    let a = ProductionId(1);
    let b = ProductionId(2);
    assert!(a < b);
    assert_eq!(ProductionId(3), ProductionId(3));
}

#[test]
fn rule_equality() {
    let r1 = Rule {
        lhs: SymbolId(10),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    let r2 = r1.clone();
    assert_eq!(r1, r2);
}

// ---------------------------------------------------------------------------
// 15. AliasSequence creation and lookup
// ---------------------------------------------------------------------------

#[test]
fn alias_sequence_creation() {
    let seq = AliasSequence {
        aliases: vec![
            Some("identifier".to_string()),
            None,
            Some("value".to_string()),
        ],
    };

    assert_eq!(seq.aliases.len(), 3);
    assert_eq!(seq.aliases[0].as_deref(), Some("identifier"));
    assert!(seq.aliases[1].is_none());
    assert_eq!(seq.aliases[2].as_deref(), Some("value"));
}

#[test]
fn alias_sequence_all_none() {
    let seq = AliasSequence {
        aliases: vec![None, None, None],
    };
    assert!(seq.aliases.iter().all(|a| a.is_none()));
}

#[test]
fn alias_sequence_empty() {
    let seq = AliasSequence { aliases: vec![] };
    assert!(seq.aliases.is_empty());
}

// ---------------------------------------------------------------------------
// Additional coverage: SymbolMetadata, registry iteration, index maps
// ---------------------------------------------------------------------------

#[test]
fn symbol_metadata_equality() {
    let a = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    };
    let b = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    };
    let c = SymbolMetadata {
        visible: false,
        named: true,
        hidden: false,
        terminal: false,
    };
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn registry_iter_preserves_insertion_order() {
    let mut reg = SymbolRegistry::new();
    let meta_term = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let meta_nt = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    };

    reg.register("plus", meta_term);
    reg.register("expr", meta_nt);

    let names: Vec<&str> = reg.iter().map(|(name, _)| name).collect();
    // "end" is first (registered in new()), then "plus", then "expr"
    assert_eq!(names, vec!["end", "plus", "expr"]);
}

#[test]
fn registry_to_index_map() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let id_a = reg.register("a", meta);
    let id_b = reg.register("b", meta);

    let idx_map = reg.to_index_map();
    // "end" is index 0, "a" is index 1, "b" is index 2
    assert_eq!(idx_map[&SymbolId(0)], 0);
    assert_eq!(idx_map[&id_a], 1);
    assert_eq!(idx_map[&id_b], 2);
}

#[test]
fn registry_to_symbol_map() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    reg.register("x", meta);

    let sym_map = reg.to_symbol_map();
    assert_eq!(sym_map[&0], SymbolId(0)); // end
    assert_eq!(sym_map[&1], SymbolId(1)); // x
}

#[test]
fn registry_lookup_nonexistent() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("nonexistent"), None);
    assert_eq!(reg.get_name(SymbolId(999)), None);
    assert_eq!(reg.get_metadata(SymbolId(999)), None);
    assert!(!reg.contains_id(SymbolId(999)));
}

#[test]
fn external_token_in_registry() {
    let mut reg = SymbolRegistry::new();
    let ext_meta = SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    };
    let id = reg.register("_newline", ext_meta);

    let _ext = ExternalToken {
        name: "_newline".to_string(),
        symbol_id: id,
    };

    assert_eq!(reg.get_id("_newline"), Some(id));
}
