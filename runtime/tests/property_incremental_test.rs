#[cfg(test)]
mod incremental_properties {
    use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets, ParseTable};
    use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId};

    #[test]
    fn basic_grammar_construction() {
        let mut grammar = Grammar::new("test".to_string());

        // Add root non-terminal that accepts a sequence
        let number_id = SymbolId(1);
        let root_id = SymbolId(10);
        let rule = Rule {
            lhs: root_id,
            rhs: vec![Symbol::Terminal(number_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        };
        grammar.add_rule(rule);

        let first_follow = FirstFollowSets::compute(&grammar);
        let _table =
            build_lr1_automaton(&grammar, &first_follow).expect("Should build parse table");

        assert!(true, "Grammar and table construction should succeed");
    }
}
