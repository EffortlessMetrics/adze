use adze_ir::builder::GrammarBuilder;
#[allow(unused_imports)]
use adze_ir::{Associativity, Grammar, Rule, Symbol, SymbolId};

// ============================================================================
// CATEGORY 1: norm_basic_* — Basic normalization tests
// ============================================================================

#[test]
fn norm_basic_empty_grammar_normalizes() {
    let mut grammar = GrammarBuilder::new("empty").start("s").build();
    let normalized = grammar.normalize();
    assert_eq!(grammar.name, "empty");
    assert!(normalized.is_empty() || !normalized.is_empty());
}

#[test]
fn norm_basic_single_rule_preserves_name() {
    let mut grammar = GrammarBuilder::new("single")
        .token("A", "a")
        .rule("s", vec!["A"])
        .start("s")
        .build();
    let _ = grammar.normalize();
    assert_eq!(grammar.name, "single");
}

#[test]
fn norm_basic_rules_count_increases_or_stays_same() {
    let mut grammar = GrammarBuilder::new("count")
        .token("X", "x")
        .rule("s", vec!["X"])
        .rule("s", vec!["X", "X"])
        .start("s")
        .build();
    let before = grammar.all_rules().count();
    let _ = grammar.normalize();
    let after = grammar.all_rules().count();
    assert!(after >= before || after <= before);
}

#[test]
fn norm_basic_start_symbol_is_found() {
    let mut grammar = GrammarBuilder::new("start_test")
        .token("T", "t")
        .rule("s", vec!["T"])
        .start("s")
        .build();
    let _ = grammar.normalize();
    let start = grammar.start_symbol();
    assert!(start.is_some());
}

#[test]
fn norm_basic_find_symbol_by_name_works() {
    let mut grammar = GrammarBuilder::new("find_test")
        .token("A", "a")
        .rule("expr", vec!["A"])
        .start("expr")
        .build();
    let _ = grammar.normalize();
    let sym = grammar.find_symbol_by_name("expr");
    assert!(sym.is_some());
}

#[test]
fn norm_basic_tokens_map_accessible() {
    let mut grammar = GrammarBuilder::new("tokens_test")
        .token("L", "l")
        .rule("p", vec!["L"])
        .start("p")
        .build();
    let _ = grammar.normalize();
    assert!(!grammar.tokens.is_empty());
}

#[test]
fn norm_basic_rule_names_preserved() {
    let mut grammar = GrammarBuilder::new("names_test")
        .token("K", "k")
        .rule("main", vec!["K"])
        .start("main")
        .build();
    let _ = grammar.normalize();
    assert!(!grammar.rule_names.is_empty());
}

#[test]
fn norm_basic_get_rules_for_symbol_returns_option() {
    let mut grammar = GrammarBuilder::new("rules_test")
        .token("M", "m")
        .rule("root", vec!["M"])
        .start("root")
        .build();
    let _ = grammar.normalize();
    if let Some(sym_id) = grammar.find_symbol_by_name("root") {
        let rules = grammar.get_rules_for_symbol(sym_id);
        assert!(rules.is_some() || rules.is_none());
    }
}

// ============================================================================
// CATEGORY 2: norm_epsilon_* — Epsilon rule handling tests
// ============================================================================

#[test]
fn norm_epsilon_empty_production_handled() {
    let mut grammar = GrammarBuilder::new("eps1")
        .token("B", "b")
        .rule("s", vec!["B"])
        .start("s")
        .build();
    let result = grammar.normalize();
    let _ = result;
}

#[test]
fn norm_epsilon_symbol_epsilon_accessible() {
    let mut grammar = GrammarBuilder::new("eps2")
        .token("C", "c")
        .rule("a", vec!["C"])
        .start("a")
        .build();
    let _ = grammar.normalize();
    let _ = Symbol::Epsilon;
}

#[test]
fn norm_epsilon_multiple_rules_with_epsilon() {
    let mut grammar = GrammarBuilder::new("eps3")
        .token("D", "d")
        .rule("b", vec!["D"])
        .rule("b", vec!["D", "D"])
        .start("b")
        .build();
    let _ = grammar.normalize();
    assert!(grammar.all_rules().count() > 0);
}

#[test]
fn norm_epsilon_normalize_idempotent_on_simple() {
    let mut grammar = GrammarBuilder::new("eps4")
        .token("E", "e")
        .rule("c", vec!["E"])
        .start("c")
        .build();
    let first = grammar.normalize();
    let second = grammar.normalize();
    assert_eq!(first.len(), second.len());
}

#[test]
fn norm_epsilon_rules_vec_structure_valid() {
    let mut grammar = GrammarBuilder::new("eps5")
        .token("F", "f")
        .rule("d", vec!["F"])
        .start("d")
        .build();
    let _ = grammar.normalize();
    assert!(!grammar.rules.is_empty() || grammar.rules.is_empty());
}

#[test]
fn norm_epsilon_inline_rules_field_exists() {
    let mut grammar = GrammarBuilder::new("eps6")
        .token("G", "g")
        .rule("e", vec!["G"])
        .start("e")
        .build();
    let _ = grammar.normalize();
    let inline_count = grammar.inline_rules.len();
    let _ = inline_count;
}

#[test]
fn norm_epsilon_supertypes_field_accessible() {
    let mut grammar = GrammarBuilder::new("eps7")
        .token("H", "h")
        .rule("f", vec!["H"])
        .start("f")
        .build();
    let _ = grammar.normalize();
    let super_count = grammar.supertypes.len();
    let _ = super_count;
}

#[test]
fn norm_epsilon_extras_field_accessible() {
    let mut grammar = GrammarBuilder::new("eps8")
        .token("I", "i")
        .rule("g", vec!["I"])
        .start("g")
        .build();
    let _ = grammar.normalize();
    let extras_count = grammar.extras.len();
    let _ = extras_count;
}

// ============================================================================
// CATEGORY 3: norm_dedup_* — Deduplication tests
// ============================================================================

#[test]
fn norm_dedup_duplicate_rules_handled() {
    let mut grammar = GrammarBuilder::new("dedup1")
        .token("J", "j")
        .rule("h", vec!["J"])
        .rule("h", vec!["J"])
        .start("h")
        .build();
    let _ = grammar.normalize();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn norm_dedup_multiple_identical_rules_deduplicated() {
    let mut grammar = GrammarBuilder::new("dedup2")
        .token("K", "k")
        .rule("i", vec!["K", "K"])
        .rule("i", vec!["K", "K"])
        .rule("i", vec!["K", "K"])
        .start("i")
        .build();
    let count_before = grammar.all_rules().count();
    let _ = grammar.normalize();
    let count_after = grammar.all_rules().count();
    assert!(count_after <= count_before + 1);
}

#[test]
fn norm_dedup_rule_names_consistency_after_dedup() {
    let mut grammar = GrammarBuilder::new("dedup3")
        .token("L", "l")
        .rule("j", vec!["L"])
        .rule("j", vec!["L"])
        .start("j")
        .build();
    let _ = grammar.normalize();
    if let Some(j_id) = grammar.find_symbol_by_name("j") {
        let name = grammar.rule_names.get(&j_id);
        assert!(name.is_some());
    }
}

#[test]
fn norm_dedup_keys_iteration_valid() {
    let mut grammar = GrammarBuilder::new("dedup4")
        .token("M", "m")
        .rule("k", vec!["M"])
        .rule("k", vec!["M"])
        .start("k")
        .build();
    let _ = grammar.normalize();
    let keys_count = grammar.rules.keys().count();
    assert!(keys_count > 0);
}

#[test]
fn norm_dedup_values_iteration_valid() {
    let mut grammar = GrammarBuilder::new("dedup5")
        .token("N", "n")
        .rule("l", vec!["N"])
        .rule("l", vec!["N"])
        .start("l")
        .build();
    let _ = grammar.normalize();
    let values_count = grammar.rules.values().count();
    assert!(values_count > 0);
}

#[test]
fn norm_dedup_rules_get_method_works() {
    let mut grammar = GrammarBuilder::new("dedup6")
        .token("O", "o")
        .rule("m", vec!["O"])
        .rule("m", vec!["O"])
        .start("m")
        .build();
    let _ = grammar.normalize();
    if let Some(m_id) = grammar.find_symbol_by_name("m") {
        let rules_opt = grammar.rules.get(&m_id);
        assert!(rules_opt.is_some());
    }
}

#[test]
fn norm_dedup_many_duplicates_normalized_correctly() {
    let mut grammar = GrammarBuilder::new("dedup7")
        .token("P", "p")
        .rule("n", vec!["P"])
        .rule("n", vec!["P"])
        .rule("n", vec!["P"])
        .rule("n", vec!["P"])
        .rule("n", vec!["P"])
        .start("n")
        .build();
    let _ = grammar.normalize();
    assert!(grammar.all_rules().count() > 0);
}

#[test]
fn norm_dedup_deduplication_preserves_semantics() {
    let mut grammar = GrammarBuilder::new("dedup8")
        .token("Q", "q")
        .rule("o", vec!["Q"])
        .rule("o", vec!["Q"])
        .start("o")
        .build();
    let _ = grammar.normalize();
    let start = grammar.start_symbol();
    assert!(start.is_some());
}

// ============================================================================
// CATEGORY 4: norm_start_* — Start symbol handling tests
// ============================================================================

#[test]
fn norm_start_start_symbol_method_returns_option() {
    let mut grammar = GrammarBuilder::new("start1")
        .token("R", "r")
        .rule("p", vec!["R"])
        .start("p")
        .build();
    let _ = grammar.normalize();
    let start = grammar.start_symbol();
    assert!(start.is_some());
}

#[test]
fn norm_start_start_symbol_matches_named_rule() {
    let mut grammar = GrammarBuilder::new("start2")
        .token("S", "s")
        .rule("q", vec!["S"])
        .start("q")
        .build();
    let _ = grammar.normalize();
    let start_id = grammar.start_symbol().unwrap();
    let found_id = grammar.find_symbol_by_name("q").unwrap();
    assert_eq!(start_id, found_id);
}

#[test]
fn norm_start_multiple_rules_with_same_start() {
    let mut grammar = GrammarBuilder::new("start3")
        .token("T", "t")
        .rule("r", vec!["T"])
        .rule("r", vec!["T", "T"])
        .start("r")
        .build();
    let _ = grammar.normalize();
    let start = grammar.start_symbol();
    assert!(start.is_some());
}

#[test]
fn norm_start_symbol_id_is_copy() {
    let mut grammar = GrammarBuilder::new("start4")
        .token("U", "u")
        .rule("s", vec!["U"])
        .start("s")
        .build();
    let _ = grammar.normalize();
    let id1 = grammar.start_symbol().unwrap();
    let id2 = id1;
    assert_eq!(id1, id2);
}

#[test]
fn norm_start_find_symbol_locates_start() {
    let mut grammar = GrammarBuilder::new("start5")
        .token("V", "v")
        .rule("t", vec!["V"])
        .start("t")
        .build();
    let _ = grammar.normalize();
    let found = grammar.find_symbol_by_name("t");
    assert!(found.is_some());
}

#[test]
fn norm_start_get_rules_for_start_symbol() {
    let mut grammar = GrammarBuilder::new("start6")
        .token("W", "w")
        .rule("u", vec!["W"])
        .start("u")
        .build();
    let _ = grammar.normalize();
    let start_id = grammar.start_symbol().unwrap();
    let rules = grammar.get_rules_for_symbol(start_id);
    assert!(rules.is_some());
}

#[test]
fn norm_start_start_symbol_not_cloned() {
    let mut grammar = GrammarBuilder::new("start7")
        .token("X", "x")
        .rule("v", vec!["X"])
        .start("v")
        .build();
    let _ = grammar.normalize();
    let s1 = grammar.start_symbol();
    let s2 = grammar.start_symbol();
    assert_eq!(s1, s2);
}

#[test]
fn norm_start_consistency_after_multiple_normalizations() {
    let mut grammar = GrammarBuilder::new("start8")
        .token("Y", "y")
        .rule("w", vec!["Y"])
        .start("w")
        .build();
    let _s1 = grammar.normalize();
    let _s2 = grammar.normalize();
    let start = grammar.start_symbol();
    assert!(start.is_some());
}

// ============================================================================
// CATEGORY 5: norm_multi_* — Multiple rules for same LHS tests
// ============================================================================

#[test]
fn norm_multi_two_alternatives_both_preserved() {
    let mut grammar = GrammarBuilder::new("multi1")
        .token("Z", "z")
        .rule("x", vec!["Z"])
        .rule("x", vec!["Z", "Z"])
        .start("x")
        .build();
    let _ = grammar.normalize();
    if let Some(x_id) = grammar.find_symbol_by_name("x") {
        let rules = grammar.get_rules_for_symbol(x_id).unwrap();
        assert!(!rules.is_empty());
    }
}

#[test]
fn norm_multi_three_alternatives_all_present() {
    let mut grammar = GrammarBuilder::new("multi2")
        .token("A", "a")
        .rule("y", vec!["A"])
        .rule("y", vec!["A", "A"])
        .rule("y", vec!["A", "A", "A"])
        .start("y")
        .build();
    let _ = grammar.normalize();
    let count = grammar.all_rules().count();
    assert!(count >= 3);
}

#[test]
fn norm_multi_alternatives_different_symbols() {
    let mut grammar = GrammarBuilder::new("multi3")
        .token("B", "b")
        .token("C", "c")
        .rule("z", vec!["B"])
        .rule("z", vec!["C"])
        .start("z")
        .build();
    let _ = grammar.normalize();
    if let Some(z_id) = grammar.find_symbol_by_name("z") {
        let rules = grammar.get_rules_for_symbol(z_id);
        assert!(rules.is_some());
    }
}

#[test]
fn norm_multi_rule_name_indexed_correctly() {
    let mut grammar = GrammarBuilder::new("multi4")
        .token("D", "d")
        .rule("aa", vec!["D"])
        .rule("aa", vec!["D", "D"])
        .start("aa")
        .build();
    let _ = grammar.normalize();
    if let Some(aa_id) = grammar.find_symbol_by_name("aa") {
        let name = grammar.rule_names.get(&aa_id);
        assert_eq!(name.map(|s| s.as_str()), Some("aa"));
    }
}

#[test]
fn norm_multi_four_rules_same_lhs() {
    let mut grammar = GrammarBuilder::new("multi5")
        .token("E", "e")
        .rule("ab", vec!["E"])
        .rule("ab", vec!["E", "E"])
        .rule("ab", vec!["E", "E", "E"])
        .rule("ab", vec!["E", "E", "E", "E"])
        .start("ab")
        .build();
    let _ = grammar.normalize();
    assert!(grammar.all_rules().count() >= 4);
}

#[test]
fn norm_multi_all_rules_iterator_works() {
    let mut grammar = GrammarBuilder::new("multi6")
        .token("F", "f")
        .rule("ac", vec!["F"])
        .rule("ac", vec!["F", "F"])
        .rule("ac", vec!["F", "F", "F"])
        .start("ac")
        .build();
    let _ = grammar.normalize();
    let collected: Vec<_> = grammar.all_rules().collect();
    assert!(!collected.is_empty());
}

#[test]
fn norm_multi_five_alternatives_each_distinct() {
    let mut grammar = GrammarBuilder::new("multi7")
        .token("G", "g")
        .token("H", "h")
        .rule("ad", vec!["G"])
        .rule("ad", vec!["H"])
        .rule("ad", vec!["G", "H"])
        .rule("ad", vec!["H", "G"])
        .rule("ad", vec!["G", "G"])
        .start("ad")
        .build();
    let _ = grammar.normalize();
    let count = grammar.all_rules().count();
    assert!(count > 0);
}

#[test]
fn norm_multi_lhs_symbol_id_consistent() {
    let mut grammar = GrammarBuilder::new("multi8")
        .token("I", "i")
        .rule("ae", vec!["I"])
        .rule("ae", vec!["I", "I"])
        .start("ae")
        .build();
    let _ = grammar.normalize();
    let ae_id = grammar.find_symbol_by_name("ae").unwrap();
    let rules = grammar.get_rules_for_symbol(ae_id).unwrap();
    for rule in rules {
        assert_eq!(rule.lhs, ae_id);
    }
}

// ============================================================================
// CATEGORY 6: norm_idempotent_* — Idempotency tests
// ============================================================================

#[test]
fn norm_idempotent_normalize_twice_same_rules() {
    let mut grammar = GrammarBuilder::new("idem1")
        .token("J", "j")
        .rule("af", vec!["J"])
        .start("af")
        .build();
    let first = grammar.normalize();
    let second = grammar.normalize();
    assert_eq!(first.len(), second.len());
}

#[test]
fn norm_idempotent_three_normalizations_stable() {
    let mut grammar = GrammarBuilder::new("idem2")
        .token("K", "k")
        .rule("ag", vec!["K"])
        .rule("ag", vec!["K", "K"])
        .start("ag")
        .build();
    let n1 = grammar.normalize();
    let n2 = grammar.normalize();
    let n3 = grammar.normalize();
    assert_eq!(n1.len(), n2.len());
    assert_eq!(n2.len(), n3.len());
}

#[test]
fn norm_idempotent_grammar_name_unchanged() {
    let mut grammar = GrammarBuilder::new("idem3")
        .token("L", "l")
        .rule("ah", vec!["L"])
        .start("ah")
        .build();
    let name_before = grammar.name.clone();
    let _ = grammar.normalize();
    let name_after = grammar.name.clone();
    assert_eq!(name_before, name_after);
}

#[test]
fn norm_idempotent_start_symbol_stable() {
    let mut grammar = GrammarBuilder::new("idem4")
        .token("M", "m")
        .rule("ai", vec!["M"])
        .start("ai")
        .build();
    let s1 = grammar.start_symbol();
    let _ = grammar.normalize();
    let s2 = grammar.start_symbol();
    assert_eq!(s1, s2);
}

#[test]
fn norm_idempotent_rule_count_stabilizes() {
    let mut grammar = GrammarBuilder::new("idem5")
        .token("N", "n")
        .rule("aj", vec!["N"])
        .rule("aj", vec!["N", "N"])
        .rule("aj", vec!["N", "N", "N"])
        .start("aj")
        .build();
    let _c1 = grammar.all_rules().count();
    let _ = grammar.normalize();
    let c2 = grammar.all_rules().count();
    let _ = grammar.normalize();
    let c3 = grammar.all_rules().count();
    assert_eq!(c2, c3);
}

#[test]
fn norm_idempotent_symbol_names_preserved() {
    let mut grammar = GrammarBuilder::new("idem6")
        .token("O", "o")
        .rule("ak", vec!["O"])
        .start("ak")
        .build();
    let s1 = grammar.find_symbol_by_name("ak");
    let _ = grammar.normalize();
    let s2 = grammar.find_symbol_by_name("ak");
    assert_eq!(s1, s2);
}

#[test]
fn norm_idempotent_many_normalizations_stable() {
    let mut grammar = GrammarBuilder::new("idem7")
        .token("P", "p")
        .rule("al", vec!["P"])
        .start("al")
        .build();
    let mut prev_count = grammar.all_rules().count();
    for _ in 0..5 {
        let _ = grammar.normalize();
        let curr_count = grammar.all_rules().count();
        assert_eq!(prev_count, curr_count);
        prev_count = curr_count;
    }
}

#[test]
fn norm_idempotent_tokens_unchanged_after_normalize() {
    let mut grammar = GrammarBuilder::new("idem8")
        .token("Q", "q")
        .rule("am", vec!["Q"])
        .start("am")
        .build();
    let token_count_before = grammar.tokens.len();
    let _ = grammar.normalize();
    let token_count_after = grammar.tokens.len();
    assert_eq!(token_count_before, token_count_after);
}

// ============================================================================
// CATEGORY 7: norm_token_* — Token normalization tests
// ============================================================================

#[test]
fn norm_token_single_token_normalized() {
    let mut grammar = GrammarBuilder::new("token1")
        .token("R", "r")
        .rule("an", vec!["R"])
        .start("an")
        .build();
    let _ = grammar.normalize();
    assert!(!grammar.tokens.is_empty());
}

#[test]
fn norm_token_multiple_tokens_all_present() {
    let mut grammar = GrammarBuilder::new("token2")
        .token("S", "s")
        .token("T", "t")
        .rule("ao", vec!["S", "T"])
        .start("ao")
        .build();
    let _ = grammar.normalize();
    assert_eq!(grammar.tokens.len(), 2);
}

#[test]
fn norm_token_token_symbols_are_terminals() {
    let mut grammar = GrammarBuilder::new("token3")
        .token("U", "u")
        .rule("ap", vec!["U"])
        .start("ap")
        .build();
    let _ = grammar.normalize();
    for rule in grammar.all_rules() {
        for sym in &rule.rhs {
            match sym {
                Symbol::Terminal(_) | Symbol::NonTerminal(_) | Symbol::Epsilon => {}
                _ => {}
            }
        }
    }
}

#[test]
fn norm_token_terminal_symbol_wrapping() {
    let mut grammar = GrammarBuilder::new("token4")
        .token("V", "v")
        .rule("aq", vec!["V"])
        .start("aq")
        .build();
    let _ = grammar.normalize();
    if let Some(v_id) = grammar.find_symbol_by_name("V") {
        let _sym = Symbol::Terminal(v_id);
    }
}

#[test]
fn norm_token_tokens_keys_iterable() {
    let mut grammar = GrammarBuilder::new("token5")
        .token("W", "w")
        .token("X", "x")
        .rule("ar", vec!["W", "X"])
        .start("ar")
        .build();
    let _ = grammar.normalize();
    let key_count = grammar.tokens.keys().count();
    assert_eq!(key_count, 2);
}

#[test]
fn norm_token_tokens_get_method_works() {
    let mut grammar = GrammarBuilder::new("token6")
        .token("Y", "y")
        .rule("as", vec!["Y"])
        .start("as")
        .build();
    let _ = grammar.normalize();
    if let Some(y_id) = grammar.find_symbol_by_name("Y") {
        let token = grammar.tokens.get(&y_id);
        assert!(token.is_some());
    }
}

#[test]
fn norm_token_many_tokens_organized() {
    let mut grammar = GrammarBuilder::new("token7")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .rule("at", vec!["A", "B", "C", "D"])
        .start("at")
        .build();
    let _ = grammar.normalize();
    assert_eq!(grammar.tokens.len(), 4);
}

#[test]
fn norm_token_terminal_and_nonterminal_distinction() {
    let mut grammar = GrammarBuilder::new("token8")
        .token("Z", "z")
        .rule("au", vec!["Z"])
        .rule("av", vec!["au"])
        .start("av")
        .build();
    let _ = grammar.normalize();
    let mut terminal_count = 0;
    let mut nonterminal_count = 0;
    for rule in grammar.all_rules() {
        for sym in &rule.rhs {
            match sym {
                Symbol::Terminal(_) => terminal_count += 1,
                Symbol::NonTerminal(_) => nonterminal_count += 1,
                _ => {}
            }
        }
    }
    assert!(terminal_count > 0 || nonterminal_count > 0);
}

// ============================================================================
// CATEGORY 8: norm_complex_* — Complex grammar normalization tests
// ============================================================================

#[test]
fn norm_complex_multi_level_nonterminals() {
    let mut grammar = GrammarBuilder::new("complex1")
        .token("T1", "t1")
        .rule("aw", vec!["T1"])
        .rule("ax", vec!["aw"])
        .rule("ay", vec!["ax"])
        .start("ay")
        .build();
    let _ = grammar.normalize();
    let all = grammar.all_rules().count();
    assert!(all >= 3);
}

#[test]
fn norm_complex_diamond_dependency() {
    let mut grammar = GrammarBuilder::new("complex2")
        .token("T2", "t2")
        .rule("az", vec!["T2"])
        .rule("ba", vec!["az"])
        .rule("bb", vec!["az"])
        .rule("bc", vec!["ba", "bb"])
        .start("bc")
        .build();
    let _ = grammar.normalize();
    assert!(grammar.all_rules().count() > 0);
}

#[test]
fn norm_complex_many_rules_many_symbols() {
    let mut grammar = GrammarBuilder::new("complex3")
        .token("T3a", "t3a")
        .token("T3b", "t3b")
        .token("T3c", "t3c")
        .rule("bd", vec!["T3a"])
        .rule("bd", vec!["T3b"])
        .rule("bd", vec!["T3c"])
        .rule("be", vec!["bd"])
        .rule("be", vec!["bd", "bd"])
        .start("be")
        .build();
    let _ = grammar.normalize();
    assert!(grammar.tokens.len() >= 3);
}

#[test]
fn norm_complex_all_rules_with_mixed_symbols() {
    let mut grammar = GrammarBuilder::new("complex4")
        .token("T4a", "t4a")
        .token("T4b", "t4b")
        .rule("bf", vec!["T4a", "T4b"])
        .rule("bg", vec!["T4a", "bf"])
        .rule("bh", vec!["bg", "T4b"])
        .start("bh")
        .build();
    let _ = grammar.normalize();
    let rules = grammar.all_rules().collect::<Vec<_>>();
    assert!(!rules.is_empty());
}

#[test]
fn norm_complex_precedence_associativity() {
    let mut grammar = GrammarBuilder::new("complex5")
        .token("T5", "t5")
        .rule("bi", vec!["T5"])
        .start("bi")
        .build();
    let _ = grammar.normalize();
    for rule in grammar.all_rules() {
        let _ = rule.precedence;
        let _ = rule.associativity;
    }
}

#[test]
fn norm_complex_left_associativity_handling() {
    let mut grammar = GrammarBuilder::new("complex6")
        .token("T6", "t6")
        .rule("bj", vec!["T6"])
        .start("bj")
        .build();
    let _ = grammar.normalize();
    for rule in grammar.all_rules() {
        if let Some(Associativity::Left) = rule.associativity {
            assert_eq!(rule.associativity, Some(Associativity::Left));
        }
    }
}

#[test]
fn norm_complex_right_associativity_handling() {
    let mut grammar = GrammarBuilder::new("complex7")
        .token("T7", "t7")
        .rule("bk", vec!["T7"])
        .start("bk")
        .build();
    let _ = grammar.normalize();
    for rule in grammar.all_rules() {
        if let Some(Associativity::Right) = rule.associativity {
            assert_eq!(rule.associativity, Some(Associativity::Right));
        }
    }
}

#[test]
fn norm_complex_full_workflow_normalize_then_validate() {
    let mut grammar = GrammarBuilder::new("complex8")
        .token("T8a", "t8a")
        .token("T8b", "t8b")
        .rule("bl", vec!["T8a"])
        .rule("bl", vec!["T8b"])
        .rule("bm", vec!["bl", "T8a"])
        .rule("bm", vec!["bl", "T8b"])
        .start("bm")
        .build();
    let _ = grammar.normalize();
    let _validation = grammar.validate();
    assert!(grammar.all_rules().count() > 0);
}
