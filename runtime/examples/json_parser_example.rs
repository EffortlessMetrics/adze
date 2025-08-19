// Example: Building and using a JSON parser with the GLR parser
use rust_sitter::glr_parser::GLRParser;
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

/// Build a complete JSON grammar
fn build_json_grammar() -> Grammar {
    let mut grammar = Grammar::new("json".to_string());

    // Allocate symbol IDs
    let mut next_symbol_id = 1u16;
    let mut next_production_id = 0u16;

    // Helper to get next symbol ID
    let mut alloc_symbol = || {
        let id = SymbolId(next_symbol_id);
        next_symbol_id += 1;
        id
    };

    let mut alloc_production = || {
        let id = ProductionId(next_production_id);
        next_production_id += 1;
        id
    };

    // Define terminal symbol IDs
    let string_id = alloc_symbol();
    let number_id = alloc_symbol();
    let true_id = alloc_symbol();
    let false_id = alloc_symbol();
    let null_id = alloc_symbol();
    let lbrace_id = alloc_symbol();
    let rbrace_id = alloc_symbol();
    let lbracket_id = alloc_symbol();
    let rbracket_id = alloc_symbol();
    let comma_id = alloc_symbol();
    let colon_id = alloc_symbol();

    // Define non-terminal symbol IDs
    let value_id = alloc_symbol();
    let object_id = alloc_symbol();
    let array_id = alloc_symbol();
    let members_id = alloc_symbol();
    let member_id = alloc_symbol();
    let elements_id = alloc_symbol();

    // Add tokens to grammar
    grammar.tokens.insert(
        string_id,
        Token {
            name: "string".to_string(),
            pattern: TokenPattern::Regex(r#""([^"\\]|\\.)*""#.to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(
                r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?".to_string(),
            ),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        true_id,
        Token {
            name: "true".to_string(),
            pattern: TokenPattern::String("true".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        false_id,
        Token {
            name: "false".to_string(),
            pattern: TokenPattern::String("false".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        null_id,
        Token {
            name: "null".to_string(),
            pattern: TokenPattern::String("null".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        lbrace_id,
        Token {
            name: "lbrace".to_string(),
            pattern: TokenPattern::String("{".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        rbrace_id,
        Token {
            name: "rbrace".to_string(),
            pattern: TokenPattern::String("}".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        lbracket_id,
        Token {
            name: "lbracket".to_string(),
            pattern: TokenPattern::String("[".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        rbracket_id,
        Token {
            name: "rbracket".to_string(),
            pattern: TokenPattern::String("]".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        comma_id,
        Token {
            name: "comma".to_string(),
            pattern: TokenPattern::String(",".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        colon_id,
        Token {
            name: "colon".to_string(),
            pattern: TokenPattern::String(":".to_string()),
            fragile: false,
        },
    );

    // Add rule names
    grammar.rule_names.insert(value_id, "value".to_string());
    grammar.rule_names.insert(object_id, "object".to_string());
    grammar.rule_names.insert(array_id, "array".to_string());
    grammar.rule_names.insert(members_id, "members".to_string());
    grammar.rule_names.insert(member_id, "member".to_string());
    grammar
        .rule_names
        .insert(elements_id, "elements".to_string());

    // Add field names
    let key_field = FieldId(0);
    let value_field = FieldId(1);
    grammar.fields.insert(key_field, "key".to_string());
    grammar.fields.insert(value_field, "value".to_string());

    // Define rules
    // value → string | number | true | false | null | object | array
    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: value_id,
        rhs: vec![Symbol::Terminal(string_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });

    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: value_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });
    next_symbol_id += 1;

    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: value_id,
        rhs: vec![Symbol::Terminal(true_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });
    next_symbol_id += 1;

    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: value_id,
        rhs: vec![Symbol::Terminal(false_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });
    next_symbol_id += 1;

    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: value_id,
        rhs: vec![Symbol::Terminal(null_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });
    next_symbol_id += 1;

    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: value_id,
        rhs: vec![Symbol::NonTerminal(object_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });
    next_symbol_id += 1;

    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: value_id,
        rhs: vec![Symbol::NonTerminal(array_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });
    next_symbol_id += 1;

    // object → { } | { members }
    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: object_id,
        rhs: vec![Symbol::Terminal(lbrace_id), Symbol::Terminal(rbrace_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });

    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: object_id,
        rhs: vec![
            Symbol::Terminal(lbrace_id),
            Symbol::NonTerminal(members_id),
            Symbol::Terminal(rbrace_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });
    next_symbol_id += 1;

    // members → member | member , members
    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: members_id,
        rhs: vec![Symbol::NonTerminal(member_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });

    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: members_id,
        rhs: vec![
            Symbol::NonTerminal(member_id),
            Symbol::Terminal(comma_id),
            Symbol::NonTerminal(members_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });
    next_symbol_id += 1;

    // member → string : value (with field names)
    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: member_id,
        rhs: vec![
            Symbol::Terminal(string_id),
            Symbol::Terminal(colon_id),
            Symbol::NonTerminal(value_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(key_field, 0), (value_field, 2)], // key at position 0, value at position 2
        production_id: prod_id,
    });

    // array → [ ] | [ elements ]
    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: array_id,
        rhs: vec![Symbol::Terminal(lbracket_id), Symbol::Terminal(rbracket_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });

    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: array_id,
        rhs: vec![
            Symbol::Terminal(lbracket_id),
            Symbol::NonTerminal(elements_id),
            Symbol::Terminal(rbracket_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });
    next_symbol_id += 1;

    // elements → value | value , elements
    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: elements_id,
        rhs: vec![Symbol::NonTerminal(value_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });

    let prod_id = alloc_production();
    grammar.add_rule(Rule {
        lhs: elements_id,
        rhs: vec![
            Symbol::NonTerminal(value_id),
            Symbol::Terminal(comma_id),
            Symbol::NonTerminal(elements_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: prod_id,
    });

    grammar
}

fn main() {
    println!("Building JSON grammar...");
    let grammar = build_json_grammar();

    // Validate grammar
    match grammar.validate() {
        Ok(()) => println!("Grammar is valid!"),
        Err(e) => {
            eprintln!("Grammar validation failed: {}", e);
            return;
        }
    }

    println!("\nComputing FIRST/FOLLOW sets...");
    let first_follow = FirstFollowSets::compute(&grammar);

    println!("Building LR(1) automaton...");
    match build_lr1_automaton(&grammar, &first_follow) {
        Ok(parse_table) => {
            println!("Parse table built successfully!");
            println!("- {} states", parse_table.state_count);
            println!("- {} symbols", parse_table.symbol_count);

            // Create GLR parser - clone the grammar since GLRParser takes ownership
            let mut parser = GLRParser::new(parse_table, grammar.clone());

            // Example: Parse a simple JSON value
            let input = "\"hello\"";
            println!("\nParsing: {}", input);

            // In a real implementation, we'd have a lexer that produces these tokens
            // For now, we'll manually provide the tokens
            parser.process_token(SymbolId(1), input, 0); // string token
            parser.process_eof(input.len());

            if let Some(tree) = parser.get_best_parse() {
                println!("Parse successful!");
                println!("Root symbol: {:?}", tree.node.symbol_id);
            } else {
                println!("Parse failed!");
            }
        }
        Err(e) => {
            eprintln!("Failed to build parse table: {:?}", e);
        }
    }
}
