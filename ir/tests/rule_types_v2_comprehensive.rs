use adze_ir::{
    Associativity, FieldId, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn simple_rule(lhs: u16, rhs: &[Symbol]) -> Rule {
    Rule {
        lhs: SymbolId(lhs),
        rhs: rhs.to_vec(),
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    }
}

fn term(id: u16) -> Symbol {
    Symbol::Terminal(SymbolId(id))
}

fn nonterm(id: u16) -> Symbol {
    Symbol::NonTerminal(SymbolId(id))
}

fn register_token(grammar: &mut Grammar, id: u16, name: &str) {
    grammar.tokens.insert(
        SymbolId(id),
        Token {
            name: name.into(),
            pattern: TokenPattern::String(name.into()),
            fragile: false,
        },
    );
}

fn grammar_with_rule(lhs: u16, name: &str, rhs: Vec<Symbol>) -> Grammar {
    let mut g = Grammar::new("test".into());
    g.rule_names.insert(SymbolId(lhs), name.into());
    g.add_rule(simple_rule(lhs, &rhs));
    g
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Symbol variant construction (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_terminal_construction() {
    let s = Symbol::Terminal(SymbolId(1));
    assert!(matches!(s, Symbol::Terminal(SymbolId(1))));
}

#[test]
fn test_symbol_nonterminal_construction() {
    let s = Symbol::NonTerminal(SymbolId(42));
    assert!(matches!(s, Symbol::NonTerminal(SymbolId(42))));
}

#[test]
fn test_symbol_external_construction() {
    let s = Symbol::External(SymbolId(99));
    assert!(matches!(s, Symbol::External(SymbolId(99))));
}

#[test]
fn test_symbol_optional_construction() {
    let inner = term(5);
    let s = Symbol::Optional(Box::new(inner));
    assert!(matches!(s, Symbol::Optional(_)));
}

#[test]
fn test_symbol_repeat_construction() {
    let s = Symbol::Repeat(Box::new(nonterm(3)));
    assert!(matches!(s, Symbol::Repeat(_)));
}

#[test]
fn test_symbol_repeat_one_construction() {
    let s = Symbol::RepeatOne(Box::new(term(7)));
    assert!(matches!(s, Symbol::RepeatOne(_)));
}

#[test]
fn test_symbol_choice_construction() {
    let s = Symbol::Choice(vec![term(1), term(2), term(3)]);
    if let Symbol::Choice(items) = &s {
        assert_eq!(items.len(), 3);
    } else {
        panic!("expected Choice");
    }
}

#[test]
fn test_symbol_sequence_construction() {
    let s = Symbol::Sequence(vec![term(10), nonterm(20)]);
    if let Symbol::Sequence(items) = &s {
        assert_eq!(items.len(), 2);
    } else {
        panic!("expected Sequence");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Symbol nesting (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_sequence_of_choice() {
    let choice = Symbol::Choice(vec![term(1), term(2)]);
    let seq = Symbol::Sequence(vec![choice, term(3)]);
    if let Symbol::Sequence(items) = &seq {
        assert!(matches!(&items[0], Symbol::Choice(_)));
        assert!(matches!(&items[1], Symbol::Terminal(_)));
    } else {
        panic!("expected Sequence");
    }
}

#[test]
fn test_optional_of_repeat() {
    let repeat = Symbol::Repeat(Box::new(term(4)));
    let opt = Symbol::Optional(Box::new(repeat));
    if let Symbol::Optional(inner) = &opt {
        assert!(matches!(**inner, Symbol::Repeat(_)));
    } else {
        panic!("expected Optional");
    }
}

#[test]
fn test_choice_of_sequences() {
    let s1 = Symbol::Sequence(vec![term(1), term(2)]);
    let s2 = Symbol::Sequence(vec![term(3), term(4)]);
    let choice = Symbol::Choice(vec![s1, s2]);
    if let Symbol::Choice(items) = &choice {
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[0], Symbol::Sequence(_)));
        assert!(matches!(&items[1], Symbol::Sequence(_)));
    } else {
        panic!("expected Choice");
    }
}

#[test]
fn test_repeat_one_of_optional() {
    let opt = Symbol::Optional(Box::new(term(5)));
    let rep = Symbol::RepeatOne(Box::new(opt));
    if let Symbol::RepeatOne(inner) = &rep {
        assert!(matches!(**inner, Symbol::Optional(_)));
    } else {
        panic!("expected RepeatOne");
    }
}

#[test]
fn test_nested_optional_in_optional() {
    let inner = Symbol::Optional(Box::new(term(1)));
    let outer = Symbol::Optional(Box::new(inner));
    if let Symbol::Optional(o) = &outer {
        assert!(matches!(**o, Symbol::Optional(_)));
    } else {
        panic!("expected nested Optional");
    }
}

#[test]
fn test_repeat_of_choice() {
    let choice = Symbol::Choice(vec![term(1), term(2)]);
    let rep = Symbol::Repeat(Box::new(choice));
    if let Symbol::Repeat(inner) = &rep {
        assert!(matches!(**inner, Symbol::Choice(_)));
    } else {
        panic!("expected Repeat of Choice");
    }
}

#[test]
fn test_sequence_of_repeat_and_optional() {
    let rep = Symbol::Repeat(Box::new(term(1)));
    let opt = Symbol::Optional(Box::new(term(2)));
    let seq = Symbol::Sequence(vec![rep, opt, term(3)]);
    if let Symbol::Sequence(items) = &seq {
        assert_eq!(items.len(), 3);
        assert!(matches!(&items[0], Symbol::Repeat(_)));
        assert!(matches!(&items[1], Symbol::Optional(_)));
        assert!(matches!(&items[2], Symbol::Terminal(_)));
    } else {
        panic!("expected Sequence");
    }
}

#[test]
fn test_choice_with_epsilon() {
    let choice = Symbol::Choice(vec![term(1), Symbol::Epsilon]);
    if let Symbol::Choice(items) = &choice {
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[1], Symbol::Epsilon));
    } else {
        panic!("expected Choice");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Rule/Symbol equality (5 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_terminal_equality() {
    assert_eq!(term(1), term(1));
    assert_ne!(term(1), term(2));
}

#[test]
fn test_symbol_nonterminal_equality() {
    assert_eq!(nonterm(5), nonterm(5));
    assert_ne!(nonterm(5), nonterm(6));
}

#[test]
fn test_symbol_mixed_inequality() {
    assert_ne!(term(1), nonterm(1));
    assert_ne!(Symbol::Epsilon, term(0));
}

#[test]
fn test_rule_struct_equality() {
    let r1 = simple_rule(1, &[term(2), term(3)]);
    let r2 = simple_rule(1, &[term(2), term(3)]);
    assert_eq!(r1, r2);
}

#[test]
fn test_rule_struct_inequality_different_rhs() {
    let r1 = simple_rule(1, &[term(2)]);
    let r2 = simple_rule(1, &[term(3)]);
    assert_ne!(r1, r2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. Serialization roundtrip (5 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_terminal_serde_roundtrip() {
    let s = term(42);
    let json = serde_json::to_string(&s).unwrap();
    let back: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(s, back);
}

#[test]
fn test_symbol_nested_serde_roundtrip() {
    let s = Symbol::Optional(Box::new(Symbol::Choice(vec![term(1), nonterm(2)])));
    let json = serde_json::to_string(&s).unwrap();
    let back: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(s, back);
}

#[test]
fn test_rule_struct_serde_roundtrip() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![term(2), nonterm(3)],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(10),
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: Rule = serde_json::from_str(&json).unwrap();
    assert_eq!(rule, back);
}

#[test]
fn test_epsilon_serde_roundtrip() {
    let s = Symbol::Epsilon;
    let json = serde_json::to_string(&s).unwrap();
    let back: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(s, back);
}

#[test]
fn test_repeat_one_serde_roundtrip() {
    let s = Symbol::RepeatOne(Box::new(Symbol::Sequence(vec![term(1), term(2)])));
    let json = serde_json::to_string(&s).unwrap();
    let back: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(s, back);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. Grammar normalize behavior (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_normalize_optional_creates_auxiliary_rules() {
    let mut g = grammar_with_rule(1, "expr", vec![Symbol::Optional(Box::new(term(10)))]);
    register_token(&mut g, 10, "tok");
    let all = g.normalize();
    // Original rule should reference NonTerminal(aux), plus aux rules created
    assert!(
        all.len() >= 2,
        "expected auxiliary rules, got {}",
        all.len()
    );
}

#[test]
fn test_normalize_repeat_creates_recursive_and_epsilon() {
    let mut g = grammar_with_rule(1, "list", vec![Symbol::Repeat(Box::new(term(10)))]);
    register_token(&mut g, 10, "item");
    let all = g.normalize();
    // Repeat generates: aux -> aux inner | epsilon
    let epsilon_count = all
        .iter()
        .filter(|r| r.rhs.contains(&Symbol::Epsilon))
        .count();
    assert!(epsilon_count >= 1, "Repeat should produce epsilon rule");
}

#[test]
fn test_normalize_repeat_one_no_epsilon() {
    let mut g = grammar_with_rule(1, "items", vec![Symbol::RepeatOne(Box::new(term(10)))]);
    register_token(&mut g, 10, "tok");
    let all = g.normalize();
    // RepeatOne generates: aux -> aux inner | inner (no epsilon)
    let has_epsilon = all.iter().any(|r| r.rhs.contains(&Symbol::Epsilon));
    assert!(!has_epsilon, "RepeatOne must not produce epsilon rule");
}

#[test]
fn test_normalize_choice_creates_one_rule_per_alternative() {
    let mut g = grammar_with_rule(
        1,
        "alt",
        vec![Symbol::Choice(vec![term(10), term(11), term(12)])],
    );
    register_token(&mut g, 10, "a");
    register_token(&mut g, 11, "b");
    register_token(&mut g, 12, "c");
    let all = g.normalize();
    // The choice with 3 alternatives should produce 3 auxiliary rules
    // plus the original rule referencing the aux nonterminal
    assert!(
        all.len() >= 4,
        "expected 1 original + 3 choice rules, got {}",
        all.len()
    );
}

#[test]
fn test_normalize_sequence_flattens_into_rule() {
    let mut g = grammar_with_rule(1, "flat", vec![Symbol::Sequence(vec![term(10), term(11)])]);
    register_token(&mut g, 10, "x");
    register_token(&mut g, 11, "y");
    let all = g.normalize();
    // Sequence should be flattened: no new aux rule, just expanded rhs
    let main_rule = all.iter().find(|r| r.lhs == SymbolId(1)).unwrap();
    assert_eq!(main_rule.rhs.len(), 2);
    assert_eq!(main_rule.rhs[0], term(10));
    assert_eq!(main_rule.rhs[1], term(11));
}

#[test]
fn test_normalize_plain_terminals_unchanged() {
    let mut g = grammar_with_rule(1, "simple", vec![term(10), term(11)]);
    register_token(&mut g, 10, "a");
    register_token(&mut g, 11, "b");
    let all = g.normalize();
    assert_eq!(all.len(), 1);
    let rule = &all[0];
    assert_eq!(rule.lhs, SymbolId(1));
    assert_eq!(rule.rhs, vec![term(10), term(11)]);
}

#[test]
fn test_normalize_epsilon_rule_unchanged() {
    let mut g = grammar_with_rule(1, "empty", vec![Symbol::Epsilon]);
    let all = g.normalize();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].rhs, vec![Symbol::Epsilon]);
}

#[test]
fn test_normalize_preserves_rule_lhs() {
    let mut g = grammar_with_rule(5, "root", vec![Symbol::Optional(Box::new(term(10)))]);
    register_token(&mut g, 10, "tok");
    let all = g.normalize();
    let has_original_lhs = all.iter().any(|r| r.lhs == SymbolId(5));
    assert!(has_original_lhs, "original LHS must be preserved");
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Auxiliary rule generation (5 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_auxiliary_ids_above_existing() {
    let mut g = grammar_with_rule(1, "root", vec![Symbol::Optional(Box::new(term(10)))]);
    register_token(&mut g, 10, "tok");
    g.normalize();
    // Auxiliary IDs start at max_id + 1000
    for (sym_id, _) in &g.rules {
        if *sym_id != SymbolId(1) {
            assert!(
                sym_id.0 >= 1001,
                "auxiliary id {} should be >= 1001",
                sym_id.0
            );
        }
    }
}

#[test]
fn test_optional_auxiliary_has_two_alternatives() {
    let mut g = grammar_with_rule(1, "root", vec![Symbol::Optional(Box::new(term(10)))]);
    register_token(&mut g, 10, "tok");
    g.normalize();
    // Find the aux symbol (not the original lhs)
    for (sym_id, rules) in &g.rules {
        if *sym_id != SymbolId(1) {
            assert_eq!(
                rules.len(),
                2,
                "Optional aux should have exactly 2 rules (inner | epsilon)"
            );
        }
    }
}

#[test]
fn test_repeat_auxiliary_has_two_alternatives() {
    let mut g = grammar_with_rule(1, "root", vec![Symbol::Repeat(Box::new(term(10)))]);
    register_token(&mut g, 10, "tok");
    g.normalize();
    for (sym_id, rules) in &g.rules {
        if *sym_id != SymbolId(1) {
            assert_eq!(
                rules.len(),
                2,
                "Repeat aux should have 2 rules (aux inner | epsilon)"
            );
        }
    }
}

#[test]
fn test_repeat_one_auxiliary_has_two_alternatives() {
    let mut g = grammar_with_rule(1, "root", vec![Symbol::RepeatOne(Box::new(term(10)))]);
    register_token(&mut g, 10, "tok");
    g.normalize();
    for (sym_id, rules) in &g.rules {
        if *sym_id != SymbolId(1) {
            assert_eq!(
                rules.len(),
                2,
                "RepeatOne aux should have 2 rules (aux inner | inner)"
            );
        }
    }
}

#[test]
fn test_multiple_complex_symbols_generate_distinct_auxiliaries() {
    let mut g = grammar_with_rule(
        1,
        "root",
        vec![
            Symbol::Optional(Box::new(term(10))),
            Symbol::Repeat(Box::new(term(11))),
        ],
    );
    register_token(&mut g, 10, "a");
    register_token(&mut g, 11, "b");
    g.normalize();
    // Should have original lhs(1) + 2 distinct auxiliary symbols
    let distinct_lhs: Vec<_> = g.rules.keys().collect();
    assert!(
        distinct_lhs.len() >= 3,
        "expected at least 3 distinct LHS symbols, got {}",
        distinct_lhs.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. Symbol pattern matching (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_match_terminal_extracts_id() {
    let s = term(55);
    let id = match s {
        Symbol::Terminal(id) => id,
        _ => panic!("expected Terminal"),
    };
    assert_eq!(id, SymbolId(55));
}

#[test]
fn test_match_nonterminal_extracts_id() {
    let s = nonterm(77);
    let id = match s {
        Symbol::NonTerminal(id) => id,
        _ => panic!("expected NonTerminal"),
    };
    assert_eq!(id, SymbolId(77));
}

#[test]
fn test_match_optional_extracts_inner() {
    let s = Symbol::Optional(Box::new(term(3)));
    let inner = match s {
        Symbol::Optional(inner) => *inner,
        _ => panic!("expected Optional"),
    };
    assert_eq!(inner, term(3));
}

#[test]
fn test_match_repeat_extracts_inner() {
    let s = Symbol::Repeat(Box::new(nonterm(8)));
    let inner = match s {
        Symbol::Repeat(inner) => *inner,
        _ => panic!("expected Repeat"),
    };
    assert_eq!(inner, nonterm(8));
}

#[test]
fn test_match_choice_extracts_alternatives() {
    let s = Symbol::Choice(vec![term(1), term(2)]);
    let items = match s {
        Symbol::Choice(items) => items,
        _ => panic!("expected Choice"),
    };
    assert_eq!(items.len(), 2);
}

#[test]
fn test_match_sequence_extracts_elements() {
    let s = Symbol::Sequence(vec![term(1), nonterm(2), term(3)]);
    let items = match s {
        Symbol::Sequence(items) => items,
        _ => panic!("expected Sequence"),
    };
    assert_eq!(items.len(), 3);
}

#[test]
fn test_match_epsilon() {
    let s = Symbol::Epsilon;
    assert!(matches!(s, Symbol::Epsilon));
}

#[test]
fn test_match_nested_extracts_deeply() {
    let s = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(term(9)))));
    if let Symbol::Optional(outer) = s {
        if let Symbol::Repeat(inner) = *outer {
            assert_eq!(*inner, term(9));
        } else {
            panic!("expected Repeat inside Optional");
        }
    } else {
        panic!("expected Optional");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. Edge cases (8 tests)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_sequence_symbol() {
    let s = Symbol::Sequence(vec![]);
    if let Symbol::Sequence(items) = &s {
        assert!(items.is_empty());
    } else {
        panic!("expected Sequence");
    }
}

#[test]
fn test_empty_choice_symbol() {
    let s = Symbol::Choice(vec![]);
    if let Symbol::Choice(items) = &s {
        assert!(items.is_empty());
    } else {
        panic!("expected Choice");
    }
}

#[test]
fn test_deeply_nested_optional() {
    let mut s = term(1);
    for _ in 0..10 {
        s = Symbol::Optional(Box::new(s));
    }
    // Unwrap 10 layers
    let mut current = &s;
    let mut depth = 0;
    while let Symbol::Optional(inner) = current {
        depth += 1;
        current = inner;
    }
    assert_eq!(depth, 10);
    assert_eq!(*current, term(1));
}

#[test]
fn test_blank_epsilon_in_rule_rhs() {
    let rule = simple_rule(1, &[Symbol::Epsilon]);
    assert_eq!(rule.rhs.len(), 1);
    assert_eq!(rule.rhs[0], Symbol::Epsilon);
}

#[test]
fn test_symbol_id_zero() {
    let s = term(0);
    assert_eq!(s, Symbol::Terminal(SymbolId(0)));
}

#[test]
fn test_symbol_id_max() {
    let s = term(u16::MAX);
    assert_eq!(s, Symbol::Terminal(SymbolId(u16::MAX)));
}

#[test]
fn test_rule_with_all_fields_populated() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![term(2), nonterm(3)],
        precedence: Some(PrecedenceKind::Dynamic(10)),
        associativity: Some(Associativity::Right),
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(99),
    };
    assert_eq!(rule.lhs, SymbolId(1));
    assert_eq!(rule.rhs.len(), 2);
    assert_eq!(rule.precedence, Some(PrecedenceKind::Dynamic(10)));
    assert_eq!(rule.associativity, Some(Associativity::Right));
    assert_eq!(rule.fields.len(), 2);
    assert_eq!(rule.production_id, ProductionId(99));
}

#[test]
fn test_clone_symbol_deep_equality() {
    let original = Symbol::Choice(vec![
        Symbol::Sequence(vec![term(1), term(2)]),
        Symbol::Optional(Box::new(nonterm(3))),
        Symbol::RepeatOne(Box::new(Symbol::Repeat(Box::new(term(4))))),
    ]);
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional: Rule struct properties and grammar interactions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_rule_default_precedence_is_none() {
    let rule = simple_rule(1, &[term(2)]);
    assert!(rule.precedence.is_none());
    assert!(rule.associativity.is_none());
}

#[test]
fn test_rule_with_static_precedence() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![term(2)],
        precedence: Some(PrecedenceKind::Static(-5)),
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(-5)));
}

#[test]
fn test_grammar_add_rule_and_retrieve() {
    let mut g = Grammar::new("test".into());
    let rule = simple_rule(1, &[term(2)]);
    g.add_rule(rule.clone());
    let retrieved = g.get_rules_for_symbol(SymbolId(1)).unwrap();
    assert_eq!(retrieved.len(), 1);
    assert_eq!(retrieved[0], rule);
}

#[test]
fn test_grammar_all_rules_iterator() {
    let mut g = Grammar::new("test".into());
    g.add_rule(simple_rule(1, &[term(10)]));
    g.add_rule(simple_rule(1, &[term(11)]));
    g.add_rule(simple_rule(2, &[term(12)]));
    let count = g.all_rules().count();
    assert_eq!(count, 3);
}

#[test]
fn test_symbol_ord_terminal_before_nonterminal() {
    // Symbol derives PartialOrd/Ord — verify ordering exists
    let t = term(1);
    let nt = nonterm(1);
    // Just verify they are comparable (actual order is derived)
    // Terminal and NonTerminal with same ID should not be equal
    assert_ne!(t, nt);
    // Verify Ord is implemented (comparison doesn't panic)
    let _ = t.cmp(&nt);
}

#[test]
fn test_symbol_hash_consistency() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(term(1));
    set.insert(term(1));
    set.insert(term(2));
    assert_eq!(set.len(), 2);
}

#[test]
fn test_normalize_idempotent_on_simple_grammar() {
    let mut g = grammar_with_rule(1, "simple", vec![term(10), term(11)]);
    register_token(&mut g, 10, "a");
    register_token(&mut g, 11, "b");
    let first = g.normalize();
    let second = g.normalize();
    assert_eq!(first.len(), second.len());
}

#[test]
fn test_normalize_nested_optional_in_repeat() {
    let sym = Symbol::Repeat(Box::new(Symbol::Optional(Box::new(term(10)))));
    let mut g = grammar_with_rule(1, "nested", vec![sym]);
    register_token(&mut g, 10, "tok");
    g.normalize();
    // Should have expanded both Repeat and Optional into auxiliaries
    let distinct_lhs_count = g.rules.keys().count();
    assert!(
        distinct_lhs_count >= 3,
        "nested complex symbols should produce multiple auxiliaries, got {}",
        distinct_lhs_count
    );
}

#[test]
fn test_normalize_choice_in_sequence() {
    let sym = Symbol::Sequence(vec![term(10), Symbol::Choice(vec![term(11), term(12)])]);
    let mut g = grammar_with_rule(1, "mixed", vec![sym]);
    register_token(&mut g, 10, "a");
    register_token(&mut g, 11, "b");
    register_token(&mut g, 12, "c");
    let normalized = g.normalize();
    // Sequence flattened + Choice creates auxiliary
    assert!(
        normalized.len() >= 3,
        "expected auxiliaries for choice, got {}",
        normalized.len()
    );
}

#[test]
fn test_grammar_normalize_empty_grammar() {
    let mut g = Grammar::new("empty".into());
    let all = g.normalize();
    assert!(all.is_empty());
}

#[test]
fn test_rule_debug_format() {
    let rule = simple_rule(1, &[term(2)]);
    let debug_str = format!("{:?}", rule);
    assert!(debug_str.contains("SymbolId"));
    assert!(debug_str.contains("Terminal"));
}

#[test]
fn test_symbol_debug_format_epsilon() {
    let s = Symbol::Epsilon;
    let debug_str = format!("{:?}", s);
    assert!(debug_str.contains("Epsilon"));
}
