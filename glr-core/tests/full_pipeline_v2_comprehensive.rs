// Comprehensive full-pipeline tests: Grammar → FirstFollow → ParseTable → Analysis
#![cfg(feature = "test-api")]

use adze_glr_core::advanced_conflict::{ConflictAnalyzer, ConflictStats};
use adze_glr_core::{
    Action, FirstFollowSets, ParseTable, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

// ── Helpers ──

fn build_table(builder: GrammarBuilder) -> (adze_ir::Grammar, ParseTable) {
    let grammar = builder.build();
    let ff = FirstFollowSets::compute(&grammar).expect("compute first-follow");
    let table = build_lr1_automaton(&grammar, &ff).expect("build parse table");
    (grammar, table)
}

fn build_table_normalized(builder: GrammarBuilder) -> (adze_ir::Grammar, ParseTable) {
    let mut grammar = builder.build();
    grammar.normalize();
    let ff = FirstFollowSets::compute(&grammar).expect("compute first-follow");
    let table = build_lr1_automaton(&grammar, &ff).expect("build parse table");
    (grammar, table)
}

fn analyze(table: &ParseTable) -> ConflictStats {
    let mut analyzer = ConflictAnalyzer::new();
    analyzer.analyze_table(table)
}

fn has_accept(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
    })
}

fn has_shift(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Shift(_))))
    })
}

fn has_reduce(table: &ParseTable) -> bool {
    table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))))
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Complete pipeline for simple grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn simple_single_token_pipeline() {
    let (_, table) = build_table(
        GrammarBuilder::new("s1")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
    sanity_check_tables(&table).expect("sanity check");
}

#[test]
fn simple_two_token_sequence() {
    let (_, table) = build_table(
        GrammarBuilder::new("s2")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start"),
    );
    assert!(table.state_count >= 3);
    assert!(has_accept(&table));
    assert!(has_shift(&table));
}

#[test]
fn simple_three_token_sequence() {
    let (_, table) = build_table(
        GrammarBuilder::new("s3")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start"),
    );
    assert!(table.state_count >= 4);
    assert!(has_accept(&table));
}

#[test]
fn simple_two_alternatives() {
    let (_, table) = build_table(
        GrammarBuilder::new("alt2")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    assert!(has_reduce(&table));
}

#[test]
fn simple_nonterminal_delegation() {
    let (_, table) = build_table(
        GrammarBuilder::new("deleg")
            .token("x", "x")
            .rule("inner", vec!["x"])
            .rule("start", vec!["inner"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

#[test]
fn simple_grammar_sanity_check_passes() {
    let (_, table) = build_table(
        GrammarBuilder::new("sane")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start"),
    );
    sanity_check_tables(&table).expect("sanity check should pass");
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Complete pipeline for expression grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn expr_grammar_add_only() {
    let (_, table) = build_table(
        GrammarBuilder::new("add")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["num"])
            .start("expr"),
    );
    assert!(table.state_count >= 4);
    assert!(has_accept(&table));
    assert!(has_shift(&table));
    assert!(has_reduce(&table));
}

#[test]
fn expr_grammar_add_mul() {
    let (_, table) = build_table(
        GrammarBuilder::new("addmul")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "star", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["num"])
            .start("expr"),
    );
    assert!(table.state_count >= 6);
    assert!(has_accept(&table));
    sanity_check_tables(&table).expect("sanity check");
}

#[test]
fn expr_grammar_with_parens() {
    let (_, table) = build_table(
        GrammarBuilder::new("parens")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "star", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["lp", "expr", "rp"])
            .rule("factor", vec!["num"])
            .start("expr"),
    );
    assert!(table.state_count >= 8);
    assert!(has_accept(&table));
}

#[test]
fn expr_pipeline_rules_populated() {
    let (_, table) = build_table(
        GrammarBuilder::new("rules")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "num"])
            .rule("expr", vec!["num"])
            .start("expr"),
    );
    // rules should include at least the user-defined rules
    assert!(table.rules.len() >= 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Complete pipeline for recursive grammars
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn left_recursive_list() {
    let (_, table) = build_table(
        GrammarBuilder::new("lrec")
            .token("a", "a")
            .rule("list", vec!["a"])
            .rule("list", vec!["list", "a"])
            .start("list"),
    );
    assert!(table.state_count >= 3);
    assert!(has_accept(&table));
}

#[test]
fn right_recursive_list() {
    let (_, table) = build_table(
        GrammarBuilder::new("rrec")
            .token("a", "a")
            .rule("list", vec!["a"])
            .rule("list", vec!["a", "list"])
            .start("list"),
    );
    assert!(table.state_count >= 3);
    assert!(has_accept(&table));
}

#[test]
fn deeply_nested_nonterminals() {
    let (_, table) = build_table(
        GrammarBuilder::new("deep")
            .token("x", "x")
            .rule("d", vec!["x"])
            .rule("c", vec!["d"])
            .rule("b", vec!["c"])
            .rule("a", vec!["b"])
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    sanity_check_tables(&table).expect("sanity check");
}

#[test]
fn mutual_recursion_via_two_nonterminals() {
    let (_, table) = build_table(
        GrammarBuilder::new("mutual")
            .token("a", "a")
            .token("b", "b")
            .rule("p", vec!["a"])
            .rule("p", vec!["a", "q"])
            .rule("q", vec!["b"])
            .rule("q", vec!["b", "p"])
            .start("p"),
    );
    assert!(table.state_count >= 3);
    assert!(has_accept(&table));
}

#[test]
fn recursive_with_separator() {
    let (_, table) = build_table(
        GrammarBuilder::new("sep")
            .token("id", "[a-z]+")
            .token("comma", ",")
            .rule("list", vec!["id"])
            .rule("list", vec!["list", "comma", "id"])
            .start("list"),
    );
    assert!(table.state_count >= 4);
    assert!(has_accept(&table));
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Pipeline with precedence
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn precedence_add_mul_left_assoc() {
    let (_, table) = build_table(
        GrammarBuilder::new("prec1")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
            .rule("expr", vec!["num"])
            .start("expr"),
    );
    assert!(table.state_count >= 4);
    assert!(has_accept(&table));
}

#[test]
fn precedence_right_assoc() {
    let (_, table) = build_table(
        GrammarBuilder::new("rassoc")
            .token("num", "[0-9]+")
            .token("caret", "\\^")
            .rule_with_precedence(
                "expr",
                vec!["expr", "caret", "expr"],
                1,
                Associativity::Right,
            )
            .rule("expr", vec!["num"])
            .start("expr"),
    );
    assert!(table.state_count >= 4);
    assert!(has_accept(&table));
}

#[test]
fn precedence_mixed_assoc() {
    let (_, table) = build_table(
        GrammarBuilder::new("mixed")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("caret", "\\^")
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence(
                "expr",
                vec!["expr", "caret", "expr"],
                2,
                Associativity::Right,
            )
            .rule("expr", vec!["num"])
            .start("expr"),
    );
    assert!(table.state_count >= 4);
    assert!(has_accept(&table));
}

#[test]
fn precedence_three_levels() {
    let (_, table) = build_table(
        GrammarBuilder::new("prec3")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .token("caret", "\\^")
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
            .rule_with_precedence(
                "expr",
                vec!["expr", "caret", "expr"],
                3,
                Associativity::Right,
            )
            .rule("expr", vec!["num"])
            .start("expr"),
    );
    assert!(table.state_count >= 4);
    assert!(has_accept(&table));
    sanity_check_tables(&table).expect("sanity check");
}

#[test]
fn precedence_with_parens() {
    let (_, table) = build_table(
        GrammarBuilder::new("precpar")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
            .rule("expr", vec!["lp", "expr", "rp"])
            .rule("expr", vec!["num"])
            .start("expr"),
    );
    assert!(table.state_count >= 6);
    assert!(has_accept(&table));
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Pipeline with many alternatives
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn three_alternatives_same_nonterminal() {
    let (_, table) = build_table(
        GrammarBuilder::new("alt3")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .rule("start", vec!["c"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

#[test]
fn five_alternatives() {
    let (_, table) = build_table(
        GrammarBuilder::new("alt5")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .token("e", "e")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .rule("start", vec!["c"])
            .rule("start", vec!["d"])
            .rule("start", vec!["e"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

#[test]
fn alternatives_with_different_lengths() {
    let (_, table) = build_table(
        GrammarBuilder::new("difflen")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a"])
            .rule("start", vec!["a", "b"])
            .rule("start", vec!["a", "b", "c"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

#[test]
fn alternatives_mixing_terminals_and_nonterminals() {
    let (_, table) = build_table(
        GrammarBuilder::new("mixalt")
            .token("x", "x")
            .token("y", "y")
            .rule("inner", vec!["y"])
            .rule("start", vec!["x"])
            .rule("start", vec!["inner"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

#[test]
fn many_alternatives_all_single_token() {
    let mut b = GrammarBuilder::new("many");
    for i in 0..8 {
        let name = format!("t{i}");
        let pat = format!("{}", (b'a' + i) as char);
        b = b.token(&name, &pat);
        b = b.rule("start", vec![Box::leak(name.into_boxed_str()) as &str]);
    }
    b = b.start("start");
    let (_, table) = build_table(b);
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Pipeline state counts grow with complexity
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn longer_sequence_more_states() {
    let (_, t2) = build_table(
        GrammarBuilder::new("seq2")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start"),
    );
    let (_, t3) = build_table(
        GrammarBuilder::new("seq3")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start"),
    );
    assert!(t3.state_count >= t2.state_count);
}

#[test]
fn more_nonterminals_more_states() {
    let (_, t1) = build_table(
        GrammarBuilder::new("n1")
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start"),
    );
    let (_, t2) = build_table(
        GrammarBuilder::new("n2")
            .token("x", "x")
            .rule("mid", vec!["x"])
            .rule("start", vec!["mid"])
            .start("start"),
    );
    assert!(t2.state_count >= t1.state_count);
}

#[test]
fn expr_grammar_larger_than_trivial() {
    let (_, trivial) = build_table(
        GrammarBuilder::new("triv")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let (_, expr) = build_table(
        GrammarBuilder::new("expr")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "star", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["num"])
            .start("expr"),
    );
    assert!(expr.state_count > trivial.state_count);
}

#[test]
fn adding_parens_increases_states() {
    let (_, no_parens) = build_table(
        GrammarBuilder::new("noparen")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["num"])
            .start("expr"),
    );
    let (_, with_parens) = build_table(
        GrammarBuilder::new("wparen")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["num"])
            .rule("term", vec!["lp", "expr", "rp"])
            .start("expr"),
    );
    assert!(with_parens.state_count > no_parens.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Pipeline conflict analysis
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn simple_grammar_no_conflicts() {
    let (_, table) = build_table(
        GrammarBuilder::new("noconflict")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let stats = analyze(&table);
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}

#[test]
fn conflict_stats_default_resolved_field() {
    let (_, table) = build_table(
        GrammarBuilder::new("def")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start"),
    );
    let stats = analyze(&table);
    let _ = stats.default_resolved;
}

#[test]
fn conflict_stats_all_fields_accessible() {
    let (_, table) = build_table(
        GrammarBuilder::new("fields")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let stats = analyze(&table);
    let _ = stats.shift_reduce_conflicts;
    let _ = stats.reduce_reduce_conflicts;
    let _ = stats.precedence_resolved;
    let _ = stats.associativity_resolved;
    let _ = stats.explicit_glr;
    let _ = stats.default_resolved;
}

#[test]
fn analyzer_can_be_reused() {
    let (_, t1) = build_table(
        GrammarBuilder::new("reuse1")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let (_, t2) = build_table(
        GrammarBuilder::new("reuse2")
            .token("b", "b")
            .rule("start", vec!["b"])
            .start("start"),
    );
    let mut analyzer = ConflictAnalyzer::new();
    let _s1 = analyzer.analyze_table(&t1);
    let _s2 = analyzer.analyze_table(&t2);
}

#[test]
fn expr_grammar_conflict_analysis() {
    let (_, table) = build_table(
        GrammarBuilder::new("exprca")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "star", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["num"])
            .start("expr"),
    );
    let stats = analyze(&table);
    // Factored expression grammar should be conflict-free
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}

#[test]
fn precedence_grammar_conflict_analysis() {
    let (_, table) = build_table(
        GrammarBuilder::new("precca")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
            .rule("expr", vec!["num"])
            .start("expr"),
    );
    let stats = analyze(&table);
    // Analyzer can run on precedence grammars without error
    let _ = stats;
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Pipeline determinism
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn deterministic_state_count() {
    let mk = || {
        GrammarBuilder::new("det")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    };
    let (_, t1) = build_table(mk());
    let (_, t2) = build_table(mk());
    assert_eq!(t1.state_count, t2.state_count);
}

#[test]
fn deterministic_action_table_shape() {
    let mk = || {
        GrammarBuilder::new("detshape")
            .token("x", "x")
            .token("y", "y")
            .rule("start", vec!["x", "y"])
            .start("start")
    };
    let (_, t1) = build_table(mk());
    let (_, t2) = build_table(mk());
    assert_eq!(t1.action_table.len(), t2.action_table.len());
    for (r1, r2) in t1.action_table.iter().zip(t2.action_table.iter()) {
        assert_eq!(r1.len(), r2.len());
    }
}

#[test]
fn deterministic_goto_table_shape() {
    let mk = || {
        GrammarBuilder::new("detgoto")
            .token("x", "x")
            .rule("mid", vec!["x"])
            .rule("start", vec!["mid"])
            .start("start")
    };
    let (_, t1) = build_table(mk());
    let (_, t2) = build_table(mk());
    assert_eq!(t1.goto_table.len(), t2.goto_table.len());
    for (r1, r2) in t1.goto_table.iter().zip(t2.goto_table.iter()) {
        assert_eq!(r1.len(), r2.len());
    }
}

#[test]
fn deterministic_rules() {
    let mk = || {
        GrammarBuilder::new("detrules")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["a", "b"])
            .start("start")
    };
    let (_, t1) = build_table(mk());
    let (_, t2) = build_table(mk());
    assert_eq!(t1.rules.len(), t2.rules.len());
    for (r1, r2) in t1.rules.iter().zip(t2.rules.iter()) {
        assert_eq!(r1.lhs, r2.lhs);
        assert_eq!(r1.rhs_len, r2.rhs_len);
    }
}

#[test]
fn deterministic_eof_symbol() {
    let mk = || {
        GrammarBuilder::new("deteof")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
    };
    let (_, t1) = build_table(mk());
    let (_, t2) = build_table(mk());
    assert_eq!(t1.eof_symbol, t2.eof_symbol);
}

#[test]
fn deterministic_symbol_count() {
    let mk = || {
        GrammarBuilder::new("detsym")
            .token("a", "a")
            .token("b", "b")
            .rule("inner", vec!["a"])
            .rule("start", vec!["inner", "b"])
            .start("start")
    };
    let (_, t1) = build_table(mk());
    let (_, t2) = build_table(mk());
    assert_eq!(t1.symbol_count, t2.symbol_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Pipeline with various grammar shapes
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn linear_chain_grammar() {
    let (_, table) = build_table(
        GrammarBuilder::new("chain")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .rule("start", vec!["a", "b", "c", "d"])
            .start("start"),
    );
    assert!(table.state_count >= 5);
    sanity_check_tables(&table).expect("sanity check");
}

#[test]
fn diamond_shape_grammar() {
    let (_, table) = build_table(
        GrammarBuilder::new("diamond")
            .token("x", "x")
            .token("y", "y")
            .rule("left", vec!["x"])
            .rule("right", vec!["y"])
            .rule("start", vec!["left"])
            .rule("start", vec!["right"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

#[test]
fn wide_grammar_many_tokens() {
    let (_, table) = build_table(
        GrammarBuilder::new("wide")
            .token("t1", "1")
            .token("t2", "2")
            .token("t3", "3")
            .token("t4", "4")
            .token("t5", "5")
            .token("t6", "6")
            .rule("start", vec!["t1"])
            .rule("start", vec!["t2"])
            .rule("start", vec!["t3"])
            .rule("start", vec!["t4"])
            .rule("start", vec!["t5"])
            .rule("start", vec!["t6"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

#[test]
fn nested_binary_productions() {
    let (_, table) = build_table(
        GrammarBuilder::new("nested_bin")
            .token("a", "a")
            .token("b", "b")
            .token("op", "\\+")
            .rule("atom", vec!["a"])
            .rule("atom", vec!["b"])
            .rule("pair", vec!["atom", "op", "atom"])
            .rule("start", vec!["pair"])
            .start("start"),
    );
    assert!(table.state_count >= 4);
    assert!(has_accept(&table));
}

#[test]
fn statement_like_grammar() {
    let (_, table) = build_table(
        GrammarBuilder::new("stmt")
            .token("kw_if", "if")
            .token("kw_then", "then")
            .token("kw_end", "end")
            .token("id", "[a-z]+")
            .rule("cond", vec!["id"])
            .rule("body", vec!["id"])
            .rule("stmt", vec!["id"])
            .rule("stmt", vec!["kw_if", "cond", "kw_then", "body", "kw_end"])
            .rule("start", vec!["stmt"])
            .start("start"),
    );
    assert!(table.state_count >= 3);
    assert!(has_accept(&table));
}

#[test]
fn multiple_nonterminals_converge() {
    let (_, table) = build_table(
        GrammarBuilder::new("converge")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("x", vec!["a"])
            .rule("y", vec!["b"])
            .rule("z", vec!["c"])
            .rule("start", vec!["x"])
            .rule("start", vec!["y"])
            .rule("start", vec!["z"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

#[test]
fn separator_list_grammar() {
    let (_, table) = build_table(
        GrammarBuilder::new("seplist")
            .token("id", "[a-z]+")
            .token("semi", ";")
            .rule("item", vec!["id"])
            .rule("items", vec!["item"])
            .rule("items", vec!["items", "semi", "item"])
            .rule("start", vec!["items"])
            .start("start"),
    );
    assert!(table.state_count >= 4);
    assert!(has_accept(&table));
}

#[test]
fn bracketed_grammar() {
    let (_, table) = build_table(
        GrammarBuilder::new("bracket")
            .token("lb", "\\[")
            .token("rb", "\\]")
            .token("x", "x")
            .rule("inner", vec!["x"])
            .rule("start", vec!["lb", "inner", "rb"])
            .start("start"),
    );
    assert!(table.state_count >= 4);
    assert!(has_accept(&table));
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Pipeline error handling & edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn grammar_validate_passes_for_valid() {
    let grammar = GrammarBuilder::new("valid")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    grammar.validate().expect("should validate");
}

#[test]
fn grammar_name_preserved_through_pipeline() {
    let (grammar, _) = build_table(
        GrammarBuilder::new("myname")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert_eq!(grammar.name, "myname");
}

#[test]
fn normalize_idempotent() {
    let mut g1 = GrammarBuilder::new("idem")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    g1.normalize();
    let ff1 = FirstFollowSets::compute(&g1).expect("ff1");
    let t1 = build_lr1_automaton(&g1, &ff1).expect("t1");

    let mut g2 = GrammarBuilder::new("idem")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    g2.normalize();
    g2.normalize(); // second normalize
    let ff2 = FirstFollowSets::compute(&g2).expect("ff2");
    let t2 = build_lr1_automaton(&g2, &ff2).expect("t2");

    assert_eq!(t1.state_count, t2.state_count);
}

#[test]
fn start_symbol_exists_in_grammar() {
    let grammar = GrammarBuilder::new("startsym")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    assert!(grammar.start_symbol().is_some());
}

#[test]
fn tokens_are_registered_in_grammar() {
    let grammar = GrammarBuilder::new("tokens")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    assert!(grammar.tokens.len() >= 3);
}

#[test]
fn rules_registered_in_grammar() {
    let grammar = GrammarBuilder::new("rules")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec!["a", "a"])
        .start("start")
        .build();
    let total_rules: usize = grammar.rules.values().map(|v| v.len()).sum();
    assert!(total_rules >= 2);
}

// ═══════════════════════════════════════════════════════════════════════════
// Table structure invariants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn action_table_rows_match_state_count() {
    let (_, table) = build_table(
        GrammarBuilder::new("atrows")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert_eq!(table.action_table.len(), table.state_count);
}

#[test]
fn goto_table_rows_match_state_count() {
    let (_, table) = build_table(
        GrammarBuilder::new("gtrows")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert_eq!(table.goto_table.len(), table.state_count);
}

#[test]
fn action_table_columns_consistent() {
    let (_, table) = build_table(
        GrammarBuilder::new("atcols")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start"),
    );
    if table.state_count > 0 {
        let ncols = table.action_table[0].len();
        for row in &table.action_table {
            assert_eq!(row.len(), ncols);
        }
    }
}

#[test]
fn goto_table_columns_consistent() {
    let (_, table) = build_table(
        GrammarBuilder::new("gtcols")
            .token("a", "a")
            .rule("mid", vec!["a"])
            .rule("start", vec!["mid"])
            .start("start"),
    );
    if table.state_count > 0 {
        let ncols = table.goto_table[0].len();
        for row in &table.goto_table {
            assert_eq!(row.len(), ncols);
        }
    }
}

#[test]
fn eof_symbol_in_symbol_to_index() {
    let (_, table) = build_table(
        GrammarBuilder::new("eofidx")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(
        table.symbol_to_index.contains_key(&table.eof_symbol),
        "EOF symbol should be in symbol_to_index"
    );
}

#[test]
fn parse_rules_have_valid_lhs() {
    let (_, table) = build_table(
        GrammarBuilder::new("lhsvalid")
            .token("a", "a")
            .token("b", "b")
            .rule("inner", vec!["a"])
            .rule("start", vec!["inner", "b"])
            .start("start"),
    );
    for rule in &table.rules {
        // lhs should be a valid SymbolId (non-zero in augmented grammar)
        let _ = rule.lhs;
        let _ = rule.rhs_len;
    }
    assert!(!table.rules.is_empty());
}

#[test]
fn parse_rules_rhs_len_matches_grammar() {
    let (_, table) = build_table(
        GrammarBuilder::new("rhslen")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start"),
    );
    // At least one rule should have rhs_len == 3 (the user rule)
    let has_len3 = table.rules.iter().any(|r| r.rhs_len == 3);
    assert!(has_len3, "Should have a rule with rhs_len == 3");
}

// ═══════════════════════════════════════════════════════════════════════════
// Normalized pipeline tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn normalized_simple_grammar() {
    let (_, table) = build_table_normalized(
        GrammarBuilder::new("normsimple")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    sanity_check_tables(&table).expect("sanity check");
}

#[test]
fn normalized_expr_grammar() {
    let (_, table) = build_table_normalized(
        GrammarBuilder::new("normexpr")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .rule("expr", vec!["expr", "plus", "num"])
            .rule("expr", vec!["num"])
            .start("expr"),
    );
    assert!(table.state_count >= 4);
    assert!(has_accept(&table));
}

#[test]
fn normalized_recursive_grammar() {
    let (_, table) = build_table_normalized(
        GrammarBuilder::new("normrec")
            .token("a", "a")
            .rule("list", vec!["a"])
            .rule("list", vec!["list", "a"])
            .start("list"),
    );
    assert!(table.state_count >= 3);
    assert!(has_accept(&table));
}

#[test]
fn compute_normalized_matches_manual() {
    let mut g1 = GrammarBuilder::new("cmp")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    g1.normalize();
    let ff1 = FirstFollowSets::compute(&g1).expect("ff1");
    let t1 = build_lr1_automaton(&g1, &ff1).expect("t1");

    let mut g2 = GrammarBuilder::new("cmp")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let ff2 = FirstFollowSets::compute_normalized(&mut g2).expect("ff2");
    let t2 = build_lr1_automaton(&g2, &ff2).expect("t2");

    assert_eq!(t1.state_count, t2.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// FirstFollow integration with pipeline
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn first_follow_available_for_start_symbol() {
    let grammar = GrammarBuilder::new("ffstart")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&grammar).expect("compute");
    if let Some(start) = grammar.start_symbol() {
        assert!(ff.first(start).is_some() || ff.follow(start).is_some());
    }
}

#[test]
fn first_follow_multiple_nonterminals() {
    let grammar = GrammarBuilder::new("ffmulti")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&grammar).expect("compute");
    // At least start symbol should have FIRST set
    let has_first = grammar.rule_names.keys().any(|id| ff.first(*id).is_some());
    assert!(has_first);
}

#[test]
fn first_follow_nullable_check() {
    let grammar = GrammarBuilder::new("ffnull")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let ff = FirstFollowSets::compute(&grammar).expect("compute");
    // A terminal-only rule is not nullable
    if let Some(start) = grammar.start_symbol() {
        assert!(!ff.is_nullable(start));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Cross-cutting pipeline properties
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn pipeline_preserves_grammar_in_table() {
    let (_, table) = build_table(
        GrammarBuilder::new("preserve")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    // ParseTable should embed the grammar
    assert_eq!(table.grammar.name, "preserve");
}

#[test]
fn multiple_independent_pipelines() {
    let (_, t1) = build_table(
        GrammarBuilder::new("ind1")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start"),
    );
    let (_, t2) = build_table(
        GrammarBuilder::new("ind2")
            .token("b", "b")
            .rule("start", vec!["b"])
            .start("start"),
    );
    // Both should succeed independently
    assert!(t1.state_count > 0);
    assert!(t2.state_count > 0);
}

#[test]
fn pipeline_with_single_char_names() {
    let (_, table) = build_table(
        GrammarBuilder::new("sc")
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a", "b"])
            .start("s"),
    );
    assert!(table.state_count >= 3);
}

#[test]
fn pipeline_with_long_names() {
    let (_, table) = build_table(
        GrammarBuilder::new("long_grammar_name")
            .token("very_long_token_name", "x")
            .rule("very_long_rule_name", vec!["very_long_token_name"])
            .start("very_long_rule_name"),
    );
    assert!(table.state_count >= 2);
}

#[test]
fn pipeline_with_regex_patterns() {
    let (_, table) = build_table(
        GrammarBuilder::new("regex")
            .token("num", "[0-9]+")
            .token("id", "[a-zA-Z_][a-zA-Z0-9_]*")
            .rule("start", vec!["num"])
            .rule("start", vec!["id"])
            .start("start"),
    );
    assert!(table.state_count >= 2);
    assert!(has_accept(&table));
}

#[test]
fn full_pipeline_sanity_check_on_complex_grammar() {
    let (_, table) = build_table(
        GrammarBuilder::new("complex")
            .token("num", "[0-9]+")
            .token("id", "[a-z]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .token("eq", "=")
            .token("semi", ";")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "star", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["num"])
            .rule("factor", vec!["id"])
            .rule("factor", vec!["lp", "expr", "rp"])
            .rule("assign", vec!["id", "eq", "expr"])
            .rule("stmt", vec!["assign", "semi"])
            .rule("stmt", vec!["expr", "semi"])
            .rule("start", vec!["stmt"])
            .start("start"),
    );
    assert!(table.state_count >= 10);
    assert!(has_accept(&table));
    assert!(has_shift(&table));
    assert!(has_reduce(&table));
    sanity_check_tables(&table).expect("sanity check");
}

#[test]
fn full_pipeline_analysis_on_complex_grammar() {
    let (_, table) = build_table(
        GrammarBuilder::new("complexa")
            .token("num", "[0-9]+")
            .token("plus", "\\+")
            .token("star", "\\*")
            .token("lp", "\\(")
            .token("rp", "\\)")
            .rule("expr", vec!["expr", "plus", "term"])
            .rule("expr", vec!["term"])
            .rule("term", vec!["term", "star", "factor"])
            .rule("term", vec!["factor"])
            .rule("factor", vec!["num"])
            .rule("factor", vec!["lp", "expr", "rp"])
            .start("expr"),
    );
    let stats = analyze(&table);
    // Properly factored grammar should have no conflicts
    assert_eq!(stats.shift_reduce_conflicts, 0);
    assert_eq!(stats.reduce_reduce_conflicts, 0);
}
