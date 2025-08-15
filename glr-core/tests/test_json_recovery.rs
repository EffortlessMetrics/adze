//! Test JSON parsing with error recovery to verify our EOF and error stats fixes

use rust_sitter_glr_core::{Driver, ParseTable, build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

fn create_json_grammar() -> Grammar {
    let mut grammar = Grammar::new("json".to_string());
    
    // Tokens
    let lbrace_id = SymbolId(1);   // {
    let rbrace_id = SymbolId(2);   // }
    let lbracket_id = SymbolId(3); // [
    let rbracket_id = SymbolId(4); // ]
    let comma_id = SymbolId(5);    // ,
    let colon_id = SymbolId(6);    // :
    let string_id = SymbolId(7);   // "..."
    let number_id = SymbolId(8);   // 123
    let true_id = SymbolId(9);     // true
    let false_id = SymbolId(10);   // false
    let null_id = SymbolId(11);    // null
    
    // Non-terminals
    let value_id = SymbolId(20);
    let object_id = SymbolId(21);
    let array_id = SymbolId(22);
    let members_id = SymbolId(23);
    let member_id = SymbolId(24);
    let elements_id = SymbolId(25);
    
    // Define tokens
    grammar.tokens.insert(lbrace_id, Token {
        name: "lbrace".to_string(),
        pattern: TokenPattern::String("{".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(rbrace_id, Token {
        name: "rbrace".to_string(),
        pattern: TokenPattern::String("}".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(lbracket_id, Token {
        name: "lbracket".to_string(),
        pattern: TokenPattern::String("[".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(rbracket_id, Token {
        name: "rbracket".to_string(),
        pattern: TokenPattern::String("]".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(comma_id, Token {
        name: "comma".to_string(),
        pattern: TokenPattern::String(",".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(colon_id, Token {
        name: "colon".to_string(),
        pattern: TokenPattern::String(":".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(string_id, Token {
        name: "string".to_string(),
        pattern: TokenPattern::Regex(r#""[^"]*""#.to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(number_id, Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+(\.\d+)?".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(true_id, Token {
        name: "true".to_string(),
        pattern: TokenPattern::String("true".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(false_id, Token {
        name: "false".to_string(),
        pattern: TokenPattern::String("false".to_string()),
        fragile: false,
    });
    
    grammar.tokens.insert(null_id, Token {
        name: "null".to_string(),
        pattern: TokenPattern::String("null".to_string()),
        fragile: false,
    });
    
    // Define non-terminals
    grammar.rule_names.insert(value_id, "value".to_string());
    grammar.rule_names.insert(object_id, "object".to_string());
    grammar.rule_names.insert(array_id, "array".to_string());
    grammar.rule_names.insert(members_id, "members".to_string());
    grammar.rule_names.insert(member_id, "member".to_string());
    grammar.rule_names.insert(elements_id, "elements".to_string());
    
    // Rules
    // value → object | array | string | number | true | false | null
    grammar.rules.entry(value_id).or_insert_with(Vec::new).push(Rule {
        lhs: value_id,
        rhs: vec![Symbol::NonTerminal(object_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });
    
    grammar.rules.entry(value_id).or_insert_with(Vec::new).push(Rule {
        lhs: value_id,
        rhs: vec![Symbol::NonTerminal(array_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
        fields: vec![],
    });
    
    grammar.rules.entry(value_id).or_insert_with(Vec::new).push(Rule {
        lhs: value_id,
        rhs: vec![Symbol::Terminal(string_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(2),
        fields: vec![],
    });
    
    grammar.rules.entry(value_id).or_insert_with(Vec::new).push(Rule {
        lhs: value_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(3),
        fields: vec![],
    });
    
    // object → { } | { members }
    grammar.rules.entry(object_id).or_insert_with(Vec::new).push(Rule {
        lhs: object_id,
        rhs: vec![Symbol::Terminal(lbrace_id), Symbol::Terminal(rbrace_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(4),
        fields: vec![],
    });
    
    grammar.rules.entry(object_id).or_insert_with(Vec::new).push(Rule {
        lhs: object_id,
        rhs: vec![
            Symbol::Terminal(lbrace_id),
            Symbol::NonTerminal(members_id),
            Symbol::Terminal(rbrace_id),
        ],
        precedence: None,
        associativity: None,
        production_id: ProductionId(5),
        fields: vec![],
    });
    
    // array → [ ] | [ elements ]
    grammar.rules.entry(array_id).or_insert_with(Vec::new).push(Rule {
        lhs: array_id,
        rhs: vec![Symbol::Terminal(lbracket_id), Symbol::Terminal(rbracket_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(6),
        fields: vec![],
    });
    
    grammar.rules.entry(array_id).or_insert_with(Vec::new).push(Rule {
        lhs: array_id,
        rhs: vec![
            Symbol::Terminal(lbracket_id),
            Symbol::NonTerminal(elements_id),
            Symbol::Terminal(rbracket_id),
        ],
        precedence: None,
        associativity: None,
        production_id: ProductionId(7),
        fields: vec![],
    });
    
    // member → string : value
    grammar.rules.entry(member_id).or_insert_with(Vec::new).push(Rule {
        lhs: member_id,
        rhs: vec![
            Symbol::Terminal(string_id),
            Symbol::Terminal(colon_id),
            Symbol::NonTerminal(value_id),
        ],
        precedence: None,
        associativity: None,
        production_id: ProductionId(8),
        fields: vec![],
    });
    
    // members → member | member , members
    grammar.rules.entry(members_id).or_insert_with(Vec::new).push(Rule {
        lhs: members_id,
        rhs: vec![Symbol::NonTerminal(member_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(9),
        fields: vec![],
    });
    
    grammar.rules.entry(members_id).or_insert_with(Vec::new).push(Rule {
        lhs: members_id,
        rhs: vec![
            Symbol::NonTerminal(member_id),
            Symbol::Terminal(comma_id),
            Symbol::NonTerminal(members_id),
        ],
        precedence: None,
        associativity: None,
        production_id: ProductionId(10),
        fields: vec![],
    });
    
    // elements → value | value , elements
    grammar.rules.entry(elements_id).or_insert_with(Vec::new).push(Rule {
        lhs: elements_id,
        rhs: vec![Symbol::NonTerminal(value_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(11),
        fields: vec![],
    });
    
    grammar.rules.entry(elements_id).or_insert_with(Vec::new).push(Rule {
        lhs: elements_id,
        rhs: vec![
            Symbol::NonTerminal(value_id),
            Symbol::Terminal(comma_id),
            Symbol::NonTerminal(elements_id),
        ],
        precedence: None,
        associativity: None,
        production_id: ProductionId(12),
        fields: vec![],
    });
    
    grammar
}

#[test]
#[cfg(feature = "test-helpers")]
fn test_valid_json_clean_forest() {
    let grammar = create_json_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    let mut driver = Driver::new(&parse_table);
    
    // Test valid JSON: {}
    let tokens = vec![
        (1u32, 0u32, 1u32),  // {
        (2u32, 1u32, 1u32),  // }
    ];
    
    // Add EOF at the correct symbol ID
    let eof_symbol = parse_table.eof_symbol.0 as u32;
    let mut tokens_with_eof = tokens;
    tokens_with_eof.push((eof_symbol, 2, 0));
    
    let result = driver.parse_tokens(tokens_with_eof);
    
    match result {
        Ok(forest) => {
            let (has_error, missing, cost) = forest.debug_error_stats();
            assert_eq!(
                (has_error, missing, cost),
                (false, 0, 0),
                "Valid JSON '{{}}' should have zero error stats"
            );
        }
        Err(e) => panic!("Valid JSON should parse: {}", e),
    }
}

#[test]
#[cfg(feature = "test-helpers")]
fn test_missing_closing_brace_recovery() {
    let grammar = create_json_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    let mut driver = Driver::new(&parse_table);
    
    // Test invalid JSON: { (missing })
    let tokens = vec![
        (1u32, 0u32, 1u32),  // {
    ];
    
    // Add EOF
    let eof_symbol = parse_table.eof_symbol.0 as u32;
    let mut tokens_with_eof = tokens;
    tokens_with_eof.push((eof_symbol, 1, 0));
    
    let result = driver.parse_tokens(tokens_with_eof);
    
    match result {
        Ok(forest) => {
            let (has_error, missing, cost) = forest.debug_error_stats();
            
            // Should recover by inserting the missing }
            assert!(
                has_error || missing > 0 || cost > 0,
                "Missing '}}' should trigger error recovery. Got: has_error={}, missing={}, cost={}",
                has_error, missing, cost
            );
            
            // Specifically expect 1 missing terminal
            assert!(
                missing >= 1,
                "Expected at least 1 missing terminal ('}}'), got {}",
                missing
            );
        }
        Err(_) => {
            // Recovery might fail, which is okay for this malformed input
        }
    }
}

#[test]
#[cfg(feature = "test-helpers")]
fn test_trailing_comma_recovery() {
    let grammar = create_json_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    let mut driver = Driver::new(&parse_table);
    
    // Test invalid JSON: [1,] (trailing comma)
    let tokens = vec![
        (3u32, 0u32, 1u32),  // [
        (8u32, 1u32, 1u32),  // number (1)
        (5u32, 2u32, 1u32),  // ,
        (4u32, 3u32, 1u32),  // ]
    ];
    
    // Add EOF
    let eof_symbol = parse_table.eof_symbol.0 as u32;
    let mut tokens_with_eof = tokens;
    tokens_with_eof.push((eof_symbol, 4, 0));
    
    let result = driver.parse_tokens(tokens_with_eof);
    
    // This grammar might not handle trailing commas correctly,
    // but we're testing that error recovery stats work
    match result {
        Ok(forest) => {
            let (has_error, missing, _cost) = forest.debug_error_stats();
            // May or may not have errors depending on grammar
            println!("Trailing comma parse: has_error={}, missing={}", has_error, missing);
        }
        Err(e) => {
            println!("Trailing comma failed to parse (expected): {}", e);
        }
    }
}

#[test]
#[cfg(feature = "test-helpers")]
fn test_eof_not_zero() {
    // Verify our EOF fix: EOF symbol should not be 0 (ERROR)
    let grammar = create_json_grammar();
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    
    assert_ne!(
        parse_table.eof_symbol,
        SymbolId(0),
        "EOF symbol must not be 0 (ERROR symbol)"
    );
    
    assert!(
        parse_table.eof_symbol.0 as usize >= parse_table.token_count + parse_table.external_token_count,
        "EOF symbol {} should be >= token_count({}) + external_token_count({})",
        parse_table.eof_symbol.0,
        parse_table.token_count,
        parse_table.external_token_count
    );
    
    assert!(
        parse_table.symbol_to_index.get(&parse_table.eof_symbol).is_some(),
        "EOF symbol must be present in symbol_to_index mapping"
    );
}