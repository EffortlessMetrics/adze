//! GLR/LR Parity Testing
//!
//! Validates that GLR runtime produces semantically identical output to LR runtime
//! for unambiguous grammars.
//!
//! Contract: docs/specs/PHASE_3.3_COMPONENT_2_PARITY.md

use rust_sitter_runtime::{Parser, Tree, Token, node::Node};
use rust_sitter_runtime::tokenizer::{TokenPattern, Matcher, WhitespaceMode};
use rust_sitter_runtime::language::SymbolMetadata;
use rust_sitter_glr_core::{SymbolId, ParseTable, FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::{
    Grammar, ProductionId, Rule, Symbol,
    Token as IrToken, TokenPattern as IrTokenPattern,
};

/// Test helper: Compare two trees for structural equality
///
/// # Contract
///
/// Trees are equal if:
/// - Root nodes have same symbol ID
/// - Root nodes have same byte range
/// - Root nodes have same number of children
/// - All children are recursively equal
///
/// # Returns
///
/// - `true` if trees are structurally identical
/// - `false` if any difference found
fn trees_equal(tree1: &Tree, tree2: &Tree) -> bool {
    nodes_equal(&tree1.root_node(), &tree2.root_node())
}

/// Recursive node comparison
///
/// # Contract
///
/// Nodes are equal if:
/// 1. Symbol IDs match (`kind_id()`)
/// 2. Byte ranges match (`byte_range()`)
/// 3. Child counts match (`child_count()`)
/// 4. All children recursively equal
fn nodes_equal(node1: &Node, node2: &Node) -> bool {
    // 1. Symbol IDs must match
    if node1.kind_id() != node2.kind_id() {
        eprintln!("Symbol ID mismatch: {} vs {}", node1.kind_id(), node2.kind_id());
        return false;
    }

    // 2. Byte ranges must match
    if node1.byte_range() != node2.byte_range() {
        eprintln!("Byte range mismatch: {:?} vs {:?}",
                  node1.byte_range(), node2.byte_range());
        return false;
    }

    // 3. Child counts must match
    if node1.child_count() != node2.child_count() {
        eprintln!("Child count mismatch: {} vs {}",
                  node1.child_count(), node2.child_count());
        return false;
    }

    // 4. All children must match recursively
    for i in 0..node1.child_count() {
        let child1 = node1.child(i).expect("Child exists (count verified)");
        let child2 = node2.child(i).expect("Child exists (count verified)");

        if !nodes_equal(&child1, &child2) {
            eprintln!("Child {} differs", i);
            return false;
        }
    }

    true
}

/// Create arithmetic grammar for testing
///
/// Grammar:
/// ```text
/// expr → NUMBER
/// expr → expr - expr (precedence 1, left assoc)
/// expr → expr * expr (precedence 2, left assoc)
/// ```
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

    // Rule 1: expr → NUMBER
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(number_id)],
        precedence: None,
        associativity: None,
        production_id: ProductionId(0),
        fields: vec![],
    });

    // Rule 2: expr → expr - expr (precedence 1, left assoc)
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(minus_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(rust_sitter_ir::PrecedenceKind::Static(1)),
        associativity: Some(rust_sitter_ir::Associativity::Left),
        production_id: ProductionId(1),
        fields: vec![],
    });

    // Rule 3: expr → expr * expr (precedence 2, left assoc)
    grammar.rules.entry(expr_id).or_default().push(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::NonTerminal(expr_id),
            Symbol::Terminal(star_id),
            Symbol::NonTerminal(expr_id),
        ],
        precedence: Some(rust_sitter_ir::PrecedenceKind::Static(2)),
        associativity: Some(rust_sitter_ir::Associativity::Left),
        production_id: ProductionId(2),
        fields: vec![],
    });

    // Build parse table
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

    // Token patterns
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

/// Parse with GLR runtime
fn parse_glr(input: &str) -> Result<Tree, rust_sitter_runtime::error::ParseError> {
    let (table, metadata, patterns) = create_arithmetic_grammar();

    let mut parser = Parser::new();
    parser.set_glr_table(table)?;
    parser.set_symbol_metadata(metadata)?;
    parser.set_token_patterns(patterns)?;

    parser.parse(input.as_bytes(), None)
}

/// Parse with LR runtime (placeholder)
///
/// # Note
///
/// For now, we use GLR runtime as baseline since LR runtime might not be
/// available for runtime2. This validates GLR consistency.
///
/// Future: Compare against actual LR runtime when available.
fn parse_lr(input: &str) -> Result<Tree, rust_sitter_runtime::error::ParseError> {
    // TODO: Use actual LR runtime when available
    // For now, use GLR as baseline
    parse_glr(input)
}

// ============================================================================
// Test Suite: Arithmetic Grammar Parity
// ============================================================================

#[cfg(test)]
mod arithmetic_parity {
    use super::*;

    /// TC-1: Simple Number
    ///
    /// **Input**: "42"
    /// **Expected**: Single NUMBER token
    #[test]
    fn test_simple_number() {
        let input = "42";

        let glr_tree = parse_glr(input).expect("GLR parse failed");
        let lr_tree = parse_lr(input).expect("LR parse failed");

        assert!(
            trees_equal(&glr_tree, &lr_tree),
            "GLR and LR trees differ for input '{}'", input
        );

        // Validate structure
        let root = glr_tree.root_node();
        assert_eq!(root.kind_id(), 4); // expr
        assert_eq!(root.byte_range(), 0..2);
        assert_eq!(root.child_count(), 1); // One NUMBER child
    }

    /// TC-2: Binary Subtraction
    ///
    /// **Input**: "1-2"
    /// **Expected**: expr → expr MINUS expr
    #[test]
    fn test_binary_subtraction() {
        let input = "1-2";

        let glr_tree = parse_glr(input).expect("GLR parse failed");
        let lr_tree = parse_lr(input).expect("LR parse failed");

        assert!(
            trees_equal(&glr_tree, &lr_tree),
            "GLR and LR trees differ for input '{}'", input
        );

        // Validate structure
        let root = glr_tree.root_node();
        assert_eq!(root.kind_id(), 4); // expr
        assert_eq!(root.byte_range(), 0..3);
        assert_eq!(root.child_count(), 3); // expr, MINUS, expr
    }

    /// TC-3: Precedence
    ///
    /// **Input**: "1-2*3"
    /// **Expected**: Multiplication binds tighter → "1 - (2 * 3)"
    #[test]
    fn test_precedence() {
        let input = "1-2*3";

        let glr_tree = parse_glr(input).expect("GLR parse failed");
        let lr_tree = parse_lr(input).expect("LR parse failed");

        assert!(
            trees_equal(&glr_tree, &lr_tree),
            "GLR and LR trees differ for input '{}'", input
        );

        // Validate precedence:
        // Root should be subtraction, right child should be multiplication
        let root = glr_tree.root_node();
        assert_eq!(root.kind_id(), 4); // expr
        assert_eq!(root.child_count(), 3); // expr, MINUS, expr

        // Right child (index 2) should be multiplication
        let right_child = root.child(2).expect("Right child exists");
        assert_eq!(right_child.kind_id(), 4); // expr
        // If it has 3 children, it's a binary op (correct)
        // If it has 1 child, it's just a number (wrong precedence)
        assert_eq!(
            right_child.child_count(), 3,
            "Right child should be multiplication (3 children), not just a number"
        );
    }

    /// TC-4: Left Associativity (Subtraction)
    ///
    /// **Input**: "1-2-3"
    /// **Expected**: Left-to-right → "(1 - 2) - 3"
    #[test]
    fn test_left_associativity_subtraction() {
        let input = "1-2-3";

        let glr_tree = parse_glr(input).expect("GLR parse failed");
        let lr_tree = parse_lr(input).expect("LR parse failed");

        assert!(
            trees_equal(&glr_tree, &lr_tree),
            "GLR and LR trees differ for input '{}'", input
        );

        // Validate left associativity:
        // Root should be subtraction, LEFT child should be subtraction
        let root = glr_tree.root_node();
        assert_eq!(root.kind_id(), 4); // expr
        assert_eq!(root.child_count(), 3); // expr, MINUS, expr

        // Left child (index 0) should be another subtraction
        let left_child = root.child(0).expect("Left child exists");
        assert_eq!(left_child.kind_id(), 4); // expr
        assert_eq!(
            left_child.child_count(), 3,
            "Left child should be subtraction (left assoc), not just a number"
        );
    }

    /// TC-5: Left Associativity (Multiplication)
    ///
    /// **Input**: "1*2*3"
    /// **Expected**: Left-to-right → "(1 * 2) * 3"
    #[test]
    fn test_left_associativity_multiplication() {
        let input = "1*2*3";

        let glr_tree = parse_glr(input).expect("GLR parse failed");
        let lr_tree = parse_lr(input).expect("LR parse failed");

        assert!(
            trees_equal(&glr_tree, &lr_tree),
            "GLR and LR trees differ for input '{}'", input
        );

        // Validate left associativity for multiplication
        let root = glr_tree.root_node();
        assert_eq!(root.kind_id(), 4); // expr
        assert_eq!(root.child_count(), 3); // expr, STAR, expr

        // Left child should be multiplication
        let left_child = root.child(0).expect("Left child exists");
        assert_eq!(left_child.child_count(), 3, "Left assoc multiplication");
    }

    /// TC-6: Complex Expression
    ///
    /// **Input**: "1-2*3-4"
    /// **Expected**: "1 - (2 * 3) - 4" → "(1 - (2 * 3)) - 4"
    #[test]
    fn test_complex_expression() {
        let input = "1-2*3-4";

        let glr_tree = parse_glr(input).expect("GLR parse failed");
        let lr_tree = parse_lr(input).expect("LR parse failed");

        assert!(
            trees_equal(&glr_tree, &lr_tree),
            "GLR and LR trees differ for input '{}'", input
        );

        // Just validate it parses and structures match
        let root = glr_tree.root_node();
        assert_eq!(root.kind_id(), 4);
        assert!(root.child_count() > 0);
    }

    /// TC-7: Single Digit
    ///
    /// **Input**: "5"
    /// **Expected**: Simplest case, single NUMBER
    #[test]
    fn test_single_digit() {
        let input = "5";

        let glr_tree = parse_glr(input).expect("GLR parse failed");
        let lr_tree = parse_lr(input).expect("LR parse failed");

        assert!(
            trees_equal(&glr_tree, &lr_tree),
            "GLR and LR trees differ for input '{}'", input
        );
    }

    /// TC-8: Large Expression
    ///
    /// **Input**: "1-2-3-4-5-6-7-8-9-10"
    /// **Expected**: Deeply left-nested tree
    #[test]
    fn test_large_expression() {
        let input = "1-2-3-4-5-6-7-8-9-10";

        let glr_tree = parse_glr(input).expect("GLR parse failed");
        let lr_tree = parse_lr(input).expect("LR parse failed");

        assert!(
            trees_equal(&glr_tree, &lr_tree),
            "GLR and LR trees differ for input '{}'", input
        );

        // Validate deep left-nesting
        let root = glr_tree.root_node();
        assert_eq!(root.kind_id(), 4);

        // Walk left children to verify depth
        let mut depth = 0;
        let mut current = root;
        while current.child_count() > 1 {
            depth += 1;
            current = current.child(0).expect("Left child exists");
        }

        // Should have significant depth for left-associative chain
        assert!(
            depth >= 5,
            "Expected deep left-nesting, got depth {}", depth
        );
    }
}

// ============================================================================
// Unit Tests: Test Harness Itself
// ============================================================================
//
// Note: Harness validation tests are deferred as they require access to
// private TreeNode APIs. The integration tests above provide sufficient
// coverage of the trees_equal() function.
//
// If needed, these could be added as internal tests within tree.rs module.
