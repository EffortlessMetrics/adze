// Tests with real-world-like grammars to validate the pure-Rust implementation

use rust_sitter_glr_core::FirstFollowSets;
use rust_sitter_ir::{
    FieldId, Grammar, ProductionId, Rule, Symbol,
    SymbolId, Token, TokenPattern,
};
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
            pattern: TokenPattern::Regex(
                r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?".to_string(),
            ),
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

    grammar.tokens.insert(
        whitespace,
        Token {
            name: "whitespace".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: false,
        },
    );

    // Define fields
    let key_field = FieldId(0);
    let value_field = FieldId(1);

    grammar.fields.insert(key_field, "key".to_string());
    grammar.fields.insert(value_field, "value".to_string());

    // Non-terminal names
    grammar.rule_names.insert(value, "value".to_string());
    grammar.rule_names.insert(object, "object".to_string());
    grammar.rule_names.insert(pair, "pair".to_string());
    grammar.rule_names.insert(array, "array".to_string());

    // Grammar rules
    let mut prod_id = 0;

    // All rules for 'value' symbol
    grammar.rules.insert(
        value,
        vec![
            // value -> object
            Rule {
                lhs: value,
                rhs: vec![Symbol::NonTerminal(object)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod_id),
            },
            // value -> array
            Rule {
                lhs: value,
                rhs: vec![Symbol::NonTerminal(array)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod_id + 1),
            },
            // value -> string
            Rule {
                lhs: value,
                rhs: vec![Symbol::Terminal(string)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod_id + 2),
            },
            // value -> number
            Rule {
                lhs: value,
                rhs: vec![Symbol::Terminal(number)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod_id + 3),
            },
            // value -> true
            Rule {
                lhs: value,
                rhs: vec![Symbol::Terminal(true_lit)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod_id + 4),
            },
            // value -> false
            Rule {
                lhs: value,
                rhs: vec![Symbol::Terminal(false_lit)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod_id + 5),
            },
            // value -> null
            Rule {
                lhs: value,
                rhs: vec![Symbol::Terminal(null_lit)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod_id + 6),
            },
        ],
    );
    prod_id += 7;

    // All rules for 'object' symbol
    grammar.rules.insert(
        object,
        vec![
            // object -> { }
            Rule {
                lhs: object,
                rhs: vec![Symbol::Terminal(lbrace), Symbol::Terminal(rbrace)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod_id),
            },
            // object -> { pair (, pair)* }
            Rule {
                lhs: object,
                rhs: vec![
                    Symbol::Terminal(lbrace),
                    Symbol::NonTerminal(pair),
                    Symbol::Repeat(Box::new(Symbol::Sequence(vec![
                        Symbol::Terminal(comma),
                        Symbol::NonTerminal(pair),
                    ]))),
                    Symbol::Terminal(rbrace),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod_id + 1),
            },
        ],
    );
    prod_id += 2;

    // Rule for 'pair' symbol
    grammar.rules.insert(
        pair,
        vec![
            // pair -> string : value
            Rule {
                lhs: pair,
                rhs: vec![
                    Symbol::Terminal(string),
                    Symbol::Terminal(colon),
                    Symbol::NonTerminal(value),
                ],
                precedence: None,
                associativity: None,
                fields: vec![
                    (key_field, 0),   // string is the key
                    (value_field, 2), // value is the value
                ],
                production_id: ProductionId(prod_id),
            },
        ],
    );
    prod_id += 1;

    // All rules for 'array' symbol
    grammar.rules.insert(
        array,
        vec![
            // array -> [ ]
            Rule {
                lhs: array,
                rhs: vec![Symbol::Terminal(lbracket), Symbol::Terminal(rbracket)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod_id),
            },
            // array -> [ value (, value)* ]
            Rule {
                lhs: array,
                rhs: vec![
                    Symbol::Terminal(lbracket),
                    Symbol::NonTerminal(value),
                    Symbol::Repeat(Box::new(Symbol::Sequence(vec![
                        Symbol::Terminal(comma),
                        Symbol::NonTerminal(value),
                    ]))),
                    Symbol::Terminal(rbracket),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(prod_id + 1),
            },
        ],
    );

    // Set whitespace as extra
    grammar.extras = vec![whitespace];

    // Set start symbol
    // Note: In the current implementation, start symbol is determined by convention

    grammar
}

#[test]
fn test_json_node_types_generation() {
    let grammar = create_json_grammar();
    let generator = NodeTypesGenerator::new(&grammar);
    let node_types_result = generator.generate();

    // Basic validation
    let node_types = node_types_result.expect("Should generate node types");
    assert!(!node_types.is_empty());

    // Parse the JSON to verify structure
    let parsed: serde_json::Value = serde_json::from_str(&node_types).expect("Invalid JSON");
    let types = parsed.as_array().expect("Expected array");

    // Should have types for value, object, pair, array
    assert!(types.len() >= 4);

    // Find the pair type which should have fields
    let pair_type = types
        .iter()
        .find(|t| t["type"] == "pair")
        .expect("Should have pair type");

    let fields = pair_type["fields"]
        .as_object()
        .expect("pair should have fields");
    assert!(fields.contains_key("key"));
    assert!(fields.contains_key("value"));
}

#[test]
fn test_json_language_generation() {
    let grammar = create_json_grammar();

    // Create a minimal parse table for testing
    let first_follow = FirstFollowSets::compute(&grammar);
    let parse_table = rust_sitter_glr_core::build_lr1_automaton(&grammar, &first_follow)
        .expect("Should build parse table");

    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let _output = generator.generate_language_code();

    // If we get here without panicking, the generation succeeded
}
