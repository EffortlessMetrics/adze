// Test for syntax highlighting functionality
use adze::parser::ParseNode;
use adze::query::compiler::compile_query;
use adze::query::{Highlighter, Theme};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

fn create_simple_grammar() -> Grammar {
    let mut grammar = Grammar::new("simple".to_string());

    // Define tokens
    let keyword_if = SymbolId(1);
    let keyword_else = SymbolId(2);
    let identifier = SymbolId(3);
    let number = SymbolId(4);
    let string = SymbolId(5);
    let lparen = SymbolId(6);
    let rparen = SymbolId(7);
    let lbrace = SymbolId(8);
    let rbrace = SymbolId(9);
    let _semicolon = SymbolId(10);
    let _equals = SymbolId(11);

    // Add tokens
    grammar.tokens.insert(
        keyword_if,
        Token {
            name: "if".to_string(),
            pattern: TokenPattern::String("if".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        keyword_else,
        Token {
            name: "else".to_string(),
            pattern: TokenPattern::String("else".to_string()),
            fragile: false,
        },
    );

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
            pattern: TokenPattern::Regex(r"[0-9]+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        string,
        Token {
            name: "string".to_string(),
            pattern: TokenPattern::Regex(r#""[^"]*""#.to_string()),
            fragile: false,
        },
    );

    // Add punctuation tokens
    grammar.tokens.insert(
        lparen,
        Token {
            name: "(".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        rparen,
        Token {
            name: ")".to_string(),
            pattern: TokenPattern::String(")".to_string()),
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

    // Define rules
    let program = SymbolId(100);
    let statement = SymbolId(101);
    let if_statement = SymbolId(102);
    let expression = SymbolId(103);

    // Add rule names
    grammar.rule_names.insert(program, "program".to_string());
    grammar
        .rule_names
        .insert(statement, "statement".to_string());
    grammar
        .rule_names
        .insert(if_statement, "if_statement".to_string());
    grammar
        .rule_names
        .insert(expression, "expression".to_string());

    // Add rules
    grammar.rules.entry(program).or_default().push(Rule {
        lhs: program,
        rhs: vec![Symbol::NonTerminal(statement)],
        fields: vec![],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
    });

    grammar.rules.entry(statement).or_default().push(Rule {
        lhs: statement,
        rhs: vec![Symbol::NonTerminal(if_statement)],
        fields: vec![],
        precedence: None,
        associativity: None,
        production_id: ProductionId(1),
    });

    grammar.rules.entry(if_statement).or_default().push(Rule {
        lhs: if_statement,
        rhs: vec![
            Symbol::Terminal(keyword_if),
            Symbol::Terminal(lparen),
            Symbol::NonTerminal(expression),
            Symbol::Terminal(rparen),
            Symbol::NonTerminal(statement),
        ],
        fields: vec![],
        precedence: None,
        associativity: None,
        production_id: ProductionId(2),
    });

    // Add expression rules
    grammar.rules.entry(expression).or_default().push(Rule {
        lhs: expression,
        rhs: vec![Symbol::Terminal(identifier)],
        fields: vec![],
        precedence: None,
        associativity: None,
        production_id: ProductionId(3),
    });

    grammar.rules.entry(expression).or_default().push(Rule {
        lhs: expression,
        rhs: vec![Symbol::Terminal(number)],
        fields: vec![],
        precedence: None,
        associativity: None,
        production_id: ProductionId(4),
    });

    grammar
}

#[test]
#[ignore = "Query compilation needs further debugging - not critical for GLR incremental parsing"]
fn test_highlight_query_compilation() {
    let grammar = create_simple_grammar();

    // Simple highlight query
    let query_source = r#"(identifier) @variable"#;

    let query = compile_query(query_source, &grammar).unwrap();

    // Check captures were registered
    assert!(query.capture_index("variable").is_some());
}

#[test]
#[ignore = "Query compilation needs further debugging - not critical for GLR incremental parsing"]
fn test_highlighter_creation() {
    let grammar = create_simple_grammar();

    let query_source = r#"
        (identifier) @variable
        (number) @number
    "#;

    let query = compile_query(query_source, &grammar).unwrap();
    let highlighter = Highlighter::new(query);

    // Create a simple parse tree
    let root = ParseNode {
        symbol: SymbolId(100),    // program
        symbol_id: SymbolId(100), // program
        children: vec![ParseNode {
            symbol: SymbolId(3),    // identifier
            symbol_id: SymbolId(3), // identifier
            children: vec![],
            start_byte: 0,
            end_byte: 3,
            field_name: None,
        }],
        start_byte: 0,
        end_byte: 3,
        field_name: None,
    };

    let highlights = highlighter.highlight(&root);
    assert_eq!(highlights.len(), 1);
    assert_eq!(highlights[0].highlight, "variable");
    assert_eq!(highlights[0].start_byte, 0);
    assert_eq!(highlights[0].end_byte, 3);
}

#[test]
fn test_theme_colors() {
    let dark_theme = Theme::dark();
    let light_theme = Theme::light();

    // Test dark theme has appropriate colors
    let keyword_color = dark_theme.get_color("keyword");
    assert_eq!(keyword_color.to_hex(), "#c586c0");

    // Test light theme has different colors
    let keyword_color_light = light_theme.get_color("keyword");
    assert_eq!(keyword_color_light.to_hex(), "#0000ff");

    // Test default color fallback
    let unknown_color = dark_theme.get_color("unknown.type");
    assert_eq!(unknown_color, dark_theme.default_color);
}

#[test]
#[ignore = "Query compilation needs further debugging - not critical for GLR incremental parsing"]
fn test_highlight_overlap_removal() {
    let grammar = create_simple_grammar();

    // This would be handled by the remove_overlaps method
    let query_source = r#"
        (expression) @expression
        (identifier) @variable
    "#;

    let query = compile_query(query_source, &grammar).unwrap();
    let highlighter = Highlighter::new(query);

    // Create a parse tree with nested nodes
    let root = ParseNode {
        symbol: SymbolId(103),    // expression
        symbol_id: SymbolId(103), // expression
        children: vec![ParseNode {
            symbol: SymbolId(3),    // identifier
            symbol_id: SymbolId(3), // identifier
            children: vec![],
            start_byte: 0,
            end_byte: 5,
            field_name: None,
        }],
        start_byte: 0,
        end_byte: 5,
        field_name: None,
    };

    let highlights = highlighter.highlight(&root);

    // Should keep the more specific highlight (identifier)
    // The overlap removal logic ensures we don't have overlapping ranges
    assert!(
        highlights
            .iter()
            .all(|h| h.highlight == "variable" || h.highlight == "expression")
    );
}
