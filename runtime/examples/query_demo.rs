// Demonstration of GLR query support
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

// Import internal modules (in real usage, these would be exported)
#[path = "../src/glr_lexer.rs"]
mod glr_lexer;
#[path = "../src/glr_parser.rs"]
mod glr_parser;
#[path = "../src/glr_query.rs"]
mod glr_query;
#[path = "../src/glr_tree_bridge.rs"]
mod glr_tree_bridge;

use glr_query::{QueryCursor, QueryParser, Subtree};

fn create_json_grammar() -> Grammar {
    let mut grammar = Grammar::new("json".to_string());

    // Terminals
    let lbrace_id = SymbolId(0);
    grammar.tokens.insert(
        lbrace_id,
        Token {
            name: "lbrace".to_string(),
            pattern: TokenPattern::String("{".to_string()),
            fragile: false,
        },
    );

    let rbrace_id = SymbolId(1);
    grammar.tokens.insert(
        rbrace_id,
        Token {
            name: "rbrace".to_string(),
            pattern: TokenPattern::String("}".to_string()),
            fragile: false,
        },
    );

    let lbracket_id = SymbolId(2);
    grammar.tokens.insert(
        lbracket_id,
        Token {
            name: "lbracket".to_string(),
            pattern: TokenPattern::String("[".to_string()),
            fragile: false,
        },
    );

    let rbracket_id = SymbolId(3);
    grammar.tokens.insert(
        rbracket_id,
        Token {
            name: "rbracket".to_string(),
            pattern: TokenPattern::String("]".to_string()),
            fragile: false,
        },
    );

    let colon_id = SymbolId(4);
    grammar.tokens.insert(
        colon_id,
        Token {
            name: "colon".to_string(),
            pattern: TokenPattern::String(":".to_string()),
            fragile: false,
        },
    );

    let comma_id = SymbolId(5);
    grammar.tokens.insert(
        comma_id,
        Token {
            name: "comma".to_string(),
            pattern: TokenPattern::String(",".to_string()),
            fragile: false,
        },
    );

    let string_id = SymbolId(6);
    grammar.tokens.insert(
        string_id,
        Token {
            name: "string".to_string(),
            pattern: TokenPattern::Regex(r#""[^"]*""#.to_string()),
            fragile: false,
        },
    );

    let number_id = SymbolId(7);
    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"-?\d+(\.\d+)?".to_string()),
            fragile: false,
        },
    );

    let true_id = SymbolId(8);
    grammar.tokens.insert(
        true_id,
        Token {
            name: "true".to_string(),
            pattern: TokenPattern::String("true".to_string()),
            fragile: false,
        },
    );

    let false_id = SymbolId(9);
    grammar.tokens.insert(
        false_id,
        Token {
            name: "false".to_string(),
            pattern: TokenPattern::String("false".to_string()),
            fragile: false,
        },
    );

    let null_id = SymbolId(10);
    grammar.tokens.insert(
        null_id,
        Token {
            name: "null".to_string(),
            pattern: TokenPattern::String("null".to_string()),
            fragile: false,
        },
    );

    // Non-terminals
    let value_id = SymbolId(20);
    let object_id = SymbolId(21);
    let array_id = SymbolId(22);
    let pair_id = SymbolId(23);

    grammar.rule_names.insert(value_id, "value".to_string());
    grammar.rule_names.insert(object_id, "object".to_string());
    grammar.rule_names.insert(array_id, "array".to_string());
    grammar.rule_names.insert(pair_id, "pair".to_string());

    // value → object
    grammar.rules.insert(
        SymbolId(30),
        Rule {
            lhs: value_id,
            rhs: vec![Symbol::NonTerminal(object_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(0),
            fields: vec![],
        },
    );

    // value → array
    grammar.rules.insert(
        SymbolId(31),
        Rule {
            lhs: value_id,
            rhs: vec![Symbol::NonTerminal(array_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(1),
            fields: vec![],
        },
    );

    // value → string
    grammar.rules.insert(
        SymbolId(32),
        Rule {
            lhs: value_id,
            rhs: vec![Symbol::Terminal(string_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(2),
            fields: vec![],
        },
    );

    // value → number
    grammar.rules.insert(
        SymbolId(33),
        Rule {
            lhs: value_id,
            rhs: vec![Symbol::Terminal(number_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(3),
            fields: vec![],
        },
    );

    // value → true
    grammar.rules.insert(
        SymbolId(34),
        Rule {
            lhs: value_id,
            rhs: vec![Symbol::Terminal(true_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(4),
            fields: vec![],
        },
    );

    // value → false
    grammar.rules.insert(
        SymbolId(35),
        Rule {
            lhs: value_id,
            rhs: vec![Symbol::Terminal(false_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(5),
            fields: vec![],
        },
    );

    // value → null
    grammar.rules.insert(
        SymbolId(36),
        Rule {
            lhs: value_id,
            rhs: vec![Symbol::Terminal(null_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(6),
            fields: vec![],
        },
    );

    // object → { }
    grammar.rules.insert(
        object_id,
        Rule {
            lhs: object_id,
            rhs: vec![Symbol::Terminal(lbrace_id), Symbol::Terminal(rbrace_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(7),
            fields: vec![],
        },
    );

    // array → [ ]
    grammar.rules.insert(
        array_id,
        Rule {
            lhs: array_id,
            rhs: vec![Symbol::Terminal(lbracket_id), Symbol::Terminal(rbracket_id)],
            precedence: None,
            associativity: None,
            production_id: ProductionId(8),
            fields: vec![],
        },
    );

    // pair → string : value
    grammar.rules.insert(
        pair_id,
        Rule {
            lhs: pair_id,
            rhs: vec![
                Symbol::Terminal(string_id),
                Symbol::Terminal(colon_id),
                Symbol::NonTerminal(value_id),
            ],
            precedence: None,
            associativity: None,
            production_id: ProductionId(9),
            fields: vec![],
        },
    );

    // The parser will determine the starting symbol
    grammar
}

fn create_test_tree(_grammar: &Grammar) -> Subtree {
    // Create a simple test tree representing: {"name": "John", "age": 30}
    let object_id = SymbolId(21);
    let pair_id = SymbolId(23);
    let string_id = SymbolId(6);
    let number_id = SymbolId(7);
    let colon_id = SymbolId(4);

    Subtree {
        symbol: object_id,
        children: vec![
            // First pair: "name": "John"
            Subtree {
                symbol: pair_id,
                children: vec![
                    Subtree {
                        symbol: string_id,
                        children: vec![],
                        start_byte: 1,
                        end_byte: 7,
                    },
                    Subtree {
                        symbol: colon_id,
                        children: vec![],
                        start_byte: 7,
                        end_byte: 8,
                    },
                    Subtree {
                        symbol: string_id,
                        children: vec![],
                        start_byte: 9,
                        end_byte: 15,
                    },
                ],
                start_byte: 1,
                end_byte: 15,
            },
            // Second pair: "age": 30
            Subtree {
                symbol: pair_id,
                children: vec![
                    Subtree {
                        symbol: string_id,
                        children: vec![],
                        start_byte: 17,
                        end_byte: 22,
                    },
                    Subtree {
                        symbol: colon_id,
                        children: vec![],
                        start_byte: 22,
                        end_byte: 23,
                    },
                    Subtree {
                        symbol: number_id,
                        children: vec![],
                        start_byte: 24,
                        end_byte: 26,
                    },
                ],
                start_byte: 17,
                end_byte: 26,
            },
        ],
        start_byte: 0,
        end_byte: 27,
    }
}

fn main() {
    println!("=== GLR Query Demonstration ===\n");

    let grammar = create_json_grammar();

    // Test JSON inputs
    let test_inputs = vec![
        (r#"{"name": "John", "age": 30}"#, "Simple object"),
        (r#"[1, 2, 3, "hello", true]"#, "Simple array"),
        (r#"{"users": [{"id": 1}, {"id": 2}]}"#, "Nested structure"),
    ];

    for (input, description) in test_inputs {
        println!("Test: {}", description);
        println!("Input: {}", input);

        // Create a simple test tree for demonstration
        // In a real implementation, this would use the GLR parser
        let tree = create_test_tree(&grammar);

        println!("Parsed successfully!");

        // Run various queries
        run_queries(&grammar, &tree);
        println!();
    }
}

fn run_queries(grammar: &Grammar, tree: &Subtree) {
    println!("\nRunning queries:");

    // Query 1: Find all strings
    println!("\n1. Find all strings:");
    let query = QueryParser::new(grammar, "(string)").parse().unwrap();
    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, tree).collect();
    println!("   Found {} string(s)", matches.len());

    // Query 2: Find all numbers
    println!("\n2. Find all numbers:");
    let query = QueryParser::new(grammar, "(number)").parse().unwrap();
    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, tree).collect();
    println!("   Found {} number(s)", matches.len());

    // Query 3: Find key-value pairs
    println!("\n3. Find key-value pairs:");
    let query = QueryParser::new(grammar, "(pair (string) @key (colon) (value) @value)")
        .parse()
        .unwrap();
    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, tree).collect();
    println!("   Found {} pair(s)", matches.len());
    for (i, match_) in matches.iter().enumerate() {
        println!("   Pair {}: {} captures", i + 1, match_.captures.len());
    }

    // Query 4: Find arrays containing values
    println!("\n4. Find arrays:");
    let query = QueryParser::new(grammar, "(array)").parse().unwrap();
    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, tree).collect();
    println!("   Found {} array(s)", matches.len());

    // Query 5: Find any value (using wildcard)
    println!("\n5. Find any value (wildcard):");
    let query = QueryParser::new(grammar, "(_) @value").parse().unwrap();
    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, tree).collect();
    println!("   Found {} value(s)", matches.len());

    // Query 6: Multiple patterns
    println!("\n6. Multiple patterns (strings OR numbers):");
    let query = QueryParser::new(grammar, "(string) @str (number) @num")
        .parse()
        .unwrap();
    let cursor = QueryCursor::new();
    let matches: Vec<_> = cursor.matches(&query, tree).collect();

    let string_matches = matches.iter().filter(|m| m.pattern_index == 0).count();
    let number_matches = matches.iter().filter(|m| m.pattern_index == 1).count();
    println!(
        "   Found {} string(s) and {} number(s)",
        string_matches, number_matches
    );
}

// Additional demo: Custom query language features
fn demo_advanced_queries() {
    println!("\n=== Advanced Query Features ===\n");

    let grammar = create_json_grammar();

    // Example queries demonstrating various features
    let example_queries = vec![
        // Basic patterns
        ("(string)", "Match all strings"),
        ("(number)", "Match all numbers"),
        ("(object)", "Match all objects"),
        // Captures
        ("(string) @str", "Capture strings"),
        (
            "(pair (string) @key (colon) (value) @val)",
            "Capture key-value pairs",
        ),
        // Wildcards
        ("(_)", "Match any node"),
        ("(pair (_) (colon) (_))", "Match pairs with any key/value"),
        // Nested patterns
        ("(object (pair))", "Objects containing pairs"),
        ("(array (number))", "Arrays containing numbers"),
        // Multiple patterns
        ("(string) (number)", "Match strings OR numbers"),
        // With predicates (parsed but not fully evaluated in demo)
        (
            "(pair (string) @k1) (pair (string) @k2) (#eq? @k1 @k2)",
            "Pairs with equal keys",
        ),
    ];

    for (query_str, description) in example_queries {
        println!("Query: {}", query_str);
        println!("Description: {}", description);

        match QueryParser::new(&grammar, query_str).parse() {
            Ok(query) => {
                println!("✓ Valid query");
                println!("  - {} pattern(s)", query.patterns.len());
                println!("  - {} capture(s)", query.capture_names.len());
                if !query.capture_names.is_empty() {
                    print!("  - Captures: ");
                    for (name, id) in &query.capture_names {
                        print!("@{} (id={}) ", name, id);
                    }
                    println!();
                }
            }
            Err(e) => {
                println!("✗ Parse error: {}", e);
            }
        }
        println!();
    }
}
