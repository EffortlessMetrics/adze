//! Property-based tests for adze-ir core types and GrammarBuilder.

use proptest::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// 1. SymbolId / RuleId / StateId Ord properties — 15 tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_id_ord_reflexive(a in 0u16..1000) {
        let id = SymbolId(a);
        prop_assert!(id <= id);
        prop_assert!(id >= id);
    }

    #[test]
    fn symbol_id_ord_antisymmetric(a in 0u16..1000, b in 0u16..1000) {
        let x = SymbolId(a);
        let y = SymbolId(b);
        if x <= y && y <= x {
            prop_assert_eq!(x, y);
        }
    }

    #[test]
    fn symbol_id_ord_transitive(a in 0u16..500, b in 0u16..500, c in 0u16..500) {
        let mut vals = [SymbolId(a), SymbolId(b), SymbolId(c)];
        vals.sort();
        prop_assert!(vals[0] <= vals[1]);
        prop_assert!(vals[1] <= vals[2]);
        prop_assert!(vals[0] <= vals[2]);
    }

    #[test]
    fn symbol_id_eq_consistency(a in 0u16..1000) {
        let x = SymbolId(a);
        let y = SymbolId(a);
        prop_assert_eq!(x, y);
        prop_assert!(x <= y);
        prop_assert!(x >= y);
    }

    #[test]
    fn symbol_id_ne_strict(a in 0u16..999) {
        let x = SymbolId(a);
        let y = SymbolId(a + 1);
        prop_assert_ne!(x, y);
        prop_assert!(x < y);
    }

    #[test]
    fn rule_id_ord_reflexive(a in 0u16..1000) {
        let id = RuleId(a);
        prop_assert!(id <= id);
        prop_assert!(id >= id);
    }

    #[test]
    fn rule_id_ord_antisymmetric(a in 0u16..1000, b in 0u16..1000) {
        let x = RuleId(a);
        let y = RuleId(b);
        if x <= y && y <= x {
            prop_assert_eq!(x, y);
        }
    }

    #[test]
    fn rule_id_ord_transitive(a in 0u16..500, b in 0u16..500, c in 0u16..500) {
        let mut vals = [RuleId(a), RuleId(b), RuleId(c)];
        vals.sort();
        prop_assert!(vals[0] <= vals[1]);
        prop_assert!(vals[1] <= vals[2]);
        prop_assert!(vals[0] <= vals[2]);
    }

    #[test]
    fn rule_id_eq_consistency(a in 0u16..1000) {
        let x = RuleId(a);
        let y = RuleId(a);
        prop_assert_eq!(x, y);
    }

    #[test]
    fn rule_id_ne_strict(a in 0u16..999) {
        let x = RuleId(a);
        let y = RuleId(a + 1);
        prop_assert_ne!(x, y);
        prop_assert!(x < y);
    }

    #[test]
    fn state_id_ord_reflexive(a in 0u16..1000) {
        let id = StateId(a);
        prop_assert!(id <= id);
        prop_assert!(id >= id);
    }

    #[test]
    fn state_id_ord_antisymmetric(a in 0u16..1000, b in 0u16..1000) {
        let x = StateId(a);
        let y = StateId(b);
        if x <= y && y <= x {
            prop_assert_eq!(x, y);
        }
    }

    #[test]
    fn state_id_ord_transitive(a in 0u16..500, b in 0u16..500, c in 0u16..500) {
        let mut vals = [StateId(a), StateId(b), StateId(c)];
        vals.sort();
        prop_assert!(vals[0] <= vals[1]);
        prop_assert!(vals[1] <= vals[2]);
        prop_assert!(vals[0] <= vals[2]);
    }

    #[test]
    fn production_id_ord_reflexive(a in 0u16..1000) {
        let id = ProductionId(a);
        prop_assert!(id <= id);
        prop_assert!(id >= id);
    }

    #[test]
    fn production_id_ord_transitive(a in 0u16..500, b in 0u16..500, c in 0u16..500) {
        let mut vals = [ProductionId(a), ProductionId(b), ProductionId(c)];
        vals.sort();
        prop_assert!(vals[0] <= vals[1]);
        prop_assert!(vals[1] <= vals[2]);
        prop_assert!(vals[0] <= vals[2]);
    }
}

// ---------------------------------------------------------------------------
// 2. Symbol enum Debug / Clone / Eq reflexivity — 8 tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_terminal_clone_eq(a in 0u16..1000) {
        let s = Symbol::Terminal(SymbolId(a));
        let c = s.clone();
        prop_assert_eq!(&s, &c);
    }

    #[test]
    fn symbol_nonterminal_clone_eq(a in 0u16..1000) {
        let s = Symbol::NonTerminal(SymbolId(a));
        let c = s.clone();
        prop_assert_eq!(&s, &c);
    }

    #[test]
    fn symbol_external_clone_eq(a in 0u16..1000) {
        let s = Symbol::External(SymbolId(a));
        let c = s.clone();
        prop_assert_eq!(&s, &c);
    }

    #[test]
    fn symbol_epsilon_eq_reflexive(_dummy in 0u8..1) {
        let s = Symbol::Epsilon;
        prop_assert_eq!(&s, &Symbol::Epsilon);
    }

    #[test]
    fn symbol_optional_clone_eq(a in 0u16..1000) {
        let inner = Symbol::Terminal(SymbolId(a));
        let s = Symbol::Optional(Box::new(inner));
        let c = s.clone();
        prop_assert_eq!(&s, &c);
    }

    #[test]
    fn symbol_repeat_clone_eq(a in 0u16..1000) {
        let inner = Symbol::NonTerminal(SymbolId(a));
        let s = Symbol::Repeat(Box::new(inner));
        let c = s.clone();
        prop_assert_eq!(&s, &c);
    }

    #[test]
    fn symbol_debug_nonempty(a in 0u16..1000) {
        let s = Symbol::Terminal(SymbolId(a));
        let dbg = format!("{:?}", s);
        prop_assert!(!dbg.is_empty());
    }

    #[test]
    fn symbol_terminal_ne_nonterminal(a in 0u16..1000) {
        let t = Symbol::Terminal(SymbolId(a));
        let n = Symbol::NonTerminal(SymbolId(a));
        prop_assert_ne!(&t, &n);
    }
}

// ---------------------------------------------------------------------------
// 3. PrecedenceKind ordering and properties — 6 tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn precedence_kind_static_clone_eq(v in -500i16..500) {
        let p = PrecedenceKind::Static(v);
        let c = p;
        prop_assert_eq!(p, c);
    }

    #[test]
    fn precedence_kind_dynamic_clone_eq(v in -500i16..500) {
        let p = PrecedenceKind::Dynamic(v);
        let c = p;
        prop_assert_eq!(p, c);
    }

    #[test]
    fn precedence_kind_static_ne_dynamic(v in -500i16..500) {
        let s = PrecedenceKind::Static(v);
        let d = PrecedenceKind::Dynamic(v);
        prop_assert_ne!(s, d);
    }

    #[test]
    fn precedence_kind_static_debug_contains_value(v in -500i16..500) {
        let p = PrecedenceKind::Static(v);
        let dbg = format!("{:?}", p);
        prop_assert!(dbg.contains(&v.to_string()));
    }

    #[test]
    fn precedence_kind_dynamic_debug_contains_value(v in -500i16..500) {
        let p = PrecedenceKind::Dynamic(v);
        let dbg = format!("{:?}", p);
        prop_assert!(dbg.contains(&v.to_string()));
    }

    #[test]
    fn associativity_clone_eq(idx in 0u8..3) {
        let a = match idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let b = a;
        prop_assert_eq!(a, b);
    }
}

// ---------------------------------------------------------------------------
// 4. Token serde roundtrip — 4 tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn token_serde_roundtrip_string(
        name in "[a-zA-Z][a-zA-Z0-9]{0,10}",
        pattern in "[a-zA-Z0-9]{1,20}",
        fragile in proptest::bool::ANY,
    ) {
        let tok = Token {
            name: name.clone(),
            pattern: TokenPattern::String(pattern.clone()),
            fragile,
        };
        let json = serde_json::to_string(&tok).unwrap();
        let back: Token = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&tok, &back);
    }

    #[test]
    fn token_serde_roundtrip_regex(
        name in "[a-zA-Z][a-zA-Z0-9]{0,10}",
        pattern in "[a-zA-Z0-9]{1,20}",
    ) {
        let tok = Token {
            name,
            pattern: TokenPattern::Regex(pattern),
            fragile: false,
        };
        let json = serde_json::to_string(&tok).unwrap();
        let back: Token = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&tok, &back);
    }

    #[test]
    fn token_debug_contains_name(
        name in "[a-zA-Z]{3,8}",
    ) {
        let tok = Token {
            name: name.clone(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        };
        let dbg = format!("{:?}", tok);
        prop_assert!(dbg.contains(&name));
    }

    #[test]
    fn token_clone_preserves_fields(
        name in "[a-zA-Z]{2,6}",
        fragile in proptest::bool::ANY,
    ) {
        let tok = Token {
            name: name.clone(),
            pattern: TokenPattern::String("lit".into()),
            fragile,
        };
        let c = tok.clone();
        prop_assert_eq!(tok.name, c.name);
        prop_assert_eq!(tok.fragile, c.fragile);
        prop_assert_eq!(tok.pattern, c.pattern);
    }
}

// ---------------------------------------------------------------------------
// 5. GrammarBuilder determinism — 8 tests
// ---------------------------------------------------------------------------

fn build_simple_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUM", r"\d+")
        .token("PLUS", "+")
        .rule("expr", vec!["expr", "PLUS", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

proptest! {
    #[test]
    fn builder_deterministic_name(suffix in "[a-z]{1,8}") {
        let name = format!("g_{}", suffix);
        let g1 = build_simple_grammar(&name);
        let g2 = build_simple_grammar(&name);
        prop_assert_eq!(g1.name, g2.name);
    }

    #[test]
    fn builder_deterministic_token_count(suffix in "[a-z]{1,8}") {
        let name = format!("g_{}", suffix);
        let g1 = build_simple_grammar(&name);
        let g2 = build_simple_grammar(&name);
        prop_assert_eq!(g1.tokens.len(), g2.tokens.len());
    }

    #[test]
    fn builder_deterministic_rule_count(suffix in "[a-z]{1,8}") {
        let name = format!("g_{}", suffix);
        let g1 = build_simple_grammar(&name);
        let g2 = build_simple_grammar(&name);
        prop_assert_eq!(g1.rules.len(), g2.rules.len());
    }

    #[test]
    fn builder_deterministic_equality(suffix in "[a-z]{1,8}") {
        let name = format!("g_{}", suffix);
        let g1 = build_simple_grammar(&name);
        let g2 = build_simple_grammar(&name);
        prop_assert_eq!(&g1, &g2);
    }

    #[test]
    fn builder_deterministic_json(suffix in "[a-z]{1,8}") {
        let name = format!("g_{}", suffix);
        let g1 = build_simple_grammar(&name);
        let g2 = build_simple_grammar(&name);
        let j1 = serde_json::to_string(&g1).unwrap();
        let j2 = serde_json::to_string(&g2).unwrap();
        prop_assert_eq!(j1, j2);
    }

    #[test]
    fn builder_deterministic_rule_names(suffix in "[a-z]{1,8}") {
        let name = format!("g_{}", suffix);
        let g1 = build_simple_grammar(&name);
        let g2 = build_simple_grammar(&name);
        prop_assert_eq!(g1.rule_names, g2.rule_names);
    }

    #[test]
    fn builder_deterministic_extras(suffix in "[a-z]{1,8}") {
        let name = format!("g_{}", suffix);
        let g1 = build_simple_grammar(&name);
        let g2 = build_simple_grammar(&name);
        prop_assert_eq!(g1.extras, g2.extras);
    }

    #[test]
    fn builder_deterministic_precedences(suffix in "[a-z]{1,8}") {
        let name = format!("g_{}", suffix);
        let g1 = build_simple_grammar(&name);
        let g2 = build_simple_grammar(&name);
        prop_assert_eq!(g1.precedences, g2.precedences);
    }
}

// ---------------------------------------------------------------------------
// 6. Grammar clone idempotency — 4 tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn grammar_clone_eq(suffix in "[a-z]{1,6}") {
        let g = build_simple_grammar(&format!("clone_{}", suffix));
        let c = g.clone();
        prop_assert_eq!(&g, &c);
    }

    #[test]
    fn grammar_clone_name_preserved(suffix in "[a-z]{1,6}") {
        let name = format!("clone_{}", suffix);
        let g = build_simple_grammar(&name);
        let c = g.clone();
        prop_assert_eq!(&g.name, &c.name);
    }

    #[test]
    fn grammar_double_clone_eq(suffix in "[a-z]{1,6}") {
        let g = build_simple_grammar(&format!("dc_{}", suffix));
        let c1 = g.clone();
        let c2 = c1.clone();
        prop_assert_eq!(&g, &c2);
    }

    #[test]
    fn grammar_clone_tokens_match(suffix in "[a-z]{1,6}") {
        let g = build_simple_grammar(&format!("ct_{}", suffix));
        let c = g.clone();
        prop_assert_eq!(g.tokens.len(), c.tokens.len());
        for (k, v) in &g.tokens {
            prop_assert_eq!(Some(v), c.tokens.get(k));
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Grammar serde_json roundtrip — 8 tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn grammar_serde_roundtrip_name(suffix in "[a-z]{1,6}") {
        let g = build_simple_grammar(&format!("serde_{}", suffix));
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g.name, &back.name);
    }

    #[test]
    fn grammar_serde_roundtrip_rule_count(suffix in "[a-z]{1,6}") {
        let g = build_simple_grammar(&format!("serde_{}", suffix));
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(g.rules.len(), back.rules.len());
    }

    #[test]
    fn grammar_serde_roundtrip_token_count(suffix in "[a-z]{1,6}") {
        let g = build_simple_grammar(&format!("serde_{}", suffix));
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(g.tokens.len(), back.tokens.len());
    }

    #[test]
    fn grammar_serde_roundtrip_full_equality(suffix in "[a-z]{1,6}") {
        let g = build_simple_grammar(&format!("serde_{}", suffix));
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g, &back);
    }

    #[test]
    fn grammar_serde_roundtrip_rule_names(suffix in "[a-z]{1,6}") {
        let g = build_simple_grammar(&format!("serde_{}", suffix));
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g.rule_names, &back.rule_names);
    }

    #[test]
    fn grammar_serde_roundtrip_precedences(suffix in "[a-z]{1,6}") {
        let g = build_simple_grammar(&format!("serde_{}", suffix));
        let json = serde_json::to_string(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g.precedences, &back.precedences);
    }

    #[test]
    fn grammar_serde_roundtrip_pretty(suffix in "[a-z]{1,6}") {
        let g = build_simple_grammar(&format!("serde_{}", suffix));
        let json = serde_json::to_string_pretty(&g).unwrap();
        let back: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&g, &back);
    }

    #[test]
    fn grammar_serde_double_roundtrip(suffix in "[a-z]{1,6}") {
        let g = build_simple_grammar(&format!("serde_{}", suffix));
        let j1 = serde_json::to_string(&g).unwrap();
        let mid: Grammar = serde_json::from_str(&j1).unwrap();
        let j2 = serde_json::to_string(&mid).unwrap();
        prop_assert_eq!(j1, j2);
    }
}

// ---------------------------------------------------------------------------
// 8. SymbolId hashing consistency — 4 tests
// ---------------------------------------------------------------------------

fn hash_of<T: Hash>(val: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}

proptest! {
    #[test]
    fn symbol_id_hash_deterministic(a in 0u16..1000) {
        let id = SymbolId(a);
        prop_assert_eq!(hash_of(&id), hash_of(&id));
    }

    #[test]
    fn symbol_id_equal_implies_same_hash(a in 0u16..1000) {
        let x = SymbolId(a);
        let y = SymbolId(a);
        prop_assert_eq!(x, y);
        prop_assert_eq!(hash_of(&x), hash_of(&y));
    }

    #[test]
    fn rule_id_hash_deterministic(a in 0u16..1000) {
        let id = RuleId(a);
        prop_assert_eq!(hash_of(&id), hash_of(&id));
    }

    #[test]
    fn rule_id_equal_implies_same_hash(a in 0u16..1000) {
        let x = RuleId(a);
        let y = RuleId(a);
        prop_assert_eq!(x, y);
        prop_assert_eq!(hash_of(&x), hash_of(&y));
    }
}

// ---------------------------------------------------------------------------
// 9. Builder rule count matches expectation — 4 tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn builder_single_rule_count(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("one")
            .token("A", "a")
            .rule("start", vec!["A"])
            .start("start")
            .build();
        // One LHS symbol with rules
        prop_assert_eq!(g.rules.len(), 1);
        // That LHS has exactly one production
        let prods: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(prods, 1);
    }

    #[test]
    fn builder_two_alternatives_count(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("two")
            .token("A", "a")
            .token("B", "b")
            .rule("start", vec!["A"])
            .rule("start", vec!["B"])
            .start("start")
            .build();
        prop_assert_eq!(g.rules.len(), 1);
        let prods: usize = g.rules.values().map(|v| v.len()).sum();
        prop_assert_eq!(prods, 2);
    }

    #[test]
    fn builder_multiple_lhs_count(_dummy in 0u8..1) {
        let g = GrammarBuilder::new("multi")
            .token("NUM", r"\d+")
            .token("PLUS", "+")
            .rule("expr", vec!["term", "PLUS", "term"])
            .rule("term", vec!["NUM"])
            .start("expr")
            .build();
        // Two distinct LHS symbols
        prop_assert_eq!(g.rules.len(), 2);
    }

    #[test]
    fn builder_token_count_matches(n in 1u8..6) {
        let mut b = GrammarBuilder::new("toks");
        for i in 0..n {
            let name = format!("T{}", i);
            let pat = format!("t{}", i);
            b = b.token(&name, &pat);
        }
        b = b.rule("start", vec!["T0"]).start("start");
        let g = b.build();
        prop_assert_eq!(g.tokens.len(), n as usize);
    }
}

// ---------------------------------------------------------------------------
// 10. Edge cases — 6 tests
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_id_zero_valid(_dummy in 0u8..1) {
        let id = SymbolId(0);
        let dbg = format!("{:?}", id);
        prop_assert!(dbg.contains("0"));
    }

    #[test]
    fn symbol_id_max_valid(_dummy in 0u8..1) {
        let id = SymbolId(u16::MAX);
        let dbg = format!("{:?}", id);
        prop_assert!(dbg.contains(&u16::MAX.to_string()));
    }

    #[test]
    fn grammar_default_is_empty(_dummy in 0u8..1) {
        let g = Grammar::default();
        prop_assert!(g.rules.is_empty());
        prop_assert!(g.tokens.is_empty());
        prop_assert!(g.name.is_empty());
    }

    #[test]
    fn symbol_sequence_clone_eq(n in 1usize..6) {
        let syms: Vec<Symbol> = (0..n).map(|i| Symbol::Terminal(SymbolId(i as u16))).collect();
        let s = Symbol::Sequence(syms);
        let c = s.clone();
        prop_assert_eq!(&s, &c);
    }

    #[test]
    fn symbol_choice_clone_eq(n in 1usize..6) {
        let syms: Vec<Symbol> = (0..n).map(|i| Symbol::NonTerminal(SymbolId(i as u16))).collect();
        let s = Symbol::Choice(syms);
        let c = s.clone();
        prop_assert_eq!(&s, &c);
    }

    #[test]
    fn symbol_repeat_one_clone_eq(a in 0u16..1000) {
        let inner = Symbol::Terminal(SymbolId(a));
        let s = Symbol::RepeatOne(Box::new(inner));
        let c = s.clone();
        prop_assert_eq!(&s, &c);
    }
}
