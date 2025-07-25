// Example demonstrating incremental parsing with the pure-Rust parser
use rust_sitter::pure_parser::{Parser, TSLanguage, TSParseAction};
use rust_sitter::pure_incremental::{Tree, Edit, Point};

// Create a simple arithmetic language for demonstration
fn create_demo_language() -> &'static TSLanguage {
    // Symbol IDs
    const EOF: u16 = 0;
    const NUMBER: u16 = 1;
    const PLUS: u16 = 2;
    const STAR: u16 = 3;
    const LPAREN: u16 = 4;
    const RPAREN: u16 = 5;
    const EXPR: u16 = 6;
    const SUM: u16 = 7;
    const PRODUCT: u16 = 8;
    
    // Parse table (simplified)
    static PARSE_TABLE: &[u16] = &[
        // State 0: initial state
        0, 0, 0, 0, 0, 0,  // default action
        1, 0, 0, 0, 0, 0,  // NUMBER -> shift to state 1
        // ... more states
    ];
    
    // Small parse table
    static SMALL_PARSE_TABLE: &[u16] = &[0; 6];
    
    // Language definition
    static LANGUAGE: TSLanguage = TSLanguage {
        version: 14,
        symbol_count: 9,
        alias_count: 0,
        token_count: 6,
        external_token_count: 0,
        state_count: 12,
        large_state_count: 0,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
        parse_table: PARSE_TABLE.as_ptr(),
        small_parse_table: SMALL_PARSE_TABLE.as_ptr(),
        small_parse_table_map: std::ptr::null(),
        parse_actions: std::ptr::null(),
        symbol_names: std::ptr::null(),
        field_names: std::ptr::null(),
        field_map_slices: std::ptr::null(),
        field_map_entries: std::ptr::null(),
        symbol_metadata: std::ptr::null(),
        public_symbol_map: std::ptr::null(),
        alias_map: std::ptr::null(),
        alias_sequences: std::ptr::null(),
        lex_modes: std::ptr::null(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner: std::ptr::null(),
        primary_state_ids: std::ptr::null(),
    };
    
    &LANGUAGE
}

fn main() {
    println!("=== Pure-Rust Incremental Parsing Demo ===\n");
    
    // Create parser and set language
    let mut parser = Parser::new();
    let language = create_demo_language();
    parser.set_language(language).expect("Failed to set language");
    
    // Initial parse
    let source1 = "1 + 2 + 3";
    println!("Initial source: {}", source1);
    let result1 = parser.parse_string(source1);
    
    if let Some(root) = &result1.root {
        println!("Parsed successfully!");
        println!("Root node: {:?}", root);
        
        // Create a tree for incremental parsing
        let tree1 = Tree::new(root.clone(), language, source1.as_bytes());
        
        // Edit the source: change "2" to "42"
        let source2 = "1 + 42 + 3";
        println!("\nEdited source: {}", source2);
        
        // Create edit
        let edit = Edit {
            start_byte: 4,
            old_end_byte: 5,
            new_end_byte: 6,
            start_point: Point { row: 0, column: 4 },
            old_end_point: Point { row: 0, column: 5 },
            new_end_point: Point { row: 0, column: 6 },
        };
        
        // Apply edit to tree
        let mut tree2 = tree1.clone();
        tree2.edit(&edit);
        
        // Parse incrementally
        println!("Parsing incrementally...");
        let result2 = parser.parse_string_with_tree(source2, Some(&tree2));
        
        if let Some(root2) = &result2.root {
            println!("Incremental parse successful!");
            println!("New root node: {:?}", root2);
            
            // Demonstrate another edit: insert " * 4" at the end
            let source3 = "1 + 42 + 3 * 4";
            println!("\nFinal source: {}", source3);
            
            let tree2_final = Tree::new(root2.clone(), language, source2.as_bytes());
            let edit2 = Edit {
                start_byte: 10,
                old_end_byte: 10,
                new_end_byte: 14,
                start_point: Point { row: 0, column: 10 },
                old_end_point: Point { row: 0, column: 10 },
                new_end_point: Point { row: 0, column: 14 },
            };
            
            let mut tree3 = tree2_final.clone();
            tree3.edit(&edit2);
            
            let result3 = parser.parse_string_with_tree(source3, Some(&tree3));
            
            if let Some(root3) = &result3.root {
                println!("Final incremental parse successful!");
                println!("Final root node: {:?}", root3);
            }
        }
    }
    
    println!("\n=== Demo Complete ===");
}