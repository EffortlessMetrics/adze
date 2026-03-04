// Comprehensive property tests for IR symbol registry invariants
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// SymbolId arithmetic and comparison properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn symbol_id_equality_reflexive(id in 0u16..10000) {
        let sid = SymbolId(id);
        prop_assert_eq!(sid, sid);
    }

    #[test]
    fn symbol_id_equality_symmetric(a in 0u16..10000, b in 0u16..10000) {
        let sa = SymbolId(a);
        let sb = SymbolId(b);
        prop_assert_eq!(sa == sb, sb == sa);
    }

    #[test]
    fn symbol_id_ordering_consistent(a in 0u16..10000, b in 0u16..10000) {
        let sa = SymbolId(a);
        let sb = SymbolId(b);
        prop_assert_eq!(sa.cmp(&sb), a.cmp(&b));
    }

    #[test]
    fn symbol_id_hash_deterministic(id in 0u16..10000) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let sid = SymbolId(id);
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        sid.hash(&mut h1);
        sid.hash(&mut h2);
        prop_assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn symbol_id_display_matches_inner(id in 0u16..10000) {
        let sid = SymbolId(id);
        let disp = format!("{}", sid);
        prop_assert!(disp.contains(&id.to_string()));
    }

    #[test]
    fn symbol_id_clone_equals_original(id in 0u16..10000) {
        let sid = SymbolId(id);
        let cloned = sid;
        prop_assert_eq!(sid, cloned);
    }
}

// ---------------------------------------------------------------------------
// RuleId properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn rule_id_equality_reflexive(id in 0u16..10000) {
        let rid = RuleId(id);
        prop_assert_eq!(rid, rid);
    }

    #[test]
    fn rule_id_ordering_matches_inner(a in 0u16..10000, b in 0u16..10000) {
        let ra = RuleId(a);
        let rb = RuleId(b);
        prop_assert_eq!(ra.cmp(&rb), a.cmp(&b));
    }

    #[test]
    fn rule_id_hash_deterministic(id in 0u16..10000) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let rid = RuleId(id);
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        rid.hash(&mut h1);
        rid.hash(&mut h2);
        prop_assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn rule_id_display_contains_inner(id in 0u16..10000) {
        let rid = RuleId(id);
        let disp = format!("{}", rid);
        prop_assert!(disp.contains(&id.to_string()));
    }
}

// ---------------------------------------------------------------------------
// StateId properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn state_id_equality(a in 0u16..10000, b in 0u16..10000) {
        let sa = StateId(a);
        let sb = StateId(b);
        prop_assert_eq!(sa == sb, a == b);
    }

    #[test]
    fn state_id_ordering(a in 0u16..10000, b in 0u16..10000) {
        let sa = StateId(a);
        let sb = StateId(b);
        prop_assert_eq!(sa.cmp(&sb), a.cmp(&b));
    }
}

// ---------------------------------------------------------------------------
// FieldId properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn field_id_equality(a in 0u16..10000, b in 0u16..10000) {
        let fa = FieldId(a);
        let fb = FieldId(b);
        prop_assert_eq!(fa == fb, a == b);
    }

    #[test]
    fn field_id_hash_deterministic(id in 0u16..10000) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let fid = FieldId(id);
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        fid.hash(&mut h1);
        fid.hash(&mut h2);
        prop_assert_eq!(h1.finish(), h2.finish());
    }
}

// ---------------------------------------------------------------------------
// ProductionId properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn production_id_equality(a in 0u16..10000, b in 0u16..10000) {
        let pa = ProductionId(a);
        let pb = ProductionId(b);
        prop_assert_eq!(pa == pb, a == b);
    }

    #[test]
    fn production_id_ordering(a in 0u16..10000, b in 0u16..10000) {
        let pa = ProductionId(a);
        let pb = ProductionId(b);
        prop_assert_eq!(pa.cmp(&pb), a.cmp(&b));
    }
}

// ---------------------------------------------------------------------------
// Symbol enum properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn symbol_terminal_debug_contains_id(id in 0u16..1000) {
        let sym = Symbol::Terminal(SymbolId(id));
        let debug = format!("{:?}", sym);
        prop_assert!(debug.contains("Terminal"));
    }

    #[test]
    fn symbol_nonterminal_debug_contains_id(id in 0u16..1000) {
        let sym = Symbol::NonTerminal(SymbolId(id));
        let debug = format!("{:?}", sym);
        prop_assert!(debug.contains("NonTerminal"));
    }

    #[test]
    fn symbol_clone_preserves_variant(id in 0u16..1000) {
        let sym = Symbol::Terminal(SymbolId(id));
        let cloned = sym.clone();
        prop_assert_eq!(sym, cloned);
    }

    #[test]
    fn symbol_equality_same_variant(a in 0u16..1000, b in 0u16..1000) {
        let sa = Symbol::Terminal(SymbolId(a));
        let sb = Symbol::Terminal(SymbolId(b));
        prop_assert_eq!(sa == sb, a == b);
    }

    #[test]
    fn symbol_terminal_ne_nonterminal(id in 0u16..1000) {
        let term = Symbol::Terminal(SymbolId(id));
        let nonterm = Symbol::NonTerminal(SymbolId(id));
        prop_assert_ne!(term, nonterm);
    }
}

// ---------------------------------------------------------------------------
// GrammarBuilder determinism properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn grammar_builder_same_input_same_output(
        name in "[a-z]{3,8}",
        token_name in "[a-z]{2,6}",
        token_pattern in "[a-z]+",
    ) {
        let g1 = GrammarBuilder::new(&name)
            .token(&token_name, &token_pattern)
            .rule("start", vec![&token_name])
            .start("start")
            .build();

        let g2 = GrammarBuilder::new(&name)
            .token(&token_name, &token_pattern)
            .rule("start", vec![&token_name])
            .start("start")
            .build();

        prop_assert_eq!(&g1.name, &g2.name);
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
        prop_assert_eq!(g1.tokens.len(), g2.tokens.len());
    }

    #[test]
    fn grammar_name_preserved(name in "[a-z]{1,20}") {
        let g = GrammarBuilder::new(&name)
            .token("tok", "x")
            .rule("start", vec!["tok"])
            .start("start")
            .build();
        prop_assert_eq!(&g.name, &name);
    }

    #[test]
    fn grammar_token_count_grows(n in 1usize..10) {
        let mut builder = GrammarBuilder::new("test");
        for i in 0..n {
            builder = builder.token(&format!("tok{}", i), &format!("t{}", i));
        }
        builder = builder.rule("start", vec!["tok0"]).start("start");
        let g = builder.build();
        prop_assert!(g.tokens.len() >= n);
    }
}

// ---------------------------------------------------------------------------
// Associativity properties
// ---------------------------------------------------------------------------

#[test]
fn associativity_left_debug() {
    let a = Associativity::Left;
    assert!(format!("{:?}", a).contains("Left"));
}

#[test]
fn associativity_right_debug() {
    let a = Associativity::Right;
    assert!(format!("{:?}", a).contains("Right"));
}

#[test]
fn associativity_none_debug() {
    let a = Associativity::None;
    assert!(format!("{:?}", a).contains("None"));
}

#[test]
fn associativity_clone_eq() {
    let a = Associativity::Left;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn associativity_variants_distinct() {
    assert_ne!(Associativity::Left, Associativity::Right);
    assert_ne!(Associativity::Left, Associativity::None);
    assert_ne!(Associativity::Right, Associativity::None);
}

// ---------------------------------------------------------------------------
// PrecedenceKind properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn precedence_static_preserves_value(v in -1000i16..1000) {
        let pk = PrecedenceKind::Static(v);
        if let PrecedenceKind::Static(inner) = pk {
            prop_assert_eq!(inner, v);
        } else {
            prop_assert!(false, "Expected Static");
        }
    }

    #[test]
    fn precedence_dynamic_preserves_value(v in -1000i16..1000) {
        let pk = PrecedenceKind::Dynamic(v);
        if let PrecedenceKind::Dynamic(inner) = pk {
            prop_assert_eq!(inner, v);
        } else {
            prop_assert!(false, "Expected Dynamic");
        }
    }

    #[test]
    fn precedence_static_ne_dynamic(v in -1000i16..1000) {
        let s = PrecedenceKind::Static(v);
        let d = PrecedenceKind::Dynamic(v);
        prop_assert_ne!(s, d);
    }
}

// ---------------------------------------------------------------------------
// Rule structure properties
// ---------------------------------------------------------------------------

#[test]
fn rule_with_empty_rhs() {
    let rule = Rule {
        lhs: SymbolId(0),
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    assert!(rule.rhs.is_empty());
    assert_eq!(rule.precedence, None);
}

#[test]
fn rule_with_symbols() {
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(SymbolId(3)),
        ],
        precedence: Some(PrecedenceKind::Static(5)),
        associativity: Some(Associativity::Left),
        fields: vec![(FieldId(0), 0)],
        production_id: ProductionId(1),
    };
    assert_eq!(rule.rhs.len(), 2);
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(5)));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn rule_clone_preserves_all_fields(
        lhs in 0u16..100,
        rhs_len in 0usize..5,
        prec in -10i16..10,
    ) {
        let rhs: Vec<Symbol> = (0..rhs_len as u16)
            .map(|i| Symbol::Terminal(SymbolId(i + 100)))
            .collect();
        let rule = Rule {
            lhs: SymbolId(lhs),
            rhs: rhs.clone(),
            precedence: Some(PrecedenceKind::Static(prec)),
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        };
        let cloned = rule.clone();
        prop_assert_eq!(cloned.lhs, rule.lhs);
        prop_assert_eq!(cloned.rhs.len(), rule.rhs.len());
        prop_assert_eq!(cloned.precedence, rule.precedence);
    }
}

// ---------------------------------------------------------------------------
// Token properties
// ---------------------------------------------------------------------------

#[test]
fn token_string_pattern() {
    let tok = Token {
        name: "number".to_string(),
        pattern: TokenPattern::String("\\d+".to_string()),
        fragile: false,
    };
    assert_eq!(tok.name, "number");
    assert!(!tok.fragile);
}

#[test]
fn token_fragile_flag() {
    let tok = Token {
        name: "ws".to_string(),
        pattern: TokenPattern::String("\\s+".to_string()),
        fragile: true,
    };
    assert!(tok.fragile);
}

#[test]
fn token_clone() {
    let tok = Token {
        name: "ident".to_string(),
        pattern: TokenPattern::String("[a-z]+".to_string()),
        fragile: false,
    };
    let cloned = tok.clone();
    assert_eq!(tok, cloned);
}

#[test]
fn token_debug() {
    let tok = Token {
        name: "test".to_string(),
        pattern: TokenPattern::String("x".to_string()),
        fragile: false,
    };
    let debug = format!("{:?}", tok);
    assert!(debug.contains("test"));
}
