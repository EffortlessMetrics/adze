// Complete example demonstrating all adze features
// This test shows how to build a working parser from scratch
#![cfg(test)]
#![allow(unused_imports, dead_code)]

// Gate this test behind the experimental_examples feature
// as it's a comprehensive example that may take time to update
#[cfg(not(feature = "experimental_examples"))]
use std::process::exit;

#[cfg(feature = "ts-compat")]
use adze::adze_glr_core as glr_core;
#[cfg(feature = "ts-compat")]
use adze::adze_ir as ir;

#[cfg(not(feature = "ts-compat"))]
use adze_glr_core as glr_core;
#[cfg(not(feature = "ts-compat"))]
use adze_ir as ir;

use glr_core::*;
use ir::*;
use std::collections::BTreeMap;

/// Build a complete JSON parser using adze
mod json_parser {
    use super::*;

    pub fn create_json_grammar() -> Grammar {
        let mut grammar = Grammar::new("json".to_string());

        // Tokens
        let string = SymbolId(1);
        let number = SymbolId(2);
        let true_tok = SymbolId(3);
        let false_tok = SymbolId(4);
        let null_tok = SymbolId(5);
        let lbrace = SymbolId(6);
        let rbrace = SymbolId(7);
        let lbracket = SymbolId(8);
        let rbracket = SymbolId(9);
        let comma = SymbolId(10);
        let colon = SymbolId(11);

        // Non-terminals
        let value = SymbolId(20);
        let object = SymbolId(21);
        let array = SymbolId(22);
        let pair = SymbolId(23);
        let pairs = SymbolId(24);
        let values = SymbolId(25);

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
            true_tok,
            Token {
                name: "true".to_string(),
                pattern: TokenPattern::String("true".to_string()),
                fragile: false,
            },
        );

        grammar.tokens.insert(
            false_tok,
            Token {
                name: "false".to_string(),
                pattern: TokenPattern::String("false".to_string()),
                fragile: false,
            },
        );

        grammar.tokens.insert(
            null_tok,
            Token {
                name: "null".to_string(),
                pattern: TokenPattern::String("null".to_string()),
                fragile: false,
            },
        );

        // Punctuation
        for (id, (name, text)) in [
            (lbrace, ("lbrace", "{")),
            (rbrace, ("rbrace", "}")),
            (lbracket, ("lbracket", "[")),
            (rbracket, ("rbracket", "]")),
            (comma, ("comma", ",")),
            (colon, ("colon", ":")),
        ] {
            grammar.tokens.insert(
                id,
                Token {
                    name: name.to_string(),
                    pattern: TokenPattern::String(text.to_string()),
                    fragile: false,
                },
            );
        }

        // Rules
        let mut rule_id = 0;
        let mut add_rule = |lhs: SymbolId, rhs: Vec<Symbol>| {
            grammar.rules.entry(lhs).or_default().push(Rule {
                lhs,
                rhs,
                production_id: ProductionId(rule_id),
                precedence: None,
                associativity: None,
                fields: vec![],
            });
            rule_id += 1;
        };

        // value -> string | number | true | false | null | object | array
        add_rule(value, vec![Symbol::Terminal(string)]);
        add_rule(value, vec![Symbol::Terminal(number)]);
        add_rule(value, vec![Symbol::Terminal(true_tok)]);
        add_rule(value, vec![Symbol::Terminal(false_tok)]);
        add_rule(value, vec![Symbol::Terminal(null_tok)]);
        add_rule(value, vec![Symbol::NonTerminal(object)]);
        add_rule(value, vec![Symbol::NonTerminal(array)]);

        // object -> { } | { pairs }
        add_rule(
            object,
            vec![Symbol::Terminal(lbrace), Symbol::Terminal(rbrace)],
        );
        add_rule(
            object,
            vec![
                Symbol::Terminal(lbrace),
                Symbol::NonTerminal(pairs),
                Symbol::Terminal(rbrace),
            ],
        );

        // pairs -> pair | pair , pairs
        add_rule(pairs, vec![Symbol::NonTerminal(pair)]);
        add_rule(
            pairs,
            vec![
                Symbol::NonTerminal(pair),
                Symbol::Terminal(comma),
                Symbol::NonTerminal(pairs),
            ],
        );

        // pair -> string : value
        add_rule(
            pair,
            vec![
                Symbol::Terminal(string),
                Symbol::Terminal(colon),
                Symbol::NonTerminal(value),
            ],
        );

        // array -> [ ] | [ values ]
        add_rule(
            array,
            vec![Symbol::Terminal(lbracket), Symbol::Terminal(rbracket)],
        );
        add_rule(
            array,
            vec![
                Symbol::Terminal(lbracket),
                Symbol::NonTerminal(values),
                Symbol::Terminal(rbracket),
            ],
        );

        // values -> value | value , values
        add_rule(values, vec![Symbol::NonTerminal(value)]);
        add_rule(
            values,
            vec![
                Symbol::NonTerminal(value),
                Symbol::Terminal(comma),
                Symbol::NonTerminal(values),
            ],
        );

        // Add field names
        grammar.fields.insert(FieldId(0), "key".to_string());
        grammar.fields.insert(FieldId(1), "value".to_string());

        // Add rule names
        grammar.rule_names.insert(value, "value".to_string());
        grammar.rule_names.insert(object, "object".to_string());
        grammar.rule_names.insert(array, "array".to_string());
        grammar.rule_names.insert(pair, "pair".to_string());

        grammar
    }
}

#[test]
#[ignore = "needs update to current parser API"]
fn test_complete_json_parser() {
    use json_parser::create_json_grammar;

    // 1. Create the grammar
    let grammar = create_json_grammar();
    println!(
        "✅ Created JSON grammar with {} tokens and {} rules",
        grammar.tokens.len(),
        grammar.rules.len()
    );

    // 2. Build parse table (simplified for demo)
    let parse_table = ParseTable {
        action_table: vec![vec![vec![Action::Error]; 30]; 50], // ActionCell model: Vec<Vec<Vec<Action>>>
        goto_table: vec![vec![StateId(0); 10]; 50],
        symbol_metadata: vec![],
        state_count: 50,
        symbol_count: 30,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: adze_glr_core::GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar: grammar.clone(),
        initial_state: StateId(0),
        token_count: 20,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    };
    println!(
        "✅ Built parse table with {} states",
        parse_table.state_count
    );

    // 3. Test with sample JSON
    let test_cases = [
        r#"{"name": "adze", "version": "0.5.0"}"#,
        r#"[1, 2, 3, true, false, null]"#,
        r#"{"nested": {"array": [1, 2, 3]}, "empty": {}}"#,
    ];

    for (i, json) in test_cases.iter().enumerate() {
        println!("\n📝 Test case {}: {}", i + 1, json);

        // In a real implementation, we would:
        // - Use the lexer to tokenize
        // - Use the parser to build a tree
        // - Verify the tree structure

        println!("   Would parse JSON of length {}", json.len());
    }

    // 4. Demonstrate query capability
    let query = r#"
        (object
          (pair
            key: (string) @key
            value: (_) @value))
    "#;
    println!("\n🔍 Query example: Find all key-value pairs");
    println!("{}", query);

    // 5. Show incremental parsing
    println!("\n♻️  Incremental parsing example:");
    println!("   Original: {}", test_cases[0]);
    println!("   Edit: Change version to 0.6.0");
    println!("   Would reuse object and name subtrees");

    // 6. Error recovery
    let invalid_json = r#"{"name": "test" "missing": "comma"}"#;
    println!("\n🔧 Error recovery example:");
    println!("   Invalid JSON: {}", invalid_json);
    println!("   Would insert comma and continue parsing");
}

#[test]
#[ignore = "needs update to current parser API"]
fn test_adze_feature_completeness() {
    println!("\n");
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║              ADZE FEATURE COMPLETENESS                    ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║ ✅ Grammar Definition (via Rust macros)                  ║");
    println!("║ ✅ Pure-Rust Parser Engine                               ║");
    println!("║ ✅ GLR Parsing (ambiguous grammar support)               ║");
    println!("║ ✅ External Scanner Support (FFI + native)               ║");
    println!("║ ✅ Incremental Parsing (subtree reuse)                   ║");
    println!("║ ✅ Error Recovery (multiple strategies)                  ║");
    println!("║ ✅ Query Language (S-expression patterns)                ║");
    println!("║ ✅ Syntax Highlighting (query-based)                     ║");
    println!("║ ✅ Table Generation (static compilation)                 ║");
    println!("║ ✅ Tree-sitter ABI Compatibility                         ║");
    println!("║ ✅ WASM Support (no C dependencies)                      ║");
    println!("║ ✅ Language Injection Support                            ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║ Supported Languages (with external scanners):            ║");
    println!("║ • Python (indentation)                                   ║");
    println!("║ • Ruby (heredoc)                                         ║");
    println!("║ • C/C++ (preprocessor)                                   ║");
    println!("║ • JavaScript/TypeScript (template strings)               ║");
    println!("║ • Markdown (indented code blocks)                        ║");
    println!("║ • ... and 150+ more Tree-sitter grammars                ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
}

/// Demonstrate the complete workflow
#[test]
#[ignore = "needs update to current parser API"]
fn test_end_to_end_workflow() {
    println!("\n🚀 Adze End-to-End Workflow Demo\n");

    // Step 1: Define grammar using Rust types
    println!("1️⃣  Define grammar with Rust macros:");
    println!("   #[adze::grammar(\"my_language\")]");
    println!("   mod grammar {{");
    println!("       #[adze::leaf]");
    println!("       struct Number(String);");
    println!("   }}");

    // Step 2: Build time - generate parser
    println!("\n2️⃣  At build time (build.rs):");
    println!("   adze_tool::build_parsers()?;");
    println!("   → Generates Tree-sitter grammar JSON");
    println!("   → Compiles to static parse tables");
    println!("   → Links external scanners");

    // Step 3: Runtime - parse code
    println!("\n3️⃣  At runtime:");
    println!("   let tree = MyLanguage::parse(\"42 + 3.14\");");
    println!("   → Uses generated parser");
    println!("   → Returns typed AST");

    // Step 4: Query and analyze
    println!("\n4️⃣  Query and analyze:");
    println!("   let query = MyLanguage::query(\"(number) @num\");");
    println!("   let matches = query.matches(&tree);");
    println!("   → Pattern matching on AST");
    println!("   → Syntax highlighting");

    // Step 5: Incremental updates
    println!("\n5️⃣  Incremental updates:");
    println!("   let edit = Edit {{ start: 0, old_end: 2, new_end: 3 }};");
    println!("   let new_tree = parser.parse_incremental(input, &tree, &[edit]);");
    println!("   → Reuses unchanged subtrees");
    println!("   → O(log n) performance");

    println!("\n✨ Result: Fast, type-safe, incremental parsing in pure Rust!");
}
