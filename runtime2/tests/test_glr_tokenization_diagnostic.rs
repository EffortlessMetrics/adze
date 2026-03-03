//! Diagnostic tests for GLR tokenization pipeline
//!
//! This test file isolates tokenization issues in the GLR pipeline to help
//! debug Phase 3.3 parsing failures.

#![cfg(all(feature = "pure-rust", feature = "serialization"))]

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_runtime::{
    Parser,
    language::SymbolMetadata,
    tokenizer::{Matcher, TokenPattern as RuntimeTokenPattern},
};

/// Helper: Create arithmetic grammar (same as end-to-end test)
fn create_grammar() -> Grammar {
    let mut grammar = Grammar {
        name: "arithmetic".to_string(),
        ..Default::default()
    };

    let number_id = SymbolId(1);
    let expr_id = SymbolId(2);

    grammar.tokens.insert(
        number_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex("[0-9]+".to_string()),
            fragile: false,
        },
    );

    grammar.rule_names.insert(expr_id, "expr".to_string());

    grammar.rules.insert(
        expr_id,
        vec![Rule {
            lhs: expr_id,
            rhs: vec![Symbol::Terminal(number_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    let _ = grammar.get_or_build_registry();
    grammar
}

/// Diagnostic Test 1: Verify parse table structure
#[test]
fn test_parse_table_structure() {
    let grammar = create_grammar();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow)
        .unwrap()
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();

    println!("=== Parse Table Structure ===");
    println!("State count: {}", parse_table.state_count);
    println!("Symbol count: {}", parse_table.symbol_count);
    println!("Rules: {:?}", parse_table.rules);
    println!("EOF symbol: {:?}", parse_table.eof_symbol);
    println!("Start symbol: {:?}", parse_table.start_symbol);

    println!("\n=== Action Table ===");
    for (state_idx, row) in parse_table.action_table.iter().enumerate() {
        for (sym_idx, cell) in row.iter().enumerate() {
            if !cell.is_empty() {
                println!("State {} × Symbol {} → {:?}", state_idx, sym_idx, cell);
            }
        }
    }

    println!("\n=== Goto Table ===");
    for (state_idx, row) in parse_table.goto_table.iter().enumerate() {
        for (nt_idx, next_state) in row.iter().enumerate() {
            if next_state.0 != 0 || state_idx == 0 {
                println!(
                    "State {} × NT {} → State {}",
                    state_idx, nt_idx, next_state.0
                );
            }
        }
    }

    // Assertions
    assert!(parse_table.state_count > 0, "Should have states");
    assert!(
        parse_table.symbol_count >= 3,
        "Should have EOF + number + expr"
    );
    assert!(!parse_table.rules.is_empty(), "Should have rules");
}

/// Diagnostic Test 2: Verify symbol metadata configuration
#[test]
fn test_symbol_metadata_setup() {
    let metadata = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: false,
            is_supertype: false,
        }, // EOF
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }, // number
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        }, // expr
    ];

    println!("=== Symbol Metadata ===");
    for (idx, meta) in metadata.iter().enumerate() {
        println!(
            "Symbol {}: terminal={}, visible={}, supertype={}",
            idx, meta.is_terminal, meta.is_visible, meta.is_supertype
        );
    }

    assert_eq!(metadata.len(), 3, "Should have 3 symbols");
    assert!(metadata[0].is_terminal, "EOF should be terminal");
    assert!(metadata[1].is_terminal, "number should be terminal");
    assert!(!metadata[2].is_terminal, "expr should be non-terminal");
}

/// Diagnostic Test 3: Verify token patterns configuration
#[test]
fn test_token_patterns_setup() {
    let patterns = vec![
        RuntimeTokenPattern {
            symbol_id: SymbolId(0),
            matcher: Matcher::Regex(regex::Regex::new(r"$").unwrap()),
            is_keyword: false,
        },
        RuntimeTokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Regex(regex::Regex::new(r"[0-9]+").unwrap()),
            is_keyword: false,
        },
    ];

    println!("=== Token Patterns ===");
    for (idx, pattern) in patterns.iter().enumerate() {
        println!(
            "Pattern {}: symbol_id={:?}, is_keyword={}",
            idx, pattern.symbol_id, pattern.is_keyword
        );

        // Test regex matching
        if let Matcher::Regex(ref regex) = pattern.matcher {
            let test_input = "42";
            if let Some(m) = regex.find(test_input) {
                println!("  Regex matches '{}': {:?}", test_input, m);
            } else {
                println!("  Regex does NOT match '{}'", test_input);
            }
        }
    }

    assert_eq!(patterns.len(), 2, "Should have 2 token patterns");
    assert_eq!(
        patterns[1].symbol_id,
        SymbolId(1),
        "number pattern should be for SymbolId(1)"
    );
}

/// Diagnostic Test 4: Verify regex matches input
#[test]
fn test_regex_matching() {
    let number_regex = regex::Regex::new(r"[0-9]+").unwrap();
    let input = "42";

    println!("=== Regex Matching ===");
    println!("Pattern: [0-9]+");
    println!("Input: '{}'", input);

    match number_regex.find(input) {
        Some(m) => {
            println!(
                "Match found: start={}, end={}, text='{}'",
                m.start(),
                m.end(),
                &input[m.start()..m.end()]
            );
            assert_eq!(m.start(), 0, "Should match from start");
            assert_eq!(m.end(), 2, "Should match both digits");
        }
        None => {
            panic!("Regex should match '42'");
        }
    }
}

/// Diagnostic Test 5: Test Parser GLR mode setup
#[test]
fn test_parser_glr_mode_setup() {
    let grammar = create_grammar();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow)
        .unwrap()
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();

    // Leak table for 'static lifetime
    let table_static: &'static _ = Box::leak(Box::new(parse_table));

    let mut parser = Parser::new();
    parser
        .set_glr_table(table_static)
        .expect("Setting GLR table should succeed");

    println!("=== Parser Setup ===");
    println!("GLR mode: {}", parser.is_glr_mode());

    assert!(parser.is_glr_mode(), "Parser should be in GLR mode");

    // Set metadata
    let metadata = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: false,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        },
    ];
    parser
        .set_symbol_metadata(metadata)
        .expect("Setting metadata should succeed");

    // Set token patterns
    let patterns = vec![
        RuntimeTokenPattern {
            symbol_id: SymbolId(0),
            matcher: Matcher::Regex(regex::Regex::new(r"$").unwrap()),
            is_keyword: false,
        },
        RuntimeTokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Regex(regex::Regex::new(r"[0-9]+").unwrap()),
            is_keyword: false,
        },
    ];
    parser
        .set_token_patterns(patterns)
        .expect("Setting patterns should succeed");

    println!("Parser configured successfully");
}

/// Diagnostic Test 6: Check tokenizer output
#[test]
fn test_tokenizer_output() {
    use adze_runtime::tokenizer::{Tokenizer, WhitespaceMode};

    let patterns = vec![
        RuntimeTokenPattern {
            symbol_id: SymbolId(0),
            matcher: Matcher::Regex(regex::Regex::new(r"$").unwrap()),
            is_keyword: false,
        },
        RuntimeTokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Regex(regex::Regex::new(r"[0-9]+").unwrap()),
            is_keyword: false,
        },
    ];

    let tokenizer = Tokenizer::new(patterns, WhitespaceMode::Skip);
    let input = b"42";

    println!("=== Tokenizer Output ===");
    println!("Input: {:?}", std::str::from_utf8(input).unwrap());

    match tokenizer.scan(input) {
        Ok(tokens) => {
            println!("Tokens produced: {}", tokens.len());
            for (idx, token) in tokens.iter().enumerate() {
                println!(
                    "  Token {}: kind={}, start={}, end={}",
                    idx, token.kind, token.start, token.end
                );
                if token.start < input.len() as u32 && token.end <= input.len() as u32 {
                    let text = &input[token.start as usize..token.end as usize];
                    println!("    Text: {:?}", std::str::from_utf8(text).unwrap());
                }
            }
        }
        Err(e) => {
            println!("Tokenizer failed: {}", e);
            panic!("Tokenization failed: {}", e);
        }
    }
}

/// Diagnostic Test 7: Attempt minimal parse
#[test]
#[ignore] // Enable manually to see detailed error
fn test_minimal_parse_attempt() {
    let grammar = create_grammar();

    println!("=== Grammar Info ===");
    println!("Grammar name: {}", grammar.name);
    println!("Tokens: {:?}", grammar.tokens.keys().collect::<Vec<_>>());
    println!("Rule names: {:?}", grammar.rule_names);

    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow)
        .unwrap()
        .normalize_eof_to_zero()
        .with_detected_goto_indexing();

    println!("\n=== ParseTable Grammar ===");
    println!("ParseTable grammar name: {}", parse_table.grammar.name);
    println!(
        "ParseTable rule_names: {:?}",
        parse_table.grammar.rule_names
    );

    let table_static: &'static _ = Box::leak(Box::new(parse_table));

    let mut parser = Parser::new();
    parser.set_glr_table(table_static).unwrap();

    let metadata = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: false,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        },
    ];
    parser.set_symbol_metadata(metadata).unwrap();

    let patterns = vec![
        RuntimeTokenPattern {
            symbol_id: SymbolId(0),
            matcher: Matcher::Regex(regex::Regex::new(r"$").unwrap()),
            is_keyword: false,
        },
        RuntimeTokenPattern {
            symbol_id: SymbolId(1),
            matcher: Matcher::Regex(regex::Regex::new(r"[0-9]+").unwrap()),
            is_keyword: false,
        },
    ];
    parser.set_token_patterns(patterns).unwrap();

    println!("\n=== Attempting Parse ===");
    let input = b"42";
    println!("Input: {:?}", std::str::from_utf8(input).unwrap());

    match parser.parse(input, None) {
        Ok(tree) => {
            println!("✓ Parse succeeded!");
            let root = tree.root_node();
            println!("  Root kind: {}", root.kind());
            println!("  Root kind_id: {:?}", root.kind_id());
            println!("  Child count: {}", root.child_count());

            if root.child_count() > 0 {
                let child = root.child(0).unwrap();
                println!("  Child 0 kind: {}", child.kind());
                println!("  Child 0 kind_id: {:?}", child.kind_id());
            }
        }
        Err(e) => {
            println!("✗ Parse failed: {}", e);
            println!("  Error: {:?}", e);
            panic!("Parse failed: {}", e);
        }
    }
}
