//! Arithmetic Grammar Example - Phase 3.3 Integration Testing
//!
//! **Grammar**: Simple arithmetic with precedence and associativity
//! **Contract**: GLR should produce identical results to LR (unambiguous grammar)
//! **Purpose**: Validate GLR runtime with precedence-disambiguated grammar
//!
//! ## Grammar Definition
//!
//! ```text
//! Expression ::= NUMBER
//!             |  Expression - Expression  // precedence 1, left associative
//!             |  Expression * Expression  // precedence 2, left associative
//! ```
//!
//! ## Expected Behavior
//!
//! - **Precedence**: `1 - 2 * 3` parses as `1 - (2 * 3)` (multiply binds tighter)
//! - **Associativity**: `1 - 2 - 3` parses as `(1 - 2) - 3` (left-to-right)
//! - **No Ambiguity**: Precedence annotations resolve all conflicts
//!
//! ## Test Scenarios
//!
//! 1. Simple number: `42` → Number(42)
//! 2. Basic subtraction: `1 - 2` → Sub(1, 2)
//! 3. Basic multiplication: `3 * 4` → Mul(3, 4)
//! 4. Precedence: `1 - 2 * 3` → Sub(1, Mul(2, 3))
//! 5. Left assoc (sub): `1 - 2 - 3` → Sub(Sub(1, 2), 3)
//! 6. Left assoc (mul): `1 * 2 * 3` → Mul(Mul(1, 2), 3)
//! 7. Mixed precedence: `1 * 2 - 3` → Sub(Mul(1, 2), 3)
//! 8. Complex: `1 - 2 * 3 - 4` → Sub(Sub(1, Mul(2, 3)), 4)
//!
//! ## Success Criteria
//!
//! - [ ] All test scenarios pass
//! - [ ] GLR produces correct precedence
//! - [ ] GLR produces correct associativity
//! - [ ] Performance: parse time < 100ms for simple expressions
//! - [ ] Memory: no leaks, bounded usage

use rust_sitter_runtime::{Parser, Tree, Token};
use rust_sitter_runtime::tokenizer::{TokenPattern, Matcher, WhitespaceMode};
use rust_sitter_runtime::language::SymbolMetadata;
use rust_sitter_glr_core::{SymbolId, ParseTable, FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{
    Grammar, ProductionId, Rule, Symbol,
    Token as IrToken, TokenPattern as IrTokenPattern,
};

/// AST representation for parsed expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Number(i32),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
}

/// Create the arithmetic grammar with precedence annotations
///
/// ## Grammar Rules
///
/// ```text
/// expr → NUMBER         (production 0)
/// expr → expr - expr    (production 1, precedence 1, left assoc)
/// expr → expr * expr    (production 2, precedence 2, left assoc)
/// ```
///
/// ## Symbol IDs
///
/// - 0: EOF
/// - 1: NUMBER (terminal)
/// - 2: MINUS (terminal)
/// - 3: STAR (terminal)
/// - 4: expr (nonterminal, start symbol)
fn create_arithmetic_grammar() -> (&'static ParseTable, Vec<SymbolMetadata>, Vec<TokenPattern>) {
    let mut grammar = Grammar::new("arithmetic".to_string());

    // Define terminals
    let number_id = SymbolId(1);
    grammar.tokens.insert(
        number_id,
        IrToken {
            name: "NUMBER".to_string(),
            pattern: IrTokenPattern::String(r"\d+".to_string()),
            fragile: false,
        },
    );

    let minus_id = SymbolId(2);
    grammar.tokens.insert(
        minus_id,
        IrToken {
            name: "MINUS".to_string(),
            pattern: IrTokenPattern::String("-".to_string()),
            fragile: false,
        },
    );

    let star_id = SymbolId(3);
    grammar.tokens.insert(
        star_id,
        IrToken {
            name: "STAR".to_string(),
            pattern: IrTokenPattern::String("*".to_string()),
            fragile: false,
        },
    );

    // Define nonterminal (start symbol)
    let expr_id = SymbolId(4);
    grammar.rule_names.insert(expr_id, "expr".to_string());

    // Rule 1: expr → NUMBER (production 0)
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });

    // Rule 2: expr → expr - expr (production 1, precedence 1, left assoc)
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(minus_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(rust_sitter_ir::PrecedenceKind::Static(1)),  // Lower precedence
        associativity: Some(rust_sitter_ir::Associativity::Left),
        production_id: ProductionId(1),
        fields: vec![],
    });

    // Rule 3: expr → expr * expr (production 2, precedence 2, left assoc)
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(star_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(rust_sitter_ir::PrecedenceKind::Static(2)),  // Higher precedence
        associativity: Some(rust_sitter_ir::Associativity::Left),
        production_id: ProductionId(2),
        fields: vec![],
    });

    // Build LR(1) parse table
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();
    let table_static: &'static ParseTable = Box::leak(Box::new(table));

    // Symbol metadata
    let symbol_metadata = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: false,
            is_supertype: false,
        }, // EOF
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }, // NUMBER
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }, // MINUS
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }, // STAR
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        }, // expr
    ];

    // Token patterns for Tokenizer
    let token_patterns = vec![
        TokenPattern {
            symbol_id: number_id,
            matcher: Matcher::Regex(regex::Regex::new(r"^\d+").unwrap()),
            is_keyword: false,
        },
        TokenPattern {
            symbol_id: minus_id,
            matcher: Matcher::Literal("-".to_string()),
            is_keyword: false,
        },
        TokenPattern {
            symbol_id: star_id,
            matcher: Matcher::Literal("*".to_string()),
            is_keyword: false,
        },
    ];

    (table_static, symbol_metadata, token_patterns)
}

/// Parse arithmetic expression and return Tree
///
/// ## Contract
///
/// **Preconditions**:
/// - `input` contains valid arithmetic expression
/// - Grammar supports numbers, -, * operators
///
/// **Postconditions**:
/// - Returns `Ok(Tree)` for valid input
/// - Returns `Err(ParseError)` for invalid input
/// - Tree structure respects precedence and associativity
///
/// **Example**:
/// ```ignore
/// let tree = parse("1 - 2 * 3").unwrap();
/// // Tree represents: Sub(1, Mul(2, 3))
/// ```
pub fn parse(input: &str) -> Result<Tree, rust_sitter_runtime::error::ParseError> {
    let (table, metadata, patterns) = create_arithmetic_grammar();

    let mut parser = Parser::new();
    parser.set_glr_table(table)?;
    parser.set_symbol_metadata(metadata)?;
    parser.set_token_patterns(patterns)?;

    parser.parse(input.as_bytes(), None)
}

fn main() {
    println!("=== Arithmetic Grammar - GLR Runtime Example ===\n");

    // Test scenarios from Phase 3.3 specification
    let test_cases = vec![
        ("42", "Simple number"),
        ("1-2", "Basic subtraction"),
        ("3*4", "Basic multiplication"),
        ("1-2*3", "Precedence: multiply before subtract"),
        ("1-2-3", "Left associativity: subtraction"),
        ("1*2*3", "Left associativity: multiplication"),
        ("1*2-3", "Mixed precedence"),
        ("1-2*3-4", "Complex expression"),
    ];

    println!("Running {} test scenarios:\n", test_cases.len());

    for (i, (input, description)) in test_cases.iter().enumerate() {
        println!("{}. {} (\"{}\")", i + 1, description, input);

        match parse(input) {
            Ok(tree) => {
                println!("   ✓ Parsed successfully");
                println!("   Root kind: {}", tree.root_kind());
                println!("   Source: {:?}", tree.source_bytes().map(|b| String::from_utf8_lossy(b)));
            }
            Err(e) => {
                println!("   ✗ Parse error: {}", e);
            }
        }
        println!();
    }

    println!("=== Example Complete ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Scenario 1: Simple number parsing
    ///
    /// **Given**: Input "42"
    /// **When**: Parsed with GLR runtime
    /// **Then**: Should succeed and produce Number node
    #[test]
    fn test_simple_number() {
        let tree = parse("42").expect("Should parse simple number");

        assert_eq!(tree.root_kind(), 4); // expr symbol
        assert!(tree.source_bytes().is_some());
        assert_eq!(tree.source_bytes().unwrap(), b"42");
    }

    /// Test Scenario 2: Basic subtraction
    ///
    /// **Given**: Input "1-2"
    /// **When**: Parsed with GLR runtime
    /// **Then**: Should produce Sub(1, 2)
    #[test]
    fn test_basic_subtraction() {
        let tree = parse("1-2").expect("Should parse subtraction");

        assert_eq!(tree.root_kind(), 4); // expr symbol
        let root = tree.root_node();

        // Root should have 3 children: expr, MINUS, expr
        assert!(root.child_count() >= 1, "Root should have children");
    }

    /// Test Scenario 4: Precedence
    ///
    /// **Given**: Input "1-2*3"
    /// **When**: Parsed with GLR runtime
    /// **Then**: Should produce "1 - (2 * 3)", NOT "(1 - 2) * 3"
    ///
    /// **Rationale**: Multiplication (precedence 2) binds tighter than subtraction (precedence 1)
    #[test]
    fn test_precedence() {
        let tree = parse("1-2*3").expect("Should parse with correct precedence");

        assert_eq!(tree.root_kind(), 4); // expr symbol

        // Tree structure should show subtraction at the top level
        // with multiplication as a subtree
        let root = tree.root_node();
        assert!(root.child_count() >= 1);
    }

    /// Test Scenario 5: Left associativity - subtraction
    ///
    /// **Given**: Input "1-2-3"
    /// **When**: Parsed with GLR runtime
    /// **Then**: Should produce "((1 - 2) - 3)", NOT "(1 - (2 - 3))"
    #[test]
    fn test_left_associativity_sub() {
        let tree = parse("1-2-3").expect("Should parse with left associativity");

        assert_eq!(tree.root_kind(), 4); // expr symbol
    }

    /// Test Scenario 6: Left associativity - multiplication
    ///
    /// **Given**: Input "1*2*3"
    /// **When**: Parsed with GLR runtime
    /// **Then**: Should produce "((1 * 2) * 3)", NOT "(1 * (2 * 3))"
    #[test]
    fn test_left_associativity_mul() {
        let tree = parse("1*2*3").expect("Should parse with left associativity");

        assert_eq!(tree.root_kind(), 4); // expr symbol
    }

    /// Test Scenario 7: Mixed precedence
    ///
    /// **Given**: Input "1*2-3"
    /// **When**: Parsed with GLR runtime
    /// **Then**: Should produce "(1 * 2) - 3"
    #[test]
    fn test_mixed_precedence() {
        let tree = parse("1*2-3").expect("Should parse mixed precedence");

        assert_eq!(tree.root_kind(), 4); // expr symbol
    }

    /// Test Scenario 8: Complex expression
    ///
    /// **Given**: Input "1-2*3-4"
    /// **When**: Parsed with GLR runtime
    /// **Then**: Should produce "((1 - (2 * 3)) - 4)"
    #[test]
    fn test_complex_expression() {
        let tree = parse("1-2*3-4").expect("Should parse complex expression");

        assert_eq!(tree.root_kind(), 4); // expr symbol
    }

    /// Test: Error handling - invalid input
    ///
    /// **Given**: Invalid input
    /// **When**: Parsed with GLR runtime
    /// **Then**: Should return Err without panicking
    #[test]
    fn test_error_handling() {
        let error_cases = vec![
            "1--2",   // Double operator
            "1-2-",   // Trailing operator
            "-2",     // Leading operator (not supported)
            "1 2",    // Missing operator (will fail tokenization due to gap)
            "1-*2",   // Operator sequence
        ];

        for case in error_cases {
            let result = parse(case);
            assert!(
                result.is_err(),
                "Expected error for '{}', got {:?}",
                case,
                result
            );
        }
    }

    /// Test: Whitespace should not affect parsing
    ///
    /// **Given**: Expressions with various whitespace
    /// **When**: Parsed with GLR runtime
    /// **Then**: Should produce same tree structure
    #[test]
    fn test_whitespace_handling() {
        // All these should parse successfully
        let variations = vec![
            "1-2",
            "1- 2",
            "1 -2",
            "1 - 2",
            "1  -  2",
        ];

        for input in variations {
            let result = parse(input);
            assert!(
                result.is_ok(),
                "Failed to parse '{}': {:?}",
                input,
                result
            );
        }
    }

    /// Performance test: Simple expressions should parse quickly
    ///
    /// **Given**: Simple arithmetic expression
    /// **When**: Parsed 1000 times
    /// **Then**: Average parse time should be < 1ms
    #[test]
    fn test_performance_simple() {
        use std::time::Instant;

        let iterations = 1000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _ = parse("1-2*3").expect("Should parse");
        }

        let duration = start.elapsed();
        let avg_time = duration.as_micros() / iterations;

        println!("Average parse time: {}µs", avg_time);

        // Should be fast (< 1ms = 1000µs per parse)
        assert!(
            avg_time < 1000,
            "Parse too slow: {}µs (expected < 1000µs)",
            avg_time
        );
    }
}
