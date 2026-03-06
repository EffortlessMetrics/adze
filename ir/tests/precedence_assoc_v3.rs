//! Comprehensive tests for precedence and associativity in adze-ir.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, PrecedenceKind, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Look up the SymbolId for a rule name in the grammar.
fn find_symbol(grammar: &Grammar, name: &str) -> SymbolId {
    grammar
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("symbol '{name}' not found in rule_names"))
}

/// Build a minimal expression grammar with a single binary operator.
fn expr_grammar_one_op(op: &str, prec: i16, assoc: Associativity) -> Grammar {
    GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token(op, op)
        .rule_with_precedence("expr", vec!["expr", op, "expr"], prec, assoc)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Build a grammar with two binary operators at different precedence levels.
fn expr_grammar_two_ops(
    op1: &str,
    prec1: i16,
    assoc1: Associativity,
    op2: &str,
    prec2: i16,
    assoc2: Associativity,
) -> Grammar {
    GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token(op1, op1)
        .token(op2, op2)
        .rule_with_precedence("expr", vec!["expr", op1, "expr"], prec1, assoc1)
        .rule_with_precedence("expr", vec!["expr", op2, "expr"], prec2, assoc2)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

// =========================================================================
// 1. Precedence values (8 tests)
// =========================================================================

#[test]
fn test_precedence_positive_value() {
    let g = expr_grammar_one_op("+", 5, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(5)));
}

#[test]
fn test_precedence_negative_value() {
    let g = expr_grammar_one_op("+", -3, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(-3)));
}

#[test]
fn test_precedence_zero_value() {
    let g = expr_grammar_one_op("+", 0, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(0)));
}

#[test]
fn test_precedence_one() {
    let g = expr_grammar_one_op("+", 1, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(1)));
}

#[test]
fn test_precedence_large_positive() {
    let g = expr_grammar_one_op("+", 1000, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(1000)));
}

#[test]
fn test_precedence_large_negative() {
    let g = expr_grammar_one_op("+", -1000, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(-1000)));
}

#[test]
fn test_rule_without_precedence_has_none() {
    let g = expr_grammar_one_op("+", 1, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let plain_rule = rules.iter().find(|r| r.precedence.is_none()).unwrap();
    assert_eq!(plain_rule.precedence, None);
    assert_eq!(plain_rule.associativity, None);
}

#[test]
fn test_precedence_value_stored_as_static_kind() {
    let g = expr_grammar_one_op("+", 42, Associativity::Right);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    match prec_rule.precedence {
        Some(PrecedenceKind::Static(v)) => assert_eq!(v, 42),
        other => panic!("expected Static(42), got {other:?}"),
    }
}

// =========================================================================
// 2. Left associativity (8 tests)
// =========================================================================

#[test]
fn test_left_assoc_single_op() {
    let g = expr_grammar_one_op("+", 1, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::Left));
}

#[test]
fn test_left_assoc_mul() {
    let g = expr_grammar_one_op("*", 2, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::Left));
}

#[test]
fn test_left_assoc_preserves_prec() {
    let g = expr_grammar_one_op("+", 7, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(7)));
    assert_eq!(prec_rule.associativity, Some(Associativity::Left));
}

#[test]
fn test_left_assoc_two_ops_same_level() {
    let g = GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rules: Vec<_> = rules
        .iter()
        .filter(|r| r.associativity == Some(Associativity::Left))
        .collect();
    assert_eq!(prec_rules.len(), 2);
}

#[test]
fn test_left_assoc_not_equal_to_right() {
    assert_ne!(Associativity::Left, Associativity::Right);
}

#[test]
fn test_left_assoc_not_equal_to_none() {
    assert_ne!(Associativity::Left, Associativity::None);
}

#[test]
fn test_left_assoc_debug_format() {
    let s = format!("{:?}", Associativity::Left);
    assert_eq!(s, "Left");
}

#[test]
fn test_left_assoc_equality() {
    let a = Associativity::Left;
    let b = Associativity::Left;
    assert_eq!(a, b);
}

// =========================================================================
// 3. Right associativity (8 tests)
// =========================================================================

#[test]
fn test_right_assoc_single_op() {
    let g = expr_grammar_one_op("=", 1, Associativity::Right);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
}

#[test]
fn test_right_assoc_preserves_prec() {
    let g = expr_grammar_one_op("=", 3, Associativity::Right);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(3)));
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
}

#[test]
fn test_right_assoc_exponent_op() {
    let g = GrammarBuilder::new("math")
        .token("NUM", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 10, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(10)));
}

#[test]
fn test_right_assoc_negative_prec() {
    let g = expr_grammar_one_op("=", -2, Associativity::Right);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(-2)));
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
}

#[test]
fn test_right_assoc_not_equal_to_left() {
    assert_ne!(Associativity::Right, Associativity::Left);
}

#[test]
fn test_right_assoc_not_equal_to_none() {
    assert_ne!(Associativity::Right, Associativity::None);
}

#[test]
fn test_right_assoc_debug_format() {
    let s = format!("{:?}", Associativity::Right);
    assert_eq!(s, "Right");
}

#[test]
fn test_right_assoc_equality() {
    let a = Associativity::Right;
    let b = Associativity::Right;
    assert_eq!(a, b);
}

// =========================================================================
// 4. No associativity (7 tests)
// =========================================================================

#[test]
fn test_none_assoc_single_op() {
    let g = expr_grammar_one_op("=", 1, Associativity::None);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::None));
}

#[test]
fn test_none_assoc_preserves_prec() {
    let g = expr_grammar_one_op("=", 5, Associativity::None);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(5)));
}

#[test]
fn test_none_assoc_comparison_op() {
    let g = GrammarBuilder::new("cmp")
        .token("NUM", r"\d+")
        .token("<", "<")
        .rule_with_precedence("expr", vec!["expr", "<", "expr"], 4, Associativity::None)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::None));
}

#[test]
fn test_none_assoc_not_equal_to_left() {
    assert_ne!(Associativity::None, Associativity::Left);
}

#[test]
fn test_none_assoc_not_equal_to_right() {
    assert_ne!(Associativity::None, Associativity::Right);
}

#[test]
fn test_none_assoc_debug_format() {
    let s = format!("{:?}", Associativity::None);
    assert_eq!(s, "None");
}

#[test]
fn test_none_assoc_equality() {
    let a = Associativity::None;
    let b = Associativity::None;
    assert_eq!(a, b);
}

// =========================================================================
// 5. Mixed precedence (8 tests)
// =========================================================================

#[test]
fn test_mixed_two_levels_different_assoc() {
    let g = expr_grammar_two_ops("+", 1, Associativity::Left, "=", 0, Associativity::Right);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rules: Vec<_> = rules.iter().filter(|r| r.precedence.is_some()).collect();
    assert_eq!(prec_rules.len(), 2);
}

#[test]
fn test_mixed_add_mul_prec_ordering() {
    let g = expr_grammar_two_ops("+", 1, Associativity::Left, "*", 2, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();

    let add_prec = rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(1)))
        .unwrap();
    let mul_prec = rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(2)))
        .unwrap();

    match (add_prec.precedence, mul_prec.precedence) {
        (Some(PrecedenceKind::Static(a)), Some(PrecedenceKind::Static(m))) => assert!(a < m),
        _ => panic!("unexpected precedence kinds"),
    }
}

#[test]
fn test_mixed_three_levels() {
    let g = GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rules: Vec<_> = rules.iter().filter(|r| r.precedence.is_some()).collect();
    assert_eq!(prec_rules.len(), 3);
}

#[test]
fn test_mixed_left_and_right_at_different_levels() {
    let g = expr_grammar_two_ops("+", 1, Associativity::Left, "=", 0, Associativity::Right);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();

    let left_rule = rules
        .iter()
        .find(|r| r.associativity == Some(Associativity::Left))
        .unwrap();
    let right_rule = rules
        .iter()
        .find(|r| r.associativity == Some(Associativity::Right))
        .unwrap();

    assert_eq!(left_rule.precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(right_rule.precedence, Some(PrecedenceKind::Static(0)));
}

#[test]
fn test_mixed_negative_and_positive_prec() {
    let g = expr_grammar_two_ops("+", -1, Associativity::Left, "*", 1, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();

    let neg = rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(-1)))
        .unwrap();
    let pos = rules
        .iter()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(1)))
        .unwrap();

    assert_eq!(neg.associativity, Some(Associativity::Left));
    assert_eq!(pos.associativity, Some(Associativity::Left));
}

#[test]
fn test_mixed_same_prec_different_ops() {
    let g = expr_grammar_two_ops("+", 1, Associativity::Left, "-", 1, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let same_prec: Vec<_> = rules
        .iter()
        .filter(|r| r.precedence == Some(PrecedenceKind::Static(1)))
        .collect();
    assert_eq!(same_prec.len(), 2);
}

#[test]
fn test_mixed_prec_with_plain_rule() {
    let g = expr_grammar_two_ops("+", 1, Associativity::Left, "*", 2, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let plain: Vec<_> = rules.iter().filter(|r| r.precedence.is_none()).collect();
    assert_eq!(plain.len(), 1); // expr -> NUM
}

#[test]
fn test_mixed_all_three_assoc_kinds() {
    let g = GrammarBuilder::new("all_assoc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("=", "=")
        .token("<", "<")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "=", "expr"], 0, Associativity::Right)
        .rule_with_precedence("expr", vec!["expr", "<", "expr"], 2, Associativity::None)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let left_count = rules
        .iter()
        .filter(|r| r.associativity == Some(Associativity::Left))
        .count();
    let right_count = rules
        .iter()
        .filter(|r| r.associativity == Some(Associativity::Right))
        .count();
    let none_count = rules
        .iter()
        .filter(|r| r.associativity == Some(Associativity::None))
        .count();

    assert_eq!(left_count, 1);
    assert_eq!(right_count, 1);
    assert_eq!(none_count, 1);
}

// =========================================================================
// 6. Precedence after normalize (8 tests)
// =========================================================================

#[test]
fn test_normalize_preserves_rule_precedence() {
    let mut g = expr_grammar_one_op("+", 5, Associativity::Left);
    let _aux = g.normalize();
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(5)));
}

#[test]
fn test_normalize_preserves_left_assoc() {
    let mut g = expr_grammar_one_op("+", 1, Associativity::Left);
    let _aux = g.normalize();
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::Left));
}

#[test]
fn test_normalize_preserves_right_assoc() {
    let mut g = expr_grammar_one_op("=", 2, Associativity::Right);
    let _aux = g.normalize();
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
}

#[test]
fn test_normalize_preserves_none_assoc() {
    let mut g = expr_grammar_one_op("<", 3, Associativity::None);
    let _aux = g.normalize();
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::None));
}

#[test]
fn test_normalize_preserves_negative_prec() {
    let mut g = expr_grammar_one_op("+", -10, Associativity::Left);
    let _aux = g.normalize();
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(-10)));
}

#[test]
fn test_normalize_preserves_grammar_level_precedences() {
    let mut g = GrammarBuilder::new("calc")
        .token("NUM", r"\d+")
        .token("+", "+")
        .precedence(1, Associativity::Left, vec!["+"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    assert_eq!(g.precedences.len(), 1);
    let _aux = g.normalize();
    assert_eq!(g.precedences.len(), 1);
    assert_eq!(g.precedences[0].level, 1);
    assert_eq!(g.precedences[0].associativity, Associativity::Left);
}

#[test]
fn test_normalize_preserves_mixed_prec_levels() {
    let mut g = expr_grammar_two_ops("+", 1, Associativity::Left, "*", 2, Associativity::Left);
    let _aux = g.normalize();
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rules: Vec<_> = rules.iter().filter(|r| r.precedence.is_some()).collect();
    assert_eq!(prec_rules.len(), 2);
}

#[test]
fn test_normalize_preserves_zero_prec() {
    let mut g = expr_grammar_one_op("+", 0, Associativity::Left);
    let _aux = g.normalize();
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(0)));
}

// =========================================================================
// 7. Edge cases (8 tests)
// =========================================================================

#[test]
fn test_edge_max_i16_precedence() {
    let g = expr_grammar_one_op("+", i16::MAX, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(i16::MAX)));
}

#[test]
fn test_edge_min_i16_precedence() {
    let g = expr_grammar_one_op("+", i16::MIN, Associativity::Left);
    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rule = rules.iter().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(i16::MIN)));
}

#[test]
fn test_edge_many_precedence_levels() {
    let mut builder = GrammarBuilder::new("many_levels").token("NUM", r"\d+");
    // Use 10 distinct precedence levels
    let ops = ["+", "-", "*", "/", "^", "=", "<", ":", ",", ";"];
    for (i, &op) in ops.iter().enumerate() {
        builder = builder.token(op, op);
        builder = builder.rule_with_precedence(
            "expr",
            vec!["expr", op, "expr"],
            i as i16,
            Associativity::Left,
        );
    }
    let g = builder.rule("expr", vec!["NUM"]).start("expr").build();

    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let prec_rules: Vec<_> = rules.iter().filter(|r| r.precedence.is_some()).collect();
    assert_eq!(prec_rules.len(), 10);
}

#[test]
fn test_edge_same_prec_different_assoc() {
    let g = GrammarBuilder::new("conflict")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("=", "=")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "=", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let rules = g.rules.get(&find_symbol(&g, "expr")).unwrap();
    let left = rules
        .iter()
        .find(|r| r.associativity == Some(Associativity::Left))
        .unwrap();
    let right = rules
        .iter()
        .find(|r| r.associativity == Some(Associativity::Right))
        .unwrap();

    // Same precedence level, different associativity
    assert_eq!(left.precedence, Some(PrecedenceKind::Static(1)));
    assert_eq!(right.precedence, Some(PrecedenceKind::Static(1)));
    assert_ne!(left.associativity, right.associativity);
}

#[test]
fn test_edge_grammar_level_multiple_precedences() {
    let g = GrammarBuilder::new("multi_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    assert_eq!(g.precedences.len(), 2);
    assert_eq!(g.precedences[0].level, 1);
    assert_eq!(g.precedences[1].level, 2);
}

#[test]
fn test_edge_grammar_prec_symbols_resolved() {
    let g = GrammarBuilder::new("prec_sym")
        .token("NUM", r"\d+")
        .token("+", "+")
        .precedence(1, Associativity::Left, vec!["+"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    assert!(!g.precedences[0].symbols.is_empty());
    // The symbol ID for "+" should be valid (non-zero since SymbolId(0) is EOF)
    assert_ne!(g.precedences[0].symbols[0], SymbolId(0));
}

#[test]
fn test_edge_empty_precedences_by_default() {
    let g = GrammarBuilder::new("simple")
        .token("NUM", r"\d+")
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    assert!(g.precedences.is_empty());
}

#[test]
fn test_edge_copy_semantics_of_associativity() {
    let a = Associativity::Left;
    let b = a; // Copy, not move
    assert_eq!(a, b); // Both usable — proves Copy
}
