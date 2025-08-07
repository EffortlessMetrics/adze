//! Integration tests that verify ACTUAL subtree reuse is happening
//! Not just correctness, but performance and efficiency

#[cfg(feature = "incremental_glr")]
mod incremental_reuse_tests {
    use rust_sitter::glr_incremental::{
        reset_reuse_counter, get_reuse_count,
        IncrementalGLRParser, GLREdit, GLRToken
    };
    use rust_sitter::parser_v4::{Parser, Tree};
    use rust_sitter::pure_incremental::Edit;
    use rust_sitter_glr_core::ParseTable;
    use rust_sitter_ir::{Grammar, SymbolId};
    use std::sync::Arc;
    
    /// Create a simple arithmetic grammar for testing
    fn create_test_grammar() -> (Grammar, ParseTable) {
        // This would be replaced with the actual grammar creation
        // For now, using a placeholder
        use rust_sitter_ir::*;
        
        let mut grammar = Grammar::new();
        
        // Add symbols
        let expr = grammar.add_symbol(Symbol {
            name: "expr".to_string(),
            kind: SymbolKind::NonTerminal,
            rule_ids: vec![],
        });
        
        let number = grammar.add_symbol(Symbol {
            name: "number".to_string(),
            kind: SymbolKind::Terminal,
            rule_ids: vec![],
        });
        
        let plus = grammar.add_symbol(Symbol {
            name: "+".to_string(),
            kind: SymbolKind::Terminal,
            rule_ids: vec![],
        });
        
        // Add rules
        grammar.add_rule(Rule {
            lhs: expr,
            rhs: vec![expr, plus, expr],
            precedence: 0,
            associativity: None,
            is_fragile: false,
            alias: None,
            field_map: Default::default(),
        });
        
        grammar.add_rule(Rule {
            lhs: expr,
            rhs: vec![number],
            precedence: 0,
            associativity: None,
            is_fragile: false,
            alias: None,
            field_map: Default::default(),
        });
        
        grammar.start_symbol = expr;
        
        // Generate parse table
        let table = rust_sitter_glr_core::generate_parse_table(&grammar).unwrap();
        
        (grammar, table)
    }
    
    #[test]
    fn test_simple_edit_reuses_subtrees() {
        let (grammar, table) = create_test_grammar();
        
        // Reset the global reuse counter
        reset_reuse_counter();
        
        // Parse initial text: "1 + 2 + 3"
        let initial_text = b"1 + 2 + 3";
        let mut parser = Parser::new();
        parser.set_language(rust_sitter::Language::arithmetic()).unwrap();
        
        let initial_tree = parser.parse(initial_text, None).unwrap();
        
        // Make a small edit: change "2" to "5"
        // This should reuse the "1 +" prefix and " + 3" suffix
        let edited_text = b"1 + 5 + 3";
        let edit = Edit {
            start_byte: 4,
            old_end_byte: 5,
            new_end_byte: 5,
        };
        
        // Perform incremental parse
        let reparsed_tree = parser.parse_with_incremental(
            edited_text,
            Some(&initial_tree),
            &[edit],
        ).unwrap();
        
        // Check that subtrees were reused
        let reuse_count = get_reuse_count();
        assert!(
            reuse_count > 0,
            "Expected subtree reuse but got 0 reuses! The incremental parser is not reusing any subtrees."
        );
        
        // Verify the parse is still correct
        assert_eq!(reparsed_tree.root_node().kind(), "expr");
        assert_eq!(reparsed_tree.root_node().child_count(), 3);
    }
    
    #[test]
    fn test_multiple_edits_track_reuse() {
        let (grammar, table) = create_test_grammar();
        
        // Start with a longer expression
        let initial_text = b"1 + 2 + 3 + 4 + 5";
        let mut parser = Parser::new();
        parser.set_language(rust_sitter::Language::arithmetic()).unwrap();
        
        let initial_tree = parser.parse(initial_text, None).unwrap();
        
        // Edit 1: Change first number
        reset_reuse_counter();
        let edit1_text = b"9 + 2 + 3 + 4 + 5";
        let edit1 = Edit {
            start_byte: 0,
            old_end_byte: 1,
            new_end_byte: 1,
        };
        
        let tree1 = parser.parse_with_incremental(
            edit1_text,
            Some(&initial_tree),
            &[edit1],
        ).unwrap();
        
        let reuse1 = get_reuse_count();
        assert!(reuse1 > 0, "Should reuse suffix after first edit");
        
        // Edit 2: Change last number
        reset_reuse_counter();
        let edit2_text = b"9 + 2 + 3 + 4 + 8";
        let edit2 = Edit {
            start_byte: 16,
            old_end_byte: 17,
            new_end_byte: 17,
        };
        
        let tree2 = parser.parse_with_incremental(
            edit2_text,
            Some(&tree1),
            &[edit2],
        ).unwrap();
        
        let reuse2 = get_reuse_count();
        assert!(reuse2 > 0, "Should reuse prefix before last edit");
        
        // Edit 3: Change middle
        reset_reuse_counter();
        let edit3_text = b"9 + 2 + 7 + 4 + 8";
        let edit3 = Edit {
            start_byte: 8,
            old_end_byte: 9,
            new_end_byte: 9,
        };
        
        let tree3 = parser.parse_with_incremental(
            edit3_text,
            Some(&tree2),
            &[edit3],
        ).unwrap();
        
        let reuse3 = get_reuse_count();
        assert!(reuse3 > 0, "Should reuse both prefix and suffix for middle edit");
    }
    
    #[test]
    fn test_large_file_edit_efficiency() {
        let (grammar, table) = create_test_grammar();
        
        // Create a large expression
        let mut large_expr = String::new();
        for i in 0..100 {
            if i > 0 {
                large_expr.push_str(" + ");
            }
            large_expr.push_str(&i.to_string());
        }
        
        let initial_text = large_expr.as_bytes();
        let mut parser = Parser::new();
        parser.set_language(rust_sitter::Language::arithmetic()).unwrap();
        
        let initial_tree = parser.parse(initial_text, None).unwrap();
        
        // Make a tiny edit in the middle
        reset_reuse_counter();
        let mut edited = large_expr.clone();
        let middle = edited.len() / 2;
        edited.replace_range(middle..middle+1, "9");
        
        let edit = Edit {
            start_byte: middle,
            old_end_byte: middle + 1,
            new_end_byte: middle + 1,
        };
        
        let reparsed_tree = parser.parse_with_incremental(
            edited.as_bytes(),
            Some(&initial_tree),
            &[edit],
        ).unwrap();
        
        let reuse_count = get_reuse_count();
        
        // For a large file with a small edit, we should reuse MANY subtrees
        assert!(
            reuse_count >= 10,
            "For a 100-number expression with 1 edit, expected at least 10 subtree reuses, got {}",
            reuse_count
        );
        
        println!("Large file edit reused {} subtrees", reuse_count);
    }
    
    #[test]
    fn test_no_reuse_for_complete_replacement() {
        let (grammar, table) = create_test_grammar();
        
        let initial_text = b"1 + 2 + 3";
        let mut parser = Parser::new();
        parser.set_language(rust_sitter::Language::arithmetic()).unwrap();
        
        let initial_tree = parser.parse(initial_text, None).unwrap();
        
        // Replace the entire content
        reset_reuse_counter();
        let new_text = b"4 * 5 * 6";
        let edit = Edit {
            start_byte: 0,
            old_end_byte: initial_text.len(),
            new_end_byte: new_text.len(),
        };
        
        let reparsed_tree = parser.parse_with_incremental(
            new_text,
            Some(&initial_tree),
            &[edit],
        ).unwrap();
        
        let reuse_count = get_reuse_count();
        
        // When the entire file is replaced, no subtrees can be reused
        assert_eq!(
            reuse_count, 0,
            "Complete replacement should not reuse any subtrees"
        );
    }
    
    #[test]
    fn test_append_reuses_prefix() {
        let (grammar, table) = create_test_grammar();
        
        let initial_text = b"1 + 2";
        let mut parser = Parser::new();
        parser.set_language(rust_sitter::Language::arithmetic()).unwrap();
        
        let initial_tree = parser.parse(initial_text, None).unwrap();
        
        // Append to the end
        reset_reuse_counter();
        let appended_text = b"1 + 2 + 3";
        let edit = Edit {
            start_byte: initial_text.len(),
            old_end_byte: initial_text.len(),
            new_end_byte: appended_text.len(),
        };
        
        let reparsed_tree = parser.parse_with_incremental(
            appended_text,
            Some(&initial_tree),
            &[edit],
        ).unwrap();
        
        let reuse_count = get_reuse_count();
        
        assert!(
            reuse_count > 0,
            "Appending should reuse the prefix subtrees"
        );
    }
    
    #[test]
    fn test_deletion_reuses_surrounding() {
        let (grammar, table) = create_test_grammar();
        
        let initial_text = b"1 + 2 + 3 + 4";
        let mut parser = Parser::new();
        parser.set_language(rust_sitter::Language::arithmetic()).unwrap();
        
        let initial_tree = parser.parse(initial_text, None).unwrap();
        
        // Delete the middle part " + 3"
        reset_reuse_counter();
        let deleted_text = b"1 + 2 + 4";
        let edit = Edit {
            start_byte: 5,  // After "1 + 2"
            old_end_byte: 9,  // Before " + 4"
            new_end_byte: 5,  // Nothing added
        };
        
        let reparsed_tree = parser.parse_with_incremental(
            deleted_text,
            Some(&initial_tree),
            &[edit],
        ).unwrap();
        
        let reuse_count = get_reuse_count();
        
        assert!(
            reuse_count > 0,
            "Deletion should reuse surrounding subtrees"
        );
    }
}

/// Performance comparison tests
#[cfg(all(test, feature = "incremental_glr"))]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    #[ignore] // Run with --ignored to see performance comparison
    fn compare_full_vs_incremental_performance() {
        let (grammar, table) = create_test_grammar();
        
        // Create a very large expression
        let mut large_expr = String::new();
        for i in 0..1000 {
            if i > 0 {
                large_expr.push_str(" + ");
            }
            large_expr.push_str(&i.to_string());
        }
        
        let initial_text = large_expr.as_bytes();
        let mut parser = Parser::new();
        parser.set_language(rust_sitter::Language::arithmetic()).unwrap();
        
        // Initial parse
        let initial_tree = parser.parse(initial_text, None).unwrap();
        
        // Make 100 small edits and measure time
        let mut edited = large_expr.clone();
        let mut total_full_parse_time = Duration::ZERO;
        let mut total_incremental_time = Duration::ZERO;
        
        for i in 0..100 {
            // Change a random number
            let pos = (i * 10) % edited.len();
            edited.replace_range(pos..pos+1, "9");
            
            let edit = Edit {
                start_byte: pos,
                old_end_byte: pos + 1,
                new_end_byte: pos + 1,
            };
            
            // Time full reparse
            let start = Instant::now();
            let _ = parser.parse(edited.as_bytes(), None).unwrap();
            total_full_parse_time += start.elapsed();
            
            // Time incremental parse
            let start = Instant::now();
            let _ = parser.parse_with_incremental(
                edited.as_bytes(),
                Some(&initial_tree),
                &[edit],
            ).unwrap();
            total_incremental_time += start.elapsed();
        }
        
        println!("Performance comparison for 100 edits on 1000-number expression:");
        println!("  Full reparse:     {:?}", total_full_parse_time);
        println!("  Incremental:      {:?}", total_incremental_time);
        println!("  Speedup:          {:.2}x", 
            total_full_parse_time.as_secs_f64() / total_incremental_time.as_secs_f64());
        
        // Incremental should be significantly faster
        assert!(
            total_incremental_time < total_full_parse_time,
            "Incremental parsing should be faster than full reparse"
        );
    }
}