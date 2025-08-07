//! Integration tests that verify ACTUAL subtree reuse is happening
//! Not just correctness, but performance and efficiency

#[cfg(feature = "incremental_glr")]
mod incremental_reuse_tests {
    use rust_sitter::glr_incremental::{
        reset_reuse_counter, get_reuse_count,
        IncrementalGLRParser, GLREdit
    };
    use rust_sitter_glr_core::{GLRParser, build_lr1_automaton, FirstFollowSets};
    use rust_sitter_ir::{Grammar, SymbolId, Symbol, SymbolKind, Rule, Associativity, RuleId};
    use std::collections::HashMap;
    use std::time::Duration;
    
    /// Create a simple arithmetic grammar for testing
    fn create_test_grammar() -> Grammar {
        let mut grammar = Grammar {
            symbols: vec![],
            rules: vec![],
            start_symbol: SymbolId(0),
            precedences: Default::default(),
            fragile_symbols: Default::default(),
            hidden_rules: Default::default(),
            external_scanner: None,
            word_token: None,
            supertype_symbols: Default::default(),
            conflicts: Default::default(),
            inline_symbols: Default::default(),
            field_names: Default::default(),
        };
        
        // Add symbols
        let expr = SymbolId(0);
        grammar.symbols.push(Symbol::NonTerminal {
            name: "expr".to_string(),
            rule_ids: vec![],
        });
        
        let number = SymbolId(1);
        grammar.symbols.push(Symbol::Terminal {
            name: "NUMBER".to_string(),
            is_lexical: true,
        });
        
        let plus = SymbolId(2);
        grammar.symbols.push(Symbol::Terminal {
            name: "PLUS".to_string(),
            is_lexical: true,
        });
        
        // Add rules
        let rule1 = RuleId(0);
        grammar.rules.push(Rule {
            lhs: expr,
            rhs: vec![expr, plus, expr],
            precedence: Some(1),
            associativity: Some(Associativity::Left),
            is_fragile: false,
            alias: None,
            field_map: HashMap::new(),
        });
        
        let rule2 = RuleId(1);
        grammar.rules.push(Rule {
            lhs: expr,
            rhs: vec![number],
            precedence: Some(0),
            associativity: None,
            is_fragile: false,
            alias: None,
            field_map: HashMap::new(),
        });
        
        // Update rule IDs in symbols
        if let Symbol::NonTerminal { rule_ids, .. } = &mut grammar.symbols[0] {
            rule_ids.push(rule1);
            rule_ids.push(rule2);
        }
        
        grammar.start_symbol = expr;
        
        grammar
    }
    
    #[test]
    fn test_simple_edit_reuses_subtrees() {
        let grammar = create_test_grammar();
        
        // Reset the global reuse counter
        reset_reuse_counter();
        
        // Create incremental parser
        let mut parser = IncrementalGLRParser::new(grammar);
        
        // Parse initial text: "1 + 2 + 3"
        let initial_text = "1 + 2 + 3";
        let initial_tree = parser.parse(initial_text).unwrap();
        
        // Make a small edit: change "2" to "5"
        // This should reuse the "1 +" prefix and " + 3" suffix
        let edited_text = "1 + 5 + 3";
        let edit = GLREdit {
            start_byte: 4,
            old_end_byte: 5,
            new_end_byte: 5,
            start_point: (0, 4),
            old_end_point: (0, 5),
            new_end_point: (0, 5),
        };
        
        // Perform incremental parse
        let reparsed_tree = parser.reparse_with_edits(
            initial_text.as_bytes(),
            edited_text.as_bytes(),
            &initial_tree,
            vec![edit],
        ).unwrap();
        
        // Check that subtrees were reused
        let reuse_count = get_reuse_count();
        assert!(
            reuse_count > 0,
            "Expected subtree reuse but got 0 reuses! The incremental parser is not reusing any subtrees."
        );
    }
    
}