use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    AliasSequence, Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId,
    Grammar, Precedence, PrecedenceKind, ProductionId, Rule, RuleId, StateId, Symbol, SymbolId,
    SymbolMetadata, Token, TokenPattern,
};
use proptest::prelude::*;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0u16..=u16::MAX).prop_map(SymbolId)
}

fn _arb_rule_id() -> impl Strategy<Value = RuleId> {
    (0u16..=u16::MAX).prop_map(RuleId)
}

fn _arb_state_id() -> impl Strategy<Value = StateId> {
    (0u16..=u16::MAX).prop_map(StateId)
}

fn arb_field_id() -> impl Strategy<Value = FieldId> {
    (0u16..=u16::MAX).prop_map(FieldId)
}

fn arb_production_id() -> impl Strategy<Value = ProductionId> {
    (0u16..=u16::MAX).prop_map(ProductionId)
}

fn arb_precedence_kind() -> impl Strategy<Value = PrecedenceKind> {
    prop_oneof![
        any::<i16>().prop_map(PrecedenceKind::Static),
        any::<i16>().prop_map(PrecedenceKind::Dynamic),
    ]
}

fn arb_associativity() -> impl Strategy<Value = Associativity> {
    prop_oneof![
        Just(Associativity::Left),
        Just(Associativity::Right),
        Just(Associativity::None),
    ]
}

/// Leaf-level symbol (no recursion).
fn arb_leaf_symbol() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        arb_symbol_id().prop_map(Symbol::Terminal),
        arb_symbol_id().prop_map(Symbol::NonTerminal),
        arb_symbol_id().prop_map(Symbol::External),
        Just(Symbol::Epsilon),
    ]
}

/// Symbol tree with bounded depth.
fn arb_symbol() -> impl Strategy<Value = Symbol> {
    arb_leaf_symbol().prop_recursive(3, 16, 4, |inner| {
        prop_oneof![
            inner.clone().prop_map(|s| Symbol::Optional(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::Repeat(Box::new(s))),
            inner.clone().prop_map(|s| Symbol::RepeatOne(Box::new(s))),
            prop::collection::vec(inner.clone(), 1..=4).prop_map(Symbol::Choice),
            prop::collection::vec(inner, 1..=4).prop_map(Symbol::Sequence),
        ]
    })
}

fn arb_token_pattern() -> impl Strategy<Value = TokenPattern> {
    prop_oneof![
        "[a-zA-Z][a-zA-Z0-9]{0,8}".prop_map(TokenPattern::String),
        "[a-zA-Z][a-zA-Z0-9]{0,8}".prop_map(TokenPattern::Regex),
    ]
}

fn arb_symbol_metadata() -> impl Strategy<Value = SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(visible, named, hidden, terminal)| SymbolMetadata {
            visible,
            named,
            hidden,
            terminal,
        },
    )
}

/// Safe identifier: starts with a lowercase letter, followed by lowercase/digits.
fn arb_ident() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{1,7}"
}

/// Safe token pattern (only alphanumeric, no regex special chars that could panic).
fn arb_safe_pattern() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9]{0,6}"
}

// ---------------------------------------------------------------------------
// 1–5: SymbolId serde roundtrip & basic properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // 1
    #[test]
    fn symbol_id_serde_roundtrip(id in 0u16..=u16::MAX) {
        let s = SymbolId(id);
        let json = serde_json::to_string(&s).unwrap();
        let s2: SymbolId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, s2);
    }

    // 2
    #[test]
    fn rule_id_serde_roundtrip(id in 0u16..=u16::MAX) {
        let r = RuleId(id);
        let json = serde_json::to_string(&r).unwrap();
        let r2: RuleId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(r, r2);
    }

    // 3
    #[test]
    fn state_id_serde_roundtrip(id in 0u16..=u16::MAX) {
        let s = StateId(id);
        let json = serde_json::to_string(&s).unwrap();
        let s2: StateId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(s, s2);
    }

    // 4
    #[test]
    fn field_id_serde_roundtrip(id in 0u16..=u16::MAX) {
        let f = FieldId(id);
        let json = serde_json::to_string(&f).unwrap();
        let f2: FieldId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(f, f2);
    }

    // 5
    #[test]
    fn production_id_serde_roundtrip(id in 0u16..=u16::MAX) {
        let p = ProductionId(id);
        let json = serde_json::to_string(&p).unwrap();
        let p2: ProductionId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(p, p2);
    }

    // ---------------------------------------------------------------------------
    // 6–10: Symbol enum serde roundtrip
    // ---------------------------------------------------------------------------

    // 6
    #[test]
    fn terminal_serde_roundtrip(id in arb_symbol_id()) {
        let sym = Symbol::Terminal(id);
        let json = serde_json::to_string(&sym).unwrap();
        let sym2: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, sym2);
    }

    // 7
    #[test]
    fn nonterminal_serde_roundtrip(id in arb_symbol_id()) {
        let sym = Symbol::NonTerminal(id);
        let json = serde_json::to_string(&sym).unwrap();
        let sym2: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, sym2);
    }

    // 8
    #[test]
    fn external_serde_roundtrip(id in arb_symbol_id()) {
        let sym = Symbol::External(id);
        let json = serde_json::to_string(&sym).unwrap();
        let sym2: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, sym2);
    }

    // 9
    #[test]
    fn optional_serde_roundtrip(inner in arb_leaf_symbol()) {
        let sym = Symbol::Optional(Box::new(inner));
        let json = serde_json::to_string(&sym).unwrap();
        let sym2: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, sym2);
    }

    // 10
    #[test]
    fn repeat_serde_roundtrip(inner in arb_leaf_symbol()) {
        let sym = Symbol::Repeat(Box::new(inner));
        let json = serde_json::to_string(&sym).unwrap();
        let sym2: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, sym2);
    }

    // ---------------------------------------------------------------------------
    // 11–15: More symbol variants & deep trees
    // ---------------------------------------------------------------------------

    // 11
    #[test]
    fn repeat_one_serde_roundtrip(inner in arb_leaf_symbol()) {
        let sym = Symbol::RepeatOne(Box::new(inner));
        let json = serde_json::to_string(&sym).unwrap();
        let sym2: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, sym2);
    }

    // 12
    #[test]
    fn choice_serde_roundtrip(choices in prop::collection::vec(arb_leaf_symbol(), 1..=5)) {
        let sym = Symbol::Choice(choices);
        let json = serde_json::to_string(&sym).unwrap();
        let sym2: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, sym2);
    }

    // 13
    #[test]
    fn sequence_serde_roundtrip(seq in prop::collection::vec(arb_leaf_symbol(), 1..=5)) {
        let sym = Symbol::Sequence(seq);
        let json = serde_json::to_string(&sym).unwrap();
        let sym2: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, sym2);
    }

    // 14
    #[test]
    fn deep_symbol_serde_roundtrip(sym in arb_symbol()) {
        let json = serde_json::to_string(&sym).unwrap();
        let sym2: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, sym2);
    }

    // 15
    #[test]
    fn epsilon_serde_always_same(_seed in 0u32..1000) {
        let sym = Symbol::Epsilon;
        let json = serde_json::to_string(&sym).unwrap();
        let sym2: Symbol = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(sym, sym2);
    }

    // ---------------------------------------------------------------------------
    // 16–20: Clone & equality
    // ---------------------------------------------------------------------------

    // 16
    #[test]
    fn symbol_clone_eq(sym in arb_symbol()) {
        let cloned = sym.clone();
        prop_assert_eq!(&sym, &cloned);
    }

    // 17
    #[test]
    fn symbol_id_ord_consistent(a in 0u16..=u16::MAX, b in 0u16..=u16::MAX) {
        let sa = SymbolId(a);
        let sb = SymbolId(b);
        prop_assert_eq!(sa.cmp(&sb), a.cmp(&b));
    }

    // 18
    #[test]
    fn symbol_hash_consistent(sym in arb_leaf_symbol()) {
        let mut set = HashSet::new();
        set.insert(sym.clone());
        prop_assert!(set.contains(&sym));
    }

    // 19
    #[test]
    fn symbol_ne_terminal_vs_nonterminal(id in arb_symbol_id()) {
        let t = Symbol::Terminal(id);
        let nt = Symbol::NonTerminal(id);
        prop_assert_ne!(t, nt);
    }

    // 20
    #[test]
    fn rule_clone_eq(
        lhs in arb_symbol_id(),
        rhs in prop::collection::vec(arb_leaf_symbol(), 0..=4),
        prod in arb_production_id(),
    ) {
        let rule = Rule {
            lhs,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: prod,
        };
        let cloned = rule.clone();
        prop_assert_eq!(rule, cloned);
    }

    // ---------------------------------------------------------------------------
    // 21–25: Token & TokenPattern
    // ---------------------------------------------------------------------------

    // 21
    #[test]
    fn token_pattern_string_serde(s in arb_safe_pattern()) {
        let pat = TokenPattern::String(s);
        let json = serde_json::to_string(&pat).unwrap();
        let pat2: TokenPattern = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(pat, pat2);
    }

    // 22
    #[test]
    fn token_pattern_regex_serde(s in arb_safe_pattern()) {
        let pat = TokenPattern::Regex(s);
        let json = serde_json::to_string(&pat).unwrap();
        let pat2: TokenPattern = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(pat, pat2);
    }

    // 23
    #[test]
    fn token_serde_roundtrip(name in arb_ident(), pat in arb_token_pattern(), fragile in any::<bool>()) {
        let token = Token { name, pattern: pat, fragile };
        let json = serde_json::to_string(&token).unwrap();
        let token2: Token = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(token, token2);
    }

    // 24
    #[test]
    fn token_clone_eq(name in arb_ident(), pat in arb_token_pattern(), fragile in any::<bool>()) {
        let token = Token { name, pattern: pat, fragile };
        prop_assert_eq!(&token, &token.clone());
    }

    // 25
    #[test]
    fn token_pattern_ne_string_vs_regex(s in arb_safe_pattern()) {
        let a = TokenPattern::String(s.clone());
        let b = TokenPattern::Regex(s);
        prop_assert_ne!(a, b);
    }

    // ---------------------------------------------------------------------------
    // 26–30: Precedence, Associativity, ConflictDeclaration
    // ---------------------------------------------------------------------------

    // 26
    #[test]
    fn precedence_kind_serde(pk in arb_precedence_kind()) {
        let json = serde_json::to_string(&pk).unwrap();
        let pk2: PrecedenceKind = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(pk, pk2);
    }

    // 27
    #[test]
    fn associativity_serde(a in arb_associativity()) {
        let json = serde_json::to_string(&a).unwrap();
        let a2: Associativity = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(a, a2);
    }

    // 28
    #[test]
    fn precedence_serde_roundtrip(
        level in any::<i16>(),
        assoc in arb_associativity(),
        syms in prop::collection::vec(arb_symbol_id(), 0..=4),
    ) {
        let p = Precedence { level, associativity: assoc, symbols: syms };
        let json = serde_json::to_string(&p).unwrap();
        let p2: Precedence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(p, p2);
    }

    // 29
    #[test]
    fn conflict_declaration_serde(
        syms in prop::collection::vec(arb_symbol_id(), 1..=4),
    ) {
        let cd = ConflictDeclaration {
            symbols: syms,
            resolution: ConflictResolution::GLR,
        };
        let json = serde_json::to_string(&cd).unwrap();
        let cd2: ConflictDeclaration = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(cd, cd2);
    }

    // 30
    #[test]
    fn conflict_resolution_precedence_serde(pk in arb_precedence_kind()) {
        let cr = ConflictResolution::Precedence(pk);
        let json = serde_json::to_string(&cr).unwrap();
        let cr2: ConflictResolution = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(cr, cr2);
    }

    // ---------------------------------------------------------------------------
    // 31–35: Rule serde & properties
    // ---------------------------------------------------------------------------

    // 31
    #[test]
    fn rule_serde_roundtrip(
        lhs in arb_symbol_id(),
        rhs in prop::collection::vec(arb_leaf_symbol(), 0..=4),
        prec in proptest::option::of(arb_precedence_kind()),
        assoc in proptest::option::of(arb_associativity()),
        prod in arb_production_id(),
    ) {
        let rule = Rule { lhs, rhs, precedence: prec, associativity: assoc, fields: vec![], production_id: prod };
        let json = serde_json::to_string(&rule).unwrap();
        let rule2: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rule, rule2);
    }

    // 32
    #[test]
    fn rule_with_fields_serde(
        lhs in arb_symbol_id(),
        field_id in arb_field_id(),
        pos in 0u16..10,
        prod in arb_production_id(),
    ) {
        let rule = Rule {
            lhs,
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![(field_id, pos as usize)],
            production_id: prod,
        };
        let json = serde_json::to_string(&rule).unwrap();
        let rule2: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rule, rule2);
    }

    // 33
    #[test]
    fn rule_rhs_length_preserved(
        len in 0usize..=8,
        lhs in arb_symbol_id(),
        prod in arb_production_id(),
    ) {
        let rhs: Vec<Symbol> = (0..len).map(|i| Symbol::Terminal(SymbolId(i as u16))).collect();
        let rule = Rule { lhs, rhs, precedence: None, associativity: None, fields: vec![], production_id: prod };
        let json = serde_json::to_string(&rule).unwrap();
        let rule2: Rule = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(rule2.rhs.len(), len);
    }

    // 34
    #[test]
    fn external_token_serde(name in arb_ident(), id in arb_symbol_id()) {
        let et = ExternalToken { name, symbol_id: id };
        let json = serde_json::to_string(&et).unwrap();
        let et2: ExternalToken = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(et, et2);
    }

    // 35
    #[test]
    fn alias_sequence_serde(
        aliases in prop::collection::vec(proptest::option::of(arb_ident()), 0..=5),
    ) {
        let seq = AliasSequence { aliases };
        let json = serde_json::to_string(&seq).unwrap();
        let seq2: AliasSequence = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(seq, seq2);
    }

    // ---------------------------------------------------------------------------
    // 36–40: Grammar construction & serde
    // ---------------------------------------------------------------------------

    // 36
    #[test]
    fn grammar_new_name_preserved(name in arb_ident()) {
        let grammar = Grammar::new(name.clone());
        prop_assert_eq!(grammar.name, name);
        prop_assert!(grammar.rules.is_empty());
    }

    // 37
    #[test]
    fn grammar_serde_empty_roundtrip(name in arb_ident()) {
        let grammar = Grammar::new(name);
        let json = serde_json::to_string(&grammar).unwrap();
        let grammar2: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(grammar, grammar2);
    }

    // 38
    #[test]
    fn grammar_add_rule_count(
        count in 1usize..=5,
    ) {
        let mut grammar = Grammar::new("test".to_string());
        let lhs = SymbolId(1);
        grammar.tokens.insert(SymbolId(10), Token {
            name: "tok".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        });
        for i in 0..count {
            grammar.add_rule(Rule {
                lhs,
                rhs: vec![Symbol::Terminal(SymbolId(10))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            });
        }
        let rules = grammar.get_rules_for_symbol(lhs).unwrap();
        prop_assert_eq!(rules.len(), count);
    }

    // 39
    #[test]
    fn grammar_all_rules_iterator(
        n_symbols in 1usize..=4,
        rules_per in 1usize..=3,
    ) {
        let mut grammar = Grammar::new("iter_test".to_string());
        let mut prod = 0u16;
        for s in 0..n_symbols {
            let lhs = SymbolId(s as u16 + 1);
            for _ in 0..rules_per {
                grammar.add_rule(Rule {
                    lhs,
                    rhs: vec![Symbol::Epsilon],
                    precedence: None,
                    associativity: None,
                    fields: vec![],
                    production_id: ProductionId(prod),
                });
                prod += 1;
            }
        }
        prop_assert_eq!(grammar.all_rules().count(), n_symbols * rules_per);
    }

    // 40
    #[test]
    fn grammar_clone_eq(name in arb_ident()) {
        let mut grammar = Grammar::new(name);
        grammar.tokens.insert(SymbolId(1), Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        });
        grammar.add_rule(Rule {
            lhs: SymbolId(2),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        let cloned = grammar.clone();
        prop_assert_eq!(grammar, cloned);
    }

    // ---------------------------------------------------------------------------
    // 41–45: Builder API
    // ---------------------------------------------------------------------------

    // 41
    #[test]
    fn builder_single_token(
        gname in arb_ident(),
        tname in arb_ident(),
        tpat in arb_safe_pattern(),
    ) {
        let grammar = GrammarBuilder::new(&gname)
            .token(&tname, &tpat)
            .build();
        prop_assert_eq!(grammar.name, gname);
        prop_assert_eq!(grammar.tokens.len(), 1);
    }

    // 42
    #[test]
    fn builder_multiple_tokens(
        count in 1usize..=6,
    ) {
        let names: Vec<String> = (0..count).map(|i| format!("tok{}", i)).collect();
        let mut b = GrammarBuilder::new("multi");
        for n in &names {
            b = b.token(n, n);
        }
        let grammar = b.build();
        prop_assert_eq!(grammar.tokens.len(), count);
    }

    // 43
    #[test]
    fn builder_rule_creates_production(
        tok_name in arb_ident(),
        tok_pat in arb_safe_pattern(),
        rule_name in arb_ident(),
    ) {
        let grammar = GrammarBuilder::new("g")
            .token(&tok_name, &tok_pat)
            .rule(&rule_name, vec![&tok_name])
            .start(&rule_name)
            .build();
        prop_assert!(!grammar.rules.is_empty());
    }

    // 44
    #[test]
    fn builder_start_symbol_first(
        _seed in 0u32..100,
    ) {
        // With two rules the start should appear first in the map.
        let grammar = GrammarBuilder::new("g")
            .token("A", "a")
            .token("B", "b")
            .rule("beta", vec!["B"])
            .rule("alpha", vec!["A"])
            .start("alpha")
            .build();
        let first_lhs = *grammar.rules.keys().next().unwrap();
        let alpha_id = grammar.find_symbol_by_name("alpha").unwrap();
        prop_assert_eq!(first_lhs, alpha_id);
    }

    // 45
    #[test]
    fn builder_with_precedence(
        prec in -100i16..=100,
        assoc in arb_associativity(),
    ) {
        let grammar = GrammarBuilder::new("prec")
            .token("NUM", "123")
            .token("OP", "op")
            .rule_with_precedence("expr", vec!["expr", "OP", "expr"], prec, assoc)
            .rule("expr", vec!["NUM"])
            .start("expr")
            .build();
        let expr_id = grammar.find_symbol_by_name("expr").unwrap();
        let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
        let has_prec = rules.iter().any(|r| r.precedence.is_some());
        prop_assert!(has_prec);
    }

    // ---------------------------------------------------------------------------
    // 46–50: Normalization
    // ---------------------------------------------------------------------------

    // 46
    #[test]
    fn normalize_optional_creates_epsilon(
        id_val in 1u16..=500,
    ) {
        let mut grammar = Grammar::new("opt".to_string());
        let lhs = SymbolId(1);
        let inner_id = SymbolId(id_val);
        grammar.tokens.insert(inner_id, Token {
            name: "t".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        });
        grammar.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(inner_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        grammar.normalize();
        // After normalization no Optional should remain in any rhs
        for rule in grammar.all_rules() {
            for sym in &rule.rhs {
                prop_assert!(!matches!(sym, Symbol::Optional(_)));
            }
        }
    }

    // 47
    #[test]
    fn normalize_repeat_creates_epsilon(
        id_val in 1u16..=500,
    ) {
        let mut grammar = Grammar::new("rep".to_string());
        let lhs = SymbolId(1);
        let inner_id = SymbolId(id_val);
        grammar.tokens.insert(inner_id, Token {
            name: "t".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        });
        grammar.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(inner_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        grammar.normalize();
        // Epsilon should appear somewhere (repeat includes ε production)
        let has_eps = grammar.all_rules().any(|r| r.rhs.contains(&Symbol::Epsilon));
        prop_assert!(has_eps);
    }

    // 48
    #[test]
    fn normalize_repeat_one_no_epsilon(
        id_val in 1u16..=500,
    ) {
        let mut grammar = Grammar::new("rep1".to_string());
        let lhs = SymbolId(1);
        let inner_id = SymbolId(id_val);
        grammar.tokens.insert(inner_id, Token {
            name: "t".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        });
        grammar.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(inner_id)))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        grammar.normalize();
        // RepeatOne should NOT produce an ε rule among the *aux* rules it creates.
        // The aux rules are: aux -> aux inner | inner (no epsilon).
        // We check that the aux symbol's rules don't include epsilon.
        // The original lhs now points to the aux nonterminal.
        let lhs_rules = grammar.get_rules_for_symbol(lhs).unwrap();
        // The lhs rhs should be a single NonTerminal (aux)
        for r in lhs_rules {
            if let [Symbol::NonTerminal(aux_id)] = r.rhs.as_slice() {
                let aux_rules = grammar.get_rules_for_symbol(*aux_id).unwrap();
                for ar in aux_rules {
                    prop_assert!(!ar.rhs.iter().any(|s| matches!(s, Symbol::Epsilon)));
                }
            }
        }
    }

    // 49
    #[test]
    fn normalize_choice_expands(
        n_choices in 2usize..=4,
    ) {
        let mut grammar = Grammar::new("choice".to_string());
        let lhs = SymbolId(1);
        let choices: Vec<Symbol> = (0..n_choices)
            .map(|i| {
                let tid = SymbolId(100 + i as u16);
                grammar.tokens.insert(tid, Token {
                    name: format!("t{}", i),
                    pattern: TokenPattern::String(format!("t{}", i)),
                    fragile: false,
                });
                Symbol::Terminal(tid)
            })
            .collect();
        grammar.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::Choice(choices)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        grammar.normalize();
        // No Choice should remain after normalization
        for rule in grammar.all_rules() {
            for sym in &rule.rhs {
                prop_assert!(!matches!(sym, Symbol::Choice(_)));
            }
        }
    }

    // 50
    #[test]
    fn normalize_sequence_flattened(
        n_seq in 2usize..=4,
    ) {
        let mut grammar = Grammar::new("seq".to_string());
        let lhs = SymbolId(1);
        let seq: Vec<Symbol> = (0..n_seq)
            .map(|i| {
                let tid = SymbolId(100 + i as u16);
                grammar.tokens.insert(tid, Token {
                    name: format!("s{}", i),
                    pattern: TokenPattern::String(format!("s{}", i)),
                    fragile: false,
                });
                Symbol::Terminal(tid)
            })
            .collect();
        grammar.add_rule(Rule {
            lhs,
            rhs: vec![Symbol::Sequence(seq)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        grammar.normalize();
        // After normalization no Sequence should remain
        for rule in grammar.all_rules() {
            for sym in &rule.rhs {
                prop_assert!(!matches!(sym, Symbol::Sequence(_)));
            }
        }
        // The lhs rule should have n_seq terminals in its rhs
        let lhs_rules = grammar.get_rules_for_symbol(lhs).unwrap();
        prop_assert!(lhs_rules.iter().any(|r| r.rhs.len() == n_seq));
    }

    // ---------------------------------------------------------------------------
    // 51–55: SymbolMetadata, Display, misc serde
    // ---------------------------------------------------------------------------

    // 51
    #[test]
    fn symbol_metadata_serde_roundtrip(meta in arb_symbol_metadata()) {
        let json = serde_json::to_string(&meta).unwrap();
        let meta2: SymbolMetadata = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(meta, meta2);
    }

    // 52
    #[test]
    fn symbol_id_display_format(id in 0u16..=u16::MAX) {
        let s = SymbolId(id);
        let display = format!("{}", s);
        prop_assert_eq!(display, format!("Symbol({})", id));
    }

    // 53
    #[test]
    fn rule_id_display_format(id in 0u16..=u16::MAX) {
        let r = RuleId(id);
        prop_assert_eq!(format!("{}", r), format!("Rule({})", id));
    }

    // 54
    #[test]
    fn state_id_display_format(id in 0u16..=u16::MAX) {
        let s = StateId(id);
        prop_assert_eq!(format!("{}", s), format!("State({})", id));
    }

    // 55
    #[test]
    fn production_id_display_format(id in 0u16..=u16::MAX) {
        let p = ProductionId(id);
        prop_assert_eq!(format!("{}", p), format!("Production({})", id));
    }

    // ---------------------------------------------------------------------------
    // 56–60: Builder with extras, externals, fragile tokens, empty rule
    // ---------------------------------------------------------------------------

    // 56
    #[test]
    fn builder_extra_token(
        extra_name in arb_ident(),
        extra_pat in arb_safe_pattern(),
    ) {
        let grammar = GrammarBuilder::new("g")
            .token(&extra_name, &extra_pat)
            .extra(&extra_name)
            .build();
        prop_assert_eq!(grammar.extras.len(), 1);
    }

    // 57
    #[test]
    fn builder_external_token(ext_name in arb_ident()) {
        let grammar = GrammarBuilder::new("g")
            .external(&ext_name)
            .build();
        prop_assert_eq!(grammar.externals.len(), 1);
        prop_assert_eq!(grammar.externals[0].name.clone(), ext_name);
    }

    // 58
    #[test]
    fn builder_fragile_token(name in arb_ident(), pat in arb_safe_pattern()) {
        let grammar = GrammarBuilder::new("g")
            .fragile_token(&name, &pat)
            .build();
        let tok = grammar.tokens.values().next().unwrap();
        prop_assert!(tok.fragile);
    }

    // 59
    #[test]
    fn builder_empty_rule_is_epsilon(rule_name in arb_ident()) {
        let grammar = GrammarBuilder::new("g")
            .rule(&rule_name, vec![])
            .start(&rule_name)
            .build();
        let sid = grammar.find_symbol_by_name(&rule_name).unwrap();
        let rules = grammar.get_rules_for_symbol(sid).unwrap();
        prop_assert!(rules.iter().any(|r| r.rhs == vec![Symbol::Epsilon]));
    }

    // 60
    #[test]
    fn builder_grammar_serde_roundtrip(
        gname in arb_ident(),
        tname in arb_ident(),
        tpat in arb_safe_pattern(),
        rname in arb_ident(),
    ) {
        let grammar = GrammarBuilder::new(&gname)
            .token(&tname, &tpat)
            .rule(&rname, vec![&tname])
            .start(&rname)
            .build();
        let json = serde_json::to_string(&grammar).unwrap();
        let grammar2: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(grammar, grammar2);
    }

    // ---------------------------------------------------------------------------
    // 61–65: Grammar validation & find_symbol_by_name
    // ---------------------------------------------------------------------------

    // 61
    #[test]
    fn grammar_find_symbol_by_name(name in arb_ident()) {
        let grammar = GrammarBuilder::new("g")
            .token("T", "t")
            .rule(&name, vec!["T"])
            .start(&name)
            .build();
        let found = grammar.find_symbol_by_name(&name);
        prop_assert!(found.is_some());
    }

    // 62
    #[test]
    fn grammar_find_missing_symbol(name in arb_ident()) {
        let grammar = Grammar::new("empty".to_string());
        let found = grammar.find_symbol_by_name(&name);
        prop_assert!(found.is_none());
    }

    // 63
    #[test]
    fn grammar_validate_empty_passes(_seed in 0u32..100) {
        let grammar = Grammar::new("empty".to_string());
        prop_assert!(grammar.validate().is_ok());
    }

    // 64
    #[test]
    fn grammar_check_empty_terminals_ok(name in arb_ident(), pat in arb_safe_pattern()) {
        let mut grammar = Grammar::new("g".to_string());
        grammar.tokens.insert(SymbolId(1), Token {
            name,
            pattern: TokenPattern::String(pat),
            fragile: false,
        });
        prop_assert!(grammar.check_empty_terminals().is_ok());
    }

    // 65
    #[test]
    fn grammar_check_empty_terminals_fails(name in arb_ident()) {
        let mut grammar = Grammar::new("g".to_string());
        grammar.tokens.insert(SymbolId(1), Token {
            name,
            pattern: TokenPattern::String(String::new()),
            fragile: false,
        });
        prop_assert!(grammar.check_empty_terminals().is_err());
    }
}
