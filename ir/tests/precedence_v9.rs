//! Comprehensive tests for precedence handling in adze-ir grammars.
//!
//! Categories (10):
//!   1. basic_prec_*     — basic precedence values (0, positive, negative)
//!   2. assoc_*          — associativity (Left, Right, None)
//!   3. multi_level_*    — multiple precedence levels in one grammar
//!   4. inline_prec_*    — precedence with inline rules
//!   5. supertype_prec_* — precedence with supertypes
//!   6. edge_*           — edge cases (same prec different assoc, min/max i16)
//!   7. default_prec_*   — rules without precedence (default None)
//!   8. ordering_*       — precedence ordering validation
//!   9. many_levels_*    — grammars with many (10+) precedence levels
//!  10. conflict_prec_*  — interaction between precedence and conflicts

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, Grammar, Precedence, PrecedenceKind,
    Symbol, SymbolId,
};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn build_grammar_with_precedence() -> Grammar {
    GrammarBuilder::new("prec_test")
        .token("NUMBER", r"\d+")
        .token("PLUS", r"\+")
        .token("TIMES", r"\*")
        .token("MINUS", r"-")
        .token("DIVIDE", r"\/")
        .rule("expression", vec!["term"])
        .rule_with_precedence(
            "expression",
            vec!["expression", "PLUS", "expression"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence(
            "expression",
            vec!["expression", "MINUS", "expression"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence(
            "expression",
            vec!["expression", "TIMES", "expression"],
            2,
            Associativity::Left,
        )
        .rule_with_precedence(
            "expression",
            vec!["expression", "DIVIDE", "expression"],
            2,
            Associativity::Left,
        )
        .rule("term", vec!["NUMBER"])
        .start("expression")
        .build()
}

fn build_right_assoc_grammar() -> Grammar {
    GrammarBuilder::new("right_assoc")
        .token("ID", r"[a-z]+")
        .token("EQ", "=")
        .token("NUM", r"\d+")
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn build_none_assoc_grammar() -> Grammar {
    GrammarBuilder::new("none_assoc")
        .token("NUM", r"\d+")
        .token("LT", "<")
        .token("GT", ">")
        .rule_with_precedence("expr", vec!["expr", "LT", "expr"], 1, Associativity::None)
        .rule_with_precedence("expr", vec!["expr", "GT", "expr"], 1, Associativity::None)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn build_mixed_assoc_grammar() -> Grammar {
    GrammarBuilder::new("mixed")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("EQ", "=")
        .token("LT", "<")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "EQ", "expr"], 1, Associativity::Right)
        .rule_with_precedence("expr", vec!["expr", "LT", "expr"], 3, Associativity::None)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. basic_prec — basic precedence values
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn basic_prec_positive_value_stored() {
    let g = build_grammar_with_precedence();
    let rules_with_prec: Vec<_> = g
        .all_rules()
        .filter(|r| r.precedence == Some(PrecedenceKind::Static(1)))
        .collect();
    assert_eq!(rules_with_prec.len(), 2);
}

#[test]
fn basic_prec_higher_positive_value() {
    let g = build_grammar_with_precedence();
    let rules_with_prec: Vec<_> = g
        .all_rules()
        .filter(|r| r.precedence == Some(PrecedenceKind::Static(2)))
        .collect();
    assert_eq!(rules_with_prec.len(), 2);
}

#[test]
fn basic_prec_zero_value() {
    let g = GrammarBuilder::new("zero_prec")
        .token("A", "a")
        .token("B", "b")
        .rule_with_precedence("s", vec!["A", "B"], 0, Associativity::Left)
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(0)));
}

#[test]
fn basic_prec_negative_value() {
    let g = GrammarBuilder::new("neg_prec")
        .token("A", "a")
        .token("B", "b")
        .rule_with_precedence("s", vec!["A", "B"], -5, Associativity::Left)
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(-5)));
}

#[test]
fn basic_prec_static_kind_discriminant() {
    let pk = PrecedenceKind::Static(42);
    match pk {
        PrecedenceKind::Static(v) => assert_eq!(v, 42),
        _ => panic!("expected Static"),
    }
}

#[test]
fn basic_prec_dynamic_kind_discriminant() {
    let pk = PrecedenceKind::Dynamic(7);
    match pk {
        PrecedenceKind::Dynamic(v) => assert_eq!(v, 7),
        _ => panic!("expected Dynamic"),
    }
}

#[test]
fn basic_prec_static_ne_dynamic_same_value() {
    assert_ne!(PrecedenceKind::Static(3), PrecedenceKind::Dynamic(3));
}

#[test]
fn basic_prec_copy_semantics() {
    let a = PrecedenceKind::Static(10);
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn basic_prec_dynamic_negative() {
    let pk = PrecedenceKind::Dynamic(-100);
    match pk {
        PrecedenceKind::Dynamic(v) => assert_eq!(v, -100),
        _ => panic!("expected Dynamic"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. assoc — associativity (Left, Right, None)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn assoc_left_equality() {
    assert_eq!(Associativity::Left, Associativity::Left);
}

#[test]
fn assoc_right_equality() {
    assert_eq!(Associativity::Right, Associativity::Right);
}

#[test]
fn assoc_none_equality() {
    assert_eq!(Associativity::None, Associativity::None);
}

#[test]
fn assoc_left_ne_right() {
    assert_ne!(Associativity::Left, Associativity::Right);
}

#[test]
fn assoc_left_ne_none() {
    assert_ne!(Associativity::Left, Associativity::None);
}

#[test]
fn assoc_right_ne_none() {
    assert_ne!(Associativity::Right, Associativity::None);
}

#[test]
fn assoc_left_copy_semantics() {
    let a = Associativity::Left;
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn assoc_left_rules_in_arith_grammar() {
    let g = build_grammar_with_precedence();
    let left_count = g
        .all_rules()
        .filter(|r| r.associativity == Some(Associativity::Left))
        .count();
    assert_eq!(left_count, 4);
}

#[test]
fn assoc_right_rules_in_assign_grammar() {
    let g = build_right_assoc_grammar();
    let right_count = g
        .all_rules()
        .filter(|r| r.associativity == Some(Associativity::Right))
        .count();
    assert_eq!(right_count, 1);
}

#[test]
fn assoc_none_rules_in_comparison_grammar() {
    let g = build_none_assoc_grammar();
    let none_count = g
        .all_rules()
        .filter(|r| r.associativity == Some(Associativity::None))
        .count();
    assert_eq!(none_count, 2);
}

#[test]
fn assoc_right_debug_contains_right() {
    let dbg = format!("{:?}", Associativity::Right);
    assert!(dbg.contains("Right"));
}

#[test]
fn assoc_none_debug_contains_none() {
    let dbg = format!("{:?}", Associativity::None);
    assert!(dbg.contains("None"));
}

#[test]
fn assoc_rule_without_assoc_is_none() {
    let g = build_grammar_with_precedence();
    let no_assoc: Vec<_> = g
        .all_rules()
        .filter(|r| r.associativity.is_none())
        .collect();
    // "expression -> term" and "term -> NUMBER" have no associativity
    assert_eq!(no_assoc.len(), 2);
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. multi_level — multiple precedence levels in one grammar
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn multi_level_two_distinct_levels() {
    let g = build_grammar_with_precedence();
    let mut levels: Vec<i16> = g
        .all_rules()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => Option::None,
        })
        .collect();
    levels.sort();
    levels.dedup();
    assert_eq!(levels, vec![1, 2]);
}

#[test]
fn multi_level_three_levels() {
    let g = build_mixed_assoc_grammar();
    let mut levels: Vec<i16> = g
        .all_rules()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => Option::None,
        })
        .collect();
    levels.sort();
    levels.dedup();
    assert_eq!(levels, vec![1, 2, 3]);
}

#[test]
fn multi_level_each_level_has_correct_assoc() {
    let g = build_mixed_assoc_grammar();
    for rule in g.all_rules() {
        match rule.precedence {
            Some(PrecedenceKind::Static(1)) => {
                assert_eq!(rule.associativity, Some(Associativity::Right));
            }
            Some(PrecedenceKind::Static(2)) => {
                assert_eq!(rule.associativity, Some(Associativity::Left));
            }
            Some(PrecedenceKind::Static(3)) => {
                assert_eq!(rule.associativity, Some(Associativity::None));
            }
            _ => {}
        }
    }
}

#[test]
fn multi_level_total_rules_with_prec() {
    let g = build_mixed_assoc_grammar();
    let count = g.all_rules().filter(|r| r.precedence.is_some()).count();
    assert_eq!(count, 3);
}

#[test]
fn multi_level_rules_without_prec() {
    let g = build_mixed_assoc_grammar();
    let count = g.all_rules().filter(|r| r.precedence.is_none()).count();
    assert_eq!(count, 1); // expr -> NUM
}

#[test]
fn multi_level_rhs_length_for_binary_ops() {
    let g = build_mixed_assoc_grammar();
    for rule in g.all_rules() {
        if rule.precedence.is_some() {
            assert_eq!(rule.rhs.len(), 3);
        }
    }
}

#[test]
fn multi_level_five_levels() {
    let g = GrammarBuilder::new("five")
        .token("N", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .rule_with_precedence("s", vec!["s", "A", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], 2, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "C", "s"], 3, Associativity::Right)
        .rule_with_precedence("s", vec!["s", "D", "s"], 4, Associativity::None)
        .rule_with_precedence("s", vec!["s", "E", "s"], 5, Associativity::Left)
        .rule("s", vec!["N"])
        .start("s")
        .build();
    let mut levels: Vec<i16> = g
        .all_rules()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => Option::None,
        })
        .collect();
    levels.sort();
    levels.dedup();
    assert_eq!(levels, vec![1, 2, 3, 4, 5]);
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. inline_prec — precedence with inline rules
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn inline_prec_inline_rule_still_has_prec() {
    let g = GrammarBuilder::new("inline_prec")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["atom"])
        .rule("atom", vec!["NUM"])
        .inline("atom")
        .start("expr")
        .build();
    assert!(!g.inline_rules.is_empty());
    let prec_count = g.all_rules().filter(|r| r.precedence.is_some()).count();
    assert_eq!(prec_count, 1);
}

#[test]
fn inline_prec_inline_symbol_in_inline_list() {
    let g = GrammarBuilder::new("inline_test")
        .token("A", "a")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["A"])
        .inline("helper")
        .start("s")
        .build();
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn inline_prec_multiple_inlines_with_prec() {
    let g = GrammarBuilder::new("multi_inline")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .token("TIMES", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "TIMES", "expr"],
            2,
            Associativity::Left,
        )
        .rule("expr", vec!["atom"])
        .rule("atom", vec!["N"])
        .rule("wrapper", vec!["atom"])
        .inline("atom")
        .inline("wrapper")
        .start("expr")
        .build();
    assert_eq!(g.inline_rules.len(), 2);
    let prec_count = g.all_rules().filter(|r| r.precedence.is_some()).count();
    assert_eq!(prec_count, 2);
}

#[test]
fn inline_prec_precedence_values_preserved_with_inline() {
    let g = GrammarBuilder::new("inline_vals")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("e", vec!["e", "PLUS", "e"], 5, Associativity::Left)
        .rule("e", vec!["base"])
        .rule("base", vec!["N"])
        .inline("base")
        .start("e")
        .build();
    let prec_rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(5)));
}

#[test]
fn inline_prec_inline_does_not_affect_assoc() {
    let g = GrammarBuilder::new("inline_assoc")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("e", vec!["e", "PLUS", "e"], 1, Associativity::Right)
        .rule("e", vec!["base"])
        .rule("base", vec!["N"])
        .inline("base")
        .start("e")
        .build();
    let prec_rule = g.all_rules().find(|r| r.associativity.is_some()).unwrap();
    assert_eq!(prec_rule.associativity, Some(Associativity::Right));
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. supertype_prec — precedence with supertypes
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn supertype_prec_supertype_recorded() {
    let g = GrammarBuilder::new("super")
        .token("NUM", r"\d+")
        .token("STR", r#""[^"]*""#)
        .token("PLUS", r"\+")
        .rule("literal", vec!["NUM"])
        .rule("literal", vec!["STR"])
        .supertype("literal")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["literal"])
        .start("expr")
        .build();
    assert!(!g.supertypes.is_empty());
}

#[test]
fn supertype_prec_prec_rules_independent_of_supertype() {
    let g = GrammarBuilder::new("super_prec")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("value", vec!["NUM"])
        .supertype("value")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 3, Associativity::Left)
        .rule("expr", vec!["value"])
        .start("expr")
        .build();
    let prec_rule = g.all_rules().find(|r| r.precedence.is_some()).unwrap();
    assert_eq!(prec_rule.precedence, Some(PrecedenceKind::Static(3)));
}

#[test]
fn supertype_prec_supertype_rule_has_no_prec() {
    let g = GrammarBuilder::new("super_no_prec")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .rule("value", vec!["NUM"])
        .supertype("value")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["value"])
        .start("expr")
        .build();
    let value_id = g.find_symbol_by_name("value");
    assert!(value_id.is_some());
    if let Some(vid) = value_id
        && let Some(rules) = g.get_rules_for_symbol(vid)
    {
        for rule in rules {
            assert!(rule.precedence.is_none());
        }
    }
}

#[test]
fn supertype_prec_combined_with_multiple_levels() {
    let g = GrammarBuilder::new("super_multi")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("TIMES", r"\*")
        .rule("value", vec!["NUM"])
        .supertype("value")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "TIMES", "expr"],
            2,
            Associativity::Left,
        )
        .rule("expr", vec!["value"])
        .start("expr")
        .build();
    assert_eq!(g.supertypes.len(), 1);
    let prec_count = g.all_rules().filter(|r| r.precedence.is_some()).count();
    assert_eq!(prec_count, 2);
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. edge — edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn edge_same_prec_different_assoc_left_and_right() {
    let g = GrammarBuilder::new("same_prec")
        .token("N", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .rule_with_precedence("s", vec!["s", "A", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], 1, Associativity::Right)
        .rule("s", vec!["N"])
        .start("s")
        .build();
    let left = g
        .all_rules()
        .find(|r| r.associativity == Some(Associativity::Left))
        .unwrap();
    let right = g
        .all_rules()
        .find(|r| r.associativity == Some(Associativity::Right))
        .unwrap();
    assert_eq!(left.precedence, right.precedence);
}

#[test]
fn edge_same_prec_different_assoc_left_and_none() {
    let g = GrammarBuilder::new("left_none")
        .token("N", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .rule_with_precedence("s", vec!["s", "A", "s"], 2, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], 2, Associativity::None)
        .rule("s", vec!["N"])
        .start("s")
        .build();
    let left = g
        .all_rules()
        .find(|r| r.associativity == Some(Associativity::Left))
        .unwrap();
    let none = g
        .all_rules()
        .find(|r| r.associativity == Some(Associativity::None))
        .unwrap();
    assert_eq!(left.precedence, none.precedence);
    assert_ne!(left.associativity, none.associativity);
}

#[test]
fn edge_max_i16_precedence() {
    let g = GrammarBuilder::new("max_prec")
        .token("A", "a")
        .rule_with_precedence("s", vec!["A"], i16::MAX, Associativity::Left)
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(i16::MAX)));
}

#[test]
fn edge_min_i16_precedence() {
    let g = GrammarBuilder::new("min_prec")
        .token("A", "a")
        .rule_with_precedence("s", vec!["A"], i16::MIN, Associativity::Left)
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(i16::MIN)));
}

#[test]
fn edge_adjacent_prec_values() {
    let g = GrammarBuilder::new("adjacent")
        .token("N", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .rule_with_precedence("s", vec!["s", "A", "s"], -1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], 0, Associativity::Left)
        .rule("s", vec!["N"])
        .start("s")
        .build();
    let neg = g
        .all_rules()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(-1)))
        .unwrap();
    let zero = g
        .all_rules()
        .find(|r| r.precedence == Some(PrecedenceKind::Static(0)))
        .unwrap();
    assert_ne!(neg.precedence, zero.precedence);
}

#[test]
fn edge_single_rule_single_token_with_prec() {
    let g = GrammarBuilder::new("single")
        .token("X", "x")
        .rule_with_precedence("s", vec!["X"], 99, Associativity::Right)
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 1);
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(99)));
    assert_eq!(rule.associativity, Some(Associativity::Right));
}

#[test]
fn edge_all_three_assoc_at_same_prec_level() {
    let g = GrammarBuilder::new("three_assoc")
        .token("N", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule_with_precedence("s", vec!["s", "A", "s"], 5, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], 5, Associativity::Right)
        .rule_with_precedence("s", vec!["s", "C", "s"], 5, Associativity::None)
        .rule("s", vec!["N"])
        .start("s")
        .build();
    let assocs: Vec<_> = g.all_rules().filter_map(|r| r.associativity).collect();
    assert!(assocs.contains(&Associativity::Left));
    assert!(assocs.contains(&Associativity::Right));
    assert!(assocs.contains(&Associativity::None));
}

#[test]
fn edge_negative_prec_ordering() {
    let g = GrammarBuilder::new("neg_order")
        .token("N", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .rule_with_precedence("s", vec!["s", "A", "s"], -10, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], -5, Associativity::Left)
        .rule("s", vec!["N"])
        .start("s")
        .build();
    let mut precs: Vec<i16> = g
        .all_rules()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => Option::None,
        })
        .collect();
    precs.sort();
    assert_eq!(precs, vec![-10, -5]);
}

#[test]
fn edge_precedence_kind_debug_format() {
    let s = format!("{:?}", PrecedenceKind::Static(42));
    assert!(s.contains("Static"));
    assert!(s.contains("42"));
    let d = format!("{:?}", PrecedenceKind::Dynamic(-1));
    assert!(d.contains("Dynamic"));
}

#[test]
fn edge_dynamic_prec_zero() {
    let pk = PrecedenceKind::Dynamic(0);
    match pk {
        PrecedenceKind::Dynamic(v) => assert_eq!(v, 0),
        _ => panic!("expected Dynamic"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. default_prec — rules without precedence
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn default_prec_rule_has_none_prec() {
    let g = GrammarBuilder::new("no_prec")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(rule.precedence.is_none());
}

#[test]
fn default_prec_rule_has_none_assoc() {
    let g = GrammarBuilder::new("no_assoc")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(rule.associativity.is_none());
}

#[test]
fn default_prec_multiple_rules_all_none() {
    let g = GrammarBuilder::new("all_none")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A"])
        .rule("s", vec!["B"])
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    for rule in g.all_rules() {
        assert!(rule.precedence.is_none());
        assert!(rule.associativity.is_none());
    }
}

#[test]
fn default_prec_mixed_some_have_prec_some_dont() {
    let g = build_grammar_with_precedence();
    let with = g.all_rules().filter(|r| r.precedence.is_some()).count();
    let without = g.all_rules().filter(|r| r.precedence.is_none()).count();
    assert_eq!(with, 4);
    assert_eq!(without, 2);
}

#[test]
fn default_prec_epsilon_rule_has_no_prec() {
    let g = GrammarBuilder::new("eps")
        .rule("s", vec![])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert!(rule.precedence.is_none());
    assert!(rule.associativity.is_none());
}

#[test]
fn default_prec_rhs_symbols_preserved() {
    let g = GrammarBuilder::new("rhs_check")
        .token("A", "a")
        .token("B", "b")
        .rule("s", vec!["A", "B"])
        .start("s")
        .build();
    let rule = g.all_rules().next().unwrap();
    assert_eq!(rule.rhs.len(), 2);
    assert!(rule.precedence.is_none());
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. ordering — precedence ordering validation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ordering_rules_maintain_insertion_order() {
    let g = build_grammar_with_precedence();
    let expr_id = g.find_symbol_by_name("expression").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    // First rule added: expression -> term (no prec)
    assert!(rules[0].precedence.is_none());
    // Second: expression -> expression PLUS expression (prec 1)
    assert_eq!(rules[1].precedence, Some(PrecedenceKind::Static(1)));
}

#[test]
fn ordering_higher_prec_value_later_in_rules() {
    let g = build_grammar_with_precedence();
    let expr_id = g.find_symbol_by_name("expression").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    // prec 1 rules come before prec 2 rules (insertion order)
    let prec_1_idx = rules
        .iter()
        .position(|r| r.precedence == Some(PrecedenceKind::Static(1)))
        .unwrap();
    let prec_2_idx = rules
        .iter()
        .position(|r| r.precedence == Some(PrecedenceKind::Static(2)))
        .unwrap();
    assert!(prec_1_idx < prec_2_idx);
}

#[test]
fn ordering_start_symbol_rules_first() {
    let g = build_grammar_with_precedence();
    let start = g.start_symbol();
    assert!(start.is_some());
    let first_lhs = g.all_rules().next().unwrap().lhs;
    // Start symbol rules appear first in rule iteration
    let expr_id = g.find_symbol_by_name("expression").unwrap();
    assert_eq!(first_lhs, expr_id);
}

#[test]
fn ordering_all_rules_count() {
    let g = build_grammar_with_precedence();
    // 5 expression rules + 1 term rule
    assert_eq!(g.all_rules().count(), 6);
}

#[test]
fn ordering_prec_levels_can_be_collected_and_sorted() {
    let g = build_grammar_with_precedence();
    let mut levels: Vec<i16> = g
        .all_rules()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => Option::None,
        })
        .collect();
    levels.sort();
    // 1, 1, 2, 2 -> sorted dedup: 1, 2
    levels.dedup();
    assert_eq!(levels.len(), 2);
    assert!(levels[0] < levels[1]);
}

#[test]
fn ordering_precedence_decl_via_builder() {
    let g = GrammarBuilder::new("decl")
        .token("PLUS", r"\+")
        .token("TIMES", r"\*")
        .token("N", r"\d+")
        .precedence(1, Associativity::Left, vec!["PLUS"])
        .precedence(2, Associativity::Left, vec!["TIMES"])
        .rule("e", vec!["N"])
        .start("e")
        .build();
    assert_eq!(g.precedences.len(), 2);
    assert_eq!(g.precedences[0].level, 1);
    assert_eq!(g.precedences[1].level, 2);
}

#[test]
fn ordering_precedence_decl_symbols_stored() {
    let g = GrammarBuilder::new("decl_syms")
        .token("PLUS", r"\+")
        .token("MINUS", r"-")
        .precedence(1, Associativity::Left, vec!["PLUS", "MINUS"])
        .rule("e", vec!["PLUS"])
        .start("e")
        .build();
    assert_eq!(g.precedences[0].symbols.len(), 2);
}

#[test]
fn ordering_precedence_decl_assoc_stored() {
    let g = GrammarBuilder::new("decl_assoc")
        .token("A", "a")
        .precedence(5, Associativity::Right, vec!["A"])
        .rule("s", vec!["A"])
        .start("s")
        .build();
    assert_eq!(g.precedences[0].associativity, Associativity::Right);
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. many_levels — grammars with many precedence levels (10+)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn many_levels_ten_distinct_levels() {
    let g = GrammarBuilder::new("ten")
        .token("N", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .token("F", "f")
        .token("G", "g")
        .token("H", "h")
        .token("I", "i")
        .token("J", "j")
        .rule_with_precedence("s", vec!["s", "A", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], 2, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "C", "s"], 3, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "D", "s"], 4, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "E", "s"], 5, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "F", "s"], 6, Associativity::Right)
        .rule_with_precedence("s", vec!["s", "G", "s"], 7, Associativity::Right)
        .rule_with_precedence("s", vec!["s", "H", "s"], 8, Associativity::None)
        .rule_with_precedence("s", vec!["s", "I", "s"], 9, Associativity::None)
        .rule_with_precedence("s", vec!["s", "J", "s"], 10, Associativity::Left)
        .rule("s", vec!["N"])
        .start("s")
        .build();
    let mut levels: Vec<i16> = g
        .all_rules()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => Option::None,
        })
        .collect();
    levels.sort();
    levels.dedup();
    assert_eq!(levels.len(), 10);
    assert_eq!(levels, (1..=10).collect::<Vec<_>>());
}

#[test]
fn many_levels_total_rule_count() {
    let g = GrammarBuilder::new("count")
        .token("N", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .token("F", "f")
        .token("G", "g")
        .token("H", "h")
        .token("I", "i")
        .token("J", "j")
        .rule_with_precedence("s", vec!["s", "A", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], 2, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "C", "s"], 3, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "D", "s"], 4, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "E", "s"], 5, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "F", "s"], 6, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "G", "s"], 7, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "H", "s"], 8, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "I", "s"], 9, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "J", "s"], 10, Associativity::Left)
        .rule("s", vec!["N"])
        .start("s")
        .build();
    assert_eq!(g.all_rules().count(), 11);
}

#[test]
fn many_levels_negative_range() {
    let g = GrammarBuilder::new("negrange")
        .token("N", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .token("F", "f")
        .token("G", "g")
        .token("H", "h")
        .token("I", "i")
        .token("J", "j")
        .token("K", "k")
        .rule_with_precedence("s", vec!["s", "A", "s"], -5, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], -4, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "C", "s"], -3, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "D", "s"], -2, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "E", "s"], -1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "F", "s"], 0, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "G", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "H", "s"], 2, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "I", "s"], 3, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "J", "s"], 4, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "K", "s"], 5, Associativity::Left)
        .rule("s", vec!["N"])
        .start("s")
        .build();
    let mut levels: Vec<i16> = g
        .all_rules()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => Option::None,
        })
        .collect();
    levels.sort();
    assert_eq!(levels, (-5..=5).collect::<Vec<_>>());
}

#[test]
fn many_levels_mixed_assoc_across_levels() {
    let g = GrammarBuilder::new("mixed_many")
        .token("N", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .token("E", "e")
        .token("F", "f")
        .token("G", "g")
        .token("H", "h")
        .token("I", "i")
        .token("J", "j")
        .rule_with_precedence("s", vec!["s", "A", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], 2, Associativity::Right)
        .rule_with_precedence("s", vec!["s", "C", "s"], 3, Associativity::None)
        .rule_with_precedence("s", vec!["s", "D", "s"], 4, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "E", "s"], 5, Associativity::Right)
        .rule_with_precedence("s", vec!["s", "F", "s"], 6, Associativity::None)
        .rule_with_precedence("s", vec!["s", "G", "s"], 7, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "H", "s"], 8, Associativity::Right)
        .rule_with_precedence("s", vec!["s", "I", "s"], 9, Associativity::None)
        .rule_with_precedence("s", vec!["s", "J", "s"], 10, Associativity::Left)
        .rule("s", vec!["N"])
        .start("s")
        .build();
    let left_count = g
        .all_rules()
        .filter(|r| r.associativity == Some(Associativity::Left))
        .count();
    let right_count = g
        .all_rules()
        .filter(|r| r.associativity == Some(Associativity::Right))
        .count();
    let none_count = g
        .all_rules()
        .filter(|r| r.associativity == Some(Associativity::None))
        .count();
    assert_eq!(left_count, 4);
    assert_eq!(right_count, 3);
    assert_eq!(none_count, 3);
}

#[test]
fn many_levels_each_rule_has_unique_production_id() {
    let g = GrammarBuilder::new("unique_prod")
        .token("N", r"\d+")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule_with_precedence("s", vec!["s", "A", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "B", "s"], 2, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "C", "s"], 3, Associativity::Left)
        .rule("s", vec!["N"])
        .start("s")
        .build();
    let prod_ids: Vec<_> = g.all_rules().map(|r| r.production_id).collect();
    let mut unique = prod_ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(prod_ids.len(), unique.len());
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. conflict_prec — interaction between precedence and conflicts
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn conflict_prec_grammar_with_conflict_decl() {
    let mut g = GrammarBuilder::new("conflict_grammar")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("e", vec!["e", "PLUS", "e"], 1, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let e_id = g.find_symbol_by_name("e").unwrap();
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![e_id, e_id],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(1)),
    });
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn conflict_prec_conflict_resolution_by_assoc() {
    let mut g = GrammarBuilder::new("conflict_assoc")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("e", vec!["e", "PLUS", "e"], 1, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let e_id = g.find_symbol_by_name("e").unwrap();
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![e_id],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    });
    match &g.conflicts[0].resolution {
        ConflictResolution::Associativity(a) => assert_eq!(*a, Associativity::Left),
        _ => panic!("expected Associativity resolution"),
    }
}

#[test]
fn conflict_prec_conflict_resolution_glr() {
    let mut g = GrammarBuilder::new("conflict_glr")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .rule_with_precedence("e", vec!["e", "PLUS", "e"], 1, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let e_id = g.find_symbol_by_name("e").unwrap();
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![e_id],
        resolution: ConflictResolution::GLR,
    });
    match &g.conflicts[0].resolution {
        ConflictResolution::GLR => {}
        _ => panic!("expected GLR resolution"),
    }
}

#[test]
fn conflict_prec_multiple_conflicts() {
    let mut g = GrammarBuilder::new("multi_conflict")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .token("TIMES", r"\*")
        .rule_with_precedence("e", vec!["e", "PLUS", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "TIMES", "e"], 2, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    let e_id = g.find_symbol_by_name("e").unwrap();
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![e_id],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(1)),
    });
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![e_id],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(2)),
    });
    assert_eq!(g.conflicts.len(), 2);
}

#[test]
fn conflict_prec_conflict_with_dynamic_prec() {
    let mut g = GrammarBuilder::new("dyn_conflict")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Dynamic(10)),
    });
    match &g.conflicts[0].resolution {
        ConflictResolution::Precedence(PrecedenceKind::Dynamic(v)) => assert_eq!(*v, 10),
        _ => panic!("expected Dynamic precedence resolution"),
    }
}

#[test]
fn conflict_prec_prec_rules_coexist_with_conflicts() {
    let mut g = build_grammar_with_precedence();
    let expr_id = g.find_symbol_by_name("expression").unwrap();
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![expr_id],
        resolution: ConflictResolution::GLR,
    });
    // Precedence rules remain intact
    let prec_count = g.all_rules().filter(|r| r.precedence.is_some()).count();
    assert_eq!(prec_count, 4);
    assert_eq!(g.conflicts.len(), 1);
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional coverage: Precedence struct, symbol matching, serde
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn prec_struct_level_and_assoc() {
    let p = Precedence {
        level: 7,
        associativity: Associativity::Right,
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    assert_eq!(p.level, 7);
    assert_eq!(p.associativity, Associativity::Right);
    assert_eq!(p.symbols.len(), 2);
}

#[test]
fn prec_struct_empty_symbols() {
    let p = Precedence {
        level: 0,
        associativity: Associativity::None,
        symbols: Vec::new(),
    };
    assert!(p.symbols.is_empty());
}

#[test]
fn prec_struct_clone_equality() {
    let p = Precedence {
        level: 3,
        associativity: Associativity::Left,
        symbols: vec![SymbolId(10)],
    };
    let p2 = p.clone();
    assert_eq!(p, p2);
}

#[test]
fn prec_struct_different_levels_not_equal() {
    let a = Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: Vec::new(),
    };
    let b = Precedence {
        level: 2,
        associativity: Associativity::Left,
        symbols: Vec::new(),
    };
    assert_ne!(a, b);
}

#[test]
fn prec_struct_different_assoc_not_equal() {
    let a = Precedence {
        level: 1,
        associativity: Associativity::Left,
        symbols: Vec::new(),
    };
    let b = Precedence {
        level: 1,
        associativity: Associativity::Right,
        symbols: Vec::new(),
    };
    assert_ne!(a, b);
}

#[test]
fn prec_struct_debug_contains_level() {
    let p = Precedence {
        level: 42,
        associativity: Associativity::None,
        symbols: Vec::new(),
    };
    let dbg = format!("{:?}", p);
    assert!(dbg.contains("42"));
}

#[test]
fn prec_find_symbol_by_name_exists() {
    let g = build_grammar_with_precedence();
    let id = g.find_symbol_by_name("expression");
    assert!(id.is_some());
}

#[test]
fn prec_find_symbol_by_name_missing() {
    let g = build_grammar_with_precedence();
    let id = g.find_symbol_by_name("nonexistent");
    assert!(id.is_none());
}

#[test]
fn prec_get_rules_for_symbol_returns_rules() {
    let g = build_grammar_with_precedence();
    let expr_id = g.find_symbol_by_name("expression").unwrap();
    let rules = g.get_rules_for_symbol(expr_id);
    assert!(rules.is_some());
    assert_eq!(rules.unwrap().len(), 5);
}

#[test]
fn prec_rule_rhs_contains_terminals_and_nonterminals() {
    let g = build_grammar_with_precedence();
    let expr_id = g.find_symbol_by_name("expression").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    // Second rule: expression -> expression PLUS expression
    let binary_rule = &rules[1];
    assert_eq!(binary_rule.rhs.len(), 3);
    for sym in &binary_rule.rhs {
        match sym {
            Symbol::Terminal(_) | Symbol::NonTerminal(_) => {}
            _ => {}
        }
    }
}

#[test]
fn prec_grammar_name_preserved() {
    let g = build_grammar_with_precedence();
    assert_eq!(g.name, "prec_test");
}

#[test]
fn prec_grammar_tokens_populated() {
    let g = build_grammar_with_precedence();
    assert!(!g.tokens.is_empty());
}

#[test]
fn prec_rule_lhs_matches_symbol() {
    let g = build_grammar_with_precedence();
    let expr_id = g.find_symbol_by_name("expression").unwrap();
    for rule in g.all_rules() {
        if rule.precedence.is_some() {
            assert_eq!(rule.lhs, expr_id);
        }
    }
}

#[test]
fn prec_right_assoc_grammar_name() {
    let g = build_right_assoc_grammar();
    assert_eq!(g.name, "right_assoc");
}

#[test]
fn prec_none_assoc_two_operators() {
    let g = build_none_assoc_grammar();
    let prec_count = g.all_rules().filter(|r| r.precedence.is_some()).count();
    assert_eq!(prec_count, 2);
}

#[test]
fn prec_external_with_precedence() {
    let g = GrammarBuilder::new("ext_prec")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .external("INDENT")
        .rule_with_precedence("e", vec!["e", "PLUS", "e"], 1, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    assert!(!g.externals.is_empty());
    let prec_count = g.all_rules().filter(|r| r.precedence.is_some()).count();
    assert_eq!(prec_count, 1);
}

#[test]
fn prec_extra_with_precedence() {
    let g = GrammarBuilder::new("extra_prec")
        .token("N", r"\d+")
        .token("PLUS", r"\+")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule_with_precedence("e", vec!["e", "PLUS", "e"], 1, Associativity::Left)
        .rule("e", vec!["N"])
        .start("e")
        .build();
    assert!(!g.extras.is_empty());
    let prec_count = g.all_rules().filter(|r| r.precedence.is_some()).count();
    assert_eq!(prec_count, 1);
}

#[test]
fn prec_mixed_grammar_all_assoc_variants_present() {
    let g = build_mixed_assoc_grammar();
    let assocs: Vec<_> = g.all_rules().filter_map(|r| r.associativity).collect();
    assert!(assocs.contains(&Associativity::Left));
    assert!(assocs.contains(&Associativity::Right));
    assert!(assocs.contains(&Associativity::None));
}

#[test]
fn prec_kind_static_zero() {
    let pk = PrecedenceKind::Static(0);
    assert_eq!(pk, PrecedenceKind::Static(0));
}

#[test]
fn prec_kind_dynamic_max() {
    let pk = PrecedenceKind::Dynamic(i16::MAX);
    match pk {
        PrecedenceKind::Dynamic(v) => assert_eq!(v, i16::MAX),
        _ => panic!("expected Dynamic"),
    }
}

#[test]
fn prec_kind_dynamic_min() {
    let pk = PrecedenceKind::Dynamic(i16::MIN);
    match pk {
        PrecedenceKind::Dynamic(v) => assert_eq!(v, i16::MIN),
        _ => panic!("expected Dynamic"),
    }
}

#[test]
fn prec_precedence_decl_negative_level() {
    let g = GrammarBuilder::new("neg_decl")
        .token("A", "a")
        .precedence(-3, Associativity::Left, vec!["A"])
        .rule("s", vec!["A"])
        .start("s")
        .build();
    assert_eq!(g.precedences.len(), 1);
    assert_eq!(g.precedences[0].level, -3);
}

#[test]
fn prec_serde_roundtrip_associativity() {
    let assoc = Associativity::Right;
    let json = serde_json::to_string(&assoc).unwrap();
    let back: Associativity = serde_json::from_str(&json).unwrap();
    assert_eq!(assoc, back);
}

#[test]
fn prec_serde_roundtrip_precedence_kind() {
    let pk = PrecedenceKind::Static(42);
    let json = serde_json::to_string(&pk).unwrap();
    let back: PrecedenceKind = serde_json::from_str(&json).unwrap();
    assert_eq!(pk, back);
}

#[test]
fn prec_serde_roundtrip_precedence_struct() {
    let p = Precedence {
        level: 5,
        associativity: Associativity::None,
        symbols: vec![SymbolId(1), SymbolId(2)],
    };
    let json = serde_json::to_string(&p).unwrap();
    let back: Precedence = serde_json::from_str(&json).unwrap();
    assert_eq!(p, back);
}
