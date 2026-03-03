//! Insta snapshot tests for GLR core: FIRST/FOLLOW sets, LR(1) item sets,
//! and conflict reports.

use adze_glr_core::{
    ConflictResolver, ConflictType, FirstFollowSets, ItemSetCollection, build_lr1_automaton,
    conflict_inspection::count_conflicts,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Render FIRST set for a symbol as a sorted list of symbol IDs.
fn render_first(ff: &FirstFollowSets, grammar: &Grammar, symbol: SymbolId) -> String {
    let name = grammar
        .rule_names
        .get(&symbol)
        .or_else(|| grammar.tokens.get(&symbol).map(|t| &t.name))
        .cloned()
        .unwrap_or_else(|| format!("SymbolId({})", symbol.0));
    match ff.first(symbol) {
        Some(bs) => {
            let ids: Vec<usize> = bs.ones().collect();
            format!("FIRST({name}) = {ids:?}")
        }
        None => format!("FIRST({name}) = <not computed>"),
    }
}

/// Render FOLLOW set for a symbol as a sorted list of symbol IDs.
fn render_follow(ff: &FirstFollowSets, grammar: &Grammar, symbol: SymbolId) -> String {
    let name = grammar
        .rule_names
        .get(&symbol)
        .cloned()
        .unwrap_or_else(|| format!("SymbolId({})", symbol.0));
    match ff.follow(symbol) {
        Some(bs) => {
            let ids: Vec<usize> = bs.ones().collect();
            format!("FOLLOW({name}) = {ids:?}")
        }
        None => format!("FOLLOW({name}) = <not computed>"),
    }
}

/// Render all FIRST/FOLLOW sets for non-terminals in the grammar.
fn render_first_follow(ff: &FirstFollowSets, grammar: &Grammar) -> String {
    let mut lines = Vec::new();
    let mut nonterminals: Vec<_> = grammar.rules.keys().collect();
    nonterminals.sort_by_key(|id| id.0);
    for &nt in &nonterminals {
        lines.push(render_first(ff, grammar, *nt));
        lines.push(render_follow(ff, grammar, *nt));
        lines.push(format!("nullable({}) = {}", nt.0, ff.is_nullable(*nt)));
    }
    lines.join("\n")
}

/// Render an LR(1) item set in human-readable form.
fn render_item_set(item_set: &adze_glr_core::ItemSet, grammar: &Grammar) -> String {
    let mut lines = Vec::new();
    lines.push(format!("State {}:", item_set.id.0));
    let mut items: Vec<_> = item_set.items.iter().collect();
    items.sort();
    for item in items {
        if let Some(rule) = grammar
            .all_rules()
            .find(|r| r.production_id.0 == item.rule_id.0)
        {
            let lhs_name = grammar
                .rule_names
                .get(&rule.lhs)
                .cloned()
                .unwrap_or_else(|| format!("NT({})", rule.lhs.0));
            let mut rhs_parts = Vec::new();
            for (idx, sym) in rule.rhs.iter().enumerate() {
                if idx == item.position {
                    rhs_parts.push("•".to_string());
                }
                rhs_parts.push(format_symbol(sym, grammar));
            }
            if item.position >= rule.rhs.len() {
                rhs_parts.push("•".to_string());
            }
            let lookahead_name = grammar
                .tokens
                .get(&item.lookahead)
                .map(|t| t.name.clone())
                .unwrap_or_else(|| {
                    if item.lookahead.0 == 0 {
                        "$".to_string()
                    } else {
                        format!("#{}", item.lookahead.0)
                    }
                });
            lines.push(format!(
                "  {lhs_name} -> {} , {lookahead_name}",
                rhs_parts.join(" ")
            ));
        }
    }
    lines.join("\n")
}

fn format_symbol(sym: &Symbol, grammar: &Grammar) -> String {
    match sym {
        Symbol::Terminal(id) => grammar
            .tokens
            .get(id)
            .map(|t| format!("'{}'", t.name))
            .unwrap_or_else(|| format!("T({})", id.0)),
        Symbol::NonTerminal(id) => grammar
            .rule_names
            .get(id)
            .cloned()
            .unwrap_or_else(|| format!("NT({})", id.0)),
        Symbol::Epsilon => "ε".to_string(),
        other => format!("{other:?}"),
    }
}

/// Render conflict report.
fn render_conflicts(resolver: &ConflictResolver) -> String {
    if resolver.conflicts.is_empty() {
        return "No conflicts".to_string();
    }
    let mut lines = Vec::new();
    let mut conflicts: Vec<_> = resolver.conflicts.iter().collect();
    conflicts.sort_by_key(|c| (c.state.0, c.symbol.0));
    for c in conflicts {
        let ty = match c.conflict_type {
            ConflictType::ShiftReduce => "S/R",
            ConflictType::ReduceReduce => "R/R",
        };
        let actions: Vec<String> = c.actions.iter().map(|a| format!("{a:?}")).collect();
        lines.push(format!(
            "State {} on SymbolId({}): {} conflict — [{}]",
            c.state.0,
            c.symbol.0,
            ty,
            actions.join(", ")
        ));
    }
    lines.join("\n")
}

// ===========================================================================
// 1. FIRST set computation snapshots
// ===========================================================================

#[test]
fn first_sets_simple_expr() {
    let mut grammar = GrammarBuilder::new("simple_expr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("source_file", vec!["expr"])
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("source_file")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();

    let mut lines = Vec::new();
    for &nt in grammar.rules.keys().collect::<Vec<_>>().iter() {
        lines.push(render_first(&ff, &grammar, *nt));
    }
    lines.sort();
    insta::assert_snapshot!("first_sets_simple_expr", lines.join("\n"));
}

#[test]
fn first_sets_nullable_grammar() {
    let mut grammar = GrammarBuilder::new("nullable")
        .token("a", "a")
        .token("b", "b")
        .rule("source_file", vec!["S"])
        .rule("S", vec!["A", "b"])
        .rule("A", vec!["a"])
        .rule("A", vec![]) // A is nullable
        .start("source_file")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();

    let mut lines = Vec::new();
    for &nt in grammar.rules.keys().collect::<Vec<_>>().iter() {
        lines.push(render_first(&ff, &grammar, *nt));
        lines.push(format!("  nullable = {}", ff.is_nullable(*nt)));
    }
    lines.sort();
    insta::assert_snapshot!("first_sets_nullable", lines.join("\n"));
}

// ===========================================================================
// 2. FOLLOW set computation snapshots
// ===========================================================================

#[test]
fn follow_sets_simple_expr() {
    let mut grammar = GrammarBuilder::new("follow_expr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("source_file", vec!["expr"])
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("source_file")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();

    let mut lines = Vec::new();
    for &nt in grammar.rules.keys().collect::<Vec<_>>().iter() {
        lines.push(render_follow(&ff, &grammar, *nt));
    }
    lines.sort();
    insta::assert_snapshot!("follow_sets_simple_expr", lines.join("\n"));
}

#[test]
fn first_follow_combined_arithmetic() {
    let mut grammar = GrammarBuilder::new("arithmetic")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("source_file", vec!["expr"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUM"])
        .start("source_file")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    insta::assert_snapshot!(
        "first_follow_arithmetic",
        render_first_follow(&ff, &grammar)
    );
}

// ===========================================================================
// 3. LR(1) item set description snapshots
// ===========================================================================

#[test]
fn lr1_item_sets_tiny_grammar() {
    let mut grammar = GrammarBuilder::new("tiny")
        .token("a", "a")
        .rule("source_file", vec!["S"])
        .rule("S", vec!["a"])
        .start("source_file")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let mut lines = Vec::new();
    lines.push(format!("Total states: {}", collection.sets.len()));
    for item_set in &collection.sets {
        lines.push(render_item_set(item_set, &grammar));
    }
    insta::assert_snapshot!("lr1_items_tiny", lines.join("\n\n"));
}

#[test]
fn lr1_item_sets_simple_expr() {
    let mut grammar = GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("source_file", vec!["expr"])
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("source_file")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let mut lines = Vec::new();
    lines.push(format!("Total states: {}", collection.sets.len()));
    for item_set in &collection.sets {
        lines.push(render_item_set(item_set, &grammar));
    }
    insta::assert_snapshot!("lr1_items_simple_expr", lines.join("\n\n"));
}

// ===========================================================================
// 4. Conflict report snapshots
// ===========================================================================

#[test]
fn conflict_report_unambiguous() {
    let mut grammar = GrammarBuilder::new("unambig")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("source_file", vec!["expr"])
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("source_file")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &grammar, &ff);

    insta::assert_snapshot!("conflicts_unambiguous", render_conflicts(&resolver));
}

#[test]
fn conflict_report_ambiguous_expr() {
    let mut grammar = GrammarBuilder::new("ambig_expr")
        .token("a", "a")
        .rule("source_file", vec!["E"])
        .rule("E", vec!["a"])
        .rule("E", vec!["E", "E"]) // inherently ambiguous
        .start("source_file")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let resolver = ConflictResolver::detect_conflicts(&collection, &grammar, &ff);

    insta::assert_snapshot!("conflicts_ambiguous_expr", render_conflicts(&resolver));
}

#[test]
fn conflict_report_via_parse_table() {
    let mut grammar = GrammarBuilder::new("ambig_table")
        .token("a", "a")
        .rule("source_file", vec!["E"])
        .rule("E", vec!["a"])
        .rule("E", vec!["E", "E"])
        .start("source_file")
        .build();

    let ff = FirstFollowSets::compute_normalized(&mut grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).expect("should build table");
    let summary = count_conflicts(&table);

    insta::assert_snapshot!(
        "conflicts_table_summary",
        format!(
            "states={} shift_reduce={} reduce_reduce={}",
            table.state_count, summary.shift_reduce, summary.reduce_reduce
        )
    );
}
