//! Determinism verification tests for GLR core.
//!
//! These tests ensure that the full pipeline (Grammar → FIRST/FOLLOW → canonical
//! collection → parse table) produces bit-for-bit identical output on every run.
//! This is critical for reproducible builds and testing.

#![cfg(feature = "test-api")]

use adze_glr_core::{
    Action, FirstFollowSets, ItemSetCollection, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a normalized grammar and compute FIRST/FOLLOW sets.
fn compute_first_follow(grammar: &mut Grammar) -> FirstFollowSets {
    FirstFollowSets::compute_normalized(grammar).expect("FIRST/FOLLOW computation should succeed")
}

/// Run the full pipeline and return the parse table.
fn run_pipeline(grammar: &mut Grammar) -> adze_glr_core::ParseTable {
    let ff = compute_first_follow(grammar);
    build_lr1_automaton(grammar, &ff).expect("build_lr1_automaton should succeed")
}

/// Build the simple arithmetic grammar used by most tests.
fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arithmetic")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "NUM"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Build a multi-nonterminal expression grammar.
fn expression_grammar() -> Grammar {
    GrammarBuilder::new("expression")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("(", "(")
        .token(")", ")")
        .rule("expr", vec!["expr", "+", "term"])
        .rule("expr", vec!["term"])
        .rule("term", vec!["term", "*", "factor"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["(", "expr", ")"])
        .rule("factor", vec!["NUM"])
        .start("expr")
        .build()
}

/// Build an inherently ambiguous grammar (E → E+E | E*E | NUM).
fn ambiguous_grammar() -> Grammar {
    GrammarBuilder::new("ambiguous")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("E", vec!["E", "+", "E"])
        .rule("E", vec!["E", "*", "E"])
        .rule("E", vec!["NUM"])
        .start("E")
        .build()
}

/// Snapshot the action table contents as a deterministic string.
fn snapshot_action_table(table: &adze_glr_core::ParseTable) -> String {
    let mut out = String::new();
    for (si, row) in table.action_table.iter().enumerate() {
        for (ci, cell) in row.iter().enumerate() {
            if !cell.is_empty() {
                out.push_str(&format!("s{}c{}:", si, ci));
                for a in cell {
                    out.push_str(&format!("{:?},", a));
                }
                out.push('\n');
            }
        }
    }
    out
}

/// Snapshot the goto table contents as a deterministic string.
fn snapshot_goto_table(table: &adze_glr_core::ParseTable) -> String {
    let mut out = String::new();
    for (si, row) in table.goto_table.iter().enumerate() {
        for (ci, &state) in row.iter().enumerate() {
            if state.0 != 0 && state.0 != u16::MAX {
                out.push_str(&format!("s{}c{}:{}\n", si, ci, state.0));
            }
        }
    }
    out
}

/// Snapshot the full parse table (actions + gotos + metadata).
fn snapshot_parse_table(table: &adze_glr_core::ParseTable) -> String {
    let mut out = String::new();
    out.push_str(&format!("states:{}\n", table.state_count));
    out.push_str(&format!("symbols:{}\n", table.symbol_count));
    out.push_str(&format!("rules:{}\n", table.rules.len()));
    out.push_str(&format!("eof:{}\n", table.eof_symbol.0));
    out.push_str(&format!("start:{}\n", table.start_symbol.0));
    // symbol_to_index (BTreeMap → deterministic iteration)
    for (sym, idx) in &table.symbol_to_index {
        out.push_str(&format!("sym2idx:{}={}\n", sym.0, idx));
    }
    // nonterminal_to_index
    for (sym, idx) in &table.nonterminal_to_index {
        out.push_str(&format!("nt2idx:{}={}\n", sym.0, idx));
    }
    out.push_str(&snapshot_action_table(table));
    out.push_str(&snapshot_goto_table(table));
    // rules
    for (i, r) in table.rules.iter().enumerate() {
        out.push_str(&format!("rule{}:lhs={},len={}\n", i, r.lhs.0, r.rhs_len));
    }
    out
}

/// Snapshot FIRST/FOLLOW sets for a set of symbol IDs.
fn snapshot_first_follow(ff: &FirstFollowSets, symbols: &[SymbolId]) -> String {
    let mut out = String::new();
    for &sym in symbols {
        if let Some(first) = ff.first(sym) {
            let bits: Vec<usize> = first.ones().collect();
            out.push_str(&format!("FIRST({})={:?}\n", sym.0, bits));
        }
        if let Some(follow) = ff.follow(sym) {
            let bits: Vec<usize> = follow.ones().collect();
            out.push_str(&format!("FOLLOW({})={:?}\n", sym.0, bits));
        }
        out.push_str(&format!("NULLABLE({})={}\n", sym.0, ff.is_nullable(sym)));
    }
    out
}

/// Snapshot an ItemSetCollection.
fn snapshot_collection(col: &ItemSetCollection) -> String {
    let mut out = String::new();
    out.push_str(&format!("item_sets:{}\n", col.sets.len()));
    for (i, set) in col.sets.iter().enumerate() {
        let mut items: Vec<String> = set
            .items
            .iter()
            .map(|item| {
                format!(
                    "r{}p{}la{}",
                    item.rule_id.0, item.position, item.lookahead.0
                )
            })
            .collect();
        items.sort();
        out.push_str(&format!("set{}:[{}]\n", i, items.join(",")));
    }
    // goto_table entries (IndexMap order is insertion-order-deterministic)
    let mut goto_entries: Vec<String> = col
        .goto_table
        .iter()
        .map(|((from, sym), to)| format!("goto({},{})={}", from.0, sym.0, to.0))
        .collect();
    goto_entries.sort();
    for entry in &goto_entries {
        out.push_str(entry);
        out.push('\n');
    }
    out
}

/// Collect all symbol IDs that are nonterminals in the grammar.
fn nonterminal_ids(grammar: &Grammar) -> Vec<SymbolId> {
    let mut ids: Vec<SymbolId> = grammar.rules.keys().copied().collect();
    ids.sort_by_key(|s| s.0);
    ids
}

// ---------------------------------------------------------------------------
// 1. Same grammar always produces the same parse table
// ---------------------------------------------------------------------------

#[test]
fn parse_table_is_deterministic() {
    let mut g1 = arithmetic_grammar();
    let mut g2 = arithmetic_grammar();
    let snap1 = snapshot_parse_table(&run_pipeline(&mut g1));
    let snap2 = snapshot_parse_table(&run_pipeline(&mut g2));
    assert_eq!(snap1, snap2, "Parse tables differ across runs");
}

#[test]
fn parse_table_deterministic_expression_grammar() {
    let mut g1 = expression_grammar();
    let mut g2 = expression_grammar();
    let snap1 = snapshot_parse_table(&run_pipeline(&mut g1));
    let snap2 = snapshot_parse_table(&run_pipeline(&mut g2));
    assert_eq!(snap1, snap2, "Expression grammar tables differ across runs");
}

#[test]
fn parse_table_deterministic_ambiguous_grammar() {
    let mut g1 = ambiguous_grammar();
    let mut g2 = ambiguous_grammar();
    let snap1 = snapshot_parse_table(&run_pipeline(&mut g1));
    let snap2 = snapshot_parse_table(&run_pipeline(&mut g2));
    assert_eq!(snap1, snap2, "Ambiguous grammar tables differ across runs");
}

// ---------------------------------------------------------------------------
// 2. Same input always produces the same parse result
// ---------------------------------------------------------------------------

#[test]
fn parse_result_is_deterministic() {
    let mut g = arithmetic_grammar();
    let table = run_pipeline(&mut g);

    // Tokenize "NUM + NUM" as [(kind, start, end)]
    let tokens: Vec<(u32, u32, u32)> = make_arithmetic_tokens(&table);

    let mut results = Vec::new();
    for _ in 0..10 {
        let mut driver = adze_glr_core::Driver::new(&table);
        let forest = driver
            .parse_tokens(tokens.clone())
            .expect("parse should succeed");
        let view = forest.view();
        results.push(snapshot_forest(view));
    }

    let first = &results[0];
    for (i, snap) in results.iter().enumerate().skip(1) {
        assert_eq!(first, snap, "Parse result differs at iteration {}", i);
    }
}

/// Build token triples for "NUM + NUM" from the parse table's symbol mapping.
fn make_arithmetic_tokens(table: &adze_glr_core::ParseTable) -> Vec<(u32, u32, u32)> {
    // Find symbol IDs for NUM and +
    let num_id = table
        .grammar()
        .tokens
        .iter()
        .find(|(_, t)| t.name == "NUM")
        .map(|(id, _)| id.0 as u32)
        .expect("NUM token not found");
    let plus_id = table
        .grammar()
        .tokens
        .iter()
        .find(|(_, t)| t.name == "+")
        .map(|(id, _)| id.0 as u32)
        .expect("+ token not found");

    vec![
        (num_id, 0, 1),  // "1"
        (plus_id, 1, 2), // "+"
        (num_id, 2, 3),  // "2"
    ]
}

/// Snapshot a ForestView into a deterministic string.
fn snapshot_forest(view: &dyn adze_glr_core::forest_view::ForestView) -> String {
    let mut out = String::new();
    out.push_str(&format!("roots:{:?}\n", view.roots()));
    for &root in view.roots() {
        snapshot_node(view, root, 0, &mut out);
    }
    out
}

fn snapshot_node(
    view: &dyn adze_glr_core::forest_view::ForestView,
    id: u32,
    depth: usize,
    out: &mut String,
) {
    let indent = " ".repeat(depth * 2);
    let span = view.span(id);
    out.push_str(&format!(
        "{}node(id={},kind={},span={}..{})\n",
        indent,
        id,
        view.kind(id),
        span.start,
        span.end
    ));
    for &child in view.best_children(id) {
        snapshot_node(view, child, depth + 1, out);
    }
}

// ---------------------------------------------------------------------------
// 3. FIRST/FOLLOW sets are deterministic
// ---------------------------------------------------------------------------

#[test]
fn first_follow_sets_are_deterministic() {
    let mut g1 = expression_grammar();
    let mut g2 = expression_grammar();
    let ff1 = compute_first_follow(&mut g1);
    let ff2 = compute_first_follow(&mut g2);

    let nts1 = nonterminal_ids(&g1);
    let nts2 = nonterminal_ids(&g2);
    assert_eq!(nts1, nts2);

    let snap1 = snapshot_first_follow(&ff1, &nts1);
    let snap2 = snapshot_first_follow(&ff2, &nts2);
    assert_eq!(snap1, snap2, "FIRST/FOLLOW sets differ across runs");
}

#[test]
fn first_follow_deterministic_ambiguous() {
    let mut g1 = ambiguous_grammar();
    let mut g2 = ambiguous_grammar();
    let ff1 = compute_first_follow(&mut g1);
    let ff2 = compute_first_follow(&mut g2);

    let nts1 = nonterminal_ids(&g1);
    let snap1 = snapshot_first_follow(&ff1, &nts1);
    let snap2 = snapshot_first_follow(&ff2, &nts1);
    assert_eq!(snap1, snap2, "Ambiguous grammar FIRST/FOLLOW differs");
}

// ---------------------------------------------------------------------------
// 4. Canonical collection order is deterministic
// ---------------------------------------------------------------------------

#[test]
fn canonical_collection_is_deterministic() {
    let mut g1 = arithmetic_grammar();
    let mut g2 = arithmetic_grammar();
    let ff1 = compute_first_follow(&mut g1);
    let ff2 = compute_first_follow(&mut g2);

    let start1 = g1.start_symbol().unwrap();
    let start2 = g2.start_symbol().unwrap();

    // Augment grammars identically to build_lr1_automaton
    let max1 = g1
        .tokens
        .keys()
        .chain(g1.rules.keys())
        .map(|s| s.0)
        .max()
        .unwrap_or(0);
    let eof1 = SymbolId(max1 + 1);
    let aug1 = SymbolId(max1 + 2);
    add_augmented_start(&mut g1, aug1, start1);

    let max2 = g2
        .tokens
        .keys()
        .chain(g2.rules.keys())
        .map(|s| s.0)
        .max()
        .unwrap_or(0);
    let eof2 = SymbolId(max2 + 1);
    let aug2 = SymbolId(max2 + 2);
    add_augmented_start(&mut g2, aug2, start2);

    let col1 =
        ItemSetCollection::build_canonical_collection_augmented(&g1, &ff1, aug1, start1, eof1);
    let col2 =
        ItemSetCollection::build_canonical_collection_augmented(&g2, &ff2, aug2, start2, eof2);

    let snap1 = snapshot_collection(&col1);
    let snap2 = snapshot_collection(&col2);
    assert_eq!(snap1, snap2, "Canonical collections differ across runs");
}

fn add_augmented_start(grammar: &mut Grammar, aug_start: SymbolId, original_start: SymbolId) {
    use adze_ir::{ProductionId, Rule, Symbol};
    let max_prod = grammar
        .all_rules()
        .map(|r| r.production_id.0)
        .max()
        .unwrap_or(0);
    grammar.rules.insert(
        aug_start,
        vec![Rule {
            lhs: aug_start,
            rhs: vec![Symbol::NonTerminal(original_start)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(max_prod + 1),
        }],
    );
    grammar.rule_names.insert(aug_start, "$start".to_string());
}

// ---------------------------------------------------------------------------
// 5. Table compression is deterministic (action/goto layout)
// ---------------------------------------------------------------------------

#[test]
fn table_compression_is_deterministic() {
    let mut g1 = expression_grammar();
    let mut g2 = expression_grammar();
    let t1 = run_pipeline(&mut g1);
    let t2 = run_pipeline(&mut g2);

    // Verify action tables are byte-for-byte identical
    assert_eq!(
        snapshot_action_table(&t1),
        snapshot_action_table(&t2),
        "Action tables differ"
    );
    assert_eq!(
        snapshot_goto_table(&t1),
        snapshot_goto_table(&t2),
        "Goto tables differ"
    );
    // symbol_to_index ordering
    let s2i1: Vec<_> = t1.symbol_to_index.iter().collect();
    let s2i2: Vec<_> = t2.symbol_to_index.iter().collect();
    assert_eq!(s2i1, s2i2, "symbol_to_index ordering differs");
    // index_to_symbol
    assert_eq!(
        t1.index_to_symbol, t2.index_to_symbol,
        "index_to_symbol differs"
    );
}

// ---------------------------------------------------------------------------
// 6. Serialization output is deterministic
// ---------------------------------------------------------------------------

#[cfg(feature = "serialization")]
#[test]
fn serialization_is_deterministic() {
    let mut g1 = expression_grammar();
    let mut g2 = expression_grammar();
    let t1 = run_pipeline(&mut g1);
    let t2 = run_pipeline(&mut g2);

    let bytes1 = t1.to_bytes().expect("serialization 1");
    let bytes2 = t2.to_bytes().expect("serialization 2");
    assert_eq!(bytes1, bytes2, "Serialized bytes differ across runs");
}

// Snapshot-based determinism even without the serialization feature:
// comparing the full textual snapshot is equivalent.
#[test]
fn snapshot_serialization_is_deterministic() {
    let mut g1 = expression_grammar();
    let mut g2 = expression_grammar();
    let snap1 = snapshot_parse_table(&run_pipeline(&mut g1));
    let snap2 = snapshot_parse_table(&run_pipeline(&mut g2));
    assert_eq!(snap1, snap2, "Snapshot serialization differs across runs");
}

// ---------------------------------------------------------------------------
// 7. Run 100 iterations and compare outputs
// ---------------------------------------------------------------------------

#[test]
fn hundred_iterations_parse_table() {
    let mut reference_grammar = arithmetic_grammar();
    let reference = snapshot_parse_table(&run_pipeline(&mut reference_grammar));

    for i in 1..100 {
        let mut g = arithmetic_grammar();
        let snap = snapshot_parse_table(&run_pipeline(&mut g));
        assert_eq!(reference, snap, "Parse table differs at iteration {}", i);
    }
}

#[test]
fn hundred_iterations_first_follow() {
    let mut ref_grammar = expression_grammar();
    let ref_ff = compute_first_follow(&mut ref_grammar);
    let nts = nonterminal_ids(&ref_grammar);
    let reference = snapshot_first_follow(&ref_ff, &nts);

    for i in 1..100 {
        let mut g = expression_grammar();
        let ff = compute_first_follow(&mut g);
        let snap = snapshot_first_follow(&ff, &nts);
        assert_eq!(reference, snap, "FIRST/FOLLOW differs at iteration {}", i);
    }
}

#[test]
fn hundred_iterations_canonical_collection() {
    // Build reference
    let mut ref_g = arithmetic_grammar();
    let ref_ff = compute_first_follow(&mut ref_g);
    let ref_start = ref_g.start_symbol().unwrap();
    let max_s = ref_g
        .tokens
        .keys()
        .chain(ref_g.rules.keys())
        .map(|s| s.0)
        .max()
        .unwrap_or(0);
    let eof = SymbolId(max_s + 1);
    let aug = SymbolId(max_s + 2);
    add_augmented_start(&mut ref_g, aug, ref_start);
    let ref_col = ItemSetCollection::build_canonical_collection_augmented(
        &ref_g, &ref_ff, aug, ref_start, eof,
    );
    let reference = snapshot_collection(&ref_col);

    for i in 1..100 {
        let mut g = arithmetic_grammar();
        let ff = compute_first_follow(&mut g);
        let start = g.start_symbol().unwrap();
        let max_s = g
            .tokens
            .keys()
            .chain(g.rules.keys())
            .map(|s| s.0)
            .max()
            .unwrap_or(0);
        let eof = SymbolId(max_s + 1);
        let aug = SymbolId(max_s + 2);
        add_augmented_start(&mut g, aug, start);
        let col = ItemSetCollection::build_canonical_collection_augmented(&g, &ff, aug, start, eof);
        let snap = snapshot_collection(&col);
        assert_eq!(
            reference, snap,
            "Canonical collection differs at iteration {}",
            i
        );
    }
}

#[test]
fn hundred_iterations_parse_result() {
    let mut g = arithmetic_grammar();
    let table = run_pipeline(&mut g);
    let tokens = make_arithmetic_tokens(&table);

    let mut driver = adze_glr_core::Driver::new(&table);
    let forest = driver.parse_tokens(tokens.clone()).expect("parse");
    let reference = snapshot_forest(forest.view());

    for i in 1..100 {
        let mut d = adze_glr_core::Driver::new(&table);
        let f = d.parse_tokens(tokens.clone()).expect("parse");
        let snap = snapshot_forest(f.view());
        assert_eq!(reference, snap, "Parse result differs at iteration {}", i);
    }
}

// ---------------------------------------------------------------------------
// Sanity: tables produced by deterministic pipeline pass sanity checks
// ---------------------------------------------------------------------------

#[test]
fn deterministic_tables_pass_sanity_check() {
    for _ in 0..10 {
        let mut g = expression_grammar();
        let table = run_pipeline(&mut g);
        sanity_check_tables(&table).expect("sanity check failed on deterministic table");
    }
}
