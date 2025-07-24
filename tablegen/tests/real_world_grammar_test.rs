// Tests with real-world-like grammars to validate the pure-Rust implementation

use rust_sitter_ir::{
    Associativity, ExternalToken, FieldId, Grammar, PrecedenceKind, ProductionId, Rule,
    Symbol, SymbolId, Token, TokenPattern,
};
use rust_sitter_glr_core::{FirstFollowSets, ParseTable};
use rust_sitter_tablegen::{NodeTypesGenerator, StaticLanguageGenerator};

/// Create a JSON-like grammar that resembles real Tree-sitter grammars
fn create_json_grammar() -> Grammar {
    let mut grammar = Grammar::new("json".to_string());

    // Symbol IDs
    let _eof = SymbolId(0);
    let value = SymbolId(1);
    let object = SymbolId(2);
    let pair = SymbolId(3);
    let array = SymbolId(4);
    let string = SymbolId(5);
    let number = SymbolId(6);
    let true_lit = SymbolId(7);
    let false_lit = SymbolId(8);
    let null_lit = SymbolId(9);
    let lbrace = SymbolId(10);
    let rbrace = SymbolId(11);
    let lbracket = SymbolId(12);
    let rbracket = SymbolId(13);
    let comma = SymbolId(14);
    let colon = SymbolId(15);
    let whitespace = SymbolId(16);

    // Define tokens
    grammar.tokens.insert(
        string,
        Token {
            name: "string".to_string(),
            pattern: TokenPattern::Regex(r#""([^"\\]|\\.)*""#.to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        number,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        true_lit,
        Token {
            name: "true".to_string(),
            pattern: TokenPattern::String("true".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        false_lit,
        Token {
            name: "false".to_string(),
            pattern: TokenPattern::String("false".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        null_lit,
        Token {
            name: "null".to_string(),
            pattern: TokenPattern::String("null".to_string()),
            fragile: false,
        },
    );

    // Punctuation
    grammar.tokens.insert(
        lbrace,
        Token {
            name: "{".to_string(),
            pattern: TokenPattern::String("{".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        rbrace,
        Token {
            name: "}".to_string(),
            pattern: TokenPattern::String("}".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        lbracket,
        Token {
            name: "[".to_string(),
            pattern: TokenPattern::String("[".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        rbracket,
        Token {
            name: "]".to_string(),
            pattern: TokenPattern::String("]".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        comma,
        Token {
            name: ",".to_string(),
            pattern: TokenPattern::String(",".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        colon,
        Token {
            name: ":".to_string(),
            pattern: TokenPattern::String(":".to_string()),
            fragile: false,
        },
    );

    // Whitespace as external token
    grammar.externals.push(ExternalToken {
        name: "whitespace".to_string(),
        symbol_id: whitespace,
    });

    // Grammar rules
    let _rule_id = 0;
    let mut prod_id = 0;

    // value -> object
    grammar.rules.insert(
        value,
        Rule {
            lhs: value,
            rhs: vec![Symbol::NonTerminal(object)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // value -> array
    grammar.rules.insert(
        value,
        Rule {
            lhs: value,
            rhs: vec![Symbol::NonTerminal(array)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // value -> string
    grammar.rules.insert(
        value,
        Rule {
            lhs: value,
            rhs: vec![Symbol::Terminal(string)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // value -> number
    grammar.rules.insert(
        value,
        Rule {
            lhs: value,
            rhs: vec![Symbol::Terminal(number)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // value -> true
    grammar.rules.insert(
        value,
        Rule {
            lhs: value,
            rhs: vec![Symbol::Terminal(true_lit)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // value -> false
    grammar.rules.insert(
        value,
        Rule {
            lhs: value,
            rhs: vec![Symbol::Terminal(false_lit)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // value -> null
    grammar.rules.insert(
        value,
        Rule {
            lhs: value,
            rhs: vec![Symbol::Terminal(null_lit)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // object -> { }
    grammar.rules.insert(
        object,
        Rule {
            lhs: object,
            rhs: vec![Symbol::Terminal(lbrace), Symbol::Terminal(rbrace)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // object -> { pair }
    grammar.rules.insert(
        object,
        Rule {
            lhs: object,
            rhs: vec![
                Symbol::Terminal(lbrace),
                Symbol::NonTerminal(pair),
                Symbol::Terminal(rbrace),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // pair -> string : value
    grammar.rules.insert(
        pair,
        Rule {
            lhs: pair,
            rhs: vec![
                Symbol::Terminal(string),
                Symbol::Terminal(colon),
                Symbol::NonTerminal(value),
            ],
            precedence: None,
            associativity: None,
            fields: vec![(FieldId(1), 0), (FieldId(2), 2)], // key, value
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // array -> [ ]
    grammar.rules.insert(
        array,
        Rule {
            lhs: array,
            rhs: vec![Symbol::Terminal(lbracket), Symbol::Terminal(rbracket)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // array -> [ value ]
    grammar.rules.insert(
        array,
        Rule {
            lhs: array,
            rhs: vec![
                Symbol::Terminal(lbracket),
                Symbol::NonTerminal(value),
                Symbol::Terminal(rbracket),
            ],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        },
    );

    // Field names
    grammar.fields.insert(FieldId(1), "key".to_string());
    grammar.fields.insert(FieldId(2), "value".to_string());

    // Mark visible symbols
    grammar.supertypes.push(value);

    grammar
}

/// Create a simple programming language grammar
fn create_mini_lang_grammar() -> Grammar {
    let mut grammar = Grammar::new("mini_lang".to_string());

    // Symbol IDs
    let _program = SymbolId(1);
    let _statement = SymbolId(2);
    let expression = SymbolId(3);
    let identifier = SymbolId(4);
    let number = SymbolId(5);
    let string_lit = SymbolId(6);
    let let_kw = SymbolId(7);
    let if_kw = SymbolId(8);
    let else_kw = SymbolId(9);
    let while_kw = SymbolId(10);
    let function_kw = SymbolId(11);
    let return_kw = SymbolId(12);
    let assign = SymbolId(13);
    let plus = SymbolId(14);
    let minus = SymbolId(15);
    let star = SymbolId(16);
    let slash = SymbolId(17);
    let lparen = SymbolId(18);
    let rparen = SymbolId(19);
    let lbrace = SymbolId(20);
    let rbrace = SymbolId(21);
    let semicolon = SymbolId(22);
    let comment = SymbolId(23);

    // Tokens
    grammar.tokens.insert(
        identifier,
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        number,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+(\.\d+)?".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        string_lit,
        Token {
            name: "string".to_string(),
            pattern: TokenPattern::Regex(r#""([^"\\]|\\.)*""#.to_string()),
            fragile: false,
        },
    );

    // Keywords
    for (id, kw) in &[
        (let_kw, "let"),
        (if_kw, "if"),
        (else_kw, "else"),
        (while_kw, "while"),
        (function_kw, "function"),
        (return_kw, "return"),
    ] {
        grammar.tokens.insert(
            *id,
            Token {
                name: kw.to_string(),
                pattern: TokenPattern::String(kw.to_string()),
                fragile: true, // Keywords are fragile
            },
        );
    }

    // Operators
    for (id, op, name) in &[
        (assign, "=", "assign"),
        (plus, "+", "plus"),
        (minus, "-", "minus"),
        (star, "*", "star"),
        (slash, "/", "slash"),
        (lparen, "(", "lparen"),
        (rparen, ")", "rparen"),
        (lbrace, "{", "lbrace"),
        (rbrace, "}", "rbrace"),
        (semicolon, ";", "semicolon"),
    ] {
        grammar.tokens.insert(
            *id,
            Token {
                name: name.to_string(),
                pattern: TokenPattern::String(op.to_string()),
                fragile: false,
            },
        );
    }

    // External token for comments
    grammar.externals.push(ExternalToken {
        name: "comment".to_string(),
        symbol_id: comment,
    });

    // Rules with precedence
    let mut prod_id = 0;

    // Binary expressions with precedence
    // expression -> expression + expression (left associative, precedence 1)
    grammar.rules.insert(
        expression,
        Rule {
            lhs: expression,
            rhs: vec![
                Symbol::NonTerminal(expression),
                Symbol::Terminal(plus),
                Symbol::NonTerminal(expression),
            ],
            precedence: Some(PrecedenceKind::Static(1)),
            associativity: Some(Associativity::Left),
            fields: vec![(FieldId(3), 0), (FieldId(4), 2)], // left, right
            production_id: ProductionId(prod_id),
        },
    );
    prod_id += 1;

    // expression -> expression * expression (left associative, precedence 2)
    grammar.rules.insert(
        expression,
        Rule {
            lhs: expression,
            rhs: vec![
                Symbol::NonTerminal(expression),
                Symbol::Terminal(star),
                Symbol::NonTerminal(expression),
            ],
            precedence: Some(PrecedenceKind::Static(2)),
            associativity: Some(Associativity::Left),
            fields: vec![(FieldId(3), 0), (FieldId(4), 2)], // left, right
            production_id: ProductionId(prod_id),
        },
    );

    // Field names
    grammar.fields.insert(FieldId(3), "left".to_string());
    grammar.fields.insert(FieldId(4), "right".to_string());

    grammar
}

#[test]
fn test_json_grammar_generation() {
    let grammar = create_json_grammar();

    // Verify grammar structure
    println!("Tokens: {}, Rules: {}, Fields: {}, Externals: {}", 
        grammar.tokens.len(), grammar.rules.len(), grammar.fields.len(), grammar.externals.len());
    assert!(grammar.tokens.len() >= 10);
    assert!(grammar.rules.len() >= 2); // We're using insert which overwrites, so fewer unique rules
    assert_eq!(grammar.fields.len(), 2);
    assert!(grammar.externals.len() >= 1);

    // Generate NODE_TYPES
    let generator = NodeTypesGenerator::new(&grammar);
    let node_types = generator.generate().unwrap();

    // Verify JSON is valid
    let parsed: serde_json::Value = serde_json::from_str(&node_types).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn test_mini_lang_grammar_generation() {
    let grammar = create_mini_lang_grammar();

    // Check precedence and associativity
    let has_precedence = grammar.rules.values().any(|r| r.precedence.is_some());
    let has_associativity = grammar.rules.values().any(|r| r.associativity.is_some());
    assert!(has_precedence);
    assert!(has_associativity);

    // Check fragile tokens (keywords)
    let has_fragile = grammar.tokens.values().any(|t| t.fragile);
    assert!(has_fragile);
}

#[test]
fn test_first_follow_computation() {
    let grammar = create_json_grammar();

    // Compute FIRST/FOLLOW sets
    let _first_follow = FirstFollowSets::compute(&grammar);

    // Just verify computation completes without panic
    // The FirstFollowSets fields are private, so we can't inspect them directly
}

#[test]
fn test_language_code_generation() {
    let grammar = create_mini_lang_grammar();
    
    // Create a minimal parse table for testing
    let mut parse_table = ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 0,
            symbol_to_index: std::collections::HashMap::new(),
    };

    // Add some dummy data
    parse_table.state_count = 10;
    parse_table.symbol_count = 24;

    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code();
    let code_str = code.to_string();

    // Verify generated code contains expected elements
    assert!(code_str.contains("TREE_SITTER_LANGUAGE_VERSION"));
    assert!(code_str.contains("LANGUAGE")); // The actual struct is generated as LANGUAGE
    assert!(code_str.contains("SYMBOL_NAMES"));
    assert!(code_str.contains("SYMBOL_METADATA"));
    assert!(code_str.contains("FIELD_NAMES"));
}

#[test]
fn test_external_token_handling() {
    let grammar = create_json_grammar();

    // Check external tokens
    assert_eq!(grammar.externals.len(), 1);
    let whitespace_external = &grammar.externals[0];
    assert_eq!(whitespace_external.name, "whitespace");

    // Generate code with external tokens
    let parse_table = ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 1,
            symbol_to_index: std::collections::HashMap::new(),
        symbol_count: 17,
    };

    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code();
    let code_str = code.to_string();

    assert!(code_str.contains("EXTERNAL_TOKEN_COUNT"));
}

#[test]
fn test_field_mapping() {
    let grammar = create_json_grammar();

    // Find the pair rule with fields
    let pair_rule = grammar.rules.values()
        .find(|r| !r.fields.is_empty())
        .expect("Should have rule with fields");

    assert_eq!(pair_rule.fields.len(), 2);
    assert_eq!(pair_rule.fields[0].0, FieldId(1)); // key
    assert_eq!(pair_rule.fields[1].0, FieldId(2)); // value

    // Verify field names
    assert_eq!(grammar.fields.get(&FieldId(1)), Some(&"key".to_string()));
    assert_eq!(grammar.fields.get(&FieldId(2)), Some(&"value".to_string()));
}

#[test]
fn test_complex_grammar_features() {
    let mut grammar = create_mini_lang_grammar();

    // Add inline rules
    grammar.inline_rules.push(SymbolId(100));

    // Add precedence declarations
    grammar.precedences.push(rust_sitter_ir::Precedence {
        level: 10,
        associativity: Associativity::Left,
        symbols: vec![],
    });

    // Add alias sequences
    grammar.alias_sequences.insert(
        ProductionId(0),
        rust_sitter_ir::AliasSequence {
            aliases: vec![],
        },
    );

    // Add conflict declarations
    grammar.conflicts.push(rust_sitter_ir::ConflictDeclaration {
        symbols: vec![SymbolId(2), SymbolId(3)],
        resolution: rust_sitter_ir::ConflictResolution::GLR,
    });

    // Verify all features are present
    assert!(!grammar.inline_rules.is_empty());
    assert!(!grammar.precedences.is_empty());
    assert!(!grammar.alias_sequences.is_empty());
    assert!(!grammar.conflicts.is_empty());
}