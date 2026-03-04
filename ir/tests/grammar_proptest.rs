//! Property-based tests for IR grammar operations.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Symbol, SymbolId, TokenPattern};
use proptest::prelude::*;

proptest! {
    #[test]
    fn grammar_name_preserved(name in "[a-zA-Z][a-zA-Z0-9_]{0,20}") {
        let builder = GrammarBuilder::new(&name);
        let grammar = builder.build();
        prop_assert_eq!(&grammar.name, &name);
    }

    #[test]
    fn grammar_roundtrip_json(name in "[a-zA-Z][a-zA-Z0-9_]{0,10}") {
        let grammar = GrammarBuilder::new(&name)
            .token("A", "a")
            .build();
        let json = serde_json::to_string(&grammar).unwrap();
        let deserialized: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&grammar.name, &deserialized.name);
    }

    #[test]
    fn symbol_id_roundtrip(id in 0u16..10000) {
        let sid = SymbolId(id);
        let display = format!("{sid:?}");
        prop_assert!(display.contains(&id.to_string()));
    }

    #[test]
    fn symbol_terminal_preserves_id(id in 1u16..100) {
        let sym = Symbol::Terminal(SymbolId(id));
        match sym {
            Symbol::Terminal(got) => prop_assert_eq!(got, SymbolId(id)),
            _ => prop_assert!(false, "Expected Terminal"),
        }
    }

    #[test]
    fn symbol_nonterminal_preserves_id(id in 1u16..100) {
        let sym = Symbol::NonTerminal(SymbolId(id));
        match sym {
            Symbol::NonTerminal(got) => prop_assert_eq!(got, SymbolId(id)),
            _ => prop_assert!(false, "Expected NonTerminal"),
        }
    }

    #[test]
    fn token_pattern_clone_eq(s in "[a-z]{1,10}") {
        let pat = TokenPattern::String(s.clone());
        let cloned = pat.clone();
        prop_assert_eq!(&pat, &cloned);
    }

    #[test]
    fn builder_token_count(n in 1usize..10) {
        let mut builder = GrammarBuilder::new("test");
        for i in 0..n {
            builder = builder.token(&format!("T{i}"), &format!("{i}"));
        }
        let grammar = builder.build();
        prop_assert!(grammar.tokens.len() >= n);
    }

    #[test]
    fn builder_rule_count(n in 1usize..5) {
        let mut builder = GrammarBuilder::new("test")
            .token("A", "a");
        for i in 0..n {
            builder = builder.rule(&format!("rule_{i}"), vec!["A"]);
        }
        let grammar = builder.build();
        prop_assert!(grammar.rules.len() >= n);
    }

    #[test]
    fn symbol_epsilon_is_distinct(_dummy in 0..1i32) {
        let eps = Symbol::Epsilon;
        let term = Symbol::Terminal(SymbolId(1));
        let nonterm = Symbol::NonTerminal(SymbolId(1));
        prop_assert_ne!(eps.clone(), term);
        prop_assert_ne!(eps, nonterm);
    }
}
