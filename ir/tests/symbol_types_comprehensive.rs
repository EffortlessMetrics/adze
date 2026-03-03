//! Comprehensive tests for IR symbol types and their operations.

use adze_ir::{
    Associativity, FieldId, Grammar, PrecedenceKind, ProductionId, Rule, RuleId, StateId, Symbol,
    SymbolId, SymbolMetadata,
};
use std::collections::{BTreeSet, HashMap, HashSet};

// ---------------------------------------------------------------------------
// 1. Basic Symbol enum variant construction
// ---------------------------------------------------------------------------

#[test]
fn terminal_symbol_wraps_symbol_id() {
    let sym = Symbol::Terminal(SymbolId(7));
    match sym {
        Symbol::Terminal(id) => assert_eq!(id, SymbolId(7)),
        _ => panic!("expected Terminal"),
    }
}

#[test]
fn nonterminal_symbol_wraps_symbol_id() {
    let sym = Symbol::NonTerminal(SymbolId(42));
    match sym {
        Symbol::NonTerminal(id) => assert_eq!(id, SymbolId(42)),
        _ => panic!("expected NonTerminal"),
    }
}

#[test]
fn external_symbol_wraps_symbol_id() {
    let sym = Symbol::External(SymbolId(99));
    match sym {
        Symbol::External(id) => assert_eq!(id, SymbolId(99)),
        _ => panic!("expected External"),
    }
}

#[test]
fn epsilon_is_unit_variant() {
    let sym = Symbol::Epsilon;
    assert_eq!(sym, Symbol::Epsilon);
}

// ---------------------------------------------------------------------------
// 2. Complex / recursive symbol construction
// ---------------------------------------------------------------------------

#[test]
fn optional_wraps_inner_symbol() {
    let inner = Symbol::Terminal(SymbolId(1));
    let opt = Symbol::Optional(Box::new(inner.clone()));
    assert_eq!(
        opt,
        Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))))
    );
}

#[test]
fn repeat_wraps_inner_symbol() {
    let inner = Symbol::NonTerminal(SymbolId(5));
    let rep = Symbol::Repeat(Box::new(inner.clone()));
    assert_eq!(rep, Symbol::Repeat(Box::new(inner)));
}

#[test]
fn repeat_one_wraps_inner_symbol() {
    let inner = Symbol::Terminal(SymbolId(3));
    let rep1 = Symbol::RepeatOne(Box::new(inner.clone()));
    assert_eq!(rep1, Symbol::RepeatOne(Box::new(inner)));
}

#[test]
fn choice_holds_multiple_alternatives() {
    let choices = vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
        Symbol::Epsilon,
    ];
    let sym = Symbol::Choice(choices.clone());
    if let Symbol::Choice(v) = &sym {
        assert_eq!(v.len(), 3);
    } else {
        panic!("expected Choice");
    }
}

#[test]
fn sequence_holds_ordered_symbols() {
    let seq = vec![
        Symbol::Terminal(SymbolId(10)),
        Symbol::NonTerminal(SymbolId(20)),
        Symbol::Terminal(SymbolId(30)),
    ];
    let sym = Symbol::Sequence(seq.clone());
    if let Symbol::Sequence(v) = &sym {
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], Symbol::Terminal(SymbolId(10)));
        assert_eq!(v[2], Symbol::Terminal(SymbolId(30)));
    } else {
        panic!("expected Sequence");
    }
}

// ---------------------------------------------------------------------------
// 3. Deeply nested symbol construction
// ---------------------------------------------------------------------------

#[test]
fn deeply_nested_optional_repeat_choice() {
    // Optional(Repeat(Choice([Terminal(1), NonTerminal(2)])))
    let leaf_a = Symbol::Terminal(SymbolId(1));
    let leaf_b = Symbol::NonTerminal(SymbolId(2));
    let choice = Symbol::Choice(vec![leaf_a, leaf_b]);
    let repeat = Symbol::Repeat(Box::new(choice));
    let opt = Symbol::Optional(Box::new(repeat.clone()));

    // Verify nesting via matching
    match &opt {
        Symbol::Optional(inner) => match inner.as_ref() {
            Symbol::Repeat(inner2) => match inner2.as_ref() {
                Symbol::Choice(v) => assert_eq!(v.len(), 2),
                other => panic!("expected Choice, got {other:?}"),
            },
            other => panic!("expected Repeat, got {other:?}"),
        },
        other => panic!("expected Optional, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 4. Equality & inequality
// ---------------------------------------------------------------------------

#[test]
fn symbol_equality_same_variant_same_id() {
    assert_eq!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1)));
    assert_eq!(
        Symbol::NonTerminal(SymbolId(0)),
        Symbol::NonTerminal(SymbolId(0))
    );
}

#[test]
fn symbol_inequality_different_variant() {
    assert_ne!(
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(1))
    );
}

#[test]
fn symbol_inequality_same_variant_different_id() {
    assert_ne!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2)));
}

// ---------------------------------------------------------------------------
// 5. Ordering (Ord is derived)
// ---------------------------------------------------------------------------

#[test]
fn symbol_ordering_is_deterministic() {
    let a = Symbol::Terminal(SymbolId(1));
    let b = Symbol::Terminal(SymbolId(2));
    let c = Symbol::NonTerminal(SymbolId(0));

    let mut sorted = [c.clone(), b.clone(), a.clone()];
    sorted.sort();

    // Terminal < NonTerminal by variant discriminant order, then by id
    assert_eq!(sorted[0], a);
    assert_eq!(sorted[1], b);
    assert_eq!(sorted[2], c);
}

#[test]
fn symbol_ordering_in_btreeset() {
    let mut set = BTreeSet::new();
    set.insert(Symbol::Epsilon);
    set.insert(Symbol::Terminal(SymbolId(5)));
    set.insert(Symbol::Terminal(SymbolId(1)));
    set.insert(Symbol::NonTerminal(SymbolId(3)));

    // BTreeSet iterates in sorted order — just verify the count is correct
    assert_eq!(set.len(), 4);
}

// ---------------------------------------------------------------------------
// 6. Hashing — symbols can be used as HashMap keys
// ---------------------------------------------------------------------------

#[test]
fn symbols_usable_as_hashmap_keys() {
    let mut map: HashMap<Symbol, &str> = HashMap::new();
    map.insert(Symbol::Terminal(SymbolId(1)), "plus");
    map.insert(Symbol::NonTerminal(SymbolId(2)), "expr");
    map.insert(Symbol::Epsilon, "eps");

    assert_eq!(map.get(&Symbol::Terminal(SymbolId(1))), Some(&"plus"));
    assert_eq!(map.get(&Symbol::Epsilon), Some(&"eps"));
    assert_eq!(map.get(&Symbol::Terminal(SymbolId(99))), None);
}

#[test]
fn symbols_usable_in_hashset() {
    let mut set = HashSet::new();
    let sym = Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(1))));
    set.insert(sym.clone());
    set.insert(sym.clone()); // duplicate

    assert_eq!(set.len(), 1);
}

// ---------------------------------------------------------------------------
// 7. Clone
// ---------------------------------------------------------------------------

#[test]
fn clone_produces_independent_copy() {
    let original = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Repeat(Box::new(Symbol::NonTerminal(SymbolId(2)))),
    ]);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ---------------------------------------------------------------------------
// 8. Serde serialization round-trip
// ---------------------------------------------------------------------------

#[test]
fn symbol_json_roundtrip_terminal() {
    let sym = Symbol::Terminal(SymbolId(42));
    let json = serde_json::to_string(&sym).expect("serialize");
    let deserialized: Symbol = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(sym, deserialized);
}

#[test]
fn symbol_json_roundtrip_complex_nested() {
    let sym = Symbol::Optional(Box::new(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Sequence(vec![
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::RepeatOne(Box::new(Symbol::External(SymbolId(3)))),
        ]),
        Symbol::Epsilon,
    ])));
    let json = serde_json::to_string(&sym).expect("serialize");
    let deserialized: Symbol = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(sym, deserialized);
}

#[test]
fn symbol_json_roundtrip_epsilon() {
    let sym = Symbol::Epsilon;
    let json = serde_json::to_string(&sym).expect("serialize");
    let deserialized: Symbol = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(sym, deserialized);
}

// ---------------------------------------------------------------------------
// 9. ID types — Display, Eq, Ord, Hash, Serde
// ---------------------------------------------------------------------------

#[test]
fn symbol_id_display() {
    assert_eq!(format!("{}", SymbolId(0)), "Symbol(0)");
    assert_eq!(format!("{}", SymbolId(u16::MAX)), "Symbol(65535)");
}

#[test]
fn rule_id_display() {
    assert_eq!(format!("{}", RuleId(10)), "Rule(10)");
}

#[test]
fn state_id_display() {
    assert_eq!(format!("{}", StateId(255)), "State(255)");
}

#[test]
fn field_id_display() {
    assert_eq!(format!("{}", FieldId(3)), "Field(3)");
}

#[test]
fn production_id_display() {
    assert_eq!(format!("{}", ProductionId(7)), "Production(7)");
}

#[test]
fn id_types_json_roundtrip() {
    let sid = SymbolId(100);
    let json = serde_json::to_string(&sid).unwrap();
    assert_eq!(serde_json::from_str::<SymbolId>(&json).unwrap(), sid);

    let fid = FieldId(55);
    let json = serde_json::to_string(&fid).unwrap();
    assert_eq!(serde_json::from_str::<FieldId>(&json).unwrap(), fid);

    let pid = ProductionId(200);
    let json = serde_json::to_string(&pid).unwrap();
    assert_eq!(serde_json::from_str::<ProductionId>(&json).unwrap(), pid);
}

#[test]
fn symbol_id_ordering() {
    let ids = vec![SymbolId(5), SymbolId(1), SymbolId(3)];
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(sorted, vec![SymbolId(1), SymbolId(3), SymbolId(5)]);
}

#[test]
fn symbol_id_hashing() {
    let mut set = HashSet::new();
    set.insert(SymbolId(10));
    set.insert(SymbolId(20));
    set.insert(SymbolId(10)); // duplicate
    assert_eq!(set.len(), 2);
}

// ---------------------------------------------------------------------------
// 10. SymbolMetadata
// ---------------------------------------------------------------------------

#[test]
fn symbol_metadata_fields() {
    let meta = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    };
    let meta2 = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    };
    assert_eq!(meta, meta2);
}

#[test]
fn symbol_metadata_json_roundtrip() {
    let meta = SymbolMetadata {
        visible: false,
        named: true,
        hidden: true,
        terminal: true,
    };
    let json = serde_json::to_string(&meta).unwrap();
    let deser: SymbolMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(meta, deser);
}

// ---------------------------------------------------------------------------
// 11. PrecedenceKind
// ---------------------------------------------------------------------------

#[test]
fn precedence_kind_variants() {
    let s = PrecedenceKind::Static(5);
    let d = PrecedenceKind::Dynamic(-3);
    assert_ne!(s, d);

    assert_eq!(PrecedenceKind::Static(5), PrecedenceKind::Static(5));
    assert_ne!(PrecedenceKind::Static(1), PrecedenceKind::Static(2));
}

#[test]
fn precedence_kind_serde_roundtrip() {
    let kinds = vec![PrecedenceKind::Static(10), PrecedenceKind::Dynamic(-1)];
    for kind in &kinds {
        let json = serde_json::to_string(kind).unwrap();
        let deser: PrecedenceKind = serde_json::from_str(&json).unwrap();
        assert_eq!(*kind, deser);
    }
}

// ---------------------------------------------------------------------------
// 12. Associativity
// ---------------------------------------------------------------------------

#[test]
fn associativity_variants_distinct() {
    assert_ne!(Associativity::Left, Associativity::Right);
    assert_ne!(Associativity::Left, Associativity::None);
    assert_ne!(Associativity::Right, Associativity::None);
}

// ---------------------------------------------------------------------------
// 13. Rule construction with symbols
// ---------------------------------------------------------------------------

#[test]
fn rule_with_mixed_symbol_types() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(3)))),
        ],
        precedence: Some(PrecedenceKind::Static(1)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0), (FieldId(1), 2)],
        production_id: ProductionId(0),
    };

    assert_eq!(rule.rhs.len(), 3);
    assert_eq!(rule.fields.len(), 2);
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(1)));
}

// ---------------------------------------------------------------------------
// 14. Grammar integration — symbols in rules
// ---------------------------------------------------------------------------

#[test]
fn grammar_add_and_retrieve_rules_with_symbols() {
    let mut grammar = Grammar::new("test_sym".to_string());
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::NonTerminal(SymbolId(0)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.add_rule(rule.clone());

    let retrieved = grammar.get_rules_for_symbol(SymbolId(0));
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().len(), 1);
    assert_eq!(retrieved.unwrap()[0], rule);
}

#[test]
fn grammar_all_rules_iterates_symbols() {
    let mut grammar = Grammar::new("iter".to_string());
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    grammar.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let all: Vec<_> = grammar.all_rules().collect();
    assert_eq!(all.len(), 2);
}

// ---------------------------------------------------------------------------
// 15. Debug formatting
// ---------------------------------------------------------------------------

#[test]
fn debug_format_includes_variant_name() {
    let sym = Symbol::Terminal(SymbolId(5));
    let dbg = format!("{sym:?}");
    assert!(dbg.contains("Terminal"));
    assert!(dbg.contains("5"));

    let eps = Symbol::Epsilon;
    let dbg_eps = format!("{eps:?}");
    assert!(dbg_eps.contains("Epsilon"));
}

// ---------------------------------------------------------------------------
// 16. Empty Choice and Sequence edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_choice_is_valid_construction() {
    let sym = Symbol::Choice(vec![]);
    if let Symbol::Choice(v) = &sym {
        assert_eq!(v.len(), 0);
    } else {
        panic!("expected Choice");
    }
}

#[test]
fn empty_sequence_is_valid_construction() {
    let sym = Symbol::Sequence(vec![]);
    if let Symbol::Sequence(v) = &sym {
        assert_eq!(v.len(), 0);
    } else {
        panic!("expected Sequence");
    }
}

// ---------------------------------------------------------------------------
// 17. Single-element Choice and Sequence
// ---------------------------------------------------------------------------

#[test]
fn single_element_choice() {
    let sym = Symbol::Choice(vec![Symbol::Terminal(SymbolId(9))]);
    if let Symbol::Choice(v) = &sym {
        assert_eq!(v.len(), 1);
        assert_eq!(v[0], Symbol::Terminal(SymbolId(9)));
    } else {
        panic!("expected Choice");
    }
}

#[test]
fn single_element_sequence() {
    let sym = Symbol::Sequence(vec![Symbol::NonTerminal(SymbolId(4))]);
    if let Symbol::Sequence(v) = &sym {
        assert_eq!(v.len(), 1);
    } else {
        panic!("expected Sequence");
    }
}

// ---------------------------------------------------------------------------
// 18. Boundary ID values
// ---------------------------------------------------------------------------

#[test]
fn symbol_id_boundary_values() {
    let zero = SymbolId(0);
    let max = SymbolId(u16::MAX);

    assert_ne!(zero, max);
    assert!(zero < max);

    // Serde roundtrip at boundaries
    for id in [zero, max] {
        let json = serde_json::to_string(&id).unwrap();
        let deser: SymbolId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deser);
    }
}

// ---------------------------------------------------------------------------
// 19. Symbol with all wrappers composed
// ---------------------------------------------------------------------------

#[test]
fn all_wrapper_variants_compose() {
    // RepeatOne(Optional(Repeat(Terminal(1))))
    let sym = Symbol::RepeatOne(Box::new(Symbol::Optional(Box::new(Symbol::Repeat(
        Box::new(Symbol::Terminal(SymbolId(1))),
    )))));

    // Verify via serde roundtrip — structure must survive
    let json = serde_json::to_string(&sym).unwrap();
    let deser: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(sym, deser);
}

// ---------------------------------------------------------------------------
// 20. Grammar serde roundtrip with rules containing complex symbols
// ---------------------------------------------------------------------------

#[test]
fn grammar_serde_roundtrip_with_complex_symbols() {
    let mut grammar = Grammar::new("roundtrip".to_string());
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![
            Symbol::Choice(vec![
                Symbol::Terminal(SymbolId(1)),
                Symbol::Sequence(vec![
                    Symbol::NonTerminal(SymbolId(2)),
                    Symbol::Optional(Box::new(Symbol::Terminal(SymbolId(3)))),
                ]),
            ]),
            Symbol::RepeatOne(Box::new(Symbol::NonTerminal(SymbolId(4)))),
        ],
        precedence: Some(PrecedenceKind::Dynamic(2)),
        associativity: Some(Associativity::Right),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(0),
    });

    let json = serde_json::to_string(&grammar).expect("serialize grammar");
    let deser: Grammar = serde_json::from_str(&json).expect("deserialize grammar");

    assert_eq!(deser.name, "roundtrip");
    let rules = deser.get_rules_for_symbol(SymbolId(0)).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rhs.len(), 2);
}
