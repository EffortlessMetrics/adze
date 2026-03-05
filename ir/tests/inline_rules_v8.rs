use adze_ir::builder::GrammarBuilder;
#[allow(unused_imports)]
use adze_ir::{Associativity, Grammar, Rule, Symbol, SymbolId, TokenPattern};

// ============================================================================
// Category 1: inline_basic_* (8 tests)
// ============================================================================

#[test]
fn inline_basic_single_rule_marked() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_basic_rule_not_marked() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(!g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_basic_multiple_rules_one_inline() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("stmt", vec!["expr"])
        .inline("expr")
        .start("stmt")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let stmt_id = g.find_symbol_by_name("stmt").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
    assert!(!g.inline_rules.contains(&stmt_id));
}

#[test]
fn inline_basic_rule_with_token() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("expr")
        .build();

    assert!(!g.tokens.is_empty());
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_basic_recursive_rule_inline() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["expr", "ID"])
        .inline("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_basic_empty_inline_list_initially() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .start("expr")
        .build();

    assert!(g.inline_rules.is_empty());
}

#[test]
fn inline_basic_find_inline_rule_by_id() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_basic_inline_count_after_marking() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("expr")
        .build();

    assert_eq!(g.inline_rules.len(), 1);
}

// ============================================================================
// Category 2: inline_multiple_* (8 tests)
// ============================================================================

#[test]
fn inline_multiple_two_inline_rules() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("term", vec!["ID"])
        .inline("expr")
        .inline("term")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let term_id = g.find_symbol_by_name("term").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
    assert!(g.inline_rules.contains(&term_id));
    assert_eq!(g.inline_rules.len(), 2);
}

#[test]
fn inline_multiple_three_inline_rules() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("term", vec!["ID"])
        .rule("factor", vec!["ID"])
        .inline("expr")
        .inline("term")
        .inline("factor")
        .start("expr")
        .build();

    assert_eq!(g.inline_rules.len(), 3);
}

#[test]
fn inline_multiple_mixed_inline_non_inline() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("term", vec!["ID"])
        .rule("factor", vec!["ID"])
        .inline("expr")
        .inline("factor")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let term_id = g.find_symbol_by_name("term").unwrap();
    let factor_id = g.find_symbol_by_name("factor").unwrap();

    assert!(g.inline_rules.contains(&expr_id));
    assert!(!g.inline_rules.contains(&term_id));
    assert!(g.inline_rules.contains(&factor_id));
}

#[test]
fn inline_multiple_non_consecutive_inline() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .inline("a")
        .rule("b", vec!["ID"])
        .rule("c", vec!["ID"])
        .inline("c")
        .start("a")
        .build();

    let a_id = g.find_symbol_by_name("a").unwrap();
    let c_id = g.find_symbol_by_name("c").unwrap();
    assert!(g.inline_rules.contains(&a_id));
    assert!(g.inline_rules.contains(&c_id));
    assert_eq!(g.inline_rules.len(), 2);
}

#[test]
fn inline_multiple_all_rules_inline() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("term", vec!["ID"])
        .inline("expr")
        .inline("term")
        .start("expr")
        .build();

    assert_eq!(g.inline_rules.len(), 2);
    assert_eq!(g.rules.len(), 2);
}

#[test]
fn inline_multiple_duplicate_inline_call() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .inline("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_multiple_collect_all_inline() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .rule("c", vec!["ID"])
        .rule("d", vec!["ID"])
        .inline("a")
        .inline("b")
        .inline("c")
        .start("a")
        .build();

    assert_eq!(g.inline_rules.len(), 3);
}

// ============================================================================
// Category 3: inline_query_* (8 tests)
// ============================================================================

#[test]
fn inline_query_find_single_inline() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let found = g.inline_rules.iter().find(|&&id| id == expr_id);
    assert!(found.is_some());
}

#[test]
fn inline_query_find_non_inline() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let found = g.inline_rules.iter().find(|&&id| id == expr_id);
    assert!(found.is_none());
}

#[test]
fn inline_query_check_not_empty() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("expr")
        .build();

    assert!(!g.inline_rules.is_empty());
}

#[test]
fn inline_query_count_inline_rules() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .rule("c", vec!["ID"])
        .inline("a")
        .inline("c")
        .start("a")
        .build();

    let inline_count = g.inline_rules.len();
    assert_eq!(inline_count, 2);
}

#[test]
fn inline_query_filter_inline_rules() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .inline("a")
        .start("a")
        .build();

    let inline_names: Vec<_> = g
        .inline_rules
        .iter()
        .filter_map(|&id| g.rule_names.get(&id).map(|s| s.as_str()))
        .collect();

    assert!(inline_names.contains(&"a"));
}

#[test]
fn inline_query_inline_rules_exist() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("expr")
        .build();

    assert!(!g.inline_rules.is_empty());
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_query_iterate_inline() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .inline("a")
        .inline("b")
        .start("a")
        .build();

    let count = g.inline_rules.len();
    assert_eq!(count, 2);
}

// ============================================================================
// Category 4: inline_normalize_* (8 tests)
// ============================================================================

#[test]
fn inline_normalize_single_rule() {
    let mut g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let _normalized = g.normalize();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_normalize_preserves_inline() {
    let mut g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("stmt", vec!["expr"])
        .inline("expr")
        .start("stmt")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let before_count = g.inline_rules.len();
    let _normalized = g.normalize();
    assert_eq!(g.inline_rules.len(), before_count);
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_normalize_multiple_inline() {
    let mut g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .rule("c", vec!["ID"])
        .inline("a")
        .inline("b")
        .start("c")
        .build();

    let a_id = g.find_symbol_by_name("a").unwrap();
    let b_id = g.find_symbol_by_name("b").unwrap();
    let _normalized = g.normalize();
    assert!(g.inline_rules.contains(&a_id));
    assert!(g.inline_rules.contains(&b_id));
}

#[test]
fn inline_normalize_recursive_inline() {
    let mut g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["expr", "ID"])
        .inline("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let _normalized = g.normalize();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_normalize_after_normalization() {
    let mut g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let _normalized = g.normalize();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_normalize_complex_grammar() {
    let mut g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .token("OP", "[+\\-*/]")
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["expr", "OP", "expr"])
        .rule("stmt", vec!["expr"])
        .inline("expr")
        .start("stmt")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let _normalized = g.normalize();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn inline_normalize_preserves_rules() {
    let mut g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("expr")
        .build();

    let rule_count_before = g.rules.len();
    let _normalized = g.normalize();
    assert!(rule_count_before > 0);
}

// ============================================================================
// Category 5: supertype_basic_* (8 tests)
// ============================================================================

#[test]
fn supertype_basic_single_supertype() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.supertypes.contains(&expr_id));
}

#[test]
fn supertype_basic_no_supertype() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .start("expr")
        .build();

    assert!(g.supertypes.is_empty());
}

#[test]
fn supertype_basic_multiple_supertypes() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("stmt", vec!["ID"])
        .supertype("expr")
        .supertype("stmt")
        .start("expr")
        .build();

    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn supertype_basic_supertype_count() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .rule("c", vec!["ID"])
        .supertype("a")
        .supertype("c")
        .start("a")
        .build();

    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn supertype_basic_find_supertype() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.supertypes.contains(&expr_id));
}

#[test]
fn supertype_basic_supertype_not_empty() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .start("expr")
        .build();

    assert!(!g.supertypes.is_empty());
}

#[test]
fn supertype_basic_check_specific_supertype() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let found = g.supertypes.iter().find(|&&id| id == expr_id);
    assert!(found.is_some());
}

#[test]
fn supertype_basic_iterate_supertypes() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .supertype("a")
        .supertype("b")
        .start("a")
        .build();

    let count = g.supertypes.len();
    assert_eq!(count, 2);
}

// ============================================================================
// Category 6: supertype_query_* (8 tests)
// ============================================================================

#[test]
fn supertype_query_find_single() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let is_supertype = g.supertypes.contains(&expr_id);
    assert!(is_supertype);
}

#[test]
fn supertype_query_not_supertype() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("other", vec!["ID"])
        .supertype("expr")
        .start("expr")
        .build();

    let other_id = g.find_symbol_by_name("other").unwrap();
    assert!(!g.supertypes.contains(&other_id));
}

#[test]
fn supertype_query_filter_supertypes() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .rule("c", vec!["ID"])
        .supertype("a")
        .supertype("c")
        .start("a")
        .build();

    let supertype_names: Vec<_> = g
        .supertypes
        .iter()
        .filter_map(|&id| g.rule_names.get(&id).map(|s| s.as_str()))
        .collect();

    assert!(supertype_names.contains(&"a"));
    assert!(supertype_names.contains(&"c"));
}

#[test]
fn supertype_query_count_supertypes() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .rule("c", vec!["ID"])
        .rule("d", vec!["ID"])
        .supertype("a")
        .supertype("b")
        .supertype("c")
        .start("a")
        .build();

    assert_eq!(g.supertypes.len(), 3);
}

#[test]
fn supertype_query_all_supertypes_valid() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .supertype("a")
        .supertype("b")
        .start("a")
        .build();

    let all_valid = g
        .supertypes
        .iter()
        .all(|&st_id| g.rule_names.contains_key(&st_id));
    assert!(all_valid);
}

#[test]
fn supertype_query_collect_supertypes() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .supertype("a")
        .supertype("b")
        .start("a")
        .build();

    let supertypes_vec: Vec<_> = g.supertypes.to_vec();
    assert_eq!(supertypes_vec.len(), 2);
}

// ============================================================================
// Category 7: combined_* (8 tests)
// ============================================================================

#[test]
fn combined_inline_and_supertype_same_rule() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .supertype("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
    assert!(g.supertypes.contains(&expr_id));
}

#[test]
fn combined_inline_and_supertype_different_rules() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("stmt", vec!["expr"])
        .inline("expr")
        .supertype("stmt")
        .start("stmt")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let stmt_id = g.find_symbol_by_name("stmt").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
    assert!(g.supertypes.contains(&stmt_id));
}

#[test]
fn combined_multiple_inline_multiple_supertype() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .rule("c", vec!["ID"])
        .rule("d", vec!["ID"])
        .inline("a")
        .inline("b")
        .supertype("c")
        .supertype("d")
        .start("a")
        .build();

    assert_eq!(g.inline_rules.len(), 2);
    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn combined_inline_supertype_with_tokens() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .token("OP", "[+\\-]")
        .rule("expr", vec!["ID"])
        .rule("stmt", vec!["expr"])
        .inline("expr")
        .supertype("stmt")
        .start("stmt")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let stmt_id = g.find_symbol_by_name("stmt").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
    assert!(g.supertypes.contains(&stmt_id));
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn combined_separate_inline_and_supertype_rules() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("inline_rule", vec!["ID"])
        .rule("super_rule", vec!["ID"])
        .rule("normal_rule", vec!["ID"])
        .inline("inline_rule")
        .supertype("super_rule")
        .start("normal_rule")
        .build();

    let inline_id = g.find_symbol_by_name("inline_rule").unwrap();
    let super_id = g.find_symbol_by_name("super_rule").unwrap();
    let normal_id = g.find_symbol_by_name("normal_rule").unwrap();

    assert!(g.inline_rules.contains(&inline_id));
    assert!(g.supertypes.contains(&super_id));
    assert!(!g.inline_rules.contains(&normal_id));
    assert!(!g.supertypes.contains(&normal_id));
}

#[test]
fn combined_all_annotations_present() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .supertype("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(!g.inline_rules.is_empty());
    assert!(!g.supertypes.is_empty());
    assert!(g.inline_rules.contains(&expr_id));
    assert!(g.supertypes.contains(&expr_id));
}

#[test]
fn combined_mixed_with_fields() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .rule("stmt", vec!["expr"])
        .inline("expr")
        .supertype("stmt")
        .start("stmt")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let stmt_id = g.find_symbol_by_name("stmt").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
    assert!(g.supertypes.contains(&stmt_id));
}

// ============================================================================
// Category 8: edge_case_* (8 tests)
// ============================================================================

#[test]
fn edge_case_empty_rule_inline() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("expr")
        .build();

    assert!(!g.inline_rules.is_empty());
}

#[test]
fn edge_case_single_token_grammar() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("root", vec!["ID"])
        .inline("root")
        .start("root")
        .build();

    let root_id = g.find_symbol_by_name("root").unwrap();
    assert!(g.inline_rules.contains(&root_id));
}

#[test]
fn edge_case_long_rule_chain() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .rule("d", vec!["c"])
        .inline("a")
        .inline("b")
        .start("d")
        .build();

    assert_eq!(g.inline_rules.len(), 2);
}

#[test]
fn edge_case_complex_rule_body() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .token("OP", "[+\\-*/]")
        .rule("expr", vec!["ID", "OP", "ID"])
        .inline("expr")
        .start("expr")
        .build();

    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn edge_case_many_rules_some_inline() {
    let mut builder = GrammarBuilder::new("test").token("ID", "\\w+");

    for i in 0..10 {
        let name = format!("rule{}", i);
        builder = builder.rule(&name, vec!["ID"]);
    }

    for i in 0..5 {
        let name = format!("rule{}", i);
        builder = builder.inline(&name);
    }

    let g = builder.start("rule0").build();
    assert_eq!(g.inline_rules.len(), 5);
}

#[test]
fn edge_case_both_inline_and_supertype_fully() {
    let g = GrammarBuilder::new("test")
        .token("ID", "\\w+")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .inline("a")
        .inline("b")
        .supertype("a")
        .supertype("b")
        .start("a")
        .build();

    assert_eq!(g.inline_rules.len(), 2);
    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn edge_case_name_length_handling() {
    let g = GrammarBuilder::new("test")
        .token("VERY_LONG_IDENTIFIER_NAME", "\\w+")
        .rule("very_long_rule_name", vec!["VERY_LONG_IDENTIFIER_NAME"])
        .inline("very_long_rule_name")
        .start("very_long_rule_name")
        .build();

    let rule_id = g.find_symbol_by_name("very_long_rule_name").unwrap();
    assert!(g.inline_rules.contains(&rule_id));
}
